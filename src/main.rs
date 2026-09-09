//! gen-docs CLI: `build` renders a site to `public/`, `serve` (later)
//! watches and serves.

use std::path::PathBuf;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(name = "gen-docs", version, about = "VitePress-format docs site generator")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the site in SITE (default: current directory) into public/.
    Build {
        /// Site directory containing gen-docs.toml + content/.
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build { site } => {
            let started = std::time::Instant::now();
            let site_model = gen_docs::render::Site::load(&site)
                .with_context(|| format!("loading site from {}", site.display()))?;
            let out_dir = site.join("public");
            let stats = site_model.build(&site, &out_dir)?;
            println!(
                "gen-docs: {} pages + 404 + syntax.css + static/ → {} ({:.1}s)",
                stats.pages,
                out_dir.display(),
                started.elapsed().as_secs_f32()
            );
            Ok(())
        }
        Command::Serve { site: _, port: _ } => {
            bail!("serve is not implemented yet");
        }
    }
}
