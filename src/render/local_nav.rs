//! VPLocalNav (ported from partials/local_nav.html): the sub-xl bar
//! with the mobile menu button and the outline dropdown — or, on a page
//! without outline headers, a "Return to top" button. Hidden on the home
//! page.

use hypertext::{Raw, prelude::*};

use super::escape::{escape_attr, escape_text};
use super::icons::icon;
use crate::markdown::Heading;

pub fn local_nav<'a>(
    cfg: &'a crate::config::SiteConfig,
    is_home: bool,
    has_sidebar: bool,
    outline: Option<(u8, u8)>,
    headings: &'a [Heading],
) -> String {
    if is_home {
        return String::new();
    }
    // upstream's dropdown lists the aside's headers: the outline range
    let headings: Vec<&Heading> = match outline {
        Some((lo, hi)) => headings
            .iter()
            .filter(|h| h.level >= lo && h.level <= hi)
            .collect(),
        None => Vec::new(),
    };
    let has_headers = !headings.is_empty();
    // upstream VPLocalNav: there whenever the page has outline headers or
    // a sidebar; otherwise only once it has scrolled past the navbar, then
    // fixed to the top. Without headers it is `.empty`: hidden from 960px
    let scroll_only = !has_headers && !has_sidebar;
    let outline_label = cfg.outline.label();
    let return_label = cfg.return_to_top_label.clone();
    let menu_label = cfg.sidebar_menu_label.clone();
    let nav_cls = format!(
        "{} top-0 left-0 z-(--vp-z-index-local-nav) w-full pt-[var(--vp-layout-top-height,0px)] border-b border-(--vp-local-nav-divider-color) [&::before]:content-[''] [&::before]:absolute [&::before]:inset-0 [&::before]:z-[-1] [&::before]:bg-(--vp-local-nav-bg-color) [&::before]:[backdrop-filter:var(--vp-nav-backdrop-filter)] [&::before]:transition-colors [&::before]:duration-[250ms] lg:top-(--vp-nav-height) lg:[&::before]:top-[calc(-1*var(--vp-nav-height))]{}{} xl:hidden",
        if scroll_only { "fixed" } else { "sticky" },
        if has_sidebar {
            " lg:pl-(--vp-sidebar-width)"
        } else {
            ""
        },
        if has_headers { "" } else { " lg:hidden" },
    );
    let inner = rsx! {
        <div class="flex justify-between items-center">
            @if has_sidebar {
                <button type="button" class="flex items-center py-[0.75rem] px-6 pb-[0.6875rem] leading-[2] text-[0.75rem] font-medium text-text-2 transition-colors duration-500 hover:text-text-1 hover:duration-[250ms] cursor-pointer lg:hidden md:px-8" id="VPLocalNavMenu" @click="$store.ui.sidebar = true" :aria-expanded=("$store.ui.sidebar.toString()") aria-controls="VPSidebarNav">
                    (icon("align-left", "mr-2 size-[0.875rem]"))
                    <span>(menu_label)</span>
                </button>
            }
            @if has_headers {
                <div id="VPLocalNavOutlineDropdown" x-data="{ open: false }" @click.outside="open = false" @keydown.escape.window="open = false">
                <button type="button" class="group/drop relative block py-[0.75rem] px-6 pb-[0.6875rem] leading-[2] text-[0.75rem] font-medium text-text-2 transition-colors duration-500 hover:text-text-1 hover:duration-[250ms] cursor-pointer [&.open]:text-text-1 md:px-8 lg:text-[0.875rem]" id="VPOutlineDropdownButton" @click="open = !open" :aria-expanded=("open.toString()") :class=("{ open: open }") aria-controls="VPOutlineDropdownItems">
                    <span>(outline_label.clone())</span>
                    (icon("chevron-right", "inline-block align-middle ml-[0.125rem] size-[0.875rem] transition-transform duration-[250ms] group-[.open]/drop:rotate-90 lg:size-[1rem]"))
                </button>
                // picking a heading folds the dropdown away (upstream
                // onItemClick); "Return to top" scrolls this page up
                <div class="absolute top-10 right-4 left-4 grid gap-px border border-border rounded-lg bg-gutter max-h-[calc(var(--vp-vh,100vh)-5.375rem)] overflow-x-hidden overflow-y-auto overscroll-contain shadow-3 lg:right-auto lg:left-[calc(var(--vp-sidebar-width)+2rem)] lg:w-80" id="VPOutlineDropdownItems" x-cloak x-show="open" "x-collapse"="" @click="$event.target.classList.contains('outline-link') && (open = false)">
                    <div class="bg-bg-soft">
                        <a class="block px-4 leading-[3.4285714] text-[0.875rem] font-medium text-brand-1" href="#" @click="open = false; window.scrollTo({ top: 0, left: 0, behavior: 'smooth' })">(return_label.clone())</a>
                    </div>
                    <div class="py-2 bg-bg-soft">
                        (outline_list(&headings))
                    </div>
                </div>
                </div>
            } @else {
                // no headers to list: the dropdown is upstream's plain
                // "Return to top" button
                <div id="VPLocalNavOutlineDropdown">
                    <button type="button" class="relative block py-[0.75rem] px-6 pb-[0.6875rem] leading-[2] text-[0.75rem] font-medium text-text-2 transition-colors duration-500 hover:text-text-1 hover:duration-[250ms] cursor-pointer md:px-8 lg:text-[0.875rem]" @click="window.scrollTo({ top: 0, left: 0, behavior: 'smooth' })">(return_label.clone())</button>
                </div>
            }
        </div>
    }
    .render()
    .into_inner();
    let inner = Raw::dangerously_create(inner);
    if scroll_only {
        rsx! {
            <div class=(nav_cls) id="VPLocalNav" x-data="localNavScroll" x-show="scrolled" x-cloak>(inner)</div>
        }
        .render()
        .into_inner()
    } else {
        rsx! {
            <div class=(nav_cls) id="VPLocalNav" x-data="{}">(inner)</div>
        }
        .render()
        .into_inner()
    }
}

/// The dropdown's outline, nested like the aside's: upstream renders
/// VPDocOutlineItem here without `root`, so the top list is padded like
/// the nested ones.
fn outline_list(headings: &[&Heading]) -> Raw<String> {
    let items: Vec<(u8, String)> = headings
        .iter()
        .map(|h| {
            (
                h.level,
                format!(
                    "<a class=\"outline-link block leading-[2.2857143] text-[0.875rem] font-normal text-text-2 whitespace-nowrap overflow-hidden text-ellipsis transition-colors duration-500 hover:text-text-1 hover:duration-[250ms] [&.active]:text-text-1 scroll-mt-[calc(var(--vp-nav-height)+var(--vp-layout-top-height,0px)+var(--vp-doc-top-height,0px)+2rem)] scroll-mb-12\" href=\"#{}\" title=\"{}\">{}</a>",
                    escape_attr(&h.id),
                    escape_attr(&h.text),
                    escape_text(&h.text),
                ),
            )
        })
        .collect();
    Raw::dangerously_create(crate::markdown::nested_outline_list(
        &items,
        " class=\"pr-4 pl-4\"",
        " class=\"pr-4 pl-4\"",
    ))
}
