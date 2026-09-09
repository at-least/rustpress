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
use syntect::highlighting::Theme;

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
    pub id: String,
    pub text: String,
}

/// The result of rendering one page's markdown body.
#[derive(Debug, Clone, Default)]
pub struct RenderedPage {
    pub html: String,
    pub headings: Vec<Heading>,
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
}

impl MarkdownEngine {
    pub fn new() -> Result<Self, MarkdownError> {
        let mut options = Options::default();
        let ext = &mut options.extension;
        ext.table = true;
        ext.tasklist = true;
        ext.strikethrough = true;
        ext.autolink = true;
        ext.footnotes = true;
        ext.alerts = true;
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
            renderer: highlight::GdCodeRenderer,
            light,
            dark,
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
        let pre = preprocess::Preprocess { site_root, content_dir };
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
        Ok(RenderedPage { html, headings })
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
            let text = node.collect_text();
            let id = anchorizer.anchorize(&text);
            out.push(Heading { level: nh.level, id, text });
        }
    }
    out
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
