//! Parity-mechanism tests: extraction against committed samples of real
//! upstream (vitepress.dev) and local (demo build) HTML, checker
//! behavior, snapshot determinism.

use std::path::Path;

use rustpress::parity::{self, Deltas, Mode};
use serde_json::{json, Value};

const UPSTREAM_DOC: &str = "tests/parity/fixtures/upstream-doc-page.html";
const LOCAL_DOC: &str = "tests/parity/fixtures/local-doc-page.html";
const UPSTREAM_HOME: &str = "tests/parity/fixtures/upstream-home.html";
const LOCAL_HOME: &str = "tests/parity/fixtures/local-home.html";

fn extract_fixture(path: &str) -> Value {
    extract_fixture_at(path, "/guide/what-is-vitepress")
}

fn extract_fixture_at(path: &str, url: &str) -> Value {
    let html = std::fs::read_to_string(path).unwrap();
    parity::extract(&html, url).to_value().unwrap()
}

#[test]
fn upstream_landmarks_extract() {
    let fp = extract_fixture(UPSTREAM_DOC);
    assert_eq!(fp["title"], "What is VitePress? | VitePress");
    assert_eq!(fp["h1"]["id"], "what-is-vitepress");
    assert_eq!(fp["nav_title"]["logo_src"], "/vitepress-logo-mini.svg");
    // nav: two links + the version flyout
    assert_eq!(fp["nav_items"].as_array().unwrap().len(), 3);
    assert_eq!(fp["nav_items"][0]["text"], "Guide");
    assert_eq!(fp["nav_items"][2]["kind"], "group");
    assert_eq!(fp["nav_items"][2]["text"], "2.0.0-alpha.20");
    // sidebar groups are headers in document order
    assert_eq!(fp["sidebar"][0]["kind"], "group");
    assert_eq!(fp["sidebar"][0]["text"], "Introduction");
    // the deployed site SSRs the outline shell but hydrates items
    // client-side: present-but-empty, NOT null
    assert_eq!(fp["outline_title"], "On this page");
    assert_eq!(fp["outline_items"], json!([]));
    assert_eq!(fp["pager"][0]["desc"], "Next page");
    assert_eq!(fp["pager"][0]["title"], "Getting Started");
    assert!(fp["edit_link"]
        .as_str()
        .unwrap()
        .ends_with("docs/en/guide/what-is-vitepress.md"));
    assert_eq!(fp["last_updated"], "2026-07-25");
    assert_eq!(fp["site_footer"][0], "Released under the MIT License.");
    assert!(fp["has_search"].as_bool().unwrap());
    assert!(fp["block_counts"].is_object());
}

#[test]
fn local_landmarks_extract() {
    let fp = extract_fixture(LOCAL_DOC);
    assert_eq!(fp["title"], "What is VitePress? | VitePress");
    assert_eq!(fp["h1"]["id"], "what-is-vitepress");
    // our outline is fully server-rendered (unlike upstream's hydrated
    // one) — items are present, and the local-nav copy must not double
    // them
    let items = fp["outline_items"].as_array().unwrap();
    assert!(!items.is_empty());
    let ids: Vec<_> = items.iter().map(|l| l["href"].clone()).collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len(), "outline items must be deduplicated");
    // sidebar config mirrors the vitepress.dev docs sidebar
    assert_eq!(fp["sidebar"][0]["kind"], "group");
    assert_eq!(fp["sidebar"][0]["text"], "Introduction");
    // hrefs are normalized to the upstream clean-URL shape
    assert_eq!(fp["nav_items"][0]["href"], "/guide/what-is-vitepress");
}

#[test]
fn local_matches_upstream_on_committed_fixtures() {
    // the fixtures are refreshed together with the baseline; this is the
    // single-page seed of what `parity check` enforces site-wide
    let up = extract_fixture(UPSTREAM_DOC);
    let ours = extract_fixture(LOCAL_DOC);
    let mut out = Vec::new();
    parity::compare("/p", &up, &ours, Mode::Check, &mut out);
    assert!(out.is_empty(), "unexpected divergences: {out:#?}");
}

#[test]
fn home_hero_and_features_extract() {
    for (fixture, _label) in [(UPSTREAM_HOME, "upstream"), (LOCAL_HOME, "local")] {
        let fp = extract_fixture_at(fixture, "/");
        assert_eq!(fp["hero"]["lines"][0], "VitePress");
        assert_eq!(fp["hero"]["tagline"], "Markdown to beautiful docs in minutes");
        // `./guide/x` resolves to the same normalized href on both sides
        assert_eq!(fp["hero"]["actions"][0]["href"], "/guide/what-is-vitepress");
        assert!(fp["features"].as_array().unwrap().len() >= 4);
        // home has no <main> on either side: block counts stay null
        assert!(fp["block_counts"].is_null());
    }
}

#[test]
fn extraction_is_deterministic() {
    // snapshots must be byte-stable: same HTML → same fingerprint JSON
    let a = extract_fixture(UPSTREAM_DOC);
    let b = extract_fixture(UPSTREAM_DOC);
    assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
}

