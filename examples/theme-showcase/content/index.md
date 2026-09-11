---
layout: page
sidebar: false
aside: false
editLink: false
lastUpdated: false
prev: false
next: false
pageClass: theme-gallery
head:
  - tag: link
    attrs: { rel: "stylesheet", href: "/theme-gallery.css" }
  - tag: script
    attrs: { src: "/theme-gallery.js", defer: "" }
---
# UI Palettes

rustpress ships 12 built-in palettes; `vitepress` (the stock look) is the default — nothing extra is emitted.
Each card overrides only the four `--vp-c-brand-*` variables; the surrounding page stays in the site's default theme.
Click a name to see this page rebuilt with that palette active; use the navbar's appearance switch to preview dark mode.

::: tip Try it
This page is built with the stock look. Open [mono](./themes/mono/) and hit the appearance switch — the brand color goes from near-black to near-white without a card reload.
:::

::: raw
<div class="theme-gallery-grid">
  <div class="theme-preview">
    <p class="tp-name"><a href="./" aria-current="page">vitepress</a> · the default — no overrides</p>
    <div class="tp-swatches">
      <span style="background: var(--vp-c-brand-1)"></span>
      <span style="background: var(--vp-c-brand-2)"></span>
      <span style="background: var(--vp-c-brand-3)"></span>
    </div>
  </div>
  <div class="theme-preview" x-data="themePreview('green')">
    <p class="tp-name"><a href="./themes/green/">green</a> · <code>var(--vp-c-green-*)</code></p>
    <div class="tp-swatches">
      <span style="background: var(--vp-c-brand-1)"></span>
      <span style="background: var(--vp-c-brand-2)"></span>
      <span style="background: var(--vp-c-brand-3)"></span>
    </div>
  </div>
  <div class="theme-preview" x-data="themePreview('purple')">
    <p class="tp-name"><a href="./themes/purple/">purple</a> · <code>var(--vp-c-purple-*)</code></p>
    <div class="tp-swatches">
      <span style="background: var(--vp-c-brand-1)"></span>
      <span style="background: var(--vp-c-brand-2)"></span>
      <span style="background: var(--vp-c-brand-3)"></span>
    </div>
  </div>
  <div class="theme-preview" x-data="themePreview('orange')">
    <p class="tp-name"><a href="./themes/orange/">orange</a> · <code>var(--vp-c-orange-*)</code></p>
    <div class="tp-swatches">
      <span style="background: var(--vp-c-brand-1)"></span>
      <span style="background: var(--vp-c-brand-2)"></span>
      <span style="background: var(--vp-c-brand-3)"></span>
    </div>
  </div>
  <div class="theme-preview" x-data="themePreview('mono')">
    <p class="tp-name"><a href="./themes/mono/">mono</a> · neutral grays</p>
    <div class="tp-swatches">
      <span style="background: var(--vp-c-brand-1)"></span>
      <span style="background: var(--vp-c-brand-2)"></span>
      <span style="background: var(--vp-c-brand-3)"></span>
    </div>
  </div>
  <div class="theme-preview" x-data="themePreview('ocean')">
    <p class="tp-name"><a href="./themes/ocean/">ocean</a> · GitHub-blue brand</p>
    <div class="tp-swatches">
      <span style="background: var(--vp-c-brand-1)"></span>
      <span style="background: var(--vp-c-brand-2)"></span>
      <span style="background: var(--vp-c-brand-3)"></span>
    </div>
  </div>
  <div class="theme-preview" x-data="themePreview('sakura')">
    <p class="tp-name"><a href="./themes/sakura/">sakura</a> · rose-pink, 桜</p>
    <div class="tp-swatches">
      <span style="background: var(--vp-c-brand-1)"></span>
      <span style="background: var(--vp-c-brand-2)"></span>
      <span style="background: var(--vp-c-brand-3)"></span>
    </div>
  </div>
  <div class="theme-preview" x-data="themePreview('ember')">
    <p class="tp-name"><a href="./themes/ember/">ember</a> · <code>var(--vp-c-red-*)</code></p>
    <div class="tp-swatches">
      <span style="background: var(--vp-c-brand-1)"></span>
      <span style="background: var(--vp-c-brand-2)"></span>
      <span style="background: var(--vp-c-brand-3)"></span>
    </div>
  </div>
</div>
:::

## Syntax color schemes

The UI palette is one knob; the other is the `[syntax]` section, which takes any of the 218 Helix editor themes bundled with rustpress (or a path to your own TOML). One `syntax.css` per site (no rebuild needed to switch), so here a sample code block under the default `github-light` / `github-dark` pair — the [theming guide](/guide/theming#syntax-color-scheme) has the full story:

```toml [rustpress.toml]
theme = "ocean"

[syntax]
light = "catppuccin_latte"
dark = "catppuccin_mocha"
```

To try a different scheme here, edit the showcase config and run `npm run build:showcase`.
