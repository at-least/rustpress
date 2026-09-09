//! VPLocalNav (ported from partials/local_nav.html): the sub-xl sticky
//! bar with the mobile menu button and the outline dropdown. Hidden on
//! the home page.

use hypertext::prelude::*;

use super::icons::icon;
use super::Site;
use crate::markdown::Heading;

pub fn local_nav<'a>(site: &'a Site, is_home: bool, has_sidebar: bool, headings: &'a [Heading]) -> String {
    if is_home {
        return String::new();
    }
    let outline_label = site.config.outline.label.clone();
    let root_url = site.url("/");
    let nav_cls = format!(
        "sticky top-0 left-0 z-(--vp-z-index-local-nav) w-full pt-[var(--vp-layout-top-height,0px)] border-b border-(--vp-local-nav-divider-color) [&::before]:content-[''] [&::before]:absolute [&::before]:inset-0 [&::before]:z-[-1] [&::before]:bg-(--vp-local-nav-bg-color) [&::before]:[backdrop-filter:var(--vp-nav-backdrop-filter)] [&::before]:transition-colors [&::before]:duration-[250ms] lg:top-(--vp-nav-height) lg:[&::before]:top-[calc(-1*var(--vp-nav-height))]{} xl:hidden",
        if has_sidebar { " lg:pl-(--vp-sidebar-width)" } else { "" }
    );
    rsx! {
        <div class=(nav_cls) id="VPLocalNav">
            <div class="flex justify-between items-center">
                @if has_sidebar {
                    <button type="button" class="flex items-center py-[0.75rem] px-6 pb-[0.6875rem] leading-[2] text-[0.75rem] font-medium text-text-2 transition-colors duration-500 hover:text-text-1 hover:duration-[250ms] cursor-pointer lg:hidden md:px-8" id="VPLocalNavMenu" aria-expanded="false" aria-controls="VPSidebarNav">
                        (icon("align-left", "mr-2 size-[0.875rem]"))
                        <span>"Menu"</span>
                    </button>
                }
                <div id="VPLocalNavOutlineDropdown">
                    <button type="button" class="group/drop relative block py-[0.75rem] px-6 pb-[0.6875rem] leading-[2] text-[0.75rem] font-medium text-text-2 transition-colors duration-500 hover:text-text-1 hover:duration-[250ms] cursor-pointer [&.open]:text-text-1 md:px-8 lg:text-[0.875rem]" id="VPOutlineDropdownButton" aria-expanded="false" aria-controls="VPOutlineDropdownItems">
                        <span>(outline_label.clone())</span>
                        (icon("chevron-right", "inline-block align-middle ml-[0.125rem] size-[0.875rem] transition-transform duration-[250ms] group-[.open]/drop:rotate-90 lg:size-[1rem]"))
                    </button>
                    <div class="absolute top-10 right-4 left-4 grid gap-px border border-border rounded-lg bg-gutter max-h-[calc(var(--vp-vh,100vh)-5.375rem)] overflow-x-hidden overflow-y-auto overscroll-contain shadow-3 lg:right-auto lg:left-[calc(var(--vp-sidebar-width)+2rem)] lg:w-80" id="VPOutlineDropdownItems" hidden>
                        <div class="bg-bg-soft">
                            <a class="block px-4 leading-[3.4285714] text-[0.875rem] font-medium text-brand-1" href=(root_url)>"Return to top"</a>
                        </div>
                        <div class="py-2 bg-bg-soft">
                            (outline_list(headings, false))
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
    .render()
    .into_inner()
}

/// Flat outline link list (the dropdown shows one flat level, like the
/// original outline_items at root nesting).
fn outline_list<'a>(headings: &'a [Heading], nested: bool) -> impl Renderable + 'a {
    let cls = if nested { "pr-4 pl-4" } else { "relative z-[1]" };
    rsx! {
        <ul class=(cls)>
            @for h in headings {
                <li>
                    <a class=(format!(
                        "outline-link block leading-[2.2857143] text-[0.875rem] text-text-2 whitespace-nowrap overflow-hidden text-ellipsis transition-colors duration-500 hover:text-text-1 hover:duration-[250ms] [&.active]:text-text-1 scroll-mt-[calc(var(--vp-nav-height)+var(--vp-layout-top-height,0px)+var(--vp-doc-top-height,0px)+2rem)] scroll-mb-12{}",
                        if !nested { " pl-[0.8125rem]" } else { "" }
                    )) href=(format!("#{}", h.id)) title=(h.text.clone())>(h.text.clone())</a>
                </li>
            }
        </ul>
    }
}
