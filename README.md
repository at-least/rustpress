# gen-docs

**A standalone documentation site generator in Rust, speaking VitePress's content format.** Content is ordinary VitePress markdown — YAML front matter, `:::` containers, `[!NOTE]` GitHub alerts, labeled code fences, code groups, `<Badge>` — with no Node runtime at render time: a single Rust binary parses and renders every page (comrak + tree-sitter + hypertext `rsx!` templates), and the little client-side interactivity is Alpine.js over server-rendered markup. Styling is Tailwind CSS v4 with the VitePress default theme's `--vp-*` design tokens.

![Zola-free](https://img.shields.io/badge/zola-not%20required-informational) ![rust](https://img.shields.io/badge/rust-1.85%2B-orange) (edition 2024)

## Why

This repo started life as the VitePress default theme ported to Zola. Zola's constraints leaked everywhere: TOML front matter, Tera component calls replacing `:::` containers (`{% <tip kind="tip" title="" no_title={false}> %}` with every parameter mandatory), sidebars derived from directory `_index.md`s because Zola has no sidebar config, a Zola-specific syntax-highlighting pipeline. The content was born VitePress; the format conversion was pure tax. So the renderer became a Rust program and the Zola layer went away:

- **Content format = VitePress**, verbatim. Re-syncing the demo from `vitepress/docs/en` is a file copy.
- **Renderer = Rust**: comrak (GFM, alerts, footnotes, GitHub-style heading ids) behind a fence-aware preprocessor that expands `:::` containers into HTML blocks (the markdown-it-container trick — inner markdown still parses in the same comrak pass), rewrites ```` ```js{1,3-4} [npm] ```` info strings, inlines `<<< @/path` includes, and rewrites `<Badge>`. Syntax highlighting is tree-sitter (grammars compiled in) with capture-name classes, themes authored in Helix TOML format, producing a `syntax.css` scoped `html.dark` inside `@layer syntax`.
- **Markup = hypertext `rsx!`** (see `src/render/`): the whole VitePress default theme — navbar, mobile nav screen, auto sidebar, local nav, right-hand outline, doc footer with pager/edit link, home hero/features, search modal, 404.
- **Interactivity = Alpine.js 3.17** (bundled with esbuild): scrollspy, sidebar drawer and carets, flyouts, appearance toggle (the anti-FOUC script stays vanilla in `<head>`; Alpine can't run pre-paint), code-group tabs, copy buttons, and the local search modal (Ctrl/Cmd+K, `/`).
- **Search** = a `search-docs.json` (url/title/body per page) built by Rust, scored client-side (title-exact +20, title hits +5, body occurrences capped at +20/token, top 20).

## Layout

```
Cargo.toml            crate gen-docs (bin + lib)
src/                  config, content loader, sidebar resolution,
                      markdown pipeline (preprocess/highlight), hypertext
                      renderers, site assembly
demo/                 dogfood site: gen-docs.toml + content/ (VitePress
                      format, mirrored from vitepress.dev) + static/
assets/themes/        vendored github-light/dark .tmTheme files
styles/               Tailwind entry (vitepress.css) + Inter @font-face
js/alpine-entry.js    the Alpine bundle source
tests/fixtures/en/    verbatim subset of vitepress/docs/en used by tests
```

## Build & develop

```sh
npm install          # tailwindcss CLI + esbuild + alpinejs (assets only)
npm run build        # bundle app.js, build main.css, cargo build + demo build
npm test             # cargo test + full demo build
npm run dev          # tailwind/esbuild watch + gen-docs serve (stage 8)
```

The Rust build needs only `cargo` (no Node). The committed `static/main.css` is built by the Tailwind CLI from `styles/vitepress.css` + class strings living in `src/**/*.rs` (`@source "../src"`), so `cargo build` alone suffices for Rust-side changes that don't touch classes.

## Site shape

- `gen-docs.toml` — site config mirroring VitePress's `themeConfig`: `title` + `titleTemplate` (`:title`), `description`, `lang`, `base`, `srcDir`, `nav` (plain links with `activeMatch`, dropdowns), `sidebar` (absent → one auto-derived per top-level section; explicit single array; or VitePress's path-keyed `{ base, items }` map with tri-state `collapsed`), `socialLinks`, `editLink` (`:path` pattern), `footer`, `outline` (level + label), `search.provider = "local"`, `appearance` (`true`/`false`/`"dark"`/`"force"`/`"force-auto"`), `lastUpdated` (git-based) + `lastUpdatedText`, `ignoreDeadLinks` (`true` or link prefixes), `[sitemap]` (hostname → sitemap.xml), `[[head]]` extra tags, `[docFooter]` prev/next labels, `[notFound]` title/quote/linkText, `returnToTopLabel`, `darkModeSwitchLabel`, `skipToContentLabel`, `[rewrites]` (source-path mapping with `:rest*`), `[locales]` (multi-language sites), `[markdown]` (lineNumbers, codeCopyButton, math, image.lazyLoading, container labels + custom containers), `[syntax]` (the source-code color scheme — a separate setting from the UI palette). There is no swappable theme system: one design, compiled in. Colors are customizable two ways (see below).
- `content/**/*.md` — VitePress format. URLs are directory-style: `guide/x.md` → `/guide/x/`, `index.md` → `/`. Titles come from the first H1 (fence-aware); front matter keys honored: `description`, `title`, `layout: home` (+ `hero`/`features`), `outline: deep` (or a level/level-pair).
- `static/` — copied verbatim into the output root.

### Markdown extensions supported

| VitePress | gen-docs |
| --- | --- |
| `::: tip` / `warning` / `danger` / `note` / `info` / `important` / `caution` (custom titles, `{no-title}`, nested `::::`) | same syntax; labels + custom container kinds configurable |
| `::: details SUMMARY {open}` | same |
| `::: code-group` with ```` ```sh [npm] ```` fences | server-emitted tab strip |
| ```` ```js{1,3-4} ```` | `<span class="line hl">` highlighting |
| ```` ```js:line-numbers ```` / `:no-line-numbers` / `:line-numbers=2` + global `markdown.lineNumbers` | CSS-counter line numbers |
| `[!code highlight]` / `[!code focus]` / `[!code ++]` / `[!code --]` / `[!code warning]` / `[!code error]` (+ `:N` counts, `[!!code …]` escape) | per-line classes + styling |
| `> [!NOTE]` … GitHub alerts | rendered |
| `<<< @/snippets/x.ts` (incl. `#region`, `[label]`, `.ansi` stripping, `{1,3-4}` lines, `{ts:line-numbers}`) | inlined at build time |
| `<Badge type="info" text="composable" />` | `<span class="VPBadge info">…</span>` |
| front matter `outline: deep` | deeper right-hand outline |
| `[[toc]]` | nested `<nav class="table-of-contents">` |
| emoji shortcodes (`:tada:`) | rendered |
| `## Heading {#custom-id}` | custom anchor id (slug replaced) |
| `$…$` / `$$…$$` math (`markdown.math`) | MathJax (tex-svg) injected on math pages |
| `markdown.image.lazyLoading` | `loading="lazy"` on content images |

Relative `.md`/`.html` links resolve to canonical page URLs at build time (unknown targets pass through untouched, and the dead-link checker reports them unless ignored).

### Color customization (`theme.toml` + `[markdown.theme]`)

Following VitePress's "extending the default theme", UI colors are customized by overriding root-level CSS custom properties. Create a `theme.toml` next to `gen-docs.toml`:

```toml
[light]
c-brand-1 = "#508d3f"       # → --vp-c-brand-1 (the --vp- prefix is optional)
c-brand-2 = "#629a4e"
"--vp-nav-bg-color" = "#f6f6f6"   # full variable names also work

[dark]
c-brand-1 = "#83aa63"
```

The build generates `theme.css` (`:root` / `.dark` custom-property overrides) and every page links it after `main.css`. Because Tailwind utilities (`text-brand-1`, `bg-bg`, …) and the hand-written component rules both reference the `--vp-*` variables via `@theme inline`, overriding the variable reaches everything — no new classes, no CSS rebuild.

The **source-code syntax color scheme is a separate setting** — `[syntax]` in `gen-docs.toml`. Each of `light`/`dark` takes either a built-in name or a path to your own Helix TOML theme file (relative to the site dir). Built-in names: `github-light` / `github-dark` (vendored defaults), plus **all 218 Helix editor themes** are bundled and selectable by file stem:

```toml
[syntax]
light = "github-light"
dark = "catppuccin_mocha"        # any of the 218 bundled Helix themes
# dark = "themes/my-dark.toml"   # or your own Helix/TextMate-style TOML
```

Helix theme files map dotted tree-sitter capture scopes to colors, support `[palette]` named colors and `inherits` chains (resolved longest-prefix-first, cycle-safe).

### Multi-language sites

`[locales.root]` + `[locales.zh]` (label/lang/title/description) with per-locale content directories (`content/zh/**` → `/zh/**`); pair with `[rewrites]` `"en/:rest*" = ":rest*"` for VitePress's canonical `en/` layout. The navbar gets a language flyout and the mobile menu a language list; links target the same page in the other locale when it exists, else the locale root. Per-locale `lang` attribute, title and description.

## Testing

`cargo test` runs 70+ unit and integration tests; the markdown pipeline's golden tests execute the real `tests/fixtures/en` corpus end to end, and `tests/site_build.rs` builds a full site into a temp dir and checks the outputs (pages, 404, syntax.css, search index, Alpine landmarks, tag balance).

## Caveats

- Requires `cargo` 1.85+; Node 18+ only for rebuilding the CSS/JS assets (the built artifacts are committed).
- CJK search recall is whitespace-tokenization-grade, same as before.
- Math typesetting runs client-side (MathJax tex-svg from a CDN, injected only on pages containing math) — VitePress renders it at build time.
- Team page/sponsors are not implemented (the demo never configured them).
- No Vue runtime, so Vue components/`<script setup>`/`{{ }}` in markdown stay literal; no Algolia/Carbon ads (external services).

## License

CSS variables, fonts (Inter) and icons are ported from VitePress (MIT). Everything else is released under MIT.
