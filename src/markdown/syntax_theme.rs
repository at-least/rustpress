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
use std::path::Path;

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
    /// Resolve colors through the palette; styles whose colors cannot
    /// resolve (unknown palette/ANSI names) are dropped.
    fn resolve_colors(&self, palette: &BTreeMap<String, String>) -> Option<ThemeStyle> {
        let color = |c: &Option<String>| -> Option<String> {
            let raw = c.as_deref()?;
            if raw.starts_with('#') {
                Some(raw.to_string())
            } else {
                palette.get(raw).cloned()
            }
        };
        let fg = color(&self.fg)?;
        Some(ThemeStyle {
            fg: Some(fg),
            bg: self.bg.as_deref().and_then(|bg| {
                if bg.starts_with('#') {
                    Some(bg.to_string())
                } else {
                    palette.get(bg).cloned()
                }
            }),
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
    load_src(src, Path::new("."), &mut Vec::new()).ok()
}

/// Load a theme from a `.toml` file, resolving its `inherits` chain
/// (built-in names are valid parents; cycle-safe).
pub fn load(path: &Path) -> Result<SyntaxTheme, ThemeError> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| ThemeError::Parse(format!("cannot read {}: {e}", path.display())))?;
    load_src(&src, path.parent().unwrap_or(Path::new(".")), &mut Vec::new())
}

fn load_src(src: &str, base_dir: &Path, visiting: &mut Vec<String>) -> Result<SyntaxTheme, ThemeError> {
    let raw = RawTheme::parse(src)?;
    let mut theme = SyntaxTheme::default();
    for (scope, style) in &raw.styles {
        if let Some(resolved) = style.resolve_colors(&raw.palette) {
            theme.insert(scope.clone(), resolved);
        }
    }
    if let Some(parent) = &raw.inherits {
        if visiting.contains(parent) {
            return Err(ThemeError::Cycle(parent.clone()));
        }
        visiting.push(parent.clone());
        let parent_src = if parent.ends_with(".toml") {
            std::fs::read_to_string(base_dir.join(parent))
                .map_err(|e| ThemeError::Parse(format!("cannot read {}: {e}", base_dir.join(parent).display())))?
        } else {
            builtin_src(parent)
                .ok_or_else(|| ThemeError::UnknownInherit(parent.clone()))?
                .to_string()
        };
        let parent_theme = load_src(&parent_src, base_dir, visiting)?;
        visiting.pop();
        for (scope, style) in parent_theme.iter() {
            theme.styles.entry(scope.clone()).or_insert(style.clone());
        }
    }
    Ok(theme)
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
        let t = load_src(BASE, Path::new("."), &mut Vec::new()).unwrap();
        assert_eq!(t.resolve("keyword").unwrap().fg.as_deref(), Some("#d73a49"));
        assert!(t.resolve("comment").unwrap().italic);
    }

    #[test]
    fn dotted_scope_falls_back_to_parent() {
        let src = format!("{BASE}\n\"keyword.function\" = \"#005cc5\"\n");
        let t = load_src(&src, Path::new("."), &mut Vec::new()).unwrap();
        assert_eq!(t.resolve("keyword.function").unwrap().fg.as_deref(), Some("#005cc5"));
        assert_eq!(t.resolve("keyword.control").unwrap().fg.as_deref(), Some("#d73a49"));
        assert!(t.resolve("type").is_none());
    }
}
