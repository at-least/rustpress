---
description: rustpress is a static site generator written in Rust that renders VitePress-format markdown into the VitePress default theme without a Node runtime.
---

# What is rustpress?

rustpress is a [Static Site Generator](https://en.wikipedia.org/wiki/Static_site_generator) (SSG) for technical documentation. It takes source content written in [VitePress](https://vitepress.dev/)'s flavor of [Markdown](https://en.wikipedia.org/wiki/Markdown), applies the VitePress default theme to it, and generates static HTML pages that can be deployed anywhere — from a single Rust binary, with no Node.js involved at render time.

::: tip {no-title}
Just want to try it out? Skip to the [Quickstart](./getting-started).
:::

## Use Cases

- **Documentation**

  rustpress ships one theme: a port of the VitePress default theme, designed for technical documentation. It powers the page you are reading right now. Navbar, sidebar, right-hand outline, local search, prev/next links, edit links, dark mode and a home page with hero and features are all built in.

- **Existing VitePress content**

  Because the content format *is* VitePress's — YAML front matter, `:::` containers, code groups, `<<<` snippets, `<Badge>` — an existing docs folder can be built by rustpress with its markdown untouched. Only the config file changes shape (TOML instead of TypeScript). See [Coming from VitePress](./coming-from-vitepress) for what carries over and what does not.

rustpress is **not** a general-purpose site framework. There is no Vue, no plugin system, no custom themes, and no data loaders. If your site needs those, VitePress itself is the right tool.

## How it works

- **Content format = VitePress.** Every page is ordinary VitePress markdown. The [Markdown Extensions](./markdown) guide lists exactly what the renderer understands.

- **Renderer = Rust.** [comrak](https://github.com/kivikakk/comrak) parses GFM (tables, task lists, footnotes, GitHub-style heading ids) behind a fence-aware preprocessor that expands `:::` containers, code groups, line-highlight info strings, `<<<` includes and `<Badge>` into HTML. Syntax highlighting is [tree-sitter](https://tree-sitter.github.io/) with the grammars compiled in; the colors are Helix TOML themes, and all 218 bundled with the Helix editor are selectable by name.

- **Markup = compiled-in templates.** The whole default theme is rendered server-side from Rust templates. There is no runtime framework to hydrate.

- **Interactivity = a small Alpine.js bundle.** Scrollspy, the sidebar drawer, flyouts, the appearance toggle, code-group tabs, copy buttons and the local search modal (<kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>K</kbd>) run over the server-rendered markup. Pages work fully without JavaScript except for those affordances.

- **Search = a JSON index built at build time**, scored client-side. No external service.

## Performance

A rustpress build is a plain static build: every page is a complete HTML document, styled by one CSS file and one JavaScript bundle shared by the whole site. There is no client-side router, no hydration and no per-page JavaScript chunk, so first load is a single HTML fetch plus cached assets, and navigation is an ordinary page load.

Builds are fast because there is nothing to bundle: markdown in, HTML out. The site you are reading builds in well under a second.

## What about VitePress?

rustpress is a re-implementation of VitePress's *content format and default theme*, not of VitePress. VitePress is a Vite + Vue application: markdown pages compile to Vue components, themes are Vue code, and data can be loaded at build time from JavaScript. rustpress deliberately has none of that. It exists for people who want VitePress-looking documentation from a toolchain that is only `cargo`.

The relationship is tracked mechanically: the [feature-parity audit](https://github.com/at-least/rustpress/blob/main/FEATURE-PARITY.md) lists every documented VitePress option and markdown extension against what rustpress implements, and the test suite renders a verbatim copy of the VitePress docs as its golden corpus.
