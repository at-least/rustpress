//! Coverage gate for FEATURE-PARITY.md: every `##`/`###` heading in the
//! pinned upstream reference docs must appear in the audit doc, so a new
//! upstream option/feature fails `npm test` until it gets a row.
//!
//! Reads `../vitepress/docs/en` (the vuejs/vitepress clone PARITY.md
//! uses as content source). When the clone is absent the test skips —
//! same contract as `npm run diff:upstream`.

use std::fmt::Write as _;
use std::path::PathBuf;

const REFERENCE_FILES: &[&str] = &[
    "reference/site-config.md",
    "reference/default-theme-config.md",
    "guide/markdown.md",
    "reference/frontmatter-config.md",
];

/// A fence run: opening char (` or ~) and its length.
fn fence_run(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start();
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let len = trimmed.chars().take_while(|&c| c == first).count();
    (len >= 3).then_some((first, len))
}

/// `##`/`###` headings from a docs page, skipping fenced code — the
/// example headings (`## Basics`, `## My Base Section`, …) live inside
/// fences. Same fence rule as the preprocessor: a closing fence is the
/// same char at >= the opening length.
fn headings(page: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for line in page.lines() {
        if let Some((open_ch, open_len)) = fence {
            if let Some((ch, len)) = fence_run(line) {
                if ch == open_ch && len >= open_len {
                    fence = None;
                }
            }
            continue;
        }
        if let Some(run) = fence_run(line) {
            fence = Some(run);
            continue;
        }
        let Some(rest) = line.strip_prefix("## ").or_else(|| line.strip_prefix("### ")) else {
            continue;
        };
        let title = strip_markup(rest);
        if !title.is_empty() {
            out.push(title);
        }
    }
    out
}

/// Normalize a heading the way FEATURE-PARITY.md keys its rows: drop
/// `{#anchor}` suffixes, `<Badge …>` tags, and code-span backticks.
/// Other braces/angles are literal (none appear in these files today).
fn strip_markup(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find(['<', '{', '`']) {
        out.push_str(&rest[..i]);
        let (open, close): (char, char) = match rest.as_bytes()[i] {
            b'<' => ('<', '>'),
            b'{' => ('{', '}'),
            _ => ('`', '`'),
        };
        if open == '`' {
            // code-span backticks: drop the marker, keep the content
            rest = &rest[i + 1..];
            continue;
        }
        let special = match open {
            '<' => rest[i..].starts_with("<Badge"),
            '{' => rest[i..].starts_with("{#"),
            _ => true, // code-span backtick
        };
        if !special {
            out.push(open);
            rest = &rest[i + 1..];
            continue;
        }
        match rest[i + 1..].find(close) {
            Some(j) => rest = &rest[i + 1 + j + close.len_utf8()..],
            None => {
                out.push(open);
                rest = &rest[i + 1..];
            }
        }
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn feature_parity_doc_covers_upstream_reference_headings() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let docs = manifest.parent().unwrap().join("vitepress/docs/en");
    if !docs.is_dir() {
        eprintln!("feature_parity: ../vitepress/docs/en not found — skipping");
        return;
    }
    let doc = std::fs::read_to_string(manifest.join("FEATURE-PARITY.md"))
        .expect("FEATURE-PARITY.md next to Cargo.toml");

    let mut missing = String::new();
    let mut covered = 0usize;
    for file in REFERENCE_FILES {
        let page = std::fs::read_to_string(docs.join(file))
            .unwrap_or_else(|e| panic!("read {}: {e}", docs.join(file).display()));
        for heading in headings(&page) {
            if doc.contains(&heading) {
                covered += 1;
            } else {
                writeln!(missing, "  {file}: {heading}").unwrap();
            }
        }
    }
    assert!(
        missing.is_empty(),
        "FEATURE-PARITY.md is missing rows for upstream headings \
         (add them, or the audit no longer covers the pinned release):\n{missing}"
    );
    assert!(covered > 100, "suspiciously few headings covered: {covered}");
}

#[test]
fn heading_extractor_ignores_fenced_examples() {
    let page = concat!(
        "# t\n\n## Real Heading\n\n",
        "```md\n## Inside A Fence\n\n```js\nnested\n```\n\n### Also Inside\n```\n\n",
        "## Real Sub {#anchor}\n\n### `raw` <Badge type=\"info\" text=\"x\" />\n",
    );
    assert_eq!(
        headings(page),
        vec![
            "Real Heading".to_string(),
            "Real Sub".to_string(),
            "raw".to_string(),
        ]
    );
}
