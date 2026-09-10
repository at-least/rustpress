---
outline: deep
description: Reference of every theme-level configuration key in rustpress.toml — logo, nav, sidebar, outline, social links, footer, edit link, search and the UI labels.
---

# Default Theme Config

Theme config lets you customize the built-in theme. In VitePress these options sit under `themeConfig`; in `rustpress.toml` they are top-level keys next to the [site config](./site-config), because rustpress has exactly one theme and nothing else to scope them to:

```toml [rustpress.toml]
lang = "en-US"
title = "rustpress"
description = "A VitePress-format docs generator in Rust."

# theme-level keys, same level
logo = "/logo.svg"

[[nav]]
text = "Guide"
link = "/guide/"

[sidebar."/guide/"]
# ...
```

As with every key, an unknown name fails the build — see [Config Resolution](./site-config#config-resolution).

## logo

- Type: `string | { src, alt? } | { light, dark, alt? }`

Logo file to display in the nav bar, right before the site title. Accepts a path string, or a table to set an alt text or a different logo for light/dark mode. Paths are resolved against [`base`](./site-config#base); put the file in the site's `static/` directory.

```toml
logo = "/logo.svg"
```

```toml
logo = { light = "/logo-light.svg", dark = "/logo-dark.svg", alt = "rustpress" }
```

## siteTitle

- Type: `string | false`

Replaces the site [`title`](./site-config#title) in the nav bar. Set to `false` to show no title next to the logo (useful when the logo already contains the name). The `<title>` element is unaffected.

```toml
siteTitle = "Hello World"
```

## nav

- Type: `[[nav]]` array of items

The nav menu. An item is either a link (`text` + `link`) or a dropdown (`text` + `items`). More details in [Default Theme: Nav](./default-theme-nav).

```toml
[[nav]]
text = "Guide"
link = "/guide/"
activeMatch = "/guide/"

[[nav]]
text = "Dropdown Menu"
items = [
  { text = "Item A", link = "/item-1/" },
  { text = "Item B", link = "/item-2/" },
  { text = "Item C", link = "https://example.com", target = "_blank", rel = "noopener" },
]
```

Link items accept `text`, `link`, `activeMatch`, `target` and `rel`. Dropdown items accept `text` and `items` (link items only — no nested sections, no `noIcon`, and links are strings, never functions).

## sidebar

- Type: absent, `[[sidebar]]` array, or `[sidebar."/path/"]` tables

The sidebar menu. When absent, one sidebar per top-level content directory is derived automatically. An array is a single sidebar for the whole site; path-keyed tables give each URL prefix its own sidebar. More details in [Default Theme: Sidebar](./default-theme-sidebar).

```toml
[[sidebar]]
text = "Guide"

  [[sidebar.items]]
  text = "Introduction"
  link = "/introduction/"

  [[sidebar.items]]
  text = "Getting Started"
  link = "/getting-started/"
```

A sidebar item takes:

| key | meaning |
|---|---|
| `text` | the label (required) |
| `link` | the target; resolved against the nearest `base` |
| `items` | child items |
| `collapsed` | absent = not collapsible; `true` = collapsible, starts collapsed; `false` = collapsible, starts expanded |
| `base` | base path for the children's links |
| `docFooterText` | text shown when this item is a prev/next pager target |
| `target`, `rel` | link attribute overrides |

## aside

- Type: `boolean | "left"`
- Default: `true`
- Can be overridden per page via [frontmatter](./frontmatter-config#aside)

Setting this value to `false` prevents rendering of the aside container.\
Setting this value to `true` renders the aside to the right.\
Setting this value to `"left"` renders the aside to the left.

```toml
aside = "left"
```

If you only want to hide the outline, use [`outline = false`](#outline) instead.

## outline

- Type: `false | number | [number, number] | { level?, label? }`
- Default: level `[2, 3]`, label `"On this page"`
- Level can be overridden per page via [frontmatter](./frontmatter-config#outline)

The headings listed in the right-hand outline (and in the local nav dropdown on narrow screens). `false` renders no outline at all. A number lists only that heading level; a pair is an inclusive range. The table form also sets the label.

```toml
[outline]
level = [2, 3]
label = "On this page"
```

```toml
outline = 2
```

```toml
outline = false
```

Note that unlike VitePress there is no `"deep"` value at site level; use `[2, 6]`. (`outline: deep` does work in [frontmatter](./frontmatter-config#outline).)

## socialLinks

- Type: `[[socialLinks]]` array

Social account links shown with icons in the nav bar.

```toml
[[socialLinks]]
icon = "github"
link = "https://github.com/at-least/rustpress"

[[socialLinks]]
icon = "twitter"
link = "https://x.com/…"

# any other name gets a generic external-link glyph;
# pass your own SVG for a real icon
[[socialLinks]]
icon = { svg = "<svg role=\"img\" viewBox=\"0 0 24 24\" xmlns=\"http://www.w3.org/2000/svg\"><title>Dribbble</title><path d=\"M12…6.38z\"/></svg>" }
link = "https://dribbble.com/…"
ariaLabel = "cool link"        # optional but recommended
target = "_self"               # default "_blank"
```

Built-in glyphs exist for `github` and `twitter`/`x` only; there is no icon-set lookup. Any other icon name renders the generic external-link glyph, so bring an `svg` string for other services. Every social link gets `rel="noopener"`.

## footer

- Type: `[footer]` table with `message` and/or `copyright`
- Can be hidden per page via [frontmatter](./frontmatter-config#footer)

Footer configuration. The two strings are inserted as HTML, so inline links and markup work. The footer is only displayed on pages without a sidebar. See [Default Theme: Footer](./default-theme-footer).

```toml
[footer]
message = "Released under the <a href=\"https://opensource.org/licenses/MIT\">MIT License</a>."
copyright = "Copyright © 2026-present newlix"
```

## editLink

- Type: `[editLink]` table with `pattern` and optional `text`
- Can be hidden per page via [frontmatter](./frontmatter-config#editlink)

A link to edit the page on GitHub, GitLab or similar. `:path` in the pattern is replaced with the page's path relative to [`srcDir`](./site-config#srcdir) (e.g. `guide/getting-started.md`). Only the string pattern form exists. See [Default Theme: Edit Link](./default-theme-edit-link).

```toml
[editLink]
pattern = "https://github.com/at-least/rustpress/edit/main/docs/content/:path"
text = "Edit this page on GitHub"
```

## lastUpdatedText

- Type: `string`
- Default: `"Last updated"`

The label in front of the last-updated timestamp. This is a flat key — VitePress nests it as `lastUpdated.text`; in rustpress [`lastUpdated`](./site-config#lastupdated) is the boolean switch and this key is the label. There is no `formatOptions`: the timestamp is always rendered in a fixed UTC format. See [Default Theme: Last Updated](./default-theme-last-updated).

```toml
lastUpdated = true
lastUpdatedText = "Updated at"
```

## docFooter

- Type: `[docFooter]` table; `prev` and `next` are each `string | false`

Customizes the text above the previous/next links, or disables one side globally. Per-page control is in [frontmatter](./default-theme-prev-next-links).

```toml
[docFooter]
prev = "Pagina prior"
next = "Proxima pagina"
```

```toml
[docFooter]
next = false
```

## search

- Type: `[search]` table with `provider = "local"` and optional `[search.translations]`
- Default: absent — no search index and no search button

Enables the built-in local search: a JSON index built at build time and a modal (<kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>K</kbd>) scored client-side. See [Default Theme: Search](./default-theme-search) for the translation keys.

```toml
[search]
provider = "local"
```

## askAiUrl

- Type: `string`

rustpress-only. Adds a sparkle "Ask AI" link to the navbar pointing at the given URL — the place VitePress's Algolia-hosted Ask AI button would sit.

```toml
askAiUrl = "https://example.com/ask"
```

## notFound

- Type: `[notFound]` table with `title`, `quote`, `linkText`

rustpress-only. The texts of the generated `404.html`.

```toml
[notFound]
title = "Page not found"
quote = "The page you are looking for does not exist."
linkText = "Return home"
```

## darkModeSwitchLabel

- Type: `string`
- Default: `Appearance`

The label next to the appearance switch. It is only displayed in the mobile menu.

## lightModeSwitchTitle

- Type: `string`
- Default: `Switch to light theme`

Accepted for compatibility but currently unused: the appearance switch always carries the [`darkModeSwitchTitle`](#darkmodeswitchtitle) tooltip; there is no client-side swap to this title once dark mode is on.

## darkModeSwitchTitle

- Type: `string`
- Default: `Switch to dark theme`

The `title` tooltip on the appearance switch (navbar and mobile menu).

## sidebarMenuLabel

- Type: `string`
- Default: `Menu`

The label of the sidebar menu button in the local nav bar. Only displayed in the mobile view.

## returnToTopLabel

- Type: `string`
- Default: `Return to top`

The label of the return-to-top link at the top of the local nav's outline dropdown. Only displayed in the mobile view.

## langMenuLabel

- Type: `string`
- Default: `Change language`

The aria-label of the language switcher button in the navbar. Only present on multi-language sites ([i18n](../guide/i18n)).

## navMenuLabel

- Type: `string`
- Default: `Main Navigation`

The accessible label of the navigation landmarks (the navbar menu and the mobile menu).

## mobileMenuLabel

- Type: `string`
- Default: `Menu`

The aria-label of the mobile menu (hamburger) button.

## skipToContentLabel

- Type: `string`
- Default: `Skip to content`

The label of the skip-to-content link shown when navigating the site with a keyboard.

## externalLinkIcon

Not a setting: the external-link arrow after outbound links in markdown is always on. (VitePress defaults it off and makes it opt-in.)

## gradedContainers

- Type: `boolean`
- Default: `false`

Whether to color [custom containers](../guide/markdown#custom-containers), [GitHub-flavored alerts](../guide/markdown#github-flavored-alerts) and badges on a graded severity scale — danger red, warning orange, caution yellow. By default, colors match GitHub's alerts, where caution shares danger's red and warning is yellow.

```toml
gradedContainers = true
```

## Not available

These `themeConfig` options have no rustpress counterpart:

- **`i18nRouting`** — no toggle. The language switcher always targets the same page in the other locale when it exists, else that locale's root (VitePress's default behavior).
- **`algolia`** — Algolia DocSearch is an external service; only the local provider exists.
- **`carbonAds`** — external service.
- **`extraMenuLabel`** — there is no `⋯` overflow menu; a nav that doesn't fit wraps instead of collapsing.
- **`useLayout`** — a Vue composable; there is no client runtime.
