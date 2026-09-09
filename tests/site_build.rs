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
    // the fixture subset references pages that were not copied — the
    // dead-link checker would fail the build otherwise
    let config: SiteConfig = toml::from_str(
        "title = \"Fixture\"\nignoreDeadLinks = true\n\n[search]\nprovider = \"local\"\n",
    )
    .unwrap();
    let content = Content::load(&fixtures.join("en")).unwrap();
    let site = Site {
        sidebars: Sidebars::build(&config, &content),
        engine: MarkdownEngine::new(&config.markdown, &config.syntax).unwrap(),
        theme_css: false,
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

#[test]
fn dead_links_fail_the_build_and_ignore_works() {
    let fixtures = Path::new("tests/fixtures");
    let mk = |ignore: &str| {
        let config: SiteConfig = toml::from_str(&format!(
            "title = \"T\"\nignoreDeadLinks = {}\n\n[search]\nprovider = \"local\"\n",
            ignore
        ))
        .unwrap();
        let content = Content::load(&fixtures.join("en")).unwrap();
        Site {
            sidebars: Sidebars::build(&config, &content),
            engine: MarkdownEngine::new(&config.markdown, &config.syntax).unwrap(),
        theme_css: false,
            config,
            content,
        }
    };
    // routing.md links ./deploy — outside the subset
    let out = tempdir::tempdir();
    let err = mk("false").build(fixtures, out.path()).unwrap_err();
    assert!(err.to_string().contains("dead link"), "{err}");
    assert!(err.to_string().contains("guide/deploy"));

    // ignore all
    let out2 = tempdir::tempdir();
    assert!(mk("true").build(fixtures, out2.path()).is_ok());

    // ignore a single prefix: everything else still fails
    let config: SiteConfig = toml::from_str(
        "title = \"T\"\nignoreDeadLinks = [\"guide/deploy\"]\n\n[search]\nprovider = \"local\"\n",
    )
    .unwrap();
    let content = Content::load(&fixtures.join("en")).unwrap();
    let site = Site {
        sidebars: Sidebars::build(&config, &content),
        engine: MarkdownEngine::new(&config.markdown, &config.syntax).unwrap(),
        theme_css: false,
        config,
        content,
    };
    let out3 = tempdir::tempdir();
    let err = site.build(fixtures, out3.path()).unwrap_err();
    assert!(!err.to_string().contains("guide/deploy"), "ignored prefix filtered: {err}");
    assert!(err.to_string().contains("guide/custom-theme"), "others still reported");
}

#[test]
fn locales_and_rewrites_end_to_end() {
    // temp site: content/en/** + content/zh/**, rewrite en/:rest* → :rest*,
    // locales root(en) + zh — the canonical VitePress i18n layout
    let site_dir = tempdir::tempdir();
    let content = site_dir.path().join("content");
    std::fs::create_dir_all(content.join("en/guide")).unwrap();
    std::fs::create_dir_all(content.join("zh/guide")).unwrap();
    let page_body = "---\ndescription: d\n---\n\n# Page\n\nhello\n";
    std::fs::write(content.join("en/guide/page.md"), page_body).unwrap();
    std::fs::write(content.join("zh/guide/page.md"), "---\ndescription: 目录\n---\n\n# 页面\n\n内容\n").unwrap();
    std::fs::write(
        site_dir.path().join("gen-docs.toml"),
        r#"
title = "Docs"

[locales.root]
label = "English"
lang = "en"

[locales.zh]
label = "简体中文"
lang = "zh-CN"

[rewrites]
"en/:rest*" = ":rest*"

[search]
provider = "local"
"#,
    )
    .unwrap();

    let site = Site::load(site_dir.path()).unwrap();
    // rewritten URLs: en/ promoted to root, zh stays under /zh/
    assert!(site.content.get("/guide/page/").is_some(), "en rewritten to root");
    assert_eq!(site.content.get("/guide/page/").unwrap().locale, "root");
    assert!(site.content.get("/zh/guide/page/").is_some(), "zh at /zh/");
    assert_eq!(site.content.get("/zh/guide/page/").unwrap().locale, "zh");

    // switcher: en page links to its zh twin and vice versa
    let en_page = site.content.get("/guide/page/").unwrap();
    let tr = site.translations_for(en_page);
    assert_eq!(tr.len(), 2);
    let zh_entry = tr.iter().find(|(label, _, _)| label == "简体中文").unwrap();
    assert_eq!(zh_entry.1, "/zh/guide/page/");

    // build and check rendered html lang + switcher markup
    let out = tempdir::tempdir();
    site.build(site_dir.path(), out.path()).unwrap();
    let en_html = std::fs::read_to_string(out.path().join("guide/page/index.html")).unwrap();
    assert!(en_html.contains(r#"<html lang="en""#), "en lang attr");
    assert!(en_html.contains("Change language"), "flyout present");
    assert!(en_html.contains("href=\"/zh/guide/page/\""), "cross-locale link");
    let zh_html = std::fs::read_to_string(out.path().join("zh/guide/page/index.html")).unwrap();
    assert!(zh_html.contains(r#"<html lang="zh-CN""#), "zh lang attr");
    assert!(zh_html.contains(r#"<title>页面"#), "localized title");
}

#[test]
fn theme_css_is_copied_and_linked_when_present() {
    // a site WITH theme.css: copied verbatim + linked from every page
    let site_dir = tempdir::tempdir();
    std::fs::create_dir_all(site_dir.path().join("content")).unwrap();
    std::fs::write(site_dir.path().join("content/index.md"), "# Home\n").unwrap();
    std::fs::write(site_dir.path().join("gen-docs.toml"), "title = \"T\"\n").unwrap();
    std::fs::write(
        site_dir.path().join("theme.css"),
        ":root { --vp-c-brand-1: #508d3f; }\n.dark { --vp-c-brand-1: #83aa63; }\n",
    )
    .unwrap();

    let mut site = Site::load(site_dir.path()).unwrap();
    site.theme_css = true;
    let out = tempdir::tempdir();
    site.build(site_dir.path(), out.path()).unwrap();

    let css = std::fs::read_to_string(out.path().join("theme.css")).unwrap();
    assert!(css.contains("--vp-c-brand-1: #508d3f;"), "copied verbatim: {css}");
    let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
    assert!(html.contains(r#"<link rel="stylesheet" href="/theme.css">"#), "linked");

    // a site WITHOUT theme.css: no copy, no link
    let site_dir2 = tempdir::tempdir();
    std::fs::create_dir_all(site_dir2.path().join("content")).unwrap();
    std::fs::write(site_dir2.path().join("content/index.md"), "# Home\n").unwrap();
    std::fs::write(site_dir2.path().join("gen-docs.toml"), "title = \"T\"\n").unwrap();
    let site2 = Site::load(site_dir2.path()).unwrap();
    let out2 = tempdir::tempdir();
    site2.build(site_dir2.path(), out2.path()).unwrap();
    assert!(!out2.path().join("theme.css").exists());
    let html2 = std::fs::read_to_string(out2.path().join("index.html")).unwrap();
    assert!(!html2.contains("theme.css"), "no link without theme.css");
}

#[test]
fn syntax_theme_pair_is_selectable() {
    // the [syntax] section picks the syntax color scheme independently of
    // the UI palette; unknown names fail at engine construction
    let mk = |dark: &str| {
        let engine = MarkdownEngine::new(
            &gen_docs::config::Markdown::default(),
            &gen_docs::config::SyntaxThemes {
                light: "github-light".into(),
                dark: dark.into(),
            },
        );
        engine.map(|e| e.syntax_css())
    };
    assert!(mk("base16-ocean.dark").is_ok(), "syntect bundled theme by name");
    let err = mk("no-such-theme").unwrap_err();
    assert!(err.to_string().contains("no-such-theme"), "{err}");
    let css = mk("base16-ocean.dark").unwrap();
    assert!(css.contains("html.dark .st-"), "dark still scoped");
}
