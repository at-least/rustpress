---
title: Syntax Theme Gallery
description: Try every built-in syntax theme live — pick one and every code block on the page re-colors instantly.
sidebar: false
outline: false
---

# Syntax Theme Gallery

Code blocks have their own colors — the **syntax theme**. Every theme bundled with rustpress is below. **Pick a card and every code block on this page re-colors instantly** — the choice lasts for this browser session, and the stock card restores the default.

::: raw
<div id="syntax-gallery"></div>
:::

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

```toml [rustpress.toml]
[syntax]
light = "github_light"       # the defaults — any name from the cards above
dark = "catppuccin_mocha"
```

One line each for light and dark mode. The [Theming](/guide/theming/#syntax-color-scheme) guide covers custom theme files and the rest of the details.
