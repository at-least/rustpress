//! Syntax highlighting: the `gdcode` codefence renderer driving
//! tree-sitter, plus the dual-theme `syntax.css` generator over Helix
//! TOML themes.
//!
//! Every fenced block is rewritten by the preprocessor to
//! ```` ```gdcode lang=js label=npm hl=1,3-4 ````, so this renderer sees
//! the whole site's code through one path: tree-sitter highlighting with
//! `tk-`-prefixed capture classes, per-line `<span class="line">`
//! wrappers (` hl` on highlighted lines, plus `[!code …]` notation
//! classes), a server-side `<span class="lang">` label, and
//! `data-lang`/`data-name` attributes for the client-side copy button
//! and code-group tabs. Languages without a tree-sitter grammar fall
//! back to escaped plain lines.

use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

use comrak::adapters::CodefenceRendererAdapter;
use comrak::nodes::Sourcepos;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use super::syntax_theme::{SyntaxTheme, ThemeStyle};

/// The sentinel language token all code fences are rewritten to.
pub const FENCE_LANG: &str = "gdcode";

/// The tree-sitter capture names we style. `HighlightConfiguration` is
/// configured with exactly this list, so capture indices always map back
/// into it; `syntax.css` carries one `.tk-*` rule per name (the theme's
/// longest-prefix resolution decides which names actually get rules).
pub const CAPTURE_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "comment.documentation",
    "constant",
    "constant.builtin",
    "constant.builtin.boolean",
    "constructor",
    "function",
    "function.builtin",
    "function.macro",
    "function.method",
    "keyword",
    "keyword.control.conditional",
    "keyword.control.import",
    "keyword.control.repeat",
    "keyword.control.return",
    "keyword.function",
    "keyword.operator",
    "keyword.storage",
    "keyword.storage.type",
    "label",
    "namespace",
    "operator",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.escape",
    "string.special",
    "tag",
    "tag.attribute",
    "type",
    "type.builtin",
    "type.definition",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
    "markup.heading",
    "markup.bold",
    "markup.italic",
    "markup.list",
    "markup.quote",
    "markup.link.url",
    "markup.raw.inline",
    "diff.plus",
    "diff.minus",
    // legacy capture names still used by the markdown grammars
    "text.title",
    "text.literal",
    "text.uri",
    "text.emphasis",
    "text.strong",
    "text.quote",
    "text.underline",
    "text.strike",
    "text.reference",
];

/// CSS class for a capture name (dots become dashes). The markdown
/// grammars emit the legacy `text.*` capture set; they are normalized
/// onto the modern scopes `syntax.css` already carries.
pub fn tk_class(capture: &str) -> String {
    let capture = match capture {
        "text.title" => "markup.heading",
        "text.literal" => "markup.raw.inline",
        "text.uri" | "text.reference" => "markup.link.url",
        "text.emphasis" | "text.underline" | "text.strike" => "markup.italic",
        "text.strong" => "markup.bold",
        "text.quote" => "markup.quote",
        other => other,
    };
    format!("tk-{}", capture.replace('.', "-"))
}

/// Per-fence line-number setting (`ln=true` / `ln=false` / `ln=5`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineNumbers {
    On,
    Off,
    From(usize),
}

/// The parsed fence metadata carried in the rewritten info string.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FenceSpec {
    pub lang: String,
    pub label: Option<String>,
    /// The fence sits inside a `::: code-group`: its label names a tab,
    /// so it must not become a standalone block title.
    pub group: bool,
    /// 1-based highlighted line numbers.
    pub hl: Vec<usize>,
    pub ln: Option<LineNumbers>,
}

impl FenceSpec {
    /// Parse `lang=js hl=1,3-4 ln=true label=npm` (label last — it may
    /// contain spaces).
    pub fn parse_meta(meta: &str) -> FenceSpec {
        let mut spec = FenceSpec::default();
        let rest = if let Some(idx) = meta.find(" label=") {
            spec.label = Some(meta[idx + " label=".len()..].to_string());
            &meta[..idx]
        } else {
            meta
        };
        for token in rest.split_whitespace() {
            if let Some(v) = token.strip_prefix("lang=") {
                spec.lang = v.to_string();
            } else if token == "group=1" {
                spec.group = true;
            } else if let Some(v) = token.strip_prefix("hl=") {
                spec.hl = parse_line_spec(v);
            } else if let Some(v) = token.strip_prefix("ln=") {
                spec.ln = Some(match v {
                    "true" => LineNumbers::On,
                    "false" => LineNumbers::Off,
                    n => LineNumbers::From(n.parse().unwrap_or(1)),
                });
            }
        }
        // an empty label is no label
        if spec.label.as_deref().is_some_and(|l| l.is_empty()) {
            spec.label = None;
        }
        spec
    }
}

