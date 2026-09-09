//! gen-docs CLI: `build` renders a site to `public/`, `serve` (later)
//! watches and serves.

use std::path::PathBuf;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};

use gen_docs::config::SiteConfig;

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
            let config = SiteConfig::load(&site)
                .with_context(|| format!("loading site config from {}", site.display()))?;
            let content = gen_docs::content::Content::load(&gen_docs::sidebar::content_dir(&site))
                .context("loading content")?;
            let sidebars = gen_docs::sidebar::Sidebars::build(&config, &content);
            // Stage 3+: render pages, emit search index.
            println!(
                "gen-docs: {} pages, {} sidebar(s) (title: {}, lang: {}, base: {})",
                content.pages.len(),
                sidebars.trees.len(),
                config.title.as_deref().unwrap_or("(untitled)"),
                config.lang,
                config.base
            );
            Ok(())
        }
        Command::Serve { site: _, port: _ } => {
            bail!("serve is not implemented yet");
        }
    }
}
