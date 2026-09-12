---
title: Syntax Highlight Gallery
description: Try every built-in syntax highlight theme live — pick one and every code block on the page re-colors instantly.
sidebar: false
outline: false
---

# Syntax Highlight Gallery

Code blocks have their own colors — the **syntax highlight**. Every bundled theme is below. **Pick a card and every code block on this page re-colors instantly** — the choice lasts for this browser session, and the stock card restores the default.

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
[syntaxHighlight]
light = "github_light"       # the defaults — any name from the cards above
dark = "catppuccin_mocha"
```

One line each for light and dark mode — any name from the cards above.
