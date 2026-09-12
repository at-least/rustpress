---
title: Theme Gallery
description: Try every built-in rustpress theme live — pick one and the whole site re-colors instantly, light and dark.
sidebar: false
outline: false
---

# Theme Gallery

A theme is one CSS file defining every design token — surfaces, text, borders, brand, the semantic colors, shadows — for light and dark alike.

Pick a card and **the site you are reading re-colors instantly** — every page, code blocks, dark mode. The choice lasts for this browser session; the stock card restores the default.

::: raw
<div id="theme-gallery"></div>
:::

## Using one in your site

```toml [rustpress.toml]
theme = "catppuccin"   # any name from the cards above
```

That is the whole integration: the theme's CSS ships in the binary, lands in your output under `themes/`, and every page links it after the base stylesheet. Unset, `theme` gives the stock look. Code blocks have their own colors — the [Syntax Highlight Gallery](/themes/syntax-highlight/).
