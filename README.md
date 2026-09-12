# rustpress

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
Cargo.toml            crate rustpress-cli (bin + lib both named rustpress)
src/                  config, content loader, sidebar resolution,
                      markdown pipeline (preprocess/highlight), hypertext
                      renderers, site assembly
docs/                 rustpress's own documentation, built by rustpress
                      (the vitepress.dev page tree rewritten for what
                      rustpress implements): `npm run build:docs`
demo/                 parity site: rustpress.toml + content/ (VitePress
                      format, mirrored verbatim from vitepress.dev) + static/
assets/themes/        vendored github-light/dark .tmTheme files
static/               theme assets embedded into the binary: fonts +
                      the npm-built vitepress.css and js/app.js (gitignored)
styles/               Tailwind entry (vitepress.css) + Inter @font-face
js/alpine-entry.js    the Alpine bundle source
tests/fixtures/en/    verbatim subset of vitepress/docs/en used by tests
```

## Build & develop

```sh
npm install          # tailwindcss CLI + esbuild + alpinejs (assets only)
npm run build        # bundle app.js, build vitepress.css, cargo build + demo build
npm test             # cargo test + full demo build + upstream parity gate
npm run dev          # tailwind/esbuild watch + rustpress serve demo
npm run build:docs   # build docs/ (rustpress's own documentation) → docs/public
npm run dev:docs     # same watch loop over docs/
```

Before the first `npm test`, run `bash scripts/sync-upstream.sh` once to
clone vuejs/vitepress at the tag pinned in `parity/upstream-ref.txt` —
the upstream parity gates fail (rather than skip) without it;
`RUSTPRESS_ALLOW_NO_UPSTREAM=1` opts out on offline machines.

The user-facing documentation lives in `docs/` and is itself a rustpress
site; `cargo test --test docs_site` builds it and validates every link and
anchor.

`static/vitepress.css` and `static/js/app.js` are embedded into the binary at compile time (`build.rs` refuses to build without them), so run `npm install && npm run build:js && npm run build:css` once before the first `cargo build`. `vitepress.css` is built by the Tailwind CLI from `styles/vitepress.css` + class strings living in `src/**/*.rs` (`@source "../src"`); after that, `cargo build` alone suffices for Rust-side changes that don't touch classes. A binary installed from the crate is self-contained.

## Upstream parity

When VitePress ships a new version, a weekly scheduled workflow
(`drift-watch`) opens a mechanical refresh PR: re-pin landmark
fingerprints from the deployed site, re-sync the verbatim corpora, and
attach the old→new diff plus whatever divergences remain. Locally the
same moves are `npm run parity:refresh` (theme axis) and
`npm run diff:upstream` (content axis: byte-diff `demo/content` against
the pinned vuejs/vitepress clone); `npm run check:parity` gates `npm
test` until every divergence is fixed or reviewed into
`parity/known-deltas.json`. See [PARITY.md](PARITY.md).

The **feature-surface** audit — every documented upstream config option, markdown extension, and theme feature against what rustpress implements, with source references — lives in [FEATURE-PARITY.md](FEATURE-PARITY.md); `cargo test --test feature_parity` fails when upstream docs grow headings the audit doesn't cover.

## Site shape

- `rustpress.toml` — site config mirroring VitePress's `themeConfig`: `title` + `titleTemplate` (`:title`), `description`, `lang`, `base`, `srcDir`, `nav` (plain links with `activeMatch`, dropdowns), `sidebar` (absent → one auto-derived per top-level section; explicit single array; or VitePress's path-keyed `{ base, items }` map with tri-state `collapsed`), `socialLinks`, `editLink` (`:path` pattern), `footer`, `outline` (level + label), `search.provider = "local"`, `appearance` (`true`/`false`/`"dark"`/`"force"`/`"force-auto"`), `lastUpdated` (git-based) + `lastUpdatedText`, `ignoreDeadLinks` (`true` or link prefixes), `[sitemap]` (hostname → sitemap.xml), `[[head]]` extra tags, `[docFooter]` prev/next labels, `[notFound]` title/quote/linkText, `returnToTopLabel`, `darkModeSwitchLabel`, `skipToContentLabel`, `[rewrites]` (source-path mapping with `:rest*`), `[locales]` (multi-language sites), `[markdown]` (lineNumbers, codeCopyButton, math, image.lazyLoading, container labels + custom containers), `[syntaxHighlight]` (code colors — a separate setting from the theme). There is no swappable theme system: one design, compiled in. Colors are customizable two ways (see below).
- `content/**/*.md` — VitePress format. URLs are directory-style: `guide/x.md` → `/guide/x/`, `index.md` → `/`. Titles come from the first H1 (fence-aware); front matter keys honored: `description`, `title`, `titleTemplate`, `head`, `layout` (`home` + `hero`/`features`, `doc`, `page`), `outline` (`deep`, a level/level-pair, or `false`), `navbar`, `sidebar`, `aside`, `editLink`, `footer`, `lastUpdated` (bool or a date string), `pageClass`, `search: false`, and `prev`/`next` (text, `{text, link}`, or `false`).
- `static/` — copied verbatim into the output root.

### Markdown extensions supported

| VitePress | rustpress |
| --- | --- |
| `::: tip` / `warning` / `danger` / `note` / `info` / `important` / `caution` (custom titles, `{no-title}`, nested `::::`) | same syntax; labels + custom container kinds configurable |
| `::: details SUMMARY {open}` | same |
| `::: raw` | `<div class="vp-raw">` wrapper |
| `> [!NOTE]`… alerts incl. `[!DANGER]`, custom titles | rendered as containers (styled, labels configurable) |
| `::: code-group` with ```` ```sh [npm] ```` fences | server-emitted tab strip |
| ```` ```js{1,3-4} ```` | `<span class="line hl">` highlighting |
| ```` ```js:line-numbers ```` / `:no-line-numbers` / `:line-numbers=2` + global `markdown.lineNumbers` | CSS-counter line numbers |
| `[!code highlight]` / `[!code focus]` / `[!code ++]` / `[!code --]` / `[!code warning]` / `[!code error]` (+ `:N` counts, `[!!code …]` escape) | per-line classes + styling |
| `> [!NOTE]` … GitHub alerts | rendered |
| `<<< @/snippets/x.ts` (incl. `#region`, `[label]`, `.ansi` stripping, `{1,3-4}` lines, `{ts:line-numbers}`) | inlined at build time |
| `<!--@include: file.md-->` with `#region`/heading-anchor sections and `{a,b}` line ranges (also inside code fences, verbatim) | same |
| inline footnotes `^[…]` | rewritten to reference footnotes |
| `[text](url){target="_self"}` link attribute blocks | raw anchor with the attrs |
| `<Badge type="info" text="composable" />` | `<span class="VPBadge info">…</span>` |
| front matter `outline: deep` | deeper right-hand outline |
| `[[toc]]` | nested `<nav class="table-of-contents">` |
| emoji shortcodes (`:tada:`) | rendered |
| `## Heading {#custom-id}` | custom anchor id (slug replaced) |
| `$…$` / `$$…$$` math (`markdown.math`) | MathJax (tex-svg) injected on math pages |
| `markdown.image.lazyLoading` | `loading="lazy"` on content images |