/// `"1,3-4"` → `[1, 3, 4]`.
fn parse_line_spec(spec: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((a, b)) = part.split_once('-') {
            if let (Ok(a), Ok(b)) = (a.trim().parse::<usize>(), b.trim().parse::<usize>()) {
                lines.extend(a..=b);
            }
        } else if let Ok(n) = part.parse::<usize>() {
            lines.push(n);
        }
    }
    lines
}

/// Per-site renderer options (from `[markdown]` in the site config).
#[derive(Debug, Clone, Copy)]
pub struct RendererOptions {
    /// Hover copy button on code blocks.
    pub copy_button: bool,
    /// Number lines by default (per-fence `ln=` overrides).
    pub line_numbers: bool,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self { copy_button: true, line_numbers: false }
    }
}

/// Renders every `gdcode` fence.
#[derive(Debug, Clone, Copy, Default)]
pub struct GdCodeRenderer {
    pub options: RendererOptions,
}

impl GdCodeRenderer {
    /// The plugins map comrak dispatches code fences through.
    pub fn plugins_map(&self) -> HashMap<String, &dyn CodefenceRendererAdapter> {
        let mut map: HashMap<String, &dyn CodefenceRendererAdapter> = HashMap::new();
        map.insert(FENCE_LANG.to_string(), self);
        map
    }
}

/// The language registry: one entry per supported grammar, with the
/// fence tokens (canonical + aliases) that select it.
struct LanguageDef {
    tokens: &'static [&'static str],
    highlights: &'static [&'static str],
    injections: &'static str,
    language: fn() -> tree_sitter::Language,
}

const LANGUAGES: &[LanguageDef] = &[
    LanguageDef {
        tokens: &["bash", "sh", "shell", "zsh"],
        highlights: &[tree_sitter_bash::HIGHLIGHT_QUERY],
        injections: "",
        language: || tree_sitter_bash::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["javascript", "js", "jsx", "mjs", "cjs"],
        highlights: &[
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
        ],
        injections: tree_sitter_javascript::INJECTIONS_QUERY,
        language: || tree_sitter_javascript::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["typescript", "ts"],
        highlights: &[tree_sitter_typescript::HIGHLIGHTS_QUERY],
        injections: "",
        language: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
    },
    LanguageDef {
        tokens: &["tsx"],
        highlights: &[tree_sitter_typescript::HIGHLIGHTS_QUERY],
        injections: "",
        language: || tree_sitter_typescript::LANGUAGE_TSX.into(),
    },
    LanguageDef {
        tokens: &["json", "jsonc"],
        highlights: &[tree_sitter_json::HIGHLIGHTS_QUERY],
        injections: "",
        language: || tree_sitter_json::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["yaml", "yml"],
        highlights: &[tree_sitter_yaml::HIGHLIGHTS_QUERY],
        injections: "",
        language: || tree_sitter_yaml::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["toml"],
        highlights: &[tree_sitter_toml_ng::HIGHLIGHTS_QUERY],
        injections: "",
        language: || tree_sitter_toml_ng::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["html"],
        highlights: &[tree_sitter_html::HIGHLIGHTS_QUERY],
        injections: tree_sitter_html::INJECTIONS_QUERY,
        language: || tree_sitter_html::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["css"],
        highlights: &[tree_sitter_css::HIGHLIGHTS_QUERY],
        injections: "",
        language: || tree_sitter_css::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["python", "py"],
        highlights: &[tree_sitter_python::HIGHLIGHTS_QUERY],
        injections: "",
        language: || tree_sitter_python::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["rust", "rs"],
        highlights: &[tree_sitter_rust::HIGHLIGHTS_QUERY],
        injections: tree_sitter_rust::INJECTIONS_QUERY,
        language: || tree_sitter_rust::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["go", "golang"],
        highlights: &[tree_sitter_go::HIGHLIGHTS_QUERY],
        injections: "",
        language: || tree_sitter_go::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["c"],
        highlights: &[tree_sitter_c::HIGHLIGHT_QUERY],
        injections: "",
        language: || tree_sitter_c::LANGUAGE.into(),
    },
    LanguageDef {
        tokens: &["cpp", "c++"],
        highlights: &[tree_sitter_cpp::HIGHLIGHT_QUERY],
        injections: "",
        language: || tree_sitter_cpp::LANGUAGE.into(),
    },
    // markdown is registered in `configs()` below — its injections need a
    // patched query (see the comment there) and its inline grammar needs
    // registering alongside the block grammar
];

