//! Site configuration: `rustpress.toml`, a static-file mirror of VitePress's
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

fn default_light_mode_switch_title() -> String {
    "Switch to light theme".into()
}

fn default_dark_mode_switch_title() -> String {
    "Switch to dark theme".into()
}

fn default_sidebar_menu_label() -> String {
    "Menu".into()
}

fn default_lang_menu_label() -> String {
    "Change language".into()
}

fn default_nav_menu_label() -> String {
    "Main Navigation".into()
}

fn default_mobile_menu_label() -> String {
    "Menu".into()
}

/// The whole `rustpress.toml`.
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
    pub outline: OutlineConfig,

    /// Aside (outline column) position: `false` | `true` | `"left"`
    /// (default right; per-page front matter wins).
    #[serde(default)]
    pub aside: Option<crate::content::AsideSetting>,

    /// Grade container/alert/badge severity colors (danger red,
    /// warning orange, caution yellow) instead of GitHub's palette.
    #[serde(default)]
    pub graded_containers: bool,

    /// Local search. Absent → no search index, no search modal.
    #[serde(default)]
    pub search: Option<Search>,

    /// Navbar logo (VitePress `themeConfig.logo`): a path, `{ src, alt }`
    /// or `{ light, dark, alt }`.
    #[serde(default)]
    pub logo: Option<ThemeableImage>,

    /// Navbar title override (`themeConfig.siteTitle`); `false` hides
    /// the title next to the logo.
    #[serde(default)]
    pub site_title: Option<SiteTitleSetting>,

    /// Glob patterns for content files to exclude (`srcExclude`).
    /// `*` matches within a path segment, `**` across segments.
    #[serde(default)]
    pub src_exclude: Vec<String>,

    /// Optional "Ask AI" sparkle link in the navbar.
    #[serde(default)]
    pub ask_ai_url: Option<String>,

    /// Markdown feature options ([markdown] section).
    #[serde(default)]
    pub markdown: Markdown,

    /// UI theme: a bundled theme name (a CSS file shipped in the
    /// binary under themes/) or a path to a CSS file relative to the
    /// site dir (copied verbatim into the output under themes/ and
    /// linked after vitepress.css). Unset = the stock look.
    #[serde(default)]
    pub theme: Option<String>,

    /// Code colors ([code] section) — the syntax highlight, separate
    /// from the UI theme.
    #[serde(default)]
    pub code: SyntaxHighlight,

    /// Dark-mode behavior: `true` (default, toggleable, follows system),
    /// `false` (light only, no toggle), `"dark"` (dark default,
    /// toggleable), `"force"` (always dark, no toggle), `"force-auto"`
    /// (always system, no toggle).
    #[serde(default)]
    pub appearance: Appearance,

    /// Fail the build on internal links that resolve to nothing.
    /// `true` ignores all dead links; a string list ignores links that
    /// start with any listed prefix; absent/false = checking on.
    #[serde(default)]
    pub ignore_dead_links: IgnoreDeadLinks,

    /// Emit sitemap.xml when configured (`hostname` required).
    #[serde(default)]
    pub sitemap: Option<Sitemap>,

    /// Extra tags injected into every page's <head>.
    #[serde(default)]
    pub head: Vec<HeadTag>,

    /// Document title template; `:title` is replaced by the page title.
    /// `false` (upstream spelling) disables the suffix entirely.
    #[serde(default)]
    pub title_template: Option<TitleTemplate>,

    /// Directory (inside the site dir) holding the markdown content.
    #[serde(default = "default_src_dir")]
    pub src_dir: String,

    /// Dev-loop helper for sites nested in a repo that builds its theme
    /// assets outside the site dir (the bundled rustpress demo/docs): a
    /// path (relative to the site dir) whose contents are copied over the
    /// output after every build, so freshly built CSS/JS land in public/
    /// without rebuilding the binary. Unset = nothing is layered.
    #[serde(default)]
    pub static_overlay: Option<String>,

    /// Prev/next pager labels.
    #[serde(default)]
    pub doc_footer: Option<DocFooter>,

    /// 404 page texts.
    #[serde(default)]
    pub not_found: Option<NotFound>,

    #[serde(default = "default_last_updated_text")]
    pub last_updated_text: String,

    #[serde(default = "default_return_to_top")]
    pub return_to_top_label: String,

    #[serde(default = "default_dark_mode_switch_label")]
    pub dark_mode_switch_label: String,

    #[serde(default = "default_light_mode_switch_title")]
    pub light_mode_switch_title: String,

    #[serde(default = "default_dark_mode_switch_title")]
    pub dark_mode_switch_title: String,

    #[serde(default = "default_sidebar_menu_label")]
    pub sidebar_menu_label: String,

    #[serde(default = "default_lang_menu_label")]
    pub lang_menu_label: String,

    #[serde(default = "default_nav_menu_label")]
    pub nav_menu_label: String,

    #[serde(default = "default_mobile_menu_label")]
    pub mobile_menu_label: String,

    #[serde(default = "default_skip_to_content")]
    pub skip_to_content_label: String,

    /// URL rewrites: source content path pattern → destination template,
    /// in declaration order. The first matching rule wins (VitePress
    /// semantics). A pattern may end with `:rest*`; the target splices
    /// the capture back with `:rest` (or `:rest*`).
    #[serde(default, deserialize_with = "deserialize_rewrites")]
    pub rewrites: Vec<Rewrite>,

    /// Locales for multi-language sites. The special key `root` describes
    /// the top-level content; every other key names a content
    /// subdirectory (`content/zh/...` served under `/zh/`).
    #[serde(default)]
    pub locales: std::collections::BTreeMap<String, Locale>,
}

