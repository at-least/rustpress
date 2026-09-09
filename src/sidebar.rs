//! Sidebar resolution: VitePress's explicit forms (one array, or a
//! path-keyed set of `{ base, items }` sections) plus the auto-derived
//! fallback (one sidebar per top-level content section, ordered by
//! natural file order). Also the flat page order used for prev/next
//! pagers, which VitePress takes from the active sidebar.

use std::collections::BTreeMap;
use std::path::Path;

use crate::config::{Sidebar as SidebarConfig, SidebarItem, SiteConfig};
use crate::content::{natural_cmp, Content};

/// A resolved sidebar node: `url` is absolute and base-free, with a
/// trailing slash when it points at an internal page.
#[derive(Debug, Clone, PartialEq)]
pub struct SidebarNode {
    pub text: String,
    pub url: Option<String>,
    pub children: Vec<SidebarNode>,
    pub collapsed: Option<bool>,
}

/// One sidebar, applying to every URL under `prefix`.
#[derive(Debug, Clone, PartialEq)]
pub struct SidebarTree {
    pub prefix: String,
    pub items: Vec<SidebarNode>,
}

impl SidebarTree {
    /// Does this tree own `url`? (prefix match on path segments)
    fn covers(&self, url: &str) -> bool {
        url == self.prefix || self.prefix == "/" || url.starts_with(&self.prefix)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Sidebars {
    pub trees: Vec<SidebarTree>,
}

impl Sidebars {
    /// Build the site's sidebars from config + loaded content.
    pub fn build(config: &SiteConfig, content: &Content) -> Sidebars {
        match &config.sidebar {
            SidebarConfig::Auto => Sidebars {
                trees: auto_trees(content),
            },
            SidebarConfig::Items(items) => {
                let nodes = resolve_items(items, "", content);
                Sidebars {
                    trees: vec![SidebarTree { prefix: "/".into(), items: nodes }],
                }
            }
            SidebarConfig::Map(map) => Sidebars {
                trees: map
                    .iter()
                    .map(|(prefix, section)| SidebarTree {
                        prefix: ensure_slashed(prefix),
                        items: resolve_items(&section.items, section.base.as_deref().unwrap_or(""), content),
                    })
                    .collect(),
            },
        }
    }

    /// The sidebar for a page URL: longest covering prefix wins.
    pub fn for_url(&self, url: &str) -> Option<&SidebarTree> {
        self.trees
            .iter()
            .filter(|t| t.covers(url))
            .max_by_key(|t| t.prefix.len())
    }

    /// Leaves (internal links only) of `tree` in display order — the
    /// prev/next sequence.
    pub fn flatten(tree: &SidebarTree) -> Vec<(String, String)> {
        fn walk(n: &SidebarNode, out: &mut Vec<(String, String)>) {
            if let Some(url) = &n.url {
                out.push((n.text.clone(), url.clone()));
            }
            for c in &n.children {
                walk(c, out);
            }
        }
        let mut flat = Vec::new();
        for item in &tree.items {
            walk(item, &mut flat);
        }
        flat
    }

    /// (prev, next) URLs for a page, per its sidebar's flat order.
    pub fn neighbors(&self, url: &str) -> (Option<String>, Option<String>) {
        let Some(tree) = self.for_url(url) else {
            return (None, None);
        };
        let flat = Self::flatten(tree);
        match flat.iter().position(|(_, u)| u == url) {
            Some(i) => (
                i.checked_sub(1).and_then(|p| flat.get(p)).map(|(_, u)| u.clone()),
                flat.get(i + 1).map(|(_, u)| u.clone()),
            ),
            None => (None, None),
        }
    }
}

/// Auto-derive: one tree per top-level content section. `dir/index.md`
/// titles the section group (its URL becomes the group link); loose pages
/// become leaves. Ordering: natural file order from the loader.
fn auto_trees(content: &Content) -> Vec<SidebarTree> {
    // (top segment) → ((group url|None) , group title) … grouped by parent dir
    let mut sections: BTreeMap<String, Vec<&crate::content::Page>> = BTreeMap::new();
    for page in &content.pages {
        let Some(top) = top_segment(&page.url) else { continue };
        sections.entry(top).or_default().push(page);
    }
    sections
        .into_iter()
        .map(|(top, mut pages)| {
            pages.sort_by(|a, b| natural_cmp(&a.rel, &b.rel));
            // group node: index.md of the section if present
            let root = format!("/{top}/");
            let index = pages.iter().find(|p| p.url == root).cloned();
            let mut children = Vec::new();
            let mut group_url = None;
            if let Some(idx) = index {
                group_url = Some(idx.url.clone());
            }
            for p in &pages {
                if Some(&p.url) == group_url.as_ref() {
                    continue;
                }
                children.push(SidebarNode {
                    text: p.title.clone(),
                    url: Some(p.url.clone()),
                    children: Vec::new(),
                    collapsed: None,
                });
            }
            let (text, url) = match index {
                Some(idx) => (idx.title.clone(), group_url),
                None => (top.clone(), None),
            };
            SidebarTree {
                prefix: root,
                items: vec![SidebarNode { text, url, children, collapsed: Some(false) }],
            }
        })
        .collect()
}

fn top_segment(url: &str) -> Option<String> {
    let seg = url.strip_prefix('/')?.split('/').next()?;
    if seg.is_empty() {
        None
    } else {
        Some(seg.to_string())
    }
}

/// Resolve explicit config items: leaf/group links join their nearest
/// `base` (item base overrides the inherited one, VitePress semantics),
/// then internal links are canonicalized against known pages so
/// `/guide/intro` and `config`-style relatives both become the page's
/// real `/…/` URL.
fn resolve_items(items: &[SidebarItem], base: &str, content: &Content) -> Vec<SidebarNode> {
    items
        .iter()
        .map(|item| {
            let effective_base = item.base.as_deref().unwrap_or(base);
            let url = item.link.as_deref().map(|l| resolve_link(l, effective_base, content));
            SidebarNode {
                text: item.text.clone(),
                url,
                children: resolve_items(&item.items, effective_base, content),
                collapsed: item.collapsed,
            }
        })
        .collect()
}

/// Resolve one sidebar link against its base into an absolute URL.
pub fn resolve_link(link: &str, base: &str, content: &Content) -> String {
    if link.starts_with("http://")
        || link.starts_with("https://")
        || link.starts_with("mailto:")
    {
        return link.to_string();
    }
    if link.starts_with('/') {
        return canonical(link, content);
    }
    let joined = format!("{base}{link}");
    canonical(&normalize_dots(&joined), content)
}

/// Resolve `.`/`..` path segments.
fn normalize_dots(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            s => out.push(s),
        }
    }
    format!("/{}", out.join("/"))
}

