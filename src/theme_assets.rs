//! The theme's static assets — the built `vitepress.css`, `js/app.js`, the
//! Inter font files and vpkit's color themes — embedded at compile time so
//! an installed binary produces a complete site without a checkout of this
//! repository.
//!
//! `build.rs` refuses to compile when the two built files or vpkit's
//! themes are missing, so the embedded set is never silently incomplete.

use std::path::Path;

use include_dir::{Dir, include_dir};

static THEME: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

/// vpkit's color themes (an npm `file:` dependency on ../vpkit), served
/// under themes/ next to the rest.
static COLOR_THEMES: Dir = include_dir!("$CARGO_MANIFEST_DIR/node_modules/vpkit/themes");

/// Write every embedded theme file under `out_dir`, creating directories
/// as needed and overwriting files from a previous build. Callers copy
/// the site's own `static/` afterwards so it takes precedence.
pub fn extract(out_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    THEME.extract(out_dir)?;
    let themes = out_dir.join("themes");
    std::fs::create_dir_all(&themes)?;
    COLOR_THEMES.extract(themes)
}

/// Paths (relative to the output root) of every embedded file.
pub fn files() -> Vec<String> {
    fn walk(dir: &Dir<'_>, prefix: &str, out: &mut Vec<String>) {
        for f in dir.files() {
            out.push(format!("{prefix}{}", f.path().to_string_lossy()));
        }
        for d in dir.dirs() {
            walk(d, prefix, out);
        }
    }
    let mut out = Vec::new();
    walk(&THEME, "", &mut out);
    walk(&COLOR_THEMES, "themes/", &mut out);
    out.sort();
    out
}

/// Names (file stems) of the bundled UI themes (vpkit's themes/), sorted.
/// Valid values for a bare `theme = "…"` in rustpress.toml.
pub fn bundled_themes() -> Vec<String> {
    let mut names: Vec<String> = files()
        .into_iter()
        .filter_map(|rel| rel.strip_prefix("themes/").map(str::to_string))
        .filter(|rel| rel.ends_with(".css"))
        .map(|rel| rel.strip_suffix(".css").unwrap_or(&rel).to_string())
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_built_theme_files_are_embedded() {
        let files = files();
        for required in ["vitepress.css", "js/app.js"] {
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
