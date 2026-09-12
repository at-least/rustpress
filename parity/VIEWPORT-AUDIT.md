# Full-breakpoint audit: rustpress vs upstream VitePress

**Date:** 2026-09-12 · **Upstream:** VitePress v2.0.0-alpha.20 (pinned ref, local
docs build — `pnpm docs:build` in the `../vitepress` clone; verified via
`<meta name="generator">`) · **Ours:** demo build at audit time.

Where `scripts/parity-viewport.mjs` is a *regression gate* over a small
golden set, this audit is the broad sweep it was distilled from: a matrix
of **4 pages × 15 breakpoint-boundary widths × 4 states**, comparing a
mapped list of ~30 landmarks on both sides with `getBoundingClientRect`
plus `getComputedStyle` (geometry, font metrics, spacing, colors,
visibility; ±1 px tolerance), followed by side-by-side screenshot
inspection of every diverging cell. Landmark selectors are mapped
per-side because the DOM intentionally differs (Tailwind utilities +
`id`-named skeleton vs upstream semantic classes).

- Pages: `/`, `/guide/what-is-vitepress/`, `/guide/markdown/`,
  `/reference/default-theme-config/`
- Widths: 375, 390, 639/640, 767/768, 959/960, 1023/1024, 1279/1280,
  1439/1440, 1920 — both sides of every media query upstream uses
- States: light, dark (`vitepress-theme-appearance=dark`), scrolled
  +300 px (sticky navbar surface), mobile nav open (<768; the hamburger
  is hidden ≥768 on both sides — the nav screen carries the appearance
  toggle and links there, matching upstream)
- Re-run: `node scripts/viewport-audit.mjs` (`--shots` also saves
  screenshots to the gitignored `parity/audit/`; `--pages/--widths/
  --states` subset the matrix). Dumps land in
  `parity/audit/dump.json`/`findings.json`.

## Answer: Tailwind coverage is maximal, and the sweep found five real gaps (fixed)

