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
    pub has_sidebar: bool,
    /// Current page URL (canonical, base-free) for active matching.
    pub current_url: &'a str,
}

/// The whole HTML document; `content` is the VPContent body.
pub fn layout<'a>(site: &'a Site, shell: &Shell<'a>, headings: &'a [crate::markdown::Heading], content: impl Renderable + 'a) -> impl Renderable + 'a {
    let lang = site.config.lang.clone();
    let title = shell.title.clone();
    let description = shell.description.clone();
    let syntax_css = site.url("syntax.css");
    let main_css = site.url("main.css");
    let app_js = site.url("js/app.js");
    let is_home = shell.is_home;
    let has_sidebar = shell.has_sidebar;
    let current_url = shell.current_url.to_string();
    let footer = site.config.footer.clone();
    let show_footer = footer.is_some();
    let footer_message = footer.as_ref().and_then(|f| f.message.clone());
    let footer_copyright = footer.as_ref().and_then(|f| f.copyright.clone());

    let content_class = if has_sidebar {
        "grow shrink-0 m-0 w-full lg:mt-[var(--vp-layout-top-height,0px)] lg:pt-(--vp-nav-height) lg:pl-(--vp-sidebar-width) 2xl:pr-[calc((100%-var(--vp-layout-max-width))/2)] 2xl:pl-[calc((100%-var(--vp-layout-max-width))/2+var(--vp-sidebar-width))]"
    } else {
        "grow shrink-0 w-full mx-auto mt-[var(--vp-layout-top-height,0px)] max-w-full lg:pt-(--vp-nav-height)"
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
                (Raw::dangerously_create(FOUC_SCRIPT.to_string()))
            </head>
            <body class="font-sans bg-bg text-text-1 antialiased [text-rendering:optimizeLegibility] [-moz-osx-font-smoothing:grayscale] [text-autospace:normal] [text-spacing-trim:normal]">
                <a class="sr-only" href="#main">"Skip to content"</a>

                <div class="fixed inset-0 z-(--vp-z-index-backdrop) bg-(--vp-backdrop-bg-color) transition-opacity duration-500 xl:hidden" id="VPBackdrop" hidden></div>

                <header class="relative top-[var(--vp-layout-top-height,0px)] left-0 z-(--vp-z-index-nav) w-full pointer-events-none lg:fixed">
                    (navbar::navbar(site, &current_url, is_home, has_sidebar))
                    (navbar::nav_screen(site, &current_url))
                </header>

                (Raw::dangerously_create(super::local_nav::local_nav(site, is_home, has_sidebar, headings)))
                (Raw::dangerously_create(super::sidebar::sidebar(site, &current_url)))

                <div class=(content_class) id="VPContent">
                    (content)
                </div>

                @if show_footer {
                    <footer class=(format!("relative z-(--vp-z-index-footer) border-t border-gutter py-8 px-6 bg-bg md:px-8{}", if has_sidebar { " hidden" } else { "" }))>
                        <div class="mx-auto max-w-(--vp-layout-max-width) text-center [&_a]:underline [&_a]:underline-offset-[0.125rem] [&_a]:transition-colors [&_a]:duration-[250ms] [&_a:hover]:text-text-1">
                            @if let Some(message) = footer_message.clone() {
                                <p class="leading-[1.7142857] text-[0.875rem] font-medium text-text-2">(message)</p>
                            }
                            @if let Some(copyright) = footer_copyright.clone() {
                                <p class="leading-[1.7142857] text-[0.875rem] font-medium text-text-2">(copyright)</p>
                            }
                        </div>
                    </footer>
                }

                @if site.config.search.is_some() {
                    (super::search_modal::search_modal())
                }

                <script src=(app_js) defer></script>
            </body>
        </html>
    }
}

/// Blocking pre-paint script: resolve the stored appearance preference
/// (Alpine cannot run before first paint).
pub const FOUC_SCRIPT: &str = r#"<script>
      (function () {
        try {
          var t = localStorage.getItem('vitepress-theme-appearance') || 'auto';
          var dark = t === 'dark' || (t === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);
          document.documentElement.classList.toggle('dark', dark);
        } catch (e) {}
      })();
      if (/Mac|iPhone|iPad/.test(navigator.platform)) document.documentElement.classList.add('mac');
    </script>"#;
