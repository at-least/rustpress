//! Site rendering: bundles config + content + sidebars + the markdown
//! engine, renders every page to HTML via the hypertext templates, and
//! writes the static output (`public/`).

pub mod doc;
pub mod home;
pub mod icons;
pub mod layout;
pub mod local_nav;
pub mod navbar;
pub mod search_modal;
pub mod sidebar;
pub mod vpdoc;

use std::path::Path;

use std::path::PathBuf;
use std::sync::OnceLock;

use hypertext::prelude::*;

use crate::config::{IgnoreDeadLinks, OutlineLevel, SiteConfig};
use crate::content::{Content, Page};
use crate::content::OutlineSetting as PageOutline;
use crate::markdown::MarkdownEngine;
use crate::sidebar::Sidebars;

pub struct Site {
    pub config: SiteConfig,
    pub content: Content,
    pub sidebars: Sidebars,
    pub engine: MarkdownEngine,
    /// Resolved UI theme CSS (built-in palette or the user's file), to
    /// be written as `theme.css`. `None` = the stock look, nothing linked.
    pub theme_css: Option<String>,
}

impl Site {
    pub fn load(site_dir: &Path) -> Result<Site, BuildError> {
        let config = SiteConfig::load(site_dir)?;
        let content_dir = site_dir.join(&config.src_dir);
        let mut content = Content::load(&content_dir, &config.src_exclude)?;
        let sidebars = Sidebars::build(&config, &content);
        let engine = MarkdownEngine::new(&config.markdown, &config.syntax, site_dir)?;
        // git timestamps beat mtimes: a fresh clone's mtimes are checkout
        // time, which would make "last updated" meaningless
        if config.last_updated {
            for page in &mut content.pages {
                let secs = std::process::Command::new("git")
                    .args(["log", "-1", "--format=%ct", "--"])
                    .arg(&page.src)
                    .current_dir(site_dir)
                    .output()
                    .ok()
                    .and_then(|out| String::from_utf8(out.stdout).ok())
                    .and_then(|text| text.trim().parse::<i64>().ok());
                if let Some(secs) = secs {
                    page.modified = Some(
                        std::time::SystemTime::UNIX_EPOCH
                            + std::time::Duration::from_secs(secs.max(0) as u64),
                    );
                }
            }
        }
        // URL rewrites (VitePress `rewrites`): applied over source rel
        // paths; patterns may end with `:rest*` captured as `:rest`
        if !config.rewrites.is_empty() {
            for page in &mut content.pages {
                for (from, to) in &config.rewrites {
                    let dest = if let Some(prefix) = from.strip_suffix(":rest*") {
                        page.rel.strip_prefix(prefix).map(|rest| to.replace(":rest*", rest))
                    } else if *from == page.rel {
                        Some(to.clone())
                    } else {
                        None
                    };
                    if let Some(dest_rel) = dest {
                        page.url = crate::content::page_url(&normalize_rewrite_path(&dest_rel));
                    }
                }
            }
            content.by_url = content
                .pages
                .iter()
                .enumerate()
                .map(|(i, p)| (p.url.clone(), i))
                .collect();
        }

        // locale assignment from content subdirectories ([locales.zh] →
        // content/zh/**)
        for page in &mut content.pages {
            for key in config.locales.keys() {
                if key != "root"
                    && let Some(rest) = page.rel.strip_prefix(&format!("{key}/"))
                {
                    page.locale = key.clone();
                    let _ = rest;
                }
            }
        }

        // theme.css: UI palette overrides as CSS custom properties
        // (VitePress "extending the default theme" style — copied
        // verbatim and linked after main.css). Syntax colors are NOT
        // here — they live in the [syntax] section of rustpress.toml.
        // UI theme: one variable, one choice — a built-in palette name or
        // a path to a CSS file (values ending in `.css`). Unset/`vitepress`
        // = the stock look, nothing emitted.
        // unset or the default built-in: the stock look
        let selected = config.theme.as_deref().filter(|v| *v != crate::palettes::DEFAULT_NAME);
        let theme_css = match selected {
            None => None,
            Some(value) if value.ends_with(".css") => {
                let path = site_dir.join(value);
                if !path.is_file() {
                    return Err(BuildError::ThemeFile {
                        path: path.clone(),
                        value: value.to_string(),
                        available: crate::palettes::available(),
                    });
                }
                Some(std::fs::read_to_string(&path).map_err(|source| BuildError::Write {
                    path: path.clone(),
                    source,
                })?)
            }
            Some(name) => Some(
                crate::palettes::css(name)
                    .ok_or_else(|| BuildError::Theme {
                        value: name.to_string(),
                        available: crate::palettes::available(),
                    })?
                    .to_string(),
            ),
        };
        Ok(Site { config, content, sidebars, engine, theme_css })
    }

