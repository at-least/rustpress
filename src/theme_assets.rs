//! The theme's static assets — the built `main.css`, `js/app.js` and the
//! Inter font files — embedded at compile time so an installed binary
//! produces a complete site without a checkout of this repository.
//!
//! `build.rs` refuses to compile when the two built files are missing, so
//! the embedded set is never silently incomplete.

use std::path::Path;

use include_dir::{Dir, include_dir};

static THEME: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

/// Write every embedded theme file under `out_dir`, creating directories
/// as needed and overwriting files from a previous build. Callers copy
/// the site's own `static/` afterwards so it takes precedence.
pub fn extract(out_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    THEME.extract(out_dir)
}

/// Paths (relative to the output root) of every embedded file.
pub fn files() -> Vec<String> {
    fn walk(dir: &Dir<'_>, out: &mut Vec<String>) {
        for f in dir.files() {
            out.push(f.path().to_string_lossy().into_owned());
        }
        for d in dir.dirs() {
            walk(d, out);
        }
    }
    let mut out = Vec::new();
    walk(&THEME, &mut out);
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_built_theme_files_are_embedded() {
        let files = files();
        for required in ["main.css", "js/app.js"] {
            assert!(
                files.iter().any(|f| f == required),
                "missing {required} in {files:?}"
            );
        }
        assert!(
            files
                .iter()
                .any(|f| f.starts_with("fonts/") && f.ends_with(".woff2"))
        );
    }
}
