//! Site configuration: `gen-docs.toml`, a static-file mirror of VitePress's
//! `themeConfig` schema (VitePress's config is TypeScript, which a Rust
//! binary cannot execute; the keys intentionally keep VitePress's camelCase
//! names so the mapping is mechanical).
//!
//! Unknown keys are rejected (`deny_unknown_fields`) so config typos fail
//! at load time instead of silently doing nothing. (Content front matter
//! stays VitePress's YAML — see `content.rs`.)

use std::path::{Path, PathBuf};

use serde::Deserialize;

fn default_lang() -> String {
    "en".into()
}

fn default_base() -> String {
    "/".into()
}

fn default_outline_label() -> String {
    "On this page".into()
}

fn default_light_theme() -> String {
    "github-light".into()
}

fn default_dark_theme() -> String {
    "github-dark".into()
}

/// The whole `gen-docs.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SiteConfig {
    /// Site title, shown in the navbar and used as the `<title>` prefix.
    #[serde(default)]
    pub title: Option<String>,

    /// Site-wide description used for `<meta name="description">` when a
    /// page has none of its own.
    #[serde(default)]
    pub description: Option<String>,

    /// Root language of the site (`en`, `zh-TW`, …).
    #[serde(default = "default_lang")]
    pub lang: String,

    /// Base URL path the site is deployed under (`"/"`, `"/docs/"`, …).
    /// Must start and end with `/`.
    #[serde(default = "default_base")]
    pub base: String,

    /// Show "last updated" timestamps from file modification times.
    #[serde(default)]
    pub last_updated: bool,

    /// Navbar menu. Plain link items need `link`; dropdown items need
    /// `items`.
    #[serde(default)]
    pub nav: Vec<NavItem>,

    /// Sidebar. Absent → derived automatically from the content directory
    /// tree; an explicit value is VitePress's opt-in form.
    #[serde(default)]
    pub sidebar: Sidebar,

    /// Navbar social icons.
    #[serde(default)]
    pub social_links: Vec<SocialLink>,

    /// "Edit this page" link; `pattern` gets the page's source-relative
    /// path substituted for `:path` (VitePress convention).
    #[serde(default)]
    pub edit_link: Option<EditLink>,

    /// Footer message/copyright line (hidden on sidebar pages, like
    /// VitePress).
    #[serde(default)]
    pub footer: Option<Footer>,

    /// Right-hand outline ("on this page") settings.
    #[serde(default)]
    pub outline: Outline,

    /// Markdown/highlighting settings.
    #[serde(default)]
    pub markdown: Markdown,

    /// Local search. Absent → no search index, no search modal.
    #[serde(default)]
    pub search: Option<Search>,

    /// Navbar logo path, relative to the site's static/ directory
    /// (VitePress `themeConfig.logo`).
    #[serde(default)]
    pub logo: Option<String>,

    /// Optional "Ask AI" sparkle link in the navbar.
    #[serde(default)]
    pub ask_ai_url: Option<String>,
}

impl Default for SiteConfig {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

impl SiteConfig {
    pub const CONFIG_FILE: &'static str = "gen-docs.toml";

    /// Load `gen-docs.toml` from a site directory and validate it.
    pub fn load(site_dir: &Path) -> Result<SiteConfig, ConfigError> {
        let path = site_dir.join(Self::CONFIG_FILE);
        let raw = std::fs::read_to_string(&path).map_err(|source| ConfigError::Read {
            path: path.clone(),
            source,
        })?;
        let config: SiteConfig = toml::from_str(&raw).map_err(|source| ConfigError::Parse {
            path: path.clone(),
            source,
        })?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if !self.base.starts_with('/') || !self.base.ends_with('/') {
            return Err(ConfigError::Base { value: self.base.clone() });
        }
        Ok(())
    }
}

/// A navbar entry: `{ text, link, activeMatch }` (plain link) or
/// `{ text, items }` (dropdown).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NavItem {
    pub text: String,
    /// Target URL for plain link items; absent on dropdown items.
    #[serde(default)]
    pub link: Option<String>,
    /// Optional URL prefix overriding link-based active matching
    /// (VitePress `activeMatch`).
    #[serde(default)]
    pub active_match: Option<String>,
    /// Dropdown entries.
    #[serde(default)]
    pub items: Vec<NavItem>,
}

