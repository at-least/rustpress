//! VPHome (ported from templates/index.html): hero (name/text/tagline,
//! actions, image with the glow blob) and the features grid. Sponsors
//! are a later stage (core-first scope).

use hypertext::{prelude::*, Raw};

use super::icons::icon;
use super::Site;
use crate::config::SocialIcon;
use crate::content::{Feature, HeroAction};
use crate::content::Page;

pub fn home_page<'a>(site: &'a Site, page: &'a Page) -> impl Renderable + 'a {
    let hero = page.front.hero.clone().unwrap_or_default();
    let features: Vec<_> = page.front.features.clone();
    let has_image = hero.image.is_some();
    let hero_image = hero.image.clone();

    let grid = match features.len() {
        2 => " sm:w-1/2",
        3 => " md:w-1/3",
        n if n > 3 && n % 3 == 0 => " sm:w-1/2 md:w-1/3",
        n if n > 3 => " sm:w-1/2 lg:w-1/4",
        _ => "",
    };

    rsx! {
        <div class="overflow-x-clip mb-24 md:mb-32">
            <div class="-mt-[calc(var(--vp-nav-height)+var(--vp-layout-top-height,0px))] pt-[calc(var(--vp-nav-height)+var(--vp-layout-top-height,0px)+3rem)] px-6 pb-12 sm:pt-[calc(var(--vp-nav-height)+var(--vp-layout-top-height,0px)+5rem)] sm:px-12 sm:pb-16 lg:px-16 lg:pb-16">
                <div class=(format!(
                    "flex flex-col mx-auto max-w-[72rem] lg:flex-row{}",
                    if has_image { " text-center lg:text-left" } else { "" }
                ))>
                    <div class=(format!(
                        "relative z-10 order-2 grow shrink-0{} lg:order-1 lg:w-[calc((100%/3)*2)]",
                        if has_image { " lg:max-w-[37rem]" } else { "" }
                    ))>
                        <h1 class="heading flex flex-col">
                            @if let Some(name) = hero.name.clone() {
                                <span class=(format!(
                                    "w-fit max-w-[24.5rem] tracking-[-0.025rem] leading-[1.25] text-[2rem] font-bold whitespace-pre-wrap text-(--vp-home-hero-name-color) [background-image:var(--vp-home-hero-name-background)] bg-clip-text [-webkit-text-fill-color:var(--vp-home-hero-name-color)]{} sm:max-w-[36rem] sm:leading-[1.1666667] sm:text-[3rem] lg:leading-[1.1428571] lg:text-[3.5rem]",
                                    if has_image { " mx-auto lg:mx-0" } else { "" }
                                ))>(name)</span>
                            }
                            <span class=(format!(
                                "w-fit max-w-[24.5rem] tracking-[-0.025rem] leading-[1.25] text-[2rem] font-bold whitespace-pre-wrap{} sm:max-w-[36rem] sm:leading-[1.1666667] sm:text-[3rem] lg:leading-[1.1428571] lg:text-[3.5rem]",
                                if has_image { " mx-auto lg:mx-0" } else { "" }
                            ))>(hero.text.clone().unwrap_or_default())</span>
                        </h1>
                        @if let Some(tagline) = hero.tagline.clone() {
                            <p class=(format!(
                                "pt-2 max-w-[24.5rem] leading-[1.5555556] text-[1.125rem] font-medium whitespace-pre-wrap text-text-2{} sm:pt-3 sm:max-w-[36rem] sm:leading-[1.6] sm:text-[1.25rem] lg:leading-[1.5] lg:text-[1.5rem]",
                                if has_image { " mx-auto lg:mx-0" } else { "" }
                            ))>(tagline)</p>
                        }
                        @if !hero.actions.is_empty() {
                            <div class=(format!(
                                "flex flex-wrap m-[-0.375rem] pt-6{} sm:pt-8",
                                if has_image { " justify-center lg:justify-start" } else { "" }
                            ))>
                                @for action in &hero.actions {
                                    <div class="shrink-0 p-[0.375rem]">
                                        (Raw::dangerously_create(hero_button(site, action)))
                                    </div>
                                }
                            </div>
                        }
                    </div>

                    @if let Some(image) = hero_image.clone() {
                        <div class="order-1 m-[-4.75rem_-1.5rem_-3rem] sm:m-[-6.75rem_-1.5rem_-3rem] lg:order-2 lg:grow lg:m-0 lg:min-h-full">
                            <div class="relative mx-auto w-80 h-80 sm:w-[24.5rem] sm:h-[24.5rem] lg:flex lg:justify-center lg:items-center lg:w-full lg:h-full lg:[transform:translate(-2rem,-2rem)]">
                                <div class="absolute top-1/2 left-1/2 rounded-full w-48 h-48 [background-image:var(--vp-home-hero-image-background-image)] [filter:var(--vp-home-hero-image-filter)] [transform:translate(-50%,-50%)] sm:w-64 sm:h-64 lg:w-80 lg:h-80"></div>
                                (Raw::dangerously_create(hero_image_html(site, &image)))
                            </div>
                        </div>
                    }
                </div>
            </div>

            @if !features.is_empty() {
                <div class="relative px-6 sm:px-12 lg:px-16">
                    <div class="mx-auto max-w-[72rem]">
                        <ul class="flex flex-wrap m-[-0.5rem]">
                            @for feature in &features {
                                <li class=(format!("p-2 w-full{grid}"))>
                                    (Raw::dangerously_create(feature_card(site, feature)))
                                </li>
                            }
                        </ul>
                    </div>
                </div>
            }
        </div>
    }
}

