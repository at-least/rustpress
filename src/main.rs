//! rustpress CLI: `build` renders a site to `public/`, `serve` (later)
//! watches and serves.

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(name = "rustpress", version, about = "VitePress-format docs site generator")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the site in SITE (default: current directory) into public/.
    Build {
        /// Site directory containing rustpress.toml + content/.
        #[arg(value_name = "SITE", default_value = ".")]
        site: PathBuf,
    },
    /// Build and serve with live reload (not implemented yet).
    Serve {
        #[arg(value_name = "SITE", default_value = ".")]
        site: PathBuf,
        /// TCP port to listen on.
        #[arg(long, default_value_t = 4173)]
        port: u16,
    },
    /// Upstream parity fingerprints (see PARITY.md).
    Parity {
        #[command(subcommand)]
        cmd: ParityCmd,
    },
}

#[derive(Subcommand)]
enum ParityCmd {
    /// Compare a built site against the pinned upstream baseline.
    Check {
        /// Built site directory (the directory containing index.html).
        #[arg(value_name = "SITE_DIR", default_value = "demo/public")]
        site: PathBuf,
        /// Pinned upstream fingerprint file.
        #[arg(long, value_name = "FILE", default_value = "parity/upstream.json")]
        baseline: PathBuf,
        /// Reviewed, justified divergences.
        #[arg(long, value_name = "FILE", default_value = "parity/known-deltas.json")]
        deltas: PathBuf,
    },
    /// Extract fingerprints from fetched upstream HTML into a baseline.
    Snapshot {
        /// Directory of fetched pages (parity/cache layout).
        #[arg(value_name = "CACHE_DIR")]
        cache: PathBuf,
        /// Where to write the baseline JSON.
        #[arg(long, value_name = "FILE", default_value = "parity/upstream.json")]
        out: PathBuf,
        /// Page list (URLs, one per line, '#' comments).
        #[arg(long, value_name = "FILE", default_value = "parity/pages.txt")]
        pages: PathBuf,
        /// Fetch metadata JSON (generator, fetched_at, etags) merged
        /// into the baseline's meta section.
        #[arg(long, value_name = "FILE")]
        meta: Option<PathBuf>,
    },
    /// Report what changed between two baselines (e.g. after a refresh).
    Diff {
        /// Previous baseline (missing file: every page counts as new).
        #[arg(value_name = "OLD")]
        old: PathBuf,
        /// New baseline.
        #[arg(value_name = "NEW")]
        new: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build { site } => {
            let started = std::time::Instant::now();
            let site_model = rustpress::render::Site::load(&site)
                .with_context(|| format!("loading site from {}", site.display()))?;
            let out_dir = site.join("public");
            let stats = site_model.build(&site, &out_dir)?;
            println!(
                "rustpress: {} pages + 404 + syntax.css + static/ → {} ({:.1}s)",
                stats.pages,
                out_dir.display(),
                started.elapsed().as_secs_f32()
            );
            Ok(())
        }
        Command::Serve { site, port } => rustpress::serve::run(site, port)
            .await,
        Command::Parity { cmd } => match cmd {
            ParityCmd::Check {
                site,
                baseline,
                deltas,
            } => {
                let base = rustpress::parity::load_baseline(&baseline)
                    .with_context(|| format!("loading baseline {}", baseline.display()))?;
                let deltas = rustpress::parity::Deltas::load(&deltas)?;
                let mismatches = rustpress::parity::check(&site, &base, &deltas);
                if mismatches.is_empty() {
                    println!(
                        "parity: {} page(s) match the pinned upstream ({})",
                        base.pages.len(),
                        base.meta
                            .get("generator")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown version")
                    );
                    Ok(())
                } else {
                    for m in &mismatches {
                        println!("parity: {m}");
                    }
                    anyhow::bail!(
                        "parity: {} divergence(s) from the pinned upstream — fix, or review into parity/known-deltas.json (see PARITY.md)",
                        mismatches.len()
                    );
                }
            }
            ParityCmd::Snapshot {
                cache,
                out,
                pages,
                meta,
            } => {
                let page_list = rustpress::parity::parse_pages(
                    &std::fs::read_to_string(&pages)
                        .with_context(|| format!("reading page list {}", pages.display()))?,
                );
                let mut meta_map = std::collections::BTreeMap::new();
                if let Some(meta_path) = meta {
                    let raw = std::fs::read_to_string(&meta_path)
                        .with_context(|| format!("reading meta {}", meta_path.display()))?;
                    let parsed: serde_json::Value = serde_json::from_str(&raw)
                        .with_context(|| format!("parsing {}", meta_path.display()))?;
                    if let serde_json::Value::Object(map) = parsed {
                        for (k, v) in map {
                            meta_map.insert(k, v);
                        }
                    }
                }
                let baseline = rustpress::parity::snapshot(&cache, &page_list, meta_map)?;
                let json = serde_json::to_string_pretty(&baseline)?;
                if let Some(parent) = out.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&out, json + "\n")
                    .with_context(|| format!("writing {}", out.display()))?;
                println!(
                    "parity: pinned {} page(s) → {}",
                    baseline.pages.len(),
                    out.display()
                );
                Ok(())
            }
            ParityCmd::Diff { old, new } => {
                if !old.exists() {
                    println!("parity: no previous baseline at {} — this is the initial pin", old.display());
                    return Ok(());
                }
                let old_base = rustpress::parity::load_baseline(&old)?;
                let new_base = rustpress::parity::load_baseline(&new)?;
                let changes = rustpress::parity::diff(&old_base, &new_base);
                if changes.is_empty() {
                    println!("parity: upstream unchanged");
                } else {
                    for m in &changes {
                        println!("parity: {m}");
                    }
                    println!(
                        "parity: {} upstream change(s) — map each to a fix, then make `parity check` pass (see PARITY.md)",
                        changes.len()
                    );
                }
                Ok(())
            }
        },
    }
}
