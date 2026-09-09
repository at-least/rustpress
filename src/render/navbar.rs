//! VPNavBar + VPNavScreen (ported from partials/navbar.html and
//! nav_screen.html). `.top` / `.screen-open` / `.open` states are toggled
//! by the JS bundle (vanilla now, Alpine in stage 5).

use hypertext::prelude::*;
use hypertext::Raw;

use super::icons::{icon, social_icon};
use super::Site;
use crate::config::NavItem;

fn nav_item_active(item: &NavItem, current_url: &str) -> bool {
    match &item.active_match {
        Some(am) if !am.is_empty() => current_url.starts_with(am.as_str()),
        _ => match &item.link {
            Some(link) => current_url.contains(link.as_str()),
            None => false,
        },
    }
}

pub fn navbar<'a>(site: &'a Site, current_url: &'a str, is_home: bool, has_sidebar: bool) -> impl Renderable + 'a {
    let home = site.url("/");
    let site_title = site
        .config
        .title
        .clone()
        .unwrap_or_default();
    let logo = site.config.logo.as_ref().map(|l| site.url(l));
    let ask_ai = site.config.ask_ai_url.clone();
    let nav: Vec<&NavItem> = site.config.nav.iter().collect();
    let socials: Vec<_> = site.config.social_links.clone();
    let has_search = site.config.search.is_some();
    let show_social = !socials.is_empty();

    rsx! {
        <div class=(format!(
            "top relative z-[1] h-(--vp-nav-height) pointer-events-none whitespace-nowrap [--vp-nav-col-offset:0px] [&::before]:content-[''] [&::before]:absolute [&::before]:top-0 [&::before]:right-0 [&::before]:bottom-0 [&::before]:left-(--vp-nav-col-offset) [&::before]:z-[-1] [&::before]:bg-(--vp-nav-bg-color) [&::before]:[backdrop-filter:var(--vp-nav-backdrop-filter)] [&::before]:transition-colors [&::before]:duration-[250ms]{}{}{}",
            if is_home { " max-lg:[&.home:not(.screen-open)::before]:bg-transparent" } else { "" },
            if has_sidebar { " lg:[--vp-nav-col-offset:var(--vp-sidebar-width)] 2xl:[--vp-nav-col-offset:calc((100%-var(--vp-layout-max-width))/2+var(--vp-sidebar-width))]" } else { "" },
            if is_home { " lg:[&.home.top::before]:bg-(--vp-nav-home-bg-color) lg:[&.home.top::before]:[backdrop-filter:none]" } else { "" },
        )) id="VPNavBar">
            <div class="py-0 pr-2 pl-6 md:pr-8 md:pl-8">
                <div class="flex justify-between mx-auto max-w-[calc(var(--vp-layout-max-width)-4rem)] h-(--vp-nav-height) pointer-events-none">
                    <div class=(format!(
                        "title min-w-0 pointer-events-none [&_*]:pointer-events-auto{}",
                        if has_sidebar { " lg:max-w-[calc(var(--vp-sidebar-width)-2rem)]" } else if !is_home { " lg:min-w-[calc(var(--vp-sidebar-width)-2rem)]" } else { "" }
                    ))>
                        <div class=(format!(
                            "flex items-center{}",
                            if has_sidebar { " lg:max-w-[calc(var(--vp-sidebar-width)-4rem)]" } else { "" }
                        ))>
                            <a class=(format!(
                                "flex items-center w-full h-(--vp-nav-height) text-base font-semibold text-text-1 transition-opacity duration-[250ms] [&>span]:overflow-hidden [&>span]:text-ellipsis{}",
                                if has_sidebar { " lg:shrink-0" } else { "" }
                            )) href=(home)>
                                @if let Some(logo) = logo.clone() {
                                    <img class="shrink-0 mr-0 h-(--vp-nav-logo-height)" src=(logo) alt="Logo">
                                }
                                <span>(site_title)</span>
                            </a>
                        </div>
                    </div>

                    <div class="content grow shrink-0 pointer-events-none [&_*]:pointer-events-auto md:shrink md:min-w-0">
                        <div class="content-body relative flex justify-end items-center h-(--vp-nav-height)">
                            <div class="flex items-center md:gap-2 md:pl-6 lg:pl-8">
                                @if has_search {
                                    <button type="button" class="flex items-center gap-2 h-(--vp-nav-height) px-[0.875rem] py-2 text-[1.25rem] cursor-pointer md:h-auto md:py-2 md:px-3 md:bg-bg-alt md:rounded-lg md:text-[0.875rem] md:leading-none md:text-text-2" id="VPSearchButton" aria-keyshortcuts="/ control+k meta+k">
                                        (icon("search", ""))
                                        <span class="hidden md:inline md:text-[0.8125rem]">"Search"</span>
                                        <span class="hidden md:flex md:items-center md:gap-1 md:px-[0.375rem] md:py-1 md:border md:border-divider md:rounded-[0.25rem] md:text-[0.75rem]" aria-hidden="true">
                                            <kbd class="font-[inherit] font-medium before:content-['Ctrl'] [.mac_&]:before:content-['⌘']"></kbd>
                                            <kbd class="font-[inherit] font-medium before:content-['K']"></kbd>
                                        </span>
                                    </button>
                                }
                                @if let Some(ask_ai) = ask_ai.clone() {
                                    <a class="flex items-center h-(--vp-nav-height) px-[0.875rem] py-2 text-[1.25rem] md:h-auto md:p-[0.71875rem] md:transition-colors md:duration-300 md:bg-bg-alt md:rounded-lg md:text-[0.9375rem] md:text-text-2 md:hover:text-brand-1" href=(ask_ai) target="_blank" rel="noopener" aria-label="Ask AI">
                                        (icon("sparkles", ""))
                                    </a>
                                }
                            </div>

                            @if !nav.is_empty() {
                                <nav class="relative hidden md:flex md:grow md:justify-end min-w-0" aria-label="Main Navigation">
                                    <ul class="flex justify-end">
                                        @for item in &nav {
                                            <li>
                                                (Raw::dangerously_create(nav_entry(site, item, current_url)))
                                            </li>
                                        }
                                    </ul>
                                </nav>
                            }

                            <div class="flex items-center">
                                (appearance_switch("VPSwitchAppearance"))
                            </div>

                            @if show_social {
                                <div class="hidden md:flex md:items-center -mr-2 before:content-[''] before:ml-4 before:mr-2 before:w-px before:h-6 before:bg-divider">
                                    <div class="flex">
                                        @for s in &socials {
                                            <a class="flex justify-center items-center w-9 h-9 text-text-2 transition-colors duration-500 hover:text-text-1 hover:duration-[250ms]" href=(s.link.clone()) aria-label=(social_label(&s.icon)) target="_blank" rel="noopener">
                                                (social_link_icon(&s.icon))
                                            </a>
                                        }
                                    </div>
                                </div>
                            }

                            <button type="button" class="group flex justify-center items-center w-12 h-(--vp-nav-height) cursor-pointer md:hidden" id="VPNavBarHamburger" aria-label="Menu" aria-expanded="false">
                                <span class="relative w-4 h-[0.875rem] overflow-hidden" aria-hidden="true">
                                    <span class="absolute w-4 h-[2px] top-0 left-0 bg-text-1 [transition:top_0.25s,background-color_0.5s,transform_0.25s] group-hover:translate-x-1 group-[.active]:top-[0.375rem] group-[.active]:translate-x-0! group-[.active]:rotate-[225deg] group-hover:group-[.active]:bg-text-2 group-hover:group-[.active]:[transition:top_0.25s,background-color_0.25s,transform_0.25s]"></span>
                                    <span class="absolute w-4 h-[2px] top-[0.375rem] left-0 translate-x-2 bg-text-1 [transition:top_0.25s,background-color_0.5s,transform_0.25s] group-hover:translate-x-0 group-[.active]:top-[0.375rem] group-[.active]:translate-x-4! group-hover:group-[.active]:bg-text-2 group-hover:group-[.active]:[transition:top_0.25s,background-color_0.25s,transform_0.25s]"></span>
                                    <span class="absolute w-4 h-[2px] top-[0.75rem] left-0 translate-x-[0.25rem] bg-text-1 [transition:top_0.25s,background-color_0.5s,transform_0.25s] group-hover:translate-x-2 group-[.active]:top-[0.375rem] group-[.active]:translate-x-0! group-[.active]:rotate-[135deg] group-hover:group-[.active]:bg-text-2 group-hover:group-[.active]:[transition:top_0.25s,background-color_0.25s,transform_0.25s]"></span>
                                </span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <div class=(format!(
                "relative z-[-1] [transform:translateZ(0)] w-full h-px pl-[var(--vp-nav-col-offset)]{}",
                if has_sidebar { " before:content-[''] before:absolute before:top-0 before:left-[calc(var(--vp-nav-col-offset)-var(--vp-sidebar-width)+2rem)] before:w-[calc(var(--vp-sidebar-width)-4rem)] before:h-px before:bg-divider" } else { "" }
            ))>
                <div class=(format!(
                    "w-full h-px transition-colors duration-[250ms] {}",
                    if !is_home { "bg-(--vp-nav-divider-color)" } else { "bg-transparent lg:bg-(--vp-nav-divider-color) lg:[.top_&]:bg-transparent" }
                ))></div>
            </div>
        </div>
    }
}

