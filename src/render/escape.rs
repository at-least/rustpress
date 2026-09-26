//! HTML escaping and URL percent-encoding for the hand-built markup
//! strings in `src/render/`.
//! One canonical entity set (`& < > "`) for every context the
//! renderers produce: element text and double-quoted attribute
//! values. Escapers that need a narrower set keep their own local
//! helpers; do not widen their output in this module's name.

/// Escape a string for element-text context — the same `& < > "` set
/// the markdown side already implements; one implementation, two
/// names for the two call-site vocabularies.
pub(crate) fn escape_text(s: &str) -> String {
    crate::markdown::preprocess::escape_text(s)
}

/// Escape a string for a double-quoted attribute value.
pub(crate) fn escape_attr(s: &str) -> String {
    escape_text(s)
}

/// RFC 3986 percent-encoding of a path: unreserved characters and `/`
/// pass through, every other byte becomes `%XX`. `keep_ampersand`
/// leaves `&` raw for the sitemap and the search index, whose writers
/// XML/JSON-escape it themselves (a file named `a&b.md` must reach the
/// sitemap as `a&amp;b`, not `a%26b`).
pub(crate) fn percent_encode(s: &str, keep_ampersand: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                out.push(b as char)
            }
            b'&' if keep_ampersand => out.push('&'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
