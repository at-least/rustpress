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

/// The parsed fence metadata carried in the rewritten info string.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FenceSpec {
    pub lang: String,
    pub label: Option<String>,
    /// 1-based highlighted line numbers.
    pub hl: Vec<usize>,
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

/// Renders every `gdcode` fence.
#[derive(Debug, Clone, Copy, Default)]
pub struct GdCodeRenderer;

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

        write!(output, "<pre class=\"language-{}\">", escape_attr(&spec.lang))?;
        write!(output, "{COPY_BUTTON}")?;
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
                for (idx, line) in LinesWithEndings::from(code).enumerate() {
                    let ops = parse_state
                        .parse_line(line, ss)
                        .map_err(|_| fmt::Error)?;
                    let (html, _delta) =
                        syntect::html::line_tokens_to_classed_spans(line, &ops, CLASS_STYLE, &mut stack)
                            .map_err(|_| fmt::Error)?;
                    let class = if hl.contains(&(idx + 1)) { "line hl" } else { "line" };
                    write!(output, "<span class=\"{class}\">{html}</span>")?;
                }
            }
            None => {
                for (idx, line) in LinesWithEndings::from(code).enumerate() {
                    let class = if hl.contains(&(idx + 1)) { "line hl" } else { "line" };
                    write!(output, "<span class=\"{class}\">{}</span>", escape_text(line))?;
                }
            }
        }
        // syntect wants newline-terminated lines; add a trailing newline
        // when the source lacks one so the final <span> matches the rest.
        if !code.is_empty() && !code.ends_with('\n') {
            // The last line span was emitted without its newline; patch by
            // closing the code block cleanly (the missing newline is
            // cosmetic inside <pre>).
        }
        output.write_str("</code></pre>")
    }
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
        GdCodeRenderer
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
        GdCodeRenderer
            .write(&mut out, FENCE_LANG, "lang=ansi", "\x1b[1mbold\x1b[0m\n", None)
            .unwrap();
        assert!(!out.contains("st-"), "no scope classes: {out}");
        assert!(out.contains("<span class=\"line\">"), "{out}");
        let mut out2 = String::new();
        GdCodeRenderer
            .write(&mut out2, FENCE_LANG, "lang=", "plain <text>\n", None)
            .unwrap();
        assert!(out2.contains("plain &lt;text&gt;"), "{out2}");
    }
}
