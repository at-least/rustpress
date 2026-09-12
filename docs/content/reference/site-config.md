---
outline: deep
description: Complete reference of rustpress.toml site configuration options including site metadata, routing, build, theming and markdown settings.
---

# Site Config

Site config is where you define the global settings of the site: the base directory, the title, the markdown options, and so on. In rustpress everything — site-level *and* theme-level — lives in one file, `rustpress.toml`, in the site directory. This page covers the site-level keys; the theme-level keys (navbar, sidebar, footer, search, labels…) are documented in [Default Theme Config](./default-theme-config).

## Overview

### Config Resolution

The config file is always `<site>/rustpress.toml`, where `<site>` is the directory you pass to the CLI (`rustpress build docs`) — the current directory when omitted. See [Routing: Site and Source Directory](../guide/routing).

The file is plain [TOML](https://toml.io/). Key names are VitePress's camelCase names, so a `themeConfig` translates mechanically — just without the `themeConfig:` nesting:

```toml [rustpress.toml]
# site-level options
lang = "en-US"
title = "rustpress"
description = "A VitePress-format docs generator in Rust."

# theme-level options, at the same top level
logo = "logo.svg"

[[nav]]
text = "Guide"
link = "/guide/"
```

There is no dynamic or async config, no TypeScript, and no `defineConfig` helper: the config is data, read once at build time.

::: warning Unknown keys fail the build
`rustpress.toml` is parsed strictly. A key that rustpress does not know — a typo, or a VitePress option that has no rustpress counterpart — is an error at load time rather than a silent no-op:

```
Error: loading site from docs

Caused by:
    invalid TOML in docs/rustpress.toml: TOML parse error at line 1, column 1
      |
    1 | titel = "typo"
      | ^^^^^
    unknown field `titel`, expected one of `title`, `description`, …
```
:::

### Config Intellisense

There is no TypeScript type to drive IDE completion. The authoritative list of keys is this page plus [Default Theme Config](./default-theme-config); the strict parser tells you the valid names when you get one wrong.

### Typed Theme Config

rustpress ships exactly one theme, compiled into the binary, so there is no theme type to select and no custom-theme config surface. See [Coming from VitePress](../guide/coming-from-vitepress).

### Vite, Vue and Markdown Config

rustpress has no Vite and no Vue anywhere in its pipeline, so the `vite` and `vue` options do not exist. The markdown renderer (comrak + tree-sitter) is configured through the [`[markdown]`](#markdown) table below; there is no hook to reach into the parser.

### Page-Level Overrides

Some settings can be overridden for specific pages using frontmatter. See [Frontmatter Config](./frontmatter-config) for details.

### Directory-Level Overrides

Not supported. rustpress has no per-directory config files; the only scopes are the whole site (`rustpress.toml`), a locale ([`[locales]`](#locales)) and a single page (frontmatter).

## Site Metadata

### title

- Type: `string`
- Default: *(none — the navbar shows no title and `<title>` has no suffix)*
- Can be overridden per page via [frontmatter](./frontmatter-config#title)

Title for the site. It is displayed in the nav bar and used as the default suffix for every page's `<title>`, unless [`titleTemplate`](#titletemplate) is defined. A page's own title is the text of its first `<h1>` (or its frontmatter `title`), joined with the site title. With this config and page:

```toml
title = "My Awesome Site"
```

```md
# Hello
```

the document title is `Hello | My Awesome Site`. Home pages (`layout: home`) use the site title alone.

### titleTemplate

- Type: `string | false`
- Can be overridden per page via [frontmatter](./frontmatter-config#titletemplate)

Customizes each page's title suffix, or the entire title. When the template contains `:title`, it is replaced with the page title:

```toml
titleTemplate = ":title - Custom Suffix"
```

```md
# Hello
```

gives `Hello - Custom Suffix`. A template without `:title` is appended after an em dash (`Hello — Custom Suffix`). Set it to `false` to drop the suffix entirely and use the bare page title.

```toml
titleTemplate = false
```

### description

- Type: `string`
- Default: *(none)*
- Can be overridden per page via [frontmatter](./frontmatter-config#description) and per locale via [`[locales]`](#locales)

Description for the site, rendered as `<meta name="description">` on every page that has no description of its own.

```toml
description = "A rustpress site"
```

### head

- Type: `[[head]]` array of tables — `tag`, optional `attrs`, optional `children`
- Default: `[]`
- Can be appended per page via [frontmatter](./frontmatter-config#head)

Additional elements rendered inside `<head>`. They come after rustpress's own tags (stylesheets, the appearance script) and before the page's frontmatter `head` tags. Attribute values are emitted as written (double quotes are escaped); `children` is inserted verbatim as the element's inner HTML. A `<script>` is always rendered as an open/close pair; any other tag without `children` is self-closed.

There is no merging or de-duplication: every entry is emitted, in order, on every page.

#### Example: Adding a favicon

```toml
[[head]]
tag = "link"
attrs = { rel = "icon", href = "/favicon.ico" }
# put favicon.ico in the site's static/ directory
```

Renders:

```html
<link href="/favicon.ico" rel="icon"/>
```

::: tip
`head` attribute values are written out verbatim — [`base`](#base) is not prepended. With `base = "/docs/"`, write `href = "/docs/favicon.ico"`.
:::

#### Example: Adding Google Fonts

```toml
[[head]]
tag = "link"
attrs = { rel = "preconnect", href = "https://fonts.googleapis.com" }

[[head]]
tag = "link"
attrs = { rel = "preconnect", href = "https://fonts.gstatic.com", crossorigin = "" }

[[head]]
tag = "link"
attrs = { href = "https://fonts.googleapis.com/css2?family=Roboto&display=swap", rel = "stylesheet" }
```

#### Example: Registering a service worker

```toml
[[head]]
tag = "script"
attrs = { id = "register-sw" }
children = """
;(() => {
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/sw.js')
  }
})()
"""
```

#### Example: Using Google Analytics

```toml
[[head]]
tag = "script"
attrs = { async = "", src = "https://www.googletagmanager.com/gtag/js?id=TAG_ID" }

[[head]]
tag = "script"
children = """
window.dataLayer = window.dataLayer || [];
function gtag(){dataLayer.push(arguments);}
gtag('js', new Date());
gtag('config', 'TAG_ID');
"""
```

### lang

- Type: `string`
- Default: `en`
- Can be overridden per locale via [`[locales]`](#locales)

The `lang` attribute of the `<html>` element.

```toml
lang = "en-US"
```

### base

- Type: `string`
- Default: `/`

The base URL path the site is deployed under. Set it if the site lives at a sub path — for `https://foo.github.io/bar/`, use `"/bar/"`. It must start and end with a slash; anything else is rejected at load time. There is no relocatable (`./`) mode.

```toml
base = "/base/"
```

`base` is prepended to the URLs the theme generates — navbar and sidebar links, the prev/next pager, the search index, the language switcher, the logo and hero images, the stylesheet/script tags — and, as in VitePress, to every root-absolute link and image written in markdown: `[x](/guide/a/)` becomes `href="/base/guide/a/"`, `![x](/pic.png)` becomes `src="/base/pic.png"`, and relative page links (`[x](./a)`) are resolved to the target page first and then prefixed. Raw HTML in content and [`head`](#head) entries are written out verbatim.

## Routing

### cleanUrls

Not a setting: rustpress **always** emits directory-style URLs. `guide/x.md` becomes `/guide/x/` (written as `guide/x/index.html`), so no server rewrite is needed and there is no `.html` mode to turn off. See [Routing](../guide/routing#file-based-routing).

### rewrites

- Type: table of `"source path" = "destination path"`
- Default: `{}`

Custom mappings from source paths (relative to [`srcDir`](#srcdir)) to output paths. Two forms are supported: an exact file mapping, and a prefix pattern ending in `:rest*` whose captured remainder is substituted into the destination.

```toml
[rewrites]
"packages/pkg-a/src/index.md" = "pkg-a/index.md"
"en/:rest*" = ":rest*"
```

Multi-parameter patterns (`:pkg/:slug*`) and function rewrites are not supported. See [Routing: Route Rewrites](../guide/routing#route-rewrites).

## Build

### srcDir

- Type: `string`
- Default: `content`

The directory holding the markdown pages, relative to the site directory. Note that the default differs from VitePress (`.`): rustpress keeps pages under `content/` so the site directory can also hold `rustpress.toml`, `static/`, `snippets/` and the `public/` output without them mixing. See [Site and Source Directory](../guide/routing).

```toml
srcDir = "src"
```

### srcExclude

- Type: `string[]`
- Default: `[]`

Glob patterns for markdown files that should not become pages, matched against the whole source-relative path. `*` matches within one path segment, `**` matches across segments; everything else is literal.

```toml
srcExclude = ["**/README.md", "drafts/*.md"]
```

### outDir

Not configurable. The build always writes to `<site>/public/`. The CLI has no `--outDir` flag either; copy or symlink the result if it needs to live elsewhere.

### assetsDir / assetsBase / cacheDir

Not applicable. rustpress has no asset pipeline: nothing is hashed, inlined or cached, so there is no assets directory to relocate and no CDN prefix to point at. Static files are copied verbatim from `static/` — see [Asset Handling](../guide/asset-handling).

### ignoreDeadLinks

- Type: `boolean | "localhostLinks" | string[]`
- Default: `false`

By default a build fails when a page links to an internal target that does not exist — a relative or root-absolute `href`/`src` that resolves to no page and no file in the output directory:

```
Error: 2 dead link(s) found (ignoreDeadLinks silences this):
  /guide/what-is-rustpress/ → guide/does-not-exist
  /guide/what-is-rustpress/ → reference/site-config
```

Set it to `true` to skip the check entirely, or to a list of link prefixes to ignore only those:

```toml
ignoreDeadLinks = true
```

```toml
ignoreDeadLinks = ["/playground", "/repl/"]
```

`"localhostLinks"` is accepted for compatibility and behaves like the default: `http://` and `https://` targets (localhost included), `mailto:`, `data:` and `#fragment` links are never checked in the first place. Regular expressions and filter functions are not supported.

### mpa

Not a setting. Every rustpress build is static HTML plus one small Alpine.js bundle for the interactive bits; there is no SPA mode to trade away.

## Theming

### appearance

- Type: `boolean | "dark" | "force-dark" | "force-auto"`
- Default: `true`

Whether to enable dark mode (by adding the `.dark` class to the `<html>` element).

- `true` — the initial theme follows the user's preferred color scheme; a switch in the navbar lets them toggle it.
- `"dark"` — dark by default, unless the user manually toggles it.
- `false` — always light; no switch is rendered.
- `"force-dark"` — always dark; no switch is rendered. (`"force"` is accepted as an alias.)
- `"force-auto"` — always the user's preferred color scheme; no switch is rendered.

```toml
appearance = "dark"
```

The choice is applied by an inline script in `<head>` that reads the user's stored preference from local storage (`vitepress-theme-appearance`, the same key VitePress uses) before the page paints, so there is no flash of the wrong theme.

### lastUpdated

- Type: `boolean`
- Default: `false`
- Can be overridden per page via [frontmatter](./frontmatter-config#lastupdated)

Whether to look up each page's last commit time with `git log` and show it in the page footer. The label is [`lastUpdatedText`](./default-theme-config#lastupdatedtext); see [Last Updated](./default-theme-last-updated) for details, including what happens when the site is not in a git checkout.

```toml
lastUpdated = true
```

### theme

- Type: `string`
- Default: unset (the stock VitePress look)

The UI colors: the stylesheet to link after the theme's own stylesheet on every page. A bare name selects a bundled theme — a complete design (every design token, light and dark) shipped in the binary and extracted to `themes/<name>.css`; a path relative to the site directory ending in `.css` is your own file, copied verbatim into the output under `themes/<basename>` (a partial override or a full theme of your own).

```toml
theme = "catppuccin"  # any bundled name — see the Theme Gallery (/themes/)
theme = "theme.css"   # your own file
```

An unknown name or a missing file fails the build. Source-code colors are a separate setting — see [`[syntax]`](#syntax). The [Theming](../guide/theming) guide covers both.

### syntax

- Type: table with `light` and `dark` strings
- Default: `light = "github_light"`, `dark = "github_dark"`

The syntax-highlighting color schemes for code blocks, one per color mode. Each value is either a bundled theme name or a path (relative to the site directory) to a Helix-format TOML theme file. Every theme shipped with the [Helix editor](https://github.com/helix-editor/helix/tree/master/runtime/themes) is bundled and selectable by its file stem.

```toml
[syntax]
light = "github_light"
dark = "catppuccin_mocha"
```

See [Theming: Syntax Highlighting Colors](../guide/theming).

## Customization

### markdown

- Type: `[markdown]` table

Options for the markdown renderer. All keys are optional.

```toml
[markdown]
lineNumbers = true        # number every code block (default false)
codeCopyButton = true     # hover copy button on code blocks (default true)
math = true               # render $…$ / $$…$$ with MathJax (default false)

[markdown.image]
lazyLoading = true        # loading="lazy" on content images (default false)
                          # upstream's spelling `lazyLoad` is accepted too

[markdown.container]
tipLabel = "TIP"          # default title of ::: tip (likewise warningLabel,
                          # dangerLabel, noteLabel, infoLabel, importantLabel,
                          # cautionLabel, detailsLabel)

[[markdown.container.custom]]
name = "success"          # registers ::: success
kind = "tip"              # styled like this builtin kind (default tip)
label = "SUCCESS"         # default title (default: NAME uppercased)
```

- **`lineNumbers`** — global line numbers; a fence can opt out with `:no-line-numbers` or in with `:line-numbers` regardless of the global value.
- **`codeCopyButton`** — set `false` to drop the copy button from every code block.
- **`math`** — enables `$…$` inline and `$$…$$` display math. Typesetting runs client-side (MathJax from a CDN), injected only on pages that contain math.
- **`image.lazyLoading`** — adds `loading="lazy"` to every image in page content.
- **`container.*Label`** — the default title text for each container kind, also used for the matching GitHub-flavored alert.
- **`container.custom`** — additional `::: name` container keywords; each reuses one builtin kind's styling.

There is no `anchor`, `toc`, `headers`, `snippet`, `include`, `theme` or `config()` option: heading ids are always GitHub-style slugs (`{#custom}` overrides one), `[[toc]]` always lists h2–h3, and highlighting is configured by [`[syntax]`](#syntax). See [Markdown Extensions](../guide/markdown) for what the renderer does with each construct.

### vite / vue

Do not exist; see [Vite, Vue and Markdown Config](#vite-vue-and-markdown-config).

### locales

- Type: table of locale tables; the key `root` is the top-level content, every other key names a content subdirectory
- Default: `{}` (single-language site)

Declares a multi-language site. Pages under `content/<key>/` belong to locale `<key>` and are served under `/<key>/`; everything else is `root`. Each locale can set the switcher `label` (required), its own `lang`, `title` and `description`.

```toml
[locales.root]
label = "English"
lang = "en"

[locales.zh]
label = "简体中文"
lang = "zh-Hans"
title = "rustpress 文档"
description = "用 Rust 写的 VitePress 格式文档生成器"
```

Per-locale `head`, `titleTemplate`, theme config (nav/sidebar) and markdown labels are not supported; those are site-wide. See [Internationalization](../guide/i18n).

### sitemap

- Type: `[sitemap]` table with `hostname`
- Default: *(none — no sitemap is written)*

Emits `sitemap.xml` listing every page. `<lastmod>` entries come from [`lastUpdated`](#lastupdated) when it is enabled.

```toml
[sitemap]
hostname = "https://example.com"
```

See [Sitemap Generation](../guide/sitemap-generation).

## Build Hooks

There are none. `buildEnd`, `postRender`, `transformHead`, `transformHtml` and `transformPageData` are JavaScript functions that run inside VitePress's Node build; rustpress is a single binary with no plugin interface. The static [`head`](#head) key covers the common `transformHead` use of adding tags to every page, and frontmatter [`head`](./frontmatter-config#head) covers per-page tags.
