//! VPDoc (ported from partials/doc_body.html): right-hand outline aside,
//! the `.vp-doc` markdown content, doc footer (edit link, last updated,
//! prev/next pager).

use hypertext::{Raw, prelude::*};

use super::Site;
use super::icons::icon;
use crate::content::Page;
use crate::markdown::{Heading, RenderedPage};

// the args mirror the upstream template's slots one to one; a params
// struct would just move the same names behind a field access
#[allow(clippy::too_many_arguments)]
pub fn doc_page<'a>(
    site: &'a Site,
    page: &'a Page,
    rendered: &'a RenderedPage,
    prev: Option<&'a super::PagerLink>,
    next: Option<&'a super::PagerLink>,
    has_sidebar: bool,
    outline: Option<(u8, u8)>,
    aside: Option<bool>,
    edit_on: bool,
    last_updated: Option<&'a (String, String)>,
) -> impl Renderable + 'a {
    let outline_label = site.config.outline.label();
    let headings: Vec<&Heading> = match outline {
        Some((lo, hi)) => rendered
            .headings
            .iter()
            .filter(|h| h.level >= lo && h.level <= hi)
            .collect(),
        None => Vec::new(),
    };
    let show_outline = !headings.is_empty();
    let aside_left = aside == Some(true);
    let edit_link = site.config.edit_link.as_ref().filter(|_| edit_on).map(|e| {
        let text = e
            .text
            .clone()
            .unwrap_or_else(|| "Edit this page on GitHub".into());
        (
            e.pattern.replace(":path", &percent_encode_path(&page.rel)),
            text,
        )
    });
    let last_updated = last_updated.cloned();
    let doc_footer = site.config.doc_footer.clone().unwrap_or_default();
    // `docFooter.prev/next: false` disables that pager side
    let (prev, prev_label) = match pager_label(&doc_footer.prev, "Previous page") {
        Ok(label) => (prev, label),
        Err(()) => (None, String::new()),
    };
    let (next, next_label) = match pager_label(&doc_footer.next, "Next page") {
        Ok(label) => (next, label),
        Err(()) => (None, String::new()),
    };
    let updated_label = site.config.last_updated_text.clone();
    let show_footer =
        edit_link.is_some() || last_updated.is_some() || prev.is_some() || next.is_some();

    let wrapper_cls = if has_sidebar {
        "mx-auto w-full xl:flex xl:justify-center"
    } else {
        "mx-auto w-full lg:flex lg:justify-center lg:max-w-[62rem] 2xl:max-w-[69rem]"
    };
    let aside_cls = format!(
        "relative hidden grow pl-8 w-full max-w-64 xl:block{}",
        if aside_left {
            " xl:order-1"
        } else {
            " order-2"
        }
    );
    let content_cls = format!(
        "relative mx-auto w-full lg:px-8 lg:pb-32 xl:m-0 xl:min-w-[40rem]{}{}",
        if aside_left {
            " xl:order-2"
        } else {
            " xl:order-1"
        },
        if !has_sidebar {
            " lg:max-w-[47rem] 2xl:max-w-[49rem]"
        } else {
            ""
        }
    );

    rsx! {
        <div class="w-full pt-8 px-6 pb-24 md:pt-12 md:pb-32 md:px-8 lg:pt-12 lg:pb-0 lg:px-8" x-data="docPage">
            <div class=(wrapper_cls)>
                @if aside.is_some() {
                    <div class=(aside_cls)>
                    <div class="fixed bottom-0 z-10 w-56 h-8 bg-[linear-gradient(transparent,var(--vp-c-bg)_70%)] pointer-events-none"></div>
                    <div class="fixed top-0 pt-[calc(var(--vp-nav-height)+var(--vp-layout-top-height,0px)+var(--vp-doc-top-height,0px)+3rem)] w-56 h-screen overflow-x-hidden overflow-y-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                        <div class="flex flex-col grow min-h-[calc(100vh-(var(--vp-nav-height)+var(--vp-layout-top-height,0px)+3rem))] pb-8">
                            <div class="flex flex-col grow">
                                @if show_outline {
                                    <nav class="VPDocAsideOutline block" aria-labelledby="doc-outline-aria-label">
                                        <div class="relative border-l border-divider pl-4 text-[0.8125rem] font-medium">
                                            <div class="absolute top-8 left-[-1px] z-0 opacity-0 w-[2px] rounded-[2px] h-[1.125rem] bg-brand-1 [transition:top_0.25s_cubic-bezier(0.1,1,0.5,1),background-color_0.5s,opacity_0.25s]" id="VPOutlineMarker"></div>
                                            <div class="leading-[2.2857143] text-[0.875rem] font-semibold" id="doc-outline-aria-label" role="heading" aria-level="2">
                                                (outline_label.clone())
                                            </div>
                                            (outline_links(&headings))
                                        </div>
                                    </nav>
                                }
                                <div class="grow"></div>
                            </div>
                        </div>
                    </div>
                    </div>
                }

                <div class=(content_cls)>
                    <div class="mx-auto max-w-[43rem]">
                        <main>
                            <div id="main" class=(super::vpdoc::vpdoc_class())>
                                (Raw::dangerously_create(rendered.html.clone()))
                            </div>
                        </main>

                        @if show_footer {
                            <footer class="mt-16">
                                @if edit_link.is_some() || last_updated.is_some() {
                                    <div class="pb-[1.125rem] sm:flex sm:justify-between sm:items-center sm:pb-[0.875rem]">
                                        @if let Some((href, text)) = edit_link.clone() {
                                            <div class="edit-link">
                                                <a class="flex items-center border-0 leading-[2.2857143] text-[0.875rem] font-medium text-brand-1 transition-colors duration-[250ms] hover:text-brand-2" href=(href)>
                                                    (icon("square-pen", "mr-2"))
                                                    (text)
                                                </a>
                                            </div>
                                        }
                                        @if let Some((datetime, display)) = last_updated.clone() {
                                            <div class="last-updated">
                                                <p class="VPDocFooterLastUpdated text-[0.875rem] leading-[1.7142857] sm:leading-[2.2857143] font-medium text-text-2">
                                                    (format!("{}: ", updated_label))
                                                    <time datetime=(datetime)>(display)</time>
                                                </p>
                                            </div>
                                        }
                                    </div>
                                }

                                @if prev.is_some() || next.is_some() {
                                    <nav class="border-t border-divider pt-6 grid gap-y-2 sm:grid-cols-2 sm:gap-x-4" aria-labelledby="doc-footer-aria-label">
                                        <span class="sr-only" id="doc-footer-aria-label">"Pager"</span>
                                        <div class="pager">
                                            @if let Some(p) = prev {
                                                <a class="block border border-divider rounded-lg px-4 pt-[0.6875rem] pb-[0.8125rem] w-full h-full transition-colors duration-[250ms] hover:border-brand-1" href=(p.href.clone())>
                                                    <span class="block leading-[1.6666667] text-[0.75rem] font-medium text-text-2">(prev_label.clone())</span>
                                                    <span class="block leading-[1.4285714] text-[0.875rem] font-medium text-brand-1 transition-colors duration-[250ms]">(p.text.clone())</span>
                                                </a>
                                            }
                                        </div>
                                        <div class="pager">
                                            @if let Some(n) = next {
                                                <a class="block border border-divider rounded-lg px-4 pt-[0.6875rem] pb-[0.8125rem] w-full h-full transition-colors duration-[250ms] hover:border-brand-1 ml-auto text-right" href=(n.href.clone())>
                                                    <span class="block leading-[1.6666667] text-[0.75rem] font-medium text-text-2">(next_label.clone())</span>
                                                    <span class="block leading-[1.4285714] text-[0.875rem] font-medium text-brand-1 transition-colors duration-[250ms]">(n.text.clone())</span>
                                                </a>
                                            }
                                        </div>
                                    </nav>
                                }
                            </footer>
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}

/// `None` = use the default label; `Some(Ok(text))` = override;
/// `Some(Err(()))` = `false`, that pager side is disabled.
fn pager_label(label: &Option<crate::config::PagerLabel>, default: &str) -> Result<String, ()> {
    match label {
        None => Ok(default.to_string()),
        Some(crate::config::PagerLabel::Text(t)) => Ok(t.clone()),
        Some(crate::config::PagerLabel::Off(_)) => Err(()),
    }
}

/// Nested outline links (h3 nests under the preceding h2, VitePress's
/// outline behavior). The links are pre-rendered strings; the nesting is
/// the shared list builder (`markdown::nested_outline_list`) so the
/// outline and `[[toc]]` cannot drift apart again.
fn outline_links<'a>(headings: &'a [&'a Heading]) -> Raw<String> {
    let items: Vec<(u8, String)> = headings
        .iter()
        .map(|h| {
            (
                h.level,
                format!(
                    "<a class=\"outline-link block leading-[2.2857143] text-[0.875rem] font-normal text-text-2 whitespace-nowrap overflow-hidden text-ellipsis transition-colors duration-500 hover:text-text-1 hover:duration-[250ms] [&.active]:text-text-1\" href=\"#{}\" title=\"{}\">{}</a>",
                    h.id,
                    escape_attr(&h.text),
                    escape_text(&h.text),
                ),
            )
        })
        .collect();
    Raw::dangerously_create(crate::markdown::nested_outline_list(
        &items,
        " class=\"relative z-[1]\"",
        " class=\"pr-4 pl-4\"",
    ))
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Percent-encode a source-relative path for use in a URL: `/`
/// separators stay, unreserved bytes (RFC 3986) pass through, anything
/// else (spaces, non-ASCII, quotes) goes out as `%XX` UTF-8 bytes.
fn percent_encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for b in path.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// (datetime, display) for a file modification time — ISO `YYYY-MM-DD`
/// for the `<time>` attribute plus the VitePress-style medium label
/// ("Aug 13, 2026, 4:52:09 PM", what `toLocaleString(…, { dateStyle:
/// "medium", timeStyle: "medium" })` renders in an en-US locale), UTC
/// (civil-from-days; no chrono dependency).
pub fn format_date(t: std::time::SystemTime) -> (String, String) {
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let day_secs = secs.rem_euclid(86_400);
    let (hh, mi, ss) = (day_secs / 3600, day_secs % 3600 / 60, day_secs % 60);
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let hour12 = match hh {
        0 => 12,
        h if h > 12 => h - 12,
        h => h,
    };
    let suffix = if hh < 12 { "AM" } else { "PM" };
    (
        format!("{y:04}-{m:02}-{d:02}"),
        format!(
            "{} {d}, {y:04}, {hour12}:{mi:02}:{ss:02} {suffix}",
            MONTHS[(m - 1) as usize]
        ),
    )
}

/// Howard Hinnant's civil-from-days algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1)); // 2024-01-01
        assert_eq!(civil_from_days(20_652), (2026, 7, 18));
    }

    #[test]
    fn outline_nests_a_shallower_heading_after_a_deeper_one() {
        // ## A, #### Deep, ### B under `outline: deep`: B is a child of
        // A, not a sibling that escaped A's subtree when Deep's nesting
        // closed
        let mk = |level: u8, id: &str, text: &str| Heading {
            level,
            id: id.into(),
            text: text.into(),
            rendered_id: id.into(),
        };
        let headings = [mk(2, "a", "A"), mk(4, "deep", "Deep"), mk(3, "b", "B")];
        let refs: Vec<&Heading> = headings.iter().collect();
        let html = outline_links(&refs).into_inner();
        let a = html.find(">A</a>").unwrap();
        let b = html.find(">B</a>").unwrap();
        let nested_open = html[a..].find("<ul").unwrap() + a;
        let first_close = html.find("</ul>").unwrap();
        assert!(
            b > nested_open && b < first_close,
            "B must nest inside A's child list: {html}"
        );
    }
}
