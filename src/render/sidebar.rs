//! VPSidebar (ported from partials/sidebar.html + the sidebar_items
//! Tera component): recursive groups with tri-state `collapsed`
//! semantics matching VitePress — `Some(true)` starts collapsed,
//! `Some(false)` starts expanded with a caret, `None` = static group.

use hypertext::prelude::*;
use hypertext::Raw;

use super::icons::icon;
use super::Site;
use crate::sidebar::SidebarNode;

const GROUP_CLS: &str =
    "[.group+&]:border-t [.group+&]:border-divider [.group+&]:pt-2.5 lg:w-[calc(var(--vp-sidebar-width)-4rem)]";
const ITEM_CLS: &str = "group/item item relative flex w-full";
const INDICATOR_CLS: &str =
    "indicator absolute top-[0.375rem] bottom-[0.375rem] left-[calc(-1rem-1px)] w-[2px] rounded-[2px] transition-colors duration-[250ms]";
const CARET_BTN_CLS: &str = "caret flex justify-center items-center -mr-[0.4375rem] w-8 h-8 shrink-0 text-text-3 cursor-pointer transition-colors duration-[250ms] group-hover/item:text-text-2 hover:text-text-1!";
const CARET_ICON_CLS: &str =
    "text-[1.125rem] rotate-90 transition-transform duration-[250ms] group-[.collapsed]/side:rotate-0";

