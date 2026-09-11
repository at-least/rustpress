# Upstream parity: tracking VitePress releases

rustpress re-implements the VitePress default theme, but its DOM
intentionally diverges (Tailwind utilities instead of upstream's
semantic classes, Alpine.js instead of Vue, tree-sitter `tk-*` code
spans instead of Shiki). So parity is *not* DOM identity — it is two
mechanical checks that together answer the question: **"VitePress
released a new version — which parts of rustpress need to change?"**

## The three axes

| axis | what it catches | mechanism |
|---|---|---|
| content | new/changed/removed pages, markdown source drift | `npm run diff:upstream` — byte diff of `demo/content` against a vuejs/vitepress clone's `docs/en`; enforced automatically by `cargo test --test upstream_sync` whenever the clone is present |
| theme/behavior | nav, sidebar, outline, doc footer, hero, features, footer, search, block structure | `npm run check:parity` — landmark fingerprints of our build vs the pinned snapshot `parity/upstream.json` |
| viewport | breakpoint behavior: responsive utilities, breakpoint media queries, geometry at every breakpoint boundary | `npm run check:viewport` — headless-Chromium geometry probes against goldens in `parity/viewport-goldens.json` (verified pixel-for-pixel against the pinned upstream), plus an exact count of the px media queries in the compiled CSS |

Both axes key off the same input: a vuejs/vitepress clone at the tag
pinned in `parity/upstream-ref.txt` (currently the tag matching the
deployed vitepress.dev release recorded in `parity/upstream.json`).
`bash scripts/sync-upstream.sh` clones or updates `../vitepress` to that
ref; the upstream-dependent tests (`upstream_sync`, `feature_parity`)
**fail** when the clone is missing — never silently skip — with that
command in the message. For offline local runs set
`RUSTPRESS_ALLOW_NO_UPSTREAM=1` to turn the failure into a visible
skip; CI always syncs the clone (`.github/workflows/ci.yml`).

The viewport axis needs no upstream clone — it compares against
recorded goldens — but it does need the `playwright-chromium`
devDependency installed (CI caches the browser). It follows the same
fail-by-default convention: `RUSTPRESS_ALLOW_NO_PLAYWRIGHT=1` turns a
missing install into a visible skip. Some goldens encode demo content
(feature count decides grid columns, title length decides wrap): after
legitimately editing `demo/content`, refresh with
`node scripts/parity-viewport.mjs --update` and eyeball the golden diff.

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

Usually you don't do anything: `drift-watch`
(`.github/workflows/drift-watch.yml`, weekly + on demand) compares the
deployed site's generator version with the pinned baseline and, when
upstream moved, opens a mechanical refresh PR — ref bump, corpus
re-sync, re-pinned fingerprints, the old→new landmark diff, and the
divergences that still remain (those are the human work). Review that PR
like the manual flow below; note CI does not auto-run on PRs opened
with `GITHUB_TOKEN`, so push an empty commit to the branch to get a run.

The manual flow, for doing it locally or when the automation can't:

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

- `parity/upstream-ref.txt`: the vuejs/vitepress tag every verbatim
  corpus and the parity baseline are pinned to. `drift-watch` bumps it
  mechanically; a manual bump means re-syncing the clone
  (`scripts/sync-upstream.sh`), the corpora, and the fingerprints
  together — the gates above keep the three from drifting apart.
- `parity/pages.txt`: keep representative pages — home, the code-heavy
  guide pages, deep-sidebar reference pages. `npm run diff:upstream`
  lists upstream pages you haven't pinned; add one whenever a new
  upstream feature lands there, or the theme axis can't see it.
- `tests/fixtures/en`: a verbatim subset of the upstream docs corpus.
  Never edit fixtures by hand — `cargo test --test upstream_sync`
  byte-compares them (and `demo/content`) against the pinned clone; the
  only legitimate change is a re-copy from upstream.
- `tests/parity/fixtures/`: committed HTML samples (upstream + local)
  unit-test the extractor; refresh them from the caches/next build when
  either side's markup changes (`local-*` comes from `demo/public`).
- `parity/cache/` is transient fetched HTML — not committed.
- The checker is a gate, not a goal: a divergence means *something
  changed upstream* — fix rustpress, or review it into
  `parity/known-deltas.json` with a reason. Never widen the checker to
  make noise disappear.