#[test]
fn check_skips_null_upstream_and_presence_only_fields() {
    // last_updated values are site-local by nature: presence-only
    let up = json!({"last_updated": "2020-01-01", "title": "x"});
    let ours = json!({"last_updated": "2026-09-09", "title": "x"});
    let mut out = Vec::new();
    parity::compare("/p", &up, &ours, Mode::Check, &mut out);
    assert!(out.is_empty());
    // but a missing timestamp on our side IS a divergence
    let ours = json!({"last_updated": Value::Null, "title": "x"});
    let mut out = Vec::new();
    parity::compare("/p", &up, &ours, Mode::Check, &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].path, "last_updated");
}

#[test]
fn known_deltas_cover_mismatches() {
    let up = json!({"pager": [{"desc": "Next page", "title": "Deploy", "href": "/guide/deploy"}]});
    let ours = json!({"pager": [{"desc": "Next page", "title": "Deploy Your VitePress Site", "href": "/guide/deploy"}]});
    let mut out = Vec::new();
    parity::compare("/guide/deploy", &up, &ours, Mode::Check, &mut out);
    assert_eq!(out.len(), 1);

    let deltas: Deltas = serde_json::from_str(
        r#"{"deltas": [{"page": "/guide/deploy", "path": "pager[0].title",
             "upstream": "Deploy", "reason": "demo intentionally renames this page"}]}"#,
    )
    .unwrap();
    let remaining: Vec<_> = out
        .iter()
        .filter(|m| !deltas.covers(&m.page, &m.path, &m.upstream, &m.ours))
        .collect();
    assert!(remaining.is_empty());

    // a delta with non-matching upstream value does not cover it
    let deltas: Deltas = serde_json::from_str(
        r#"{"deltas": [{"page": "/guide/deploy", "path": "pager[0].title",
             "upstream": "Other", "reason": "nope"}]}"#,
    )
    .unwrap();
    let remaining: Vec<_> = out
        .iter()
        .filter(|m| !deltas.covers(&m.page, &m.path, &m.upstream, &m.ours))
        .collect();
    assert_eq!(remaining.len(), 1);
}

#[test]
fn diff_mode_reports_upstream_changes() {
    let old = json!({"pager": [{"desc": "Next page", "title": "Getting Started", "href": "/x"}]});
    let new = json!({"pager": [{"desc": "Next page", "title": "Quickstart", "href": "/y"}]});
    let mut out = Vec::new();
    parity::compare("/guide/a", &old, &new, Mode::Diff, &mut out);
    let paths: Vec<_> = out.iter().map(|m| m.path.as_str()).collect();
    assert!(paths.contains(&"pager[0].title"));
    assert!(paths.contains(&"pager[0].href"));
}

#[test]
fn snapshot_is_byte_stable_and_check_names_breakages() {
    // cache dir with two fetched pages (any HTML works for stability)
    let tmp = std::env::temp_dir().join(format!(
        "rustpress-parity-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    let upstream_html = std::fs::read_to_string(UPSTREAM_DOC).unwrap();
    let local_html = std::fs::read_to_string(LOCAL_DOC).unwrap();
    std::fs::write(tmp.join("guide_what-is-vitepress.html"), &upstream_html).unwrap();

    let pages = vec!["/guide/what-is-vitepress".to_string()];
    let meta = std::collections::BTreeMap::new();

    let a = parity::snapshot(&tmp, &pages, meta.clone()).unwrap();
    let b = parity::snapshot(&tmp, &pages, meta).unwrap();
    let ja = serde_json::to_string_pretty(&a).unwrap();
    let jb = serde_json::to_string_pretty(&b).unwrap();
    assert_eq!(ja, jb, "snapshot must be deterministic");

    // a site dir whose page is the LOCAL render must pass; then break
    // the sidebar and confirm the checker names the page and landmark
    let site = tmp.join("site").join("guide").join("what-is-vitepress");
    std::fs::create_dir_all(&site).unwrap();
    std::fs::write(site.join("index.html"), &local_html).unwrap();
    let deltas = Deltas::default();
    let clean = parity::check(&tmp.join("site"), &a, &deltas);
    assert!(clean.is_empty(), "unexpected: {clean:#?}");

    let broken_html = local_html.replace(">Introduction<", ">Intro<");
    assert_ne!(broken_html, local_html, "mutation not applied");
    std::fs::write(site.join("index.html"), &broken_html).unwrap();
    let mismatches = parity::check(&tmp.join("site"), &a, &deltas);
    // the sidebar rename must surface (page + landmark named)
    assert!(
        mismatches
            .iter()
            .any(|m| m.page == "/guide/what-is-vitepress" && m.path.starts_with("sidebar")),
        "expected a sidebar mismatch, got: {mismatches:#?}"
    );
    std::fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn site_file_and_slug_mapping() {
    assert_eq!(parity::slug_for("/"), "home");
    assert_eq!(parity::slug_for("/guide/markdown"), "guide_markdown");
    assert_eq!(
        parity::site_file_for(Path::new("demo/public"), "/guide/markdown"),
        Path::new("demo/public/guide/markdown/index.html")
    );
    assert_eq!(
        parity::site_file_for(Path::new("demo/public"), "/"),
        Path::new("demo/public/index.html")
    );
}
