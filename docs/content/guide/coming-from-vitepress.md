---
outline: deep
description: Migrate an existing VitePress site to rustpress — what carries over unchanged, how the config maps to rustpress.toml, and which VitePress features have no counterpart.
---

# Coming from VitePress

rustpress reads VitePress's content format, so a migration is mostly a matter of moving the config. This page lists what carries over, how `config.ts` maps to `rustpress.toml`, and — just as important — what rustpress deliberately does not do.

## Content carries over

Copy your markdown tree into `content/`. The following work exactly as in VitePress:

- YAML front matter and every [front-matter key](../reference/frontmatter-config) of the default theme (`layout`, `hero`, `features`, `outline`, `sidebar`, `aside`, `prev`/`next`, …).
- All [markdown extensions](./markdown): custom containers, GitHub alerts, code groups, line highlighting, focus/diff/error markers, line numbers, `<<<` snippets with regions, `<!--@include-->`, `[[toc]]`, footnotes, emoji, custom heading anchors, math, `<Badge>`.
- Relative links between pages, with or without the `.md` suffix.
- `public/` becomes `static/`: files are copied verbatim to the output root.

Two things to check:

- **URLs are always directory-style.** `guide/x.md` is served at `/guide/x/`, never at `/guide/x.html`. Links written with `.html` are rewritten, but external sites linking to your `.html` URLs will need redirects.
- **Vue in markdown stays literal.** `{{ expressions }}`, `<script setup>`, imported components and `<ClientOnly>` are not interpreted; they render as text or as unknown HTML tags. Remove them, or replace them with static content.

## Config mapping

`.vitepress/config.ts` becomes `rustpress.toml`. Keys keep VitePress's camelCase spelling; everything under `themeConfig` moves to the top level. A typical config:

::: code-group

```ts [.vitepress/config.ts]
export default defineConfig({
  title: 'My Docs',
  description: 'Just playing around.',
  lastUpdated: true,
  themeConfig: {
    logo: '/logo.svg',
    nav: [{ text: 'Guide', link: '/guide/', activeMatch: '/guide/' }],
    sidebar: {
      '/guide/': {
        base: '/guide/',
        items: [
          { text: 'Introduction', items: [
            { text: 'Getting Started', link: 'getting-started' }
          ]}
        ]
      }
    },
    socialLinks: [{ icon: 'github', link: 'https://github.com/me/docs' }],
    editLink: { pattern: 'https://github.com/me/docs/edit/main/docs/:path' },
    footer: { message: 'MIT Licensed' },
    search: { provider: 'local' }
  }
})
```

```toml [rustpress.toml]
title = "My Docs"
description = "Just playing around."
lastUpdated = true
logo = "/logo.svg"

[[nav]]
text = "Guide"
link = "/guide/"
activeMatch = "/guide/"

[sidebar."/guide/"]
base = "/guide/"

  [[sidebar."/guide/".items]]
  text = "Introduction"

    [[sidebar."/guide/".items.items]]
    text = "Getting Started"
    link = "getting-started"

[[socialLinks]]
icon = "github"
link = "https://github.com/me/docs"

[editLink]
pattern = "https://github.com/me/docs/edit/main/docs/:path"

[footer]
message = "MIT Licensed"

[search]
provider = "local"
```

:::

Key-by-key differences:

| VitePress | rustpress |
| --- | --- |
| `srcDir` defaults to the project root | defaults to `content` |
| `outDir` | always `<site>/public`, not configurable |
| `cleanUrls` | no option; URLs are always directory-style |
| `rewrites` with `path-to-regexp` params or a function | static map plus a trailing `:rest*` only |
| `markdown.theme` (Shiki) | [`[code]`](../reference/site-config#code) — Helix TOML themes |
| `.vitepress/theme/custom.css` | [`theme`](#theming) — a path to a CSS file |
| `themeConfig.lastUpdated.text` | flat `lastUpdatedText` |
| `themeConfig.search.options.translations` | `[search.translations]` |
| `markdown.image.lazyLoad` | `[markdown.image] lazyLoading` (the upstream spelling is accepted too) |
| `appearance: 'force-dark'` | `"force-dark"` or `"force"` |
| `locales.<x>.themeConfig` / `.head` / `.markdown` | not available; only `label`, `lang`, `title`, `description` per locale |

Unknown keys fail the build, so the first `rustpress build` after a migration is an inventory of what did not map. The complete list of options is in [Site Config](../reference/site-config) and [Default Theme Config](../reference/default-theme-config).

## Theming

rustpress ships one design, and the supported customization is **colors only** — two settings in `rustpress.toml`:

```toml
theme = "catppuccin"   # UI colors — every bundled theme in the Theme Gallery

[code]
light = "github_light" # code colors — one per mode, all in the Syntax Highlight gallery
dark = "catppuccin_mocha"
```

That is the whole surface. Overriding `--vp-*` variables in a CSS file of your own is possible but undocumented and not recommended; if you want to restyle components, swap fonts, or extend the theme itself, [VitePress](https://vitepress.dev) is the tool built for it.

## What has no counterpart

rustpress has no JavaScript at build time and no Vue at run time. These VitePress features are therefore absent by design, not on a roadmap:

- **Vue in markdown** — `<script setup>`, `{{ }}` interpolation, components, `<ClientOnly>`, `v-pre`.
- **Custom themes and theme extension** — `.vitepress/theme/index.ts`, `enhanceApp`, layout slots, overriding internal components. Customization stops at colors ([Theming](#theming)); past that, VitePress is the tool.
- **Build-time data loading** — `*.data.js` loaders, `createContentLoader`.
- **Dynamic routes** — `[param].md` with a `.paths.js` loader.
- **Build hooks and Vite/Vue config** — `transformHead`, `transformHtml`, `transformPageData`, `buildEnd`, `vite`, `vue`, `markdown.config()`.
- **Runtime API** — `useData`, `useRoute`, `useRouter`, `withBase`, `$frontmatter`, `$params`.
- **External services** — Algolia DocSearch, Carbon Ads.
- **Team page components** (`VPTeamMembers` and friends).
- **MPA mode and SSR compatibility concerns** — rustpress output is already plain static HTML with a small Alpine.js bundle; there is no hydration to opt out of.

## Things rustpress adds

A few options exist only on this side:

- `[code]` with all 218 Helix editor themes bundled.
- `[markdown] codeCopyButton = false` to drop the copy button.
- `[notFound]` title/quote/link text for the 404 page.
- `askAiUrl` for a navbar link.
- Custom `:::` containers that reuse a built-in kind's styling: `[[markdown.container.custom]]`.
- A dead-link check that fails the build by default.
