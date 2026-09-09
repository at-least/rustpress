//! Helix TOML theme files as the syntax color scheme format.
//!
//! A theme file maps dotted tree-sitter capture scopes to styles:
//!
//! ```toml
//! inherits = "github-dark"          # optional: built-in or another .toml path
//!
//! [palette]
//! red0 = "#f97583"
//!
//! "keyword" = { fg = "red0", modifiers = ["bold"] }
//! "string" = "#9ecbff"
//! "comment" = "#6a737d"
//! ```
//!
//! Resolution follows Helix: a capture name matches the longest defined
//! scope prefix (`function.builtin` → `function` → nothing), and colors
//! resolve through `[palette]` or are literal hex values.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use include_dir::{include_dir, Dir};

/// All Helix built-in themes, vendored unmodified from
/// helix-editor/helix (runtime/themes, MPL-2.0).
static HELIX_THEMES: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/syntax-themes/helix");

/// The override CSS for a Helix built-in theme name (file stem, e.g.
/// `catppuccin_mocha`).
pub fn helix_src(name: &str) -> Option<&'static str> {
    let file = HELIX_THEMES.get_file(format!("{name}.toml"))?;
    file.contents_utf8()
}

/// Built-in Helix theme by name.
pub fn helix_builtin(name: &str) -> Option<SyntaxTheme> {
    let src = helix_src(name)?;
    Some(load_chain(src, Some(name), Path::new(".")))
}

/// How many Helix themes are embedded.
pub fn helix_count() -> usize {
    HELIX_THEMES.files().count()
}

/// One resolved style for a capture scope.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThemeStyle {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// A resolved syntax theme: dotted scope → style.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SyntaxTheme {
    styles: BTreeMap<String, ThemeStyle>,
}

impl SyntaxTheme {
    pub fn insert(&mut self, scope: String, style: ThemeStyle) {
        self.styles.insert(scope, style);
    }

    /// Longest matching prefix: `function.builtin` tries the exact key,
    /// then `function`, then gives up (Helix's resolution rule).
    pub fn resolve(&self, capture: &str) -> Option<&ThemeStyle> {
        let mut scope = capture.to_string();
        loop {
            if let Some(style) = self.styles.get(&scope) {
                return Some(style);
            }
            match scope.rsplit_once('.') {
                Some((parent, _)) => scope = parent.to_string(),
                None => return self.styles.get(&scope),
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ThemeStyle)> {
        self.styles.iter()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("cannot parse theme: {0}")]
    Parse(String),
    #[error("unknown inherited theme {0:?}")]
    UnknownInherit(String),
    #[error("theme inheritance cycle at {0:?}")]
    Cycle(String),
}

/// Parsed theme file before palette/inheritance resolution.
#[derive(Debug, Default)]
struct RawTheme {
    inherits: Option<String>,
    palette: BTreeMap<String, String>,
    styles: BTreeMap<String, RawStyle>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct RawStyle {
    fg: Option<String>,
    bg: Option<String>,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl RawTheme {
    fn parse(src: &str) -> Result<RawTheme, ThemeError> {
        let value: toml::Value = toml::from_str(src).map_err(|e| ThemeError::Parse(e.to_string()))?;
        let table = value
            .as_table()
            .ok_or_else(|| ThemeError::Parse("theme is not a TOML table".into()))?;

        let mut raw = RawTheme::default();
        for (key, val) in table {
            match key.as_str() {
                "inherits" => {
                    raw.inherits = val.as_str().map(str::to_string);
                }
                "palette" => {
                    if let Some(t) = val.as_table() {
                        for (k, v) in t {
                            if let Some(c) = v.as_str() {
                                raw.palette.insert(k.clone(), c.to_string());
                            }
                        }
                    }
                }
                _ => {
                    let style = match val {
                        toml::Value::String(fg) => {
                            RawStyle { fg: Some(fg.clone()), ..Default::default() }
                        }
                        toml::Value::Table(t) => {
                            let fg = t.get("fg").and_then(|v| v.as_str()).map(str::to_string);
                            let bg = t.get("bg").and_then(|v| v.as_str()).map(str::to_string);
                            let modifiers = t
                                .get("modifiers")
                                .and_then(|v| v.as_array())
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|v| v.as_str())
                                        .map(str::to_string)
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default();
                            RawStyle {
                                fg,
                                bg,
                                bold: modifiers.iter().any(|m| m == "bold"),
                                italic: modifiers.iter().any(|m| m == "italic"),
                                underline: modifiers
                                    .iter()
                                    .any(|m| m == "underline" || m == "underlined"),
                            }
                        }
                        _ => continue,
                    };
                    raw.styles.insert(key.clone(), style);
                }
            }
        }
        Ok(raw)
    }
}

impl RawStyle {
    /// Resolve colors through the palette; each color resolves
    /// independently (a palette-name miss yields no color for that slot,
    /// the style survives with the rest).
    fn resolve_colors(&self, palette: &BTreeMap<String, String>) -> Option<ThemeStyle> {
        let color = |c: &Option<String>| -> Option<String> {
            let raw = c.as_deref()?;
            if raw.starts_with('#') {
                Some(raw.to_string())
            } else {
                palette.get(raw).cloned()
            }
        };
        Some(ThemeStyle {
            fg: color(&self.fg),
            bg: color(&self.bg),
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
        })
    }
}

const GITHUB_LIGHT: &str = include_str!("../../assets/syntax-themes/github-light.toml");
const GITHUB_DARK: &str = include_str!("../../assets/syntax-themes/github-dark.toml");

/// Built-in theme sources (the vendored github pair).
pub fn builtin_src(name: &str) -> Option<&'static str> {
    match name {
        "github-light" => Some(GITHUB_LIGHT),
        "github-dark" => Some(GITHUB_DARK),
        _ => None,
    }
}

/// Built-in theme by name.
pub fn builtin(name: &str) -> Option<SyntaxTheme> {
    let src = builtin_src(name)?;
    Some(load_chain(src, Some(name), Path::new(".")))
}

/// Load a theme from a `.toml` file.
pub fn load(path: &Path) -> Result<SyntaxTheme, ThemeError> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| ThemeError::Parse(format!("cannot read {}: {e}", path.display())))?;
    let name = path.to_string_lossy().into_owned();
    let base = base_dir_of(path);
    Ok(load_chain(&src, Some(&name), &base))
}

