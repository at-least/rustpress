//! The document shell (ported from templates/base.html): head with the
//! blocking anti-FOUC script, body scaffolding, footer, script tags.

use hypertext::prelude::*;
use hypertext::Raw;

use crate::render::navbar;
use crate::render::Site;

/// Everything the shell needs to know about the current page.
pub struct Shell<'a> {
    pub title: String,
    pub description: String,
    pub is_home: bool,
    pub has_navbar: bool,
    pub has_sidebar: bool,
    pub show_footer: bool,
    /// Extra class on the content container (front matter `pageClass`).
    pub page_class: Option<String>,
    /// Serialized per-page `head` tags, appended after the site's.
    pub head_extra: String,
    /// Current page URL (canonical, base-free) for active matching.
    pub current_url: &'a str,
    /// Page contains math → inject the MathJax loader.
    pub has_math: bool,
    /// `lang` attribute (locale-aware).
    pub lang: String,
    /// (label, href, current) language switcher entries.
    pub translations: Vec<(String, String, bool)>,
    /// The navbar title for this page (locale-aware; `None` = hidden
    /// via `siteTitle: false`).
    pub site_title: Option<String>,
}

/// The whole HTML document; `content` is the VPContent body.
pub fn layout<'a>(site: &'a Site, shell: &'a Shell<'a>, headings: &'a [crate::markdown::Heading], body_html: String) -> impl Renderable + 'a {
    let lang = shell.lang.clone();
    let title = shell.title.clone();
    let description = shell.description.clone();
    let syntax_css = site.url("syntax.css");
    let main_css = site.url("main.css");
    let app_js = site.url("js/app.js");
    let search_index_url = site.url("search-docs.json");
    let is_home = shell.is_home;
    let has_sidebar = shell.has_sidebar;
    let current_url = shell.current_url.to_string();
    let footer = site.config.footer.clone();
    let show_footer = shell.show_footer && footer.is_some();
    let head_tags = serialize_head_tags(&site.config.head);
    let head_extra = Raw::dangerously_create(shell.head_extra.clone());
    let skip_label = site.config.skip_to_content_label.clone();
    let theme_link = site.theme_css.as_ref().map(|_| site.url("theme.css"));
    let footer_message = footer.as_ref().and_then(|f| f.message.clone());
    let footer_copyright = footer.as_ref().and_then(|f| f.copyright.clone());

    let content_class = if has_sidebar {
        format!(
            "grow shrink-0 m-0 w-full lg:mt-[var(--vp-layout-top-height,0px)] lg:pt-(--vp-nav-height) lg:pl-(--vp-sidebar-width) 2xl:pr-[calc((100%-var(--vp-layout-max-width))/2)] 2xl:pl-[calc((100%-var(--vp-layout-max-width))/2+var(--vp-sidebar-width))]{}",
            shell.page_class.as_ref().map(|c| format!(" {c}")).unwrap_or_default()
        )
    } else {
        format!(
            "grow shrink-0 w-full mx-auto mt-[var(--vp-layout-top-height,0px)] max-w-full lg:pt-(--vp-nav-height){}",
            shell.page_class.as_ref().map(|c| format!(" {c}")).unwrap_or_default()
        )
    };

    rsx! {
        <html lang=(lang) class="[color-scheme:light] dark:[color-scheme:dark]">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>(title)</title>
                <meta name="description" content=(description)>
                <link rel="stylesheet" href=(syntax_css)>
                <link rel="stylesheet" href=(main_css)>
                @if let Some(href) = theme_link.clone() {
                    <link rel="stylesheet" href=(href)>
                }
                (Raw::dangerously_create(appearance_script(&site.config.appearance)))
                @if shell.has_math {
                    (Raw::dangerously_create(MATHJAX_SCRIPT.to_string()))
                }
                (head_tags)
                (head_extra)
            </head>
            <body class="font-sans bg-bg text-text-1 antialiased [text-rendering:optimizeLegibility] [-moz-osx-font-smoothing:grayscale] [text-autospace:normal] [text-spacing-trim:normal]">
                <a class="sr-only" href="#main">(skip_label)</a>

                <div class="fixed inset-0 z-(--vp-z-index-backdrop) bg-(--vp-backdrop-bg-color) transition-opacity duration-500 xl:hidden" id="VPBackdrop" x-cloak x-show="$store.ui.screen || $store.ui.sidebar" @click="$store.ui.screen = false; $store.ui.sidebar = false"></div>

                @if shell.has_navbar {
                    <header class="relative top-[var(--vp-layout-top-height,0px)] left-0 z-(--vp-z-index-nav) w-full pointer-events-none lg:fixed">
                        (navbar::navbar(site, &current_url, is_home, has_sidebar, &shell.translations, shell.site_title.as_deref()))
                        (navbar::nav_screen(site, &current_url, &shell.translations))
                    </header>
                }

                (Raw::dangerously_create(super::local_nav::local_nav(site, is_home, has_sidebar, headings)))
                (Raw::dangerously_create(super::sidebar::sidebar(site, &current_url)))

                <div class=(content_class) id="VPContent">
                    (Raw::dangerously_create(body_html.clone()))
                </div>

                @if show_footer {
                    <footer class=(format!("relative z-(--vp-z-index-footer) border-t border-gutter py-8 px-6 bg-bg md:px-8{}", if has_sidebar { " hidden" } else { "" }))>
                        <div class="mx-auto max-w-(--vp-layout-max-width) text-center [&_a]:underline [&_a]:underline-offset-[0.125rem] [&_a]:transition-colors [&_a]:duration-[250ms] [&_a:hover]:text-text-1">
                            @if let Some(message) = footer_message.clone() {
                                <p class="leading-[1.7142857] text-[0.875rem] font-medium text-text-2">(Raw::dangerously_create(message))</p>
                            }
                            @if let Some(copyright) = footer_copyright.clone() {
                                <p class="leading-[1.7142857] text-[0.875rem] font-medium text-text-2">(Raw::dangerously_create(copyright))</p>
                            }
                        </div>
                    </footer>
                }

                @if site.config.search.is_some() {
                    (super::search_modal::search_modal(&search_index_url))
                }

                <script src=(app_js) defer></script>
            </body>
        </html>
    }
}