/// Optional link attributes, omitted (not empty) when unset.
fn opt_attrs(target: Option<&str>, rel: Option<&str>) -> String {
    let esc = |s: &str| s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;");
    let mut out = String::new();
    if let Some(t) = target {
        out.push_str(&format!(r#" target="{}""#, esc(t)));
    }
    if let Some(r) = rel {
        out.push_str(&format!(r#" rel="{}""#, esc(r)));
    }
    out
}

pub fn sidebar(site: &Site, current_url: &str) -> String {
    let Some(tree) = site.sidebars.for_url(current_url) else {
        return String::new();
    };
    let items: Vec<&SidebarNode> = tree.items.iter().collect();
    rsx! {
        <aside class="VPSidebar fixed top-[var(--vp-layout-top-height,0px)] bottom-0 left-0 z-(--vp-z-index-sidebar) px-8 pt-8 pb-24 w-[calc(100vw-4rem)] max-w-80 bg-(--vp-sidebar-bg-color) opacity-0 shadow-3 max-lg:dark:shadow-(--vp-shadow-1) overflow-x-hidden overflow-y-auto overscroll-contain -translate-x-full [transition:opacity_0.5s,transform_0.25s_ease] max-lg:[&.open]:opacity-100 max-lg:[&.open]:visible max-lg:[&.open]:translate-x-0 max-lg:[&.open]:[transition:opacity_0.25s,transform_0.5s_cubic-bezier(0.19,1,0.22,1)] lg:pt-(--vp-nav-height) lg:w-(--vp-sidebar-width) lg:max-w-full lg:opacity-100 lg:visible lg:shadow-none lg:translate-x-0 2xl:pl-[max(2rem,calc((100%-(var(--vp-layout-max-width)-4rem))/2))] 2xl:w-[calc((100%-(var(--vp-layout-max-width)-4rem))/2+var(--vp-sidebar-width)-2rem)]" id="VPSidebar" :class=("{ open: $store.ui.sidebar }") x-effect="document.body.style.overflow = $store.ui.sidebar ? 'hidden' : ''">
            <div class="lg:sticky lg:top-[calc(var(--vp-nav-height)*-1)] lg:left-0 lg:z-[1] lg:-mt-(--vp-nav-height) lg:-mr-8 lg:-ml-8 lg:h-(--vp-nav-height) lg:bg-(--vp-sidebar-bg-color)"></div>
            <nav class="outline-0" id="VPSidebarNav" aria-labelledby="sidebar-aria-label" tabindex="-1">
                <span class="sr-only" id="sidebar-aria-label">"Sidebar Navigation"</span>
                @for item in &items {
                    // one `.group` wrapper per root item: the `.group + .group`
                    // divider rule needs adjacent wrapper siblings, which the
                    // old single-wrapper layout could never produce. Root
                    // wrappers carry the lg top gap upstream gives every root
                    // group; the nested wrapper inside `ul.items` must not, or
                    // each group's first link sits 10px low
                    <div class=(format!("group lg:pt-2.5 {GROUP_CLS}"))>
                        @if item.children.is_empty() && item.url.is_some() {
                            // Root-level bare links render one level deep,
                            // wrapped in a headless item — like VitePress's
                            // VPSidebarItem for link-only root entries.
                            <div class="VPSidebarItem level-0">
                                <div class=(ITEM_CLS)><div class=(INDICATOR_CLS)></div></div>
                                <ul class="items">
                                    <li>(Raw::dangerously_create(node(site, item, current_url, 1)))</li>
                                </ul>
                            </div>
                        } @else {
                            (Raw::dangerously_create(node(site, item, current_url, 0)))
                        }
                    </div>
                }
            </nav>
        </aside>
    }
    .render()
    .into_inner()
}

/// Render one sidebar node at `depth` (0 = the top group level).
fn node<'a>(site: &'a Site, n: &'a SidebarNode, current_url: &'a str, depth: usize) -> String {
    let is_active = n.url.as_deref() == Some(current_url);
    let has_active = n.url.as_deref().is_some_and(|u| current_url.starts_with(u) && u != "/");
    let collapsible = n.collapsed.is_some();
    let starts_collapsed = n.collapsed == Some(true);
    let text_cls = if depth == 0 {
        "text grow py-1 leading-[1.7142857] text-[0.875rem] font-bold text-text-1".to_string()
    } else {
        format!(
            "text grow py-1 leading-[1.7142857] text-[0.875rem] transition-colors duration-[250ms] font-medium{}",
            // the active color arrives from the leaf link below; emitting
            // `text-text-1` here too would win on stylesheet order and
            // wash the brand color out
            if is_active { "" } else if has_active { " text-text-1" } else { " text-text-2" }
        )
    };
    let section_cls = format!(
        "group/side VPSidebarItem level-{depth}{}{}{}{}",
        if depth == 0 { " pb-6 [&.collapsed]:pb-2.5" } else { "" },
        if collapsible { " collapsible" } else { "" },
        if is_active { " is-active" } else { "" },
        if has_active && !is_active { " has-active" } else { "" },
    );
    // Collapsible groups carry Alpine state; `collapsed` moves from the
    // static class list to a binding so the caret can toggle it.
    let (section_x_data, section_bind) = if collapsible {
        (
            r#"{ "open": false }"#.replace("false", if starts_collapsed { "false" } else { "true" }),
            r#"{ 'collapsed': !open }"#.to_string(),
        )
    } else {
        (String::new(), String::new())
    };
    let indicator_cls = format!(
        "{INDICATOR_CLS}{}",
        if depth >= 2 && is_active { " bg-brand-1" } else { "" }
    );
    let row_cls = format!(
        "{ITEM_CLS}{}",
        if collapsible { " cursor-pointer" } else { "" }
    );
    let text = n.text.clone();
    let children: Vec<&SidebarNode> = n.children.iter().collect();
    let href = n.url.clone().map(|u| site.url(&u));
    let has_children = !children.is_empty();

    // an empty `:class=""` is a JavaScript syntax error once Alpine
    // evaluates it, so plain sections get no bindings at all
    let section_open = if collapsible {
        format!(
            r#"<section class="{section_cls}" x-data="{}" :class="{}">"#,
            section_x_data.replace('"', "&quot;"),
            section_bind.replace('"', "&quot;")
        )
    } else {
        format!(r#"<section class="{section_cls}">"#)
    };

    // the link supplies the color itself; stripping only the trailing
    // color utility keeps a single color class on the <p> (two would
    // resolve by stylesheet order) while depth-0 headers keep `font-bold`
    let link_text_cls = text_cls
        .trim_end_matches(" text-text-1")
        .trim_end_matches(" text-text-2")
        .to_string();

    rsx! {
        (Raw::dangerously_create(section_open.clone()))
            <div class=(row_cls)>
                <div class=(indicator_cls)></div>
                @if let Some(href) = href.clone() {
                    (Raw::dangerously_create(format!(
                        r#"<a class="link flex items-center grow group/link" href="{href}"{}{}><p class="{}{}">{}</p></a>"#,
                        if is_active { r#" aria-current="page""# } else { "" },
                        opt_attrs(n.target.as_deref(), n.rel.as_deref()),
                        link_text_cls,
                        if is_active { " text-brand-1" } else if depth == 0 { " text-text-1" } else { " text-text-2" },
                        text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;"),
                    )))
                } @else {
                    <h3 class=(text_cls)>(text.clone())</h3>
                }
                @if collapsible {
                    <button type="button" class={CARET_BTN_CLS} aria-label="toggle section" @click="open = !open" :aria-expanded=("open.toString()")>
                        (icon("chevron-right", CARET_ICON_CLS))
                    </button>
                }
            </div>
            @if has_children {
                <ul class=(format!(
                    "items{} group-[.collapsed]/side:hidden",
                    if depth >= 1 { " border-l border-divider pl-4" } else { "" }
                ))>
                    <li>
                        <div class=(format!("group {GROUP_CLS}"))>
                            @for child in &children {
                                (Raw::dangerously_create(node(site, child, current_url, depth + 1)))
                            }
                        </div>
                    </li>
                </ul>
            }
        (Raw::dangerously_create("</section>"))
    }
    .render()
    .into_inner()
}
