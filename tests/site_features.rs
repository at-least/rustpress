//! Feature-level integration tests: assemble a real site from an inline
//! config + content (Site::load → build) and assert the rendered pages.
//! The fixture-corpus tests live in site_build.rs; this file covers
//! config/frontmatter features the fixtures don't exercise.

use std::path::Path;

use gen_docs::render::Site;

// Minimal tempdir (dev-dependency-free), mirroring site_build.rs.
mod tempdir {
    use std::path::{Path, PathBuf};

    pub struct Guard(PathBuf);

    impl Guard {
        pub fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    pub fn tempdir() -> Guard {
        let dir = std::env::temp_dir().join(format!(
            "gen-docs-feat-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Guard(dir)
    }
}

fn build_site(config: &str, files: &[(&str, &str)]) -> (tempdir::Guard, tempdir::Guard) {
    let site_dir = tempdir::tempdir();
    std::fs::write(site_dir.path().join("gen-docs.toml"), config).unwrap();
    for (rel, body) in files {
        let path = site_dir.path().join("content").join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, body).unwrap();
    }
    let site = Site::load(site_dir.path()).unwrap();
    let out = tempdir::tempdir();
    site.build(site_dir.path(), out.path()).unwrap();
    (site_dir, out)
}

fn page(out: &tempdir::Guard, url: &str) -> String {
    let path = if url == "/" {
        out.path().join("index.html")
    } else {
        out.path().join(url.trim_matches('/')).join("index.html")
    };
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {url}: {e}"))
}

#[test]
fn hero_action_target_and_rel_are_emitted() {
    let (_, out) = build_site(
        "title = \"T\"\n",
        &[(
            "index.md",
            "---\nlayout: home\nhero:\n  name: N\n  text: T\n  actions:\n    - theme: brand\n      text: Start\n      link: https://example.com/start\n      target: _blank\n      rel: noopener\nfeatures: []\n---\n",
        )],
    );
    let html = page(&out, "/");
    assert!(
        html.contains(r#"href="https://example.com/start" target="_blank" rel="noopener""#),
        "hero action attrs: {}",
        &html[html.find("Start").map(|i| i.saturating_sub(200)).unwrap_or(0)..]
    );
}

#[test]
fn feature_link_wraps_card_with_target_rel() {
    let (_, out) = build_site(
        "title = \"T\"\n",
        &[(
            "index.md",
            "---\nlayout: home\nhero: {name: N, text: T}\nfeatures:\n  - title: F\n    details: D\n    link: /guide/\n    linkText: More\n    target: _self\n    rel: help\n---\n",
        )],
    );
    let html = page(&out, "/");
    assert!(html.contains("<a class=\"block border"), "card wrapped in anchor");
    assert!(html.contains(r#"href="/guide/" target="_self" rel="help""#), "feature attrs");
}

#[test]
fn return_to_top_label_is_configurable() {
    let (_, out) = build_site(
        "title = \"T\"\nreturnToTopLabel = \"Nach oben\"\n",
        &[("guide/a.md", "# A\n\ntext\n")],
    );
    assert!(page(&out, "/guide/a/").contains(">Nach oben</a>"), "local nav label");
}

#[test]
fn locale_title_used_in_title_and_navbar() {
    let (_, out) = build_site(
        "title = \"Base\"\n\n[locales.root]\nlabel = \"English\"\n\n[locales.zh]\nlabel = \"中文\"\ntitle = \"基地\"",
        &[
            ("guide/a.md", "# A\n"),
            ("zh/guide/a.md", "# 甲\n"),
        ],
    );
    let root = page(&out, "/guide/a/");
    assert!(root.contains("<title>A | Base</title>"), "root uses site title");
    let zh = page(&out, "/zh/guide/a/");
    assert!(zh.contains("<title>甲 | 基地</title>"), "zh page title uses locale title");
    assert!(zh.contains("<span>基地</span>"), "navbar shows locale title");
}
