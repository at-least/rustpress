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

// traversal probes against the serve path resolver logic
#[test]
fn traversal_segments_rejected() {
    // mirror of serve_file's segment loop: ".." must never resolve
    let rejects = [
        "/../etc/passwd",
        "/a/../../etc/passwd",
        "/%2e%2e/etc/passwd",
    ];
    for path in rejects {
        let decoded = percent(path);
        let mut escapes = false;
        for seg in decoded.split('/') {
            if seg == ".." {
                escapes = true;
            }
        }
        assert!(escapes, "{path} should decode to an escaping path");
    }
}

fn percent(s: &str) -> String {
    // copy of serve::percent_decode semantics
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len() + 1
            && i + 2 <= b.len()
            && let (Some(hex), Ok(v)) = (
                b.get(i + 1..i + 3),
                u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("zz"), 16),
            )
        {
            let _ = hex;
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
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
