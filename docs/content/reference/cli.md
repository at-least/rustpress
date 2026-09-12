---
description: Reference of rustpress CLI commands — build, serve, and the parity tooling.
---

# Command Line Interface

## `rustpress build`

Build the site in the given directory into `<site>/public`.

### Usage

```sh
# build the site in the current directory
rustpress build

# build a site in a sub directory
rustpress build [site]
```

`site` is the directory containing `rustpress.toml` and the content directory (default: `.`).

The build:

1. loads and validates `rustpress.toml` (unknown keys fail here),
2. renders every markdown page to `public/<url>/index.html`, plus `404.html`,
3. writes `syntax.css` (the [syntax highlight](./site-config#syntaxhighlight)), `search-docs.json` when [search](./default-theme-search) is on, the selected theme under `themes/` when a [`theme`](../reference/site-config#theme) is set, and `sitemap.xml` when [`[sitemap]`](../guide/sitemap-generation) is configured,
4. writes the embedded theme assets (`vitepress.css`, `js/app.js`, `fonts/`, `themes/`) into `public/`, then copies `<site>/static/` and the sibling `../static/` (if any) over them — see [Theme assets](../guide/getting-started#theme-assets),
5. fails if any [dead links](../guide/routing#dead-links) were found.

On success it prints one line:

```
rustpress: 27 pages + 404 + syntax.css + static/ → docs/public (0.1s)
```

### Options

There are none. The output directory is always `<site>/public` and the base path always comes from [`base`](./site-config#base) in the config.

## `rustpress serve`

Build the site, then serve `<site>/public` over HTTP with live reload.

### Usage

```sh
rustpress serve [site]
```

### Options

| Option          | Description                                            |
| --------------- | ------------------------------------------------------ |
| `--port <port>` | TCP port to listen on (default: `4173`)                |

The server binds to `127.0.0.1` only. It watches the whole site directory (except `public/`) and rebuilds on any change; every served HTML page carries a small script that subscribes to `/@rustpress/livereload` and reloads the page after a rebuild. A build error is printed to the terminal and the previous output keeps being served.

There is no separate `preview` command: `serve` serves the same production build that `build` writes.

## `rustpress parity`

Maintainer tooling that compares a built site against fingerprints of the deployed vitepress.dev pages (`parity check`, `parity snapshot`, `parity diff`). It is what keeps the theme port honest and is documented in the repository's [PARITY.md](https://github.com/at-least/rustpress/blob/main/PARITY.md); a documentation author never needs it.

## Not available

There is no `init` scaffolding wizard. A site is two things — a `rustpress.toml` and a `content/index.md` — see [Getting Started](../guide/getting-started#file-structure).
