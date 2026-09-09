//! Full-site build integration test: assemble a Site over the fixture
//! corpus, build into a temp dir, and check the outputs.

use std::path::Path;

use gen_docs::config::SiteConfig;
use gen_docs::content::Content;
use gen_docs::markdown::MarkdownEngine;
use gen_docs::render::Site;
use gen_docs::sidebar::Sidebars;

fn build_fixture() -> (tempdir::Guard, gen_docs::render::BuildStats) {
    let fixtures = Path::new("tests/fixtures");
    let config: SiteConfig = toml::from_str("title = \"Fixture\"\n\n[search]\nprovider = \"local\"\n").unwrap();
    let content = Content::load(&fixtures.join("en")).unwrap();
    let site = Site {
        sidebars: Sidebars::build(&config, &content),
        engine: MarkdownEngine::new(&config.markdown).unwrap(),
        config,
        content,
    };
    let out = tempdir::tempdir();
    let stats = site.build(fixtures, out.path()).unwrap();
    (out, stats)
}

// Minimal tempdir (dev-dependency-free).
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
            "gen-docs-test-{}-{}",
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

#[test]
fn builds_pages_404_syntax_and_search_docs() {
    let (out, stats) = build_fixture();
    let root = out.path();
    assert_eq!(stats.pages, 8);
    for page in [
        "index.html",
        "404.html",
        "syntax.css",
        "search-docs.json",
        "guide/getting-started/index.html",
        "guide/markdown/index.html",
        "reference/runtime-api/index.html",
    ] {
        assert!(root.join(page).is_file(), "missing {page}");
    }
    let css = std::fs::read_to_string(root.join("syntax.css")).unwrap();
    assert!(css.contains("html.dark .st-"));
    let docs: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("search-docs.json")).unwrap()).unwrap();
    let entries = docs.as_array().unwrap();
    assert_eq!(entries.len(), 8);
    let home = entries.iter().find(|e| e["url"] == "/").unwrap();
    assert_eq!(home["title"], "Fixture");
    let getting_started = entries
        .iter()
        .find(|e| e["url"] == "/guide/getting-started/")
        .unwrap();
    assert_eq!(getting_started["title"], "Getting Started");
    assert!(getting_started["body"].as_str().unwrap().contains("npm"));
}

#[test]
fn built_pages_carry_alpine_and_landmarks() {
    let (out, _) = build_fixture();
    let html = std::fs::read_to_string(out.path().join("guide/getting-started/index.html")).unwrap();
    for probe in [
        "x-data=\"docPage\"",
        "x-data=\"searchModal\"",
        "data-index-url=\"/search-docs.json\"",
        "x-data=\"codeGroup\"",
        "<div class=\"tabs\">",
        "vp-copy-button",
        "$store.ui.screen",
        "aria-current=\"page\"",
    ] {
        assert!(html.contains(probe), "missing {probe}");
    }
}
