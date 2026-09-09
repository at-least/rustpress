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
            let engine = gen_docs::markdown::MarkdownEngine::new(&config.markdown)
                .context("building markdown engine")?;
            let content_dir = gen_docs::sidebar::content_dir(&site);
            // Render every page now (stage 4 writes the site output).
            let mut rendered = 0usize;
            let mut headings = 0usize;
            for page in &content.pages {
                let out = engine.render(page, &content, &site, &content_dir)?;
                rendered += 1;
                headings += out.headings.len();
            }
            println!(
                "gen-docs: {} pages rendered ({} headings), {} sidebar(s) — title: {}, lang: {}, base: {}",
                rendered,
                headings,
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
