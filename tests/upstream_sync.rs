//! Verbatim-corpus gates: the VitePress-format content rustpress tests
//! and builds consume must stay a byte-identical copy of the pinned
//! upstream release (parity/upstream-ref.txt). Hand edits to make a
//! test pass — upstream drift quietly adopted — fail here; the only fix
//! is a re-copy from the clone (see PARITY.md).

mod common;

use std::fmt::Write as _;
use std::path::Path;

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            walk(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

#[test]
fn fixtures_are_verbatim_upstream() {
    let Some(docs) = common::ensure_upstream() else { return };
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/en");
    let mut files = Vec::new();
    walk(&fixtures, &mut files).expect("walk tests/fixtures/en");
    assert!(!files.is_empty(), "fixture corpus is empty");

    let mut broken = String::new();
    for file in &files {
        let rel = file.strip_prefix(&fixtures).unwrap();
        let ours = std::fs::read(file).unwrap();
        match std::fs::read(docs.join(rel)) {
            Ok(upstream) if upstream == ours => {}
            Ok(_) => {
                writeln!(broken, "  {}: edited from upstream", rel.display()).unwrap();
            }
            Err(e) => {
                writeln!(broken, "  {}: not in the upstream clone ({e})", rel.display()).unwrap();
            }
        }
    }
    assert!(
        broken.is_empty(),
        "tests/fixtures/en diverged from the pinned upstream clone — \
         re-copy from ../vitepress/docs/en, never edit fixtures by hand:\n{broken}"
    );
}

#[test]
fn demo_content_mirrors_upstream() {
    let Some(docs) = common::ensure_upstream() else { return };
    let demo = Path::new(env!("CARGO_MANIFEST_DIR")).join("demo/content");
    let mut ours = Vec::new();
    walk(&demo, &mut ours).expect("walk demo/content");
    let mut upstream = Vec::new();
    walk(&docs, &mut upstream).expect("walk ../vitepress/docs/en");

    let mut broken = String::new();
    // both directions: demo/content is a full mirror, so extra local
    // files are drift just like missing or edited ones
    for file in &ours {
        let rel = file.strip_prefix(&demo).unwrap();
        match std::fs::read(docs.join(rel)) {
            Ok(upstream) if upstream == std::fs::read(file).unwrap() => {}
            Ok(_) => writeln!(broken, "  {}: edited from upstream", rel.display()).unwrap(),
            Err(e) => writeln!(broken, "  {}: not upstream ({e})", rel.display()).unwrap(),
        }
    }
    for file in &upstream {
        let rel = file.strip_prefix(&docs).unwrap();
        if !demo.join(rel).exists() {
            writeln!(broken, "  {}: missing from demo/content", rel.display()).unwrap();
        }
    }
    assert!(
        broken.is_empty(),
        "demo/content is not a verbatim mirror of ../vitepress/docs/en — \
         re-copy the changed pages (npm run diff:upstream lists them):\n{broken}"
    );
}
