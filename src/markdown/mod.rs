//! The markdown pipeline: preprocess (includes/containers/fences/badges)
//! → comrak parse (GFM + alerts + footnotes + GitHub-style heading ids)
//! → AST fixups (relative `.md` link rewriting to canonical page URLs)
//! → HTML with the `gdcode` highlighter.

pub mod highlight;
pub mod preprocess;

use std::path::Path;

use comrak::nodes::{NodeLink, NodeValue};
use comrak::options::{Plugins, RenderPlugins};
use comrak::{Anchorizer, Arena, Options};
use std::sync::OnceLock;
use syntect::highlighting::Theme;

use crate::config::Markdown as MarkdownConfig;
use crate::content::{Content, Page};

/// The one shipped highlight theme pair (vendored tmThemes; see
/// assets/themes/). Intentionally not configurable — the generator has a
/// single design, like the templates.
pub const THEME_LIGHT: &str = "github-light";
pub const THEME_DARK: &str = "github-dark";

/// One outline heading with its render-matching anchor id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    /// The final anchor id: the `{#custom}` attribute when present,
    /// else comrak's slug.
    pub id: String,
    pub text: String,
    /// The slug id comrak rendered (before the custom-anchor
    /// post-processing swaps it for `id`).
    pub rendered_id: String,
}

/// The result of rendering one page's markdown body.
#[derive(Debug, Clone, Default)]
pub struct RenderedPage {
    pub html: String,
    pub headings: Vec<Heading>,
    /// The page contains `$…$` / `$$…$$` math (needs the MathJax script).
    pub has_math: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
    #[error("highlight theme {name:?} not found (vendored: github-light, github-dark)")]
    Theme { name: String },
    #[error(transparent)]
    Preprocess(#[from] preprocess::PreprocessError),
}

/// Reusable engine: comrak options + highlighter themes.
pub struct MarkdownEngine {
    options: Options<'static>,
    renderer: highlight::GdCodeRenderer,
    light: Theme,
    dark: Theme,
    lazy_images: bool,
    math: bool,
    config_container: crate::config::ContainerOptions,
}

impl MarkdownEngine {
    pub fn new(config: &MarkdownConfig) -> Result<Self, MarkdownError> {
        let mut options = Options::default();
        let ext = &mut options.extension;
        ext.table = true;
        ext.tasklist = true;
        ext.strikethrough = true;
        ext.autolink = true;
        ext.footnotes = true;
        ext.alerts = true;
        // :tada:-style emoji shortcodes, like VitePress
        ext.shortcodes = true;
        if config.math {
            ext.math_dollars = true;
        }
        // GitHub-style ids on every heading; the trailing `<a
        // class="anchor">` link comrak appends is styled by the theme CSS.
        ext.header_id_prefix = Some(String::new());
        // VitePress renders raw HTML in markdown; our preprocessing emits
        // trusted wrappers (containers, badges) as raw HTML too.
        options.render.r#unsafe = true;
        options.render.tasklist_classes = true;
        let light = highlight::load_theme(THEME_LIGHT)
            .ok_or_else(|| MarkdownError::Theme { name: THEME_LIGHT.to_string() })?;
        let dark = highlight::load_theme(THEME_DARK)
            .ok_or_else(|| MarkdownError::Theme { name: THEME_DARK.to_string() })?;
        Ok(Self {
            options,
            renderer: highlight::GdCodeRenderer {
                options: highlight::RendererOptions {
                    copy_button: config.code_copy_button,
                    line_numbers: config.line_numbers,
                },
            },
            light,
            dark,
            lazy_images: config.image.lazy_loading,
            math: config.math,
            config_container: config.container.clone(),
        })
    }

    /// The dual-theme `syntax.css` content.
    pub fn syntax_css(&self) -> String {
        highlight::syntax_css(&self.light, &self.dark).expect("themes already loaded")
    }

    /// Render one page: `site_root` is where `@/` includes resolve,
    /// `content_dir` the content tree the page came from.
    pub fn render(
        &self,
        page: &Page,
        content: &Content,
        site_root: &Path,
        content_dir: &Path,
    ) -> Result<RenderedPage, MarkdownError> {
        let pre = preprocess::Preprocess { site_root, content_dir, container: self.config_container.clone() };
        let md = pre.run(&page.body, &page.rel)?;

        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &md, &self.options);
        rewrite_links(&root, page, content);
        let headings = collect_headings(&root);

