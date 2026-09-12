---
title: Syntax Theme Gallery
description: Try every built-in syntax color scheme live — pick one and every code block on the page re-colors instantly.
sidebar: false
outline: false
---

# Syntax Theme Gallery

Code blocks are colored by a [syntax color scheme](/guide/theming/#syntax-color-scheme): a pair of [Helix editor](https://docs.helix-editor.com/themes.html) theme files, one for light mode and one for dark. rustpress bundles all 218 Helix color schemes plus a `github-light` / `github-dark` pair.

Every card below paints the same snippet with that theme's own resolved colors, on its own background — dark schemes sit on their dark ground. **Pick a card and every code block on this page re-colors instantly.** The choice lasts for this browser session; the stock card restores the default.

::: raw
<div id="syntax-gallery"></div>
:::

Then watch the stage — this block, still highlighted by the page's own `[syntax]` pair until you pick:

```rust
fn render(site: &Site) -> Result<Vec<Page>> {
    let mut pages = Vec::new();
    for path in &site.paths {
        pages.push(render_page(path)?);
    }
    Ok(pages)
}
```

## Using one in your site

Each Helix file is a single scheme, so pick two — one per color mode:

```toml [rustpress.toml]
[syntax]
light = "github-light"       # any name from the cards above
dark = "catppuccin_mocha"
```

That is the whole integration: the themes are resolved at build time into a single `syntax.css` linked by every page. A value ending in `.toml` is instead read as a Helix theme file relative to your site dir, so your own scheme is just a theme file of your own. The [Theming](/guide/theming/#syntax-color-scheme) guide covers the details.
