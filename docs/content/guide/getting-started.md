---
description: Get up and running with rustpress. Build the binary, create a site directory with rustpress.toml and content/, and serve it with live reload.
---

# Getting Started

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.85 or newer (`cargo` is the only build tool the generator needs).
- [Node.js](https://nodejs.org/) 18 or newer, **once**, to produce the theme's CSS and JavaScript bundle. They are not checked in and not embedded in the binary yet, so a fresh checkout has to build them.
- A terminal and a text editor with [Markdown](https://en.wikipedia.org/wiki/Markdown) support.

### Build from source

rustpress is not on crates.io yet. When it is, the crate will be named `rustpress-cli` (the `rustpress` crate name belongs to another project) and the installed binary will still be `rustpress`. For now, clone the repository and build it. The theme's stylesheet and script are produced by npm and then embedded into the binary, so the npm step has to run before `cargo build`:

```sh
$ git clone https://github.com/at-least/rustpress
$ cd rustpress
$ npm install                      # tailwindcss CLI + esbuild + alpinejs
$ npm run build:js && npm run build:css   # → static/js/app.js, static/vitepress.css
$ cargo build --release            # → target/release/rustpress
```

`cargo install --path .` puts the binary on your `PATH`. The binary is self-contained: the theme's `vitepress.css`, `js/app.js` and the Inter font files are compiled into it (see [Theme assets](#theme-assets)), so the checkout is not needed afterwards.

## File Structure

A site is a directory containing a `rustpress.toml` and a `content/` tree of markdown files:

```
.
├─ my-docs
│  ├─ rustpress.toml
│  ├─ static/            # copied verbatim to the output root (optional)
│  └─ content
│     ├─ index.md
│     ├─ guide
│     │  └─ getting-started.md
│     └─ reference
│        └─ config.md
└─ ...
```

The site directory is the **project root**. `content/` is the [source directory](./routing#source-directory) (configurable with [`srcDir`](../reference/site-config#srcdir)), and `public/` inside the site directory is where the build output goes.

::: tip
The build writes to `<site>/public`. If using Git, add it to your `.gitignore`.
:::

### The Config File

`rustpress.toml` mirrors VitePress's config, with VitePress's `themeConfig` keys flattened to the top level and spelled exactly as VitePress spells them (camelCase):

```toml [rustpress.toml]
title = "My Docs"
description = "Just playing around."

[[nav]]
text = "Guide"
link = "/guide/getting-started/"

[[socialLinks]]
icon = "github"
link = "https://github.com/me/my-docs"
```

Unknown keys are rejected, so a typo fails the build instead of silently doing nothing. Consult the [Site Config](../reference/site-config) and [Default Theme Config](../reference/default-theme-config) references for every option.

### Source Files

Markdown files under `content/` are **source files**. rustpress uses **file-based routing**: each `.md` file becomes a directory-style URL — `index.md` is served at `/`, `guide/getting-started.md` at `/guide/getting-started/`. Routing, links and rewrites are covered in the [Routing Guide](./routing).

### Theme assets

Every generated page links `/vitepress.css`, `/js/app.js` and the fonts under `/fonts/`. These files are embedded in the `rustpress` binary and written into `public/` on every build, so a site needs nothing beyond `rustpress.toml` and `content/` to come out fully styled.

Files in the site's own `static/` are copied over the embedded ones, so a `static/vitepress.css` of your own replaces the theme stylesheet (see [Theming](./coming-from-vitepress#theming) for the supported ways to change colors before going that far). When the site sits directly under a rustpress checkout, as this documentation does at `docs/`, the checkout's freshly built `static/` is layered on top as well, which is what lets `npm run dev:docs` pick up CSS and JS changes without rebuilding the binary. See [Asset Handling](./asset-handling) for what else belongs in `static/`.

## Up and Running

Build the site:

```sh
$ rustpress build my-docs
rustpress: 3 pages + 404 + syntax.css + static/ → my-docs/public (0.0s)
```

Or start the dev server, which builds once, watches the site directory, rebuilds on every change and reloads open browser tabs:

```sh
$ rustpress serve my-docs
rustpress: serving my-docs/public on http://127.0.0.1:4173
```

Both commands default to the current directory when no site path is given. `--port` changes the dev server port. More command line usage is documented in the [CLI Reference](../reference/cli).

## What's Next?

- To better understand how markdown files are mapped to generated HTML, proceed to the [Routing Guide](./routing).

- To discover what you can do on a page, refer to the "Writing" section of the guide. A great place to start is [Markdown Extensions](./markdown).

- To explore the features provided by the theme, check out the [Default Theme Config Reference](../reference/default-theme-config).

- If you want to change the look of your site, see [Theming](./coming-from-vitepress#theming).

- If you already have a VitePress site, read [Coming from VitePress](./coming-from-vitepress) for what carries over.

- Once your documentation site takes shape, make sure to read the [Deployment Guide](./deploy).