        let plugins = Plugins {
            render: RenderPlugins {
                codefence_renderers: self.renderer.plugins_map(),
                ..Default::default()
            },
        };
        let mut html = String::new();
        comrak::format_html_with_plugins(root, &self.options, &mut html, &plugins)
            .expect("infallible string write");
        let html = apply_custom_heading_ids(&html, &headings);
        let mut html = replace_toc(&html, &headings);
        if self.lazy_images {
            html = html.replace("<img ", "<img loading=\"lazy\" ");
        }
        // comrak emits raw TeX inside data-math-style spans; delimit it
        // so MathJax's standard page scan picks it up
        let has_math = html.contains("data-math-style");
        if self.math && has_math {
            static INLINE: OnceLock<regex::Regex> = OnceLock::new();
            static DISPLAY: OnceLock<regex::Regex> = OnceLock::new();
            let inline = INLINE.get_or_init(|| regex::Regex::new(r#"(<span data-math-style="inline">)(.*?)(</span>)"#).unwrap());
            let display = DISPLAY.get_or_init(|| regex::Regex::new(r#"(?s)(<span data-math-style="display">)(.*?)(</span>)"#).unwrap());
            let html = inline.replace_all(&html, "$1\\($2\\)$3");
            let html = display.replace_all(&html, "$1\\[$2\\]$3");
            return Ok(RenderedPage {
                html: html.into_owned(),
                headings,
                has_math: true,
            });
        }
        Ok(RenderedPage { html, headings, has_math })
    }
}

/// Rewrite relative/`.md` links against the page's location into
/// canonical `/page/` URLs (anchors preserved); external links and
/// unknown targets pass through.
fn rewrite_links(root: &comrak::Node<'_>, page: &Page, content: &Content) {
    for node in root.descendants() {
        let replacement = {
            let data = node.data.borrow();
            match &data.value {
                NodeValue::Link(link) => resolve_relative(&link.url, &page.rel, content).map(|url| {
                    NodeValue::Link(Box::new(NodeLink { url, ..(**link).clone() }))
                }),
                _ => None,
            }
        };
        if let Some(value) = replacement {
            node.data.borrow_mut().value = value;
        }
    }
}

/// `./x.md`, `../y.md`, `./dir/`, `../` → canonical URL of the target
/// page when one exists. Absolute (`/…`), external, anchor-only and
/// unknown targets return None (left untouched).
pub fn resolve_relative(url: &str, page_rel: &str, content: &Content) -> Option<String> {
    if url.is_empty()
        || url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("mailto:")
        || url.starts_with('#')
        || url.starts_with('/')
    {
        return None;
    }
    let (path, frag) = match url.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (url, None),
    };
    if path.is_empty() {
        return None;
    }
    let base_dir = Path::new(page_rel).parent().unwrap_or(Path::new(""));
    let joined = base_dir.join(path);
    let mut segs: Vec<String> = Vec::new();
    for seg in joined.components() {
        match seg {
            std::path::Component::Normal(s) => {
                segs.push(s.to_string_lossy().into_owned());
            }
            std::path::Component::ParentDir => {
                segs.pop();
            }
            _ => {}
        }
    }
    let mut target = segs.join("/");
    // VitePress cleanUrls: `.md` (and `.html`) internal links resolve to
    // the page URL
    for suffix in [".md", ".html"] {
        if let Some(stripped) = target.strip_suffix(suffix) {
            target = stripped.to_string();
        }
    }
    let target = target.trim_end_matches('/');
    let frag = frag.map(|f| format!("#{f}")).unwrap_or_default();
    if target.is_empty() {
        return content.get("/").map(|_| format!("/{frag}"));
    }
    if let Some(canonical) = content.get(&format!("/{target}/")).or_else(|| content.get(&format!("/{target}"))) {
        return Some(format!("{}{}", canonical.url, frag));
    }
    None
}

/// Collect headings for the outline, mirroring comrak's render-time
/// anchorization (same Anchorizer, same document order).
fn collect_headings(root: &comrak::Node<'_>) -> Vec<Heading> {
    let mut out = Vec::new();
    let mut anchorizer = Anchorizer::new();
    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::Heading(nh) = &data.value {
            let raw = node.collect_text();
            let rendered_id = anchorizer.anchorize(&raw);
            let (text, custom) = split_heading_anchor(&raw);
            let id = custom.clone().unwrap_or_else(|| rendered_id.clone());
            out.push(Heading { level: nh.level, id, text, rendered_id });
        }
    }
    out
}

/// `Heading {#my-id}` → ("Heading", Some("my-id")).
fn split_heading_anchor(text: &str) -> (String, Option<String>) {
    if text.ends_with('}')
        && let Some(i) = text.rfind("{#") {
            return (
                text[..i].trim_end().to_string(),
                Some(text[i + 2..text.len() - 1].to_string()),
            );
        }
    (text.to_string(), None)
}

/// Swap each heading's rendered slug for its `{#custom}` id and strip the
/// literal attribute from the rendered text (VitePress heading anchors).
/// Heading order in `html` matches `headings` (both are document order).
fn apply_custom_heading_ids(html: &str, headings: &[Heading]) -> String {
    if !headings.iter().any(|h| h.id != h.rendered_id) {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    let mut iter = headings.iter();
    while let Some(open) = rest.find("<h") {
        let level = rest[open + 2..].chars().next().unwrap_or(' ');
        if !level.is_ascii_digit() || level > '6' {
            out.push_str(&rest[..open + 3]);
            rest = &rest[open + 3..];
            continue;
        }
        let close_tag = format!("</h{}>", level);
        let Some(close) = rest[open..].find(&close_tag) else {
            out.push_str(rest);
            return out;
        };
        let segment = &rest[open..open + close + close_tag.len()];
        let mut fixed = segment.to_string();
        if let Some(h) = iter.next()
            && h.id != h.rendered_id {
                fixed = fixed
                    .replace(&format!("id=\"{}\"", h.rendered_id), &format!("id=\"{}\"", h.id))
                    .replace(&format!("href=\"#{}\"", h.rendered_id), &format!("href=\"#{}\"", h.id));
                fixed = fixed.replace(&format!(" {{#{}}}", h.id), "");
            }
        out.push_str(&rest[..open]);
        out.push_str(&fixed);
        rest = &rest[open + close + close_tag.len()..];
    }
    out.push_str(rest);
    out
}

/// `[[toc]]` — the placeholder comment (emitted by the preprocessor)
/// becomes a nested table of contents, VitePress-shaped
/// (`<nav class="table-of-contents">`).
fn replace_toc(html: &str, headings: &[Heading]) -> String {
    const MARK: &str = "<!--gd-toc-->";
    if !html.contains(MARK) {
        return html.to_string();
    }
    let mut body = String::from("<ul>");
    let mut stack: Vec<u8> = Vec::new();
    for h in headings {
        if stack.is_empty() {
            body.push_str("<li>");
            stack.push(h.level);
        } else if h.level > *stack.last().unwrap() {
            body.push_str("<ul><li>");
            stack.push(h.level);
        } else {
            while stack.len() > 1 && h.level < *stack.last().unwrap() {
                body.push_str("</li></ul>");
                stack.pop();
            }
            body.push_str("</li><li>");
            *stack.last_mut().unwrap() = h.level;
        }
        body.push_str(&format!("<a href=\"#{}\">{}</a>", h.id, preprocess::escape_text(&h.text)));
    }
    while stack.len() > 1 {
        body.push_str("</li></ul>");
        stack.pop();
    }
    if !stack.is_empty() {
        body.push_str("</li>");
    }
    body.push_str("</ul>");
    let toc = format!("<nav class=\"table-of-contents\">{body}</nav>");
    // both bare and inside the paragraph comrak wraps it in
    html.replace(&format!("<p>{MARK}</p>"), &toc).replace(MARK, &toc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_link_resolution() {
        let mut content = Content::default();
        for (url, rel) in [
            ("/", "index.md"),
            ("/guide/what/", "guide/what.md"),
            ("/reference/api/", "reference/api.md"),
        ] {
            content.pages.push(crate::content::Page {
                rel: rel.into(),
                url: url.into(),
                title: String::new(),
                front: Default::default(),
                body: String::new(),
                modified: None,
                src: rel.into(),
            });
        }
        content.by_url = content
            .pages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.url.clone(), i))
            .collect();

        // sibling .md link + anchor
        assert_eq!(
            resolve_relative("./what.md#section", "guide/intro.md", &content),
            Some("/guide/what/#section".into())
        );
        // up-and-over
        assert_eq!(
            resolve_relative("../reference/api.md", "guide/intro.md", &content),
            Some("/reference/api/".into())
        );
        // directory links: "../" climbs to the root page
        assert_eq!(resolve_relative("../", "guide/intro.md", &content), Some("/".into()));
        // "./" stays in the current dir — no /guide/ page in this fixture
        assert_eq!(resolve_relative("./", "guide/what.md", &content), None);
        // unknown target untouched
        assert_eq!(resolve_relative("./missing.md", "guide/intro.md", &content), None);
        // absolute/external untouched
        assert_eq!(resolve_relative("/api/", "guide/intro.md", &content), None);
        assert_eq!(resolve_relative("https://x.y/z", "guide/intro.md", &content), None);
    }
}

