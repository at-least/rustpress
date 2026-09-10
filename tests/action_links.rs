//! Home-page hero actions and feature cards link to pages the same way
//! markdown content does: a link naming a page resolves to that page's
//! directory-style URL (with the base applied), whether written as
//! `./guide/a`, `guide/a`, `/guide/a`, or `/guide/a.md`.

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

fn hrefs(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in html.match_indices("href=\"") {
        let rest = &html[i + 6..];
        out.push(rest[..rest.find('"').unwrap()].to_string());
    }
    out
}

fn build(base: &str) -> String {
    let site = temp_site(&format!("actions{}", base.trim_matches('/')));
    write(
        &site,
        "rustpress.toml",
        &format!("title = \"probe\"\nbase = \"{base}\"\n"),
    );
    write(
        &site,
        "content/index.md",
        "---\nlayout: home\nhero:\n  name: N\n  text: T\n  actions:\n    \
         - text: rel\n      link: ./guide/a\n    \
         - text: bare\n      link: guide/a\n    \
         - text: abs\n      link: /guide/a\n    \
         - text: md\n      link: /guide/a.md\n    \
         - text: file\n      link: /pure.html\n    \
         - text: ext\n      link: https://example.com/x\n\
         features:\n  - title: F\n    details: D\n    link: ./guide/a\n---\n",
    );
    // the home layout renders no markdown body, so content links live on a page
    write(
        &site,
        "content/guide/a.md",
        "# A\n\n[abs content](/guide/a)\n[md content](/guide/a.md)\n",
    );
    write(&site, "static/pure.html", "<p>plain</p>");
    let out = site.join("public");
    Site::load(&site).unwrap().build(&site, &out).unwrap();
    let mut html = std::fs::read_to_string(out.join("index.html")).unwrap();
    let a = std::fs::read_to_string(out.join("guide/a/index.html")).unwrap();
    html.push_str(&a[a.find("class=\"vp-doc").unwrap()..]);
    let _ = std::fs::remove_dir_all(&site);
    html
}

#[test]
fn action_links_resolve_to_page_urls() {
    let got = hrefs(&build("/"));
    let page = got.iter().filter(|h| *h == "/guide/a/").count();
    // 4 actions + 1 feature card + 2 content links on guide/a
    assert_eq!(
        page, 7,
        "expected every page link as /guide/a/, got {got:#?}"
    );
    for bad in ["/guide/a", "/guide/a.md"] {
        assert!(
            !got.iter().any(|h| h == bad),
            "unnormalised {bad} in {got:#?}"
        );
    }
    assert!(
        got.iter().any(|h| h == "/pure.html"),
        "static file link kept: {got:#?}"
    );
    assert!(got.iter().any(|h| h == "https://example.com/x"), "{got:#?}");
}

#[test]
fn action_links_get_the_base() {
    let got = hrefs(&build("/docs/"));
    let page = got.iter().filter(|h| *h == "/docs/guide/a/").count();
    assert_eq!(
        page, 7,
        "expected every page link as /docs/guide/a/, got {got:#?}"
    );
    assert!(got.iter().any(|h| h == "/docs/pure.html"), "{got:#?}");
    assert!(
        !got.iter().any(|h| h == "/guide/a/" || h == "/guide/a"),
        "base-free link in {got:#?}"
    );
}
