//! The appearance switch and the sidebar must not leave Alpine with
//! anything to choke on: the switch carries both tooltip titles so the
//! bundle can show the right one for the current mode, and sidebar
//! sections only carry Alpine bindings when they are collapsible (an
//! empty `:class=""` is a JavaScript syntax error at runtime).

use std::path::{Path, PathBuf};

use rustpress::render::Site;

fn temp_site(name: &str) -> PathBuf {
    // every test in this file passes the same `name`, and cargo runs them
    // in parallel threads of one process — key the directory per thread or
    // the tests clobber each other's temp sites
    let dir = std::env::temp_dir().join(format!(
        "rustpress-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("content/guide")).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, body: &str) {
    std::fs::write(dir.join(rel), body).unwrap();
}

fn build() -> String {
    let site = temp_site("switch");
    write(
        &site,
        "rustpress.toml",
        "title = \"probe\"\nlightModeSwitchTitle = \"Go light\"\ndarkModeSwitchTitle = \"Go dark\"\n\
         [[sidebar]]\ntext = \"Plain\"\nitems = [{ text = \"A\", link = \"/guide/a\" }]\n\
         [[sidebar]]\ntext = \"Folded\"\ncollapsed = true\nitems = [{ text = \"B\", link = \"/guide/b\" }]\n",
    );
    write(&site, "content/index.md", "# Home\n");
    write(&site, "content/guide/a.md", "# A\n");
    write(&site, "content/guide/b.md", "# B\n");
    let out = site.join("public");
    Site::load(&site).unwrap().build(&site, &out).unwrap();
    let html = std::fs::read_to_string(out.join("guide/a/index.html")).unwrap();
    let _ = std::fs::remove_dir_all(&site);
    html
}

#[test]
fn switch_carries_both_titles() {
    let html = build();
    let switches = html.matches("class=\"VPSwitch VPSwitchAppearance").count();
    assert_eq!(switches, 2, "navbar + nav screen");
    assert_eq!(
        html.matches("data-title-light=\"Go light\"").count(),
        2,
        "{html}"
    );
    assert_eq!(
        html.matches("data-title-dark=\"Go dark\"").count(),
        2,
        "{html}"
    );
}

#[test]
fn switch_icons_keep_their_12px_size() {
    // the icon helper's default `size-[1em]` sorts after `size-3` in the
    // stylesheet; a caller-supplied size must be the only one emitted
    let html = build();
    let sized: Vec<&str> = html
        .match_indices("<svg")
        .map(|(i, _)| &html[i..i + 400])
        .filter(|svg| svg.contains("size-3"))
        .collect();
    assert_eq!(sized.len(), 4, "sun + moon in navbar and nav screen");
    for svg in sized {
        assert!(
            !svg.contains("size-[1em]"),
            "competing default size in {svg}"
        );
    }
}

#[test]
fn sidebar_sections_bind_alpine_only_when_collapsible() {
    let html = build();
    assert!(
        !html.contains(":class=\"\""),
        "empty :class binding in {html}"
    );
    assert!(!html.contains("x-data=\"\""), "empty x-data in {html}");
    let bound = html.matches(":class=\"{ 'collapsed': !open }\"").count();
    assert_eq!(bound, 1, "only the collapsible group is bound: {html}");
}
