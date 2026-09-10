//! Integration test against a committed subset of the real
//! vitepress.dev docs (`tests/fixtures/en`, copied verbatim from a clone
//! of vuejs/vitepress) — so the loader is exercised against genuine
//! VitePress-format content, not hand-written samples.

use gen_docs::config::SiteConfig;
use gen_docs::content::{Content, FeatureIcon, OutlineSetting};
use gen_docs::sidebar::Sidebars;

fn fixtures() -> Content {
    Content::load(std::path::Path::new("tests/fixtures/en"), &[]).expect("fixture content loads")
}

#[test]
fn loads_all_fixture_pages_with_expected_urls() {
    let content = fixtures();
    let urls: Vec<&str> = content.pages.iter().map(|p| p.url.as_str()).collect();
    assert_eq!(
        urls,
        vec![
            "/",
            "/guide/asset-handling/",
            "/guide/getting-started/",
            "/guide/markdown/",
            "/guide/routing/",
            "/guide/what-is-vitepress/",
            "/reference/runtime-api/",
            "/reference/site-config/",
        ]
    );
}

#[test]
fn titles_come_from_h1_not_filenames() {
    let content = fixtures();
    assert_eq!(content.get("/guide/what-is-vitepress/").unwrap().title, "What is VitePress?");
    assert_eq!(content.get("/guide/getting-started/").unwrap().title, "Getting Started");
    assert_eq!(content.get("/reference/runtime-api/").unwrap().title, "Runtime API");
    // site-config.md documents frontmatter in fenced examples; its real H1
    // precedes any fence
    assert_eq!(content.get("/reference/site-config/").unwrap().title, "Site Config");
}

#[test]
fn home_page_front_matter() {
    let content = fixtures();
    let home = content.get("/").unwrap();
    assert!(home.is_home());
    let hero = home.front.hero.as_ref().expect("hero");
    assert_eq!(hero.name.as_deref(), Some("VitePress"));
    assert_eq!(
        hero.text.as_deref(),
        Some("Vite & Vue Powered Static Site Generator")
    );
    assert!(hero.actions.iter().any(|a| a.link == "./guide/what-is-vitepress"));
    assert!(home.front.features.iter().any(|f| f.title == "Focus on your content"));
    assert!(matches!(
        &home.front.features[0].icon,
        Some(FeatureIcon::Text(t)) if t.starts_with("<span")
    ));
}

#[test]
fn outline_deep_front_matter_survives() {
    let content = fixtures();
    // routing.md / site-config.md / markdown.md use `outline: deep`
    let page = content.get("/guide/routing/").unwrap();
    assert_eq!(page.front.outline, Some(OutlineSetting::Deep));
    // pages without it carry None
    let plain = content.get("/guide/what-is-vitepress/").unwrap();
    assert_eq!(plain.front.outline, None);
}

#[test]
fn auto_sidebar_two_trees_in_natural_order() {
    let content = fixtures();
    let sb = Sidebars::build(&SiteConfig::default(), &content);
    assert_eq!(sb.trees.len(), 2);
    let guide = sb.for_url("/guide/routing/").expect("guide sidebar");
    assert_eq!(guide.prefix, "/guide/");
    let group = &guide.items[0];
    // no guide/index.md in the fixture → group titled by section, no link
    assert_eq!(group.text, "guide");
    assert_eq!(group.url, None);
    let child_titles: Vec<&str> = group.children.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(
        child_titles,
        vec![
            "Asset Handling",
            "Getting Started",
            "Markdown Extensions",
            "Routing",
            "What is VitePress?",
        ]
    );
    let (prev, next) = sb.neighbors("/guide/getting-started/");
    assert_eq!(prev.as_ref().map(|(t, u)| (t.as_str(), u.as_str())), Some(("Asset Handling", "/guide/asset-handling/")));
    assert_eq!(next.as_ref().map(|(t, u)| (t.as_str(), u.as_str())), Some(("Markdown Extensions", "/guide/markdown/")));
}

#[test]
fn explicit_vitepress_style_sidebar_resolves_base_chains() {
    // Mirrors a slice of vitepress.dev's own config: the Default Theme
    // group uses base '/reference/default-theme-' with relative links.
    let content = fixtures();
    let config: SiteConfig = toml::from_str(
        r#"
[sidebar."/guide/"]

  [[sidebar."/guide/".items]]
  text = "Guide"

    [[sidebar."/guide/".items.items]]
    text = "What is VitePress?"
    link = "/guide/what-is-vitepress/"

    [[sidebar."/guide/".items.items]]
    text = "Getting Started"
    link = "/guide/getting-started/"

[sidebar."/reference/"]

  [[sidebar."/reference/".items]]
  text = "Reference"
"#,
    )
    .unwrap();
    let sb = Sidebars::build(&config, &content);
    let guide = sb.for_url("/guide/markdown/").unwrap();
    assert_eq!(guide.items[0].children.len(), 2);
    assert_eq!(
        guide.items[0].children[1].url.as_deref(),
        Some("/guide/getting-started/")
    );
    // prev/next only within configured leaves
    let (prev, next) = sb.neighbors("/guide/getting-started/");
    assert_eq!(
        prev.as_ref().map(|(_, u)| u.as_str()),
        Some("/guide/what-is-vitepress/")
    );
    assert_eq!(next, None);
}
