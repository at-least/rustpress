---
title: Theme Gallery
description: Try every built-in rustpress theme live — pick one and the whole site re-skins instantly, light and dark.
sidebar: false
outline: false
---

# Theme Gallery

A theme is one CSS file defining every design token — surfaces, text, borders, brand, the semantic colors, shadows — for light and dark alike.

Pick a card below and **the site you are reading re-skins instantly** — every page, links, containers, code blocks, dark mode, everything. The choice lasts for this browser session; the stock card restores the default.

::: raw
<div id="theme-gallery"></div>
:::

## Using one in your site

```toml [rustpress.toml]
theme = "catppuccin"   # any name from the cards above
```

That is the whole integration: the theme's CSS ships in the binary, lands in your output under `themes/`, and every page links it after the base stylesheet. Your own design is a `.css` file of your own — copy any bundled file as the starting point. The [Theming](/guide/theming/) guide covers the details.

Code blocks have their own color layer, chosen from a different shelf: the [Syntax Theme Gallery](/themes/syntax/) demos all 218 bundled Helix schemes and the `github-light` / `github-dark` pair the same way.
