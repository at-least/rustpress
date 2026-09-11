//! The bundled-theme contract.
//!
//! A bundled theme is a *complete design*: it defines every design token
//! the compiled base stylesheet consumes, so no stock value leaks through
//! when it is linked. These tests derive the required token set from
//! `styles/vitepress.css` itself (the structural layer — utilities and
//! component rules — resolves through these `--vp-*` custom properties)
//! and then hold every `static/themes/*.css` to it, plus WCAG contrast
//! minimums for the roles each token plays (body text, links, buttons,
//! badge text) in both light and dark mode.

use std::path::PathBuf;

const RAMPS: [&str; 7] = ["gray", "indigo", "purple", "green", "yellow", "orange", "red"];
/// Absolute colors, defined once in the base and never themed.
const ABSOLUTES: [&str; 2] = ["white", "black"];

fn repo(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

struct Block {
    selector: String,
    declarations: Vec<(String, String)>,
}

/// Parse a stylesheet into top-level `selector { decl; decl; }` blocks,
/// comments stripped. Theme files and the token layer of the base are
/// flat, which is all this needs to read.
fn parse_blocks(css: &str) -> Vec<Block> {
    let css = strip_comments(css);
    let mut blocks = Vec::new();
    let mut rest = css.as_str();
    while let Some(open) = rest.find('{') {
        let selector = rest[..open].split_whitespace().collect::<Vec<_>>().join(" ");
        let mut depth = 1;
        let mut close = open + 1;
        let bytes = rest.as_bytes();
        while close < bytes.len() && depth > 0 {
            match bytes[close] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            close += 1;
        }
        let body = &rest[open + 1..close.saturating_sub(1)];
        let declarations = body
            .split(';')
            .filter_map(|decl| decl.split_once(':'))
            .map(|(prop, value)| (prop.trim().to_string(), value.trim().to_string()))
            .filter(|(prop, _)| !prop.is_empty())
            .collect();
        blocks.push(Block { selector, declarations });
        rest = &rest[close..];
    }
    blocks
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let end = rest[start..].find("*/").map(|i| start + i + 2).unwrap_or(rest.len());
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

fn is_ramp(token: &str) -> bool {
    RAMPS.iter().any(|r| token == *r || token.starts_with(&format!("{r}-")))
}

/// Every `--vp-c-*` token referenced anywhere in the base stylesheet,
/// minus absolutes — the design contract. Derived, not hand-maintained,
/// so it cannot drift from what the structure consumes.
fn required_tokens() -> Vec<String> {
    let css = std::fs::read_to_string(repo("styles/vitepress.css")).unwrap();
    let refs: Vec<String> = parse_refs(&css);
    let mut tokens: Vec<String> = refs
        .into_iter()
        .filter(|t| !is_ramp(t) && !ABSOLUTES.contains(&t.as_str()))
        .collect();
    tokens.sort();
    tokens.dedup();
    assert!(tokens.len() >= 30, "contract implausibly small: {tokens:?}");
    tokens
}

fn parse_refs(css: &str) -> Vec<String> {
    let mut refs = Vec::new();
    for decl in declarations(css) {
        // Guard: a ramp token may only appear *inside* a token-layer
        // definition (`--vp-c-success-1: var(--vp-c-green-1)`), never in
        // a structural rule (`background-color: var(--vp-c-green-soft)`)
        // — otherwise themes that don't define ramps leak stock colors.
        for name in decl.value_matches() {
            if is_ramp(&name) {
                assert!(
                    decl.property.starts_with("--vp-c-"),
                    "ramp token --vp-c-{name} referenced by structural rule `{}` in the base",
                    decl.property
                );
            }
            refs.push(name);
        }
    }
    refs
}

struct Decl {
    property: String,
    value: String,
}

fn declarations(css: &str) -> Vec<Decl> {
    let css = strip_comments(css);
    let parts: Vec<&str> = css.split([';', '{', '}']).collect();
    parts
        .into_iter()
        .filter_map(|part| part.split_once(':'))
        .map(|(property, value)| Decl {
            property: property.trim().to_string(),
            value: value.trim().to_string(),
        })
        .collect()
}

impl Decl {
    fn value_matches(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = self.value.as_str();
        while let Some(start) = rest.find("var(--vp-c-") {
            rest = &rest[start + "var(--vp-c-".len()..];
            let end = rest.find(')').unwrap_or(rest.len());
            out.push(rest[..end].trim().to_string());
            rest = &rest[end..];
        }
        out
    }
}

struct ThemeMode {
    values: std::collections::HashMap<String, String>,
}

fn theme_file(name: &str) -> String {
    let path = repo(&format!("static/themes/{name}.css"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

fn mode_values(css: &str, selector: &str) -> ThemeMode {
    let mut values = std::collections::HashMap::new();
    for block in parse_blocks(css) {
        if block.selector == selector {
            for (prop, value) in block.declarations {
                values.insert(prop, value);
            }
        }
    }
    ThemeMode { values }
}

impl ThemeMode {
    fn hex(&self, token: &str) -> (u8, u8, u8) {
        let value = self
            .values
            .get(&format!("--vp-c-{token}"))
            .unwrap_or_else(|| panic!("token --vp-c-{token} not defined"));
        let hex = value
            .strip_prefix('#')
            .unwrap_or_else(|| panic!("--vp-c-{token} = {value:?}: expected a hex literal"));
        assert!(hex.len() == 6, "--vp-c-{token} = {value:?}: expected 6-digit hex");
        let n = u32::from_str_radix(hex, 16).expect("valid hex");
        (((n >> 16) & 0xff) as u8, ((n >> 8) & 0xff) as u8, (n & 0xff) as u8)
    }

    /// Resolve a token to a solid color: hex directly, `rgba(r,g,b,a)`
    /// composited over the given backdrop.
    fn color_over(&self, token: &str, backdrop: (u8, u8, u8)) -> (u8, u8, u8) {
        let value = self
            .values
            .get(&format!("--vp-c-{token}"))
            .unwrap_or_else(|| panic!("token --vp-c-{token} not defined"));
        if let Some(hex) = value.strip_prefix('#') {
            assert!(hex.len() == 6, "--vp-c-{token} = {value:?}: expected 6-digit hex");
            let n = u32::from_str_radix(hex, 16).expect("valid hex");
            return (((n >> 16) & 0xff) as u8, ((n >> 8) & 0xff) as u8, (n & 0xff) as u8);
        }
        let rgba: Vec<f64> = value
            .strip_prefix("rgba(")
            .and_then(|v| v.strip_suffix(")"))
            .unwrap_or_else(|| panic!("--vp-c-{token} = {value:?}: expected #hex or rgba()"))
            .split(',')
            .map(|n| n.trim().parse::<f64>().expect("numeric rgba component"))
            .collect();
        assert!(rgba.len() == 4, "--vp-c-{token} = {value:?}");
        let mix = |fg: f64, bg: u8| (fg * rgba[3] + f64::from(bg) * (1.0 - rgba[3])).round() as u8;
        (
            mix(rgba[0], backdrop.0),
            mix(rgba[1], backdrop.1),
            mix(rgba[2], backdrop.2),
        )
    }
}

fn luminance((r, g, b): (u8, u8, u8)) -> f64 {
    let channel = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

fn contrast(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = (la.max(lb), la.min(lb));
    (hi + 0.05) / (lo + 0.05)
}

fn assert_contrast(mode: &ThemeMode, token: &str, against: (u8, u8, u8), min: f64, label: &str) {
    let ratio = contrast(mode.hex(token), against);
    assert!(
        ratio >= min - 1e-9,
        "{label}: --vp-c-{token} {} has contrast {ratio:.2} < {min} against {}",
        format!("{:?}", mode.hex(token)),
        format!("{against:?}"),
    );
}

fn bundled_theme_names() -> Vec<String> {
    let dir = repo("static/themes");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "css"))
        .filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    assert!(!names.is_empty(), "no bundled themes found");
    names
}

#[test]
fn graded_containers_override_survives_themes() {
    // gradedContainers opts into GitHub-style severity colors; the rule
    // must carry its own literals with :has() specificity so it beats
    // any theme's :root/.dark warning/caution (which load later).
    let css = std::fs::read_to_string(repo("styles/vitepress.css")).unwrap();
    for selector in [
        ":root:has(.vp-graded-containers)",
        ":root.dark:has(.vp-graded-containers)",
    ] {
        let blocks = parse_blocks(&css);
        let block = blocks
            .iter()
            .find(|b| b.selector == selector)
            .unwrap_or_else(|| panic!("missing `{selector}` rule"));
        for token in ["--vp-c-warning-1", "--vp-c-caution-1"] {
            let value = block
                .declarations
                .iter()
                .find(|(prop, _)| prop == token)
                .unwrap_or_else(|| panic!("`{selector}` does not define {token}"));
            assert!(
                value.1.starts_with('#') || value.1.starts_with("rgba("),
                "`{selector}` {token} = {:?}: expected a literal, not a stock ramp reference",
                value.1
            );
        }
    }
}

#[test]
fn docs_theme_index_matches_the_bundled_set() {
    // docs/static/themes.json feeds the docs site's live theme gallery
    // (generated by scripts/gen-theme-index.mjs, run by build:docs).
    // It must list exactly what the binary embeds — a stale index ships
    // a stale gallery.
    let json = std::fs::read_to_string(repo("docs/static/themes.json"))
        .expect("docs/static/themes.json exists (run: node scripts/gen-theme-index.mjs)");
    let listed: Vec<String> = serde_json::from_str(&json).expect("a JSON array of names");
    let bundled = rustpress::theme_assets::bundled_themes();
    assert_eq!(listed, bundled, "docs theme index drifted from static/themes/");
}

#[test]
fn themes_are_complete_designs() {
    let required = required_tokens();
    for name in bundled_theme_names() {
        let css = theme_file(&name);
        for selector in [":root", ".dark"] {
            let mode = mode_values(&css, selector);
            let missing: Vec<&String> = required
                .iter()
                .filter(|token| !mode.values.contains_key(&format!("--vp-c-{token}")))
                .collect();
            assert!(
                missing.is_empty(),
                "theme {name}, {selector}: missing tokens {missing:?} — a bundled theme must define the whole contract"
            );
        }
        let root = mode_values(&css, ":root");
        for i in 1..=5 {
            assert!(
                root.values.contains_key(&format!("--vp-shadow-{i}")),
                "theme {name}: missing --vp-shadow-{i}"
            );
        }
    }
}

#[test]
fn themes_meet_contrast_minimums() {
    let white = (0xff, 0xff, 0xff);
    for name in bundled_theme_names() {
        let css = theme_file(&name);
        for selector in [":root", ".dark"] {
            let mode = mode_values(&css, selector);
            let label = format!("theme {name} {selector}");
            let bg = mode.hex("bg");

            // body text
            assert_contrast(&mode, "text-1", bg, 7.0, &label);
            // secondary text also sits on the sidebar / code-block surfaces
            assert_contrast(&mode, "text-2", bg, 4.5, &label);
            assert_contrast(&mode, "text-2", mode.hex("bg-alt"), 4.5, &label);
            assert_contrast(&mode, "text-2", mode.hex("bg-soft"), 4.5, &label);
            // muted text
            assert_contrast(&mode, "text-3", bg, 3.0, &label);

            // links and inline code
            assert_contrast(&mode, "brand-1", bg, 4.5, &label);
            // hero button: white text on the brand background in every
            // state (default -3, hover -2)
            assert_contrast(&mode, "brand-3", white, 3.0, &label);
            assert_contrast(&mode, "brand-2", white, 3.0, &label);

            // badge / container foregrounds, measured against their own
            // soft background (composited over the page bg when rgba)
            for kind in ["tip", "note", "success", "important", "warning", "danger", "caution"] {
                let soft = mode.color_over(&format!("{kind}-soft"), bg);
                assert_contrast(&mode, &format!("{kind}-1"), soft, 4.5, &label);
            }
        }
    }
}
