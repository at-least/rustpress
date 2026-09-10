//! `static/` is embedded into the binary (see `src/theme_assets.rs`).
//! Recompile when it changes, and refuse to build without the two files
//! that npm produces — otherwise `cargo install` / `cargo publish` would
//! ship a binary whose sites have no stylesheet or script.

fn main() {
    println!("cargo:rerun-if-changed=static");
    println!("cargo:rerun-if-changed=build.rs");
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let missing: Vec<&str> = ["static/main.css", "static/js/app.js"]
        .into_iter()
        .filter(|rel| !manifest.join(rel).is_file())
        .collect();
    if !missing.is_empty() {
        panic!(
            "rustpress: theme assets {} are missing; run `npm install && npm run build:js && npm run build:css` first (they are embedded into the binary)",
            missing.join(", ")
        );
    }
}