    /// Prefix a canonical path with the configured base. Relative asset
    /// names ("syntax.css") are rooted first so they resolve from any
    /// page depth.
    pub fn url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("mailto:") {
            return path.to_string();
        }
        let base = self.config.base.trim_end_matches('/');
        if path == "/" {
            return format!("{base}/");
        }
        let rooted = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        format!("{base}{rooted}")
    }

    /// Render one page's full HTML document.
    pub fn render_page(&self, page: &Page) -> Result<String, BuildError> {
        let rendered = self
            .engine
            .render(page, &self.content, &self.content_root(), &self.content_dir())?;
        self.render_page_inner(page, &rendered)
    }

    fn render_page_inner(
        &self,
        page: &Page,
        rendered: &crate::markdown::RenderedPage,
    ) -> Result<String, BuildError> {
        use crate::content::{AsideSetting, LastUpdatedSetting};

        // per-page toggles (front matter wins)
        let has_sidebar =
            self.sidebars.for_url(&page.url).is_some() && page.front.sidebar != Some(false);
        let has_navbar = page.front.navbar != Some(false);
        let show_footer = self.config.footer.is_some() && page.front.footer != Some(false);
        let edit_on = page.front.edit_link != Some(false);
        let aside_left = AsideSetting::resolve(page.front.aside.as_ref(), self.config.aside.as_ref());
        let is_home = page.is_home();
        let is_page_layout = page.front.layout.as_deref() == Some("page");

        // pager: sidebar flat order, then front-matter prev/next overrides
        let (prev, next) = self.sidebars.neighbors(&page.url);
        let prev = apply_pager_override(page.front.prev.as_ref(), prev, self);
        let next = apply_pager_override(page.front.next.as_ref(), next, self);
        let outline = outline_range(&self.config, page);
        // last-updated: page Date override > git/mtime > front-matter off
        let last_updated = match &page.front.last_updated {
            Some(LastUpdatedSetting::Toggle(false)) => None,
            Some(LastUpdatedSetting::Date(d)) => Some((d.clone(), d.clone())),
            _ => self
                .config
                .last_updated
                .then(|| page.modified.map(doc::format_date).map(|d| (d.0, d.1)))
                .flatten(),
        };

        let title = self.document_title(page, is_home);
        let locale = self.config.locales.get(&page.locale).cloned();
        let description = page
            .front
            .description
            .clone()
            .or_else(|| locale.as_ref().and_then(|l| l.description.clone()))
            .or_else(|| self.config.description.clone())
            .unwrap_or_default();
        let shell = layout::Shell {
            title,
            description,
            is_home,
            has_navbar,
            has_sidebar,
            show_footer,
            page_class: page.front.page_class.clone(),
            head_extra: layout::serialize_head_tags_to_string(&page.front.head),
            current_url: &page.url,
            has_math: rendered.has_math,
            lang: self.locale_lang(page),
            translations: self.translations_for(page),
            site_title: self.navbar_site_title(page),
        };

        let body = if is_home {
            home::home_page(self, page).render().into_inner()
        } else {
            // `layout: page` strips the doc chrome (aside, edit link,
            // timestamps, pager) like upstream's VPPage
            let (aside, edit_on, last_updated, outline) = if is_page_layout {
                (None, false, None, None)
            } else {
                (aside_left, edit_on, last_updated, outline)
            };
            doc::doc_page(
                self,
                page,
                rendered,
                prev.as_ref(),
                next.as_ref(),
                has_sidebar,
                outline,
                aside,
                edit_on,
                last_updated.as_ref(),
            )
            .render()
            .into_inner()
        };
        let document = layout::layout(self, &shell, &rendered.headings, body);
        let html = document.render().into_inner().to_string();
        // hypertext omits the doctype; prepend it like the Zola build.
        Ok(format!("<!DOCTYPE html>\n{html}"))
    }

    /// Build the whole site into `out_dir`.
    pub fn build(&self, site_dir: &Path, out_dir: &Path) -> Result<BuildStats, BuildError> {
        let mut stats = BuildStats::default();
        let mut search_docs: Vec<serde_json::Value> = Vec::new();
        let search_enabled = self.config.search.is_some();
        let mut dead_links: Vec<(String, Vec<String>)> = Vec::new();
        for page in &self.content.pages {
            let rendered = self
                .engine
                .render(page, &self.content, &self.content_root(), &self.content_dir())?;
            let dead = find_dead_links(&rendered.html, page, self);
            let dead: Vec<String> = match &self.config.ignore_dead_links {
                IgnoreDeadLinks::IgnorePrefixes(prefixes) => dead
                    .into_iter()
                    .filter(|l| !prefixes.iter().any(|p| l.starts_with(p)))
                    .collect(),
                _ => dead,
            };
            if !dead.is_empty() {
                dead_links.push((page.url.clone(), dead));
            }
            if search_enabled && page.front.search != Some(false) {
                let title = if page.is_home() {
                    self.config.title.clone().unwrap_or_else(|| page.title.clone())
                } else {
                    page.title.clone()
                };
                search_docs.push(serde_json::json!({
                    "url": self.url(&page.url),
                    "title": title,
                    "body": plain_text(&rendered.html),
                }));
            }
            let html = self.render_page_with(page, rendered)?;
            let file = out_dir.join(page.url.trim_start_matches('/')).join("index.html");
            write_file(&file, html.as_bytes())?;
            stats.pages += 1;
        }
        // 404
        let not_found = self.render_404()?;
        write_file(&out_dir.join("404.html"), not_found.as_bytes())?;
        // theme assets embedded in the binary go first; the site's own
        // static/ is copied over them, and when the site lives nested
        // under a repo root that also has a static/ dir (the dogfood
        // layout) the root's freshly built assets are layered on top so
        // `npm run dev` picks up CSS/JS changes without a Rust rebuild
        crate::theme_assets::extract(out_dir).map_err(|source| BuildError::Write {
            path: out_dir.to_path_buf(),
            source,
        })?;
        let static_dir = site_dir.join("static");
        if static_dir.is_dir() {
            copy_dir(&static_dir, out_dir)?;
        }
        if let Some(root) = site_dir.parent() {
            let root_static = root.join("static");
            if root_static.is_dir() {
                copy_dir(&root_static, out_dir)?;
            }
        }
        // generated assets are written AFTER the static copy: a
        // same-named leftover in static/ (e.g. a stale syntax.css) must
        // never shadow the freshly generated one
        write_file(&out_dir.join("syntax.css"), self.engine.syntax_css().as_bytes())?;
        if search_enabled {
            let json = serde_json::to_string(&search_docs).expect("serializable docs");
            write_file(&out_dir.join("search-docs.json"), json.as_bytes())?;
        }
        if let Some(css) = &self.theme_css {
            write_file(&out_dir.join("theme.css"), css.as_bytes())?;
            stats.theme = true;
        }
        if let Some(sitemap) = &self.config.sitemap {
            let xml = self.sitemap_xml(&sitemap.hostname);
            write_file(&out_dir.join("sitemap.xml"), xml.as_bytes())?;
            stats.sitemap = true;
        }
        match &self.config.ignore_dead_links {
            // IgnoreAll skips the report; Check, IgnoreLocalhost (a
            // no-op: http(s) targets are never collected as dead links)
            // and IgnorePrefixes report (prefix filtering ran earlier)
            IgnoreDeadLinks::Check
            | IgnoreDeadLinks::IgnoreLocalhost
            | IgnoreDeadLinks::IgnorePrefixes(_) => {
                let mut broken: Vec<String> = Vec::new();
                for (page_url, links) in &dead_links {
                    for link in links {
                        if !path_exists_on_disk(out_dir, link) {
                            broken.push(format!("  {page_url} → {link}"));
                        }
                    }
                }
                if !broken.is_empty() {
                    return Err(BuildError::DeadLinks {
                        report: format!(
                            "{} dead link(s) found (ignoreDeadLinks silences this):\n{}",
                            broken.len(),
                            broken.join("\n")
                        ),
                    });
                }
            }
            IgnoreDeadLinks::IgnoreAll => {}
        }
        Ok(stats)
    }

    /// Language switcher entries for a page: (label, href, is_current).
    /// Same-rel-path page in the other locale when it exists, else that
    /// locale's root (VitePress behavior).
    pub fn translations_for(&self, page: &Page) -> Vec<(String, String, bool)> {
        if self.config.locales.len() <= 1 {
            return Vec::new();
        }
        let own_base = if page.locale == "root" {
            String::new()
        } else {
            format!("/{}", page.locale)
        };
        let url_rest = page.url.strip_prefix(&own_base).unwrap_or(&page.url);
        let mut out = Vec::new();
        for (key, loc) in &self.config.locales {
            let target_base = if key == "root" {
                String::new()
            } else {
                format!("/{key}")
            };
            let candidate = format!("{target_base}{url_rest}");
            let is_current = *key == page.locale;
            let href = if is_current {
                self.url(&page.url)
            } else if self.content.by_url.contains_key(&candidate) {
                self.url(&candidate)
            } else {
                self.url(&format!("{target_base}/"))
            };
            out.push((loc.label.clone(), href, is_current));
        }
        out
    }

    /// The `lang` attribute for a page (its locale's code).
    pub fn locale_lang(&self, page: &Page) -> String {
        self.config
            .locales
            .get(&page.locale)
            .and_then(|l| l.lang.clone())
            .unwrap_or_else(|| self.config.lang.clone())
    }

    /// The site title a locale's pages carry (its `title` override, else
    /// the site title).
    fn locale_title(&self, locale: &str) -> Option<String> {
        self.config
            .locales
            .get(locale)
            .and_then(|l| l.title.clone())
            .or_else(|| self.config.title.clone())
    }

    /// The site-title part of `<title>` (locale-aware, NOT affected by
    /// `siteTitle`, which upstream scopes to the navbar).
    fn document_site_title(&self, page: &Page) -> String {
        self.locale_title(&page.locale).unwrap_or_default()
    }

    /// The navbar title: `siteTitle` wins (text override, or hidden),
    /// else the locale-aware site title.
    fn navbar_site_title(&self, page: &Page) -> Option<String> {
        match &self.config.site_title {
            Some(crate::config::SiteTitleSetting::Text(t)) => Some(t.clone()),
            Some(crate::config::SiteTitleSetting::Hide(_)) => None,
            None => Some(self.document_site_title(page)),
        }
    }

    /// `<title>`: VitePress's `titleTemplate` semantics — `:title` is
    /// replaced with the page title; `false` drops the suffix; without
    /// a template it's `Page | Site` (home pages use the site title
    /// alone). The page's front-matter template wins over the site's.
    fn document_title(&self, page: &Page, is_home: bool) -> String {
        let site_title = self.document_site_title(page);
        if is_home || page.title.is_empty() {
            return site_title;
        }
        use crate::content::FrontmatterTitleTemplate as FTT;
        match &page.front.title_template {
            Some(FTT::Off(_)) => page.title.clone(),
            Some(FTT::Tmpl(t)) if t.contains(":title") => t.replace(":title", &page.title),
            Some(FTT::Tmpl(t)) => format!("{} — {}", page.title, t),
            None => match &self.config.title_template {
                Some(crate::config::TitleTemplate::Off(_)) => page.title.clone(),
                Some(crate::config::TitleTemplate::Tmpl(t)) if t.contains(":title") => {
                    t.replace(":title", &page.title)
                }
                Some(crate::config::TitleTemplate::Tmpl(t)) => format!("{} — {}", page.title, t),
                None => format!("{} | {}", page.title, site_title),
            },
        }
    }

    /// render_page with a precomputed markdown render (build renders
    /// each page once and reuses the result for the search index).
    fn render_page_with(
        &self,
        page: &Page,
        rendered: crate::markdown::RenderedPage,
    ) -> Result<String, BuildError> {
        let rendered = &rendered;
        let html = self.render_page_inner(page, rendered)?;
        Ok(html)
    }

    /// sitemap.xml from every page URL, lastmod from the page timestamps.
    fn sitemap_xml(&self, hostname: &str) -> String {
        let host = hostname.trim_end_matches('/');
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
        for page in &self.content.pages {
            xml.push_str("  <url>\n");
            xml.push_str(&format!("    <loc>{}{}</loc>\n", host, self.url(&page.url)));
            if let Some(t) = page.modified {
                let (date, _) = doc::format_date(t);
                xml.push_str(&format!("    <lastmod>{}</lastmod>\n", date));
            }
            xml.push_str("  </url>\n");
        }
        xml.push_str("</urlset>\n");
        xml
    }

    fn render_404(&self) -> Result<String, BuildError> {
        let nf = self.config.not_found.clone().unwrap_or_default();
        let title = format!(
            "{} | {}",
            nf.title.clone().unwrap_or_else(|| "Page not found".into()),
            self.config.title.clone().unwrap_or_default()
        );
        let shell = layout::Shell {
            title,
            description: String::new(),
            is_home: true,
            has_navbar: true,
            has_sidebar: false,
            show_footer: false,
            page_class: None,
            head_extra: String::new(),
            current_url: "/404.html",
            has_math: false,
            lang: self.config.lang.clone(),
            translations: Vec::new(),
            site_title: match &self.config.site_title {
                Some(crate::config::SiteTitleSetting::Text(t)) => Some(t.clone()),
                Some(crate::config::SiteTitleSetting::Hide(_)) => None,
                None => self.locale_title("root"),
            },
        };
        let home = self.url("/");
        let nf_title = nf.title.clone().unwrap_or_else(|| "Page not found".into());
        let nf_quote = nf.quote.clone().unwrap_or_else(|| "The page you are looking for does not exist.".into());
        let nf_link = nf.link_text.clone().unwrap_or_else(|| "Return home".into());
        let content = hypertext::rsx! {
            <div class="w-full pt-8 px-6 pb-24 md:pt-12 md:pb-32 md:px-8 lg:pt-12 lg:pb-0 lg:px-8">
                <div class="mx-auto w-full lg:flex lg:justify-center lg:max-w-[62rem] 2xl:max-w-[69rem]">
                    <div class="relative mx-auto w-full lg:px-8 lg:pb-32 xl:order-1 xl:m-0 xl:min-w-[40rem] lg:max-w-[47rem] 2xl:max-w-[49rem]">
                        <div class="mx-auto max-w-[43rem]">
                            <main>
                                <div id="main" class=(vpdoc::vpdoc_class())>
                                    <h1>(nf_title)</h1>
                                    <p>(nf_quote)</p>
                                    <p><a href=(home)>(nf_link)</a></p>
                                </div>
                            </main>
                        </div>
                    </div>
                </div>
            </div>
        };
        let content_html = content.render().into_inner();
        let document = layout::layout(self, &shell, &[], content_html);
        Ok(format!("<!DOCTYPE html>\n{}", document.render().into_inner()))
    }

    fn content_root(&self) -> std::path::PathBuf {
        // The engine resolves `@/` includes against the site dir; content
        // dir for relative ones. Both are the running site's layout.
        self.content_dir()
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default()
    }

    fn content_dir(&self) -> std::path::PathBuf {
        // Recovered from any page's src path (all pages share the root).
        self.content
            .pages
            .first()
            .and_then(|p| p.src.parent().map(|d| d.to_path_buf()))
            .unwrap_or_default()
    }
}

