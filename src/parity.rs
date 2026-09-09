//! Upstream parity fingerprints: a mechanical basis for tracking VitePress
//! releases. We render landmark structure (nav, sidebar, outline, doc
//! footer, hero, features, …) of a built site into a stable JSON
//! fingerprint per page. `parity snapshot` pins the deployed upstream
//! (vitepress.dev) into `parity/upstream.json`; `parity check` re-extracts
//! the same landmarks from our build and reports every unexplained
//! divergence; `parity diff` shows what an upstream refresh changed.
//!
//! The DOM intentionally diverges from upstream (Tailwind utilities,
//! Alpine.js, tree-sitter code spans), so fingerprints compare *landmarks*
//! — text, hrefs, order, presence — never class names or markup. State
//! that only exists after client hydration (outline items on the deployed
//! site, active/collapsed flags) is recorded as absent rather than
//! guessed: `null` means "not in the source HTML", and `check` skips
//! fields the upstream side cannot see.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use scraper::CaseSensitivity;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Fingerprint model
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Link {
    pub text: String,
    pub href: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NavTitle {
    pub text: String,
    pub href: String,
    pub logo_src: Option<String>,
}

/// One top-level navbar entry: a plain link or a flyout group.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum NavItem {
    Link {
        text: String,
        href: String,
    },
    Group {
        text: String,
        items: Vec<Link>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SidebarEntry {
    /// "group" (section header, no link) or "link".
    pub kind: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    /// Nesting depth: number of enclosing `ul` elements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PagerItem {
    /// "Previous page" / "Next page".
    pub desc: String,
    pub title: String,
    pub href: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Hero {
    /// The `h1.heading` spans in order (name, tagline text, …).
    pub lines: Vec<String>,
    pub tagline: Option<String>,
    pub actions: Vec<Link>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Feature {
    pub title: String,
    pub details: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PageFingerprint {
    pub title: Option<String>,
    pub h1: Option<H1>,
    pub nav_title: Option<NavTitle>,
    pub nav_items: Vec<NavItem>,
    pub sidebar: Vec<SidebarEntry>,
    pub outline_title: Option<String>,
    /// `None`: the outline is not in the source HTML at all;
    /// `Some(items)`: outline present (possibly empty — e.g. upstream
    /// SSRs the outline shell but hydrates the items client-side).
    pub outline_items: Option<Vec<Link>>,
    pub pager: Vec<PagerItem>,
    pub edit_link: Option<String>,
    /// The `datetime` attribute of the last-updated timestamp. Values are
    /// site-specific (our git log vs upstream's); `check` treats this
    /// field as presence-only.
    pub last_updated: Option<String>,
    /// Paragraph texts of the page footer (message, copyright).
    pub site_footer: Option<Vec<String>>,
    pub hero: Option<Hero>,
    pub features: Option<Vec<Feature>>,
    pub has_search: bool,
    pub has_main: bool,
    /// Sanity guard: counts of block-level content elements inside
    /// `<main>`. `None` when the page has no `<main>` (the home page on
    /// both sides).
    pub block_counts: Option<BTreeMap<String, usize>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct H1 {
    pub text: String,
    pub id: Option<String>,
}

impl PageFingerprint {
    pub fn to_value(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

struct Sels {
    title: Selector,
    h1_heading: Selector,
    main: Selector,
    nav_title: Selector,
    nav_container: Selector,
    sidebar_nav: Selector,
    outline_title: Selector,
    outline_link: Selector,
    outline_container: Selector,
    pager_link: Selector,
    edit_link: Selector,
    last_updated: Selector,
    footer: Selector,
    hero_actions_upstream: Selector,
    hero_actions_local: Selector,
    feature_upstream: Selector,
    feature_local: Selector,
    search: Selector,
}

fn sel(s: &str) -> Selector {
    Selector::parse(s).expect("static CSS selector")
}

impl Sels {
    fn build() -> Self {
        Self {
            title: sel("title"),
            h1_heading: sel("main h1, .vp-doc h1, h1.heading"),
            main: sel("main"),
            nav_title: sel("header .title a"),
            nav_container: sel("header nav[aria-label='Main Navigation']"),
            sidebar_nav: sel("aside.VPSidebar nav"),
            outline_title: sel("#doc-outline-aria-label, .outline-title"),
            outline_link: sel("a.outline-link"),
            outline_container: sel("nav.VPDocAsideOutline"),
            pager_link: sel("div.pager a"),
            edit_link: sel("div.edit-link a"),
            last_updated: sel("p[class*=LastUpdated] time"),
            footer: sel("footer"),
            hero_actions_upstream: sel(".VPHero .actions a"),
            hero_actions_local: sel("h1.heading + p + div a, h1.heading + div a"),
            feature_upstream: sel(".VPFeature article"),
            feature_local: sel("li > div > article"),
            search: sel(".VPNavBarSearch, #VPSearchButton"),
        }
    }
}

/// Collapse whitespace and drop zero-width spaces (upstream header anchors
/// embed one inside `h1`).
fn norm_text(s: &str) -> String {
    let cleaned: String = s.chars().filter(|&c| c != '\u{200b}').collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Upstream uses clean URLs (`/guide/x`), we emit directory URLs
/// (`/guide/x/`); compare both without the trailing slash.
fn norm_href(h: &str) -> String {
    let t = h.trim();
    let t = t.strip_prefix("./").unwrap_or(t);
    if t.len() > 1 {
        t.trim_end_matches('/').to_string()
    } else {
        t.to_string()
    }
}

/// Resolve base-relative links (`./guide/x` — how the upstream home page
/// links its hero actions) against the page URL, then normalize.
fn resolve_href(raw: &str, page_url: &str) -> String {
    let t = raw.trim();
    let t = match t.strip_prefix("./") {
        Some(rest) => {
            let dir = match page_url.rfind('/') {
                Some(0) | None => "/",
                Some(i) => &page_url[..=i],
            };
            format!("{dir}{rest}")
        }
        None => t.to_string(),
    };
    norm_href(&t)
}

fn el_text(el: ElementRef) -> String {
    norm_text(&el.text().collect::<String>())
}

fn has_class(el: &scraper::node::Element, class: &str) -> bool {
    el.has_class(class, CaseSensitivity::CaseSensitive)
}

fn el_href(el: ElementRef) -> Option<String> {
    el.value().attr("href").map(norm_href)
}

fn first<'a>(doc: &'a Html, s: &Selector) -> Option<ElementRef<'a>> {
    doc.select(s).next()
}

/// True when any ancestor of `el` is an element named `name`.
fn has_ancestor_named(el: ElementRef, name: &str) -> bool {
    let mut cur = el.parent();
    while let Some(node) = cur {
        if let Some(p) = ElementRef::wrap(node) {
            if p.value().name() == name {
                return true;
            }
            cur = p.parent();
        } else {
            break;
        }
    }
    false
}

/// Ordered links from the first selector that yields any, with
/// base-relative hrefs resolved against `page_url`.
fn links_from(doc: &Html, sels: &[&Selector], page_url: &str) -> Vec<Link> {
    for s in sels {
        let links: Vec<Link> = doc
            .select(s)
            .filter_map(|a| {
                let href = a
                    .value()
                    .attr("href")
                    .map(|h| resolve_href(h, page_url))?;
                let text = el_text(a);
                (!text.is_empty()).then_some(Link { text, href })
            })
            .collect();
        if !links.is_empty() {
            return links;
        }
    }
    Vec::new()
}

/// Extract the landmark fingerprint from a rendered page.
pub fn extract(html: &str, url: &str) -> PageFingerprint {
    let doc = Html::parse_document(html);
    let s = Sels::build();

    let title = first(&doc, &s.title).map(|t| norm_text(&t.text().collect::<String>()));

    let h1 = first(&doc, &s.h1_heading).map(|h| H1 {
        text: el_text(h),
        id: h.value().attr("id").map(str::to_string),
    });

    let nav_title = first(&doc, &s.nav_title).map(|a| NavTitle {
        text: el_text(a),
        href: el_href(a).unwrap_or_default(),
        logo_src: a
            .select(&sel("img"))
            .next()
            .and_then(|img| img.value().attr("src").map(str::to_string)),
    });

    let nav_items = extract_nav_items(&doc, &s);
    let sidebar = extract_sidebar(&doc, &s);
    let (outline_title, outline_items) = extract_outline(&doc, &s);
    let pager = extract_pager(&doc, &s);
    let edit_link = first(&doc, &s.edit_link).and_then(|a| el_href(a));
    let last_updated = first(&doc, &s.last_updated)
        .and_then(|t| t.value().attr("datetime"))
        // timestamps differ by nature (their git log vs ours); keep the
        // date prefix so diffs still show coarse drift
        .map(|d| d.chars().take(10).collect::<String>());

    let site_footer = doc
        .select(&s.footer)
        .last()
        .map(|f| {
            f.select(&sel("p"))
                .map(|p| el_text(p))
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|texts| !texts.is_empty());

    let hero = first(&doc, &s.h1_heading).and_then(|h1| {
        let is_home = has_class(h1.value(), "heading");
        if !is_home {
            return None;
        }
        let lines: Vec<String> = h1
            .select(&sel("span"))
            .map(el_text)
            .filter(|t| !t.is_empty())
            .collect();
        if lines.is_empty() {
            return None;
        }
        let tagline = doc
            .select(&sel("h1.heading + p"))
            .next()
            .map(el_text)
            .filter(|t| !t.is_empty());
        let mut actions =
            links_from(&doc, &[&s.hero_actions_upstream, &s.hero_actions_local], url);
        actions.dedup_by(|a, b| a.href == b.href);
        Some(Hero {
            lines,
            tagline,
            actions,
        })
    });

    let mut features: Vec<Feature> = Vec::new();
    for article in doc
        .select(&s.feature_upstream)
        .chain(doc.select(&s.feature_local))
    {
        let Some(title) = article.select(&sel("h2")).next().map(el_text) else {
            continue;
        };
        if title.is_empty() || features.iter().any(|f: &Feature| f.title == title) {
            continue;
        }
        let details = article
            .select(&sel("p"))
            .next()
            .map(el_text)
            .unwrap_or_default();
        features.push(Feature { title, details });
    }
    let features = (!features.is_empty()).then_some(features);

    let has_search = first(&doc, &s.search).is_some();
    let main = first(&doc, &s.main);
    let has_main = main.is_some();
    let block_counts = main.map(|m| {
        let mut counts = BTreeMap::new();
        for tag in ["h2", "h3", "h4", "h5", "h6", "pre", "table", "ul", "ol", "blockquote"] {
            let n = m.select(&sel(tag)).count();
            counts.insert(tag.to_string(), n);
        }
        counts
    });

    PageFingerprint {
        title,
        h1,
        nav_title,
        nav_items,
        sidebar,
        outline_title,
        outline_items,
        pager,
        edit_link,
        last_updated,
        site_footer,
        hero,
        features,
        has_search,
        has_main,
        block_counts,
    }
}

/// Walk the first (desktop) navigation: `ul > li` with a direct `a` is a
/// plain link; an `li` wrapping a flyout becomes a group labelled by its
/// button text.
fn extract_nav_items(doc: &Html, s: &Sels) -> Vec<NavItem> {
    let Some(nav) = first(doc, &s.nav_container) else {
        return Vec::new();
    };
    let Some(list) = nav.select(&sel("ul")).next() else {
        return Vec::new();
    };
    let mut items = Vec::new();
    for li in list.children().filter_map(ElementRef::wrap) {
        if li.value().name() != "li" {
            continue;
        }
        if let Some(a) = li.children().filter_map(ElementRef::wrap).find(|c| c.value().name() == "a") {
            if let Some(href) = el_href(a) {
                items.push(NavItem::Link {
                    text: el_text(a),
                    href,
                });
                continue;
            }
        }
        // flyout group
        let label = li
            .select(&sel("button"))
            .next()
            .map(el_text)
            .filter(|t| !t.is_empty())
            .or_else(|| li.select(&sel(".text")).next().map(el_text))
            .unwrap_or_default();
        let group_items: Vec<Link> = li
            .select(&sel("a"))
            .filter_map(|a| {
                let href = el_href(a)?;
                let text = el_text(a);
                (!text.is_empty()).then_some(Link { text, href })
            })
            .collect();
        if !label.is_empty() || !group_items.is_empty() {
            items.push(NavItem::Group {
                text: label,
                items: group_items,
            });
        }
    }
    items
}

/// Document-order walk of the sidebar: section headers are `.text`
/// elements with no `Link` ancestor; everything that *is* a link becomes
/// a link entry tagged with its `ul` nesting depth.
fn extract_sidebar(doc: &Html, s: &Sels) -> Vec<SidebarEntry> {
    let Some(nav) = first(doc, &s.sidebar_nav) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for node in nav.descendants() {
        let Some(el) = ElementRef::wrap(node) else {
            continue;
        };
        let name = el.value().name();
        if name == "a" {
            let Some(href) = el_href(el) else { continue };
            if href.starts_with('#') {
                continue;
            }
            let text = el_text(el);
            if text.is_empty() {
                continue;
            }
            let depth = {
                let mut d = 0;
                let mut cur = el.parent();
                while let Some(node) = cur {
                    if let Some(p) = ElementRef::wrap(node) {
                        if p.value().name() == "ul" {
                            d += 1;
                        }
                        cur = p.parent();
                    } else {
                        break;
                    }
                }
                d
            };
            entries.push(SidebarEntry {
                kind: "link".into(),
                text,
                href: Some(href),
                depth: Some(depth),
            });
        } else if has_class(el.value(), "text") {
            let inside_link = has_ancestor_named(el, "a");
            let parent_is_item = el
                .parent()
                .and_then(ElementRef::wrap)
                .map(|p| has_class(p.value(), "item"))
                .unwrap_or(false);
            if inside_link || !parent_is_item {
                continue;
            }
            let text = el_text(el);
            if !text.is_empty() {
                entries.push(SidebarEntry {
                    kind: "group".into(),
                    text,
                    href: None,
                    depth: None,
                });
            }
        }
    }
    entries
}

/// The deployed upstream SSRs the outline shell but hydrates the items
/// client-side, so items are recorded whenever an outline container
/// exists — empty is a meaningful value, not a failure.
fn extract_outline(doc: &Html, s: &Sels) -> (Option<String>, Option<Vec<Link>>) {
    let container = first(doc, &s.outline_container);
    let has_local = container.is_none() && first(doc, &s.outline_link).is_some();
    if container.is_none() && !has_local {
        return (None, None);
    }
    let title = first(doc, &s.outline_title).map(el_text).filter(|t| !t.is_empty());
    // scope the items to the aside outline only: the local-nav dropdown
    // repeats them (upstream hydrates that copy client-side, we render
    // both server-side)
    let items: Vec<Link> = match container {
        Some(c) => c
            .select(&s.outline_link)
            .filter_map(|a| {
                let href = el_href(a)?;
                let text = el_text(a);
                (!text.is_empty()).then_some(Link { text, href })
            })
            .collect(),
        None => Vec::new(),
    };
    (title, Some(items))
}

fn extract_pager(doc: &Html, s: &Sels) -> Vec<PagerItem> {
    doc.select(&s.pager_link)
        .filter_map(|a| {
            let href = el_href(a)?;
            let spans: Vec<String> = a
                .select(&sel("span"))
                .map(el_text)
                .filter(|t| !t.is_empty())
                .collect();
            let desc = spans.first().cloned().unwrap_or_default();
            let title = spans.get(1).cloned().unwrap_or_default();
            (!href.is_empty()).then_some(PagerItem { desc, title, href })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Baseline (snapshot) files
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Baseline {
    #[serde(default)]
    pub meta: BTreeMap<String, Value>,
    pub pages: BTreeMap<String, Value>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Delta {
    pub page: String,
    pub path: String,
    #[serde(default)]
    pub upstream: Option<Value>,
    #[serde(default)]
    pub ours: Option<Value>,
    pub reason: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Deltas {
    #[serde(default)]
    pub deltas: Vec<Delta>,
}

impl Deltas {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("reading deltas {}", path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
    }

    pub fn covers(&self, page: &str, path: &str, up: &Value, ours: &Value) -> bool {
        self.deltas.iter().any(|d| {
            if d.page != page {
                return false;
            }
            let path_match = path == d.path
                || path.starts_with(&format!("{}.", d.path))
                || path.starts_with(&format!("{}[", d.path));
            if !path_match {
                return false;
            }
            if let Some(want) = &d.upstream {
                if want != up {
                    return false;
                }
            }
            if let Some(want) = &d.ours {
                if want != ours {
                    return false;
                }
            }
            true
        })
    }
}

/// Slug used for both cache files and `pages.txt` entries.
pub fn slug_for(url: &str) -> String {
    if url == "/" {
        "home".to_string()
    } else {
        url.trim_matches('/').replace('/', "_")
    }
}

/// File inside a built site directory that renders `url`.
pub fn site_file_for(site_dir: &Path, url: &str) -> PathBuf {
    if url == "/" {
        site_dir.join("index.html")
    } else {
        site_dir.join(format!("{}/index.html", url.trim_matches('/')))
    }
}

/// Build a baseline from a directory of fetched upstream HTML files
/// (named by [`slug_for`]) plus the `pages.txt` list.
pub fn snapshot(cache_dir: &Path, pages: &[String], meta: BTreeMap<String, Value>) -> Result<Baseline> {
    let mut pages_out = BTreeMap::new();
    for url in pages {
        let file = cache_dir.join(format!("{}.html", slug_for(url)));
        let html = std::fs::read_to_string(&file)
            .with_context(|| format!("reading cached page {}", file.display()))?;
        let fp = extract(&html, url);
        pages_out.insert(url.clone(), fp.to_value()?);
    }
    Ok(Baseline { meta, pages: pages_out })
}

pub fn load_baseline(path: &Path) -> Result<Baseline> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading baseline {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

pub fn parse_pages(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

// ---------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Mismatch {
    pub page: String,
    pub path: String,
    pub upstream: Value,
    pub ours: Value,
}

impl std::fmt::Display for Mismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}: upstream {} != ours {}",
            self.page,
            self.path,
            brief(&self.upstream),
            brief(&self.ours)
        )
    }
}

fn brief(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::String(s) => format!("{s:?}"),
        other => {
            let s = other.to_string();
            if s.len() > 80 {
                format!("{}…", &s[..80])
            } else {
                s
            }
        }
    }
}

/// Fields where an empty upstream list carries no information (the
/// deployed site SSRs the shell and hydrates the content client-side):
/// `check` skips them when the upstream side is empty, `diff` still
/// reports them.
const SKIP_WHEN_UPSTREAM_EMPTY: &[&str] = &["outline_items"];

/// Fields whose *value* legitimately differs between sites (timestamps
/// generated from each repo's own git history): compare presence only.
const PRESENCE_ONLY: &[&str] = &["last_updated"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// Ours must explain every non-null upstream landmark (nulls on the
    /// upstream side mean "not visible in source HTML" and are skipped).
    Check,
    /// Report every change between two baselines, including to/from null.
    Diff,
}

pub fn compare(page: &str, up: &Value, ours: &Value, mode: Mode, out: &mut Vec<Mismatch>) {
    walk(page, String::new(), up, ours, mode, out);
}

fn is_presence_only(path: &str) -> bool {
    PRESENCE_ONLY.iter().any(|f| {
        path == *f
            || path.ends_with(&format!(".{f}"))
            || path.ends_with(&format!(".{f}[]"))
    })
}

fn walk(page: &str, path: String, up: &Value, ours: &Value, mode: Mode, out: &mut Vec<Mismatch>) {
    match (up, ours) {
        (Value::Object(uo), Value::Object(oo)) => {
            for (k, uv) in uo {
                let child = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                match oo.get(k) {
                    Some(ov) => walk(page, child, uv, ov, mode, out),
                    None => {
                        if mode == Mode::Diff {
                            out.push(Mismatch {
                                page: page.into(),
                                path: format!("{child} (removed upstream)"),
                                upstream: uv.clone(),
                                ours: Value::Null,
                            });
                        }
                    }
                }
            }
            if mode == Mode::Diff {
                for (k, ov) in oo {
                    if !uo.contains_key(k) {
                        let child = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                        out.push(Mismatch {
                            page: page.into(),
                            path: format!("{child} (added by us)"),
                            upstream: Value::Null,
                            ours: ov.clone(),
                        });
                    }
                }
            }
        }
        (Value::Array(uarr), Value::Array(oarr)) => {
            if mode == Mode::Check
                && uarr.is_empty()
                && SKIP_WHEN_UPSTREAM_EMPTY.iter().any(|f| *f == path)
            {
                return;
            }
            if uarr.len() != oarr.len() {
                // only worth its own line when the whole list differs in
                // size; element-level noise is folded into this one report
                out.push(Mismatch {
                    page: page.into(),
                    path: format!("{path}[]"),
                    upstream: Value::String(format!("{} items", uarr.len())),
                    ours: Value::String(format!("{} items", oarr.len())),
                });
                return;
            }
            for (i, (u, o)) in uarr.iter().zip(oarr).enumerate() {
                walk(page, format!("{path}[{i}]"), u, o, mode, out);
            }
        }
        _ => {
            if up == ours {
                return;
            }
            if mode == Mode::Check {
                if up.is_null() {
                    return; // upstream cannot see this landmark
                }
                if is_presence_only(&path) && !up.is_null() && !ours.is_null() {
                    return; // both present; values are site-local by nature
                }
            }
            out.push(Mismatch {
                page: page.into(),
                path,
                upstream: up.clone(),
                ours: ours.clone(),
            });
        }
    }
}

/// Run the parity check over a built site directory. Returns every
/// unexplained divergence.
pub fn check(site_dir: &Path, baseline: &Baseline, deltas: &Deltas) -> Vec<Mismatch> {
    let mut all = Vec::new();
    for (url, up) in &baseline.pages {
        let file = site_file_for(site_dir, url);
        let Ok(html) = std::fs::read_to_string(&file) else {
            all.push(Mismatch {
                page: url.clone(),
                path: "page".into(),
                upstream: Value::String(file.display().to_string()),
                ours: Value::Null,
            });
            continue;
        };
        let ours = extract(&html, url).to_value().expect("fingerprint serializes");
        let mut out = Vec::new();
        compare(url, up, &ours, Mode::Check, &mut out);
        out.retain(|m| !deltas.covers(&m.page, &m.path, &m.upstream, &m.ours));
        all.extend(out);
    }
    all
}

/// Human-readable report of what changed between two baselines.
pub fn diff(old: &Baseline, new: &Baseline) -> Vec<Mismatch> {
    let mut urls: Vec<&String> = old.pages.keys().chain(new.pages.keys()).collect();
    urls.sort();
    urls.dedup();
    let mut all = Vec::new();
    for url in urls {
        match (old.pages.get(url), new.pages.get(url)) {
            (Some(u), Some(o)) => {
                let page = url.clone();
                compare(&page, u, o, Mode::Diff, &mut all);
            }
            (Some(_), None) => all.push(Mismatch {
                page: url.clone(),
                path: "page".into(),
                upstream: Value::String("present".into()),
                ours: Value::String("removed".into()),
            }),
            (None, Some(_)) => all.push(Mismatch {
                page: url.clone(),
                path: "page".into(),
                upstream: Value::String("absent".into()),
                ours: Value::String("added".into()),
            }),
            (None, None) => unreachable!(),
        }
    }
    all
}
