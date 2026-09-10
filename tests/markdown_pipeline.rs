//! End-to-end markdown pipeline tests over the real vitepress docs
//! fixture subset: full preprocess → comrak → highlighter path.
//!
//! Fixture reality check: markdown.md documents features via
//! Input/Output example *fences* — its containers/alerts/code-groups stay
//! literal by design. Real rendered constructs live in getting-started /
//! routing / runtime-api; anything else gets a synthetic-page test here.

use rustpress::content::{Content, Page};
use rustpress::markdown::MarkdownEngine;
use std::path::Path;

fn engine() -> MarkdownEngine {
    engine_with(&rustpress::config::Markdown::default())
}

fn engine_with(md: &rustpress::config::Markdown) -> MarkdownEngine {
    MarkdownEngine::new(
        md,
        &rustpress::config::SyntaxThemes::default(),
        std::path::Path::new("tests/fixtures"),
        "/",
    )
    .expect("engine")
}

fn fixture(url: &str) -> rustpress::markdown::RenderedPage {
    let site_root = Path::new("tests/fixtures");
    let content_dir = site_root.join("en");
    let content = Content::load(&content_dir, &[]).expect("content");
    let page = content.get(url).expect("page").clone();
    engine()
        .render(&page, &content, site_root, &content_dir)
        .expect("render")
}

fn synthetic(body: &str) -> rustpress::markdown::RenderedPage {
    let site_root = Path::new("tests/fixtures");
    let content_dir = site_root.join("en");
    let content = Content::default();
    let page = Page { body: body.to_string(), ..synthetic_page() };
    engine()
        .render(&page, &content, site_root, &content_dir)
        .expect("render")
}

#[test]
fn getting_started_renders_containers_code_groups_and_highlighting() {
    let out = fixture("/guide/getting-started/");
    let html = &out.html;
    assert!(html.contains("<div class=\"custom-block tip\">"), "tip container");
    assert!(html.contains("<p class=\"custom-block-title\">NOTE</p>"), "custom title");
    assert!(html.matches("<div class=\"vp-code-group\" x-data=\"codeGroup\">").count() == 4, "4 code groups with tabs component");
    assert!(html.matches("<div class=\"tabs\">").count() == 4, "tab strips emitted");
    assert!(html.contains("<label for=\"group-1-0\">npm</label>"), "npm tab label in strip");
    assert!(html.contains("<span class=\"lang\">npm</span>"), "npm lang label");
    assert!(html.contains("data-name=\"pnpm\""), "pnpm data-name");
    assert!(html.contains("class=\"language-sh\""), "sh fences highlighted");
    assert!(html.contains("tk-"), "tree-sitter capture classes");
    // the <<< @/snippets/init.ansi include: ANSI stripped, fenced
    assert!(!html.contains("\x1b["), "ansi escapes stripped");
    assert!(!html.contains("<<<"), "include line gone");
    assert!(html.contains("class=\"language-ansi\""), "ansi fence present");
}

#[test]
fn markdown_extensions_examples_stay_literal_but_gfm_renders() {
    let out = fixture("/guide/markdown/");
    let html = &out.html;
    // example containers/alerts are inside fences: literal
    assert!(html.contains("::: tip"), "container syntax literal in examples");
    assert!(html.contains("[!NOTE]"), "alert syntax literal in examples");
    assert!(html.contains("[!code highlight]"), "code markers literal");
    // real GFM constructs render
    assert!(html.contains("<table>"), "tables");
    assert!(html.contains("task-list-item"), "task lists");
    assert!(html.contains("footnotes"), "footnotes section");
    assert!(html.contains("<h2 id="), "heading ids");
}

#[test]
fn badges_render_in_runtime_api_headings() {
    let out = fixture("/reference/runtime-api/");
    assert!(
        out.html.matches("<span class=\"VPBadge info\">composable</span>").count() >= 4,
        "badge spans in headings"
    );
}

#[test]
fn internal_md_links_resolve_but_missing_targets_stay() {
    // what-is-vitepress.md carries real cross-links in the fixture subset
    let out = fixture("/guide/what-is-vitepress/");
    assert!(out.html.contains("href=\"/guide/getting-started/\""), "plain relative link");
    assert!(out.html.contains("href=\"/guide/markdown/\""), "another resolved link");
    assert!(
        out.html.contains("href=\"/guide/routing/#dynamic-routes\""),
        "anchor preserved"
    );
    // targets outside the fixture subset stay untouched (no fake URLs)
    assert!(out.html.contains("href=\"./custom-theme\""), "unknown target untouched");
    // nothing keeps a .md href anywhere
    assert!(!out.html.contains(".md\""), "no .md hrefs remain");
}

#[test]
fn headings_toc_ids_match_rendered_anchors() {
    let out = fixture("/guide/markdown/");
    for h in out.headings.iter().filter(|h| h.level == 2).take(5) {
        let needle = format!("<h2 id=\"{}\"", h.id);
        assert!(out.html.contains(&needle), "heading {h:?} id matches render");
    }
}