/// Map a URL to the canonical form of the page it points at when one
/// exists (adds the trailing slash), else pass through (external/anchor
/// targets keep their exact shape).
fn canonical(url: &str, content: &Content) -> String {
    let stripped = url.split('#').next().unwrap_or(url);
    if content.get(stripped).is_some() {
        return stripped.to_string();
    }
    let slashed = if stripped.ends_with('/') {
        stripped.to_string()
    } else {
        format!("{stripped}/")
    };
    if content.get(&slashed).is_some() {
        return slashed;
    }
    url.to_string()
}

fn ensure_slashed(prefix: &str) -> String {
    let with_leading = if prefix.starts_with('/') { prefix } else { &format!("/{prefix}") };
    if with_leading.ends_with('/') {
        with_leading.to_string()
    } else {
        format!("{with_leading}/")
    }
}

/// Load-side helper used by the CLI: content dir is fixed as `content/`.
pub fn content_dir(site_dir: &Path) -> std::path::PathBuf {
    site_dir.join("content")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Page;

    fn page(url: &str, rel: &str, title: &str) -> Page {
        Page {
            rel: rel.into(),
            url: url.into(),
            title: title.into(),
            front: Default::default(),
            body: String::new(),
            modified: None,
            src: rel.into(),
        }
    }

    fn content_with(urls: &[(&str, &str, &str)]) -> Content {
        let mut c = Content::default();
        for (url, rel, title) in urls {
            c.pages.push(page(url, rel, title));
        }
        c.by_url = c.pages.iter().enumerate().map(|(i, p)| (p.url.clone(), i)).collect();
        c
    }

    #[test]
    fn resolve_link_bases() {
        let content = content_with(&[
            ("/reference/default-theme-config/", "reference/default-theme-config.md", "Config"),
            ("/guide/intro/", "guide/intro.md", "Intro"),
        ]);
        // nearest base wins (group base replaces section base, VitePress)
        assert_eq!(
            resolve_link("config", "/reference/default-theme-", &content),
            "/reference/default-theme-config/"
        );
        // absolute internal: gains trailing slash via canonicalization
        assert_eq!(resolve_link("/guide/intro", "", &content), "/guide/intro/");
        // relative with .. navigation
        assert_eq!(
            resolve_link("../intro", "/guide/other/", &content),
            "/guide/intro/"
        );
        // external untouched
        assert_eq!(
            resolve_link("https://example.com/x", "", &content),
            "https://example.com/x"
        );
        // unknown internal target: pass through
        assert_eq!(resolve_link("/missing/", "", &content), "/missing/");
    }

    #[test]
    fn auto_sidebar_groups_by_top_section() {
        let content = content_with(&[
            ("/", "index.md", "Home"),
            ("/guide/", "guide/index.md", "Guide"),
            ("/guide/b-intro/", "guide/b-intro.md", "B Intro"),
            ("/guide/a-start/", "guide/a-start.md", "A Start"),
            ("/reference/api/", "reference/api.md", "API"),
        ]);
        let sb = Sidebars::build(&SiteConfig::default(), &content);
        assert_eq!(sb.trees.len(), 2);
        let guide = sb.for_url("/guide/a-start/").unwrap();
        assert_eq!(guide.prefix, "/guide/");
        assert_eq!(guide.items.len(), 1);
        let group = &guide.items[0];
        assert_eq!(group.text, "Guide");
        assert_eq!(group.url.as_deref(), Some("/guide/"));
        // natural file order: a-start before b-intro
        assert_eq!(group.children[0].text, "A Start");
        assert_eq!(group.children[1].text, "B Intro");
        let reference = sb.for_url("/reference/api/").unwrap();
        assert_eq!(reference.prefix, "/reference/");
        // home page has no sidebar
        assert!(sb.for_url("/").is_none());
    }

    #[test]
    fn prev_next_follow_flat_sidebar_order() {
        let content = content_with(&[
            ("/guide/", "guide/index.md", "Guide"),
            ("/guide/a/", "guide/a.md", "A"),
            ("/guide/b/", "guide/b.md", "B"),
            ("/guide/c/", "guide/c.md", "C"),
        ]);
        let sb = Sidebars::build(&SiteConfig::default(), &content);
        assert_eq!(sb.neighbors("/guide/b/"), (Some("/guide/a/".into()), Some("/guide/c/".into())));
        assert_eq!(sb.neighbors("/guide/a/"), (Some("/guide/".into()), Some("/guide/b/".into())));
        assert_eq!(sb.neighbors("/guide/c/"), (Some("/guide/b/".into()), None));
    }

    #[test]
    fn explicit_map_sidebar_longest_prefix_wins() {
        let content = content_with(&[
            ("/reference/default-theme-config/", "reference/dt-config.md", "Config"),
            ("/reference/api/", "reference/api.md", "API"),
            ("/reference/api/hooks/", "reference/api/hooks.md", "Hooks"),
        ]);
        // The Default Theme group rewrites link "config" via its base, the
        // vitepress.dev pattern; "/reference/api/" is a deeper key so the
        // hooks page must get its own tree.
        let config: SiteConfig = toml::from_str(
            r#"
[sidebar."/reference/"]

  [[sidebar."/reference/".items]]
  text = "Default Theme"
  base = "/reference/default-theme-"

    [[sidebar."/reference/".items.items]]
    text = "Overview"
    link = "config"

[sidebar."/reference/api/"]

  [[sidebar."/reference/api/".items]]
  text = "Hooks"
  link = "/reference/api/hooks/"
"#,
        )
        .unwrap();
        let sb = Sidebars::build(&config, &content);
        let dt = sb.for_url("/reference/default-theme-config/").unwrap();
        assert_eq!(dt.prefix, "/reference/");
        assert_eq!(dt.items[0].url, None);
        assert_eq!(
            dt.items[0].children[0].url.as_deref(),
            Some("/reference/default-theme-config/")
        );
        let api = sb.for_url("/reference/api/hooks/").unwrap();
        assert_eq!(api.prefix, "/reference/api/");
        assert_eq!(api.items[0].url.as_deref(), Some("/reference/api/hooks/"));
    }

    #[test]
    fn single_array_sidebar_applies_site_wide() {
        let content = content_with(&[("/a/", "a.md", "A"), ("/b/", "b.md", "B")]);
        let config: SiteConfig = toml::from_str(
            r#"
[[sidebar]]
text = "All"

  [[sidebar.items]]
  text = "A"
  link = "/a"

  [[sidebar.items]]
  text = "B"
  link = "/b/"
"#,
        )
        .unwrap();
        let sb = Sidebars::build(&config, &content);
        assert_eq!(sb.trees.len(), 1);
        assert_eq!(sb.trees[0].prefix, "/");
        let (prev, next) = sb.neighbors("/a/");
        assert_eq!(prev, None);
        assert_eq!(next.as_deref(), Some("/b/"));
    }
}