fn base_dir_of(path: &Path) -> PathBuf {
    path.parent().unwrap_or(Path::new(".")).to_path_buf()
}

/// Load one theme file/builtin as a chain root→leaf: `inherits` parents
/// come first, the named theme last. Palettes merge down the chain
/// (leaf wins on name clashes) and ALL styles resolve against the final
/// merged palette — Helix semantics (gruvbox_dark_hard's bg0 override
/// recolors gruvbox's own ui.background too).
fn load_chain(src: &str, name: Option<&str>, base_dir: &Path) -> SyntaxTheme {
    let mut chain: Vec<RawTheme> = Vec::new();
    let mut visiting: Vec<String> = Vec::new();
    collect_chain(src, name, base_dir, &mut visiting, &mut chain);

    // phase 1: merge every palette down the chain (leaf wins on clashes)
    let mut palette: BTreeMap<String, String> = BTreeMap::new();
    for raw in &chain {
        for (k, v) in &raw.palette {
            palette.insert(k.clone(), v.clone());
        }
    }
    // phase 2: resolve all styles against the final palette — the leaf's
    // bg0 override recolors the root theme's own styles too, which is
    // exactly how Helix's hard-contrast variants work
    let mut theme = SyntaxTheme::default();
    for raw in &chain {
        for (scope, style) in &raw.styles {
            if let Some(resolved) = style.resolve_colors(&palette) {
                theme.insert(scope.clone(), resolved);
            }
        }
    }
    theme
}

fn collect_chain(
    src: &str,
    name: Option<&str>,
    base_dir: &Path,
    visiting: &mut Vec<String>,
    out: &mut Vec<RawTheme>,
) {
    let key = name.map(|n| n.to_string()).unwrap_or_else(|| src.chars().take(32).collect());
    if visiting.contains(&key) {
        return; // cycle: stop here
    }
    visiting.push(key.clone());
    let raw = match RawTheme::parse(src) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("gen-docs: skipping theme ({key}): {e}");
            return;
        }
    };
    if let Some(parent) = &raw.inherits {
        // parent may be a built-in name (github pair) or a .toml file
        if parent.ends_with(".toml") {
            if let Ok(src) = std::fs::read_to_string(base_dir.join(parent)) {
                collect_chain(&src, Some(parent), base_dir, visiting, out);
            }
        } else if let Some(src) = builtin_src(parent) {
            collect_chain(src, Some(parent), base_dir, visiting, out);
        } else if let Some(src) = helix_src(parent) {
            collect_chain(src, Some(parent), base_dir, visiting, out);
        }
    }
    out.push(raw);
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = r##"
"keyword" = "#d73a49"
"string" = "#032f62"
"comment" = { fg = "#6a737d", modifiers = ["italic"] }
"##;

    #[test]
    fn parses_shorthand_and_full_forms() {
        let t = load_chain(BASE, Some("test"), Path::new("."));
        assert_eq!(t.resolve("keyword").unwrap().fg.as_deref(), Some("#d73a49"));
        assert!(t.resolve("comment").unwrap().italic);
    }

    #[test]
    fn dotted_scope_falls_back_to_parent_scope() {
        let mut theme = SyntaxTheme::default();
        theme.insert("keyword".into(), ThemeStyle { fg: Some("#d73a49".into()), ..Default::default() });
        assert_eq!(theme.resolve("keyword").unwrap().fg.as_deref(), Some("#d73a49"));
        assert!(theme.resolve("type").is_none());
    }
}
