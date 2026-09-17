//! Shared test helpers: the upstream-clone policy plus the tempdir used
//! by the build tests. Each integration binary compiles its own copy of
//! this module and may not use every helper, hence the allow.

use std::path::PathBuf;

/// Not every test binary that includes this module uses every helper.
#[allow(dead_code)]
pub fn upstream_docs() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("vitepress/docs/en")
}

/// `Some(clone dir)` when present; `None` for an explicit, visible skip
/// (only when `RUSTPRESS_ALLOW_NO_UPSTREAM=1` is set); panics with the
/// actionable fix when the clone is absent and not opted out.
#[allow(dead_code)]
pub fn ensure_upstream() -> Option<PathBuf> {
    let docs = upstream_docs();
    if docs.is_dir() {
        return Some(docs);
    }
    if std::env::var_os("RUSTPRESS_ALLOW_NO_UPSTREAM").is_some() {
        eprintln!(
            "skipping upstream gate: ../vitepress/docs/en not found \
             (RUSTPRESS_ALLOW_NO_UPSTREAM is set)"
        );
        return None;
    }
    panic!(
        "../vitepress/docs/en not found — the upstream parity gates need \
         the pinned clone.\n  run: bash scripts/sync-upstream.sh\n  \
         (or set RUSTPRESS_ALLOW_NO_UPSTREAM=1 to skip upstream gates on \
         this machine)"
    )
}

/// Minimal dev-dependency-free tempdir shared by the integration tests:
/// keyed by pid + thread + nanos (tests run on parallel threads of one
/// process), wiped on drop.
#[allow(dead_code)]
pub struct TempDir(std::path::PathBuf);

impl TempDir {
    #[allow(dead_code)]
    pub fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[allow(dead_code)]
pub fn tempdir(label: &str) -> TempDir {
    // the per-process counter keeps calls on one thread distinct even
    // within a clock tick; pid/thread/nanos separate whole binaries
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "rustpress-{label}-{}-{:?}-{n}-{}",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    TempDir(dir)
}