Relative `.md`/`.html` links resolve to canonical page URLs at build time (unknown targets pass through untouched, and the dead-link checker reports them unless ignored).

### Color customization (`theme`)

Following VitePress's "extending the default theme", UI colors are customized with one CSS file of `--vp-*` custom-property overrides. A bundled theme is a complete design — one CSS file defining every design token (surfaces, text, borders, brand, semantics, shadows), light and dark, held to completeness, ramp-monotonicity and WCAG-contrast tests. 200 ship with the binary: 24 curated flagships plus every vendored Helix palette, auto-mapped by scripts/gen-themes.py. `theme` selects it by name, or by a path to your own file:

```toml
theme = "catppuccin"  # any bundled name — 200 ship with the binary
theme = "theme.css"   # your own file
```

```css
:root {
  --vp-c-brand-1: #508d3f;
  --vp-c-brand-2: #629a4e;
}
.dark {
  --vp-c-brand-1: #83aa63;
}
```

The selected file is linked on every page after the theme stylesheet (`/vitepress.css`), by its own name under `/themes/` — a custom file is copied there verbatim, bundled ones ship with the binary. Because Tailwind utilities (`text-brand-1`, `bg-bg`, …) and the hand-written component rules both reference the `--vp-*` variables via `@theme inline`, overriding the variable reaches everything — no new classes, no CSS rebuild. Unset, `theme` gives the stock VitePress look.

The **syntax highlight is a separate setting** — `[syntaxHighlight]` in `rustpress.toml`. Each of `light`/`dark` takes either a bundled theme name or a path to your own Helix TOML theme file (relative to the site dir). **All 218 Helix editor themes** are bundled and selectable by file stem — `github_light` / `github_dark` are the defaults:

```toml
[syntaxHighlight]
light = "github_light"
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
- Team page/sponsors, Algolia/Carbon ads (external services), and the
  navbar overflow `...` collapse (needs client-side measurement) are not
  implemented; see FEATURE-PARITY.md for the remaining gaps.
- No Vue runtime, so Vue components/`<script setup>`/`{{ }}` in markdown stay literal; no Algolia/Carbon ads (external services).

## License

CSS variables, fonts (Inter) and icons are ported from VitePress (MIT). Everything else is released under MIT.
