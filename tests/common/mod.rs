//! Shared policy for upstream-dependent tests: the pinned
//! vuejs/vitepress clone (`../vitepress/docs/en`) is required — absent
//! means FAIL with the fix in the message, never a silent skip.
//! `RUSTPRESS_ALLOW_NO_UPSTREAM=1` restores an explicit, visible skip
//! for offline local runs; CI always syncs the clone.

use std::path::PathBuf;

pub fn upstream_docs() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("vitepress/docs/en")
}

/// `Some(clone dir)` when present; `None` for an explicit, visible skip
/// (only when `RUSTPRESS_ALLOW_NO_UPSTREAM=1` is set); panics with the
/// actionable fix when the clone is absent and not opted out.
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
