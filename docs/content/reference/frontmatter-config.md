---
outline: deep
description: Reference of every frontmatter key rustpress honors in a Markdown page, and what each one does to the rendered page.
---

# Frontmatter Config

Frontmatter enables page-based configuration. In every markdown file, you can use frontmatter to override site-level or theme-level options for that page only. There are also options that exist *only* in frontmatter (`layout`, `hero`, `features`, `prev`/`next`…).

Example usage:

```md
---
title: Docs with rustpress
editLink: false
---
```

The block must be the first thing in the file: a `---` line, YAML, and a closing `---` line. Keys are camelCase, as in VitePress. Keys rustpress does not know are tolerated and ignored, so custom metadata does not break a build — but there is no `$frontmatter` global or `{{ }}` interpolation to read it back with; such text stays literal.

::: tip JSON-style frontmatter
JSON is valid YAML, so a flow mapping between the dashes parses too:

```md
---
{ "title": "Blogging Like a Hacker", "editLink": false }
---
```
:::

## title

- Type: `string`

Title for the page. It replaces the title otherwise inferred from the page's first `<h1>` (or, failing that, the file name): in the `<title>` element, the sidebar entry of an auto-derived sidebar, and the search index.

```yaml
---
title: rustpress
---
```

## titleTemplate

- Type: `string | false`

The suffix for the `<title>`. Same semantics as [config.titleTemplate](./site-config#titletemplate) and overrides the site-level value for this page: `:title` is replaced with the page title; a template without `:title` is appended after an em dash; `false` uses the page title alone.

```yaml
---
title: rustpress
titleTemplate: A VitePress-format docs generator in Rust
---
```

## description

- Type: `string`

Description for the page, rendered as `<meta name="description">`. Overrides the locale's and the site's [description](./site-config#description).

```yaml
---
description: rustpress
---
```

## head

- Type: list of `{ tag, attrs?, children? }`

Extra head tags for this page, appended after the site-level [`head`](./site-config#head) tags. The shape is the same as in `rustpress.toml` — a mapping per tag, **not** VitePress's nested array form (`- - meta` / `- name: …`), which fails to parse.

```yaml
---
head:
  - tag: meta
    attrs: { name: keywords, content: super duper SEO }
  - tag: script
    children: "console.log('hello')"
---
```

## Default Theme Only

The following frontmatter options control the built-in theme.

### layout

- Type: `doc | home | page`
- Default: `doc`

Determines the layout of the page.

- `doc` — the documentation layout: sidebar, right-hand outline, edit link, last-updated stamp and prev/next pager around the rendered markdown.
- `home` — the home-page layout. Add [`hero`](#hero-home-page-only) and [`features`](#features-home-page-only) to build a landing page; the markdown body is not rendered.
- `page` — like `doc` but without most of the doc chrome: no outline aside, no edit link, no last-updated stamp. The prev/next pager and the sidebar remain, and the markdown body keeps the same content styling as `doc`.

```yaml
---
layout: doc
---
```

### hero <Badge type="info" text="home page only" />

Defines the hero section when `layout` is `home`: `name`, `text`, `tagline`, `image` and `actions`. See [Default Theme: Home Page](./default-theme-home-page).

### features <Badge type="info" text="home page only" />

Defines the feature cards when `layout` is `home`. See [Default Theme: Home Page](./default-theme-home-page).

### navbar

- Type: `boolean`
- Default: `true`

Whether to render the [navbar](./default-theme-nav) (and, with it, the search modal) on this page.

```yaml
---
navbar: false
---
```

### sidebar

- Type: `boolean`
- Default: `true`

Whether the page is laid out as a [sidebar](./default-theme-sidebar) page.

```yaml
---
sidebar: false
---
```

::: warning Partial
`sidebar: false` currently switches the page to the no-sidebar content layout (centered, wider column, footer visible) but the sidebar panel itself is still rendered when a sidebar matches the page's URL. To have no sidebar on a page today, keep it out of every configured sidebar path instead.
:::

### aside

- Type: `boolean | 'left'`
- Default: `true`

Where the aside (the outline column) goes in the `doc` layout. Overrides the site-level [`aside`](./default-theme-config#aside).

Setting this value to `false` prevents rendering of the aside container.\
Setting this value to `true` renders the aside to the right.\
Setting this value to `'left'` renders the aside to the left.

```yaml
---
aside: false
---
```

### outline

- Type: `number | [number, number] | 'deep' | false`
- Default: the site-level [`outline.level`](./default-theme-config#outline) (`[2, 3]`)

The heading levels shown in the outline for this page. A number shows only that level, a pair is an inclusive range, `deep` means `[2, 6]`, and `false` hides the outline (the aside column stays unless [`aside`](#aside) is also `false`). `outline: true` is rejected as meaningless.

```yaml
---
outline: [2, 4]
---
```

### lastUpdated

- Type: `boolean | string`
- Default: `true`

Whether to display the [last updated](./default-theme-last-updated) stamp in the page footer (only when the site-level [`lastUpdated`](./site-config#lastupdated) is on). A string is shown verbatim instead of the git timestamp.

```yaml
---
lastUpdated: false
---
```

```yaml
---
lastUpdated: 2026-01-02
---
```

### editLink

- Type: `boolean`
- Default: `true`

Whether to display the [edit link](./default-theme-edit-link) in the page footer.

```yaml
---
editLink: false
---
```

### footer

- Type: `boolean`
- Default: `true`

Whether to display the site [footer](./default-theme-footer) on this page. (The footer is hidden on sidebar pages regardless.)

```yaml
---
footer: false
---
```

### pageClass

- Type: `string`

Adds an extra class to the page's content container (`#VPContent`).

```yaml
---
pageClass: custom-page-class
---
```

You can then target the page from the CSS file selected by [`theme`](./site-config#theme):

```css
.custom-page-class {
  /* page-specific styles */
}
```

### search

- Type: `boolean`
- Default: `true`

Set to `false` to leave this page out of the local [search](./default-theme-search) index.

```yaml
---
search: false
---
```

### prev / next

- Type: `false | string | { text, link }`

Override the [prev/next links](./default-theme-prev-next-links) the sidebar order would produce: `false` hides that side, a string replaces the link text and keeps the target, an object sets both.

```yaml
---
prev: false
next:
  text: Site Config
  link: /reference/site-config/
---
```

### isHome

Not supported. The home layout is selected only by `layout: home`; there are no custom layouts to force home-page elements into.
