//! The markdown pipeline: preprocess (includes/containers/fences/badges)
//! → comrak parse (GFM + alerts + footnotes + GitHub-style heading ids)
//! → AST fixups (relative `.md` link rewriting to canonical page URLs)
//! → HTML with the `gdcode` highlighter.

pub mod highlight;
pub mod preprocess;
pub mod syntax_theme;

use std::path::Path;

use comrak::nodes::{NodeLink, NodeValue};
use comrak::options::{Plugins, RenderPlugins};
use comrak::{Anchorizer, Arena, Options};
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

use crate::config::Markdown as MarkdownConfig;
use crate::content::{Content, Page};

/// One outline heading with its render-matching anchor id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    /// The final anchor id: the `{#custom}` attribute when present, else
    /// the VitePress slug (`slugify` below — the mdit-vue algorithm that
    /// decides every upstream anchor, not comrak's GitHub anchorizer).
    pub id: String,
    pub text: String,
    /// The slug id comrak rendered (the `<hN id>` post-processing swaps
    /// `rendered_id` for `id`); pairing against comrak's own output keeps
    /// raw-HTML headings from consuming an id mapping (they never get one
    /// from comrak).
    pub rendered_id: String,
    /// The literal `{#…}` attribute the author wrote, before
    /// deduplication. Kept so the rendered-text strip can remove it
    /// even when the final id was suffixed (`## B {#c}` → `c-1`) or
    /// happens to equal the slug (`## {#foo}` → `foo`).
    pub custom_attr: Option<String>,
    /// Token-rule text (no HTML tags / badge text / shortcode emoji,
    /// marker stripped, trimmed) — the `Permalink to “…”` aria-label
    /// body, matching upstream's permalink title computation.
    pub permalink_title: String,
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
    #[error(
        "cannot load syntax theme {value:?}: {detail} (built-in names: the file stems of all {} vendored Helix themes, e.g. github_light, catppuccin_mocha; a value ending in .toml is resolved as a Helix theme file relative to the site dir)",
        syntax_theme::helix_count()
    )]
    ThemeLoad { value: String, detail: String },
    #[error(transparent)]
    Preprocess(#[from] preprocess::PreprocessError),
}

/// Reusable engine: comrak options + highlighter themes.
pub struct MarkdownEngine {
    options: Options<'static>,
    renderer: highlight::GdCodeRenderer,
    light: syntax_theme::SyntaxTheme,
    dark: syntax_theme::SyntaxTheme,
    lazy_images: bool,
    math: bool,
    config_container: crate::config::ContainerOptions,
    /// Site `base` (`/` or `/sub/`), prefixed onto root-absolute content
    /// links and images the way VitePress's link plugin does.
    base: String,
}

impl MarkdownEngine {
    pub fn new(
        markdown: &MarkdownConfig,
        code: &crate::config::SyntaxHighlight,
        base_dir: &std::path::Path,
        base: &str,
    ) -> Result<Self, MarkdownError> {
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
        if markdown.math {
            ext.math_dollars = true;
        }
        // ids on every heading — comrak's GitHub anchorizer renders them
        // (uniqueness + pairing key), then apply_heading_rewrites swaps
        // in the VitePress `slugify`-compatible id, `tabindex="-1"` and
        // the `header-anchor` permalink shape.
        ext.header_id_prefix = Some(String::new());
        // VitePress renders raw HTML in markdown; our preprocessing emits
        // trusted wrappers (containers, badges) as raw HTML too.
        options.render.r#unsafe = true;
        options.render.tasklist_classes = true;
        // each [code] value is a vendored Helix theme's file stem
        // or a path to a Helix TOML theme file (relative to base_dir)
        let load = |value: &str| -> Result<syntax_theme::SyntaxTheme, MarkdownError> {
            if value.ends_with(".toml") {
                let path = base_dir.join(value);
                syntax_theme::load(&path).map_err(|e| MarkdownError::ThemeLoad {
                    value: value.to_string(),
                    detail: e.to_string(),
                })
            } else {
                syntax_theme::helix_builtin(value).ok_or_else(|| {
                    // a dashed name that exists underscored is almost
                    // always a leftover of the removed vendored pair
                    // (github-light → github_light)
                    let hint = match value.replace('-', "_") {
                        u if u != value && syntax_theme::helix_builtin(&u).is_some() => {
                            format!(" — did you mean {u:?}?")
                        }
                        _ => String::new(),
                    };
                    MarkdownError::ThemeLoad {
                        value: value.to_string(),
                        detail: format!("not a built-in name{hint}"),
                    }
                })
            }
        };
        let light = load(&code.light)?;
        let dark = load(&code.dark)?;
        Ok(Self {
            options,
            renderer: highlight::GdCodeRenderer {
                options: highlight::RendererOptions {
                    copy_button: markdown.code_copy_button,
                    line_numbers: markdown.line_numbers,
                },
            },
            light,
            dark,
            lazy_images: markdown.image.lazy_loading,
            math: markdown.math,
            config_container: markdown.container.clone(),
            base: base.to_string(),
        })
    }

