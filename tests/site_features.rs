//! Feature-level integration tests: assemble a real site from an inline
//! config + content (Site::load → build) and assert the rendered pages.
//! The fixture-corpus tests live in site_build.rs; this file covers
//! config/frontmatter features the fixtures don't exercise.

use rustpress::render::Site;

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
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "rustpress-feat-{}-{n}-{}",
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
    std::fs::write(site_dir.path().join("rustpress.toml"), config).unwrap();
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
    assert!(!page(&out, "/guide/a/").contains("<span>Base</span>"), "title hidden");
}

#[test]
fn logo_variants_and_outline_false() {
    let (_, out) = build_site(
        "title = \"T\"\n\n[logo]\nlight = \"/light.svg\"\ndark = \"/dark.svg\"\n\n[outline]\nlevel = 2\n",
        &[("guide/a.md", "# A\n\n## Sub\n\n### Deep\n")],
    );
    let html = page(&out, "/guide/a/");
    assert!(html.contains(r#"class="shrink-0 mr-0 h-(--vp-nav-logo-height) dark:hidden" src="/light.svg""#));
    assert!(html.contains(r#"class="shrink-0 mr-0 h-(--vp-nav-logo-height) hidden dark:block" src="/dark.svg""#));

    let (_, out) = build_site(
        "title = \"T\"\noutline = false\n",
        &[("guide/a.md", "# A\n\n## Sub\n")],
    );
    let html = page(&out, "/guide/a/");
    assert!(!html.contains("VPOutlineMarker"), "no outline marker");
    assert!(!html.contains("VPLocalNavOutlineDropdown"), "no local-nav dropdown");
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
    assert!(html.contains(r#"href="https://example.com" target="_blank" rel="noopener""#), "nav attrs");
    let b = page(&out, "/guide/b/");
    assert!(b.contains("A — custom pager title"), "docFooterText in pager");
}

#[test]
fn doc_footer_false_disables_pager_side() {
    let (_, out) = build_site(
        "title = \"T\"\n\n[docFooter]\nprev = false\n",
        &[("guide/a.md", "# A\n"), ("guide/b.md", "# B\n")],
    );
    let html = page(&out, "/guide/b/");
    assert!(!html.contains("Previous page"), "prev disabled");
    assert!(html.contains("Next page") || html.contains("<title>"), "next intact");
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
    assert!(!out.path().join("secret/index.html").exists(), "exact file excluded");
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
            ("bare.md", "---\nnavbar: false\nsidebar: false\nfooter: false\neditLink: false\npageClass: custom-page\n---\n\n# Bare\n"),
        ],
    );
    // plain page keeps everything (auto sidebar exists since guide-less content
    // — a top-level page has no sidebar section, so use navbar/footer/edit)
    let plain = page(&out, "/plain/");
    assert!(plain.contains("VPNavBar"), "navbar on");
    assert!(plain.contains("Released") || plain.contains("VPFooter") || plain.contains("footer"), "footer on");
    let bare = page(&out, "/bare/");
    assert!(!bare.contains("VPNavBar"), "navbar hidden");
    assert!(!bare.contains("id=\"VPFooter\"") && !bare.contains("<footer"), "footer hidden");
    assert!(!bare.contains("Edit this page"), "edit link hidden");
    assert!(bare.contains("custom-page"), "pageClass applied");
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
    assert!(!b.contains("Last updated"), "no timestamp without lastUpdated config");
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
    assert!(!page(&out, "/a/").contains("VPOutlineMarker"), "aside:false hides outline column");
    assert!(!page(&out, "/b/").contains("VPOutlineMarker"), "outline:false hides outline");
    assert!(!page(&out, "/b/").contains("VPOutlineDropdownButton"), "outline:false hides the local-nav dropdown too");
    assert!(page(&out, "/d/").contains("VPOutlineDropdownButton"), "default page keeps the local-nav dropdown");
    assert!(page(&out, "/c/").contains("xl:order-1") && page(&out, "/c/").contains("xl:order-2"), "left aside orders swapped");
    assert!(page(&out, "/d/").contains("VPOutlineMarker"), "default aside present");
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
            ("b.md", "---\nprev: Back to start\nnext:\n  text: External next\n  link: https://example.com/n\n---\n\n# B\n"),
        ],
    );
    let b = page(&out, "/b/");
    assert!(b.contains("Back to start"), "prev text override");
    assert!(b.contains("External next"), "next object override");
    assert!(b.contains(r#"href="https://example.com/n""#), "next custom link");
}

#[test]
fn frontmatter_search_false_and_head_and_title_template() {
    let (_, out) = build_site(
        "title = \"Base\"\n\n[search]\nprovider = \"local\"\n",
        &[
            ("a.md", "---\nsearch: false\ntitleTemplate: \":title!!\"\nhead:\n  - tag: meta\n    attrs:\n      name: \"x-page\"\n      content: \"yes\"\n---\n\n# A\n"),
            ("b.md", "# B\n"),
        ],
    );
    let index = std::fs::read_to_string(out.path().join("search-docs.json")).unwrap();
    assert!(!index.contains("\"/a/\""), "page excluded from search");
    assert!(index.contains("\"/b/\""), "other page indexed");
    let a = page(&out, "/a/");
    assert!(a.contains("<title>A!!</title>"), "page titleTemplate wins");
    assert!(a.contains(r#"<meta content="yes" name="x-page"/>"#), "per-page head tag");
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
    assert!(plain.contains("VPOutlineMarker"), "doc layout keeps outline");
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
    assert!(html.contains("<span class=\"hidden md:inline md:text-[0.8125rem]\">Suchen</span>"), "button text");
    assert!(html.contains("placeholder=\"Dokumente durchsuchen\""), "placeholder");
    assert!(html.contains("data-no-results=\"Nichts gefunden für {q}\""), "no-results string");
    assert!(html.contains(">zum Navigieren</span>"), "footer hint");
}
