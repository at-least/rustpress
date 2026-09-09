# gen-docs

**A standalone documentation site generator in Rust, speaking VitePress's content format.** Content is ordinary VitePress markdown — YAML front matter, `:::` containers, `[!NOTE]` GitHub alerts, labeled code fences, code groups, `<Badge>` — with no Node runtime at render time: a single Rust binary parses and renders every page (comrak + syntect + hypertext `rsx!` templates), and the little client-side interactivity is Alpine.js over server-rendered markup. Styling is Tailwind CSS v4 with the VitePress default theme's `--vp-*` design tokens.

![Zola-free](https://img.shields.io/badge/zola-not%20required-informational) ![rust](https://img.shields.io/badge/rust-1.85%2B-orange) (edition 2024)

## Why

This repo started life as the VitePress default theme ported to Zola. Zola's constraints leaked everywhere: TOML front matter, Tera component calls replacing `:::` containers (`{% <tip kind="tip" title="" no_title={false}> %}` with every parameter mandatory), sidebars derived from directory `_index.md`s because Zola has no sidebar config, a Zola-specific syntax-highlighting pipeline. The content was born VitePress; the format conversion was pure tax. So the renderer became a Rust program and the Zola layer went away:

- **Content format = VitePress**, verbatim. Re-syncing the demo from `vitepress/docs/en` is a file copy.
- **Renderer = Rust**: comrak (GFM, alerts, footnotes, GitHub-style heading ids) behind a fence-aware preprocessor that expands `:::` containers into HTML blocks (the markdown-it-container trick — inner markdown still parses in the same comrak pass), rewrites ```` ```js{1,3-4} [npm] ```` info strings, inlines `<<< @/path` includes, and rewrites `<Badge>`. Syntax highlighting is syntect with scope classes, dual github-light/github-dark tmThemes (converted from Shiki's VS Code themes), producing a `syntax.css` scoped `html.dark` inside `@layer syntax`.
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
styles/               Tailwind entry (main.css) + Inter @font-face
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

The Rust build needs only `cargo` (no Node). The committed `static/main.css` is built by the Tailwind CLI from `styles/` + class strings living in `src/**/*.rs` (`@source "../src"`), so `cargo build` alone suffices for Rust-side changes that don't touch classes.

## Site shape

- `gen-docs.toml` — site config mirroring VitePress's `themeConfig`: `title`, `nav` (plain links with `activeMatch`, dropdowns), `sidebar` (absent → one auto-derived per top-level section; explicit single array; or VitePress's path-keyed `{ base, items }` map with tri-state `collapsed`), `socialLinks`, `editLink` (`:path` pattern), `footer`, `outline`, `markdown.theme` (light/dark tmTheme names), `search.provider = "local"`.
- `content/**/*.md` — VitePress format. URLs are directory-style: `guide/x.md` → `/guide/x/`, `index.md` → `/`. Titles come from the first H1 (fence-aware); front matter keys honored: `description`, `title`, `layout: home` (+ `hero`/`features`), `outline: deep` (or a level/level-pair).
- `static/` — copied verbatim into the output root.

### Markdown extensions supported

| VitePress | gen-docs |
| --- | --- |
| `::: tip` / `warning` / `danger` / `note` / `info` / `important` / `caution` (custom titles, `{no-title}`, nested `::::`) | same syntax |
| `::: details SUMMARY {open}` | same |
| `::: code-group` with ```` ```sh [npm] ```` fences | server-emitted tab strip |
| ```` ```js{1,3-4} ```` | `<span class="line hl">` highlighting |
| `> [!NOTE]` … GitHub alerts | rendered |
| `<<< @/snippets/x.ts` (incl. `#region`, `[label]`, `.ansi` stripping) | inlined at build time |
| `<Badge type="info" text="composable" />` | `<span class="VPBadge info">…</span>` |
| front matter `outline: deep` | deeper right-hand outline |
| `[!code highlight]`, `:line-numbers` | not yet (literal text / stripped), matching the old Zola build |

Relative `.md`/`.html` links resolve to canonical page URLs at build time (unknown targets pass through untouched).

## Testing

`cargo test` runs 70+ unit and integration tests; the markdown pipeline's golden tests execute the real `tests/fixtures/en` corpus end to end, and `tests/site_build.rs` builds a full site into a temp dir and checks the outputs (pages, 404, syntax.css, search index, Alpine landmarks, tag balance).

## Caveats

- Requires `cargo` 1.85+; Node 18+ only for rebuilding the CSS/JS assets (the built artifacts are committed).
- Single language (the old `fr` demo locale had no content; i18n is a later stage).
- CJK search recall is whitespace-tokenization-grade, same as before.
- Team page/sponsors from the Zola theme are deferred (the demo never configured them).

## License

CSS variables, fonts (Inter) and icons are ported from VitePress (MIT). Everything else is released under MIT.
