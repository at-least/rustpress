//! HTML escaping for the hand-built markup strings in `src/render/`.
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
