//! The `.vp-doc` markdown-body class string: a faithful port of the
//! theme's `vpdoc_classes` Tera component — the rendered-markdown markup
//! (headings, lists, tables, containers, badges, code blocks) has no
//! per-element template to attach classes to, so everything is expressed
//! as `[&_...]` arbitrary variants on the wrapper. Tailwind's scanner
//! picks the string up from this source file.
//!
//! Adapted from the Zola build: `pre.giallo` → `pre` (all code blocks
//! flow through the gdcode renderer now), `.giallo-l` line wrappers →
//! `.line`, line-number columns dropped, `.line.hl` highlighting added.
//! Every line but the first starts with exactly one space — gluing two
//! utilities together silently disables both (the rule the old Python
//! checker enforced; `vpdoc_classes_smoke` guards it here).

/// Heading/paragraph/list/table/… utilities for rendered markdown.
pub const VPDOC_CLASSES: &str = concat!(
    // headings: relative, weights, overflow breaking
    "[&_:is(h1,h2,h3,h4,h5,h6)]:relative [&_:is(h1,h2,h3,h4,h5,h6)]:font-semibold",
    " [&_:is(h1,h2,h3,h4,h5,h6)]:outline-none [&_:is(h1,h2,h3,h4,h5,h6)]:break-words",
    // paragraph flow: overflow breaking
    " [&_p]:break-words",
    // lists: overflow breaking
    " [&_li]:break-words",
    // headings: sizes, leads, tracking, scroll margins
    " [&_h1]:tracking-[-0.02em] [&_h1]:leading-[1.4285714] [&_h1]:text-[1.75rem] md:[&_h1]:leading-[1.25]",
    " md:[&_h1]:text-[2rem] [&_h2]:mt-12 [&_h2]:mb-4 [&_h2]:border-t [&_h2]:border-divider [&_h2]:pt-6",
    " [&_h2]:tracking-[-0.02em] [&_h2]:leading-[1.3333333] [&_h2]:text-[1.5rem]",
    " [&_h2]:[--vp-extra-scroll-margin:0px] [&_h3]:mt-8 [&_h3]:tracking-[-0.01em] [&_h3]:leading-[1.4]",
    " [&_h3]:text-[1.25rem] [&_h4]:mt-6 [&_h4]:tracking-[-0.01em] [&_h4]:leading-[1.3333333]",
    " [&_h4]:text-[1.125rem]",
    " [&_[id]]:scroll-mt-[calc(2.9375rem+var(--vp-layout-top-height,0px)+var(--vp-extra-scroll-margin,1.5rem))]",
    " lg:[&_[id]]:scroll-mt-[calc(var(--vp-nav-height)+2.9375rem+var(--vp-layout-top-height,0px)+var(--vp-extra-scroll-margin,1.5rem))]",
    " xl:[&_[id]]:scroll-mt-[calc(var(--vp-nav-height)+var(--vp-layout-top-height,0px)+var(--vp-extra-scroll-margin,1.5rem))]",
    // paragraph flow: blocks, links, inline code
    " [&_p]:my-4 [&_p]:leading-[1.75] [&_img]:my-4 [&_summary]:my-4 [&_blockquote]:my-4",
    " [&_blockquote]:border-l-2 [&_blockquote]:border-divider [&_blockquote]:pl-4",
    " [&_blockquote]:text-text-2 [&_blockquote]:transition-colors [&_blockquote]:duration-500",
    // paragraph flow: blockquote children
    " [&_blockquote>p]:m-0 [&_blockquote>p]:text-[1rem] [&_blockquote>p]:transition-colors",
    " [&_blockquote>p]:duration-500",
    // paragraph flow: blocks, links, inline code
    " [&_a]:font-medium [&_a]:text-brand-1 [&_a]:underline [&_a]:underline-offset-[0.125rem]",
    " [&_a]:transition-colors [&_a]:duration-[250ms]",
    // paragraph flow: links
    " [&_a:hover]:text-brand-2",
    // paragraph flow: blocks, links, inline code
    " [&_:not(pre,:is(h1,h2,h3,h4,h5,h6))>code]:text-[0.875em]",
    " [&_:not(pre,:is(h1,h2,h3,h4,h5,h6))>code]:text-(--vp-code-color)",
    " [&_:not(pre)>code]:rounded-[0.25rem] [&_:not(pre)>code]:py-[0.1875rem]",
    " [&_:not(pre)>code]:px-[0.375rem] [&_:not(pre)>code]:bg-(--vp-code-bg)",
    " [&_:not(pre)>code]:[transition:color_0.25s,background-color_0.5s]",
    // headings: code inside headings
    " [&_:is(h1,h2,h3,h4)>code]:text-[0.9em]",
    // paragraph flow: blocks, links, inline code
    " [&_a>code]:text-(--vp-code-link-color)",
    // paragraph flow: inline code links
    " [&_a:hover>code]:text-(--vp-code-link-hover-color)",
    // lists, task lists, footnotes
    " [&_ul]:my-4 [&_ul]:pl-5 [&_ul]:list-disc [&_ol]:my-4 [&_ol]:pl-5 [&_ol]:list-decimal [&_li+li]:mt-2",
    " [&_li>ol]:my-2 [&_li>ul]:my-2 [&_li>p:first-child]:mt-0 [&_li>p:last-child]:mb-0",
    " [&_li.task-list-item]:list-none [&_.task-list-item-checkbox]:mr-[0.25rem]",
    " [&_.task-list-item-checkbox]:mb-[0.125rem] [&_.task-list-item-checkbox]:ml-[-1.25rem]",
    " [&_.task-list-item-checkbox]:align-middle [&_.task-list-item-checkbox]:accent-(--vp-c-brand-1)",
    " [&_.footnote-ref_a]:no-underline [&_.footnote-backref]:no-underline [&_.footnotes]:text-[0.875rem]",
    // tables
    " [&_table]:block [&_table]:border-collapse [&_table]:my-5 [&_table]:overflow-x-auto [&_tr]:border-t",
    " [&_tr]:border-divider [&_tr]:bg-bg [&_tr]:transition-colors [&_tr]:duration-500",
    // tables: zebra rows
    " [&_tr:nth-child(2n)]:bg-bg-soft",
    // tables
    " [&_th]:border [&_th]:border-divider [&_th]:px-4 [&_th]:py-2 [&_th]:text-left [&_th]:text-[0.875rem]",
    " [&_th]:font-semibold [&_th]:text-text-2 [&_th]:bg-bg-soft [&_td]:border [&_td]:border-divider",
    " [&_td]:px-4 [&_td]:py-2 [&_td]:text-[0.875rem]",
    // horizontal rules
    " [&_hr]:my-4 [&_hr]:border-0 [&_hr]:border-t [&_hr]:border-divider",
    // code blocks: pre, lines, lang label, copy button
    " [&_pre]:relative [&_pre]:z-[1] [&_pre]:my-4 [&_pre]:-mx-6",
    " [&_pre]:py-5 [&_pre]:bg-(--vp-code-block-bg) [&_pre]:overflow-x-auto",
    " [&_pre]:text-left [&_pre]:transition-colors [&_pre]:duration-500",
    " sm:[&_pre]:mx-0 sm:[&_pre]:rounded-lg [&_pre_code]:block",
    " [&_pre_code]:px-6 [&_pre_code]:w-fit [&_pre_code]:min-w-full",
    " [&_pre_code]:[tab-size:4] [&_pre_code]:leading-(--vp-code-line-height)",
    " [&_pre_code]:text-[0.875em] [&_pre_code]:text-(--vp-code-block-color)",
    " [&_pre_code]:transition-colors [&_pre_code]:duration-500 [&_.line]:inline-block",
    " [&_.line]:min-h-[1lh] [&_.line]:w-full [&_.line.hl]:bg-(--vp-code-line-highlight-color)",
    " [&_pre>.lang]:absolute [&_pre>.lang]:top-[0.125rem] [&_pre>.lang]:right-2",
    " [&_pre>.lang]:z-[2] [&_pre>.lang]:text-[0.75rem] [&_pre>.lang]:font-medium",
    " [&_pre>.lang]:select-none [&_pre>.lang]:text-(--vp-code-lang-color)",
    " [&_pre>.lang]:[transition:color_0.4s,opacity_0.4s] [&_pre:hover>.lang]:opacity-0",
    " [&_pre>button.vp-copy-button:focus+.lang]:opacity-0 [&_pre_.vp-copy-button]:absolute",
    " [&_pre_.vp-copy-button]:top-2 [&_pre_.vp-copy-button]:right-2",
    " [&_pre_.vp-copy-button]:z-[3] [&_pre_.vp-copy-button]:block",
    " [&_pre_.vp-copy-button]:w-10 [&_pre_.vp-copy-button]:h-8",
    " [&_pre_.vp-copy-button]:border",
    " [&_pre_.vp-copy-button]:border-(--vp-code-copy-code-border-color)",
    " [&_pre_.vp-copy-button]:rounded-[0.25rem]",
    " [&_pre_.vp-copy-button]:bg-(--vp-code-copy-code-bg)",
    " [&_pre_.vp-copy-button]:text-text-2 [&_pre_.vp-copy-button]:opacity-0",
    " [&_pre_.vp-copy-button]:cursor-pointer [&_pre_.vp-copy-button]:transition-opacity",
    " [&_pre_.vp-copy-button]:duration-[250ms] [&_pre:hover_.vp-copy-button]:opacity-100",
    " [&_pre_.vp-copy-button:focus]:opacity-100",
    " [&_pre_.vp-copy-button:hover]:border-(--vp-code-copy-code-hover-border-color)",
    " [&_pre_.vp-copy-button:hover]:bg-(--vp-code-copy-code-hover-bg)",
    " [&_pre_.vp-copy-button.copied]:border-(--vp-code-copy-code-hover-border-color)",
    " [&_pre_.vp-copy-button.copied]:bg-(--vp-code-copy-code-hover-bg)",
    " [&_pre_.vp-copy-button.copied_.icon-copy]:hidden",
    " [&_pre_.vp-copy-button:not(.copied)_.icon-copied]:hidden",
    // external-link arrow icon (after pseudo-element)
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:content-['\\2060']",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:inline",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:ml-1",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:pl-[0.6875rem]",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:bg-current",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:text-text-3",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:align-middle",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:text-[0.5625rem]",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:[mask-image:url('data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 24 24%27%3E%3Cpath d=%27M0 0h24v24H0V0z%27 fill=%27none%27/%3E%3Cpath d=%27M9 5v2h6.59L4 18.59 5.41 20 17 8.41V15h2V5H9z%27/%3E%3C/svg%3E')]",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:[mask-position:center]",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:[mask-repeat:no-repeat]",
    " [&_:is(a[href*='://'],a[target='_blank']):not(.no-icon,svg a,:has(img,svg))]:after:[mask-size:100%_100%]",
);