fn nav_entry<'a>(site: &'a Site, item: &'a NavItem, current_url: &'a str) -> String {
    let active = nav_item_active(item, current_url);
    let html = if item.items.is_empty() {
        let href = site.url(&item.link.clone().unwrap_or_default());
        let cls = format!(
            "flex items-center min-h-(--vp-nav-height) px-3 leading-normal text-[0.875rem] font-medium text-text-1 transition-colors duration-[250ms] hover:text-brand-1{}",
            if active { " text-brand-1" } else { "" }
        );
        let text = item.text.clone();
        rsx! {
            <a class=(cls) href=(href)>
                <span>(text)</span>
            </a>
        }
        .render()
        .into_inner()
    } else {
        let group_cls = format!(
            "VPFlyout relative group/flyout hover:text-brand-1 transition-colors duration-[250ms]{}",
            if active { " active" } else { "" }
        );
        let label_cls = format!(
            "flex items-center leading-(--vp-nav-height) text-[0.875rem] font-medium transition-colors duration-[250ms]{}",
            if active { " text-brand-1 group-hover/flyout:text-brand-2" } else { " text-text-1 group-hover/flyout:text-text-2" }
        );
        let text = item.text.clone();
        let children: Vec<&NavItem> = item.items.iter().collect();
        rsx! {
            <div class=(group_cls)>
                <button type="button" class="button flex items-center px-3 h-(--vp-nav-height) text-text-1 transition-colors duration-500 cursor-pointer" aria-expanded="false" aria-haspopup="true">
                    <span class=(label_cls)>
                        <span>(text)</span>
                        (icon("chevron-down", "ml-1 size-[0.875rem]"))
                    </span>
                </button>
                <div class="menu absolute top-[calc(var(--vp-nav-height)/2+1.25rem)] right-0 opacity-0 invisible transition-[opacity,visibility] duration-[250ms] group-hover/flyout:opacity-100 group-hover/flyout:visible group-focus-within/flyout:opacity-100 group-focus-within/flyout:visible group-[.open]/flyout:opacity-100 group-[.open]/flyout:visible">
                    <div class="rounded-xl p-3 min-w-32 border border-divider bg-bg-elv shadow-3 transition-colors duration-500 max-h-[calc(100vh-var(--vp-nav-height))] overflow-y-auto">
                        <ul>
                            @for child in &children {
                                <li>
                                    <a class=(format!(
                                        "block rounded-md px-3 leading-[2.2857143] text-[0.875rem] font-medium text-left whitespace-nowrap text-text-1 transition-[background-color,color] duration-[250ms] hover:text-brand-1 hover:bg-default-soft{}",
                                        if child.link.as_deref().is_some_and(|l| current_url.contains(l)) { " text-brand-1" } else { "" }
                                    )) href=(site.url(&child.link.clone().unwrap_or_default()))>(child.text.clone())</a>
                                </li>
                            }
                        </ul>
                    </div>
                </div>
            </div>
        }
        .render()
        .into_inner()
    };
    html
}