/// The hero `<img>`(s): one image, or a light/dark pair switched by the
/// color scheme.
fn hero_image_html(site: &Site, image: &crate::content::HeroImage) -> String {
    fn esc(s: &str) -> String {
        s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")
    }
    fn img(site: &Site, src: &str, alt: &str, extra: &str) -> String {
        format!(
            r#"<img class="absolute top-1/2 left-1/2 max-w-48 max-h-48 w-full h-full object-contain [transform:translate(-50%,-50%)] [filter:drop-shadow(-2px_4px_6px_rgba(0,0,0,0.2))] p-[1.125rem] sm:max-w-64 sm:max-h-64 lg:max-w-80 lg:max-h-80{extra}" src="{}" alt="{}">"#,
            esc(&site.url(src)),
            esc(alt)
        )
    }
    match image {
        crate::content::HeroImage::Simple(src) => img(site, src, "", ""),
        crate::content::HeroImage::Detailed { src, alt } => {
            img(site, src, alt.as_deref().unwrap_or(""), "")
        }
        crate::content::HeroImage::Dual { light, dark, alt } => [
            img(site, light, alt.as_deref().unwrap_or(""), " dark:hidden"),
            img(site, dark, alt.as_deref().unwrap_or(""), " hidden dark:block"),
        ]
        .concat(),
    }
}