/// Effective outline heading-level range: page front matter overrides
/// the site config (`deep` → 2–6, `false` → none). `None` = disabled.
pub fn outline_range(config: &SiteConfig, page: &Page) -> Option<(u8, u8)> {
    if !config.outline.enabled() {
        return None;
    }
    let site = match config.outline.level() {
        Some(OutlineLevel::Single(n)) => (n, n),
        Some(OutlineLevel::Range((a, b))) => (a, b),
        None => (2, 3),
    };
    match &page.front.outline {
        Some(PageOutline::Off) => None,
        Some(PageOutline::Deep) => Some((2, 6)),
        Some(PageOutline::Level(n)) => Some((*n, *n)),
        Some(PageOutline::Range((a, b))) => Some((*a, *b)),
        None => Some(site),
    }
}

/// One pager entry (prev or next): display text + target URL.
pub struct PagerLink {
    pub text: String,
    pub href: String,
}

/// Front-matter `prev`/`next` on the sidebar-derived neighbor:
/// `false` hides the side, a string swaps the display text, and an
/// object is a full custom link (upstream prev-next.ts semantics).
fn apply_pager_override(
    setting: Option<&crate::content::PrevNext>,
    derived: Option<(String, String)>,
    site: &Site,
) -> Option<PagerLink> {
    use crate::content::PrevNext;
    match setting {
        Some(PrevNext::Off(_)) => None,
        None => derived.map(|(text, url)| PagerLink { text, href: site.url(&url) }),
        Some(PrevNext::Text(text)) => derived.map(|(_, url)| PagerLink {
            text: text.clone(),
            href: site.url(&url),
        }),
        Some(PrevNext::Obj { text, link, .. }) => Some(PagerLink {
            text: text.clone(),
            href: site.url(link),
        }),
    }
}

