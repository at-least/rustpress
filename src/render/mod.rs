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

use hypertext::prelude::*;

use crate::config::{OutlineLevel, SiteConfig};
use crate::content::{Content, Page};
use crate::content::OutlineSetting as PageOutline;
use crate::markdown::MarkdownEngine;
use crate::sidebar::Sidebars;

pub struct Site {
    pub config: SiteConfig,
    pub content: Content,
    pub sidebars: Sidebars,
    pub engine: MarkdownEngine,
}

impl Site {
    pub fn load(site_dir: &Path) -> Result<Site, BuildError> {
        let config = SiteConfig::load(site_dir)?;
        let content_dir = site_dir.join("content");
        let content = Content::load(&content_dir)?;
        let sidebars = Sidebars::build(&config, &content);
        let engine = MarkdownEngine::new(&config.markdown)?;
        Ok(Site { config, content, sidebars, engine })
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
        let has_sidebar = self.sidebars.for_url(&page.url).is_some();
        let (prev, next) = self.sidebars.neighbors(&page.url);
        let prev = prev.and_then(|u| self.content.get(&u));
        let next = next.and_then(|u| self.content.get(&u));
        let outline = outline_range(&self.config, page);
        let is_home = page.is_home();

        let title = if is_home {
            self.config.title.clone().unwrap_or_default()
        } else {
            match (&self.config.title, &page.front.title) {
                (Some(site), Some(_)) => format!("{} | {site}", page.title),
                (Some(site), None) => format!("{} | {site}", page.title),
                (None, _) => page.title.clone(),
            }
        };
        let description = page
            .front
            .description
            .clone()
            .or_else(|| self.config.description.clone())
            .unwrap_or_default();

        let shell = layout::Shell {
            title,
            description,
            is_home,
            has_sidebar,
            current_url: &page.url,
        };

        let body = if is_home {
            home::home_page(self, page).render().into_inner()
        } else {
            doc::doc_page(self, page, rendered, prev, next, has_sidebar, outline).render().into_inner()
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
        for page in &self.content.pages {
            let rendered = self
                .engine
                .render(page, &self.content, &self.content_root(), &self.content_dir())?;
            if search_enabled {
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
        // generated assets
        write_file(&out_dir.join("syntax.css"), self.engine.syntax_css().as_bytes())?;
        if search_enabled {
            let json = serde_json::to_string(&search_docs).expect("serializable docs");
            write_file(&out_dir.join("search-docs.json"), json.as_bytes())?;
        }
        // site static/ copied verbatim; when the site lives nested under
        // a repo root that also has a static/ dir (the dogfood layout),
        // the root's built assets (app.js, main.css) are layered on top
        // so a plain `gen-docs build` produces a complete deployable site
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
        Ok(stats)
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

    fn render_404(&self) -> Result<String, BuildError> {
        let title = format!("Page not found | {}", self.config.title.clone().unwrap_or_default());
        let shell = layout::Shell {
            title,
            description: String::new(),
            is_home: true,
            has_sidebar: false,
            current_url: "/404.html",
        };
        let home = self.url("/");
        let content = hypertext::rsx! {
            <div class="w-full pt-8 px-6 pb-24 md:pt-12 md:pb-32 md:px-8 lg:pt-12 lg:pb-0 lg:px-8">
                <div class="mx-auto w-full lg:flex lg:justify-center lg:max-w-[62rem] 2xl:max-w-[69rem]">
                    <div class="relative mx-auto w-full lg:px-8 lg:pb-32 xl:order-1 xl:m-0 xl:min-w-[40rem] lg:max-w-[47rem] 2xl:max-w-[49rem]">
                        <div class="mx-auto max-w-[43rem]">
                            <main>
                                <div id="main" class=(vpdoc::vpdoc_class())>
                                    <h1>"Page not found"</h1>
                                    <p>"The page you are looking for does not exist."</p>
                                    <p><a href=(home)>"Return home"</a></p>
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
/// the site config (`deep` → 2–6).
pub fn outline_range(config: &SiteConfig, page: &Page) -> (u8, u8) {
    let site = match config.outline.level {
        Some(OutlineLevel::Single(n)) => (n, n),
        Some(OutlineLevel::Range((a, b))) => (a, b),
        None => (2, 3),
    };
    match &page.front.outline {
        Some(PageOutline::Deep) => (2, 6),
        Some(PageOutline::Level(n)) => (*n, *n),
        Some(PageOutline::Range((a, b))) => (*a, *b),
        None => site,
    }
}

#[derive(Debug, Default)]
pub struct BuildStats {
    pub pages: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    Content(#[from] crate::content::ContentError),
    #[error(transparent)]
    Markdown(#[from] crate::markdown::MarkdownError),
    #[error("cannot write {path}: {source}")]
    Write { path: std::path::PathBuf, source: std::io::Error },
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
