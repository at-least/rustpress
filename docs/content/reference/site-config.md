---
outline: deep
description: Every rustpress.toml key — types, defaults, one-line descriptions.
---

# Site Config

Everything lives in one file, `rustpress.toml`, in the site directory — site settings and layout settings at the same top level, all camelCase. The file is plain TOML, read once at build time.

## Config Resolution

The config file is always `<site>/rustpress.toml`, where `<site>` is the directory you pass to the CLI (`rustpress build docs`) — the current directory when omitted. See [Routing: Site and Source Directory](../guide/routing).

::: warning Unknown keys fail the build
Parsing is strict. A key rustpress does not know — a typo, or a VitePress option with no rustpress counterpart — is an error at load time rather than a silent no-op:

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

There is no dynamic or async config, no TypeScript, no `defineConfig`: the config is data. Per-page overrides live in front matter — see [Frontmatter Config](./frontmatter-config).

## Site

### title

- Type: `string`
- Default: unset

Site title, shown in the navbar and used as the `<title>` prefix.

### titleTemplate

- Type: `string` or `false`
- Default: unset — `"page | site title"`

A template for each page's `<title>`: `:title` is replaced by the page title (`":title — My Docs"`); a template without `:title` is appended after an em dash; `false` uses the page title alone.

### description

- Type: `string`
- Default: unset

Site-wide description, used for `<meta name="description">` when a page has none of its own.

### lang

- Type: `string`
- Default: `"en"`

Root language of the site (`en`, `zh-TW`, …) — the `<html lang>` attribute.

### base

- Type: `string`
- Default: `"/"`

Base URL path the site is deployed under. Must start and end with `/`.

### head

- Type: array of `[[head]]` tables
- Default: `[]`

Extra tags injected into every page's `<head>`. Each entry is one tag: `tag`, inline-table `attrs`, optional `children`.

```toml
[[head]]
tag = "link"
attrs = { rel = "icon", type = "image/svg+xml", href = "/logo.svg" }
```

`href`/`src` values are copied verbatim — include the `base` yourself.

### lastUpdated

- Type: `boolean`
- Default: `false`

Show a "last updated" timestamp in every page's footer, from git timestamps. The label is `lastUpdatedText`; per-page opt-out via frontmatter.

## Sources and URLs

### srcDir

- Type: `string`
- Default: `"content"`

Directory (inside the site dir) holding the markdown source.

### srcExclude

- Type: `string` array
- Default: `[]`

Glob patterns for content files to exclude: `*` matches within a path segment, `**` across segments.

### rewrites

- Type: map of `source → destination`
- Default: `{}`