#[test]
fn github_alerts_and_details_render() {
    let out = synthetic("> [!NOTE]\n> this is a note\n\n::: details Click {open}\ncontent\n:::\n");
    // alerts ARE containers here (VitePress semantics): styled, labelable
    assert!(out.html.contains("<div class=\"custom-block note\">"), "note container");
    assert!(out.html.contains("<p class=\"custom-block-title\">NOTE</p>"), "default title");
    assert!(out.html.contains("<details class=\"custom-block details\" open>"), "details");
    assert!(out.html.contains("<summary>Click</summary>"));
}

#[test]
fn danger_alert_and_custom_alert_titles() {
    let out = synthetic(
        "> [!DANGER]\n> boom\n\n> [!WARNING] Watch out\n> careful\n\n> [!TIP]\n> multi\n>\n> paragraph\n",
    );
    assert!(out.html.contains("<div class=\"custom-block danger\">"), "danger container");
    assert!(out.html.contains("<p class=\"custom-block-title\">Watch out</p>"), "alert custom title");
    // a `>`-only line continues the alert body as a paragraph break
    let tip = out.html.find("custom-block tip").expect("tip container");
    let after = &out.html[tip..];
    assert!(after.contains("<p>multi</p>") && after.contains("<p>paragraph</p>"), "split paragraphs");
}

#[test]
fn custom_container_alert_kind() {
    let md = rustpress::config::Markdown {
        container: toml::from_str(
            "[[custom]]\nname = \"success\"\nkind = \"tip\"\nlabel = \"SUCCESS\"\n",
        )
        .unwrap(),
        ..Default::default()
    };
    let out = engine_with(&md);
    let site_root = std::path::Path::new("tests/fixtures");
    let content_dir = site_root.join("en");
    let page = Page { body: "> [!SUCCESS]\n> yes\n".into(), ..synthetic_page() };
    let out = out
        .render(&page, &Content::default(), site_root, &content_dir)
        .expect("render");
    assert!(out.html.contains("<div class=\"custom-block tip\">"), "custom kind styling");
    assert!(out.html.contains("<p class=\"custom-block-title\">SUCCESS</p>"), "custom label");
}

fn synthetic_page() -> Page {
    Page {
        rel: "page.md".into(),
        url: "/page/".into(),
        title: "P".into(),
        front: Default::default(),
        body: String::new(),
        modified: None,
        src: "page.md".into(),
        locale: "root".into(),
    }
}

#[test]
fn hl_lines_and_vitepress_fence_syntax() {
    let out = synthetic("```js{2}\nconst a = 1;\nconst b = 2;\n```\n\n```ts [config.ts]\nlet x: number = 1;\n```\n");
    assert!(out.html.contains("<span class=\"line hl\">"), "hl line");
    assert!(out.html.contains("data-name=\"config.ts\""), "label");
    assert!(out.html.contains("class=\"language-ts\""), "ts highlighted");
}

#[test]
fn syntax_css_dual_theme() {
    let css = engine().syntax_css();
    assert!(css.starts_with("@layer syntax"));
    assert!(css.contains("html.dark .tk-"));
    assert!(css.lines().any(|l| l.starts_with(".tk-") && !l.contains("html.dark")));
}

#[test]
fn container_after_paragraph_interrupts_correctly_through_comrak() {
    let out = synthetic("a paragraph\n::: tip\ninner **bold**\n:::\nafter\n");
    assert!(out.html.contains("<div class=\"custom-block tip\">"), "div survives: {}",
        &out.html[..out.html.len().min(300)]);
    assert!(out.html.contains("inner <strong>bold</strong>"), "inner markdown parsed");
}


#[test]
fn toc_placeholder_becomes_table_of_contents() {
    let out = synthetic("[[toc]]\n\n## One\n\ntext\n\n### Deep\n\n## Two\n");
    assert!(
        out.html.contains("<nav class=\"table-of-contents\"><ul><li><a href=\"#one\">One</a><ul><li><a href=\"#deep\">Deep</a></li></ul></li><li><a href=\"#two\">Two</a></li></ul></nav>"),
        "nested toc: {}",
        out.html[out.html.find("table-of-contents").map(|i| i.saturating_sub(30)).unwrap_or(0)..].chars().take(260).collect::<String>()
    );
}

#[test]
fn custom_heading_anchors_replace_slugs() {
    let out = synthetic("## Deep Dive {#dive}\n\ntext\n\n## Plain\n");
    assert!(out.html.contains("<h2 id=\"dive\">Deep Dive<a href=\"#dive\""), "custom id: {}",
        out.html[out.html.find("<h2").unwrap()..].chars().take(160).collect::<String>());
    assert!(out.html.contains("<h2 id=\"plain\">Plain<a href=\"#plain\""), "plain slug kept");
    assert!(!out.html.contains("{#dive}"), "attr literal stripped from heading text");
    assert!(!out.html.contains("id=\"deep-dive\""), "slug replaced");
    assert_eq!(out.headings[0].id, "dive");
    assert_eq!(out.headings[0].text, "Deep Dive");
}