**CSS axis.** The rendered DOM carries **zero inline `style=` attributes**
— every component style is Tailwind utility classes in the `rsx!`
templates. The stylesheet entry (`styles/vitepress.css`, ~810 lines)
splits into: `--vp-*` design tokens under `:root`/`.dark` (mirrors
upstream's token layer — not expressible as utilities), a small
base-parity block (button cursor, `overflow-wrap`, focus outlines,
`prefers-reduced-motion` — upstream's base.css equivalents), and three
justified raw rules: the external-link-icon mask (data-URI mask), the
code-block title bar (unlayered so it outranks the `.vp-doc` pre
utilities), and the code-line notation rules (added by this audit, see
below). Breakpoints are declared in px in `@theme` exactly matching
upstream's media queries, and axis 1 of the viewport gate pins the
exact media-query inventory.

**Geometry axis.** With the fixes below applied, doc pages are
pixel-stable against upstream across every width band: content column,
sidebar (320 px drawer <960, 272 static ≥960), local nav (48→52 at 960,
gone at 1280), aside outline (≥1280), footer, pager stacking, hero
steps (32/48/56), feature grid (1/2/4 columns), dark mode, scrolled
navbar surface, and the mobile nav screen + backdrop geometry all
match. All 84 code blocks on the code-heavy page render at identical
heights to upstream, and highlighted lines are full-bleed to the pixel
(`--vp-code-line-highlight-color` band = pre width, 688 px at 768).

### Fixed by this audit (each now pinned by a viewport golden)

| # | Divergence | Root cause | Fix |
|---|---|---|---|
| 1 | "Last updated" text at font-weight 400 vs upstream 500 | missing `font-medium` (the 2026-09 line-height fix stopped one property short of upstream's `VPLastUpdated` style) | `src/render/doc.rs` |
| 2 | Outline links at weight 500 vs upstream 400 | links inherited the container's `font-medium`; upstream's `.outline-link` sets an explicit 400 | explicit `font-normal` on both outline link renderers (`doc.rs`, `local_nav.rs`) |
| 3 | Navbar appearance switch visible <768 | upstream `.VPNavBarAppearance` is `display:none` until `width>=48rem` (the nav screen carries the toggle on phones) | `hidden md:flex` wrapper (`navbar.rs`) |
| 4 | Every code line ~2 px too tall (24-line block: 624 vs 571; +496 px page height on the code-heavy page) | all `.line` spans were `inline-block` + `min-h-[1lh]` + `w-full`; upstream keeps plain lines inline (the `\n` between spans forms the line boxes at exactly `--vp-code-line-height`) and gives only *backgrounded* lines `inline-block` + `width:calc(100%+3rem); margin:0 -1.5rem; padding:0 1.5rem` | inline line model + full-bleed classes for `.hl/.warning/.error/.diff.*`; focus notation now blurs non-focus lines like upstream's `.has-focused-lines`; diff lines gained the `::before` `+`/`-` gutter symbol with the until-then-unused `--vp-code-line-diff-*-symbol-color` tokens (`vpdoc.rs`, `vitepress.css`) |
| 5 | `<<< @/snippets/x.js{2}` rendered only line 2 | the brace spec was treated as a line *selection*; upstream's `{2}` *highlights* line 2 and renders every line | spec now passes through to the fence info (`hl=2`), include tests rewritten (`preprocess.rs`) |
| 6 | Code blocks inside `<details>`/custom blocks sat 16 px too low and lost their radius on phones | upstream scopes `.custom-block div[class*=language-] { margin: .5rem 0; border-radius: .5rem }`; ours kept the outer `my-4`/`-mx-6` | `[&_.custom-block_pre]:mx-0 [&_.custom-block_pre]:my-2 [&_.custom-block_pre]:rounded-lg` (`vpdoc.rs`) |

Verifying evidence for #4/#5: per-block height diff across all 84 pre
blocks went 2 differing (−47 px each) → 0; the two truncated snippet
includes render 3 lines again with line 2 highlighted. For #6: all 21
custom blocks match after the fix. A post-fix review pass caught two
latent issues in the new line CSS, also fixed: highlighted lines inside
`:line-numbers` blocks started their text 24 px left of the plain lines
(the full-bleed margin ate the code padding while the number gutter's
`padding-left` won the cascade — compensated to 2.75rem), and a brace
spec like `{1,2 :line-numbers}` silently dropped the flag (the emitted
fence now keeps `:line-numbers` on the lang token, where the info
rewriter looks for it).

### Explained differences (not gaps)

- **Navbar right-side composition.** Upstream's docs site carries an
  Ask-AI button (new v2 component), a translations flyout (multilingual
  site config), and a "more" menu; rustpress's demo navbar mirrors the
  shared components (search, menu links, version flyout, appearance,
  GitHub). This shifts first-menu-link x-position at some widths —
  config/component surface, not breakpoint CSS (tracked on the feature
  axis, see FEATURE-PARITY.md).
- **`.vp-doc` is `position:relative` upstream only after hydration**
  (Vue injects an inline style at runtime; the SSG HTML has none). No
  geometry impact; not reproduced by design.
- **Hero buttons** center their label with flex instead of upstream's
  fixed line-height; measured identical at every width including the
  375 px wrap boundary (labels never wrap).
- **DOM layering** differs where visuals don't: feature cards split
  bg/border and padding across nested elements (upstream:
  `.VPFeature` outer + padded box; ours: single padded card + boxed
  inner), code blocks wrap `pre` in a `div[class*=language-]` upstream
  vs styled `pre` here — per-property dumps line up once compared at
  the visual-card level.
### Remaining known differences

All located and quantified; none are breakpoint-CSS regressions.

- **Math tables (feature gap, not CSS).** The Markdown page's equations
  table renders raw `$…$` source here vs typeset MathML upstream, so the
  table runs ~130 px taller at 375 px. Already tracked on the feature
  axis (FEATURE-PARITY.md, "Math Equations" — client-side MathJax vs
  upstream build-time typesetting).
- **Two ~16 px segment offsets and one 8 px segment offset** on the
  markdown/reference pages (between the named h2 anchors: after
  *github-flavored-alerts*, before *import-code-snippets* /
  *image-lazy-loading*, and the default-theme-config intro). Every
  measured block type (pre, custom block, details, code group, table
  rows/cells, headings, lists, paragraphs) matches individually; the
  residue sits between blocks. Width-independent to weakly
  width-dependent, invisible without overlaying screenshots.
- **"Last updated" line width ±2 px** (274 vs 272 at 768): self-hosted
  Inter vs upstream's font stack hinting — same weight, size, and
  layout properties.
- **Heading anchors:** `emoji` vs `emoji-` slug for a heading ending in
  an emoji (comrak slug rule difference); anchor-only, no visual effect.
- **Empty highlighted line inside `:line-numbers` blocks:** ours shows a
  24 px band (the line-number counter pseudo-element gives the empty
  span a line box), upstream's collapses to 0. No occurrence in the
  corpus; noted for completeness. For the same pseudo-element reason, a
  diff line inside a `:line-numbers` block shows the number gutter but
  not the `+`/`-` symbol (upstream separates the two); also absent from
  the corpus.

### Flake note

Upstream hydrates the outline and nav widgets after `networkidle`; a
read issued immediately after `fonts.ready` can catch a half-hydrated
DOM (one probe measured `.outline-link` weight 400 vs 500 across runs —
the CSS truth is 400 from upstream's scoped style). The audit tool
settles 500 ms after `fonts.ready` before dumping.

## Maintenance

Re-run the audit after Tailwind refactors or upstream re-pins (the
matrix is the input for refreshing `parity/viewport-goldens.json`
coverage). The gate's golden set is the audit's permanent subset: any
new divergence class found here should graduate into a `CASES` entry.
