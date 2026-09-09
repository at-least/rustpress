//! Syntax highlighting: the `gdcode` codefence renderer (a comrak
//! `CodefenceRendererAdapter`) driving syntect with scope-name CSS
//! classes, plus the dual-theme `syntax.css` generator.
//!
//! Every fenced block is rewritten by the preprocessor to
//! ```` ```gdcode lang=js label=npm hl=1,3-4 ````, so this renderer sees
//! the whole site's code through one path: syntect highlighting with
//! `st-`-prefixed scope classes, per-line `<span class="line">` wrappers
//! (` hl` on highlighted lines), a server-side `<span class="lang">`
//! label, and `data-lang`/`data-name` attributes for the client-side
//! copy button and code-group tabs.

use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;
use std::sync::OnceLock;

use comrak::adapters::CodefenceRendererAdapter;
use comrak::nodes::Sourcepos;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::ClassStyle;
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};
use syntect::util::LinesWithEndings;

/// The sentinel language token all code fences are rewritten to.
pub const FENCE_LANG: &str = "gdcode";

/// Scope classes are prefixed to keep them out of the global CSS
/// namespace (`.st-source.st.js` instead of `.source.js`).
const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "st-" };

const GITHUB_LIGHT: &str = include_str!("../../assets/themes/github-light.tmTheme");
const GITHUB_DARK: &str = include_str!("../../assets/themes/github-dark.tmTheme");

/// The site's syntax definitions (syntect defaults, newline-flavoured).
pub fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// Load a highlight theme by name: the vendored github-light/github-dark
/// pair (converted from Shiki's VS Code themes), else syntect's bundled
/// set.
pub fn load_theme(name: &str) -> Option<Theme> {
    let vendored = match name {
        "github-light" => Some(GITHUB_LIGHT),
        "github-dark" => Some(GITHUB_DARK),
        _ => None,
    };
    if let Some(src) = vendored {
        return ThemeSet::load_from_reader(&mut Cursor::new(src)).ok();
    }
    static DEFAULTS: OnceLock<ThemeSet> = OnceLock::new();
    DEFAULTS
        .get_or_init(ThemeSet::load_defaults)
        .themes
        .get(name)
        .cloned()
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
    /// 1-based highlighted line numbers.
    pub hl: Vec<usize>,
    pub ln: Option<LineNumbers>,
}

impl FenceSpec {
    /// Parse `lang=js hl=1,3-4 label=npm` (label last — it may contain
    /// spaces).
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
    for line in lines {
        let mut line_classes: Vec<&'static str> = Vec::new();
        let mut out = String::with_capacity(line.len());
        let mut last = 0usize;
        for cap in re.captures_iter(line) {
            let whole = cap.get(0).unwrap();
            out.push_str(&line[last..whole.start()]);
            last = whole.end();
            if cap.get(1).is_some() {
                // escaped: drop one `!`, keep the notation as literal text
                out.push_str(&format!("[!code {}{}]", &cap[2], cap.get(3).map(|n| format!(":{}", n.as_str())).unwrap_or_default()));
                continue;
            }
            let class = class_of(&cap[2]);
            line_classes.push(class);
            if let Some(n) = cap.get(3).and_then(|n| n.as_str().parse::<usize>().ok()) {
                // highlight:N also covers the next N-1 lines
                for extra in classes.iter_mut().skip(clean.len() + 1).take(n.saturating_sub(1)) {
                    extra.push(class);
                }
            }
        }
        out.push_str(&line[last..]);
        classes[clean.len()].append(&mut line_classes);
        clean.push(out);
    }
    (clean, classes)
}

impl GdCodeRenderer {
    /// The plugins map comrak dispatches code fences through.
    pub fn plugins_map(&self) -> HashMap<String, &dyn CodefenceRendererAdapter> {
        let mut map: HashMap<String, &dyn CodefenceRendererAdapter> = HashMap::new();
        map.insert(FENCE_LANG.to_string(), self);
        map
    }

