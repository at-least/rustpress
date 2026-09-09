//! End-to-end markdown pipeline tests over the real vitepress docs
//! fixture subset: full preprocess → comrak → highlighter path.
//!
//! Fixture reality check: markdown.md documents features via
//! Input/Output example *fences* — its containers/alerts/code-groups stay
//! literal by design. Real rendered constructs live in getting-started /
//! routing / runtime-api; anything else gets a synthetic-page test here.

use gen_docs::config::Markdown as MarkdownConfig;
use gen_docs::content::{Content, Page};
use gen_docs::markdown::MarkdownEngine;
use std::path::Path;

fn engine() -> MarkdownEngine {
    MarkdownEngine::new(&MarkdownConfig::default()).expect("engine")
}

fn fixture(url: &str) -> gen_docs::markdown::RenderedPage {
    let site_root = Path::new("tests/fixtures");
    let content_dir = site_root.join("en");
    let content = Content::load(&content_dir).expect("content");
    let page = content.get(url).expect("page").clone();
    engine()
        .render(&page, &content, site_root, &content_dir)
        .expect("render")
}

fn synthetic(body: &str) -> gen_docs::markdown::RenderedPage {
    let site_root = Path::new("tests/fixtures");
    let content_dir = site_root.join("en");
    let content = Content::default();
    let page = Page {
        rel: "page.md".into(),
        url: "/page/".into(),
        title: "P".into(),
        front: Default::default(),
        body: body.to_string(),
        modified: None,
        src: "page.md".into(),
    };
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
    assert!(html.contains("st-"), "syntect scope classes");
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
    assert!(out.html.contains("markdown-alert"), "alert div");
    assert!(out.html.contains("<details class=\"custom-block details\" open>"), "details");
    assert!(out.html.contains("<summary>Click</summary>"));
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
    assert!(css.contains("html.dark .st-"));
    assert!(css.lines().any(|l| l.starts_with(".st-") && !l.contains("html.dark")));
}

#[test]
fn container_after_paragraph_interrupts_correctly_through_comrak() {
    let out = synthetic("a paragraph\n::: tip\ninner **bold**\n:::\nafter\n");
    assert!(out.html.contains("<div class=\"custom-block tip\">"), "div survives: {}",
        &out.html[..out.html.len().min(300)]);
    assert!(out.html.contains("inner <strong>bold</strong>"), "inner markdown parsed");
}

#[test]
fn syntax_css_carries_github_palette() {
    let css = engine().syntax_css();
    // github-light global: fg #24292e bg #ffffff; github-dark fg #e1e4e8.
    // (Block backgrounds ultimately come from --vp-code-block-bg in the
    // theme CSS, VitePress-style.)
    assert!(
        css.lines().any(|l| !l.contains("html.dark") && l.contains("#ffffff")),
        "light palette present"
    );
    assert!(
        css.contains("html.dark .st-code") && css.contains("#e1e4e8"),
        "dark palette present"
    );
}