/// Container/badge/code-group utilities — the rest of the original
/// component (c19–c24), unchanged from the Zola build.
pub const VPDOC_CLASSES_2: &str = concat!(
    // custom containers (info/tip/warning/…) and their children
    "[&_.custom-block]:my-4 [&_.custom-block]:rounded-lg [&_.custom-block]:border",
    " [&_.custom-block]:border-transparent [&_.custom-block]:px-4 [&_.custom-block]:py-2",
    " [&_.custom-block]:leading-[1.7142857] [&_.custom-block]:text-[0.875rem]",
    " [&_.custom-block]:text-text-2 [&_.custom-block:has(>.custom-block-title,>summary)]:pt-4",
    " [&_.custom-block-title]:font-semibold [&_.custom-block_p]:my-2",
    " [&_.custom-block_p]:leading-[1.7142857] [&_.custom-block_p:first-child]:m-0",
    " [&_.custom-block.details_summary]:mt-0 [&_.custom-block.details_summary]:mb-2",
    " [&_.custom-block.details_summary]:font-bold [&_.custom-block.details_summary]:cursor-pointer",
    " [&_.custom-block.details_summary]:select-none [&_.custom-block.details_summary+p]:my-2",
    " [&_.custom-block_a]:[transition:color_0.25s,opacity_0.25s] [&_.custom-block_a:hover]:opacity-75",
    " [&_.custom-block_code]:text-(--vp-custom-block-code-font-size) [&_.custom-block_th]:text-[0.875rem]",
    " [&_.custom-block_th]:text-inherit [&_.custom-block_blockquote>p]:text-[0.875rem]",
    " [&_.custom-block_blockquote>p]:text-inherit [&_.custom-block_.vp-code-group]:mt-2",
    " [&_.custom-block_.vp-code-group_.tabs]:m-0 [&_.custom-block_.vp-code-group_.tabs]:rounded-t-lg",
    " [&_.custom-block.info]:bg-(--vp-custom-block-info-bg)",
    " [&_.custom-block.info]:text-(--vp-custom-block-info-text)",
    " [&_.custom-block.info_a]:text-(--vp-c-brand-1) [&_.custom-block.info_a:hover]:text-(--vp-c-brand-2)",
    " [&_.custom-block.info_a:hover>code]:text-(--vp-c-brand-2)",
    " [&_.custom-block.info_code]:text-(--vp-c-brand-1)",
    " [&_.custom-block.info_code]:bg-(--vp-custom-block-info-code-bg)",
    " [&_.custom-block.note]:bg-(--vp-custom-block-note-bg)",
    " [&_.custom-block.note]:text-(--vp-custom-block-note-text)",
    " [&_.custom-block.note_a]:text-(--vp-c-brand-1) [&_.custom-block.note_a:hover]:text-(--vp-c-brand-2)",
    " [&_.custom-block.note_a:hover>code]:text-(--vp-c-brand-2)",
    " [&_.custom-block.note_code]:text-(--vp-c-brand-1)",
    " [&_.custom-block.note_code]:bg-(--vp-custom-block-note-code-bg)",
    " [&_.custom-block.tip]:bg-(--vp-custom-block-tip-bg)",
    " [&_.custom-block.tip]:text-(--vp-custom-block-tip-text) [&_.custom-block.tip_a]:text-(--vp-c-tip-1)",
    " [&_.custom-block.tip_a:hover]:text-(--vp-c-tip-2)",
    " [&_.custom-block.tip_a:hover>code]:text-(--vp-c-tip-2)",
    " [&_.custom-block.tip_code]:text-(--vp-c-tip-1)",
    " [&_.custom-block.tip_code]:bg-(--vp-custom-block-tip-code-bg)",
    " [&_.custom-block.important]:bg-(--vp-custom-block-important-bg)",
    " [&_.custom-block.important]:text-(--vp-custom-block-important-text)",
    " [&_.custom-block.important_a]:text-(--vp-c-important-1)",
    " [&_.custom-block.important_a:hover]:text-(--vp-c-important-2)",
    " [&_.custom-block.important_a:hover>code]:text-(--vp-c-important-2)",
    " [&_.custom-block.important_code]:text-(--vp-c-important-1)",
    " [&_.custom-block.important_code]:bg-(--vp-custom-block-important-code-bg)",
    " [&_.custom-block.caution]:bg-(--vp-custom-block-caution-bg)",
    " [&_.custom-block.caution]:text-(--vp-custom-block-caution-text)",
    " [&_.custom-block.caution_a]:text-(--vp-c-caution-1)",
    " [&_.custom-block.caution_a:hover]:text-(--vp-c-caution-2)",
    " [&_.custom-block.caution_a:hover>code]:text-(--vp-c-caution-2)",
    " [&_.custom-block.caution_code]:text-(--vp-c-caution-1)",
    " [&_.custom-block.caution_code]:bg-(--vp-custom-block-caution-code-bg)",
    " [&_.custom-block.warning]:bg-(--vp-custom-block-warning-bg)",
    " [&_.custom-block.warning]:text-(--vp-custom-block-warning-text)",
    " [&_.custom-block.warning_a]:text-(--vp-c-warning-1)",
    " [&_.custom-block.warning_a:hover]:text-(--vp-c-warning-2)",
    " [&_.custom-block.warning_a:hover>code]:text-(--vp-c-warning-2)",
    " [&_.custom-block.warning_code]:text-(--vp-c-warning-1)",
    " [&_.custom-block.warning_code]:bg-(--vp-custom-block-warning-code-bg)",
    " [&_.custom-block.danger]:bg-(--vp-custom-block-danger-bg)",
    " [&_.custom-block.danger]:text-(--vp-custom-block-danger-text)",
    " [&_.custom-block.danger_a]:text-(--vp-c-danger-1)",
    " [&_.custom-block.danger_a:hover]:text-(--vp-c-danger-2)",
    " [&_.custom-block.danger_a:hover>code]:text-(--vp-c-danger-2)",
    " [&_.custom-block.danger_code]:text-(--vp-c-danger-1)",
    " [&_.custom-block.danger_code]:bg-(--vp-custom-block-danger-code-bg)",
    " [&_.custom-block.details]:bg-(--vp-custom-block-details-bg)",
    " [&_.custom-block.details]:text-(--vp-custom-block-details-text)",
    " [&_.custom-block.details_a]:text-(--vp-c-brand-1)",
    " [&_.custom-block.details_a:hover]:text-(--vp-c-brand-2)",
    " [&_.custom-block.details_code]:bg-(--vp-custom-block-details-code-bg)",
    // VPBadge
    " [&_.VPBadge]:inline-block [&_.VPBadge]:ml-[0.125rem] [&_.VPBadge]:border",
    " [&_.VPBadge]:border-transparent [&_.VPBadge]:rounded-xl [&_.VPBadge]:px-[0.625rem]",
    " [&_.VPBadge]:leading-[1.8333333] [&_.VPBadge]:text-[0.75rem] [&_.VPBadge]:font-medium",
    " [&_.VPBadge]:whitespace-nowrap [&_.VPBadge]:translate-y-[-0.125rem]",
    " [&_.VPBadge.small]:px-[0.375rem] [&_.VPBadge.small]:leading-[1.8]",
    " [&_.VPBadge.small]:text-[0.625rem] [&_.VPBadge.small]:translate-y-[-0.5rem]",
    // VPBadge inside headings
    " [&_:is(h1,h2)>.VPBadge]:my-0 [&_:is(h1,h2)>.VPBadge]:ml-[0.125rem]",
    " [&_:is(h1,h2)>.VPBadge]:align-middle",
    " [&_h2>.VPBadge]:px-2 [&_:is(h3,h4,h5,h6)>.VPBadge]:align-middle",
    " [&_:is(h4,h5,h6)>.VPBadge]:leading-[1.5]",
    // VPBadge colors
    " [&_.VPBadge.info]:border-(--vp-badge-info-border) [&_.VPBadge.info]:text-(--vp-badge-info-text)",
    " [&_.VPBadge.info]:bg-(--vp-badge-info-bg) [&_.VPBadge.note]:border-(--vp-badge-note-border)",
    " [&_.VPBadge.note]:text-(--vp-badge-note-text) [&_.VPBadge.note]:bg-(--vp-badge-note-bg)",
    " [&_.VPBadge.tip]:border-(--vp-badge-tip-border) [&_.VPBadge.tip]:text-(--vp-badge-tip-text)",
    " [&_.VPBadge.tip]:bg-(--vp-badge-tip-bg) [&_.VPBadge.important]:border-(--vp-badge-important-border)",
    " [&_.VPBadge.important]:text-(--vp-badge-important-text)",
    " [&_.VPBadge.important]:bg-(--vp-badge-important-bg)",
    " [&_.VPBadge.caution]:border-(--vp-badge-caution-border)",
    " [&_.VPBadge.caution]:text-(--vp-badge-caution-text) [&_.VPBadge.caution]:bg-(--vp-badge-caution-bg)",
    " [&_.VPBadge.warning]:border-(--vp-badge-warning-border)",
    " [&_.VPBadge.warning]:text-(--vp-badge-warning-text) [&_.VPBadge.warning]:bg-(--vp-badge-warning-bg)",
    " [&_.VPBadge.danger]:border-(--vp-badge-danger-border)",
    " [&_.VPBadge.danger]:text-(--vp-badge-danger-text) [&_.VPBadge.danger]:bg-(--vp-badge-danger-bg)",
    // code groups
    " [&_.vp-code-group]:mt-4 [&_.vp-code-group_.tabs]:relative [&_.vp-code-group_.tabs]:flex",
    " [&_.vp-code-group_.tabs]:-mr-6 [&_.vp-code-group_.tabs]:-ml-6 [&_.vp-code-group_.tabs]:px-3",
    " [&_.vp-code-group_.tabs]:bg-(--vp-code-tab-bg) [&_.vp-code-group_.tabs]:overflow-x-auto",
    " [&_.vp-code-group_.tabs]:overflow-y-hidden",
    " [&_.vp-code-group_.tabs]:shadow-[inset_0_-1px_var(--vp-code-tab-divider)]",
    " sm:[&_.vp-code-group_.tabs]:mx-0 sm:[&_.vp-code-group_.tabs]:rounded-t-lg",
    " [&_.vp-code-group_.tabs_input]:fixed [&_.vp-code-group_.tabs_input]:opacity-0",
    " [&_.vp-code-group_.tabs_input]:pointer-events-none [&_.vp-code-group_.tabs_label]:relative",
    " [&_.vp-code-group_.tabs_label]:inline-block [&_.vp-code-group_.tabs_label]:border-b",
    " [&_.vp-code-group_.tabs_label]:border-b-transparent [&_.vp-code-group_.tabs_label]:px-3",
    " [&_.vp-code-group_.tabs_label]:leading-[3.4285714] [&_.vp-code-group_.tabs_label]:text-[0.875rem]",
    " [&_.vp-code-group_.tabs_label]:font-medium",
    " [&_.vp-code-group_.tabs_label]:text-(--vp-code-tab-text-color)",
    " [&_.vp-code-group_.tabs_label]:whitespace-nowrap [&_.vp-code-group_.tabs_label]:cursor-pointer",
    " [&_.vp-code-group_.tabs_label]:transition-colors [&_.vp-code-group_.tabs_label]:duration-[250ms]",
    " [&_.vp-code-group_.tabs_label]:after:content-[''] [&_.vp-code-group_.tabs_label]:after:absolute",
    " [&_.vp-code-group_.tabs_label]:after:right-2 [&_.vp-code-group_.tabs_label]:after:-bottom-px",
    " [&_.vp-code-group_.tabs_label]:after:left-2 [&_.vp-code-group_.tabs_label]:after:z-[1]",
    " [&_.vp-code-group_.tabs_label]:after:h-[2px] [&_.vp-code-group_.tabs_label]:after:rounded-[2px]",
    " [&_.vp-code-group_.tabs_label]:after:bg-transparent",
    " [&_.vp-code-group_.tabs_label]:after:transition-colors",
    " [&_.vp-code-group_.tabs_label]:after:duration-[250ms]",
    " [&_.vp-code-group_.tabs_label:hover]:text-(--vp-code-tab-hover-text-color)",
    " [&_.vp-code-group_.tabs_input:checked+label]:text-(--vp-code-tab-active-text-color)",
    " [&_.vp-code-group_.tabs_input:checked+label]:after:bg-(--vp-code-tab-active-bar-color)",
    " [&_.vp-code-group_.blocks>pre]:hidden [&_.vp-code-group_.blocks>pre]:mt-0!",
    " [&_.vp-code-group_.blocks>pre]:rounded-tl-none! [&_.vp-code-group_.blocks>pre]:rounded-tr-none!",
    " [&_.vp-code-group_.blocks>pre.active]:block",
);

/// The full wrapper class list.
pub fn vpdoc_class() -> String {
    format!("vp-doc {VPDOC_CLASSES} {VPDOC_CLASSES_2}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The old scripts/check_vpdoc_classes.py rules, minus the Tera
    /// escaping concerns: no double spaces, no space before a closing
    /// bracket gluing, every utility parseable (starts with `[`, `.`, a
    /// letter, or a `(`-prefixed variant), and the two halves join with
    /// exactly one space.
    #[test]
    fn vpdoc_classes_smoke() {
        for blob in [VPDOC_CLASSES, VPDOC_CLASSES_2] {
            assert!(!blob.starts_with(' '), "no leading space");
            assert!(!blob.ends_with(' '), "no trailing space");
            assert!(!blob.contains("  "), "no double spaces");
            assert!(blob.matches("[&_").count() > 100, "expected a full utility set");
            // brackets and parens balance across the whole blob (utilities
            // may contain spaces inside :is()/:not() selectors, so they
            // cannot be split on whitespace for this check)
            for (open, close) in [('[', ']'), ('(', ')')] {
                assert_eq!(
                    blob.matches(open).count(),
                    blob.matches(close).count(),
                    "unbalanced {open}{close}"
                );
            }
        }
    }
}
