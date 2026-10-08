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
        &rustpress::config::SyntaxHighlight::default(),
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
    let page = Page {
        body: body.to_string(),
        ..synthetic_page()
    };
    engine()
        .render(&page, &content, site_root, &content_dir)
        .expect("render")
}

#[test]
fn getting_started_renders_containers_code_groups_and_highlighting() {
    let out = fixture("/guide/getting-started/");
    let html = &out.html;
    assert!(
        html.contains("<div class=\"custom-block tip\">"),
        "tip container"
    );
    assert!(
        html.contains("<p class=\"custom-block-title\">NOTE</p>"),
        "custom title"
    );
    assert!(
        html.matches("<div class=\"vp-code-group\" x-data=\"codeGroup\">")
            .count()
            == 4,
        "4 code groups with tabs component"
    );
    assert!(
        html.matches("<div class=\"tabs\">").count() == 4,
        "tab strips emitted"
    );
    assert!(
        html.contains("<label for=\"group-1-0\">npm</label>"),
        "npm tab label in strip"
    );
    assert!(
        html.contains("<span class=\"lang\">sh</span>"),
        "group pane corner shows the language, not the tab label (upstream)"
    );
    assert!(html.contains("data-name=\"pnpm\""), "pnpm data-name");
    assert!(
        html.contains("class=\"language-sh\""),
        "sh fences highlighted"
    );
    assert!(html.contains("tk-"), "tree-sitter capture classes");
    // the <<< @/snippets/init.ansi include: ANSI stripped, fenced
    assert!(!html.contains("\x1b["), "ansi escapes stripped");
    assert!(!html.contains("<<<"), "include line gone");
    assert!(
        html.contains("class=\"language-ansi\""),
        "ansi fence present"
    );
}

#[test]
fn markdown_extensions_examples_stay_literal_but_gfm_renders() {
    let out = fixture("/guide/markdown/");
    let html = &out.html;
    // example containers/alerts are inside fences: literal
    assert!(
        html.contains("::: tip"),
        "container syntax literal in examples"
    );
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
        out.html
            .matches("<span class=\"VPBadge info\">composable</span>")
            .count()
            >= 4,
        "badge spans in headings"
    );
}

#[test]
fn internal_md_links_resolve_but_missing_targets_stay() {
    // what-is-vitepress.md carries real cross-links in the fixture subset
    let out = fixture("/guide/what-is-vitepress/");
    assert!(
        out.html.contains("href=\"/guide/getting-started/\""),
        "plain relative link"
    );
    assert!(
        out.html.contains("href=\"/guide/markdown/\""),
        "another resolved link"
    );
    assert!(
        out.html.contains("href=\"/guide/routing/#dynamic-routes\""),
        "anchor preserved"
    );
    // targets outside the fixture subset stay untouched (no fake URLs)
    assert!(
        out.html.contains("href=\"./custom-theme\""),
        "unknown target untouched"
    );
    // nothing keeps a .md href anywhere
    assert!(!out.html.contains(".md\""), "no .md hrefs remain");
}

#[test]
fn headings_toc_ids_match_rendered_anchors() {
    let out = fixture("/guide/markdown/");
    for h in out.headings.iter().filter(|h| h.level == 2).take(5) {
        let needle = format!("<h2 id=\"{}\"", h.id);
        assert!(
            out.html.contains(&needle),
            "heading {h:?} id matches render"
        );
    }
}

#[test]
fn heading_ids_follow_upstream_mdit_slug_rules() {
    // VitePress anchors come from @mdit-vue slugify over text/code tokens
    // only: HTML (including the whole Badge — its text is an attribute)
    // and shortcode emoji are dropped; specials collapse into single
    // dashes; apostrophes are runs too. Expectations computed with the
    // upstream slugify itself.
    let out = synthetic(
        "## Emoji :tada:\n\n## useData <Badge type=\"info\" text=\"composable\" />\n\n## What's Next\n\n## Adding a meta name=\"og:title\" tag\n\n## 123 Start\n",
    );
    let html = &out.html;
    for id in [
        "emoji",                           // shortcode emoji dropped, no trailing dash
        "usedata",                         // badge text excluded from the slug
        "what-s-next",                     // apostrophe is a special run → dash
        "adding-a-meta-name-og-title-tag", // quotes/colon specials
        "_123-start",                      // leading digit gets a _ prefix
    ] {
        assert!(
            html.contains(&format!("<h2 id=\"{id}\" tabindex=\"-1\">")),
            "{id}"
        );
    }
    // the permalink is the upstream header-anchor shape, not comrak's
    assert!(
        html.contains("<a class=\"header-anchor\" href=\"#emoji\" aria-label=\"Permalink to \u{201c}Emoji\u{201d}\">&#8203;</a>"),
        "upstream permalink with marker-less title"
    );
    assert!(
        html.contains("aria-label=\"Permalink to \u{201c}useData\u{201d}\""),
        "badge text out of the permalink label too"
    );
    assert!(!html.contains("class=\"anchor\""));
    // outline text still shows what the reader sees (badge + emoji)
    assert!(out.headings.iter().any(|h| h.text.contains("🎉")));
}

#[test]
fn footnote_section_gets_the_separator_hr() {
    let out = synthetic("text[^1]\n\n[^1]: note\n");
    assert!(
        out.html
            .contains("<hr class=\"footnotes-sep\"><section class=\"footnotes\""),
        "markdown-it-footnote separator before the footnotes section"
    );
}

#[test]
fn code_group_pane_corner_shows_language_and_atat_include_unescapes() {
    // upstream group panes label the corner with the LANGUAGE (tab text
    // stays in the strip); vitepress.dev's docs corpus escapes includes
    // inside display fences as `@@include` and its code transformer
    // unescapes them back to `@include` — same effect, core-side here
    let out = synthetic(
        "::: code-group\n\n```js [npm]\nconst x = 1;\n```\n\n```sh [pnpm]\npnpm i\n```\n\n:::\n\n```md\n<!--@@include: ./parts/basics.md-->\n```\n",
    );
    let html = &out.html;
    assert!(html.contains(">npm</label>"), "tab strip labels kept");
    assert!(
        html.contains("<span class=\"lang\">js</span>"),
        "pane corner = language"
    );
    assert!(
        !html.contains("<span class=\"lang\">npm</span>"),
        "tab label no longer doubles as the corner label"
    );
    assert!(
        !html.contains("@@include") && html.contains("@include"),
        "display fences render the @@-escape as @include"
    );
}

