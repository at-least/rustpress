//! Feature-level integration tests: assemble a real site from an inline
//! config + content (Site::load → build) and assert the rendered pages.
//! The fixture-corpus tests live in site_build.rs; this file covers
//! config/frontmatter features the fixtures don't exercise.

use rustpress::render::Site;

mod common;
use common::tempdir;

fn build_site(config: &str, files: &[(&str, &str)]) -> (common::TempDir, common::TempDir) {
    let site_dir = tempdir("site-features");
    std::fs::write(site_dir.path().join("rustpress.toml"), config).unwrap();
    for (rel, body) in files {
        let path = site_dir.path().join("content").join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, body).unwrap();
    }
    let site = Site::load(site_dir.path()).unwrap();
    let out = tempdir("site-features-out");
    site.build(site_dir.path(), out.path()).unwrap();
    (site_dir, out)
}

fn page(out: &common::TempDir, url: &str) -> String {
    let path = if url == "/" {
        out.path().join("index.html")
    } else {
        out.path().join(url.trim_matches('/')).join("index.html")
    };
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {url}: {e}"))
}

#[test]
fn relative_includes_resolve_from_the_content_root_without_a_root_index() {
    // content_dir() recovered the content root from the first page's
    // src parent — wrong whenever no content/index.md exists (the
    // URL-sorted first page is nested), so every relative and `@/`
    // include failed with a doubled path
    let (_, out) = build_site(
        "title = \"T\"\n",
        &[
            ("guide/only.md", "# Only\n\n<!--@include: ./part.md-->\n"),
            ("guide/part.md", "PART BODY\n"),
        ],
    );
    let html = page(&out, "/guide/only/");
    assert!(html.contains("PART BODY"), "sibling include expands");
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
        &html[html
            .find("Start")
            .map(|i| i.saturating_sub(200))
            .unwrap_or(0)..]
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
    assert!(
        html.contains("<a class=\"block border"),
        "card wrapped in anchor"
    );
    assert!(
        html.contains(r#"href="/guide/" target="_self" rel="help""#),
        "feature attrs"
    );
}

#[test]
fn return_to_top_label_is_configurable() {
    let (_, out) = build_site(
        "title = \"T\"\nreturnToTopLabel = \"Nach oben\"\n",
        &[("guide/a.md", "# A\n\ntext\n")],
    );
    assert!(
        page(&out, "/guide/a/").contains(">Nach oben</a>"),
        "local nav label"
    );
}

#[test]
fn locale_title_used_in_title_and_navbar() {
    let (_, out) = build_site(
        "title = \"Base\"\n\n[locales.root]\nlabel = \"English\"\n\n[locales.zh]\nlabel = \"中文\"\ntitle = \"基地\"",
        &[("guide/a.md", "# A\n"), ("zh/guide/a.md", "# 甲\n")],
    );
    let root = page(&out, "/guide/a/");
    assert!(
        root.contains("<title>A | Base</title>"),
        "root uses site title"
    );
    let zh = page(&out, "/zh/guide/a/");
    assert!(
        zh.contains("<title>甲 | 基地</title>"),
        "zh page title uses locale title"
    );
    assert!(
        zh.contains("<span>基地</span>"),
        "navbar shows locale title"
    );
}

#[test]
fn search_index_is_per_locale() {
    // upstream's local search builds one index per locale: a zh page
    // must not search (or fetch) the English pages, and vice versa
    let (_, out) = build_site(
        "title = \"T\"\n\n[search]\nprovider = \"local\"\n\n[locales.root]\nlabel = \"English\"\n\n[locales.zh]\nlabel = \"中文\"\n\n[locales.ja]\nlabel = \"日本語\"\n",
        &[
            ("guide/a.md", "# A\n"),
            ("zh/guide/a.md", "# 甲\n"),
            // a locale whose only page opts out still gets an (empty)
            // index, so its search button never fetches a missing file
            ("ja/guide/a.md", "---\nsearch: false\n---\n\n# あ\n"),
        ],
    );
    assert_eq!(urls_in(&out, "search-docs.json"), ["/guide/a/"]);
    assert_eq!(urls_in(&out, "zh/search-docs.json"), ["/zh/guide/a/"]);
    assert!(urls_in(&out, "ja/search-docs.json").is_empty());
    // each page's modal fetches its own locale's index
    assert!(page(&out, "/guide/a/").contains("data-index-url=\"/search-docs.json\""));
    assert!(page(&out, "/zh/guide/a/").contains("data-index-url=\"/zh/search-docs.json\""));
    assert!(page(&out, "/ja/guide/a/").contains("data-index-url=\"/ja/search-docs.json\""));

    // the 404 page searches the root index, which therefore exists even
    // when every page sits under a locale prefix
    let (_, out) = build_site(
        "title = \"T\"\n\n[search]\nprovider = \"local\"\n\n[locales.zh]\nlabel = \"中文\"\n",
        &[("zh/guide/a.md", "# 甲\n")],
    );
    assert!(urls_in(&out, "search-docs.json").is_empty());
    assert!(
        std::fs::read_to_string(out.path().join("404.html"))
            .unwrap()
            .contains("data-index-url=\"/search-docs.json\"")
    );
}

fn urls_in(out: &common::TempDir, rel: &str) -> Vec<String> {
    let json =
        std::fs::read_to_string(out.path().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let docs: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    docs.iter()
        .map(|d| d["url"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn site_title_setting_text_and_hide() {
    let (_, out) = build_site(
        "title = \"Base\"\nsiteTitle = \"Custom Brand\"\n",
        &[("guide/a.md", "# A\n")],
    );
    assert!(page(&out, "/guide/a/").contains("<span>Custom Brand</span>"));
    // <title> keeps the config title (upstream scopes siteTitle to the navbar)
    assert!(page(&out, "/guide/a/").contains("<title>A | Base</title>"));

    let (_, out) = build_site(
        "title = \"Base\"\nsiteTitle = false\n",
        &[("guide/a.md", "# A\n")],
    );
    assert!(
        !page(&out, "/guide/a/").contains("<span>Base</span>"),
        "title hidden"
    );
}

#[test]
fn logo_variants_and_outline_false() {
    let (_, out) = build_site(
        "title = \"T\"\n\n[logo]\nlight = \"/light.svg\"\ndark = \"/dark.svg\"\n\n[outline]\nlevel = 2\n",
        &[("guide/a.md", "# A\n\n## Sub\n\n### Deep\n")],
    );
    let html = page(&out, "/guide/a/");
    assert!(html.contains(
        r#"class="shrink-0 mr-0 h-(--vp-nav-logo-height) dark:hidden" src="/light.svg""#
    ));
    assert!(html.contains(
        r#"class="shrink-0 mr-0 h-(--vp-nav-logo-height) hidden dark:block" src="/dark.svg""#
    ));

    let (_, out) = build_site(
        "title = \"T\"\noutline = false\n",
        &[("guide/a.md", "# A\n\n## Sub\n")],
    );
    let html = page(&out, "/guide/a/");
    assert!(!html.contains("VPOutlineMarker"), "no outline marker");
    assert!(
        !html.contains("VPLocalNavOutlineDropdown"),
        "no local-nav dropdown"
    );
}

#[test]
fn scheme_links_pass_through_nav_and_hero() {
    // only http/https/mailto used to pass through: tel: got rooted into
    // /tel:… (404 on the site) and mailto: hero actions too
    let (_, out) = build_site(
        "title = \"T\"\n\n[[nav]]\ntext = \"Call\"\nlink = \"tel:+15551234\"\n",
        &[(
            "index.md",
            "---\nlayout: home\nhero:\n  name: N\n  text: T\n  actions:\n    - text: Mail\n      link: mailto:hi@example.com\nfeatures: []\n---\n",
        )],
    );
    let html = page(&out, "/");
    assert!(html.contains(r#"href="tel:+15551234""#), "nav tel: {html}");
    assert!(
        html.contains(r#"href="mailto:hi@example.com""#),
        "hero mailto:"
    );
}

#[test]
fn locale_switcher_prefix_strip_is_segment_safe() {
    // with the canonical "en/:rest*" = ":rest*" rewrite, a page named
    // english.md lands at /english/ while its locale base is /en — a
    // naive strip_prefix produced rest "glish/" and the switcher fell
    // back to the locale root instead of the /zh/english/ twin
    let dir = tempdir("locales");
    std::fs::write(
        dir.path().join("rustpress.toml"),
        "title = \"T\"\n\n[locales.en]\nlabel = \"English\"\nlang = \"en\"\n\n[locales.zh]\nlabel = \"简体中文\"\nlang = \"zh-CN\"\n\n[rewrites]\n\"en/:rest*\" = \":rest*\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("content/en")).unwrap();
    std::fs::create_dir_all(dir.path().join("content/zh")).unwrap();
    std::fs::write(dir.path().join("content/en/english.md"), "# E\n").unwrap();
    std::fs::write(dir.path().join("content/zh/english.md"), "# Z\n").unwrap();
    let site = Site::load(dir.path()).unwrap();
    let page = site.content.get("/english/").unwrap();
    assert_eq!(page.locale, "en");
    let tr = site.translations_for(page);
    let zh = tr
        .iter()
        .find(|(label, _, _)| label == "简体中文")
        .expect("zh entry");
    assert_eq!(zh.1, "/zh/english/", "twin found across locales");
}

#[test]
fn rest_rewrite_prefix_matches_on_segment_boundaries() {
    // a `:rest*` prefix captured across the segment boundary:
    // "guide:rest*" also rewrote guide2/… to docs/2/…
    let (_, out) = build_site(
        "title = \"T\"\n\n[rewrites]\n\"guide:rest*\" = \"docs/:rest*\"\n",
        &[("guide/x.md", "# X\n"), ("guide2/y.md", "# Y\n")],
    );
    assert!(
        out.path().join("docs/x/index.html").is_file(),
        "guide/ rewritten to docs/"
    );
    assert!(
        out.path().join("guide2/y/index.html").is_file(),
        "guide2 keeps its URL"
    );
    assert!(
        !out.path().join("docs/2/y/index.html").exists(),
        "no capture across the segment boundary"
    );
}

#[test]
fn rest_rewrite_boundary_slash_is_not_captured() {
    // "guide:rest*" matches guide/x.md at the segment boundary; the
    // boundary slash belongs to neither side, so "docs/:rest*" must give
    // /docs/x/, not /docs//x/. Path::join hides a double slash in the
    // written output, but page.url reaches the sidebar, sitemap, search
    // index and internal links verbatim, so assert on it directly
    let dir = tempdir("site-features");
    std::fs::write(
        dir.path().join("rustpress.toml"),
        "title = \"T\"\n\n[rewrites]\n\"guide:rest*\" = \"docs/:rest*\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("content/guide")).unwrap();
    std::fs::write(dir.path().join("content/guide/x.md"), "# X\n").unwrap();
    let site = Site::load(dir.path()).unwrap();
    let urls: Vec<&str> = site.content.pages.iter().map(|p| p.url.as_str()).collect();
    assert!(site.content.get("/docs/x/").is_some(), "urls: {urls:?}");
    assert!(!urls.iter().any(|u| u.contains("//")), "urls: {urls:?}");
}

#[test]
fn bare_rest_rewrite_captures_every_page() {
    // a bare `:rest*` has no prefix at all: the segment-boundary filter
    // must not turn it into a rule that matches nothing
    let (_, out) = build_site(
        "title = \"T\"\n\n[rewrites]\n\":rest*\" = \"docs/:rest*\"\n",
        &[("guide/x.md", "# X\n")],
    );
    assert!(
        out.path().join("docs/guide/x/index.html").is_file(),
        "whole path captured and re-rooted"
    );
}

#[test]
fn dead_link_checker_ignores_data_src_and_checks_spaced_equals() {
    // the href/src regex matched attribute-name suffixes (data-src=,
    // xlink:href=) and missed spaced equals, both valid raw HTML
    let dir = tempdir("dead-links");
    std::fs::write(dir.path().join("rustpress.toml"), "title = \"T\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("content")).unwrap();
    std::fs::write(
        dir.path().join("content/index.md"),
        "# H\n\n<img data-src=\"/no-such-image.png\">\n\n<a href = \"/also-gone/\">x</a>\n",
    )
    .unwrap();
    let site = Site::load(dir.path()).unwrap();
    let out = tempdir("dead-links-out");
    let err = site.build(dir.path(), out.path()).unwrap_err().to_string();
    assert!(
        err.contains("also-gone"),
        "href with spaced equals is checked: {err}"
    );
    assert!(
        !err.contains("no-such-image"),
        "data-src is not a link target: {err}"
    );
}

#[test]
fn frontmatter_outline_reenables_a_site_disabled_outline() {
    // the site gate used to return None unconditionally, so the
    // documented per-page override ("level can be overridden per page
    // via frontmatter") could never turn the outline back on
    let (_, out) = build_site(
        "title = \"T\"\noutline = false\n",
        &[(
            "guide/a.md",
            "---\noutline: deep\n---\n\n# A\n\n## Sub\n\n### Deep\n",
        )],
    );
    let html = page(&out, "/guide/a/");
    assert!(
        html.contains("VPOutlineMarker"),
        "front matter overrides the site-off gate"
    );
}

#[test]
fn nav_and_sidebar_link_attrs_with_doc_footer_text() {
    let (_, out) = build_site(
        r#"
title = "T"

[[nav]]
text = "External"
link = "https://example.com"
target = "_blank"
rel = "noopener"

[[sidebar]]
text = "Guide"

  [[sidebar.items]]
  text = "A"
  link = "/guide/a/"
  docFooterText = "A — custom pager title"

  [[sidebar.items]]
  text = "B"
  link = "/guide/b/"
"#,
        &[("guide/a.md", "# A\n"), ("guide/b.md", "# B\n")],
    );
    let html = page(&out, "/guide/a/");
    assert!(
        html.contains(r#"href="https://example.com" target="_blank" rel="noopener""#),
        "nav attrs"
    );
    let b = page(&out, "/guide/b/");
    assert!(
        b.contains("A — custom pager title"),
        "docFooterText in pager"
    );
}

#[test]
fn doc_footer_false_disables_pager_side() {
    let (_, out) = build_site(
        "title = \"T\"\n\n[docFooter]\nprev = false\n",
        &[("guide/a.md", "# A\n"), ("guide/b.md", "# B\n")],
    );
    let html = page(&out, "/guide/b/");
    assert!(!html.contains("Previous page"), "prev disabled");
    assert!(
        html.contains("Next page") || html.contains("<title>"),
        "next intact"
    );
}

#[test]
fn title_template_false_drops_suffix() {
    let (_, out) = build_site(
        "title = \"Base\"\ntitleTemplate = false\n",
        &[("guide/a.md", "# A\n")],
    );
    assert!(page(&out, "/guide/a/").contains("<title>A</title>"));
}

#[test]
fn src_exclude_globs_drop_pages() {
    let (_, out) = build_site(
        "title = \"T\"\nsrcExclude = [\"drafts/**\", \"secret.md\"]\n",
        &[
            ("guide/a.md", "# A\n"),
            ("drafts/x.md", "# Draft\n"),
            ("drafts/sub/y.md", "# Draft 2\n"),
            ("secret.md", "# Secret\n"),
        ],
    );
    assert!(out.path().join("guide/a/index.html").is_file());
    assert!(!out.path().join("drafts").exists(), "** excludes nested");
    assert!(
        !out.path().join("secret/index.html").exists(),
        "exact file excluded"
    );
}

#[test]
fn footer_allows_inline_html() {
    let (_, out) = build_site(
        r#"title = "T"

[footer]
message = "Released under <a href=\"/license/\">MIT</a>.""#,
        &[("about.md", "# About\n")],
    );
    assert!(page(&out, "/about/").contains("Released under <a href=\"/license/\">MIT</a>."));
}

#[test]
fn a11y_labels_are_configurable() {
    let (_, out) = build_site(
        "title = \"T\"\nnavMenuLabel = \"Hauptnavigation\"\nmobileMenuLabel = \"Menü\"\nsidebarMenuLabel = \"Seitenleiste\"\n",
        &[("guide/a.md", "# A\n")],
    );
    let html = page(&out, "/guide/a/");
    assert!(html.contains(r#"aria-label="Hauptnavigation""#));
    assert!(html.contains(r#"aria-label="Menü""#));
    assert!(html.contains("<span>Seitenleiste</span>"));
}

#[test]
fn frontmatter_page_toggles() {
    let (_, out) = build_site(
        "title = \"T\"\nlastUpdated = false\n\n[footer]\nmessage = \"m\"\ncopyright = \"c\"\n\n[editLink]\npattern = \"https://x/:path\"\n",
        &[
            ("plain.md", "# Plain\n"),
            (
                "bare.md",
                "---\nnavbar: false\nsidebar: false\nfooter: false\neditLink: false\npageClass: custom-page\n---\n\n# Bare\n",
            ),
        ],
    );
    // plain page keeps everything (auto sidebar exists since guide-less content
    // — a top-level page has no sidebar section, so use navbar/footer/edit)
    let plain = page(&out, "/plain/");
    assert!(plain.contains("VPNavBar"), "navbar on");
    assert!(
        plain.contains("Released") || plain.contains("VPFooter") || plain.contains("footer"),
        "footer on"
    );
    let bare = page(&out, "/bare/");
    assert!(!bare.contains("VPNavBar"), "navbar hidden");
    assert!(
        !bare.contains("id=\"VPFooter\"") && !bare.contains("<footer"),
        "footer hidden"
    );
    assert!(!bare.contains("Edit this page"), "edit link hidden");
    assert!(bare.contains("custom-page"), "pageClass applied");
}

#[test]
fn frontmatter_sidebar_false_hides_the_sidebar_element() {
    // `sidebar: false` must remove the fixed desktop aside entirely,
    // not just the content column's padding class — otherwise the
    // sidebar paints over the sidebar-less layout (VitePress hides it)
    let (_, out) = build_site(
        "title = \"T\"\n",
        &[
            ("guide/bare.md", "---\nsidebar: false\n---\n\n# Bare\n"),
            ("guide/normal.md", "# Normal\n"),
        ],
    );
    let bare = page(&out, "/guide/bare/");
    assert!(!bare.contains("id=\"VPSidebar\""), "sidebar element hidden");
    let normal = page(&out, "/guide/normal/");
    assert!(
        normal.contains("id=\"VPSidebar\""),
        "sidebar kept on normal pages"
    );
}

#[test]
fn frontmatter_last_updated_date_override() {
    let (_, out) = build_site(
        "title = \"T\"\n",
        &[
            ("a.md", "---\nlastUpdated: 2020-01-02\n---\n\n# A\n"),
            ("b.md", "# B\n"),
        ],
    );
    let a = page(&out, "/a/");
    assert!(a.contains("2020-01-02"), "date override shown");
    let b = page(&out, "/b/");
    assert!(
        !b.contains("Last updated"),
        "no timestamp without lastUpdated config"
    );
}

#[test]
fn frontmatter_aside_and_outline_false() {
    let (_, out) = build_site(
        "title = \"T\"\n",
        &[
            ("a.md", "---\naside: false\n---\n\n# A\n\n## S1\n"),
            ("b.md", "---\noutline: false\n---\n\n# B\n\n## S2\n"),
            ("c.md", "---\naside: left\n---\n\n# C\n\n## S3\n"),
            ("d.md", "# D\n\n## S4\n"),
        ],
    );
    assert!(
        !page(&out, "/a/").contains("VPOutlineMarker"),
        "aside:false hides outline column"
    );
    assert!(
        !page(&out, "/b/").contains("VPOutlineMarker"),
        "outline:false hides outline"
    );
    assert!(
        !page(&out, "/b/").contains("VPOutlineDropdownButton"),
        "outline:false hides the local-nav dropdown too"
    );
    assert!(
        page(&out, "/d/").contains("VPOutlineDropdownButton"),
        "default page keeps the local-nav dropdown"
    );
    assert!(
        page(&out, "/c/").contains("xl:order-1") && page(&out, "/c/").contains("xl:order-2"),
        "left aside orders swapped"
    );
    assert!(
        page(&out, "/d/").contains("VPOutlineMarker"),
        "default aside present"
    );
}

#[test]
fn frontmatter_prev_next_overrides() {
    let (_, out) = build_site(
        r#"title = "T"

[[sidebar]]
text = "S"

  [[sidebar.items]]
  text = "A"
  link = "/a/"

  [[sidebar.items]]
  text = "B"
  link = "/b/"
"#,
        &[
            ("a.md", "# A\n"),
            (
                "b.md",
                "---\nprev: Back to start\nnext:\n  text: External next\n  link: https://example.com/n\n---\n\n# B\n",
            ),
        ],
    );
    let b = page(&out, "/b/");
    assert!(b.contains("Back to start"), "prev text override");
    assert!(b.contains("External next"), "next object override");
    assert!(
        b.contains(r#"href="https://example.com/n""#),
        "next custom link"
    );
}

#[test]
fn frontmatter_search_false_and_head_and_title_template() {
    let (_, out) = build_site(
        "title = \"Base\"\n\n[search]\nprovider = \"local\"\n",
        &[
            (
                "a.md",
                "---\nsearch: false\ntitleTemplate: \":title!!\"\nhead:\n  - tag: meta\n    attrs:\n      name: \"x-page\"\n      content: \"yes\"\n---\n\n# A\n",
            ),
            ("b.md", "# B\n"),
        ],
    );
    let index = std::fs::read_to_string(out.path().join("search-docs.json")).unwrap();
    assert!(!index.contains("\"/a/\""), "page excluded from search");
    assert!(index.contains("\"/b/\""), "other page indexed");
    let a = page(&out, "/a/");
    assert!(a.contains("<title>A!!</title>"), "page titleTemplate wins");
    assert!(
        a.contains(r#"<meta content="yes" name="x-page"/>"#),
        "per-page head tag"
    );
}

#[test]
fn layout_page_strips_doc_chrome() {
    let (_, out) = build_site(
        "title = \"T\"\n\n[editLink]\npattern = \"https://x/:path\"\n",
        &[
            ("plain.md", "# P\n\n## Sub\n"),
            ("bare.md", "---\nlayout: page\n---\n\n# Bare\n\n## Sub\n"),
        ],
    );
    let bare = page(&out, "/bare/");
    assert!(!bare.contains("VPOutlineMarker"), "no outline");
    assert!(!bare.contains("Edit this page"), "no edit link");
    assert!(!bare.contains("Previous page"), "no pager");
    let plain = page(&out, "/plain/");
    assert!(
        plain.contains("VPOutlineMarker"),
        "doc layout keeps outline"
    );
}

#[test]
fn graded_containers_and_search_translations() {
    let (_, out) = build_site(
        r#"title = "T"
gradedContainers = true

[search]
provider = "local"

[search.translations]
buttonText = "Suchen"
placeholder = "Dokumente durchsuchen"
noResultsText = "Nichts gefunden für {q}"
navigateText = "zum Navigieren""#,
        &[("guide/a.md", "# A\n")],
    );
    let html = page(&out, "/guide/a/");
    assert!(html.contains("vp-graded-containers"), "graded body class");
    assert!(
        html.contains("<span class=\"hidden md:inline md:text-[0.8125rem]\">Suchen</span>"),
        "button text"
    );
    assert!(
        html.contains("placeholder=\"Dokumente durchsuchen\""),
        "placeholder"
    );
    assert!(
        html.contains("data-no-results=\"Nichts gefunden für {q}\""),
        "no-results string"
    );
    assert!(html.contains(">zum Navigieren</span>"), "footer hint");
}

#[test]
fn edit_link_paths_are_percent_encoded() {
    // a page name with a space (or non-ASCII bytes) must reach the edit
    // URL percent-encoded — a raw space in the href is invalid
    let (_, out) = build_site(
        "title = \"T\"\n\n[editLink]\npattern = \"https://x/edit/:path\"\n",
        &[("guide/a b.md", "# A\n"), ("guide/指南.md", "# ZH\n")],
    );
    let space = page(&out, "/guide/a b/");
    assert!(
        space.contains("https://x/edit/guide/a%20b.md"),
        "space encoded: {}",
        space[space
            .find("edit-link")
            .map(|i| i.saturating_sub(40))
            .unwrap_or(0)..]
            .chars()
            .take(220)
            .collect::<String>()
    );
    assert!(!space.contains("edit/guide/a b.md"), "raw space leaked");
    let zh = page(&out, "/guide/指南/");
    assert!(
        zh.contains("https://x/edit/guide/%E6%8C%87%E5%8D%97.md"),
        "utf-8 bytes encoded"
    );
}

#[test]
fn nav_and_sidebar_link_urls_are_attribute_escaped() {
    // config link strings are interpolated into hand-built href
    // attributes; a quote in them must not break out of the attribute
    let (_, out) = build_site(
        "title = \"T\"\n\n[[nav]]\ntext = \"Quoted\"\nlink = '/a\"b/'\n\n[[sidebar]]\ntext = \"S\"\nlink = '/s\"x/'\n",
        &[("index.md", "# H\n")],
    );
    let home = page(&out, "/");
    assert!(
        home.contains("href=\"/a&quot;b/\""),
        "escaped nav href: {}",
        home
    );
    assert!(
        !home.contains("href=\"/a\"b/\""),
        "raw quote must not reach the nav href"
    );
    assert!(
        home.contains("href=\"/s&quot;x/\""),
        "escaped sidebar href: {}",
        home
    );
}

#[test]
fn childless_non_void_head_tags_do_not_self_close() {
    // <title/> is an unclosed element to the HTML5 parser; only void
    // elements may self-close, everything else gets an explicit end tag
    let (_, out) = build_site(
        "title = \"T\"\n\n[[head]]\ntag = \"title\"\n\n[[head]]\ntag = \"link\"\nattrs = { href = \"/a.css\", rel = \"stylesheet\" }\n",
        &[("index.md", "# H\n")],
    );
    let home = page(&out, "/");
    assert!(
        home.contains("<title></title>"),
        "non-void tag gets an end tag"
    );
    assert!(!home.contains("<title/>"), "no self-closed title");
    assert!(home.contains("<link"), "void tag still emitted");
}

#[test]
fn mathjax_head_emits_only_elements_and_escaped_delimiters() {
    // the snippet is dropped raw into <head>: JS `//` comments between
    // the tags would parse as visible body text, and an unescaped
    // `"\("` in a JS string degrades to `"("` — turning bare
    // parentheses into math delimiters on every math page
    let (_, out) = build_site(
        "title = \"T\"\n[markdown]\nmath = true\n",
        &[("index.md", "# H\n\n$x$\n")],
    );
    let home = page(&out, "/");
    assert!(
        home.contains("</script><script defer src=\"https://cdn.jsdelivr.net"),
        "config script and CDN script are adjacent, no raw text between"
    );
    assert!(
        home.contains(r#"["\\(", "\\)"]"#),
        "inline delimiters reach MathJax as \\( \\)"
    );
    assert!(
        home.contains(r#"["\\[", "\\]"]"#),
        "display delimiters reach MathJax as \\[ \\]"
    );
}

#[test]
fn mathjax_is_pinned_with_integrity() {
    // the CDN script executes on every math page: a floating version
    // tag plus a missing integrity attribute meant a CDN compromise or
    // surprise upgrade shipped straight into built sites
    let (_, out) = build_site(
        "title = \"T\"\n[markdown]\nmath = true\n",
        &[("index.md", "# H\n\n$x$\n")],
    );
    let home = page(&out, "/");
    assert!(home.contains("mathjax@3.2.2/"), "version pinned");
    assert!(
        home.contains("integrity=\"sha384-"),
        "integrity attribute present"
    );
    assert!(
        home.contains("crossorigin=\"anonymous\""),
        "crossorigin set"
    );
}