/// Compile every grammar's `HighlightConfiguration` once; the map is
/// keyed by every token (canonical + aliases) that selects it. A grammar
/// whose queries fail to compile is skipped (its fences fall back to
/// escaped plain lines).
fn configs() -> &'static HashMap<&'static str, &'static HighlightConfiguration> {
    static CONFIGS: OnceLock<HashMap<&'static str, &'static HighlightConfiguration>> = OnceLock::new();
    CONFIGS.get_or_init(|| {
        let mut map = HashMap::new();
        for def in LANGUAGES {
            let highlights = def.highlights.join("\n");
            // deliberately leaked: the registry lives for the whole process
            let Ok(config) = Box::leak(Box::new(HighlightConfiguration::new(
                (def.language)(),
                def.tokens[0],
                &highlights,
                def.injections,
                "",
            ))) else {
                eprintln!("syntax: {} query failed to compile", def.tokens[0]);
                continue;
            };
            config.configure(CAPTURE_NAMES);
            for token in def.tokens {
                map.insert(*token, &*config);
            }
        }
        // The markdown injections ship without `injection.include-children`,
        // but `(inline)` and `code_fence_content` are container nodes —
        // without the predicate the injected ranges come back empty and
        // nothing inside a markdown fence ever gets styled. Re-add the
        // affected patterns with the predicate set.
        let md_injections: &'static str = Box::leak(
            format!(
                "{}\n\
                 ((inline) @injection.content\n\
                 \x20 (#set! injection.language \"markdown_inline\")\n\
                 \x20 (#set! injection.include-children))\n\
                 ((fenced_code_block\n\
                 \x20  (info_string (language) @injection.language)\n\
                 \x20  (code_fence_content) @injection.content)\n\
                 \x20 (#set! injection.include-children))\n",
                tree_sitter_md::INJECTION_QUERY_BLOCK
            )
            .into_boxed_str(),
        );
        let Ok(md) = Box::leak(Box::new(HighlightConfiguration::new(
            tree_sitter_md::LANGUAGE.into(),
            "markdown",
            tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
            md_injections,
            "",
        ))) else {
            eprintln!("syntax: markdown query failed to compile");
            return map;
        };
        md.configure(CAPTURE_NAMES);
        for token in ["markdown", "md"] {
            map.insert(token, &*md);
        }
        // the block grammar injects `markdown_inline` for paragraph text;
        // the inline grammar then injects fenced-code languages by name
        let Ok(inline) = Box::leak(Box::new(HighlightConfiguration::new(
            tree_sitter_md::INLINE_LANGUAGE.into(),
            "markdown-inline",
            tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
            tree_sitter_md::INJECTION_QUERY_INLINE,
            "",
        ))) else {
            eprintln!("syntax: markdown-inline query failed to compile");
            return map;
        };
        inline.configure(CAPTURE_NAMES);
        for token in ["markdown_inline", "markdown-inline"] {
            map.insert(token, &*inline);
        }
        map
    })
}

fn config_for(token: &str) -> Option<&'static HighlightConfiguration> {
    let token = token.trim().to_lowercase();
    if token.is_empty() || matches!(token.as_str(), "text" | "txt" | "plain" | "ansi") {
        return None;
    }
    configs().get(token.as_str()).copied()
}