    fn syntax_for(&self, lang: &str) -> Option<&'static syntect::parsing::SyntaxReference> {
        let ss = syntax_set();
        let l = lang.trim().to_lowercase();
        if l.is_empty() || l == "text" || l == "txt" || l == "plain" || l == "ansi" {
            return None;
        }
        ss.find_syntax_by_token(&l)
            .or_else(|| ss.find_syntax_by_extension(&l))
    }

    fn line_numbers(&self, spec: &FenceSpec) -> (bool, Option<usize>) {
        match spec.ln {
            Some(LineNumbers::On) => (true, None),
            Some(LineNumbers::Off) => (false, None),
            Some(LineNumbers::From(n)) => (true, Some(n)),
            None => (self.options.line_numbers, None),
        }
    }
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
        let raw_lines: Vec<String> = LinesWithEndings::from(code).map(String::from).collect();
        let (lines, notation_classes) = line_notations(&raw_lines);
        let has_focus = notation_classes.iter().any(|c| c.contains(&"focus"));
        let (show_ln, ln_start) = self.line_numbers(&spec);

        let mut pre_class = format!("language-{}", escape_attr(&spec.lang));
        if has_focus {
            pre_class.push_str(" has-focus");
        }
        if show_ln {
            pre_class.push_str(" line-numbers");
        }
        write!(output, "<pre class=\"{pre_class}\"")?;
        if show_ln && ln_start.is_some_and(|n| n > 1) {
            write!(output, " style=\"counter-reset: gdln {};\"", ln_start.unwrap() - 1)?;
        }
        write!(output, ">")?;
        if self.options.copy_button {
            write!(output, "{COPY_BUTTON}")?;
        }
        if let Some(label) = &spec.label {
            write!(output, "<span class=\"lang\">{}</span>", escape_text(label))?;
        }
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

        let ss = syntax_set();
        match self.syntax_for(&spec.lang) {
            Some(syntax) => {
                let mut parse_state = ParseState::new(syntax);
                let mut stack = ScopeStack::new();
                // Scopes left open at the start of the current line
                // (multi-line strings, block comments). Each line's spans
                // are self-balanced: reopen the carried scopes at the line
                // start, close everything at the line end, so the .line
                // wrappers never interleave with syntect's spans.
                let mut carried: Vec<String> = Vec::new();
                for (idx, line) in lines.iter().enumerate() {
                    let ops = parse_state
                        .parse_line(line, ss)
                        .map_err(|_| fmt::Error)?;
                    let (mut html, delta) =
                        syntect::html::line_tokens_to_classed_spans(line, &ops, CLASS_STYLE, &mut stack)
                            .map_err(|_| fmt::Error)?;
                    let mut prefix = String::new();
                    for scope in &carried {
                        prefix.push_str(&scope_span(scope));
                    }
                    let open_now = (carried.len() as isize + delta).max(0) as usize;
                    html.push_str(&"</span>".repeat(open_now));
                    carried = stack.to_string().split_whitespace().map(String::from).collect();
                    let class = line_class(idx + 1, &hl, &notation_classes[idx]);
                    write!(output, "<span class=\"{class}\">{prefix}{html}</span>")?;
                }
            }
            None => {
                for (idx, line) in lines.iter().enumerate() {
                    let class = line_class(idx + 1, &hl, &notation_classes[idx]);
                    write!(output, "<span class=\"{class}\">{}</span>", escape_text(line))?;
                }
            }
        }
        output.write_str("</code></pre>")
    }
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
/// The dual-theme stylesheet: light rules unscoped, dark rules under
/// `html.dark`, all inside `@layer syntax` so Tailwind utilities win.
pub fn syntax_css(light: &Theme, dark: &Theme) -> Result<String, syntect::Error> {
    let l = syntect::html::css_for_theme_with_class_style(light, CLASS_STYLE)?;
    let d = syntect::html::css_for_theme_with_class_style(dark, CLASS_STYLE)?;
    let mut out = String::from("@layer syntax {\n");
    out.push_str(&l);
    out.push('\n');
    for line in d.lines() {
        let t = line.trim_start();
        if t.starts_with('.') {
            out.push_str("html.dark ");
            out.push_str(line);
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str("}\n");
    Ok(out)
}

/// The copy button (icons toggled by the `.copied` class rules in the
/// vp-doc styles); gdCopyCode lives in the Alpine entry.
const COPY_BUTTON: &str = "<button class=\"vp-copy-button\" type=\"button\" aria-label=\"Copy code\" onclick=\"gdCopyCode(this)\"><svg class=\"icon-copy\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" aria-hidden=\"true\"><rect width=\"8\" height=\"4\" x=\"8\" y=\"2\" rx=\"1\" ry=\"1\"/><path d=\"M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2\"/></svg><svg class=\"icon-copied\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" aria-hidden=\"true\"><rect width=\"8\" height=\"4\" x=\"8\" y=\"2\" rx=\"1\" ry=\"1\"/><path d=\"M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2\"/><path d=\"m9 14l2 2l4-4\"/></svg></button>";

/// One reopened scope: a span carrying the scope's st--prefixed atoms.
fn scope_span(scope: &str) -> String {
    let classes: Vec<String> = scope.split('.').map(|atom| format!("st-{atom}")).collect();
    format!("<span class=\"{}\">", classes.join(" "))
}

fn escape_attr(s: &str) -> String {
    escape_text(s).replace('\'', "&#39;")
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_parsing() {
        let s = FenceSpec::parse_meta("lang=js hl=1,3-4 label=npm");
        assert_eq!(s.lang, "js");
        assert_eq!(s.label.as_deref(), Some("npm"));
        assert_eq!(s.hl, vec![1, 3, 4]);

        let s = FenceSpec::parse_meta("lang=sh label=.vitepress/config.js");
        assert_eq!(s.lang, "sh");
        assert_eq!(s.label.as_deref(), Some(".vitepress/config.js"));
        assert!(s.hl.is_empty());

        let s = FenceSpec::parse_meta("lang=");
        assert_eq!(s.lang, "");
    }

    #[test]
    fn vendored_themes_load_and_css_generates() {
        let light = load_theme("github-light").expect("github-light loads");
        let dark = load_theme("github-dark").expect("github-dark loads");
        let css = syntax_css(&light, &dark).expect("css generates");
        assert!(css.starts_with("@layer syntax"));
        assert!(css.contains("html.dark .st-"), "dark rules scoped: {}", &css[..css.len().min(400)]);
        assert!(css.contains(".st-"));
    }

    #[test]
    fn renderer_emits_vitepress_shaped_markup() {
        let mut out = String::new();
        GdCodeRenderer::default()
            .write(&mut out, FENCE_LANG, "lang=js hl=2 label=a.js", "const a = 1;\nconst b = 2;\n", None)
            .unwrap();
        assert!(out.starts_with("<pre class=\"language-js\">"), "{out}");
        assert!(out.contains("<span class=\"lang\">a.js</span>"));
        assert!(out.contains("data-lang=\"js\""));
        assert!(out.contains("data-name=\"a.js\""));
        assert!(out.contains("<span class=\"line\">"));
        assert!(out.contains("<span class=\"line hl\">"));
        assert!(out.ends_with("</code></pre>"));
        // js highlighting: keyword scopes present
        assert!(out.contains("st-keyword"), "{out}");
    }

    #[test]
    fn plain_langs_escape_instead_of_highlight() {
        let mut out = String::new();
        GdCodeRenderer::default()
            .write(&mut out, FENCE_LANG, "lang=ansi", "\x1b[1mbold\x1b[0m\n", None)
            .unwrap();
        assert!(!out.contains("st-"), "no scope classes: {out}");
        assert!(out.contains("<span class=\"line\">"), "{out}");
        let mut out2 = String::new();
        GdCodeRenderer::default()
            .write(&mut out2, FENCE_LANG, "lang=", "plain <text>\n", None)
            .unwrap();
        assert!(out2.contains("plain &lt;text&gt;"), "{out2}");
    }
}

#[cfg(test)]
mod span_balance_tests {
    use super::*;
    use comrak::adapters::CodefenceRendererAdapter;

    #[test]
    fn spans_balance_per_block_even_with_cross_line_scopes() {
        // js template literal + block comment: scopes span lines
        let code = "const s = `multi\nline`;\n/* block\ncomment */\nlet x = 1;\n";
        let mut out = String::new();
        GdCodeRenderer::default().write(&mut out, FENCE_LANG, "lang=js", code, None).unwrap();
        let body = out.split("<code").nth(1).unwrap();
        let body = &body[..body.find("</code>").unwrap()];
        assert_eq!(
            body.matches("<span").count(),
            body.matches("</span>").count(),
            "spans balance across the block: {out}"
        );
        // every .line wrapper is self-contained: within one wrapper's
        // segment (up to the next wrapper), spans open and close equally
        let mut rest = body;
        while let Some(i) = rest.find("<span class=\"line") {
            let after = &rest[i..];
            let open_end = after.find('>').unwrap() + 1;
            let next = after[open_end..]
                .find("<span class=\"line")
                .map(|n| open_end + n)
                .unwrap_or(after.len());
            let mut segment = &after[open_end..next];
            // each segment ends with the wrapper's own close (except the
            // last, whose close precedes </code>) — discount one
            if let Some(t) = segment.rfind("</span>")
                && (t + "</span>".len() == segment.len()
                    || segment[t + 7..].trim_start().starts_with("</code>"))
                {
                    segment = &segment[..t];
                }
            assert_eq!(
                segment.matches("<span").count(),
                segment.matches("</span>").count(),
                "line wrapper segment unbalanced: {segment}"
            );
            rest = &after[open_end..];
        }
    }
}

#[cfg(test)]
mod notation_tests {
    use super::*;
    use comrak::adapters::CodefenceRendererAdapter;

    fn render(code: &str, meta: &str, options: RendererOptions) -> String {
        let mut out = String::new();
        GdCodeRenderer { options }
            .write(&mut out, FENCE_LANG, meta, code, None)
            .unwrap();
        out
    }

    #[test]
    fn notations_become_line_classes_and_vanish_from_text() {
        let out = render(
            "a [!code highlight]\nb\n// [!code focus]\n+ added [!code ++]\n- gone [!code --]\nw [!code warning]\ne [!code error]\n",
            "lang=txt",
            RendererOptions::default(),
        );
        assert!(!out.contains("[!code"), "markers stripped: {out}");
        assert!(out.contains("<span class=\"line hl\">"), "{out}");
        assert!(out.contains("<pre class=\"language-txt has-focus\">"), "{out}");
        assert!(out.contains("<span class=\"line focus\">"), "{out}");
        assert!(out.contains("<span class=\"line diff add\">"), "{out}");
        assert!(out.contains("<span class=\"line diff remove\">"), "{out}");
        assert!(out.contains("<span class=\"line warning\">"), "{out}");
        assert!(out.contains("<span class=\"line error\">"), "{out}");
    }

    #[test]
    fn notation_with_count_covers_following_lines() {
        let out = render("a [!code highlight:3]\nb\nc\nd\n", "lang=txt", RendererOptions::default());
        // lines 1..=3 get hl
        let hits = out.matches("<span class=\"line hl\">").count();
        assert_eq!(hits, 3, "{out}");
    }

    #[test]
    fn escaped_notation_keeps_literal_text() {
        let out = render("x [!!code highlight] y\n", "lang=txt", RendererOptions::default());
        assert!(out.contains("[!code highlight]"), "literal kept: {out}");
        assert!(!out.contains("line hl"), "{out}");
    }

    #[test]
    fn per_fence_line_numbers() {
        let on = render("a\nb\n", "lang=txt ln=true", RendererOptions::default());
        assert!(on.contains("<pre class=\"language-txt line-numbers\">"), "{on}");
        let from = render("a\nb\n", "lang=txt ln=5", RendererOptions::default());
        assert!(from.contains("counter-reset: gdln 4;"), "{from}");
        let off = render("a\n", "lang=txt ln=false", RendererOptions { line_numbers: true, copy_button: true });
        assert!(!off.contains("line-numbers"), "{off}");
        let global = render("a\n", "lang=txt", RendererOptions { line_numbers: true, copy_button: true });
        assert!(global.contains("line-numbers"), "{global}");
    }

    #[test]
    fn copy_button_can_be_disabled() {
        let out = render("a\n", "lang=txt", RendererOptions { line_numbers: false, copy_button: false });
        assert!(!out.contains("vp-copy-button"), "{out}");
    }

    #[test]
    fn meta_hl_and_notations_coexist() {
        let out = render("a\nb [!code highlight]\n", "lang=txt hl=1", RendererOptions::default());
        assert_eq!(out.matches("<span class=\"line hl\">").count(), 2, "{out}");
    }
}
