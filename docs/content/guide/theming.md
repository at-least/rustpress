---
outline: deep
description: Customize the rustpress default theme with your own CSS variables, different fonts, and a separate syntax color scheme for code blocks.
---

# Theming

rustpress ships exactly one theme, a port of the VitePress default theme, compiled into the binary. Consult the [Default Theme Config Overview](../reference/default-theme-config) for the options that shape its behavior. What its colors and fonts look like is controlled by two independent settings:

1. the **UI colors** — the `theme` key, a path to a CSS file of your own;
2. the **syntax color scheme** for code blocks — the `[syntax]` section, which takes Helix editor theme files.

::: tip {no-title}
The two are deliberately separate: switching the UI to a purple brand color does not touch code colors, and picking `catppuccin_mocha` for code does not touch the UI.
:::

## UI Colors

Set `theme` to a path (relative to the site directory) ending in `.css`. The file is copied verbatim to `theme.css` in the output and linked on every page **after** the theme's own stylesheet, so its rules win:

```toml [rustpress.toml]
theme = "theme.css"
```

```css [theme.css]
:root {
  --vp-c-brand-1: #646cff;
  --vp-c-brand-2: #747bff;
}

.dark {
  --vp-c-brand-1: #8b91ff;
}
```

The design is expressed entirely through the same `--vp-*` custom properties VitePress uses (see the [default theme CSS variables](https://github.com/vuejs/vitepress/blob/main/src/client/theme-default/styles/vars.css) upstream; `styles/vitepress.css` in the rustpress repository is the local copy). Both the hand-written component rules and the utility classes resolve through those variables, so overriding a variable reaches everything that uses it — no rebuild required. Any CSS is allowed in the file, including ordinary selectors, `color-mix()` and gradients.

Unset, `theme` gives the stock VitePress look; nothing extra is emitted. Any other value fails at load time with an error naming the path it looked for.

### Navbar

The navbar draws a single background surface controlled by CSS variables, so its look can be changed without touching component internals:

```css
:root {
  /* bar height and background */
  --vp-nav-height: 4rem;
  --vp-nav-bg-color: var(--vp-c-bg);

  /* background while on top of the home page (unscrolled);
     set to var(--vp-nav-bg-color) to opt out of the transparent treatment */
  --vp-nav-home-bg-color: transparent;

  /* filter applied to the content behind the bar */
  --vp-nav-backdrop-filter: none;

  /* the bar's bottom rule and the mobile menu background */
  --vp-nav-divider-color: var(--vp-c-gutter);
  --vp-nav-screen-bg-color: var(--vp-c-bg);

  /* the logo image height */
  --vp-nav-logo-height: 1.5rem;
}
```

For example, a frosted-glass navbar:

```css
:root {
  --vp-nav-bg-color: color-mix(in srgb, var(--vp-c-bg) 65%, transparent);
  --vp-nav-backdrop-filter: saturate(180%) blur(8px);
}
```

The local nav (the "Menu / On this page" bar on small screens) follows the navbar surface: `--vp-local-nav-bg-color` defaults to `var(--vp-nav-bg-color)`.

::: warning
`backdrop-filter` has a measurable scroll performance cost, especially on large or high-DPI screens. When using a translucent bar, also check text contrast over your page content.
:::

Nav items that do not fit the navbar width wrap or scroll; there is no overflow `⋯` menu.

## Using Different Fonts

rustpress uses [Inter](https://rsms.me/inter/) as the default font. The font files are part of the theme's static assets and are always shipped; there is no font-less variant of the theme. To use a different font, override the font variables in your theme CSS file and provide the `@font-face` rules yourself:

```css [theme.css]
@font-face {
  font-family: "My Font";
  src: url("/fonts/my-font.woff2") format("woff2");
  font-display: swap;
}

:root {
  --vp-font-family-base: "My Font", ui-sans-serif, system-ui, sans-serif; /* normal text font */
  --vp-font-family-mono: ui-monospace, "SFMono-Regular", Menlo, monospace; /* code font */
}
```

Put the font files in the site's `static/` directory (see [Asset Handling](./asset-handling)). To preload them, add a `[[head]]` entry in `rustpress.toml`:

```toml [rustpress.toml]
[[head]]
tag = "link"
attrs = { rel = "preload", href = "/fonts/my-font.woff2", as = "font", type = "font/woff2", crossorigin = "" }
```

## Syntax Color Scheme

Code blocks are highlighted by tree-sitter and colored by a pair of themes — one for light mode, one for dark — set in the `[syntax]` section. Each value is one of:

- `github-light` / `github-dark`, the defaults;
- the file stem of any of the **218 themes bundled with the [Helix](https://helix-editor.com/) editor**, for example `catppuccin_latte`, `catppuccin_mocha`, `gruvbox`, `nord`, `onedark`, `solarized_light`, `tokyonight`;
- a path, relative to the site directory, to a Helix-format TOML theme file of your own (the value must end in `.toml`).

```toml [rustpress.toml]
[syntax]
light = "github-light"
dark = "catppuccin_mocha"
# dark = "themes/my-dark.toml"   # or your own file
```

The build generates a `syntax.css` from the pair: one rule per capture name for the light theme, the same set scoped under `html.dark` for the dark theme. A name that is neither built-in nor a `.toml` path fails the build with a message saying so.

### Writing a theme file

A Helix theme maps dotted tree-sitter capture scopes to styles. Colors are literal hex values or names from the file's `[palette]` table; `inherits` chains onto a built-in name or another file, so a theme can be a handful of overrides:

```toml [themes/my-dark.toml]
inherits = "github-dark"

[palette]
red0 = "#f97583"

"keyword" = { fg = "red0", modifiers = ["bold"] }
"string" = "#9ecbff"
"comment" = { fg = "#6a737d", modifiers = ["italic"] }
```

Resolution follows Helix: a capture matches the longest scope prefix the theme defines (`function.builtin` falls back to `function`, then to nothing). The foreground color, the background color and the `bold`, `italic` and `underline` modifiers are turned into CSS; other modifiers are ignored.

## Appearance Toggle

Dark mode is built in: the navbar switch flips the `.dark` class on `<html>`, the choice is remembered, and an inline script applies it before first paint so there is no flash. Where the browser supports the View Transitions API the flip is animated with `document.startViewTransition`, and falls back to an instant switch elsewhere. Use the [`appearance`](../reference/site-config#appearance) option to change the default mode, disable the toggle, or force one mode.

## What you cannot do

The theme is compiled in, so the customization surface stops at CSS. There is no way to register components, fill layout slots, or override internal components — everything in the VitePress guide that starts with `.vitepress/theme/index.js` has no counterpart here. See [Coming from VitePress](./coming-from-vitepress) for the full list of what does not carry over.