/// VitePress's `sidebar` accepts one array (single sidebar) or an object
/// keyed by URL path prefix (one sidebar per section). Absent → derive
/// from the content tree.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Sidebar {
    /// Not configured: derive one sidebar per top-level content section
    /// from the directory tree.
    #[default]
    Auto,
    /// One sidebar for the whole site.
    Items(Vec<SidebarItem>),
    /// Path-keyed sidebars: the longest matching prefix wins
    /// (`'/reference/': { base, items }`).
    Map(BTreeMapPrefixSections),
}

/// Newtype so path-keyed sidebar maps keep insertion order in error
/// messages; resolution always picks the longest matching prefix, so
/// iteration order is irrelevant.
pub type BTreeMapPrefixSections = std::collections::BTreeMap<String, SidebarSection>;

/// One path-keyed sidebar: `items` links are resolved against `base`
/// when set (VitePress `base: '/reference/default-theme-'` pattern).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SidebarSection {
    #[serde(default)]
    pub base: Option<String>,
    pub items: Vec<SidebarItem>,
}

/// A sidebar node: leaf (`link`) or group (`items`); `collapsed` is
/// VitePress's tri-state — `true` starts collapsed, `false` starts
/// expanded with a caret, absent = not collapsible.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SidebarItem {
    pub text: String,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub items: Vec<SidebarItem>,
    #[serde(default)]
    pub collapsed: Option<bool>,
    #[serde(default)]
    pub base: Option<String>,
}