fn default_true() -> bool {
    true
}

/// One `[rewrites]` rule. Kept as an ordered vec of pairs (not a map)
/// because rules apply first-match-wins in declaration order.
#[derive(Debug, Clone, PartialEq)]
pub struct Rewrite {
    /// Source content path pattern; may end with `:rest*`.
    pub from: String,
    /// Destination template; may splice the capture with `:rest`.
    pub to: String,
}

/// Deserializes the `[rewrites]` table preserving declaration order
/// (`toml`'s `preserve_order` feature makes map iteration follow the
/// document).
fn deserialize_rewrites<'de, D>(d: D) -> Result<Vec<Rewrite>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = Vec<Rewrite>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a table mapping source paths to destination templates")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut m: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some((from, to)) = m.next_entry::<String, String>()? {
                out.push(Rewrite { from, to });
            }
            Ok(out)
        }
    }
    d.deserialize_map(V)
}

fn default_src_dir() -> String {
    "content".into()
}

fn default_last_updated_text() -> String {
    "Last updated".into()
}

fn default_return_to_top() -> String {
    "Return to top".into()
}

fn default_dark_mode_switch_label() -> String {
    "Appearance".into()
}

fn default_skip_to_content() -> String {
    "Skip to content".into()
}

impl Default for SiteConfig {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

impl SiteConfig {
    pub const CONFIG_FILE: &'static str = "rustpress.toml";

    /// Load `rustpress.toml` from a site directory and validate it.
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
            return Err(ConfigError::Base {
                value: self.base.clone(),
            });
        }
        for item in &self.nav {
            validate_nav_item(item)?;
        }
        for rule in &self.rewrites {
            if rule.from.is_empty() {
                return Err(ConfigError::Rewrite {
                    rule: rule.from.clone(),
                    problem: "empty source pattern".into(),
                });
            }
            if rule.to.is_empty() {
                return Err(ConfigError::Rewrite {
                    rule: rule.from.clone(),
                    problem: "empty destination".into(),
                });
            }
            // the only capture syntax is a trailing `:rest*`; any other
            // `:` (a path-to-regexp param like `:pkg`, or a misplaced
            // capture) would silently match nothing
            if let Some(i) = rule.from.find(':') {
                if rule.from[i..] != *":rest*" {
                    return Err(ConfigError::Rewrite {
                        rule: rule.from.clone(),
                        problem:
                            "only a trailing `:rest*` capture is supported in the source pattern"
                                .into(),
                    });
                }
            } else if rule.to.contains(":rest") {
                return Err(ConfigError::Rewrite {
                    rule: rule.from.clone(),
                    problem: "the target splices :rest but the source pattern captures nothing \
                              (end it with `:rest*`)"
                        .into(),
                });
            }
        }
        Ok(())
    }
}