#[test]
fn github_alerts_and_details_render() {
    let out = synthetic("> [!NOTE]\n> this is a note\n\n::: details Click {open}\ncontent\n:::\n");
    // alerts ARE containers here (VitePress semantics): styled, labelable
    assert!(
        out.html.contains("<div class=\"custom-block note\">"),
        "note container"
    );
    assert!(
        out.html
            .contains("<p class=\"custom-block-title\">NOTE</p>"),
        "default title"
    );
    assert!(
        out.html
            .contains("<details class=\"custom-block details\" open>"),
        "details"
    );
    assert!(out.html.contains("<summary>Click</summary>"));
}

#[test]
fn danger_alert_and_custom_alert_titles() {
    let out = synthetic(
        "> [!DANGER]\n> boom\n\n> [!WARNING] Watch out\n> careful\n\n> [!TIP]\n> multi\n>\n> paragraph\n",
    );
    assert!(
        out.html.contains("<div class=\"custom-block danger\">"),
        "danger container"
    );
    assert!(
        out.html
            .contains("<p class=\"custom-block-title\">Watch out</p>"),
        "alert custom title"
    );
    // a `>`-only line continues the alert body as a paragraph break
    let tip = out.html.find("custom-block tip").expect("tip container");
    let after = &out.html[tip..];
    assert!(
        after.contains("<p>multi</p>") && after.contains("<p>paragraph</p>"),
        "split paragraphs"
    );
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
    let page = Page {
        body: "> [!SUCCESS]\n> yes\n".into(),
        ..synthetic_page()
    };
    let out = out
        .render(&page, &Content::default(), site_root, &content_dir)
        .expect("render");
    assert!(
        out.html.contains("<div class=\"custom-block tip\">"),
        "custom kind styling"
    );
    assert!(
        out.html
            .contains("<p class=\"custom-block-title\">SUCCESS</p>"),
        "custom label"
    );
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
    let out = synthetic(
        "```js{2}\nconst a = 1;\nconst b = 2;\n```\n\n```ts [config.ts]\nlet x: number = 1;\n```\n",
    );
    assert!(out.html.contains("<span class=\"line hl\">"), "hl line");
    assert!(out.html.contains("data-name=\"config.ts\""), "label");
    assert!(out.html.contains("class=\"language-ts\""), "ts highlighted");
}

#[test]
fn syntax_css_dual_theme() {
    let css = engine().syntax_css();
    assert!(css.starts_with("@layer syntax"));
    assert!(css.contains("html.dark .tk-"));
    assert!(
        css.lines()
            .any(|l| l.starts_with(".tk-") && !l.contains("html.dark"))
    );
}

#[test]
fn container_after_paragraph_interrupts_correctly_through_comrak() {
    let out = synthetic("a paragraph\n::: tip\ninner **bold**\n:::\nafter\n");
    assert!(
        out.html.contains("<div class=\"custom-block tip\">"),
        "div survives: {}",
        &out.html[..out.html.len().min(300)]
    );
    assert!(
        out.html.contains("inner <strong>bold</strong>"),
        "inner markdown parsed"
    );
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
    assert!(
        out.html.contains(
            "<h2 id=\"dive\" tabindex=\"-1\">Deep Dive<a class=\"header-anchor\" href=\"#dive\""
        ),
        "custom id: {}",
        out.html[out.html.find("<h2").unwrap()..]
            .chars()
            .take(160)
            .collect::<String>()
    );
    assert!(
        out.html.contains(
            "<h2 id=\"plain\" tabindex=\"-1\">Plain<a class=\"header-anchor\" href=\"#plain\""
        ),
        "plain slug kept"
    );
    assert!(
        !out.html.contains("{#dive}"),
        "attr literal stripped from heading text"
    );
    assert!(!out.html.contains("id=\"deep-dive\""), "slug replaced");
    assert_eq!(out.headings[0].id, "dive");
    assert_eq!(out.headings[0].text, "Deep Dive");
}

#[test]
fn duplicate_and_self_anchorizing_custom_ids_keep_no_literal() {
    // the literal-text strip used to key off the FINAL (deduplicated)
    // id, so a second `## B {#c}` (final id c-1) and a self-anchorizing
    // `## {#foo}` (final id == rendered slug, filtered out entirely)
    // shipped the `{#…}` attribute as visible heading text
    let out = synthetic("## A {#c}\n\n## B {#c}\n\ntext\n");
    let start = out.html.find("<h2 id=\"c-1\"").expect("deduped id");
    let second = &out.html[start..];
    assert!(
        second.contains("<h2 id=\"c-1\" tabindex=\"-1\">B<a"),
        "second heading text carries no literal: {}",
        second.chars().take(120).collect::<String>()
    );
    assert!(!out.html.contains("{#c}"), "both literals stripped");

    let out = synthetic("## {#foo}\n\ntext\n");
    assert!(
        out.html.contains("<h2 id=\"foo\" tabindex=\"-1\">"),
        "self-anchorizing custom id still applies: {}",
        &out.html[out.html.find("<h2").unwrap()..out.html.find("<h2").unwrap() + 80]
    );
    assert!(!out.html.contains("{#foo}"), "attr literal stripped");
}