/// MathJax (SVG) with the standard delimiters; loaded only on pages
/// whose markdown contained math. pre/code are skipped so code fences
/// stay literal.
pub const MATHJAX_SCRIPT: &str = r#"<script>
window.MathJax = {
  tex: {
    inlineMath: [["\(", "\)"]],
    displayMath: [["\[", "\]"]],
    skipHtmlTags: ["script", "noscript", "style", "textarea", "pre", "code"]
  },
  svg: { fontCache: "global" }
};
</script>
<script defer src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-svg.js"></script>"#;


/// Serialize the `[[head]]` config tags to raw HTML.
pub(crate) fn serialize_head_tags_to_string(tags: &[crate::config::HeadTag]) -> String {
    serialize_head_tags(tags).into_inner()
}

pub(crate) fn serialize_head_tags(tags: &[crate::config::HeadTag]) -> Raw<String> {
    let mut out = String::new();
    for tag in tags {
        out.push('<');
        out.push_str(&tag.tag);
        for (k, v) in &tag.attrs {
            out.push_str(&format!(" {}=\"{}\"", k, v.replace('"', "&quot;")));
        }
        if tag.tag == "script" || tag.children.is_some() {
            out.push('>');
            out.push_str(tag.children.as_deref().unwrap_or(""));
            out.push_str(&format!("</{}>", tag.tag));
        } else {
            out.push_str("/>");
        }
    }
    Raw::dangerously_create(out)
}

/// Blocking pre-paint script: resolve the stored appearance preference
/// (Alpine cannot run before first paint). Behavior follows the
/// `appearance` setting.
pub fn appearance_script(appearance: &crate::config::Appearance) -> String {
    use crate::config::Appearance as A;
    let body = match appearance {
        A::Toggleable { default_dark: false } => r#"
        var t = localStorage.getItem('vitepress-theme-appearance') || 'auto';
        var dark = t === 'dark' || (t === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);
        document.documentElement.classList.toggle('dark', dark);"#.to_string(),
        A::Toggleable { default_dark: true } => r#"
        var s = localStorage.getItem('vitepress-theme-appearance');
        var dark = s !== 'light';
        document.documentElement.classList.toggle('dark', dark);"#.to_string(),
        A::LightOnly => r#"
        document.documentElement.classList.remove('dark');"#.to_string(),
        A::ForceDark => r#"
        document.documentElement.classList.add('dark');"#.to_string(),
        A::ForceAuto => r#"
        document.documentElement.classList.toggle('dark', window.matchMedia('(prefers-color-scheme: dark)').matches);"#.to_string(),
    };
    format!(
        r#"<script>(function () {{ try {{{body}}} catch (e) {{}} }})(); if (/Mac|iPhone|iPad/.test(navigator.platform)) document.documentElement.classList.add('mac');</script>"#
    )
}
