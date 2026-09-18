//! HTML escaping for the hand-built markup strings in `src/render/`.
//! One canonical entity set (`& < > "`) for every context the
//! renderers produce: element text and double-quoted attribute
//! values. Escapers that need a narrower set keep their own local
//! helpers; do not widen their output in this module's name.

/// Escape a string for element-text context.
pub(crate) fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

/// Escape a string for a double-quoted attribute value.
pub(crate) fn escape_attr(s: &str) -> String {
    escape_text(s)
}
