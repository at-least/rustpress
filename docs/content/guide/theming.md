---
title: Theming
description: Change the colors — a UI theme and a syntax color scheme, two one-liners in rustpress.toml. Deeper customization is not the goal; VitePress is the tool for that.
---

# Theming

rustpress ships one design. The supported customization is **colors only** — two settings, each a one-liner in `rustpress.toml`. If you want to restyle components, swap fonts, or extend the theme itself, that complexity is out of scope by design: [VitePress](https://vitepress.dev) is the tool built for it.

## UI colors

```toml [rustpress.toml]
theme = "catppuccin"   # any bundled theme — try them in the Theme Gallery
```

Every bundled theme is previewed live in the [Theme Gallery](/themes/); the name in the card is the value. Unset, `theme` gives the stock look.

## Code colors

```toml [rustpress.toml]
[syntax]
light = "github_light"       # the defaults — any name from the gallery
dark = "catppuccin_mocha"
```

One line per color mode. Every bundled scheme is previewed live in the [Syntax Theme Gallery](/themes/syntax/); the name in the card is the value. The exact behavior is in the [`[syntax]` reference](../reference/site-config#syntax).

## What about everything else?

Any `--vp-*` CSS variable *can* be overridden in a theme CSS file of your own (copy a bundled one as the starting point) — possible, but undocumented and not recommended. Pick a different theme instead; that is what the two hundred of them are for.
