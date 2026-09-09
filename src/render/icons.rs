//! Inline SVG icons (lucide, ISC) — hypertext's `rsx!` element set is
//! HTML-only, so like crashcart every icon is a fixed literal string
//! wrapped in `Raw::dangerously_create` (no dynamic content reaches
//! icons, so this is safe) with the caller's class appended.

use hypertext::Raw;

pub fn icon(name: &str, class: &str) -> Raw<String> {
    let base = match name {
        "search" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="m21 21l-4.34-4.34"/><circle cx="11" cy="11" r="8"/></svg>"#,
        "sparkles" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" aria-hidden="true"><path d="M11.017 2.814a1 1 0 0 1 1.966 0l1.051 5.558a2 2 0 0 0 1.594 1.594l5.558 1.051a1 1 0 0 1 0 1.966l-5.558 1.051a2 2 0 0 0-1.594 1.594l-1.051 5.558a1 1 0 0 1-1.966 0l-1.051-5.558a2 2 0 0 0-1.594-1.594l-5.558-1.051a1 1 0 0 1 0-1.966l5.558-1.051a2 2 0 0 0 1.594-1.594zM20 2v4m2-2h-4"/><circle cx="4" cy="20" r="2"/></svg>"#,
        "chevron-right" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="m9 18l6-6l-6-6"/></svg>"#,
        "chevron-down" => r#"<svg class="inline-block size-[1em] rotate-90 {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="m9 18l6-6l-6-6"/></svg>"#,
        "sun" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M4.93 4.93l1.41 1.41m11.32 11.32l1.41 1.41M2 12h2m16 0h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></svg>"#,
        "moon" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="M12 3a6 6 0 0 0 9 9a9 9 0 1 1-9-9"/></svg>"#,
        "align-left" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="M15 12H3m14 6H3M21 6H3"/></svg>"#,
        "heart" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2c-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"/></svg>"#,
        "square-pen" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.375 2.625a1 1 0 0 1 3 3l-9.013 9.014a2 2 0 0 1-.853.505l-2.873.84a.5.5 0 0 1-.62-.62l.84-2.873a2 2 0 0 1 .506-.852z"/></svg>"#,
        "plus" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="M5 12h14m-7-7v14"/></svg>"#,
        "delete" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="M10 5a2 2 0 0 0-1.344.519l-6.328 5.74a1 1 0 0 0 0 1.481l6.328 5.741A2 2 0 0 0 10 19h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2zm2 4l6 6m0-6l-6 6"/></svg>"#,
        "github" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 .297c-6.63 0-12 5.373-12 12c0 5.303 3.438 9.8 8.205 11.385c.6.113.82-.258.82-.577c0-.285-.01-1.04-.015-2.04c-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729c1.205.084 1.838 1.236 1.838 1.236c1.07 1.835 2.809 1.305 3.495.998c.108-.776.417-1.305.76-1.605c-2.665-.3-5.466-1.332-5.466-5.93c0-1.31.465-2.38 1.235-3.22c-.135-.303-.54-1.523.105-3.176c0 0 1.005-.322 3.3 1.23c.96-.267 1.98-.399 3-.405c1.02.006 2.04.138 3 .405c2.28-1.552 3.285-1.23 3.285-1.23c.645 1.653.24 2.873.12 3.176c.765.84 1.23 1.91 1.23 3.22c0 4.61-2.805 5.625-5.475 5.92c.42.36.81 1.096.81 2.22c0 1.606-.015 2.896-.015 3.286c0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>"#,
        "copy" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>"#,
        "copy-checked" => r#"<svg class="inline-block size-[1em] {c}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><path d="m9 14l2 2l4-4"/></svg>"#,
        _ => "",
    };
    Raw::dangerously_create(base.replace("{c}", class))
}

/// The X (twitter) and generic external-link glyphs used by social links.
pub fn social_icon(kind: &str, class: &str) -> Raw<String> {
    let svg = match kind {
        "github" => {
            return icon("github", class);
        }
        "twitter" | "x" => r#"<span class="flex w-5 h-5"><svg class="fill-current size-full" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M18.9 2.1h3.4l-7.4 8.5 8.7 11.5h-6.8l-5.3-7-6.1 7H1.9l7.9-9.1L1.4 2.1h7l4.8 6.3 5.7-6.3Zm-1.2 18h1.9L7.4 4H5.4l12.3 16.1Z"/></svg></span>"#.to_string(),
        _ => r#"<span class="flex w-5 h-5"><svg class="fill-current size-full" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path fill="none" stroke="currentColor" stroke-width="2" d="M14 5h5v5m0-5L10 14M9 5H6a2 2 0 0 0-2 2v11a2 2 0 0 0 2 2h11a2 2 0 0 0 2-2v-3"/></svg></span>"#.to_string(),
    };
    let _ = class; // glyph size is fixed by the wrapping span, like the Tera theme
    Raw::dangerously_create(svg)
}