    /// The dual-theme `syntax.css` content.
    pub fn syntax_css(&self) -> String {
        highlight::syntax_css(&self.light, &self.dark)
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
        let pre = preprocess::Preprocess {
            site_root,
            content_dir,
            container: self.config_container.clone(),
        };
        let md = pre.run(&page.body, &page.rel)?;

        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &md, &self.options);
        rewrite_links(&root, page, content, &self.base);
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
        let html = resolve_marked_urls(html, page, content, &self.base);
        let html = self.render_titles(&html, page, content);
        let html = apply_heading_rewrites(&html, &headings);
        let html = replace_toc(&html, &headings);
        let mut html = upstream_footnotes(&html);
        if self.lazy_images {
            html = html.replace("<img ", "<img loading=\"lazy\" ");
        }
        // comrak emits raw TeX inside data-math-style spans; delimit it
        // so MathJax's standard page scan picks it up
        let has_math = html.contains("data-math-style");
        if self.math && has_math {
            static INLINE: OnceLock<regex::Regex> = OnceLock::new();
            static DISPLAY: OnceLock<regex::Regex> = OnceLock::new();
            let inline = INLINE.get_or_init(|| {
                regex::Regex::new(r#"(<span data-math-style="inline">)(.*?)(</span>)"#).unwrap()
            });
            let display = DISPLAY.get_or_init(|| {
                regex::Regex::new(r#"(?s)(<span data-math-style="display">)(.*?)(</span>)"#)
                    .unwrap()
            });
            let html = inline.replace_all(&html, "$1\\($2\\)$3");
            let html = display.replace_all(&html, "$1\\[$2\\]$3");
            return Ok(RenderedPage {
                html: html.into_owned(),
                headings,
                has_math: true,
            });
        }
        Ok(RenderedPage {
            html,
            headings,
            has_math,
        })
    }
}

impl MarkdownEngine {
    /// Container titles arrive as escaped text in elements marked with
    /// `INLINE_MARK` (the preprocessor runs before comrak parses); render
    /// each as inline markdown with the page's own options and link
    /// rules, as upstream's md.renderInline does. A title that does not
    /// parse as one paragraph (`# x`, `1. x`) keeps its literal text.
    fn render_titles(&self, html: &str, page: &Page, content: &Content) -> String {
        if !html.contains(preprocess::INLINE_MARK) {
            return html.to_string();
        }
        static TITLE: OnceLock<regex::Regex> = OnceLock::new();
        // titles are escaped text: no `<` before the closing tag
        let title = TITLE.get_or_init(|| {
            regex::Regex::new(
                r#"(<summary|<p class="custom-block-title") data-gd-inline>([^<]*)(</summary>|</p>)"#,
            )
            .unwrap()
        });
        title
            .replace_all(html, |c: &regex::Captures| {
                let inline = self
                    .inline_html(&unescape_minimal(&c[2]), page, content)
                    .unwrap_or_else(|| c[2].to_string());
                format!("{}>{inline}{}", &c[1], &c[3])
            })
            .into_owned()
    }

    /// `md` rendered as inline markdown, or `None` when it does not parse
    /// as a single paragraph.
    fn inline_html(&self, md: &str, page: &Page, content: &Content) -> Option<String> {
        let md = preprocess::rewrite_badges(md);
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &md, &self.options);
        let mut blocks = root.children();
        let first = blocks.next()?;
        if blocks.next().is_some() || !matches!(first.data.borrow().value, NodeValue::Paragraph) {
            return None;
        }
        rewrite_links(&root, page, content, &self.base);
        let mut html = String::new();
        comrak::format_html(root, &self.options, &mut html).expect("infallible string write");
        let html = resolve_marked_urls(html, page, content, &self.base);
        Some(
            html.strip_prefix("<p>")?
                .strip_suffix("</p>\n")?
                .to_string(),
        )
    }
}

