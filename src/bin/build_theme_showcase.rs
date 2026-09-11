// Build the theme showcase into docs/public/themes/.
//
// The showcase site (examples/theme-showcase/) is built twice per palette:
// once at the gallery root with `theme` unset (the stock VitePress look),
// then once per palette into `themes/<name>/` with a `theme = "<name>"`
// line injected into a copy of the config. A `palettes.json` index (name →
// override CSS) is written next to the root index for the gallery's
// Alpine `themePreview` component to fetch.
//
// Syntax previews are NOT rebuilt per scheme: 218 Helix themes × one
// full build each would be too slow, and `[syntax]` changes only the
// emitted `syntax.css`, which any page can hot-swap via Alpine.
//
// Usage: cargo run --quiet --bin build_theme_showcase
//        (from the repo root; writes into docs/public/themes/)

use std::collections::BTreeMap;
use std::path::PathBuf;

use rustpress::palettes;
use rustpress::render::Site;

fn main() -> anyhow::Result<()> {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let showcase = repo.join("examples/theme-showcase");
    let out_root = repo.join("docs/public/themes");

    if out_root.exists() {
        std::fs::remove_dir_all(&out_root)?;
    }
    std::fs::create_dir_all(&out_root)?;

    // 1. The gallery home page: no theme, palette index next to it.
    let site = Site::load(&showcase)?;
    let mut index: BTreeMap<&str, &str> = BTreeMap::new();
    for p in palettes::PALETTES {
        index.insert(p.name, p.css);
    }
    let json = serde_json::to_string_pretty(&index)?;
    site.build_with_extra_assets(&showcase, &out_root, &[("palettes.json", json.as_bytes())])?;

    // 2. One rebuild per palette into themes/<name>/.
    let config_toml = std::fs::read_to_string(showcase.join("rustpress.toml"))?;
    for p in palettes::PALETTES {
        let themed = showcase.with_file_name(format!("theme-showcase-{}", p.name));
        std::fs::create_dir_all(&themed)?;
        copy_dir_contents(&showcase, &themed)?;
        // The theme key must be a top-level key — inserting before the
        // first `[section]` header keeps it there (appending would land
        // inside the last table).
        let mut themed_toml = config_toml.clone();
        let insert_at = themed_toml
            .find("\n[")
            .map(|i| i + 1)
            .unwrap_or(themed_toml.len());
        themed_toml.insert_str(insert_at, &format!("theme = \"{}\"\n\n", p.name));
        std::fs::write(themed.join("rustpress.toml"), &themed_toml)?;

        // Asset URLs in generated HTML are base-absolute (`/main.css`,
        // `/theme.css`). Serving the palette build one level below the
        // showcase root makes "../" = the root, so prefixing every asset
        // URL with "../" resolves correctly. The same rewrite skips
        // content links (already rewritten above) and the gallery's
        // injected <script>/<link> hrefs (already "/themes/…"-prefixed).
        let index_md = std::fs::read_to_string(themed.join("content/index.md"))?;
        let index_md = index_md
            .replace("./themes/", "../")
            .replace("href=\"./\" aria-current=\"page\"", "href=\"../../\"")
            .replace(" aria-current=\"page\"", "");
        std::fs::write(themed.join("content/index.md"), index_md)?;

        let site = Site::load(&themed)?;
        let out = out_root.join("themes").join(p.name);
        site.build(&themed, &out)?;
        // `main.css`, `syntax.css`, `theme.css` and `js/app.js` inside
        // every HTML page are generated with site.url() — base is "/" on
        // the showcase, so they come out as `/main.css` etc. Under the
        // nested URL path that 404s, so rewrite them two levels up. The
        // palette's own `theme.css` (the macchiato colors) is written to
        // the palette's own directory and is the one asset that must be
        // RELATIVE to the page, so "./theme.css" instead.
        // Content links are unaffected (they were rewritten to ../… above).
        let index_html = out.join("index.html");
        let html = std::fs::read_to_string(&index_html)?;
        let html = html
            .replace("href=\"/main.css\"", "href=\"../../main.css\"")
            .replace("href=\"/syntax.css\"", "href=\"../../syntax.css\"")
            .replace("href=\"/theme.css\"", "href=\"./theme.css\"")
            .replace("src=\"/js/app.js\"", "src=\"../../js/app.js\"")
            .replace("href=\"/search-docs.json\"", "href=\"../../search-docs.json\"")
            // frontmatter-injected static assets (see the site
            // config's own [[head]] rules): one `..` for the palette
            // subdirectory, i.e. /theme-gallery.* → ../../theme-gallery.*
            .replace("href=\"/theme-gallery", "href=\"../../theme-gallery")
            .replace("src=\"/theme-gallery", "src=\"../../theme-gallery");
        std::fs::write(&index_html, html)?;
        std::fs::remove_dir_all(&themed)?;
    }

    println!(
        "showcase: home + {} palettes → docs/public/themes/",
        palettes::PALETTES.len()
    );
    Ok(())
}

fn copy_dir_contents(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_dir_contents(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