/// A navbar social icon: a known name (`github`, `twitter`, `discord`, …)
/// or an inline `svg` string.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum SocialIcon {
    Name(String),
    Svg { svg: String },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SocialLink {
    pub icon: SocialIcon,
    pub link: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EditLink {
    /// URL template; `:path` is replaced with the page's source-relative
    /// path (e.g. `guide/getting-started.md`).
    pub pattern: String,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Footer {
    pub message: Option<String>,
    pub copyright: Option<String>,
}

/// `level` is a single heading level or a `[min, max]` pair
/// (VitePress: `outline: { level: [2, 3] }`; default h2–h3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum OutlineLevel {
    Single(u8),
    Range((u8, u8)),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outline {
    #[serde(default)]
    pub level: Option<OutlineLevel>,
    #[serde(default = "default_outline_label")]
    pub label: String,
}

impl Default for Outline {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Markdown {
    /// Dual syntax-highlighting themes (syntect tmTheme names), matching
    /// the VitePress default pair.
    #[serde(default)]
    pub theme: HighlightThemes,
}

impl Default for Markdown {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct HighlightThemes {
    #[serde(default = "default_light_theme")]
    pub light: String,
    #[serde(default = "default_dark_theme")]
    pub dark: String,
}

impl Default for HighlightThemes {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Search {
    /// Only `local` is supported (documents JSON + modal).
    pub provider: SearchProvider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum SearchProvider {
    Local,
}

/// Errors loading or validating `gen-docs.toml`.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("invalid TOML in {path}: {source}")]
    Parse { path: PathBuf, source: toml::de::Error },
    #[error("invalid base {value:?}: must start and end with '/'")]
    Base { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> SiteConfig {
        toml::from_str(src).expect("parse")
    }

    #[test]
    fn empty_file_gives_defaults() {
        let c = SiteConfig::default();
        assert_eq!(c.lang, "en");
        assert_eq!(c.base, "/");
        assert_eq!(c.outline.label, "On this page");
        assert_eq!(c.markdown.theme.light, "github-light");
        assert_eq!(c.markdown.theme.dark, "github-dark");
        assert!(c.nav.is_empty());
        assert_eq!(c.sidebar, Sidebar::Auto);
        assert!(c.search.is_none());
        assert!(!c.last_updated);
    }

    #[test]
    fn full_config_round_trip() {
        let c = parse(
            r#"
title = "My Docs"
description = "Example site"
lang = "zh-TW"
base = "/docs/"
lastUpdated = true

[[nav]]
text = "Guide"
link = "/guide/"
activeMatch = "/guide/"

[[nav]]
text = "More"

  [[nav.items]]
  text = "Reference"
  link = "/reference/"

[[socialLinks]]
icon = "github"
link = "https://github.com/me/repo"

[editLink]
pattern = "https://github.com/me/repo/edit/main/docs/:path"
text = "Edit this page"

[footer]
message = "Released under the MIT License."
copyright = "Copyright (c) 2026 Me"

[outline]
level = [2, 3]
label = "On this page"

[markdown.theme]
light = "github-light"
dark = "github-dark"

[search]
provider = "local"
"#,
        );
        assert_eq!(c.title.as_deref(), Some("My Docs"));
        assert_eq!(c.lang, "zh-TW");
        assert_eq!(c.base, "/docs/");
        assert!(c.last_updated);
        assert_eq!(c.nav.len(), 2);
        assert_eq!(c.nav[0].active_match.as_deref(), Some("/guide/"));
        assert_eq!(c.nav[1].items.len(), 1);
        assert_eq!(c.social_links[0].icon, SocialIcon::Name("github".into()));
        assert_eq!(
            c.edit_link.as_ref().unwrap().pattern,
            "https://github.com/me/repo/edit/main/docs/:path"
        );
        assert_eq!(c.outline.level, Some(OutlineLevel::Range((2, 3))));
        assert_eq!(c.search, Some(Search { provider: SearchProvider::Local }));
        assert_eq!(c.markdown.theme.dark, "github-dark");
    }

    #[test]
    fn social_link_svg_icon_form() {
        let c = parse(
            r#"
[[socialLinks]]
link = "https://example.com"

  [socialLinks.icon]
  svg = "<svg></svg>"
"#,
        );
        assert_eq!(
            c.social_links[0].icon,
            SocialIcon::Svg { svg: "<svg></svg>".into() }
        );
    }

    #[test]
    fn sidebar_single_array_form() {
        let c = parse(
            r#"
[[sidebar]]
text = "Guide"

  [[sidebar.items]]
  text = "Intro"
  link = "/guide/intro"

[[sidebar]]
text = "Reference"
collapsed = false
"#,
        );
        match c.sidebar {
            Sidebar::Items(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[1].collapsed, Some(false));
                assert_eq!(items[0].items[0].link.as_deref(), Some("/guide/intro"));
            }
            other => panic!("expected Items, got {other:?}"),
        }
    }

    #[test]
    fn sidebar_path_keyed_map_form() {
        let c = parse(
            r#"
[sidebar."/guide/"]

  [[sidebar."/guide/".items]]
  text = "Intro"
  link = "/guide/intro"

[sidebar."/reference/"]
base = "/reference/"

  [[sidebar."/reference/".items]]
  text = "Default Theme"
  base = "/reference/default-theme-"

    [[sidebar."/reference/".items.items]]
    text = "Overview"
    link = "config"
"#,
        );
        match c.sidebar {
            Sidebar::Map(map) => {
                assert_eq!(map.len(), 2);
                let r = &map["/reference/"];
                assert_eq!(r.base.as_deref(), Some("/reference/"));
                let group = &r.items[0];
                assert_eq!(group.base.as_deref(), Some("/reference/default-theme-"));
                assert_eq!(group.items[0].link.as_deref(), Some("config"));
            }
            other => panic!("expected Map, got {other:?}"),
        }
    }

    #[test]
    fn outline_level_single_number() {
        let c = parse("outline.level = 2\n");
        assert_eq!(c.outline.level, Some(OutlineLevel::Single(2)));
    }

    #[test]
    fn unknown_key_is_rejected() {
        let err = toml::from_str::<SiteConfig>("titel = \"typo\"\n").unwrap_err();
        assert!(err.to_string().contains("titel"), "error should name the bad key: {err}");
    }

    #[test]
    fn bad_base_is_rejected_by_validate() {
        let c = SiteConfig { base: "docs".into(), ..Default::default() };
        assert!(matches!(c.validate(), Err(ConfigError::Base { .. })));
    }
}