/// comrak's footnote HTML reshaped into what upstream's
/// @mdit/plugin-footnote emits: `[n]` reference text (`[n:k]` for the
/// k-th repeat), ids by number (`footnote1`, `footnote-ref1:1`) instead of
/// label, the separator rule and list classes, and `↩︎` backrefs — the
/// U+FE0E keeps the arrow a text glyph, where a bare U+21A9 can draw as
/// an emoji on Apple platforms. Pages without footnotes pass through.
fn upstream_footnotes(html: &str) -> String {
    if !html.contains("data-footnote-ref") {
        return html.to_string();
    }
    static REF: OnceLock<regex::Regex> = OnceLock::new();
    static ITEM: OnceLock<regex::Regex> = OnceLock::new();
    static BACKREF: OnceLock<regex::Regex> = OnceLock::new();
    let reference = REF.get_or_init(|| {
        regex::Regex::new(
            r#"<sup class="footnote-ref"><a href="\#fn-([^"]*)" id="fnref-([^"]*)" data-footnote-ref>(\d+)</a></sup>"#,
        )
        .unwrap()
    });
    // label → footnote number, and comrak reference id → upstream tag
    let mut number = std::collections::HashMap::new();
    let mut tag_of = std::collections::HashMap::new();
    let html = reference.replace_all(html, |c: &regex::Captures| {
        let (label, id, n) = (&c[1], &c[2], &c[3]);
        // comrak ids the m-th reference `label-m` (m ≥ 2); upstream
        // tags it `n:(m-1)`
        let tag = match id.strip_prefix(label).and_then(|rest| rest.strip_prefix('-')) {
            None => n.to_string(),
            Some(m) => {
                let m: u32 = m.parse().expect("comrak numbers repeat references");
                format!("{n}:{}", m - 1)
            }
        };
        number.insert(label.to_string(), n.to_string());
        tag_of.insert(id.to_string(), tag.clone());
        format!(
            r##"<sup class="footnote-ref"><a href="#footnote{n}">[{tag}]</a><a class="footnote-anchor" id="footnote-ref{tag}"></a></sup>"##
        )
    });
    let html = html.replace(
        "<section class=\"footnotes\" data-footnotes>\n<ol>",
        "<hr class=\"footnotes-sep\"><section class=\"footnotes\"><ol class=\"footnotes-list\">",
    );
    let item = ITEM.get_or_init(|| regex::Regex::new(r#"<li id="fn-([^"]*)">"#).unwrap());
    let html = item.replace_all(&html, |c: &regex::Captures| {
        // comrak renders only referenced footnotes
        let n = &number[&c[1]];
        format!(r#"<li id="footnote{n}" class="footnote-item">"#)
    });
    let backref = BACKREF.get_or_init(|| {
        regex::Regex::new(
            r#"<a href="\#fnref-([^"]*)" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="[^"]*" aria-label="[^"]*">↩(?:<sup class="footnote-ref">\d+</sup>)?</a>"#,
        )
        .unwrap()
    });
    backref
        .replace_all(&html, |c: &regex::Captures| {
            let tag = &tag_of[&c[1]];
            format!(
                "<a href=\"#footnote-ref{tag}\" class=\"footnote-backref\">\u{21a9}\u{fe0e}</a>"
            )
        })
        .into_owned()
}

/// Rewrite relative/`.md` links against the page's location into
/// canonical `/page/` URLs (anchors preserved), then prefix `base` onto
/// every root-absolute link and image URL, as VitePress's link plugin
/// does; external, protocol-relative, anchor-only and mailto URLs and
/// raw HTML pass through.
fn rewrite_links(root: &comrak::Node<'_>, page: &Page, content: &Content, base: &str) {
    for node in root.descendants() {
        let replacement = {
            let data = node.data.borrow();
            match &data.value {
                NodeValue::Link(link) => {
                    let resolved = resolve_relative(&link.url, &page.rel, content)
                        .or_else(|| resolve_root(&link.url, content));
                    let url = resolved.as_deref().unwrap_or(&link.url);
                    let rebased = with_base(base, url);
                    (rebased != link.url).then(|| {
                        NodeValue::Link(Box::new(NodeLink {
                            url: rebased,
                            ..(**link).clone()
                        }))
                    })
                }
                NodeValue::Image(image) => {
                    let rebased = with_base(base, &image.url);
                    (rebased != image.url).then(|| {
                        NodeValue::Image(Box::new(NodeLink {
                            url: rebased,
                            ..(**image).clone()
                        }))
                    })
                }
                _ => None,
            }
        };
        if let Some(value) = replacement {
            node.data.borrow_mut().value = value;
        }
    }
}

/// Anchors and images emitted by the `{attrs}` rewrite are raw HTML
/// before comrak parses, so their URLs bypass the node resolution in
/// `rewrite_links`. They carry a `data-gd-mdlink` marker; resolve them
/// here with the same rules — links: relative `.md` → canonical page
/// URL, then base prefixing (unknown and external targets keep their
/// href); images: base prefixing only — and strip the marker. Pages
/// without the marker pass through untouched.
fn resolve_marked_urls(html: String, page: &Page, content: &Content, base: &str) -> String {
    const MARK: &str = "data-gd-mdlink";
    if !html.contains(MARK) {
        return html;
    }
    static TAG: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static HREF: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static SRC: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let tag = TAG.get_or_init(|| regex::Regex::new(r#"<(a|img)\b([^>]*)>"#).unwrap());
    // anchored on a preceding space (or the start) so an authored
    // `data-href="…"` / `data-src="…"` is not taken for the emitter's own
    let href = HREF.get_or_init(|| regex::Regex::new(r#"(?:^|\s)href="([^"]*)""#).unwrap());
    let src = SRC.get_or_init(|| regex::Regex::new(r#"(?:^|\s)src="([^"]*)""#).unwrap());
    tag.replace_all(&html, |c: &regex::Captures| {
        let attrs = &c[2];
        if !attrs.contains(MARK) {
            return c[0].to_string();
        }
        let is_img = &c[1] == "img";
        let (name, attr) = if is_img { ("src", src) } else { ("href", href) };
        let raw = attr
            .captures(attrs)
            .map(|h| unescape_minimal(&h[1]))
            .unwrap_or_default();
        let url = if is_img {
            with_base(base, &raw)
        } else {
            resolve_relative(&raw, &page.rel, content)
                .or_else(|| resolve_root(&raw, content))
                .map_or_else(|| with_base(base, &raw), |r| with_base(base, &r))
        };
        // the emitters write the marker with a leading space; take that
        // space with it so no double space is left between attributes
        let unmarked = attrs.replace(&format!(" {MARK}"), "").replace(MARK, "");
        // parse_attrs emits quoted pairs only, so every `name="…"` (the
        // emitter's and any authored one) is gone after this; `rest` is
        // the remaining attributes verbatim
        let rest = attr.replace_all(&unmarked, "");
        let rest = rest.trim();
        format!(
            r#"<{} {name}="{}"{}>"#,
            &c[1],
            preprocess::escape_text(&url),
            if rest.is_empty() {
                String::new()
            } else {
                format!(" {rest}")
            }
        )
    })
    .into_owned()
}

/// Reverse the minimal attribute escaping the raw-anchor emitters apply
/// to the intermediate href, so resolution sees the author's URL.
fn unescape_minimal(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Join the site `base` onto a root-absolute URL (`/guide/` → `/base/guide/`);
/// anything else — relative, external, protocol-relative (`//host`),
/// anchor, mailto — is returned unchanged.
pub fn with_base(base: &str, url: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.is_empty() || !url.starts_with('/') || url.starts_with("//") {
        return url.to_string();
    }
    format!("{base}{url}")
}

/// `./x.md`, `../y.md`, `./dir/`, `../` → canonical URL of the target
/// page when one exists. Absolute (`/…`), external, anchor-only and
/// unknown targets return None (left untouched).
/// `/guide/a`, `/guide/a.md`, `/guide/a.html`: a root-absolute link that
/// names a page resolves to that page's URL (VitePress cleanUrls), so a
/// directory-style site never emits the slash-less form a static host
/// cannot serve. Links that name no page are returned unchanged (`None`).
pub fn resolve_root(url: &str, content: &Content) -> Option<String> {
    let rest = url.strip_prefix('/')?;
    if rest.starts_with('/') {
        return None; // protocol-relative
    }
    resolve_relative(rest, "index.md", content)
}

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
    // source-space resolution: links name source files, pages carry the
    // (possibly rewritten) URL
    for cand in [
        format!("{target}.md"),
        format!("{target}/index.md"),
        target.to_string(),
    ] {
        if let Some(i) = content.by_rel.get(&cand) {
            return Some(format!("{}{}", content.pages[*i].url, frag));
        }
    }
    if target.is_empty()
        && let Some(i) = content.by_rel.get("index.md")
    {
        return Some(format!("{}{frag}", content.pages[*i].url));
    }
    None
}

/// Collect headings for the outline, mirroring comrak's render-time
/// anchorization (same Anchorizer, same document order).
fn collect_headings(root: &comrak::Node<'_>) -> Vec<Heading> {
    // pass 1: gather in document order with comrak-mirroring rendered ids
    let mut anchorizer = Anchorizer::new();
    let mut rows: Vec<(u8, String, Option<String>, String, String, String)> = Vec::new();
    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::Heading(nh) = &data.value {
            let raw = collect_text_with_shortcodes(node);
            let rendered_id = anchorizer.anchorize(&raw);
            let (text, custom) = split_heading_anchor(&raw);
            // the auto slug uses the VitePress token rules (no HTML
            // tokens — this excludes <Badge> text entirely, since badge
            // text lives in an attribute of the raw HTML that becomes the
            // VPBadge span — and no shortcode-expanded emoji), slugified
            // with the mdit-vue algorithm so anchors match vitepress.dev
            let slug_text = split_heading_anchor(&collect_slug_text(node)).0;
            let slug = slugify(&slug_text);
            rows.push((
                nh.level,
                text,
                custom,
                rendered_id,
                slug,
                slug_text.trim().to_string(),
            ));
        }
    }
    // pass 2: every custom id is reserved up front (an auto slug must
    // never take a custom id that appears LATER in the document), then
    // finals are assigned in document order — customs claim theirs
    // (later duplicates suffix), auto slugs dedupe against everything
    let reserved: std::collections::HashSet<String> =
        rows.iter().filter_map(|(_, _, c, ..)| c.clone()).collect();
    let mut used = reserved.clone();
    let mut claimed: std::collections::HashSet<String> = std::collections::HashSet::new();
    let suffix = |base: &str, used: &std::collections::HashSet<String>| -> String {
        let mut n = 1;
        loop {
            let cand = format!("{base}-{n}");
            if !used.contains(&cand) {
                break cand;
            }
            n += 1;
        }
    };
    let mut out = Vec::new();
    for (level, text, custom, rendered_id, slug, permalink_title) in rows {
        let custom_attr = custom.clone();
        let id = match custom {
            Some(c) if !claimed.contains(&c) => {
                claimed.insert(c.clone());
                c
            }
            // upstream's anchor plugin THROWS on a colliding user-defined
            // id; we keep our gentler up-front reservation and suffix
            Some(c) => suffix(&c, &used),
            None if !used.contains(&slug) => slug.clone(),
            None => suffix(&slug, &used), // same -1/-2 scheme as upstream
        };
        used.insert(id.clone());
        out.push(Heading {
            level,
            id,
            text,
            rendered_id,
            custom_attr,
            permalink_title,
        });
    }
    out
}

/// comrak's `collect_text` skips `ShortCode` nodes, which would drop the
/// emoji from outline labels and heading aria-labels ("Emoji 🎉" became
/// "Emoji"); mirror it but append the resolved emoji.
fn collect_text_with_shortcodes<'a>(
    node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>,
) -> String {
    fn walk<'a>(
        node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>,
        out: &mut String,
    ) {
        match &node.data.borrow().value {
            NodeValue::Text(literal) => out.push_str(literal),
            NodeValue::Code(code) => out.push_str(&code.literal),
            NodeValue::LineBreak | NodeValue::SoftBreak => out.push(' '),
            NodeValue::Math(math) => out.push_str(math.literal.trim_end()),
            NodeValue::ShortCode(shortcode) => out.push_str(&shortcode.emoji),
            _ => {
                for child in node.children() {
                    walk(child, out);
                }
            }
        }
    }
    let mut out = String::new();
    walk(node, &mut out);
    out
}

/// Heading text for the anchor slug, mirroring VitePress's
/// `getTokensText` (markdown.ts): only `text` and `code_inline` tokens
/// count — `html_inline` (raw HTML tags, and entirely our `<Badge>`
/// rewrite, whose visible text lives in the markdown source as an HTML
/// attribute) and `emoji` (shortcode-expanded `:tada:`) are dropped, and
/// soft/hard line breaks carry no characters.
fn collect_slug_text<'a>(
    node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>,
) -> String {
    fn walk<'a>(
        node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>,
        out: &mut String,
        skip_badge: &mut bool,
    ) {
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::Text(literal) if !*skip_badge => out.push_str(literal),
                NodeValue::Code(code) if !*skip_badge => out.push_str(&code.literal),
                NodeValue::HtmlInline(html) => {
                    // the badges' wrapping span swallows its inner text
                    // (upstream never has it: it sits in the <Badge> tag's
                    // `text` attribute); anything else's INNER text stays
                    *skip_badge = !*skip_badge && html.contains("class=\"VPBadge");
                }
                NodeValue::ShortCode(_) | NodeValue::SoftBreak | NodeValue::LineBreak => {}
                NodeValue::Math(_) => {}
                _ => walk(child, out, skip_badge),
            }
        }
    }
    let mut out = String::new();
    let mut skip_badge = false;
    walk(node, &mut out, &mut skip_badge);
    out
}

/// The mdit-vue `slugify` (@mdit-vue/shared) — the function behind every
/// upstream heading anchor: NFKD, strip combining marks and control
/// chars, a run of specials/spaces becomes one `-`, dashes collapse and
/// trim, a leading digit gets a `_` prefix, lowercase at the end.
pub fn slugify(text: &str) -> String {
    let is_special = |c: char| {
        // JS `\s` (incl. U+FEFF) plus the literal mdit-vue special set
        c.is_whitespace()
            || c == '\u{feff}'
            || matches!(
                c,
                '~' | '`'
                    | '!'
                    | '@'
                    | '#'
                    | '$'
                    | '%'
                    | '^'
                    | '&'
                    | '*'
                    | '('
                    | ')'
                    | '-'
                    | '_'
                    | '+'
                    | '='
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '|'
                    | '\\'
                    | ';'
                    | ':'
                    | '"'
                    | '\''
                    | '\u{201c}'
                    | '\u{201d}'
                    | '\u{2018}'
                    | '\u{2019}'
                    | '<'
                    | '>'
                    | ','
                    | '.'
                    | '?'
                    | '/'
            )
    };
    let mut out = String::with_capacity(text.len());
    let mut last_dash = false;
    for c in text.nfkd() {
        if ('\u{0300}'..='\u{036f}').contains(&c) || ('\u{0000}'..='\u{001f}').contains(&c) {
            continue;
        }
        if is_special(c) {
            if !last_dash {
                out.push('-');
                last_dash = true;
            }
            continue;
        }
        last_dash = false;
        out.extend(c.to_lowercase());
    }
    let trimmed = out.trim_matches('-');
    if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// `Heading {#my-id}` → ("Heading", Some("my-id")).
fn split_heading_anchor(text: &str) -> (String, Option<String>) {
    if text.ends_with('}')
        && let Some(i) = text.rfind("{#")
    {
        return (
            text[..i].trim_end().to_string(),
            Some(text[i + 2..text.len() - 1].to_string()),
        );
    }
    (text.to_string(), None)
}

/// Rewrite every comrak-rendered heading segment into the upstream
/// form: swap the rendered slug for the final id (custom attribute
/// or parity slug), strip the literal `{#…}` attribute from the heading
/// text, add `tabindex="-1"` (upstream sets it on every anchored
/// heading), and replace comrak's trailing `<a class="anchor">` with
/// upstream's permalink (`class="header-anchor"`, `aria-label="Permalink
/// to “title”"`, zero-width-space body).
/// Pairing is by id, not position: only an `<hN>` segment carrying a
/// heading's rendered slug is its render, so raw HTML headings in the
/// source (allowed by `render.unsafe`) never consume a mapping and one
/// unmatched heading cannot desync the rest.
fn apply_heading_rewrites(html: &str, headings: &[Heading]) -> String {
    // rendered ids are unique (the anchorizer dedups), so a plain list of
    // (rendered → heading) pairs is an unambiguous lookup table. A
    // heading whose custom attr exists also enters the map when its
    // final id equals the rendered slug — there is no id swap to do, but
    // the literal attribute still has to come out of the text.
    let map: Vec<(&str, &Heading)> = headings
        .iter()
        .filter(|h| !h.rendered_id.is_empty())
        .map(|h| (h.rendered_id.as_str(), h))
        .collect();
    if map.is_empty() {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(open) = rest.find("<h") {
        let level = rest[open + 2..].chars().next().unwrap_or(' ');
        if !level.is_ascii_digit() || level > '6' {
            // advance past "<h" plus the whole char: a blind open + 3
            // would split a multi-byte char after <h
            let skip = open + 2 + level.len_utf8();
            out.push_str(&rest[..skip]);
            rest = &rest[skip..];
            continue;
        }
        let close_tag = format!("</h{}>", level);
        let Some(close) = rest[open..].find(&close_tag) else {
            out.push_str(rest);
            return out;
        };
        let segment = &rest[open..open + close + close_tag.len()];
        let mut fixed = segment.to_string();
        if let Some((rendered, heading)) = map
            .iter()
            .find(|(rendered, _)| fixed.contains(&format!("id=\"{rendered}\"")))
        {
            let escaped = preprocess::escape_text(&heading.id);
            fixed = fixed
                .replace(&format!("id=\"{rendered}\""), &format!("id=\"{escaped}\""))
                .replace(
                    &format!("href=\"#{rendered}\""),
                    &format!("href=\"#{escaped}\""),
                );
            // tabindex=-1 on the heading tag itself (first `id=` in the
            // segment is the opener comrak rendered)
            fixed = fixed.replacen(
                &format!("id=\"{escaped}\""),
                &format!("id=\"{escaped}\" tabindex=\"-1\""),
                1,
            );
            // the literal attribute inside the heading TEXT (and the
            // anchor's aria-label, rendered from the same raw text) is
            // entity-escaped by comrak (& → &amp;), so strip the escaped
            // form too. The author-written attr is the thing to strip:
            // the final id may have been suffixed (`## B {#c}` → `c-1`)
            // or equal the slug (`## {#foo}`).
            if let Some(attr) = &heading.custom_attr {
                let attr_escaped = preprocess::escape_text(attr);
                // space-prefixed first, so `B {#c}` doesn't leave a
                // trailing space; the bare form covers `## {#foo}`, whose
                // text node is exactly the attribute
                for literal in [
                    format!(" {{#{attr}}}"),
                    format!(" {{#{attr_escaped}}}"),
                    format!("{{#{attr}}}"),
                    format!("{{#{attr_escaped}}}"),
                ] {
                    fixed = fixed.replace(&literal, "");
                }
            }
            // comrak's tail anchor → upstream's permalink shape
            let marker = format!("<a href=\"#{escaped}\" aria-label=\"Link to heading '");
            if let Some(start) = fixed.find(&marker)
                && let Some(end) = fixed[start..]
                    .find("class=\"anchor\"></a>")
                    .map(|i| start + i + "class=\"anchor\"></a>".len())
            {
                let title = preprocess::escape_text(&heading.permalink_title);
                fixed.replace_range(
                    start..end,
                    &format!(
                        "<a class=\"header-anchor\" href=\"#{escaped}\" aria-label=\"Permalink to \u{201c}{title}\u{201d}\">&#8203;</a>"
                    ),
                );
            }
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
    // VitePress's [[toc]] covers h2–h3 regardless of deeper headings —
    // a fixed ±1 band, so the deep-skip demotion in the list builder
    // never fires here today
    const TOC_LEVELS: (u8, u8) = (2, 3);
    if !html.contains(MARK) {
        return html.to_string();
    }
    let items: Vec<(u8, String)> = headings
        .iter()
        .filter(|h| h.level >= TOC_LEVELS.0 && h.level <= TOC_LEVELS.1)
        .map(|h| {
            (
                h.level,
                format!(
                    "<a href=\"#{}\">{}</a>",
                    preprocess::escape_text(&h.id),
                    preprocess::escape_text(&h.text)
                ),
            )
        })
        .collect();
    let toc = format!(
        "<nav class=\"table-of-contents\">{}</nav>",
        nested_outline_list(&items, "", "")
    );
    // both bare and inside the paragraph comrak wraps it in
    html.replace(&format!("<p>{MARK}</p>"), &toc)
        .replace(MARK, &toc)
}

/// A nested outline list from `(level, link-html)` pairs — a deeper
/// heading opens a child list, a shallower one closes back (VitePress's
/// outline nesting). `outer_attr`/`nested_attr` go inside the `<ul>`
/// tags verbatim (leading space + `class="…"`, or empty).
///
/// A heading that escapes only *part* of the nesting (h2 → h4 → h3)
/// demotes the skipped entry in place: h3 stays inside h2's subtree as
/// a sibling of h4, instead of closing h4's list and landing beside h2.
/// (The pre-fix behavior closed past it, escaped the subtree, and let
/// every later heading inherit the corrupted levels.)
pub(crate) fn nested_outline_list(
    items: &[(u8, String)],
    outer_attr: &str,
    nested_attr: &str,
) -> String {
    let mut html = format!("<ul{outer_attr}>");
    let mut stack: Vec<u8> = Vec::new();
    for (level, link) in items {
        let level = *level;
        if stack.is_empty() {
            html.push_str("<li>");
            stack.push(level);
        } else if level > *stack.last().unwrap() {
            html.push_str(&format!("<ul{nested_attr}><li>"));
            stack.push(level);
        } else {
            // pop only entries the heading fully escapes; stop before
            // skipping one (h2 → h4 → h3 must not close h4's list).
            // Each pop lands on `below >= level`, so `level <= top`
            // holds here: the heading joins the top's own list as a
            // demoted sibling (h4's entry becomes h3), which keeps the
            // stack truthful for every heading after it.
            while stack.len() > 1
                && level < *stack.last().unwrap()
                && level <= stack[stack.len() - 2]
            {
                html.push_str("</li></ul>");
                stack.pop();
            }
            html.push_str("</li><li>");
            *stack.last_mut().unwrap() = level;
        }
        html.push_str(link);
    }
    while stack.len() > 1 {
        html.push_str("</li></ul>");
        stack.pop();
    }
    if !stack.is_empty() {
        html.push_str("</li>");
    }
    html.push_str("</ul>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    fn li(level: u8, label: &str) -> (u8, String) {
        (level, format!("[{label}]"))
    }

    #[test]
    fn nested_outline_list_demotes_a_partially_escaped_level() {
        // h2 → h4 → h3: h3 stays inside h2's subtree, in h4's list
        let html = nested_outline_list(&[li(2, "A"), li(4, "Deep"), li(3, "B")], "", "");
        assert_eq!(
            html,
            "<ul><li>[A]<ul><li>[Deep]</li><li>[B]</li></ul></li></ul>"
        );
    }

    #[test]
    fn nested_outline_list_fully_escaped_levels_close() {
        // h2 → h3 → h4 → h3′ → h2′: each escape closes exactly one list
        let html = nested_outline_list(
            &[li(2, "A"), li(3, "B"), li(4, "C"), li(3, "B2"), li(2, "E")],
            "",
            "",
        );
        assert_eq!(
            html,
            "<ul><li>[A]<ul><li>[B]<ul><li>[C]</li></ul></li><li>[B2]</li></ul></li><li>[E]</li></ul>"
        );
    }

    #[test]
    fn nested_outline_list_attrs_pass_through() {
        let html = nested_outline_list(
            &[li(2, "A"), li(3, "B")],
            " class=\"outer\"",
            " class=\"inner\"",
        );
        assert!(html.starts_with("<ul class=\"outer\">"), "{html}");
        assert!(html.contains("<ul class=\"inner\"><li>"), "{html}");
    }

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
                locale: "root".into(),
            });
        }
        content.by_url = content
            .pages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.url.clone(), i))
            .collect();
        content.by_rel = content
            .pages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.rel.clone(), i))
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
        assert_eq!(
            resolve_relative("../", "guide/intro.md", &content),
            Some("/".into())
        );
        // "./" stays in the current dir — no /guide/ page in this fixture
        assert_eq!(resolve_relative("./", "guide/what.md", &content), None);
        // unknown target untouched
        assert_eq!(
            resolve_relative("./missing.md", "guide/intro.md", &content),
            None
        );
        // absolute/external untouched
        assert_eq!(resolve_relative("/api/", "guide/intro.md", &content), None);
        assert_eq!(
            resolve_relative("https://x.y/z", "guide/intro.md", &content),
            None
        );
    }

    #[test]
    fn custom_heading_ids_survive_raw_html_headings() {
        // raw HTML in markdown is allowed (render.unsafe); a raw <h2> must
        // not consume a heading slot — every {#custom} anchor on the page
        // still applies and no literal attribute leaks into the output
        let engine = MarkdownEngine::new(
            &MarkdownConfig::default(),
            &crate::config::SyntaxHighlight::default(),
            Path::new("."),
            "/",
        )
        .unwrap();
        let page = Page {
            rel: "x.md".into(),
            url: "/x/".into(),
            title: "x".into(),
            front: Default::default(),
            body: "<h2>Raw HTML heading</h2>\n\n# First {#custom-one}\n\n## Second {#custom-two}\n"
                .into(),
            modified: None,
            src: "x.md".into(),
            locale: "root".into(),
        };
        let rendered = engine
            .render(&page, &Content::default(), Path::new("."), Path::new("."))
            .unwrap();
        let html = &rendered.html;
        assert!(html.contains("id=\"custom-one\""), "{html}");
        assert!(html.contains("id=\"custom-two\""), "{html}");
        assert!(!html.contains("{#custom"), "literal anchor leaked: {html}");
        assert!(!html.contains("id=\"first-custom-one\""), "{html}");
        assert!(!html.contains("id=\"second-custom-two\""), "{html}");
    }
}