/// Upstream's nav items are either link-type or children-type; a leaf
/// without a `link` would render as an `<a href="/">`, and a `link`
/// alongside `items` would be silently ignored by the dropdown branch.
fn validate_nav_item(item: &NavItem) -> Result<(), ConfigError> {
    if item.link.is_some() && !item.items.is_empty() {
        return Err(ConfigError::Nav {
            text: item.text.clone(),
            problem: "a dropdown item cannot also set link (it would be ignored)".into(),
        });
    }
    if item.link.is_none() && item.items.is_empty() {
        return Err(ConfigError::Nav {
            text: item.text.clone(),
            problem: "a nav item needs either a link or dropdown items".into(),
        });
    }
    for child in &item.items {
        if child.link.is_none() && child.items.is_empty() {
            return Err(ConfigError::Nav {
                text: child.text.clone(),
                problem: "a dropdown entry needs a link".into(),
            });
        }
    }
    Ok(())
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
    /// Link `target` attribute override.
    #[serde(default)]
    pub target: Option<String>,
    /// Link `rel` attribute override.
    #[serde(default)]
    pub rel: Option<String>,
}

/// VitePress's `sidebar` accepts one array (single sidebar) or an object
/// keyed by URL path prefix (one sidebar per section). Absent → derive
/// from the content tree. A hand-written `Deserialize` (instead of
/// `untagged`) so a typo inside an item surfaces as "unknown field
/// `lnik`" rather than "did not match any variant of untagged enum".
#[derive(Debug, Clone, Default, PartialEq)]
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

impl<'de> Deserialize<'de> for Sidebar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        let value = toml::Value::deserialize(deserializer)?;
        match value {
            toml::Value::Array(_) => Vec::<SidebarItem>::deserialize(value)
                .map(Sidebar::Items)
                .map_err(|e| D::Error::custom(format!("invalid sidebar item list: {e}"))),
            toml::Value::Table(_) => BTreeMapPrefixSections::deserialize(value)
                .map(Sidebar::Map)
                .map_err(|e| D::Error::custom(format!("invalid path-keyed sidebar: {e}"))),
            _ => Err(D::Error::custom(
                "sidebar must be an array of items or a table keyed by URL prefix",
            )),
        }
    }
}

/// Newtype alias for the path-keyed sidebar map. Keys sort
/// lexicographically (BTreeMap); resolution always picks the longest
/// matching prefix, so iteration order is irrelevant.
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
    /// Custom text shown when this item is a prev/next pager target.
    #[serde(default)]
    pub doc_footer_text: Option<String>,
    /// Link `target` attribute override.
    #[serde(default)]
    pub target: Option<String>,
    /// Link `rel` attribute override.
    #[serde(default)]
    pub rel: Option<String>,
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
    /// Accessible label override (default: the icon's name).
    #[serde(default)]
    pub aria_label: Option<String>,
    /// Link `target` override (default `_blank`).
    #[serde(default)]
    pub target: Option<String>,
}

