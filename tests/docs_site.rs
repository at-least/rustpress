//! The dogfood documentation site under `docs/` must always build: a
//! passing build proves every config key is known, every page renders,
//! and no internal link is dead. On top of the build's own dead-link
//! check, this test validates `#fragment` targets, which the build does
//! not (it ignores fragments).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use rustpress::render::Site;

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn tempdir() -> TempDir {
    let dir = std::env::temp_dir().join(format!(
        "rustpress-docs-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    TempDir(dir)
}

fn markdown_files(dir: &Path) -> usize {
    let mut n = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            n += markdown_files(&path);
        } else if path.extension().is_some_and(|e| e == "md") {
            n += 1;
        }
    }
    n
}

fn html_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            html_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "html") {
            out.push(path);
        }
    }
}

fn attr_values<'a>(html: &'a str, attr: &str) -> Vec<&'a str> {
    let needle = format!("{attr}=\"");
    html.match_indices(&needle)
        .filter_map(|(i, _)| html[i + needle.len()..].split('"').next())
        .collect()
}

#[test]
fn docs_site_builds_with_every_page_and_valid_anchors() {
    let site_dir = Path::new("docs");
    let site = Site::load(site_dir).expect("docs/rustpress.toml + content load");
    let out = tempdir();
    let stats = site
        .build(site_dir, &out.0)
        .expect("docs build (dead links fail here)");

    let expected = markdown_files(&site_dir.join("content"));
    assert_eq!(
        stats.pages, expected,
        "every markdown file under docs/content is a page"
    );
    assert!(stats.sitemap, "docs config enables [sitemap]");
    assert!(
        out.0.join("search-docs.json").is_file(),
        "docs config enables local search"
    );

    // fragment validation: url → set of element ids, then every internal
    // href with a fragment must name an id on its target page
    let mut files = Vec::new();
    html_files(&out.0, &mut files);
    let mut ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut pages: Vec<(String, String)> = Vec::new();
    for file in &files {
        let rel = file
            .strip_prefix(&out.0)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let url = match rel.strip_suffix("index.html") {
            Some(dir) => format!("/{dir}"),
            None => format!("/{rel}"),
        };
        let html = std::fs::read_to_string(file).unwrap();
        ids.insert(
            url.clone(),
            attr_values(&html, "id")
                .into_iter()
                .map(str::to_string)
                .collect(),
        );
        pages.push((url, html));
    }
    let mut broken = Vec::new();
    for (url, html) in &pages {
        for href in attr_values(html, "href") {
            let Some((path, frag)) = href.split_once('#') else {
                continue;
            };
            if frag.is_empty() || path.starts_with("http") {
                continue;
            }
            // theme chrome, not content: the skip link targets the doc
            // layout's `#main`, which the home layout does not render
            if href == "#main" {
                continue;
            }
            let target = if path.is_empty() {
                url.clone()
            } else {
                path.to_string()
            };
            match ids.get(&target) {
                Some(set) if set.contains(frag) => {}
                Some(_) => broken.push(format!("{url} → {href} (no such id on target)")),
                None => broken.push(format!("{url} → {href} (no such page)")),
            }
        }
    }
    assert!(
        broken.is_empty(),
        "broken fragment links:\n{}",
        broken.join("\n")
    );
}