/// Shiki-style line notations: `[!code highlight]` / `hl` / `focus` /
/// `++` / `--` / `warning` / `error`, with `:N` propagating over the
/// next N-1 lines; `[!!code …]` renders the notation literally.
fn line_notations(lines: &[String]) -> (Vec<String>, Vec<Vec<&'static str>>) {
    static NOTATION: OnceLock<regex::Regex> = OnceLock::new();
    let re = NOTATION.get_or_init(|| {
        regex::Regex::new(r"\[!(!)?code\s+(highlight|hl|focus|warning|error|\+\+|--)(?::(\d+))?\]").unwrap()
    });
    let mut clean: Vec<String> = Vec::with_capacity(lines.len());
    let mut classes: Vec<Vec<&'static str>> = vec![Vec::new(); lines.len()];
    let class_of = |kind: &str| match kind {
        "highlight" | "hl" => "hl",
        "focus" => "focus",
        "++" => "diff add",
        "--" => "diff remove",
        "warning" => "warning",
        _ => "error",
    };
    for (idx, line) in lines.iter().enumerate() {
        let mut line_classes: Vec<&'static str> = Vec::new();
        let mut out = String::with_capacity(line.len());
        let mut last = 0usize;
        for cap in re.captures_iter(line) {
            let whole = cap.get(0).unwrap();
            out.push_str(&line[last..whole.start()]);
            last = whole.end();
            if cap.get(1).is_some() {
                // escaped: drop one `!`, keep the notation as literal text
                out.push_str(&format!(
                    "[!code {}{}]",
                    &cap[2],
                    cap.get(3).map(|n| format!(":{}", n.as_str())).unwrap_or_default()
                ));
                continue;
            }
            let class = class_of(&cap[2]);
            line_classes.push(class);
            if let Some(n) = cap.get(3).and_then(|n| n.as_str().parse::<usize>().ok()) {
                // highlight:N also covers the next N-1 lines
                for extra in classes.iter_mut().skip(idx + 1).take(n.saturating_sub(1)) {
                    extra.push(class);
                }
            }
        }
        out.push_str(&line[last..]);
        classes[idx].extend(line_classes.iter().copied());
        clean.push(out);
    }
    (clean, classes)
}

