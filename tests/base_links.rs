//! With a non-root `base`, links and images written in markdown content
//! get the base prefix the same way VitePress's link plugin applies it:
//! every internal URL that starts with `/` (relative page links are
//! resolved to such URLs first) is joined with the base; external,
//! anchor-only and mailto links, and raw HTML, are left alone.

use std::path::{Path, PathBuf};

use rustpress::render::Site;

fn temp_site(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rustpress-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("content/guide")).unwrap();
    std::fs::create_dir_all(dir.join("static")).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, body: &str) {
    std::fs::write(dir.join(rel), body).unwrap();
}

fn attrs(html: &str) -> Vec<String> {
    // every href="…"/src="…" inside the rendered markdown body
    let start = html.find("class=\"vp-doc").unwrap_or(0);
    let body = &html[start..];
    let mut out = Vec::new();
    for key in ["href=\"", "src=\""] {
        for (i, _) in body.match_indices(key) {
            let rest = &body[i + key.len()..];
            let end = rest.find('"').unwrap();
            out.push(format!("{key}{}\"", &rest[..end]));
        }
    }
    out
}

fn probe_site(base: &str, ignore_dead_links: bool, protocol_relative: bool) -> PathBuf {
    let site = temp_site(&format!("base{}", base.trim_matches('/')));
    write(
        &site,
        "rustpress.toml",
        &format!("title = \"probe\"\nbase = \"{base}\"\nignoreDeadLinks = {ignore_dead_links}\n"),
    );
    let mut home = String::from(
        "# Home\n\n\
         [rel](./guide/a)\n\
         [abs](/guide/a/)\n\
         [abs b](/guide/b/)\n\
         [file](/pure.html){target=\"_self\"}\n\
         [frag](/guide/a/#top)\n\
         ![img](/pic.png)\n\
         ![rel img](./local.png)\n\
         <a href=\"/pure.html\">raw</a>\n\
         [ext](https://example.com/x)\n\
         [anchor](#home)\n\
         [mail](mailto:a@b.c)\n",
    );
    if protocol_relative {
        // the dead-link checker currently treats `//host/…` as internal
        // (a separate issue), so only the prefix test carries this one
        home.push_str("[proto](//cdn.example.com/y)\n");
    }
    write(&site, "content/index.md", &home);
    write(
        &site,
        "content/guide/a.md",
        "# A\n\n[up](../)\n[sib](./b.md)\n",
    );
    write(&site, "content/guide/b.md", "# B\n");
    write(&site, "static/pic.png", "");
    write(&site, "static/local.png", "");
    write(&site, "static/pure.html", "<p>plain</p>");
    site
}

#[test]
fn content_links_and_images_get_the_base_prefix() {
    let site = probe_site("/docs/", true, true);
    let out = site.join("public");
    Site::load(&site).unwrap().build(&site, &out).unwrap();

    let home = std::fs::read_to_string(out.join("index.html")).unwrap();
    let got = attrs(&home);
    for expected in [
        "href=\"/docs/guide/a/\"",
        "href=\"/docs/guide/b/\"",
        "href=\"/docs/pure.html\"",
        "href=\"/docs/guide/a/#top\"",
        "src=\"/docs/pic.png\"",
        // untouched
        "src=\"./local.png\"",
        "href=\"/pure.html\"",
        "href=\"https://example.com/x\"",
        "href=\"//cdn.example.com/y\"",
        "href=\"#home\"",
        "href=\"mailto:a@b.c\"",
    ] {
        assert!(
            got.contains(&expected.to_string()),
            "missing {expected} in {got:#?}"
        );
    }
    for forbidden in ["href=\"/guide/a/\"", "src=\"/pic.png\""] {
        assert!(
            !got.contains(&forbidden.to_string()),
            "base-free {forbidden} in {got:#?}"
        );
    }
    // raw HTML is not rewritten (VitePress does not either), so exactly one
    // base-free `/pure.html` remains: the raw `<a>`; the markdown link got the base
    let raw = got.iter().filter(|a| *a == "href=\"/pure.html\"").count();
    assert_eq!(raw, 1, "expected one untouched raw href, got {got:#?}");

    let a = std::fs::read_to_string(out.join("guide/a/index.html")).unwrap();
    let got = attrs(&a);
    assert!(
        got.contains(&"href=\"/docs/\"".to_string()),
        "`../` → base root, got {got:#?}"
    );
    assert!(
        got.contains(&"href=\"/docs/guide/b/\"".to_string()),
        "got {got:#?}"
    );
    let _ = std::fs::remove_dir_all(&site);
}

#[test]
fn root_base_output_is_unchanged() {
    let site = probe_site("/", false, false);
    let out = site.join("public");
    Site::load(&site).unwrap().build(&site, &out).unwrap();
    let home = std::fs::read_to_string(out.join("index.html")).unwrap();
    let got = attrs(&home);
    for expected in [
        "href=\"/guide/a/\"",
        "href=\"/pure.html\"",
        "src=\"/pic.png\"",
    ] {
        assert!(
            got.contains(&expected.to_string()),
            "missing {expected} in {got:#?}"
        );
    }
    let _ = std::fs::remove_dir_all(&site);
}

#[test]
fn dead_link_check_understands_base_prefixed_links() {
    // the probe above builds with ignoreDeadLinks = false: every
    // base-prefixed link must still be recognised as alive …
    let site = probe_site("/docs/", false, false);
    let out = site.join("public");
    Site::load(&site).unwrap().build(&site, &out).unwrap();
    // … and a genuinely missing page must still fail the build
    write(
        &site,
        "content/guide/b.md",
        "# B\n\n[nope](/guide/missing/)\n",
    );
    let err = Site::load(&site)
        .unwrap()
        .build(&site, &out)
        .expect_err("a dead link must fail the build");
    let msg = format!("{err:#}");
    assert!(msg.contains("dead link"), "unexpected error: {msg}");
    assert!(msg.contains("/guide/missing"), "unexpected error: {msg}");
    let _ = std::fs::remove_dir_all(&site);
}
