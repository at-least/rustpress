---
outline: deep
description: Customize the rustpress default theme with your own CSS variables, different fonts, and a separate syntax color scheme for code blocks.
---

# Theming

rustpress ships exactly one theme, a port of the VitePress default theme, compiled into the binary. Consult the [Default Theme Config Overview](../reference/default-theme-config) for the options that shape its behavior. What its colors and fonts look like is controlled by two independent settings:

1. the **UI colors** — the `theme` key, which selects a stylesheet by name (bundled) or by path (your own file);
2. the **syntax color scheme** for code blocks — the `[syntax]` section, which takes Helix editor theme files.

::: tip {no-title}
The two are deliberately separate: switching the UI to a purple brand color does not touch code colors, and picking `catppuccin_mocha` for code does not touch the UI.
:::

## UI Colors

`theme` picks the stylesheet to link on every page **after** the theme's own stylesheet (`vitepress.css`), so its rules win. Two ways to point at it:

### Bundled themes

Set `theme` to a bare name (`github`, `catppuccin`, `nord`, `rose-pine` — see the [Theme Gallery](/themes/)):

```toml [rustpress.toml]
theme = "catppuccin"
```

A bundled theme is a **complete design**: one file defining every design token the structure consumes — surfaces, text, borders, the brand ramp, the neutral ramp, the seven semantic colors (tip/note/success/important/warning/danger/caution) and the shadow scale — with light and dark values of its own. Linked after `vitepress.css`, it leaves none of the stock *color* design in effect — what survives from the base is the structural layer (layout, components, utility classes), which resolves through these tokens, plus a few opt-in knobs like fonts and the backdrop scrim a theme may also override by declaring the same custom properties.

Two repo tests hold every bundled theme to that bar: **completeness** (the required token set is derived from what the base stylesheet actually references; each theme must define all of it in both `:root` and `.dark`) and **WCAG contrast** (body text ≥ 7:1, secondary text ≥ 4.5:1 also on the sidebar and code-block surfaces, links ≥ 4.5:1, badge text vs its own soft container background ≥ 4.5:1, button backgrounds vs white text ≥ 3:1 in every state — in both modes). Where a published palette's accent cannot hold those ratios in a foreground role, the theme deepens it and says so in its header comment.

Every bundled theme is a plain CSS file shipped in the binary under `themes/`; all of them land in the output's `themes/` directory, and the selected one is linked as `/themes/<name>.css`. An unknown name fails at load time with an error listing the bundled names.

::: tip {no-title}
See them before you choose — the [Theme Gallery](/themes/) re-skins this entire site, live, one click per theme.
:::

### Your own CSS file

Set `theme` to a path (relative to the site directory) ending in `.css`. The file is copied verbatim into the output under `themes/<basename>` and linked there:

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

A file of your own can be a partial override (a few brand variables, like above) — that is customization. To author a full theme, copy a bundled `themes/<name>.css` out of the output (or from `static/themes/` in the repo) as your template and change the values; the bundled files are themselves just this contract spelled out. Note that a custom file takes its basename into `themes/`: your own `catppuccin.css` replaces the bundled `themes/catppuccin.css` in the output — `theme = "catppuccin.css"` resolves to your copy, not the bundled one.

Unset, `theme` gives the stock VitePress look; nothing extra is linked.

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