/// The class list for one rendered line: `line`, `hl` (from meta or a
/// notation), plus notation classes (focus / diff / warning / error).
fn line_class(number: usize, hl: &std::collections::HashSet<usize>, notation: &[&'static str]) -> String {
    let mut class = String::from("line");
    if hl.contains(&number) || notation.contains(&"hl") {
        class.push_str(" hl");
    }
    for c in notation {
        if *c != "hl" {
            class.push(' ');
            class.push_str(c);
        }
    }
    class
}

const COPY_BUTTON: &str = "<button class=\"vp-copy-button\" type=\"button\" aria-label=\"Copy code\" onclick=\"gdCopyCode(this)\"><svg class=\"icon-copy\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" aria-hidden=\"true\"><rect width=\"8\" height=\"4\" x=\"8\" y=\"2\" rx=\"1\" ry=\"1\"/><path d=\"M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2\"/></svg><svg class=\"icon-copied\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" aria-hidden=\"true\"><rect width=\"8\" height=\"4\" x=\"8\" y=\"2\" rx=\"1\" ry=\"1\"/><path d=\"M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2\"/><path d=\"m9 14l2 2l4-4\"/></svg></button>";

fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

fn escape_attr(s: &str) -> String {
    escape_text(s).replace('\'', "&#39;")
}

impl CodefenceRendererAdapter for GdCodeRenderer {
    fn write(
        &self,
        output: &mut dyn fmt::Write,
        lang: &str,
        meta: &str,
        code: &str,
        _sourcepos: Option<Sourcepos>,
    ) -> fmt::Result {
        debug_assert_eq!(lang, FENCE_LANG);
        let spec = FenceSpec::parse_meta(meta);
        let hl: std::collections::HashSet<usize> = spec.hl.iter().copied().collect();

        // line notations ([!code …]) are stripped from the text and turn
        // into per-line classes
        let raw_lines: Vec<String> = code.split_inclusive('\n').map(String::from).collect();
        let (lines, notation_classes) = line_notations(&raw_lines);
        let has_focus = notation_classes.iter().any(|c| c.contains(&"focus"));
        let (show_ln, ln_start) = match spec.ln {
            Some(LineNumbers::On) => (true, None),
            Some(LineNumbers::Off) => (false, None),
            Some(LineNumbers::From(n)) => (true, Some(n)),
            None => (self.options.line_numbers, None),
        };

        let mut pre_class = format!("language-{}", escape_attr(&spec.lang));
        if has_focus {
            pre_class.push_str(" has-focus");
        }
        if show_ln {
            pre_class.push_str(" line-numbers");
        }
        // A labeled fence outside a code group renders upstream's
        // `.vp-code-block-title` card: the label becomes the title bar
        // text (snippet includes use the filename) and the corner lang
        // label stays empty, exactly like the deployed site.
        let titled = spec.label.is_some() && !spec.group;
        if titled {
            let title = spec.label.as_deref().unwrap_or_default();
            write!(
                output,
                "<div class=\"vp-code-block-title\"><div class=\"vp-code-block-title-bar\"><span class=\"vp-code-block-title-text\" data-title=\"{}\">{}</span></div>",
                escape_attr(title),
                escape_text(title)
            )?;
        }
        write!(output, "<pre class=\"{pre_class}\"")?;
        if show_ln && ln_start.is_some_and(|n| n > 1) {
            write!(output, " style=\"counter-reset: gdln {};\"", ln_start.unwrap() - 1)?;
        }
        write!(output, ">")?;
        if self.options.copy_button {
            write!(output, "{COPY_BUTTON}")?;
        }
        // VitePress always renders the corner label: the fence's `[title]`
        // when present, otherwise the language name (empty for plain
        // fences, which renders invisible)
        let label = if titled {
            ""
        } else {
            spec.label.as_deref().unwrap_or(spec.lang.as_str())
        };
        write!(output, "<span class=\"lang\">{}</span>", escape_text(label))?;
        write!(
            output,
            "<code class=\"language-{}\" data-lang=\"{}\"",
            escape_attr(&spec.lang),
            escape_attr(&spec.lang)
        )?;
        if let Some(label) = &spec.label {
            write!(output, " data-name=\"{}\"", escape_attr(label))?;
        }
        output.write_str(">")?;

        if spec.lang.eq_ignore_ascii_case("ansi") {
            // SGR escape sequences → classed spans, like Shiki's `ansi`
            // grammar upstream; tree-sitter has no grammar to offer here
            render_ansi(output, &lines, &hl, &notation_classes)?;
            output.write_str("</code></pre>")?;
            if titled {
                output.write_str("</div>")?;
            }
            return Ok(());
        }

        let stripped = lines.concat();
        match config_for(&spec.lang) {
            Some(config) => {
                // walk highlight events into classed segments over the
                // (notation-stripped) source
                let mut highlighter = Highlighter::new();
                // a closure (not the fn item): keeps lifetime inference
                // local, and silences clippy's redundant-closure suggestion
                #[allow(clippy::redundant_closure)]
                let events = highlighter
                    .highlight(config, stripped.as_bytes(), None, |name| config_for(name))
                    .map_err(|_| fmt::Error)?;
                let mut segments: Vec<(usize, usize, Option<&'static str>)> = Vec::new();
                let mut stack: Vec<&'static str> = Vec::new();
                for event in events {
                    match event.map_err(|_| fmt::Error)? {
                        HighlightEvent::HighlightStart(h) => {
                            stack.push(CAPTURE_NAMES.get(h.0).copied().unwrap_or(""));
                        }
                        HighlightEvent::HighlightEnd => {
                            stack.pop();
                        }
                        HighlightEvent::Source { start, end } => {
                            segments.push((start, end, stack.last().copied()));
                        }
                    }
                }

                // render line by line; each line is composed from the
                // segments clipped to it, so `.line` wrappers never
                // interleave with capture spans (same trick as Shiki)
                let bytes = stripped.as_bytes();
                let mut starts = vec![0usize];
                for (i, b) in bytes.iter().enumerate() {
                    if *b == b'\n' {
                        starts.push(i + 1);
                    }
                }
                // a source ending in a newline produces one trailing empty
                // start; it has no content and no notation entry
                starts.retain(|&s| s < bytes.len() || s == 0);
                for (idx, &start) in starts.iter().enumerate() {
                    let next = starts.get(idx + 1).copied().unwrap_or(bytes.len());
                    // the newline goes BETWEEN the line spans, never inside:
                    // `.line` is an inline-block in a `white-space: pre`
                    // container and only stacks at a break between spans
                    let end = if next > start && bytes[next - 1] == b'\n' {
                        next - 1
                    } else {
                        next
                    };
                    let class = line_class(idx + 1, &hl, &notation_classes[idx]);
                    if idx > 0 {
                        output.write_str("\n")?;
                    }
                    write!(output, "<span class=\"{class}\">")?;
                    let mut pos = start;
                    for (s, e, capture) in &segments {
                        let seg_start = (*s).max(start);
                        let seg_end = (*e).min(end);
                        if seg_end <= seg_start {
                            continue;
                        }
                        if seg_start > pos {
                            output.write_str(&escape_text(std::str::from_utf8(&bytes[pos..seg_start]).unwrap()))?;
                            pos = seg_start;
                        }
                        match capture {
                            Some(name) => write!(
                                output,
                                "<span class=\"{}\">{}</span>",
                                tk_class(name),
                                escape_text(std::str::from_utf8(&bytes[pos..seg_end]).unwrap())
                            )?,
                            None => output.write_str(&escape_text(std::str::from_utf8(&bytes[pos..seg_end]).unwrap()))?,
                        }
                        pos = seg_end;
                    }
                    if pos < end {
                        output
                            .write_str(&escape_text(std::str::from_utf8(&bytes[pos..end]).unwrap()))?;
                    }
                    output.write_str("</span>")?;
                }
            }
            None => {
                for (idx, line) in lines.iter().enumerate() {
                    let class = line_class(idx + 1, &hl, &notation_classes[idx]);
                    let text = line.strip_suffix('\n').unwrap_or(line);
                    if idx > 0 {
                        output.write_str("\n")?;
                    }
                    write!(output, "<span class=\"{class}\">{}</span>", escape_text(text))?;
                }
            }
        }
        output.write_str("</code></pre>")?;
        if titled {
            output.write_str("</div>")?;
        }
        Ok(())
    }
}

/// The dual-theme stylesheet: light rules unscoped, dark rules under
/// `html.dark`, all inside `@layer syntax` so Tailwind utilities win.
/// Each standard capture name resolves through the theme's longest
/// prefix; names the theme doesn't style emit no rule (they inherit the
/// block's base color from the `.vp-doc` rules).
pub fn syntax_css(light: &SyntaxTheme, dark: &SyntaxTheme) -> String {
    let block = |theme: &SyntaxTheme, scope_prefix: &str, out: &mut String| {
        for name in CAPTURE_NAMES {
            let Some(style) = theme.resolve(name) else { continue };
            let ThemeStyle { fg, bg, bold, italic, underline } = style;
            if fg.is_none() && bg.is_none() && !bold && !italic && !underline {
                continue;
            }
            out.push_str(&format!("{scope_prefix}.{} {{\n", tk_class(name)));
            if let Some(fg) = fg {
                out.push_str(&format!("  color: {fg};\n"));
            }
            if let Some(bg) = bg {
                out.push_str(&format!("  background-color: {bg};\n"));
            }
            if *bold {
                out.push_str("  font-weight: 600;\n");
            }
            if *italic {
                out.push_str("  font-style: italic;\n");
            }
            if *underline {
                out.push_str("  text-decoration: underline;\n");
            }
            out.push_str("}\n");
        }
    };
    let mut out = String::from("@layer syntax {\n");
    block(light, "", &mut out);
    block(dark, "html.dark ", &mut out);
    out.push_str(ANSI_CSS);
    out.push_str("}\n");
    out
}

/// SGR classes for one segment; empty when the state is plain.
impl SgrState {
    fn classes(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.bold {
            parts.push("ansi-bold");
        }
        if self.dim {
            parts.push("ansi-dim");
        }
        if self.italic {
            parts.push("ansi-italic");
        }
        if self.underline {
            parts.push("ansi-underline");
        }
        if let Some(fg) = self.fg {
            parts.push(fg);
        }
        parts.join(" ")
    }

    fn apply_sgr(&mut self, params: &str) {
        // unparsable junk maps to an out-of-range sentinel (ignored by the
        // match) rather than 0, which would reset the whole style
        let nums: Vec<u16> = params
            .split(';')
            .map(|p| match p.parse::<u16>() {
                Ok(n) => n,
                // an empty parameter means 0 per ECMA-48
                _ if p.is_empty() => 0,
                Err(_) => u16::MAX,
            })
            .collect();
        let mut i = 0;
        while i < nums.len() {
            match nums[i] {
                0 => {
                    self.fg = None;
                    self.bold = false;
                    self.dim = false;
                    self.italic = false;
                    self.underline = false;
                }
                1 => self.bold = true,
                2 => self.dim = true,
                3 => self.italic = true,
                4 => self.underline = true,
                // 21 is "doubly underlined" per ECMA-48 but "bold off" in
                // every mainstream terminal emulator
                21 | 22 => {
                    self.bold = false;
                    self.dim = false;
                }
                23 => self.italic = false,
                24 => self.underline = false,
                30..=37 => self.fg = Some(FG_CLASSES[(nums[i] - 30) as usize]),
                38 | 48 | 58 => {
                    // extended color: `5;N` (256-color) or `2;R;G;B` —
                    // consume the arguments (2 + 1 or 2 + 4 parameters
                    // total) so following codes survive; the docs corpus
                    // only ever uses the basics, so the color itself
                    // stays untouched. A malformed form consumes only
                    // the intro parameter.
                    let step = match nums.get(i + 1) {
                        Some(5) => 2,
                        Some(2) => 4,
                        _ => 0,
                    };
                    i += step;
                }
                // SGR 39 = terminal default foreground: shiki gives it the
                // theme's editor foreground, which differs from the code
                // block's muted base color
                39 => self.fg = Some("ansi-fg-default"),
                90..=97 => self.fg = Some(FG_CLASSES[(nums[i] - 90 + 8) as usize]),
                // 40–47/49/100–107: backgrounds, not styled
                _ => {}
            }
            i += 1;
        }
    }

    /// Split one rendered line into (class-list, text) segments, carrying
    /// SGR state across segment (and via `&mut self`, line) boundaries.
    fn segments(&mut self, text: &str) -> Vec<(String, String)> {
        let bytes = text.as_bytes();
        let mut out: Vec<(String, String)> = Vec::new();
        let mut plain = String::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                let params_start = i + 2;
                let mut j = params_start;
                while j < bytes.len() && (0x30..=0x3f).contains(&bytes[j]) {
                    j += 1;
                }
                let params_end = j;
                // intermediate bytes, then one final byte 0x40–0x7e
                while j < bytes.len() && (0x20..=0x2f).contains(&bytes[j]) {
                    j += 1;
                }
                if j < bytes.len() && (0x40..=0x7e).contains(&bytes[j]) {
                    if bytes[j] == b'm' {
                        if !plain.is_empty() {
                            out.push((self.classes(), std::mem::take(&mut plain)));
                        }
                        self.apply_sgr(&text[params_start..params_end]);
                    }
                    i = j + 1;
                    continue;
                }
                // unterminated CSI: drop the whole partial sequence —
                // emitting the raw parameter bytes would show them as text
                i = j;
                continue;
            }
            let ch_len = text[i..].chars().next().map(char::len_utf8).unwrap_or(1);
            plain.push_str(&text[i..i + ch_len]);
            i += ch_len;
        }
        if !plain.is_empty() {
            out.push((self.classes(), plain));
        }
        out
    }
}