/// `themeConfig.logo` / hero images: a plain path, `{ src, alt }`, or a
/// `{ light, dark }` pair switched by the color scheme. Hand-written
/// `Deserialize` so an unknown inner field names itself.
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeableImage {
    Simple(String),
    Detailed {
        src: String,
        alt: Option<String>,
    },
    Dual {
        light: String,
        dark: String,
        alt: Option<String>,
    },
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ImageDetailed {
    src: String,
    #[serde(default)]
    alt: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ImageDual {
    light: String,
    dark: String,
    #[serde(default)]
    alt: Option<String>,
}

impl<'de> Deserialize<'de> for ThemeableImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        let value = toml::Value::deserialize(deserializer)?;
        match value {
            toml::Value::String(s) => Ok(ThemeableImage::Simple(s)),
            toml::Value::Table(ref t) => {
                // route by shape so the reported error is the relevant one
                let dual = t.contains_key("light") || t.contains_key("dark");
                if dual {
                    ImageDual::deserialize(value)
                        .map(|d| ThemeableImage::Dual {
                            light: d.light,
                            dark: d.dark,
                            alt: d.alt,
                        })
                        .map_err(|e| D::Error::custom(format!("invalid image: {e}")))
                } else {
                    ImageDetailed::deserialize(value)
                        .map(|d| ThemeableImage::Detailed { src: d.src, alt: d.alt })
                        .map_err(|e| D::Error::custom(format!("invalid image: {e}")))
                }
            }
            _ => Err(D::Error::custom(
                "invalid image: expected a path string, a { src, alt } table, or a { light, dark } table",
            )),
        }
    }
}

/// `themeConfig.siteTitle`: a string override or `false` to hide.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum SiteTitleSetting {
    Text(String),
    Hide(bool),
}

/// `titleTemplate`: `":title …"` template or `false` (no suffix).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TitleTemplate {
    Tmpl(String),
    Off(bool),
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
/// (VitePress: `outline: { level: [2, 3] }`; default h2–h3). A
/// one-element array (`[2]`) is accepted too and means "that level
/// only" — the shape vitepress.dev's deployed config resolves to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineLevel {
    Single(u8),
    SingleList([u8; 1]),
    Range((u8, u8)),
}

impl<'de> Deserialize<'de> for OutlineLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        const EXPECTED: &str = "outline level must be a heading level number or a [min, max] pair";
        let value = toml::Value::deserialize(deserializer)?;
        match value {
            toml::Value::Integer(n) => u8::try_from(n)
                .map(OutlineLevel::Single)
                .map_err(|_| D::Error::custom(EXPECTED)),
            toml::Value::Array(_) => {
                if let Ok([n]) = <[u8; 1]>::deserialize(value.clone()) {
                    return Ok(OutlineLevel::SingleList([n]));
                }
                <(u8, u8)>::deserialize(value)
                    .map(|(a, b)| OutlineLevel::Range((a, b)))
                    .map_err(|_| D::Error::custom(EXPECTED))
            }
            _ => Err(D::Error::custom(EXPECTED)),
        }
    }
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

/// The `outline` setting: `false` (no outline), a bare level
/// (`outline = 2` / `[2, 3]`), or the `{ level, label }` table.
/// Hand-written `Deserialize` so a bad level names the setting instead
/// of "did not match any variant of untagged enum".
#[derive(Debug, Clone, PartialEq)]
pub enum OutlineConfig {
    Off(bool),
    Level(OutlineLevel),
    Full(Outline),
}

impl<'de> Deserialize<'de> for OutlineConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        const EXPECTED: &str =
            "outline must be false, a level number, a [min, max] pair, or a { level, label } table";
        let value = toml::Value::deserialize(deserializer)?;
        match value {
            toml::Value::Boolean(b) => Ok(OutlineConfig::Off(b)),
            v @ (toml::Value::Integer(_) | toml::Value::Array(_)) => {
                OutlineLevel::deserialize(v).map(OutlineConfig::Level).map_err(|_| D::Error::custom(EXPECTED))
            }
            toml::Value::Table(_) => Outline::deserialize(value)
                .map(OutlineConfig::Full)
                .map_err(|e| D::Error::custom(format!("invalid outline table: {e}"))),
            _ => Err(D::Error::custom(EXPECTED)),
        }
    }
}

impl Default for OutlineConfig {
    fn default() -> Self {
        OutlineConfig::Full(Outline::default())
    }
}

impl OutlineConfig {
    pub fn enabled(&self) -> bool {
        !matches!(self, OutlineConfig::Off(false))
    }

    pub fn label(&self) -> String {
        match self {
            OutlineConfig::Full(o) => o.label.clone(),
            _ => default_outline_label(),
        }
    }

    pub fn level(&self) -> Option<OutlineLevel> {
        match self {
            OutlineConfig::Off(_) => None,
            OutlineConfig::Level(l) => Some(*l),
            OutlineConfig::Full(o) => o.level,
        }
    }
}

