//! The theme's built assets (vitepress.css, js/app.js, fonts) are embedded in
//! the binary, so a site that lives anywhere — not next to the repo's
//! static/ — still builds into a complete, styled site.

use rustpress::render::Site;

fn temp_site(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rustpress-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("content")).unwrap();
    dir
}

#[test]
fn site_outside_the_repo_gets_the_embedded_theme_assets() {
    let site = temp_site("embedded");
    std::fs::write(site.join("rustpress.toml"), "title = \"probe\"\n").unwrap();
    std::fs::write(site.join("content/index.md"), "# Hello\n").unwrap();
    assert!(
        !site.parent().unwrap().join("static").is_dir(),
        "temp dir must not have a sibling static/ or the test proves nothing"
    );

    let out = site.join("public");
    Site::load(&site).unwrap().build(&site, &out).unwrap();

    for rel in ["vitepress.css", "js/app.js"] {
        let file = out.join(rel);
        assert!(
            file.is_file(),
            "missing embedded theme asset {}",
            file.display()
        );
        assert!(
            std::fs::metadata(&file).unwrap().len() > 0,
            "embedded theme asset {rel} is empty"
        );
    }
    let fonts = out.join("fonts");
    let woff2 = std::fs::read_dir(&fonts)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .filter(|e| e.path().extension().is_some_and(|x| x == "woff2"))
                .count()
        })
        .unwrap_or(0);
    assert!(woff2 > 0, "no .woff2 fonts under {}", fonts.display());
    let _ = std::fs::remove_dir_all(&site);
}

#[test]
fn embedded_fonts_are_the_ones_vitecss_names() {
    // the @font-face rules come from vitecss (fonts.css) while the files
    // they name are embedded from static/fonts/ — a copy of vitecss's
    // fonts/, which must not drift from it
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ours = root.join("static/fonts");
    let theirs = root.join("node_modules/vitecss/fonts");
    let names = |dir: &std::path::Path| {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("read {dir:?}: {e}"))
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    };
    assert_eq!(
        names(&ours),
        names(&theirs),
        "static/fonts/ and vitecss's fonts/ hold different files"
    );
    for name in names(&ours) {
        assert!(
            std::fs::read(ours.join(&name)).unwrap() == std::fs::read(theirs.join(&name)).unwrap(),
            "static/fonts/{name} differs from vitecss's copy"
        );
    }
}

#[test]
fn site_static_overrides_the_embedded_theme_assets() {
    let site = temp_site("override");
    std::fs::write(site.join("rustpress.toml"), "title = \"probe\"\n").unwrap();
    std::fs::write(site.join("content/index.md"), "# Hello\n").unwrap();
    std::fs::create_dir_all(site.join("static")).unwrap();
    std::fs::write(site.join("static/vitepress.css"), "/* site override */\n").unwrap();

    let out = site.join("public");
    Site::load(&site).unwrap().build(&site, &out).unwrap();

    let css = std::fs::read_to_string(out.join("vitepress.css")).unwrap();
    assert_eq!(css, "/* site override */\n");
    assert!(
        out.join("js/app.js").is_file(),
        "the rest of the theme is still embedded"
    );
    let _ = std::fs::remove_dir_all(&site);
}