#[test]
fn math_renders_with_delimiters_and_flags_page() {
    let site_root = Path::new("tests/fixtures");
    let content_dir = site_root.join("en");
    let config = rustpress::config::Markdown {
        math: true,
        ..Default::default()
    };
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
    let out = engine
        .render(&page, &Content::default(), site_root, &content_dir)
        .unwrap();
    assert!(out.has_math, "page flagged");
    assert!(
        out.html
            .contains(r#"<span data-math-style="inline">\(a^2\)</span>"#),
        "inline delim: {}",
        &out.html[..out.html.len().min(500)]
    );
    assert!(out.html.contains(r"\[b_c\]"), "display delim");
}

#[test]
fn non_math_pages_unflagged_and_untouched() {
    let out = synthetic("just text $ not math $\n");
    assert!(!out.has_math);
}

// ---- stage-4 markdown engine features ---------------------------------

fn include_site(files: &[(&str, &str)], page_body: &str) -> rustpress::markdown::RenderedPage {
    include_site_result(files, page_body).expect("render")
}

fn include_site_result(
    files: &[(&str, &str)],
    page_body: &str,
) -> Result<rustpress::markdown::RenderedPage, rustpress::markdown::MarkdownError> {
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
    let page = Page {
        body: page_body.to_string(),
        ..synthetic_page()
    };
    let out = engine().render(&page, &Content::default(), &dir, &content_dir);
    let _ = std::fs::remove_dir_all(&dir);
    out
}

#[test]
fn markdown_include_strips_part_file_frontmatter() {
    // upstream strips the front matter of included .md files; shipping
    // it raw rendered a thematic break plus the key: value lines
    let out = include_site(
        &[(
            "parts/changelog.md",
            "---\ntitle: Changelog\n---\n\nChangelog body.\n",
        )],
        "<!--@include: ./parts/changelog.md-->\n",
    );
    assert!(out.html.contains("Changelog body."), "body included");
    assert!(!out.html.contains("title: Changelog"), "front matter gone");
    assert!(!out.html.contains("<hr"), "no stray thematic break");

    // a bare line range counts raw file lines (front matter included —
    // that is what makes the numbers stable), so it must NOT strip
    let out = include_site(
        &[("parts/ranged.md", "---\ntitle: R\n---\n\nfirst\nsecond\n")],
        "<!--@include: ./parts/ranged.md{5,6}-->\n",
    );
    assert!(
        out.html.contains("first") && out.html.contains("second"),
        "range numbering spans the front matter"
    );
    assert!(
        !out.html.contains("title: R"),
        "out-of-range lines stay out"
    );
}

#[test]
fn nested_github_alerts_expand_to_nested_containers() {
    // `> > [!TIP]` inside an alert used to fall through to comrak's
    // built-in alerts extension, whose markdown-alert classes nothing
    // styles; it must expand to a nested custom-block like GitHub's
    let out = synthetic("> [!NOTE]\n> outer\n>\n> > [!TIP]\n> > inner\n");
    let note = out.html.find("custom-block note").expect("outer note");
    let tip = out.html.find("custom-block tip").expect("nested tip");
    let close = out.html.rfind("</div>").expect("outer close");
    assert!(
        note < tip && tip < close,
        "tip block nested inside the note block"
    );
    assert!(out.html.contains("inner"), "nested body kept");
    assert!(
        !out.html.contains("[!TIP]"),
        "no literal marker in the output"
    );
    assert!(
        !out.html.contains("markdown-alert"),
        "comrak's unstyled alert renderer not used"
    );
}

#[test]
fn heading_section_extraction_matches_rendered_unicode_ids() {
    // the local slugify kept ASCII only, while comrak renders CJK and
    // accented headings with their letters intact — so the anchor the
    // docs tell authors to copy from the rendered page never matched
    let out = include_site(
        &[("parts/zh.md", "## 中文标题\n\nSection body.\n")],
        "<!--@include: ./parts/zh.md#中文标题-->\n",
    );
    assert!(
        out.html.contains("Section body."),
        "CJK heading anchor matches its rendered id"
    );
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
    assert!(
        !out.html.contains("line one") && !out.html.contains("line four"),
        "range applied"
    );

    let out = include_site(
        &[(
            "parts/sections.md",
            "# Top\n\nintro\n\n## My Base Section\n\nbase body\n\n### My Sub Section\n\nsub body\n\n## Another Section\n\nother body\n",
        )],
        "<!--@include: ./parts/sections.md#my-base-section-->\n",
    );
    assert!(
        out.html.contains("base body") && out.html.contains("sub body"),
        "section + nested"
    );
    assert!(
        !out.html.contains("other body"),
        "stops at same-level heading"
    );
    assert!(!out.html.contains("intro"), "starts at the matched heading");

    let out = include_site(
        &[(
            "parts/r.md",
            "// #region demo\nkept line\n// #endregion\ndropped\n",
        )],
        "<!--@include: ./parts/r.md#demo-->\n",
    );
    assert!(
        out.html.contains("kept line") && !out.html.contains("dropped"),
        "region selected"
    );
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
fn fence_include_without_trailing_newline_keeps_fence_closed() {
    // the included file's last line has no trailing newline: the closing
    // fence must still close the block instead of gluing onto the
    // inserted content and swallowing the rest of the page
    let out = include_site(
        &[("snip2.txt", "raw text line")],
        "```md\n<!--@include: ./snip2.txt-->\n```\n\nafter the fence\n",
    );
    assert!(out.html.contains("raw text line"), "include expanded");
    let code_end = out.html.find("</code></pre>").expect("code block closed");
    let after = out
        .html
        .find("after the fence")
        .expect("rest of the page kept");
    assert!(
        after > code_end,
        "following paragraph must sit after the closed block"
    );
}

#[test]
fn markdown_include_accepts_space_after_comment_open() {
    // upstream's directive regex is <!--\s*@include:\s*…-->; a space
    // after the comment opener must not ship the directive verbatim
    let out = include_site(
        &[("parts/s.md", "SPACE-FORM BODY\n")],
        "<!-- @include: ./parts/s.md-->\n",
    );
    assert!(
        out.html.contains("SPACE-FORM BODY"),
        "space form expands: {}",
        out.html
    );
}

#[test]
fn list_base_clears_when_the_item_ends() {
    // a column-0 paragraph after a blank ends the list (lazy
    // continuation only survives without an intervening blank): a later
    // 4-space fence is a top-level indented code block again, not an
    // item fence — the base must not leak across the page
    let out = synthetic("1. item\n\ntext\n\n    ```js\n    const a = 1\n    ```\n");
    assert!(
        !out.html.contains(r#"data-lang="js""#),
        "no item fence after the list ended: {}",
        out.html
    );
}

#[test]
fn list_item_fences_without_inline_marker_stay_fences() {
    // regression: the ≤3-space opener cap also rejected the 4-space
    // indented fences CommonMark allows inside list items (content
    // column 3 for "1. "), un-styling every fenced block under a list
    // — the shape demo/content/guide/cms.md ships
    let out = synthetic("1. Do it:\n\n    ```js\n    const a = 1\n    ```\n\ntail\n");
    assert!(
        out.html.contains(r#"data-lang="js""#),
        "item fence keeps its info rewrite: {}",
        out.html
    );
    assert_eq!(out.html.matches("<pre").count(), 1, "single fenced block");
}

#[test]
fn region_include_matches_tokens_keeps_indent_and_nests() {
    // upstream semantics (include.ts): the name is an exact token (demo
    // does not match demo2), a nested #region inside the selected one
    // does not end it early, and the body keeps its indentation — only
    // `<<<` snippet regions are dedented (snippet.ts)
    let part = "fn wrapper() {\n    #region demo2\n    wrong body\n    #endregion\n    #region demo\n    let a = 1;\n    #region inner\n    let b = 2;\n    #endregion\n    let c = 3;\n    #endregion\n}\n";
    let out = include_site(
        &[("parts/r.md", part)],
        "<!--@include: ./parts/r.md#demo-->\n",
    );
    assert!(
        out.html
            .contains("<pre><code>let a = 1;\nlet b = 2;\nlet c = 3;\n</code></pre>"),
        "four-space body kept as an indented code block: {}",
        out.html
    );
    assert!(
        !out.html.contains("wrong body"),
        "demo2 not matched by substring"
    );
    assert!(!out.html.contains("fn wrapper"), "extraction bounds hold");

    // the snippet form dedents the same region (the body is tokenized
    // by the highlighter, so the lines are checked by their spans)
    let out = include_site(&[("parts/r.rs", part)], "<<< ./parts/r.rs#demo\n");
    assert_eq!(
        out.html
            .matches("<span class=\"line\"><span class=\"tk-keyword\">let</span>")
            .count(),
        3,
        "three dedented `let` lines, each starting its line span: {}",
        out.html
    );
    assert!(
        !out.html.contains("<span class=\"line\">    "),
        "snippet region dedented: {}",
        out.html
    );
}

#[test]
fn heading_section_anchor_matches_deduplicated_rendered_ids() {
    // the rendered page numbers repeated headings (`foo`, `foo-1`); the
    // anchor an author copies for the second one must select it
    let out = include_site(
        &[(
            "parts/dup.md",
            "## Foo\n\nfirst body\n\n## Foo\n\nsecond body\n",
        )],
        "<!--@include: ./parts/dup.md#foo-1-->\n",
    );
    assert!(
        out.html.contains("second body") && !out.html.contains("first body"),
        "#foo-1 selects the second heading: {}",
        out.html
    );
}

#[test]
fn include_inside_a_tab_indented_list_item_fence_stays_in_the_item() {
    // the include pad re-expressed the opener's column in spaces from a
    // byte count, so a tab-indented item fence padded one space: the
    // included lines sat left of the item's content column and comrak
    // ended the item (and its fence) there
    let out = include_site(
        &[("parts/x.md", "included line\n")],
        "1. item\n\n\t```\n\t<!--@include: ./parts/x.md-->\n\t```\n\nafter\n",
    );
    let li_end = out.html.find("</li>").expect("a list item");
    let code = out.html.find("included line").expect("included text");
    assert!(
        code < li_end,
        "included text inside the list item: {}",
        out.html
    );
    assert!(
        !out.html.contains("<p>included line</p>"),
        "not a top-level paragraph: {}",
        out.html
    );
}

#[test]
fn image_attribute_blocks_stay_images_and_get_the_base() {
    // `![alt](src){attrs}` used to become an <a href=src>alt</a>; the
    // attrs belong on the <img>, and its src gets the site base like
    // every other image URL
    let engine = MarkdownEngine::new(
        &rustpress::config::Markdown::default(),
        &rustpress::config::SyntaxHighlight::default(),
        Path::new("tests/fixtures"),
        "/b/",
    )
    .expect("engine");
    let page = Page {
        body: "![alt](/img.png){width=\"100\"}\n".into(),
        ..synthetic_page()
    };
    let out = engine
        .render(
            &page,
            &Content::default(),
            Path::new("tests/fixtures"),
            &Path::new("tests/fixtures").join("en"),
        )
        .expect("render");
    assert!(
        out.html
            .contains(r#"<img src="/b/img.png" alt="alt" width="100">"#),
        "image element with attrs and base: {}",
        out.html
    );
    assert!(!out.html.contains("<a href"), "not a link: {}", out.html);
    assert!(!out.html.contains("data-gd-mdlink"), "marker stripped");
}

#[test]
fn attribute_blocks_keep_data_href_and_data_src() {
    // the post-pass strips the emitter's own href/src before rebuilding
    // the tag; without an attribute-name boundary it also ate the tail
    // of an authored `data-href` / `data-src`, leaving a dangling `data-`
    let engine = MarkdownEngine::new(
        &rustpress::config::Markdown::default(),
        &rustpress::config::SyntaxHighlight::default(),
        Path::new("tests/fixtures"),
        "/b/",
    )
    .expect("engine");
    let page = Page {
        body: "[t](/a/){data-href=\"x\"}\n\n![a](/i.png){data-src=\"y\"}\n".into(),
        ..synthetic_page()
    };
    let out = engine
        .render(
            &page,
            &Content::default(),
            Path::new("tests/fixtures"),
            &Path::new("tests/fixtures").join("en"),
        )
        .expect("render");
    assert!(
        out.html.contains(r#"<a href="/b/a/" data-href="x">t</a>"#),
        "authored data-href kept: {}",
        out.html
    );
    assert!(
        out.html
            .contains(r#"<img src="/b/i.png" alt="a" data-src="y">"#),
        "authored data-src kept: {}",
        out.html
    );
}

#[test]
fn fenced_include_cannot_break_out_of_the_authors_fence() {
    // a partial that itself contains fences must not close the author's
    // fence: verbatim insertion stays verbatim, but the emitted opening
    // marker has to out-run any closing run the partial brings in, or
    // everything after the directive is swallowed as code
    let out = include_site(
        &[("fenced.md", "```js\nx\n```\n")],
        "```md\n<!--@include: ./fenced.md-->\n```\n\nAFTER-PARA\n",
    );
    assert!(
        out.html.contains("<p>AFTER-PARA</p>"),
        "rest of the page must stay a paragraph: {}",
        out.html
    );
    // the partial's own fence runs survive as code-block content
    // (tokenized into spans, so the pieces are asserted separately)
    assert!(out.html.contains("```"), "partial fences kept as content");
    assert!(
        out.html.contains(">x<"),
        "partial body line kept: {}",
        out.html
    );
    assert_eq!(
        out.html.matches("<pre").count(),
        1,
        "one code block, not two: {}",
        out.html
    );
}

#[test]
fn fenced_include_outgrows_tilde_fences_too() {
    let out = include_site(
        &[("tilde.md", "~~~sh\ny\n~~~\n")],
        "~~~md\n<!--@include: ./tilde.md-->\n~~~\n\nAFTER-TILDE\n",
    );
    assert!(
        out.html.contains("<p>AFTER-TILDE</p>"),
        "tilde fence not broken out of: {}",
        out.html
    );
    assert_eq!(out.html.matches("<pre").count(), 1, "single block");
}

#[test]
fn two_fenced_includes_still_leave_one_block() {
    let out = include_site(
        &[("a.md", "```\np\n```\n"), ("b.md", "```\nq\n```\n")],
        "```md\n<!--@include: ./a.md-->\n<!--@include: ./b.md-->\n```\n\nTAIL\n",
    );
    assert!(
        out.html.contains("<p>TAIL</p>"),
        "tail intact: {}",
        out.html
    );
    assert_eq!(out.html.matches("<pre").count(), 1, "single block");
}

#[test]
fn unclosed_fence_with_include_flushes_verbatim() {
    // no closing fence: everything runs to EOF, nothing can break out,
    // and the buffer must still be flushed
    let out = include_site(
        &[("loose.md", "text\n")],
        "opening line\n```md\n<!--@include: ./loose.md-->\nstill inside\n",
    );
    assert!(
        out.html.contains("still inside"),
        "body flushed: {}",
        out.html
    );
    assert!(out.html.contains("text"), "include flushed");
}

#[test]
fn markdown_include_line_range_with_ranges() {
    // `{2,3-4}` = lines 2, 3 and 4 — a hyphen range inside the spec must
    // select its lines, not silently fall through to EOF
    let out = include_site(
        &[("parts/ranged.md", "l1\nl2\nl3\nl4\nl5\n")],
        "<!--@include: ./parts/ranged.md{2,3-4}-->\n",
    );
    assert!(
        out.html.contains("l2") && out.html.contains("l3") && out.html.contains("l4"),
        "lines 2, 3, 4 selected"
    );
    assert!(
        !out.html.contains("l1") && !out.html.contains("l5"),
        "range bounds respected"
    );
}

#[test]
fn markdown_include_line_range_rejects_garbage_loudly() {
    // a range spec that parses to nothing sensible must fail the build
    // naming the spec — not silently fall through to EOF ({2,5-}, {1,x}),
    // select nothing ({4,2}, {0,2}), or ship the whole file
    for spec in ["{2,5-}", "{1,x}", "{4,2}", "{0,2}", "{9,}", "{1,99}"] {
        let err = include_site_result(
            &[("parts/ranged2.md", "l1\nl2\nl3\nl4\nl5\n")],
            &format!("<!--@include: ./parts/ranged2.md{spec}-->\n"),
        )
        .err()
        .unwrap_or_else(|| panic!("{spec} must fail the build"));
        assert!(
            err.to_string().contains(spec),
            "error must name the spec {spec}: {err}"
        );
    }
    // documented open forms keep their meaning
    let out = include_site(
        &[("parts/ranged3.md", "l1\nl2\nl3\n")],
        "<!--@include: ./parts/ranged3.md{2,}-->\n",
    );
    assert!(
        out.html.contains("l2") && out.html.contains("l3") && !out.html.contains("l1"),
        "{{2,}} = line 2 to EOF: {out:?}"
    );
    let out = include_site(
        &[("parts/ranged4.md", "l1\nl2\nl3\n")],
        "<!--@include: ./parts/ranged4.md{,2}-->\n",
    );
    assert!(
        out.html.contains("l1") && out.html.contains("l2") && !out.html.contains("l3"),
        "{{,2}} = line 1 to 2: {out:?}"
    );
}

#[test]
fn alert_quote_lazy_continuation_stays_inside() {
    // a wrapped line without `>` is paragraph continuation text inside
    // the quote (CommonMark/GFM lazy continuation), not a paragraph
    // after the container
    let out = synthetic("> [!NOTE]\n> Applies to version 2 and\nlater versions only.\n\nafter\n");
    let note_start = out.html.find("custom-block note").expect("note container");
    let note_close = out.html[note_start..].find("</div>").expect("note closed") + note_start;
    let later = out
        .html
        .find("later versions only.")
        .expect("continuation text kept");
    assert!(
        note_start < later && later < note_close,
        "lazy line stays inside the container"
    );
    // constructs that can interrupt a paragraph still end the quote
    let out = synthetic("> [!NOTE]\n> point\n- a list item\n");
    let note_start = out.html.find("custom-block note").expect("note container");
    let note_close = out.html[note_start..].find("</div>").expect("note closed") + note_start;
    let list = out.html.find("<li>").expect("list rendered");
    assert!(list > note_close, "a list after the quote is not absorbed");
}

#[test]
fn code_include_read_failure_fails_render() {
    // a typo'd <<< target used to ship the literal directive line (a
    // <<< line inside a fence is verbatim by design; the directive form
    // stands alone and generates its own fence)
    let err = match include_site_result(
        &[("keep.txt", "x\n")],
        "before\n\n<<< @/no-such-file.js\n\nafter\n",
    ) {
        Ok(_) => panic!("a missing snippet must fail the render"),
        Err(e) => e,
    };
    assert!(err.to_string().contains("no-such-file"), "{err}");
}

#[test]
fn include_section_miss_fails_render() {
    // a named section that matches nothing must fail, not emit an
    // empty include
    let err = match include_site_result(
        &[("parts/s.md", "## Real\n\nbody\n")],
        "<!--@include: ./parts/s.md#no-such-section-->\n",
    ) {
        Ok(_) => panic!("a section miss must fail the render"),
        Err(e) => e,
    };
    assert!(err.to_string().contains("no-such-section"), "{err}");
}

#[test]
fn broken_theme_toml_fails_engine_init() {
    // a theme file with a TOML syntax error, or a missing `inherits`
    // parent, used to be downgraded to an eprintln and silently
    // produced unstyled code
    let dir = std::env::temp_dir().join(format!("gd-theme-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("bad.toml"), "styles = [broken").unwrap();
    let code = rustpress::config::SyntaxHighlight {
        light: "bad.toml".into(),
        dark: "bad.toml".into(),
    };
    let err = match MarkdownEngine::new(&rustpress::config::Markdown::default(), &code, &dir, "/") {
        Ok(_) => panic!("a broken theme file fails the engine"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains("bad.toml"),
        "error names the theme: {err}"
    );

    // a missing `inherits` parent is the same silent--unstyled class
    std::fs::write(
        dir.join("child.toml"),
        "inherits = \"no-such-parent.toml\"\n",
    )
    .unwrap();
    let code = rustpress::config::SyntaxHighlight {
        light: "child.toml".into(),
        dark: "child.toml".into(),
    };
    let err = match MarkdownEngine::new(&rustpress::config::Markdown::default(), &code, &dir, "/") {
        Ok(_) => panic!("a missing inherits parent fails the engine"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains("no-such-parent"),
        "error names the parent: {err}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn duplicate_custom_heading_ids_are_deduped() {
    // auto slugs dedup ("x", "x-1"); a repeated {#dup} used to emit the
    // same DOM id twice, so the outline linked one heading twice
    let out = synthetic("## A {#dup}\n\ntext\n\n## B {#dup}\n");
    assert_eq!(
        out.html.matches("id=\"dup\"").count(),
        1,
        "exactly one id=\"dup\""
    );
    assert!(
        out.html.contains("id=\"dup-1\""),
        "the second heading got a suffixed id"
    );
}

#[test]
fn inline_footnotes_render() {
    let out = include_site(
        &[],
        "An inline^[careful reader] footnote and code `keep ^[this]` literal.\n",
    );
    // the code-span literal must keep its ^[ marker — the old
    // disjunction passed whenever NO ^[ appeared, pinning nothing
    assert!(
        out.html.contains("keep ^[this]"),
        "code-span literal intact: {}",
        out.html
    );
    assert!(
        out.html.contains("footnote-ref"),
        "became a footnote reference"
    );
    assert!(out.html.contains("careful reader"), "content kept");
}

#[test]
fn link_attribute_blocks() {
    let out = include_site(
        &[],
        "[Home](/guide/){target=\"_self\"}\n\n[Plain](/other/)\n",
    );
    assert!(
        out.html
            .contains(r#"<a href="/guide/" target="_self">Home</a>"#),
        "attrs applied"
    );
    assert!(
        out.html.contains(r#"><a href="/other/">Plain</a>"#)
            || out.html.contains(r#"<a href="/other/">Plain</a>"#),
        "plain link untouched"
    );
}

#[test]
fn link_attribute_blocks_resolve_internal_md_targets() {
    // attrs links are emitted as raw anchors before comrak parses them,
    // so the .md → canonical-URL resolution must still reach them —
    // otherwise a valid link ships dead or fails the dead-link gate
    let site_root = Path::new("tests/fixtures");
    let content_dir = site_root.join("en");
    let mut content = Content::default();
    content.pages.push(Page {
        rel: "guide/routing.md".into(),
        url: "/guide/routing/".into(),
        ..synthetic_page()
    });
    content.by_url = content
        .pages
        .iter()
        .enumerate()
        .map(|(i, p)| (p.url.clone(), i))
        .collect();
    content.by_rel = content
        .pages
        .iter()
        .enumerate()
        .map(|(i, p)| (p.rel.clone(), i))
        .collect();
    let render = |body: &str| {
        let page = Page {
            body: body.to_string(),
            ..synthetic_page()
        };
        engine()
            .render(&page, &content, site_root, &content_dir)
            .expect("render")
    };
    let out = render("[API](./guide/routing.md){target=\"_blank\"}\n");
    assert!(
        out.html
            .contains(r#"<a href="/guide/routing/" target="_blank">API</a>"#),
        "internal .md target resolved: {}",
        out.html
    );
    let out = render("[S](./guide/routing.md#anchor){target=\"_blank\"}\n");
    assert!(
        out.html.contains(r#"href="/guide/routing/#anchor""#),
        "fragment preserved: {}",
        out.html
    );
    let out = render("[G](https://github.com/o/r/README.md){target=\"_blank\"}\n");
    assert!(
        out.html
            .contains(r#"href="https://github.com/o/r/README.md""#),
        "external .md untouched: {}",
        out.html
    );
}

#[test]
fn raw_container_wraps_vp_raw() {
    let out = synthetic("::: raw\n<b>embedded</b>\n:::\n");
    assert!(
        out.html.contains("<div class=\"vp-raw\">"),
        "vp-raw wrapper"
    );
    assert!(out.html.contains("<b>embedded</b>"));
}

#[test]
fn auto_slug_colliding_with_custom_id_dedupes() {
    // an auto slug that collides with an EARLIER custom {#id} must be
    // suffixed, not silently emit the same DOM id twice
    let out = synthetic("## B {#dup}\n\nx\n\n## dup\n");
    assert_eq!(
        out.html.matches("id=\"dup\"").count(),
        1,
        "one id=dup: {}",
        out.html
    );
    let ids: Vec<&str> = out.headings.iter().map(|h| h.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(ids.len(), sorted.len(), "outline ids unique: {ids:?}");
    // forward order keeps working
    let out = synthetic("## dup\n\nx\n\n## B {#dup}\n");
    assert_eq!(
        out.html.matches("id=\"dup\"").count(),
        1,
        "one id=dup: {}",
        out.html
    );
    // duplicate customs still dedupe
    let out = synthetic("## A {#c}\n\n## B {#c}\n");
    assert_eq!(
        out.html.matches("id=\"c\"").count(),
        1,
        "one id=c: {}",
        out.html
    );
}

#[test]
fn fence_on_a_list_marker_line_is_a_fence() {
    // CommonMark: a fence may open on the list-marker line itself
    // ("- ```md"). The preprocessor's fence scanners must track it, or
    // every pass rewrites/expands code-fence content inside it
    let out = synthetic("- ```md\n  ::: tip\n  hi\n  ```\n");
    assert!(
        !out.html.contains("custom-block"),
        "container must stay literal inside the code block: {}",
        out.html
    );
    assert!(
        !out.html.contains("gdcode"),
        "closing fence must not be rewritten: {}",
        out.html
    );
    assert_eq!(
        out.html.matches("<div").count(),
        out.html.matches("</div>").count(),
        "balanced divs: {}",
        out.html
    );
    assert!(out.html.contains("<ul>"), "list kept");
}

#[test]
fn custom_heading_ids_are_attribute_escaped() {
    // a {#custom} id is spliced into id= and href=# attributes by
    // hand-built strings; a quote in it must not break the attribute
    let out = synthetic("## Setup {#a\"b}\n\nbody\n\n[[toc]]\n");
    assert!(
        out.html.contains("id=\"a&quot;b\""),
        "escaped heading id: {}",
        out.html
    );
    assert!(
        !out.html.contains("id=\"a\"b\""),
        "raw quote must not reach the attribute: {}",
        out.html
    );
    assert!(
        out.html.contains("href=\"#a&quot;b\""),
        "escaped toc/anchor href: {}",
        out.html
    );
}

#[test]
fn nested_includes_resolve_against_the_including_file() {
    // upstream resolves a nested relative include against the file that
    // contains it (processIncludes recurses with file = includePath), so
    // parts/a.md's `./b.md` must find parts/b.md — not content/b.md
    let out = include_site(
        &[
            ("parts/a.md", "<!--@include: ./b.md-->\n"),
            ("parts/b.md", "from b\n"),
        ],
        "<!--@include: ./parts/a.md-->\n",
    );
    assert!(
        out.html.contains("from b"),
        "nested include resolved against the including file: {}",
        out.html
    );
}

#[test]
fn shiki_style_line_notations_become_line_classes() {
    // the [!code …] notation path had no direct test: the fixture only
    // ever contains the literal syntax inside md fences, where it must
    // stay untouched
    let out = synthetic(
        "```ts\nconst a = 1; // [!code highlight]\nconst b = 2; // [!code ++]\nconst c = 3; // [!code error:2]\nconst d = 4;\n```\n",
    );
    assert!(
        out.html.contains("<span class=\"line hl\">"),
        "highlight line classed: {}",
        out.html
    );
    assert!(
        out.html.contains("diff add"),
        "++ line classed: {}",
        out.html
    );
    assert!(
        out.html.contains("highlighted-error") || out.html.contains("error"),
        "error line classed: {}",
        out.html
    );
    assert!(
        !out.html.contains("[!code"),
        "notations stripped from output: {}",
        out.html
    );
    // a marker shown as text uses the documented double-bang escape
    // (the transform applies to every fence, matching upstream)
    let out = synthetic("```md\n// [!!code highlight]\n```\n");
    // the md grammar tokenizes the text, so the pieces are asserted
    // separately instead of one contiguous string
    assert!(
        out.html.contains("!code highlight"),
        "escaped notation literal: {}",
        out.html
    );
    assert!(
        !out.html.contains("[!!code"),
        "escape marker removed: {}",
        out.html
    );
    assert!(
        !out.html.contains("line hl"),
        "no highlight class applied to the escaped line: {}",
        out.html
    );

    // an emptied comment loses its marker AND the separating space,
    // like upstream's shiki transformer: `x = 1 // [!code hl]` → `x = 1`
    // (plain-text fence: grammar highlighting interleaves tk-* spans)
    let out = synthetic("```txt\nx = 1 // [!code highlight]\n```\n");
    assert!(
        out.html.contains("<span class=\"line hl\">x = 1</span>"),
        "empty comment shell removed: {}",
        out.html[out.html.find("<code").unwrap()..]
            .chars()
            .take(160)
            .collect::<String>()
    );
}

#[test]
fn unclosed_fence_with_include_cannot_be_broken_out_of() {
    // even with no author closer, the included partial's own closing
    // run must not terminate the (opener-lengthened) fence: the rest
    // of the page stays code, not live markdown
    let out = include_site(
        &[("open.md", "```js\nx\n```\n")],
        "```md\n<!--@include: ./open.md-->\nAFTER\n",
    );
    assert!(
        !out.html.contains("<p>AFTER"),
        "AFTER must stay inside the code block: {}",
        out.html
    );
}

#[test]
fn attrs_values_with_angle_brackets_survive_the_post_pass() {
    // the marked-anchor post-pass stops at the first '>'; attr values
    // must be entity-escaped so a '>' inside a title cannot truncate
    // the tag and leak the rest as text
    let out = include_site(&[], "[t](/other/){title=\"a>b\"}\n");
    assert!(
        out.html.contains("title=\"a&gt;b\""),
        "value escaped: {}",
        out.html
    );
    assert!(!out.html.contains("\"> "), "no leaked markup: {}", out.html);
}

#[test]
fn custom_heading_strip_handles_escaped_id_text() {
    // an id containing & renders as &amp; inside the heading text; the
    // {#…} literal must still be stripped from the visible text
    let out = synthetic("## T {#a&b}\n");
    assert!(
        !out.html.contains("{#a"),
        "literal attribute stripped: {}",
        out.html
    );
}

#[test]
fn attrs_values_keep_authored_entities() {
    // an entity the author already wrote must not double-escape
    // (title="Q&amp;A" used to become Q&amp;amp;A); a bare & still escapes
    let out = include_site(&[], "[t](/other/){title=\"Q&amp;A\"}\n");
    assert!(
        out.html.contains("title=\"Q&amp;A\""),
        "authored entity kept once: {}",
        out.html
    );
    let out = include_site(&[], "[t](/other/){title=\"Q&A\"}\n");
    assert!(
        out.html.contains("title=\"Q&amp;A\""),
        "bare & escaped: {}",
        out.html
    );
}

#[test]
fn list_marker_fence_includes_carry_the_content_indent() {
    // inside a marker-line fence ("- ```md") the block's content column
    // is the list indent: a column-0 inserted line would end the list
    // item — and the fenced block with it — no matter how long the
    // opener marker is (de-indentation, not a closing run)
    let out = include_site(
        &[("fenced3.md", "```js\nx\n```\n")],
        "- ```md\n  <!--@include: ./fenced3.md-->\n  ```\n\nAFTER\n",
    );
    assert!(out.html.contains("<ul>"), "list kept: {}", out.html);
    assert_eq!(
        out.html.matches("<pre").count(),
        1,
        "single code block, no breakout: {}",
        out.html
    );
    // the block sits inside the list item, and the text after the
    // author's closer stays after it
    let li = out.html.find("<li>").expect("list item");
    let li_end = out.html.find("</li>").expect("list item closed");
    let pre = out.html.find("<pre").expect("code block");
    let after = out.html.find("<p>AFTER</p>").expect("AFTER paragraph");
    assert!(
        li < pre && pre < li_end,
        "block inside the item: {}",
        out.html
    );
    assert!(after > li_end, "AFTER after the list: {}", out.html);
}

#[test]
fn unclosed_tilde_fence_with_include_flushes_intact() {
    let out = include_site(
        &[("loose2.md", "~~~js\nx\n~~~\n")],
        "~~~md\n<!--@include: ./loose2.md-->\nstill inside\n",
    );
    assert!(
        out.html.contains("still inside"),
        "body flushed: {}",
        out.html
    );
    assert!(out.html.contains("~~~"), "tilde runs kept as content");
}

#[test]
fn ordered_marker_fences_carry_indent_too() {
    // `10. ` has a four-column content indent: the inserted lines must
    // land at column 4 or they fall out of the item
    let out = include_site(
        &[("fenced5.md", "```js\nx\n```\n")],
        "10. ```md\n    <!--@include: ./fenced5.md-->\n    ```\n\nAFTER\n",
    );
    assert!(out.html.contains("<ol"), "ordered list kept: {}", out.html);
    assert_eq!(
        out.html.matches("<pre").count(),
        1,
        "single code block, no breakout: {}",
        out.html
    );
    let li = out.html.find("<li>").expect("list item");
    let li_end = out.html.find("</li>").expect("list item closed");
    let pre = out.html.find("<pre").expect("code block");
    let after = out.html.find("<p>AFTER</p>").expect("AFTER paragraph");
    assert!(
        li < pre && pre < li_end,
        "block inside the item: {}",
        out.html
    );
    assert!(after > li_end, "AFTER after the list: {}", out.html);
}

#[test]
fn unclosed_list_marker_fence_flushes_with_indent() {
    // pad + EOF flush combined: an unclosed marker-line fence whose
    // include carries a closer must still contain the whole body
    let out = include_site(
        &[("open3.md", "```js\nx\n```\n")],
        "- ```md\n  <!--@include: ./open3.md-->\n",
    );
    assert!(out.html.contains("<ul>"), "list kept: {}", out.html);
    assert!(out.html.contains(">x<"), "include content present as code");
    assert!(!out.html.contains("<p>x"), "x must not become a paragraph");
    // structural: exactly one block, and it stays inside the list item
    // (without the pad these fail: the injected closer ends the item)
    assert_eq!(
        out.html.matches("<pre").count(),
        1,
        "single code block: {}",
        out.html
    );
    let li = out.html.find("<li>").expect("list item");
    let li_end = out.html.find("</li>").expect("list item closed");
    let pre = out.html.find("<pre").expect("code block");
    assert!(
        li < pre && pre < li_end,
        "block inside the item: {}",
        out.html
    );
}

#[test]
fn container_titles_render_inline_markdown() {
    // upstream renders `:::` titles with md.renderInline (code, emphasis,
    // emoji); a title that is not inline content stays literal, and
    // GitHub alert titles stay plain text — all checked against the
    // pinned upstream build
    let out = synthetic(concat!(
        "::: details Example Netlify `_headers` file\nbody\n:::\n\n",
        "::: warning Avoid `<style scoped>` in **Markdown**\nbody\n:::\n\n",
        "::: tip Celebrate :tada:\nbody\n:::\n\n",
        "::: info # not a heading\nbody\n:::\n\n",
        "::: danger 1. not a list\nbody\n:::\n\n",
        "> [!NOTE] Alert with `code` & <b>\n> body\n",
    ));
    let h = &out.html;
    for want in [
        "<summary>Example Netlify <code>_headers</code> file</summary>",
        "<p class=\"custom-block-title\">Avoid <code>&lt;style scoped&gt;</code> in <strong>Markdown</strong></p>",
        "<p class=\"custom-block-title\">Celebrate 🎉</p>",
        "<p class=\"custom-block-title\"># not a heading</p>",
        "<p class=\"custom-block-title\">1. not a list</p>",
        "<p class=\"custom-block-title\">Alert with `code` &amp; &lt;b&gt;</p>",
    ] {
        assert!(h.contains(want), "missing {want}\n{h}");
    }
    assert!(!h.contains("data-gd-inline"), "marker leaked: {h}");
}

#[test]
fn footnotes_take_upstreams_shape() {
    // @mdit/plugin-footnote's output (what upstream renders), from a
    // probe of the pinned build: `[n]` / `[n:k]` reference text, ids by
    // number not label, the list classes, and `↩︎` backrefs — U+FE0E
    // keeps the arrow a text glyph, where a bare U+21A9 can draw as an
    // emoji on Apple platforms
    let out = synthetic(concat!(
        "First[^1], again[^1], named[^note], inline^[Inline body.].\n\n",
        "[^1]: One.\n",
        "[^note]: Named.\n",
    ));
    let h = &out.html;
    for want in [
        "First<sup class=\"footnote-ref\"><a href=\"#footnote1\">[1]</a><a class=\"footnote-anchor\" id=\"footnote-ref1\"></a></sup>",
        "again<sup class=\"footnote-ref\"><a href=\"#footnote1\">[1:1]</a><a class=\"footnote-anchor\" id=\"footnote-ref1:1\"></a></sup>",
        "named<sup class=\"footnote-ref\"><a href=\"#footnote2\">[2]</a><a class=\"footnote-anchor\" id=\"footnote-ref2\"></a></sup>",
        "inline<sup class=\"footnote-ref\"><a href=\"#footnote3\">[3]</a><a class=\"footnote-anchor\" id=\"footnote-ref3\"></a></sup>",
        "<hr class=\"footnotes-sep\"><section class=\"footnotes\"><ol class=\"footnotes-list\">",
        "<li id=\"footnote1\" class=\"footnote-item\">",
        "<li id=\"footnote2\" class=\"footnote-item\">",
        "<li id=\"footnote3\" class=\"footnote-item\">",
        "One. <a href=\"#footnote-ref1\" class=\"footnote-backref\">\u{21a9}\u{fe0e}</a> <a href=\"#footnote-ref1:1\" class=\"footnote-backref\">\u{21a9}\u{fe0e}</a></p>",
        "Named. <a href=\"#footnote-ref2\" class=\"footnote-backref\">\u{21a9}\u{fe0e}</a></p>",
        "Inline body. <a href=\"#footnote-ref3\" class=\"footnote-backref\">\u{21a9}\u{fe0e}</a></p>",
    ] {
        assert!(h.contains(want), "missing {want}\n{h}");
    }
    // no comrak-shaped leftovers point at ids that no longer exist
    assert!(!h.contains("fnref-") && !h.contains("id=\"fn-"), "{h}");
}

#[test]
fn task_list_checkboxes_are_labelled() {
    // upstream (@mdit/plugin-tasklist) wraps each item's text in a
    // <label for> its checkbox — an unlabelled checkbox fails Lighthouse's
    // "form elements have labels" audit. Shape from a probe of the pinned
    // build: page-wide task-item-N ids, the label holding the first
    // paragraph's inline content (inside the <p> of a loose list), and
    // task-list-container on the list
    let out = synthetic(concat!(
        "- [ ] Plain item\n",
        "- [x] Item with `code`\n",
        "- [ ] Parent item\n",
        "  - [x] Nested child\n",
        "\nLoose list:\n\n",
        "- [ ] Loose one\n\n",
        "- [x] Loose two\n",
    ));
    let h: String = out.html.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        "<ul class=\"task-list-container\"> <li class=\"task-list-item\"><input type=\"checkbox\" class=\"task-list-item-checkbox\" id=\"task-item-0\" disabled=\"disabled\"><label class=\"task-list-item-label\" for=\"task-item-0\"> Plain item</label></li>",
        "<input type=\"checkbox\" class=\"task-list-item-checkbox\" id=\"task-item-1\" checked=\"checked\" disabled=\"disabled\"><label class=\"task-list-item-label\" for=\"task-item-1\"> Item with <code>code</code></label></li>",
        "<label class=\"task-list-item-label\" for=\"task-item-2\"> Parent item</label> <ul class=\"task-list-container\">",
        "id=\"task-item-3\" checked=\"checked\" disabled=\"disabled\"><label class=\"task-list-item-label\" for=\"task-item-3\"> Nested child</label>",
        "<li class=\"task-list-item\"> <p><input type=\"checkbox\" class=\"task-list-item-checkbox\" id=\"task-item-4\" disabled=\"disabled\"><label class=\"task-list-item-label\" for=\"task-item-4\"> Loose one</label></p>",
        "<p><input type=\"checkbox\" class=\"task-list-item-checkbox\" id=\"task-item-5\" checked=\"checked\" disabled=\"disabled\"><label class=\"task-list-item-label\" for=\"task-item-5\"> Loose two</label></p>",
    ] {
        assert!(h.contains(want), "missing {want}\n{h}");
    }
    assert!(!h.contains("contains-task-list"), "{h}");
}