// The code colors ([code] below) select from
// the vendored Helix themes by file stem; the defaults live in
// default_light_theme / default_dark_theme.

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Search {
    /// Only `local` is supported (documents JSON + modal).
    pub provider: SearchProvider,
    /// UI strings for the button and modal (upstream
    /// `search.options.translations` subset).
    #[serde(default)]
    pub translations: SearchTranslations,
}

/// Translatable local-search strings. Keys follow upstream's
/// `translations`; `{q}` in `noResultsText` is the query placeholder.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchTranslations {
    /// Navbar button label.
    #[serde(default = "default_search_button_text")]
    pub button_text: String,
    /// Navbar button aria-label.
    #[serde(default = "default_search_button_text")]
    pub button_aria_label: String,
    /// Modal input placeholder.
    #[serde(default = "default_search_placeholder")]
    pub placeholder: String,
    /// Shown when a query matches nothing; `{q}` = the query.
    #[serde(default = "default_search_no_results")]
    pub no_results_text: String,
    /// Clear-button title.
    #[serde(default = "default_search_reset")]
    pub reset_button_title: String,
    /// Footer hint after the arrow keys.
    #[serde(default = "default_search_navigate")]
    pub navigate_text: String,
    /// Footer hint after Enter.
    #[serde(default = "default_search_select")]
    pub select_text: String,
    /// Footer hint after Esc.
    #[serde(default = "default_search_close")]
    pub close_text: String,
}

impl Default for SearchTranslations {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

fn default_search_button_text() -> String {
    "Search".into()
}

fn default_search_placeholder() -> String {
    "Search docs".into()
}

fn default_search_no_results() -> String {
    "No results for \"{q}\"".into()
}

fn default_search_reset() -> String {
    "Clear".into()
}

fn default_search_navigate() -> String {
    "to navigate".into()
}

fn default_search_select() -> String {
    "to select".into()
}

fn default_search_close() -> String {
    "to close".into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum SearchProvider {
    Local,
}

fn default_light_theme() -> String {
    "github_light".into()
}

fn default_dark_theme() -> String {
    "github_dark".into()
}

/// The syntax highlight theme pair.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SyntaxHighlight {
    #[serde(default = "default_light_theme")]
    pub light: String,
    #[serde(default = "default_dark_theme")]
    pub dark: String,
}

impl Default for SyntaxHighlight {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

/// Dark-mode behavior (see [`SiteConfig::appearance`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    Toggleable { default_dark: bool },
    LightOnly,
    ForceDark,
    ForceAuto,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance::Toggleable {
            default_dark: false,
        }
    }
}

impl Appearance {
    /// Should the navbar toggle switch render at all?
    pub fn toggleable(&self) -> bool {
        !matches!(
            self,
            Appearance::LightOnly | Appearance::ForceDark | Appearance::ForceAuto
        )
    }
}

impl<'de> Deserialize<'de> for Appearance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Bool(bool),
            Word(String),
        }
        // a wrong-typed value (an integer, say) used to leak the private
        // "untagged enum Raw" name; name the setting instead
        let raw = Raw::deserialize(deserializer).map_err(|_| {
            D::Error::custom(
                "appearance must be true, false, \"dark\", \"force\", \"force-dark\", or \"force-auto\"",
            )
        })?;
        match raw {
            Raw::Bool(true) => Ok(Appearance::Toggleable {
                default_dark: false,
            }),
            Raw::Bool(false) => Ok(Appearance::LightOnly),
            Raw::Word(w) => match w.as_str() {
                "dark" => Ok(Appearance::Toggleable { default_dark: true }),
                "force" | "force-dark" => Ok(Appearance::ForceDark),
                "force-auto" => Ok(Appearance::ForceAuto),
                other => Err(D::Error::custom(format!(
                    "unknown appearance {other:?}: expected true, false, \"dark\", \"force\", \"force-dark\", \"force-auto\""
                ))),
            },
        }
    }
}