/// VPSwitchAppearance — one instance in the navbar, one in the nav
/// screen; the JS bundle syncs `aria-checked` across both.
fn appearance_switch(id: &'static str) -> impl Renderable {
    rsx! {
        <button type="button" id=(id) class="VPSwitch VPSwitchAppearance relative block w-10 h-[1.375rem] shrink-0 rounded-[0.6875rem] border border-(--vp-input-border-color) bg-(--vp-input-switch-bg-color) transition-colors duration-[250ms] hover:border-brand-1 cursor-pointer" role="switch" aria-label="Appearance" aria-checked="false" title="Toggle dark mode">
            <span class="absolute top-px left-px w-[1.125rem] h-[1.125rem] rounded-full bg-(--vp-c-neutral-inverse) shadow-1 transition-transform duration-[250ms] dark:translate-x-[1.125rem]">
                <span class="relative block w-[1.125rem] h-[1.125rem] rounded-full overflow-hidden">
                    (icon("sun", "absolute top-[0.1875rem] left-[0.1875rem] size-3 text-text-2 dark:text-text-1 transition-opacity duration-[250ms] opacity-100 dark:opacity-0"))
                    (icon("moon", "absolute top-[0.1875rem] left-[0.1875rem] size-3 text-text-2 dark:text-text-1 transition-opacity duration-[250ms] opacity-0 dark:opacity-100"))
                </span>
            </span>
        </button>
    }
    .render()
    .into_inner()
}