/// VPFeature card. With `link` the whole card is the anchor (upstream
/// VPFeature); `target`/`rel` pass through on both features and hero
/// action buttons.
fn feature_card(site: &Site, feature: &Feature) -> String {
    let icon_html = feature.icon.as_ref().map(|i| feature_icon_html(site, i));
    let body = rsx! {
        <article class="VPFeature flex flex-col p-6 h-full">
            @if let Some(icon_html) = icon_html.clone() {
                <div class="icon flex justify-center items-center mb-5 rounded-md bg-default-soft w-12 h-12 text-[1.5rem] transition-colors duration-[250ms]">
                    (Raw::dangerously_create(icon_html))
                </div>
            }
            <h2 class="leading-[1.5] text-[1rem] font-semibold">(feature.title.clone())</h2>
            @if let Some(details) = feature.details.clone() {
                <p class="grow pt-2 leading-[1.7142857] text-[0.875rem] font-medium text-text-2">(details)</p>
            }
            @if let Some(link_text) = feature.link_text.clone() {
                <div class="pt-2">
                    <p class="flex items-center text-[0.875rem] font-medium text-brand-1">
                        (link_text)
                        (icon("chevron-right", "ml-[0.375rem] size-[0.875rem]"))
                    </p>
                </div>
            }
        </article>
    }
    .render()
    .into_inner();
    let card_cls = "block border border-bg-soft rounded-xl h-full bg-bg-soft transition-colors duration-[250ms]";
    match &feature.link {
        Some(link) => {
            let href = resolve_action_link(site, link);
            let mut open = format!(r#"<a class="{card_cls} cursor-pointer" href="{}""#, escape_attr(&href));
            if let Some(t) = &feature.target {
                open.push_str(&format!(r#" target="{}""#, escape_attr(t)));
            }
            if let Some(r) = &feature.rel {
                open.push_str(&format!(r#" rel="{}""#, escape_attr(r)));
            }
            format!("{open}>{body}</a>")
        }
        None => format!(r#"<div class="{card_cls}">{body}</div>"#),
    }
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")
}

/// `features[].icon`: emoji/HTML text, an image, or a light/dark pair.
fn feature_icon_html(site: &Site, icon: &crate::content::FeatureIcon) -> String {
    fn esc(s: &str) -> String {
        s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")
    }
    fn img(
        site: &Site,
        src: &str,
        alt: &str,
        width: Option<u32>,
        height: Option<u32>,
        extra: &str,
    ) -> String {
        let mut out = format!(
            r#"<img src="{}" alt="{}" class="w-12 h-12 rounded-md{extra}""#,
            esc(&site.url(src)),
            esc(alt)
        );
        if let Some(w) = width {
            out.push_str(&format!(r#" width="{w}""#));
        }
        if let Some(h) = height {
            out.push_str(&format!(r#" height="{h}""#));
        }
        out.push('>');
        out
    }
    match icon {
        // VitePress treats `features[].icon` strings as raw inline HTML —
        // the upstream theme ships `<span class="memo">`-style slots that
        // its custom CSS paints into icons, so they must pass through
        // unescaped
        crate::content::FeatureIcon::Text(text) => text.clone(),
        crate::content::FeatureIcon::Image { src, alt, width, height } => {
            img(site, src, alt.as_deref().unwrap_or(""), *width, *height, "")
        }
        crate::content::FeatureIcon::Dual { light, dark, alt, width, height } => [
            img(site, light, alt.as_deref().unwrap_or(""), *width, *height, " dark:hidden"),
            img(site, dark, alt.as_deref().unwrap_or(""), *width, *height, " hidden dark:block"),
        ]
        .concat(),
    }
}

/// VPButton — `theme`: brand | alt | sponsor, build-time known so the
/// per-theme custom properties are emitted directly. `target`/`rel`
/// pass through when set (built as a string: rsx renders `None` attrs
/// as empty `target=""`, upstream omits them).
fn hero_button(site: &Site, action: &HeroAction) -> String {
    let theme_cls = match action.theme.as_deref() {
        Some("alt") => "border-(--vp-button-alt-border) text-(--vp-button-alt-text) bg-(--vp-button-alt-bg) hover:border-(--vp-button-alt-hover-border) hover:text-(--vp-button-alt-hover-text) hover:bg-(--vp-button-alt-hover-bg) active:border-(--vp-button-alt-active-border) active:text-(--vp-button-alt-active-text) active:bg-(--vp-button-alt-active-bg)",
        Some("sponsor") => "border-(--vp-button-sponsor-border) text-(--vp-button-sponsor-text) bg-(--vp-button-sponsor-bg) hover:border-(--vp-button-sponsor-hover-border) hover:text-(--vp-button-sponsor-hover-text) hover:bg-(--vp-button-sponsor-hover-bg) active:border-(--vp-button-sponsor-active-border) active:text-(--vp-button-sponsor-active-text) active:bg-(--vp-button-sponsor-active-bg)",
        _ => "border-(--vp-button-brand-border) text-(--vp-button-brand-text) bg-(--vp-button-brand-bg) hover:border-(--vp-button-brand-hover-border) hover:text-(--vp-button-brand-hover-text) hover:bg-(--vp-button-brand-hover-bg) active:border-(--vp-button-brand-active-border) active:text-(--vp-button-brand-active-text) active:bg-(--vp-button-brand-active-bg)",
    };
    let href = resolve_action_link(site, &action.link);
    let cls = format!("inline-flex items-center justify-center {theme_cls} h-10 rounded-[1.25rem] px-5 text-[0.875rem] border text-center font-semibold whitespace-nowrap no-underline transition-colors duration-[250ms] active:duration-100");
    let mut open = format!(r#"<a class="{cls}" href="{}""#, escape_attr(&href));
    if let Some(t) = &action.target {
        open.push_str(&format!(r#" target="{}""#, escape_attr(t)));
    }
    if let Some(r) = &action.rel {
        open.push_str(&format!(r#" rel="{}""#, escape_attr(r)));
    }
    format!(
        "{open}>{}</a>",
        action.text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
    )
}

/// Hero action links are written relative to the page (VitePress
/// convention: `./guide/what-is-vitepress` on the home page).
pub fn resolve_action_link(site: &Site, link: &str) -> String {
    if link.starts_with("http://") || link.starts_with("https://") {
        return link.to_string();
    }
    // a link naming a page gets the page's URL, like content links do
    let rooted = if link.starts_with('/') {
        link.to_string()
    } else {
        format!("/{link}")
    };
    if let Some(url) = crate::markdown::resolve_root(&rooted, &site.content) {
        return site.url(&url);
    }
    if link.starts_with('/') {
        return site.url(link);
    }
    // ./x or x from the site root → /x (static files and the like)
    let mut segs: Vec<&str> = Vec::new();
    for seg in link.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                segs.pop();
            }
            s => segs.push(s),
        }
    }
    let clean = segs
        .iter()
        .map(|s| s.strip_suffix(".md").unwrap_or(s))
        .collect::<Vec<_>>()
        .join("/");
    site.url(&format!("/{clean}"))
}

/// Unused today (sponsors deferred) but shared by the later stage; keep
/// the social label logic in one place.
pub fn _social_label(icon: &SocialIcon) -> String {
    match icon {
        SocialIcon::Name(n) => n.clone(),
        SocialIcon::Svg { .. } => "link".into(),
    }
}