/// `ignoreDeadLinks`: `true` (ignore all), `"localhostLinks"` (never
/// check localhost URLs — ours skips all http(s) targets anyway, so
/// this parses but behaves like checking), or a list of link prefixes.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum IgnoreDeadLinks {
    #[default]
    Check,
    IgnoreAll,
    IgnoreLocalhost,
    IgnorePrefixes(Vec<String>),
}

impl<'de> Deserialize<'de> for IgnoreDeadLinks {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Bool(bool),
            Word(String),
            Prefixes(Vec<String>),
        }
        let raw = Raw::deserialize(deserializer).map_err(|_| {
            serde::de::Error::custom(
                "ignoreDeadLinks must be true, false, \"localhostLinks\", or a list of link prefixes",
            )
        })?;
        match raw {
            Raw::Bool(true) => Ok(IgnoreDeadLinks::IgnoreAll),
            Raw::Bool(false) => Ok(IgnoreDeadLinks::Check),
            Raw::Word(w) if w == "localhostLinks" => Ok(IgnoreDeadLinks::IgnoreLocalhost),
            Raw::Word(other) => Err(serde::de::Error::custom(format!(
                "unknown ignoreDeadLinks {other:?}: expected true, false, \"localhostLinks\", or a list of prefixes"
            ))),
            Raw::Prefixes(v) => Ok(IgnoreDeadLinks::IgnorePrefixes(v)),
        }
    }
}

/// `[sitemap]` — emits sitemap.xml.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Sitemap {
    pub hostname: String,
}

/// One `[[head]]` tag: `tag`, inline-table `attrs`, optional `children`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct HeadTag {
    pub tag: String,
    #[serde(default)]
    pub attrs: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub children: Option<String>,
}

/// Markdown feature switches ([markdown] section).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Markdown {
    /// Number the lines of every code block (per-fence `:line-numbers`
    /// / `:no-line-numbers` overrides).
    #[serde(default)]
    pub line_numbers: bool,

    /// Render the hover copy button on code blocks.
    #[serde(default = "default_true")]
    pub code_copy_button: bool,

    /// Render `$…$` / `$$…$$` TeX via client-side MathJax.
    #[serde(default)]
    pub math: bool,

    #[serde(default)]
    pub image: ImageOptions,

    #[serde(default)]
    pub container: ContainerOptions,
}

impl Default for Markdown {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ImageOptions {
    /// Add `loading="lazy"` to content images. Accepts upstream's
    /// `lazyLoad` spelling too.
    #[serde(default, alias = "lazyLoad")]
    pub lazy_loading: bool,
}

/// Container title labels and custom container kinds. Keys are the
/// VitePress `markdown.container` label names.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContainerOptions {
    #[serde(default)]
    pub tip_label: Option<String>,
    #[serde(default)]
    pub warning_label: Option<String>,
    #[serde(default)]
    pub danger_label: Option<String>,
    #[serde(default)]
    pub note_label: Option<String>,
    #[serde(default)]
    pub info_label: Option<String>,
    #[serde(default)]
    pub important_label: Option<String>,
    #[serde(default)]
    pub caution_label: Option<String>,
    #[serde(default)]
    pub details_label: Option<String>,
    /// Custom `::: name` containers: rendered with `kind`'s styling and
    /// the given label (default: the name uppercased).
    #[serde(default, rename = "custom")]
    pub custom: Vec<CustomContainer>,
}