/// VPNavScreen — the full-screen mobile menu.
pub fn nav_screen<'a>(site: &'a Site, current_url: &'a str) -> impl Renderable + 'a {
    let nav: Vec<&NavItem> = site.config.nav.iter().collect();
    let socials = site.config.social_links.clone();
    let has_nav = !nav.is_empty();
    rsx! {
        <div class="fixed inset-0 pt-[calc(var(--vp-nav-height)+var(--vp-layout-top-height,0px)+1px)] pr-8 pl-8 bg-(--vp-nav-screen-bg-color) w-full overflow-y-auto overscroll-contain transition-colors duration-[250ms] pointer-events-auto opacity-100 md:hidden" id="VPNavScreen" hidden>
            <div class="mx-auto pt-6 pb-24 max-w-[18rem]">
                <nav class="menu" aria-label="Main Navigation">
                    <ul>
                        @for item in &nav {
                            <li>
                                (Raw::dangerously_create(screen_entry(site, item, current_url)))
                            </li>
                        }
                    </ul>
                </nav>

                <div class=(format!("appearance flex justify-center items-center pt-3{}", if has_nav { " mt-4" } else { "" }))>
                    <span class="label mr-3 text-[0.875rem] font-medium text-text-1">"Appearance"</span>
                    (appearance_switch("VPSwitchAppearanceScreen"))
                </div>

                @if !socials.is_empty() {
                    <div class="social-links mt-4">
                        <div class="flex">
                            @for s in &socials {
                                <a class="flex justify-center items-center w-9 h-9 text-text-2 transition-colors duration-500 hover:text-text-1 hover:duration-[250ms]" href=(s.link.clone()) aria-label=(social_label(&s.icon)) target="_blank" rel="noopener">
                                    (social_link_icon(&s.icon))
                                </a>
                            }
                        </div>
                    </div>
                }
            </div>
        </div>
    }
}

fn screen_entry<'a>(site: &'a Site, item: &'a NavItem, current_url: &'a str) -> String {
    let active = nav_item_active(item, current_url);
    let html = if item.items.is_empty() {
        let cls = format!(
            "block border-b border-divider pt-3 pb-[0.6875rem] leading-[1.7142857] text-[0.875rem] font-medium text-text-1 transition-colors duration-[250ms] hover:text-brand-1{}",
            if active { " text-brand-1" } else { "" }
        );
        let href = site.url(&item.link.clone().unwrap_or_default());
        let text = item.text.clone();
        rsx! {
            <a class=(cls) href=(href)>(text)</a>
        }
        .render()
        .into_inner()
    } else {
        let group_cls = format!("VPNavScreenMenuGroup group{}", if active { " active" } else { "" });
        let text = item.text.clone();
        let children: Vec<&NavItem> = item.items.iter().collect();
        rsx! {
            <div class=(group_cls)>
                <button type="button" class="button cursor-pointer" aria-expanded="false">
                    <span class="button-text">(text)</span>
                    (icon("plus", "button-icon"))
                </button>
                <ul class="items" hidden>
                    @for child in &children {
                        <li>
                            <a class="block border-b border-divider pt-3 pb-[0.6875rem] leading-[1.7142857] text-[0.875rem] font-medium text-text-1 transition-colors duration-[250ms] hover:text-brand-1" href=(site.url(&child.link.clone().unwrap_or_default()))>(child.text.clone())</a>
                        </li>
                    }
                </ul>
            </div>
        }
        .render()
        .into_inner()
    };
    html
}

fn social_label(icon: &crate::config::SocialIcon) -> String {
    match icon {
        crate::config::SocialIcon::Name(n) => n.clone(),
        crate::config::SocialIcon::Svg { .. } => "link".into(),
    }
}

fn social_link_icon(icon: &crate::config::SocialIcon) -> hypertext::Raw<String> {
    match icon {
        crate::config::SocialIcon::Name(n) => social_icon(n, "size-5"),
        crate::config::SocialIcon::Svg { svg } => hypertext::Raw::dangerously_create(svg.clone()),
    }
}
