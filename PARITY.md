# Upstream parity: tracking VitePress releases

rustpress re-implements the VitePress default theme, but its DOM
intentionally diverges (Tailwind utilities instead of upstream's
semantic classes, Alpine.js instead of Vue, tree-sitter `tk-*` code
spans instead of Shiki). So parity is *not* DOM identity — it is two
mechanical checks that together answer the question: **"VitePress
released a new version — which parts of rustpress need to change?"**

## The two axes

| axis | what it catches | mechanism |
|---|---|---|
| content | new/changed/removed pages, markdown source drift | `npm run diff:upstream` — byte diff of `demo/content` against a vuejs/vitepress clone's `docs/en` |
| theme/behavior | nav, sidebar, outline, doc footer, hero, features, footer, search, block structure | `npm run check:parity` — landmark fingerprints of our build vs the pinned snapshot `parity/upstream.json` |

The pinned baseline was extracted from the deployed upstream site
(vitepress.dev, `<meta name="generator">` records the exact version —
currently `VitePress v2.0.0-alpha.20`) for the 14 pages in
`parity/pages.txt`. These gates cover the *pinned pages*; the whole
documented feature surface (every config option, markdown extension,
theme feature) is audited in [FEATURE-PARITY.md](FEATURE-PARITY.md),
gated by `cargo test --test feature_parity`. Each page's fingerprint stores landmarks only:
text, hrefs, order, presence — never class names or markup. Fields the
upstream side cannot see are recorded as `null`/`[]` and skipped by the
checker (e.g. the deployed site hydrates outline items client-side);
`last_updated` compares presence only, since the timestamps come from
each repo's own git history.

## When VitePress releases a new version

```sh
# 0. see what changed upstream
git -C ../vitepress fetch origin
git -C ../vitepress log --oneline HEAD..origin/main          # commits
git -C ../vitepress diff HEAD origin/main -- docs/en         # content
git -C ../vitepress diff HEAD origin/main -- src client      # theme source

# 1. content axis: what changed in the docs corpus
git -C ../vitepress checkout origin/main
npm run diff:upstream          # byte diff demo/content vs docs/en
#   → re-copy changed pages from ../vitepress/docs/en into demo/content
#     (plus snippets/components when include targets change),
#     and update demo/rustpress.toml when their sidebar/nav config changed

# 2. theme axis: re-pin the baseline and see what changed upstream
npm run parity:refresh
#   → fetches the deployed site (which tracks the new release),
#     rewrites parity/upstream.json, and prints the old→new landmark diff
#   → each line names the page + landmark that changed upstream

# 3. fix rustpress until the gate is green
npm run check:parity
#   → every unexplained divergence prints as
#     "<page>: <landmark>: upstream X != ours Y" and exits non-zero
npm test                       # full suite, parity gate included

# 4. feature surface: audit new/changed upstream options
cargo test --test feature_parity
#   → fails on every docs/en reference heading FEATURE-PARITY.md
#     doesn't cover yet; update its rows (it also lists parsed-but-dead
#     config keys and intentional divergences with source refs)
```

To map a changed landmark to code, use the owner table below. When a
divergence is *intentional* (a documented, accepted difference), add an
entry to `parity/known-deltas.json` instead of changing code — every
entry requires a `reason`.

## Landmark → source map

| fingerprint landmark | upstream component | ours |
|---|---|---|
| `nav_title` | `VPNavBarTitle` | `src/render/navbar.rs` |
| `nav_items` (links + flyout groups) | `VPNavBarMenu(Link|Group)` | `src/render/navbar.rs` |
| `sidebar` (groups, links, depth) | `VPSidebar(Item)` | `src/render/sidebar.rs` + `src/sidebar.rs` (resolution, prev/next order) |
| `outline_title` / `outline_items` | `VPDocAsideOutline` | `src/render/doc.rs` (`outline_links`) |
| `pager` | `VPDocFooter` prev/next | `src/render/doc.rs` + `Sidebars::neighbors` |
| `edit_link` / `last_updated` | `VPDocFooter` edit-info | `src/render/doc.rs` |
| `site_footer` | `VPFooter` | `src/render/layout.rs` |
| `hero` / `features` | `VPHero` / `VPFeature` | `src/render/home.rs` |
| `has_search` | `VPNavBarSearch` | `src/render/navbar.rs` + `src/render/search_modal.rs` |
| `block_counts` | `.vp-doc` content | `src/markdown/` (preprocess + comrak) |
| extraction/check engine | — | `src/parity.rs` (`rustpress parity check/snapshot/diff`) |

## Reference source

The deployed vitepress.dev is the pin source (it always reflects the
released docs). `../vitepress` (a git clone of vuejs/vitepress) is the
content source and the place to read upstream theme source; the
deployed generator version and per-page etags are recorded in
`parity/upstream.json` → `meta` when you refresh.

## Maintenance rules

- `parity/pages.txt`: keep representative pages — home, the code-heavy
  `guide/markdown`, deep-sidebar reference pages. Add a page whenever a
  new upstream feature lands in the docs corpus.
- `tests/parity/fixtures/`: committed HTML samples (upstream + local)
  unit-test the extractor; refresh them from the caches/next build when
  either side's markup changes (`local-*` comes from `demo/public`).
- `parity/cache/` is transient fetched HTML — not committed.
- The checker is a gate, not a goal: a divergence means *something
  changed upstream* — fix rustpress, or review it into
  `parity/known-deltas.json` with a reason. Never widen the checker to
  make noise disappear.