impl ContainerOptions {
    /// Display label for a builtin kind.
    pub fn label_for(&self, kind: &str) -> String {
        let upper = kind.to_uppercase();
        match kind {
            "tip" => self.tip_label.clone(),
            "warning" => self.warning_label.clone(),
            "danger" => self.danger_label.clone(),
            "note" => self.note_label.clone(),
            "info" => self.info_label.clone(),
            "important" => self.important_label.clone(),
            "caution" => self.caution_label.clone(),
            "details" => self
                .details_label
                .clone()
                .or_else(|| Some("Details".into())),
            _ => None,
        }
        .unwrap_or(upper)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CustomContainer {
    /// The `::: name` keyword.
    pub name: String,
    /// Builtin kind whose styling is reused (default: tip).
    #[serde(default)]
    pub kind: Option<String>,
    /// Title row label (default: the name uppercased).
    #[serde(default)]
    pub label: Option<String>,
}

/// `[docFooter]` — prev/next pager labels; `false` on either side
/// disables that pager.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DocFooter {
    #[serde(default)]
    pub prev: Option<PagerLabel>,
    #[serde(default)]
    pub next: Option<PagerLabel>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PagerLabel {
    Text(String),
    Off(bool),
}

/// `[notFound]` — 404 page texts.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NotFound {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub quote: Option<String>,
    #[serde(default)]
    pub link_text: Option<String>,
}

/// One locale (`[locales.root]`, `[locales.zh]`, …).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Locale {
    /// Display name in the language switcher.
    pub label: String,
    /// Language code (`lang` attribute).
    #[serde(default)]
    pub lang: Option<String>,
    /// Locale-specific site title.
    #[serde(default)]
    pub title: Option<String>,
    /// Locale-specific description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Errors loading or validating `rustpress.toml`.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid TOML in {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("invalid base {value:?}: must start and end with '/'")]
    Base { value: String },
    #[error("invalid rewrite rule {rule:?}: {problem}")]
    Rewrite { rule: String, problem: String },
    #[error("invalid nav item {text:?}: {problem}")]
    Nav { text: String, problem: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> SiteConfig {
        toml::from_str(src).expect("parse")
    }

    #[test]
    fn sidebar_typos_name_the_field() {
        let err = toml::from_str::<SiteConfig>(
            "title = \"T\"\n[[sidebar]]\ntext = \"Guide\"\nlnik = \"/guide/\"\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("lnik"), "{err}");

        let err = toml::from_str::<SiteConfig>(
            "title = \"T\"\n[sidebar.\"/guide/\"]\nbse = \"/x/\"\n[[sidebar.\"/guide/\".items]]\ntext = \"A\"\nlink = \"/a/\"\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("bse"), "{err}");
    }

    #[test]
    fn themeable_image_typos_name_the_field() {
        // the error must name the unknown field, not just "untagged enum"
        let err =
            toml::from_str::<SiteConfig>("title = \"T\"\nlogo = { srcs = \"x.png\" }\n")
                .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown field") && msg.contains("srcs"), "{msg}");
    }

    #[test]
    fn wrong_typed_scalars_name_the_setting() {
        // a wrong-typed value used to leak "untagged enum Raw"; the
        // message must name the setting and its expected shapes
        let err =
            toml::from_str::<SiteConfig>("title = \"T\"\nappearance = 123\n").unwrap_err();
        assert!(
            err.to_string().contains("appearance must be"),
            "{err}"
        );
        let err =
            toml::from_str::<SiteConfig>("title = \"T\"\nignoreDeadLinks = 123\n").unwrap_err();
        assert!(
            err.to_string().contains("ignoreDeadLinks must be"),
            "{err}"
        );
        let err = toml::from_str::<SiteConfig>(
            "title = \"T\"\n[outline]\nlevel = \"two\"\n",
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("outline level must be"),
            "{err}"
        );
    }

    #[test]
    fn nav_items_need_a_link_or_a_dropdown() {
        // a linkless leaf used to render an <a href="/">
        let cfg: SiteConfig = toml::from_str("title = \"T\"\n[[nav]]\ntext = \"X\"\n").unwrap();
        assert!(cfg.validate().unwrap_err().to_string().contains("nav"));
        // link alongside items: the link would be silently ignored
        let cfg: SiteConfig = toml::from_str(
            "title = \"T\"\n[[nav]]\ntext = \"X\"\nlink = \"/a/\"\n[[nav.items]]\ntext = \"Y\"\nlink = \"/b/\"\n",
        )
        .unwrap();
        assert!(cfg.validate().unwrap_err().to_string().contains("nav"));
        // dropdown children need links too
        let cfg: SiteConfig = toml::from_str(
            "title = \"T\"\n[[nav]]\ntext = \"X\"\n[[nav.items]]\ntext = \"Y\"\n",
        )
        .unwrap();
        assert!(cfg.validate().unwrap_err().to_string().contains("nav"));
        // a well-formed dropdown still parses
        let cfg: SiteConfig = toml::from_str(
            "title = \"T\"\n[[nav]]\ntext = \"X\"\n[[nav.items]]\ntext = \"Y\"\nlink = \"/b/\"\n",
        )
        .unwrap();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn empty_file_gives_defaults() {
        let c = SiteConfig::default();
        assert_eq!(c.lang, "en");
        assert_eq!(c.base, "/");
        assert_eq!(c.outline.label(), "On this page");
        assert!(c.nav.is_empty());
        assert_eq!(c.sidebar, Sidebar::Auto);
        assert!(c.search.is_none());
        assert!(!c.last_updated);
    }

    #[test]
    fn syntax_section_configurable() {
        // syntax highlight: separate top-level section, defaults to
        // Helix's github_light/github_dark, overridable by name
        let c = parse("");
        assert_eq!(c.code.light, "github_light");
        assert_eq!(c.code.dark, "github_dark");
        let c = parse("[code]\nlight = \"base16-ocean.light\"\ndark = \"base16-ocean.dark\"\n");
        assert_eq!(c.code.light, "base16-ocean.light");
        assert_eq!(c.code.dark, "base16-ocean.dark");
        // theme key no longer exists under [markdown]
        let err = toml::from_str::<SiteConfig>("[markdown.theme]\nlight = \"x\"\n").unwrap_err();
        assert!(err.to_string().contains("theme"), "{err}");
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
        assert_eq!(c.outline.level(), Some(OutlineLevel::Range((2, 3))));
        assert_eq!(
            c.search.as_ref().map(|s| s.provider),
            Some(SearchProvider::Local)
        );
        assert_eq!(c.search.unwrap().translations.button_text, "Search");
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
            SocialIcon::Svg {
                svg: "<svg></svg>".into()
            }
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
        assert_eq!(c.outline.level(), Some(OutlineLevel::Single(2)));
    }

    #[test]
    fn unknown_key_is_rejected() {
        let err = toml::from_str::<SiteConfig>("titel = \"typo\"\n").unwrap_err();
        assert!(
            err.to_string().contains("titel"),
            "error should name the bad key: {err}"
        );
    }

    #[test]
    fn bad_base_is_rejected_by_validate() {
        let c = SiteConfig {
            base: "docs".into(),
            ..Default::default()
        };
        assert!(matches!(c.validate(), Err(ConfigError::Base { .. })));
    }

    #[test]
    fn appearance_accepts_upstream_force_dark_spelling() {
        let c = parse("appearance = \"force-dark\"\n");
        assert_eq!(c.appearance, Appearance::ForceDark);
        assert!(!c.appearance.toggleable());
    }

    #[test]
    fn image_lazyload_alias() {
        let c = parse("[markdown.image]\nlazyLoad = true\n");
        assert!(c.markdown.image.lazy_loading);
        let c = parse("[markdown.image]\nlazyLoading = true\n");
        assert!(c.markdown.image.lazy_loading);
    }

    #[test]
    fn container_labels_including_details() {
        let opts: ContainerOptions =
            toml::from_str("detailsLabel = \"Details der Seite\"\n").unwrap();
        assert_eq!(opts.label_for("details"), "Details der Seite");
        assert_eq!(ContainerOptions::default().label_for("details"), "Details");
    }

    #[test]
    fn markdown_container_section_parses_custom_kinds() {
        let c = parse(
            "[markdown.container]\nnoteLabel = \"Nota\"\n\n[[markdown.container.custom]]\nname = \"success\"\nkind = \"tip\"\nlabel = \"SUCCESS\"\n",
        );
        assert_eq!(c.markdown.container.label_for("note"), "Nota");
        assert_eq!(c.markdown.container.custom.len(), 1);
        assert_eq!(c.markdown.container.custom[0].name, "success");
        assert_eq!(c.markdown.container.custom[0].kind.as_deref(), Some("tip"));
    }
}
