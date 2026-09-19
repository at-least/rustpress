use comrak::adapters::CodefenceRendererAdapter;
use rustpress::markdown::highlight::GdCodeRenderer;

#[test]
fn scope_same_line_open_close() {
    // opens and closes within one line: no carry
    let code = "let x = 1;\n";
    let mut out = String::new();
    GdCodeRenderer::default()
        .write(&mut out, "gdcode", "lang=js", code, None)
        .unwrap();
    let body = out.split("<code").nth(1).unwrap();
    let body = &body[..body.find("</code>").unwrap()];
    assert_eq!(
        body.matches("<span").count(),
        body.matches("</span>").count()
    );
}

#[test]
fn last_line_without_newline() {
    let mut out = String::new();
    GdCodeRenderer::default()
        .write(&mut out, "gdcode", "lang=js", "let x = 1;", None)
        .unwrap();
    assert!(out.contains("<span class=\"line\">"));
}

#[test]
fn empty_lines_inside_carried_scope() {
    let code = "/* c\n\n\nd */\nlet x = 1;\n";
    let mut out = String::new();
    GdCodeRenderer::default()
        .write(&mut out, "gdcode", "lang=js", code, None)
        .unwrap();
    let body = out.split("<code").nth(1).unwrap();
    let body = &body[..body.find("</code>").unwrap()];
    assert_eq!(
        body.matches("<span").count(),
        body.matches("</span>").count()
    );
}

// NOTE: the traversal probes that used to live here mirrored
// serve_file's segment loop instead of testing it — they stayed green
// even if the real resolver drifted. The real coverage now lives in
// src/serve.rs (serves_files_and_rejects_traversal, plus the redirect
// test), which exercises the actual server path.

#[test]
fn empty_code_with_grammar_renders_no_lines() {
    // a placeholder ```js fence with no content used to panic with
    // "index out of bounds: the len is 0 but the index is 0": the
    // phantom first-line start survived the retain on an empty source
    // and indexed the empty notation_classes
    let mut out = String::new();
    GdCodeRenderer::default()
        .write(&mut out, "gdcode", "lang=js", "", None)
        .unwrap();
    assert!(out.contains("<code"), "still a code block: {out:?}");
    assert!(
        !out.contains("<span class=\"line\">"),
        "an empty source has no lines: {out:?}"
    );

    // same for an empty fence carrying hl / line-number specs
    let mut out = String::new();
    GdCodeRenderer::default()
        .write(&mut out, "gdcode", "lang=js hl=1 ln=true", "", None)
        .unwrap();
    assert!(
        !out.contains("<span class=\"line\">"),
        "specs change nothing on an empty source: {out:?}"
    );
}

/// `.line` is an inline-block inside a `white-space: pre` container, so
/// lines only stack when the newline sits BETWEEN the spans (Shiki's
/// layout); a newline inside the span leaves every line on one row.
#[test]
fn newlines_sit_between_line_spans() {
    for (lang, code) in [("toml", "a = 1\nb = 2\n"), ("nosuchlang", "a = 1\nb = 2\n")] {
        let mut out = String::new();
        GdCodeRenderer::default()
            .write(&mut out, "gdcode", &format!("lang={lang}"), code, None)
            .unwrap();
        let body = out.split("<code").nth(1).unwrap();
        let body = &body[..body.find("</code>").unwrap()];
        assert!(
            body.contains("</span>\n<span class=\"line\">"),
            "{lang}: newline between lines: {body:?}"
        );
        assert!(
            !body.contains("\n</span>"),
            "{lang}: newline inside a line span: {body:?}"
        );
        assert!(
            !body.ends_with('\n'),
            "{lang}: trailing newline before </code>: {body:?}"
        );
        assert_eq!(
            body.matches("<span class=\"line\">").count(),
            2,
            "{lang}: {body:?}"
        );
    }
}

#[test]
fn theme_color_values_must_be_css_hex() {
    // theme files are third-party data interpolated straight into
    // syntax.css; an off-hex value must fail the theme load, not reach
    // the emitted stylesheet
    let dir = std::env::temp_dir().join(format!("gd-theme-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let theme = dir.join("evil.toml");
    std::fs::write(
        &theme,
        "theme = \"evil\"\n\n[palette]\nred0 = \"#fff; } html { display:none } x{ color:#000\"\n\n[\"keyword\"]\nfg = \"red0\"\n",
    )
    .unwrap();
    let err = rustpress::markdown::syntax_theme::load(&theme).unwrap_err();
    assert!(
        err.to_string().contains("invalid color"),
        "junk palette value must fail the load: {err}"
    );

    // hex literals get the same gate
    std::fs::write(
        &theme,
        "theme = \"evil\"\n\n[\"keyword\"]\nfg = \"#fff; } html { display:none } x{ color:#000\"\n",
    )
    .unwrap();
    let err = rustpress::markdown::syntax_theme::load(&theme).unwrap_err();
    assert!(
        err.to_string().contains("invalid color"),
        "junk literal must fail the load: {err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