#[derive(Debug, Default)]
pub struct BuildStats {
    pub pages: usize,
    pub sitemap: bool,
    pub theme: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("unknown theme {value:?} — expected a built-in palette ({available}) or a path to a .css file")]
    Theme { value: String, available: String },
    #[error("theme file {path:?} not found (theme = {value:?}); built-in palettes: {available}")]
    ThemeFile { path: PathBuf, value: String, available: String },
    #[error("{}", .report)]
    DeadLinks { report: String },
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    Content(#[from] crate::content::ContentError),
    #[error(transparent)]
    Markdown(#[from] crate::markdown::MarkdownError),
    #[error("cannot write {path}: {source}")]
    Write { path: std::path::PathBuf, source: std::io::Error },
}


/// Candidates for dead links: internal href/src targets in one page's
/// html that are neither known pages nor resolvable to an output path.
/// Absolute (base-free) and relative links are both normalized to
/// base-free output paths; the disk check happens after the copy.
fn find_dead_links(html: &str, page: &Page, site: &Site) -> Vec<String> {
    static HREF: OnceLock<regex::Regex> = OnceLock::new();
    let re = HREF.get_or_init(|| regex::Regex::new(r#"(?:href|src)="([^"]+)""#).unwrap());
    let mut out = Vec::new();
    for cap in re.captures_iter(html) {
        let raw = &cap[1];
        if raw.starts_with("http://")
            || raw.starts_with("https://")
            || raw.starts_with("mailto:")
            || raw.starts_with("data:")
            || raw.starts_with('#')
        {
            continue;
        }
        let no_frag = raw.split('#').next().unwrap_or(raw);
        if no_frag.is_empty() {
            continue;
        }
        if no_frag.starts_with('/') {
            // absolute, base-free: page set or output file
            let target = no_frag.trim_end_matches('/');
            if !site.content.by_url.contains_key(&format!("{target}/"))
                && !site.content.by_url.contains_key(&format!("{target}.html"))
            {
                out.push(target.to_string());
            }
        } else {
            // relative: resolve against the page's SOURCE directory, the
            // VitePress semantics our markdown linker uses too
            match crate::markdown::resolve_relative(no_frag, &page.rel, &site.content) {
                Some(_) => {} // alive page
                None => {
                    let mut segs: Vec<String> = Path::new(&page.rel)
                        .parent()
                        .map(|d| {
                            d.components()
                                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                                .collect()
                        })
                        .unwrap_or_default();
                    for seg in no_frag.split('/') {
                        match seg {
                            "." => {}
                            ".." => {
                                segs.pop();
                            }
                            s => segs.push(s.to_string()),
                        }
                    }
                    let path = segs.join("/");
                    let url = format!("/{}/", path.trim_end_matches('/'));
                    if !site.content.by_url.contains_key(&url) {
                        out.push(path);
                    }
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Disk check for a dead-link candidate: pages already failed the page
/// set, so this only clears output files/assets.
fn path_exists_on_disk(out_dir: &Path, link: &str) -> bool {
    let rel = link.trim_start_matches('/');
    out_dir.join(rel).is_file()
        || out_dir.join(rel).join("index.html").is_file()
        || out_dir.join(format!("{rel}.html")).is_file()
}

/// Search body text: rendered HTML with tags stripped, entities
/// decoded, whitespace collapsed to single spaces.
fn plain_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            '&' if !in_tag => {
                // decode the handful of entities comrak/hypertext emit
                let mut entity = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ';' || entity.len() > 8 {
                        break;
                    }
                    entity.push(c);
                    chars.next();
                }
                let _ = chars.next(); // consume ';'
                let decoded: String = match entity.as_str() {
                    "amp" => "&".into(),
                    "lt" => "<".into(),
                    "gt" => ">".into(),
                    "quot" => "\"".into(),
                    "apos" => "'".into(),
                    "nbsp" => "\u{a0}".into(),
                    other => format!("&{other};"),
                };
                out.push_str(&decoded);
            }
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalize a rewrite destination to a content-rel path (page_url
/// input shape).
fn normalize_rewrite_path(path: &str) -> String {
    let p = path.trim_start_matches('/');
    if p.ends_with(".md") || p.ends_with("index.md") {
        p.to_string()
    } else {
        format!("{p}.md")
    }
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), BuildError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| BuildError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::write(path, bytes).map_err(|source| BuildError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), BuildError> {
    for entry in std::fs::read_dir(from).map_err(|source| BuildError::Write {
        path: from.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| BuildError::Write {
            path: from.to_path_buf(),
            source,
        })?;
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|source| BuildError::Write {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            std::fs::copy(entry.path(), &target).map_err(|source| BuildError::Write {
                path: target,
                source,
            })?;
        }
    }
    Ok(())
}