fn render_ansi(
    output: &mut dyn fmt::Write,
    lines: &[String],
    hl: &std::collections::HashSet<usize>,
    notation_classes: &[Vec<&'static str>],
) -> fmt::Result {
    let mut sgr = SgrState::default();
    for (idx, line) in lines.iter().enumerate() {
        let class = line_class(idx + 1, hl, &notation_classes[idx]);
        let text = line.strip_suffix('\n').unwrap_or(line);
        if idx > 0 {
            output.write_str("\n")?;
        }
        write!(output, "<span class=\"{class}\">")?;
        for (classes, seg) in sgr.segments(text) {
            if seg.is_empty() {
                continue;
            }
            if classes.is_empty() {
                output.write_str(&escape_text(&seg))?;
            } else {
                write!(
                    output,
                    "<span class=\"{classes}\">{}</span>",
                    escape_text(&seg)
                )?;
            }
        }
        output.write_str("</span>")?;
    }
    Ok(())
}

#[derive(Default)]
struct SgrState {
    fg: Option<&'static str>,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
}

/// Class fragments for SGR 30–37/90–97.
const FG_CLASSES: [&str; 16] = [
    "ansi-fg-black",
    "ansi-fg-red",
    "ansi-fg-green",
    "ansi-fg-yellow",
    "ansi-fg-blue",
    "ansi-fg-magenta",
    "ansi-fg-cyan",
    "ansi-fg-white",
    "ansi-fg-bright-black",
    "ansi-fg-bright-red",
    "ansi-fg-bright-green",
    "ansi-fg-bright-yellow",
    "ansi-fg-bright-blue",
    "ansi-fg-bright-magenta",
    "ansi-fg-bright-cyan",
    "ansi-fg-bright-white",
];

/// GitHub terminal palette (light and dark). The light entries for green,
/// cyan and bright-black — and every dark entry the rendered docs use — are
/// measured off the deployed vitepress.dev, whose pinned shiki predates
/// the current @shikijs/themes palette; the rest follow @shikijs/themes.
const ANSI_CSS: &str = r#"  .ansi-bold { font-weight: 700; }
  .ansi-dim { opacity: 0.67; }
  .ansi-italic { font-style: italic; }
  .ansi-underline { text-decoration: underline; }
  .ansi-fg-default { color: #24292e; }
  .ansi-fg-black { color: #24292e; }
  .ansi-fg-red { color: #d73a49; }
  .ansi-fg-green { color: #0e790b; }
  .ansi-fg-yellow { color: #dbab09; }
  .ansi-fg-blue { color: #0366d6; }
  .ansi-fg-magenta { color: #5a32a3; }
  .ansi-fg-cyan { color: #06747a; }
  .ansi-fg-white { color: #6a737d; }
  .ansi-fg-bright-black { color: #6c676f; }
  .ansi-fg-bright-red { color: #cb2431; }
  .ansi-fg-bright-green { color: #22863a; }
  .ansi-fg-bright-yellow { color: #b08800; }
  .ansi-fg-bright-blue { color: #005cc5; }
  .ansi-fg-bright-magenta { color: #5a32a3; }
  .ansi-fg-bright-cyan { color: #3192aa; }
  .ansi-fg-bright-white { color: #d1d5da; }
  html.dark .ansi-fg-default { color: #e1e4e8; }
  html.dark .ansi-fg-black { color: #586069; }
  html.dark .ansi-fg-red { color: #ea4a5a; }
  html.dark .ansi-fg-green { color: #34d058; }
  html.dark .ansi-fg-yellow { color: #ffea7f; }
  html.dark .ansi-fg-blue { color: #2188ff; }
  html.dark .ansi-fg-magenta { color: #b392f0; }
  html.dark .ansi-fg-cyan { color: #39c5cf; }
  html.dark .ansi-fg-white { color: #d1d5da; }
  html.dark .ansi-fg-bright-black { color: #959da5; }
  html.dark .ansi-fg-bright-red { color: #f97583; }
  html.dark .ansi-fg-bright-green { color: #85e89d; }
  html.dark .ansi-fg-bright-yellow { color: #ffea7f; }
  html.dark .ansi-fg-bright-blue { color: #79b8ff; }
  html.dark .ansi-fg-bright-magenta { color: #b392f0; }
  html.dark .ansi-fg-bright-cyan { color: #56d4dd; }
  html.dark .ansi-fg-bright-white { color: #fafbfc; }
"#;

#[cfg(test)]
mod sgr_tests {
    use super::*;

    #[test]
    fn sgr_segments_carry_state_and_reset() {
        let mut s = SgrState::default();
        let segs = s.segments("\x1b[1m\x1b[36mhi\x1b[0m there");
        assert_eq!(
            segs,
            vec![
                ("ansi-bold ansi-fg-cyan".into(), "hi".into()),
                ("".into(), " there".into()),
            ]
        );
    }

    #[test]
    fn truecolor_consumes_exactly_five_params() {
        // 38;2;R;G;B is five parameters: the trailing code after it must
        // survive (a bold here was being swallowed)
        let mut s = SgrState::default();
        let segs = s.segments("\x1b[38;2;255;0;0;1mB");
        assert_eq!(segs, vec![("ansi-bold".into(), "B".into())]);
    }

    #[test]
    fn junk_params_do_not_reset() {
        let mut s = SgrState::default();
        let segs = s.segments("\x1b[1m\x1b[38:5:2mB");
        assert_eq!(segs, vec![("ansi-bold".into(), "B".into())]);
    }

    #[test]
    fn unterminated_csi_is_dropped_not_emitted() {
        let mut s = SgrState::default();
        let segs = s.segments("ok\x1b[38;2;2");
        assert_eq!(segs, vec![("".into(), "ok".into())]);
    }
}