URL rewrites. Keys are source paths (relative to the source directory, `.md` spelling); values are destinations. A pattern may end with `:rest*` (captured as `:rest`). See [Routing: Route Rewrites](../guide/routing#route-rewrites).

### ignoreDeadLinks

- Type: `boolean` | `"localhostLinks"` | `string` array
- Default: `false` — internal dead links fail the build

`true` ignores all dead links; a list of strings ignores links starting with any of the listed prefixes. External (`http(s)`, `mailto:`) targets are never checked.

### sitemap

- Type: table with a `hostname` string
- Default: unset — no sitemap

Emits `sitemap.xml`. The hostname is the origin alone — page URLs already carry the `base`. See [Sitemap Generation](../guide/sitemap-generation).

### locales

- Type: map of locale tables (`label`, `lang`, `title`, `description`)
- Default: `{}`

Multi-language sites: the special `root` key describes the top-level content; every other key names a content subdirectory (`content/zh/` served under `/zh/`). See [Internationalization](../guide/i18n).

## Theming

### theme

- Type: `string` — a bundled name or a `.css` path relative to the site dir
- Default: unset — the stock look

The UI theme: a complete design (every design token, light and dark) shipped in the binary, or your own CSS file copied verbatim. Previewed live in the [Theme gallery](/themes/).

### code

- Type: table with `light` and `dark` strings
- Default: `light = "github_light"`, `dark = "github_dark"`

Code colors, one per color mode. Each value is a bundled theme's file stem or a path to a TOML theme file. All 218 themes vendored from the [Helix editor](https://github.com/helix-editor/helix/tree/master/runtime/themes) are bundled. Previewed live in the [Syntax Highlight gallery](/syntax-highlight/).

```toml
[code]
light = "github_light"
dark = "catppuccin_mocha"
```

### appearance

- Type: `true` | `false` | `"dark"` | `"force"` | `"force-auto"`
- Default: `true`

Dark mode: `true` — toggleable, follows the system; `false` — light only, no toggle; `"dark"` — dark default, toggleable; `"force"` / `"force-auto"` — always dark / always system, no toggle.

### gradedContainers

- Type: `boolean`
- Default: `false`

Color graded containers and badges by severity (danger red, warning orange, caution yellow) instead of one brand color.

## Layout

### nav

- Type: array of `[[nav]]` tables
- Default: `[]` — just the site title

Navbar entries. A plain link item: `{ text, link, activeMatch?, target?, rel? }`. A dropdown: `{ text, items = [...] }`. `activeMatch` is a URL prefix overriding link-based active highlighting.

### sidebar

- Type: absent | one array | a map of path prefixes
- Default: absent — one sidebar derived per top-level content section

One array configures a single sidebar; a map (`"/guide/": { base, items }`) configures one per section, longest matching prefix wins. Items are `{ text, link?, items?, collapsed?, base?, docFooterText?, target?, rel? }`; `collapsed` is tri-state — `true` starts collapsed, `false` starts expanded, absent = not collapsible. See [Sidebar](./default-theme-sidebar).

### outline

- Type: `false` | a level (`2`, `[2]`, `[2, 3]`) | `{ level, label }`
- Default: level `[2, 3]`, label `"On this page"`

The right-hand "on this page" outline.

### aside

- Type: `false` | `true` | `"left"`
- Default: right side

Position of the outline column.

### socialLinks

- Type: array of `[[socialLinks]]` tables
- Default: `[]`

Navbar icons: `{ icon, link, ariaLabel?, target? }` — `icon` is a known name (`github`, `twitter`, `discord`, …) or `{ svg = "…" }`. `target` defaults to `_blank`.

### editLink

- Type: table with `pattern` and optional `text`
- Default: unset

"Edit this page" link; `:path` in the pattern is replaced with the page's source-relative path.

### footer

- Type: table with `message` and/or `copyright`
- Default: unset

Footer lines, hidden on sidebar pages.

### logo

- Type: `string` | `{ src, alt }` | `{ light, dark, alt }`
- Default: unset

Navbar logo, optionally switched per color mode.

### siteTitle

- Type: `string` | `false`
- Default: unset — the `title` is used

Navbar title override; `false` hides the title next to the logo. The `<title>` element is unaffected.

### askAiUrl

- Type: `string`
- Default: unset

Adds an "Ask AI" sparkle link in the navbar.

### docFooter

- Type: table with `prev` and/or `next`
- Default: `"Previous page"` / `"Next page"`

Prev/next pager labels; `false` on a side disables that pager.

### notFound

- Type: table with `title`, `quote`, `linkText`
- Defaults: `"PAGE NOT FOUND"`, VitePress's 404 quote, `"Take me home"`

Texts for the 404 page.

### UI strings

Flat string keys for interface labels — the defaults are the English originals:

| Key | Default |
| --- | --- |
| `lastUpdatedText` | `"Last updated"` |
| `returnToTopLabel` | `"Return to top"` |
| `darkModeSwitchLabel` | `"Appearance"` |
| `lightModeSwitchTitle` | `"Switch to light theme"` |
| `darkModeSwitchTitle` | `"Switch to dark theme"` |
| `sidebarMenuLabel` | `"Menu"` |
| `langMenuLabel` | `"Change language"` |
| `navMenuLabel` | `"Main Navigation"` |
| `mobileMenuLabel` | `"Menu"` |
| `skipToContentLabel` | `"Skip to content"` |

## Markdown

### markdown

- Type: table
- Default: everything off except `codeCopyButton`

```toml
[markdown]
lineNumbers = false        # number every code block (per-fence :line-numbers overrides)
codeCopyButton = true      # hover copy button on code blocks
math = false               # render $…$ / $$…$$ TeX via client-side MathJax

[markdown.image]
lazyLoading = false        # loading="lazy" on content images

[markdown.container]       # title labels for the built-in ::: kinds
tipLabel = "TIP"
custom = [{ name = "warning-quiet", kind = "warning", label = "Heads up" }]
```

`[markdown.container]` keys: `tipLabel`, `warningLabel`, `dangerLabel`, `noteLabel`, `infoLabel`, `importantLabel`, `cautionLabel`, `detailsLabel` — each overrides that kind's title row (default: the kind uppercased). A `custom` list adds new `::: name` containers reusing a built-in `kind`'s styling with a custom `label` (default: the name uppercased). See [Markdown Extensions](../guide/markdown).
