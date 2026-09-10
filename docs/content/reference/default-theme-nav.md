---
description: Configure the navigation bar in rustpress — site title, logo, menu links, dropdowns and social links.
---

# Nav

The Nav is the navigation bar displayed on top of the page. It contains the site title, global menu links, etc.

## Site Title and Logo

By default, nav shows the title of the site referencing [`title`](./site-config#title). If you would like to change what's displayed on nav, you may define custom text in the `siteTitle` option. The `<title>` of the document is not affected.

```toml [rustpress.toml]
siteTitle = "My Custom Title"
```

If you have a logo for your site, you can display it by passing in the path to the image. You should place the logo within `static` directly, and define the absolute path to it.

```toml [rustpress.toml]
logo = "/my-logo.svg"
```

When adding a logo, it gets displayed along with the site title. If your logo is all you need and if you would like to hide the site title text, set `false` to the `siteTitle` option.

```toml [rustpress.toml]
logo = "/my-logo.svg"
siteTitle = false
```

You can also pass a table as logo if you want to add `alt` attribute or customize it based on dark/light mode. Refer [`logo`](./default-theme-config#logo) for details.

## Navigation Links

You may define `[[nav]]` entries to add links to your nav.

```toml [rustpress.toml]
[[nav]]
text = "Guide"
link = "/guide/"

[[nav]]
text = "Config"
link = "/config/"

[[nav]]
text = "Changelog"
link = "https://github.com/..."
```

The `text` is the actual text displayed in nav, and the `link` is the link that will be navigated to when the text is clicked. For the link, set path to the actual file without `.md` prefix, and always start with `/`. The configured [`base`](./site-config#base) is prepended at build time.

Nav links can also be dropdown menus. To do this, set `items` on the entry instead of `link`.

```toml [rustpress.toml]
[[nav]]
text = "Guide"
link = "/guide/"

[[nav]]
text = "Dropdown Menu"
items = [
  { text = "Item A", link = "/item-1/" },
  { text = "Item B", link = "/item-2/" },
  { text = "Item C", link = "/item-3/" },
]
```

Note that dropdown menu title (`Dropdown Menu` in the above example) can not have `link` property since it becomes a button to open dropdown dialog.

Dropdowns are one level deep: every entry in `items` is rendered as a link. VitePress's nested "sections" inside a dropdown (an item that itself has `items`) are not supported — such an item renders as a plain link and its children are dropped.

### Customize link's "active" state

Nav menu items will be highlighted when the current page is under the matching path. If you would like to customize the path to be matched, define `activeMatch` property with a URL prefix.

```toml [rustpress.toml]
[[nav]]
# This link gets active state when the user is
# on any page under `/config/`.
text = "Guide"
link = "/guide/"
activeMatch = "/config/"
```

::: warning
Unlike VitePress, `activeMatch` is a plain path prefix, not a regular expression.
:::

### Customize link's "target" and "rel" attributes

By default, nav links carry no `target` or `rel` attributes. You can set them per entry (on top-level links and on dropdown items alike).

```toml [rustpress.toml]
[[nav]]
text = "Merchandise"
link = "https://www.thegithubshop.com/"
target = "_self"
rel = "sponsored"
```

## Social Links

Refer [`socialLinks`](./default-theme-config#sociallinks).

## Ask AI Link

rustpress has no Algolia integration, so VitePress's "Ask AI" entry point has a small stand-in: `askAiUrl` adds a sparkle icon next to the search button that opens the given URL in a new tab.

```toml [rustpress.toml]
askAiUrl = "https://example.com/ask"
```

## Custom Components

Not available. rustpress has no Vue runtime, so the `component`/`props` forms of nav entries and the overflow `⋯` menu that VitePress collapses long navs into are not implemented; a nav that does not fit the bar wraps or scrolls instead.