#[test]
fn math_renders_with_delimiters_and_flags_page() {
    let site_root = Path::new("tests/fixtures");
    let content_dir = site_root.join("en");
    let config = rustpress::config::Markdown { math: true, ..Default::default() };
    let engine = engine_with(&config);
    let page = Page {
        rel: "p.md".into(),
        url: "/p/".into(),
        title: "P".into(),
        front: Default::default(),
        body: "Inline $a^2$ and $$b_c$$ here.\n".into(),
        modified: None,
        src: "p.md".into(),
        locale: "root".into(),
    };
    let out = engine.render(&page, &Content::default(), site_root, &content_dir).unwrap();
    assert!(out.has_math, "page flagged");
    assert!(out.html.contains(r#"<span data-math-style="inline">\(a^2\)</span>"#), "inline delim: {}",
        &out.html[..out.html.len().min(500)]);
    assert!(out.html.contains(r"\[b_c\]"), "display delim");
}

#[test]
fn non_math_pages_unflagged_and_untouched() {
    let out = synthetic("just text $ not math $\n");
    assert!(!out.has_math);
}

// ---- stage-4 markdown engine features ---------------------------------

fn include_site(files: &[(&str, &str)], page_body: &str) -> rustpress::markdown::RenderedPage {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "gd-include-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    for (rel, body) in files {
        let path = dir.join("content").join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }
    let content_dir = dir.join("content");
    let page = Page { body: page_body.to_string(), ..synthetic_page() };
    let out = engine()
        .render(&page, &Content::default(), &dir, &content_dir)
        .expect("render");
    let _ = std::fs::remove_dir_all(&dir);
    out
}

#[test]
fn markdown_include_selectors() {
    let out = include_site(
        &[(
            "parts/basics.md",
            "line one\nline two\nline three\nline four\n",
        )],
        "A\n\n<!--@include: ./parts/basics.md{2,3}-->\n\nB\n",
    );
    assert!(out.html.contains("line two") && out.html.contains("line three"));
    assert!(!out.html.contains("line one") && !out.html.contains("line four"), "range applied");

    let out = include_site(
        &[(
            "parts/sections.md",
            "# Top\n\nintro\n\n## My Base Section\n\nbase body\n\n### My Sub Section\n\nsub body\n\n## Another Section\n\nother body\n",
        )],
        "<!--@include: ./parts/sections.md#my-base-section-->\n",
    );
    assert!(out.html.contains("base body") && out.html.contains("sub body"), "section + nested");
    assert!(!out.html.contains("other body"), "stops at same-level heading");
    assert!(!out.html.contains("intro"), "starts at the matched heading");

    let out = include_site(
        &[("parts/r.md", "// #region demo\nkept line\n// #endregion\ndropped\n")],
        "<!--@include: ./parts/r.md#demo-->\n",
    );
    assert!(out.html.contains("kept line") && !out.html.contains("dropped"), "region selected");
}

#[test]
fn include_directive_inside_code_fence_inserts_verbatim() {
    let out = include_site(
        &[("snip.txt", "raw text line\n")],
        "```md\n<!--@include: ./snip.txt-->\n```\n",
    );
    assert!(out.html.contains("raw text line"), "fence include expanded");
    assert!(
        !out.html.contains("&lt;!--@include"),
        "directive itself must not appear"
    );
}

#[test]
fn inline_footnotes_render() {
    let out = include_site(
        &[],
        "An inline^[careful reader] footnote and code `keep ^[this]` literal.\n",
    );
    assert!(!out.html.contains("^[") || out.html.contains("<code>keep ^[this]</code>"));
    assert!(out.html.contains("footnote-ref"), "became a footnote reference");
    assert!(out.html.contains("careful reader"), "content kept");
}

#[test]
fn link_attribute_blocks() {
    let out = include_site(
        &[],
        "[Home](/guide/){target=\"_self\"}\n\n[Plain](/other/)\n",
    );
    assert!(
        out.html.contains(r#"<a href="/guide/" target="_self">Home</a>"#),
        "attrs applied"
    );
    assert!(
        out.html.contains(r#"><a href="/other/">Plain</a>"#)
            || out.html.contains(r#"<a href="/other/">Plain</a>"#),
        "plain link untouched"
    );
}

#[test]
fn raw_container_wraps_vp_raw() {
    let out = synthetic("::: raw\n<b>embedded</b>\n:::\n");
    assert!(out.html.contains("<div class=\"vp-raw\">"), "vp-raw wrapper");
    assert!(out.html.contains("<b>embedded</b>"));
}
