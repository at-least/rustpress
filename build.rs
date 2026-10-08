//! `static/` is embedded into the binary (see `src/theme_assets.rs`), and
//! so are vpkit's color themes and Helix palettes (`node_modules/vpkit`,
//! an npm `file:` dependency on ../vpkit). Recompile when any of them
//! changes, and refuse to build without them — otherwise `cargo install` /
//! `cargo publish` would ship a binary whose sites have no stylesheet,
//! script, themes or code colors.

fn main() {
    println!("cargo:rerun-if-changed=static");
    println!("cargo:rerun-if-changed=node_modules/vpkit/themes");
    println!("cargo:rerun-if-changed=node_modules/vpkit/helix");
    println!("cargo:rerun-if-changed=build.rs");
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let missing: Vec<&str> = ["static/vitepress.css", "static/js/app.js"]
        .into_iter()
        .filter(|rel| !manifest.join(rel).is_file())
        .collect();
    if !missing.is_empty() {
        panic!(
            "rustpress: theme assets {} are missing; run `npm install && npm run build:js && npm run build:css` first (they are embedded into the binary)",
            missing.join(", ")
        );
    }
    let missing: Vec<&str> = ["node_modules/vpkit/themes", "node_modules/vpkit/helix"]
        .into_iter()
        .filter(|rel| !manifest.join(rel).is_dir())
        .collect();
    if !missing.is_empty() {
        panic!(
            "rustpress: {} missing; clone https://github.com/at-least/vpkit next to this repository and run `npm install` (they are embedded into the binary)",
            missing.join(", ")
        );
    }
}
