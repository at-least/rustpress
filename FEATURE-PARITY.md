# Feature parity: gen-docs vs upstream VitePress

This is the **feature-surface audit**: every documented upstream
capability, row by row, against what gen-docs actually does (claims
verified against source, not README prose). It complements
[PARITY.md](PARITY.md), whose two mechanical gates cover only the 14
pinned demo pages — this file answers "VitePress shipped a feature we
never looked at" for the *whole* documented surface.

- **Pinned upstream:** `vuejs/vitepress` @ `3e681e2` (v2.0.0-alpha.20,
  2026-09-10 audit)
- **Coverage gate:** `tests/feature_parity.rs` extracts every `##`/`###`
  heading from the four reference files in `../vitepress/docs/en` and
  fails if a heading is not covered here. No clone checked out → test
  skips. Matching is substring-based, so short generic names (`title`,
  `nav`) match incidentally — the gate's job is to catch new,
  distinctively-named upstream options.

| status | meaning |
|---|---|
| implemented | confirmed against source (column 3 cites it) |
| partial | subset of upstream; the note names what is missing |
| diverged | works, but by a documented different mechanism/shape |
| missing | upstream has it, we don't, no architectural blocker |
| n/a | needs a runtime gen-docs doesn't have (Vue/Vite/Node/external service) |

Status summary: implemented 49, partial 26, diverged 13, missing 42,
n/a 24 (154 audited rows).

## How to update on a new release

```sh
git -C ../vitepress fetch origin && git -C ../vitepress checkout origin/main
cargo test --test feature_parity     # fails on every heading not covered here
# → add/update rows for the failing headings, then re-audit only the
#   sections whose docs/en file changed (sections here mirror those
#   files 1:1, in the same order):
git -C ../vitepress diff <pinned>..HEAD --name-only -- docs/en
# → finally update the "Pinned upstream" line above
```

---

## reference/site-config.md

### Overview

Config resolution / intellisense / typed-theme-config are TypeScript &
tooling concerns of the `.vitepress/config.ts` ecosystem — gen-docs has
a single `gen-docs.toml` with `deny_unknown_fields`, so typos fail at
load (`src/config.rs:28`).

| heading | status | ours | note |
|---|---|---|---|
| Config Resolution | n/a | `gen-docs.toml` | no JS config cascade; one TOML file (`src/config.rs:205`) |
| Config Intellisense | n/a | — | no TS types to IntelliSense |
| Typed Theme Config | n/a | — | single compiled-in theme; no `defineConfig` surface |
| Vite, Vue & Markdown Config | n/a | — | no Vite/Vue anywhere in the pipeline |
| Page-Level Overrides | partial | `src/content.rs:17` | only `title`/`description`/`outline` (+ home keys) per page |
| Directory-Level Overrides | missing | — | no per-directory front matter files |

### Site Metadata

| heading | status | ours | note |
|---|---|---|---|
| title | implemented | `src/config.rs:31` | navbar text + `<title>` default (`src/render/mod.rs:364`) |
| titleTemplate | partial | `src/render/mod.rs:364-377` | `:title` + plain-suffix forms; `false` (drop suffix) unsupported |
| description | implemented | `src/config.rs:36` | meta tag; locale/frontmatter fallback (`src/render/mod.rs:183-189`) |
| head | partial | `src/config.rs:483`, `src/render/layout.rs:127-144` | site-level `[[head]]` only; no per-page/per-locale head, no dedup/merge |
| lang | implemented | `src/config.rs:41` | `<html lang>`; per-locale override (`src/render/mod.rs:356-362`) |
| base | implemented | `src/config.rs:46` | validated + prepended (`src/render/mod.rs:143-157`) |

### Routing

| heading | status | ours | note |
|---|---|---|---|
| cleanUrls | diverged | `src/content.rs:286-294` | gen-docs *always* emits directory URLs (`/guide/x/`); there is no `.html`-suffix mode to disable |
| rewrites | partial | `src/render/mod.rs:66-88` | static map + `:rest*` suffix capture only; no multi-param `:pkg/:slug*` patterns |

### Build

| heading | status | ours | note |
|---|---|---|---|
| srcDir | implemented | `src/config.rs:140` | default `content` (upstream default `.`) |
| srcExclude | missing | — | no exclusion globs |
| outDir | diverged | `src/render/mod.rs:260` | fixed `<site>/public`, not configurable |
| assetsDir | n/a | — | no Vite asset pipeline; assets copied verbatim, no hashing |
| assetsBase | missing | — | CDN prefix for generated assets; nothing hashed to serve |
| icons | n/a | — | iconify collection pipeline needs Node; our icons are compiled-in SVG |
| cacheDir | n/a | — | no Vite cache |
| ignoreDeadLinks | partial | `src/config.rs:445-471` | `true` or array of prefixes; no `localhostLinks`, regex, or function forms |
| mpa | n/a | — | gen-docs is always a static build with Alpine islands (MPA-shaped by construction) |

### Theming

| heading | status | ours | note |
|---|---|---|---|
| appearance | implemented | `src/config.rs:396-443` | `true`/`false`/`"dark"`/`"force"`/`"force-auto"` (upstream spells `force-dark`; we accept `force`), anti-FOUC script `src/render/layout.rs:149-170` |
| lastUpdated | implemented | `src/render/mod.rs:47-64` | git timestamps; upstream uses author date (`%ai`), we use committer date (`%ct`) |

### Customization

| heading | status | ours | note |
|---|---|---|---|
| markdown | partial | `src/config.rs:491-574` | ours: `lineNumbers`, `codeCopyButton`, `math`, `image.lazyLoading`, `container.*`; missing: `anchor`, `toc`, `theme`, `headers`, `snippet.*`, `include.*`, `config()` hook. Key divergence: upstream's image option is `lazyLoad`, ours `lazyLoading` |
| vite | n/a | — | no Vite |
| vue | n/a | — | no Vue |

### Build Hooks

buildEnd / postRender / transformHead / transformHtml / transformPageData —
all **n/a**: JavaScript build hooks need the Node build process gen-docs
replaces. The Rust binary has no plugin ABI; `[[head]]` covers the
common `transformHead` use.

## reference/default-theme-config.md

| heading | status | ours | note |
|---|---|---|---|
| i18nRouting | diverged | `src/render/mod.rs:324-353` | no toggle; language switcher always targets the same page in the other locale (upstream's default behavior), else the locale root |
| logo | partial | `src/config.rs:84`, `src/render/navbar.rs:66-68` | string path only; no `{src, alt}` / `{light, dark}` variants |
| siteTitle | missing | — | navbar always shows config `title`; no `false` to hide |
| nav | partial | `src/config.rs:234`, `src/render/navbar.rs:94-227` | `text`/`link`/`items`/`activeMatch` incl. flyouts; missing `target`/`rel`/`noIcon`, no overflow `...` collapse menu |
| sidebar | partial | `src/config.rs:251-295`, `src/sidebar.rs` | array + path-keyed multi-sidebar, tri-state `collapsed`, per-item/section `base`; missing per-item `docFooterText`/`rel`/`target` |
| aside | missing | — | no way to move the outline left or disable the aside |
| outline | partial | `src/config.rs:330`, `src/render/doc.rs:61-75` | `level` + `label`; `outline: false` (disable) unsupported |
| socialLinks | partial | `src/config.rs:299`, `src/render/navbar.rs:135-145` | named icons + `{svg}`; missing `ariaLabel`/`target` |
| footer | implemented | `src/config.rs:325`, `src/render/layout.rs:87-98` | `message`/`copyright`, hidden when a sidebar is shown (matches upstream) |
| editLink | implemented | `src/config.rs:315`, `src/render/doc.rs:90-97` | `pattern` (`:path`) + `text`; function pattern form n/a |
| lastUpdated | diverged | `src/config.rs:151` | ours is the flat `lastUpdatedText` key (upstream nests `lastUpdated.text`); no `formatOptions` — dates are fixed UTC format |
| algolia | missing | — | external service (README caveat) |
| carbonAds | missing | — | external service |
| docFooter | partial | `src/config.rs:590`, `src/render/doc.rs:37-41` | `prev`/`next` labels; `false` to disable one side unsupported |
| darkModeSwitchLabel | implemented | `src/render/navbar.rs:256,284` | |
| lightModeSwitchTitle | missing | — | toggle has no hover title |
| darkModeSwitchTitle | missing | — | toggle has no hover title |
| sidebarMenuLabel | missing | — | mobile menu heading hardcoded |
| returnToTopLabel | partial | `src/config.rs:153` | **parsed but never read** — `src/render/local_nav.rs:17` hardcodes "Return to top" (audit finding #1) |
| langMenuLabel | missing | — | a11y label for the language toggle |
| navMenuLabel | missing | — | a11y landmark label |
| mobileMenuLabel | missing | — | a11y label for the hamburger |
| extraMenuLabel | missing | — | (and the `...` overflow menu itself is missing, see `nav`) |
| skipToContentLabel | implemented | `src/render/layout.rs:41,71` | |
| externalLinkIcon | diverged | `src/render/vpdoc.rs:108` | arrow is always on; upstream defaults off, opt-in |
| gradedContainers | missing | — | no graded severity colors for containers/alerts/badges |
| useLayout | n/a | — | Vue composable |

## reference/frontmatter-config.md

| heading | status | ours | note |
|---|---|---|---|
| title | implemented | `src/content.rs:22,242-248` | falls back to first H1, then file stem |
| titleTemplate | missing | — | no per-page template |
| description | implemented | `src/content.rs:25` | |
| head | missing | — | no per-page head tags |
| Default Theme Only | — | — | subsections below |
| layout | partial | `src/content.rs:28`, `src/render/mod.rs:201-215` | `home` switches to the home renderer; `page` (unstyled) is treated as `doc`; custom component names n/a |
| hero | implemented | `src/content.rs:73-106`, `src/render/home.rs` | `name`/`text`/`tagline`/`image{src,alt}`/`actions`; see findings #5 for `target` |
| features | implemented | `src/content.rs:108-123`, `src/render/home.rs:81-113` | icon as raw HTML/emoji; no `{light,dark}`/`width`/`height` icon forms |
| navbar | missing | — | no per-page navbar toggle |
| sidebar | missing | — | no per-page sidebar toggle |
| aside | missing | — | no per-page aside control |
| outline | partial | `src/content.rs:31,38-70` | `deep`/level/range; `false` unsupported |
| lastUpdated | missing | — | no per-page toggle or Date override |
| editLink | missing | — | no per-page toggle |
| footer | missing | — | no per-page toggle |
| pageClass | missing | — | no extra page class hook |
| isHome | n/a | — | exists to force home detection for custom (Vue) layouts |

## guide/markdown.md

### Header Anchors / Links

| heading | status | ours | note |
|---|---|---|---|
| Header Anchors | implemented | `src/markdown/mod.rs:260-327` | GitHub-style slugs |
| Custom anchors | implemented | `src/markdown/mod.rs:260-327` | `{#custom-id}` replaces the slug |
| Links | — | | group heading |
| Internal Links | implemented | `src/markdown/mod.rs:183-258` | resolved to canonical URLs at build time (no router — static) |
| Page Suffix | diverged | `src/content.rs:286-294` | always directory URLs; `.html` suffix never emitted |
| External Links | diverged | probe 2026-09-10 | upstream adds `target="_blank" rel="noreferrer"` to outbound links; we add nothing (and the external arrow is always on — see `externalLinkIcon`) |
| Frontmatter | implemented | `src/content.rs:17` | YAML; unknown keys tolerated |
| GitHub-Style Tables | implemented | comrak GFM | |
| Task Lists | implemented | comrak GFM | |
| Footnotes | partial | comrak footnotes | `[^1]` references + definitions; inline `^[...]` stays literal (comrak doesn't support it — probe 2026-09-10) |
| Emoji :tada: | implemented | comrak | shortcodes rendered |
| Table of Contents | implemented | `src/markdown/preprocess.rs:306`, `src/markdown/mod.rs:329-372` | `[[toc]]` nested h2–h3; `markdown.toc` options n/a |

### Custom Containers

| heading | status | ours | note |
|---|---|---|---|
| Custom Containers | — | | group heading |
| Default Title | implemented | `src/markdown/preprocess.rs:259-390` | info/tip/warning/danger + note/important/caution kinds |
| Custom Title | implemented | `src/markdown/preprocess.rs:350-378` | text after the kind; `{no-title}` too |
| Registering New Containers | diverged | `src/config.rs:578-588` | `[markdown.container.custom]` `{name, kind, label}` reuses a builtin kind's styling; upstream's are unstyled + your CSS |
| Nesting | implemented | `src/markdown/preprocess.rs:269-321` | fence-length nesting (`::::`), incl. in list items |
| Additional Attributes | partial | `src/markdown/preprocess.rs:468-483` | `{open}` on details works; the generic attrs plugin (`{target=...}` on arbitrary elements) does not |
| raw | missing | — | no `::: raw` / `vp-raw` style isolation (our pages have no client styles to isolate) |
| GitHub-flavored Alerts | partial | comrak alerts | `> [!NOTE/TIP/IMPORTANT/WARNING/CAUTION]` + custom title text render; `[!DANGER]` (VitePress extension) falls back to a plain blockquote — probe 2026-09-10 |

### Code blocks

| heading | status | ours | note |
|---|---|---|---|
| Syntax Highlighting in Code Blocks | diverged | `src/markdown/highlight.rs` | tree-sitter `tk-*` spans by design (PARITY.md); fixed grammar set (bash/js/ts/tsx/json/yaml/toml/html/css/python/rust/go/c/cpp + aliases), others escape-only |
| Line Highlighting in Code Blocks | implemented | `src/markdown/preprocess.rs:485-572` | `{4,7-13,16}` + `[!code highlight]` |
| Focus in Code Blocks | implemented | `src/markdown/highlight.rs:320-369` | `[!code focus]` (+ `:N`), blur styling |
| Colored Diffs in Code Blocks | implemented | `src/markdown/highlight.rs:320-369` | `[!code ++]` / `[!code --]` |
| Errors and Warnings in Code Blocks | implemented | `src/markdown/highlight.rs:320-369` | `[!code error]` / `[!code warning]` |
| Line Numbers | implemented | `src/markdown/highlight.rs:425-442` | `:line-numbers`/`:no-line-numbers`/`=N` + global `markdown.lineNumbers` |
| Import Code Snippets | implemented | `src/markdown/preprocess.rs:109-242` | `<<< @/`, `#region`, `{lang}`, `[label]`, `{lines}`, `:line-numbers`, `.ansi` stripping; `snippet.stripRegionMarkers`/`silent` options n/a |
| Code Groups | implemented | `src/markdown/preprocess.rs:329-435` | server-emitted tab strip (radio inputs + Alpine) |
| Markdown File Inclusion | partial | `src/markdown/preprocess.rs:90-103,201-212` | recursive `<!--@include:-->` (depth 8); no line ranges `{3,}`, `#region`, header-section selection, `include.silent`, `rebaseRelativeUrls` |
| Including Code Files | missing | probe 2026-09-10 | the include directive inside code fences stays literal; use `<<<` imports instead |
| Math Equations | diverged | `src/markdown/mod.rs:77-79,164-178` | `$…$`/`$$…$$` via client-side MathJax CDN injected per page; upstream typesets at build |
| Image Lazy Loading | diverged | `src/config.rs:522-528` | works, but the key is `image.lazyLoading` (upstream: `image.lazyLoad`) |
| Advanced Configuration | n/a | — | `markdown.anchor`/`toc`/`config()` need the markdown-it JS engine |

## guide/routing.md

| feature | status | ours | note |
|---|---|---|---|
| File-based routing | implemented | `src/content.rs:286-294` | URL shape diverges (directory URLs) |
| srcDir | implemented | `src/config.rs:140` | |
| Linking between pages | partial | `src/markdown/mod.rs:183-258` | `.md`/`.html`/relative all resolve; link-attribute syntax `{target="_self"}` unsupported |
| cleanUrls | diverged | — | always on (see site-config row) |
| Route rewrites | partial | `src/render/mod.rs:66-88` | `:rest*` only |
| Dynamic routes (`[pkg].md` + `paths.js`) | n/a | — | JS data loaders |
| `defineRoutes` / `watch` / `$params` / `<!-- @content -->` | n/a | — | dynamic-route machinery |

## guide/i18n.md

| feature | status | ours | note |
|---|---|---|---|
| `locales` (root + per-locale) | partial | `src/config.rs:612-627`, `src/render/mod.rs:90-101` | `label`/`lang`/`description` honored; locale `title` parsed but unused (finding #3); no per-locale `head`/`themeConfig` |
| Language switcher | implemented | `src/render/mod.rs:324-353`, `src/render/navbar.rs:106-127` | same-page targeting, locale-root fallback |
| Per-locale markdown strings | missing | — | container labels/code-copy texts are global only |
| Separate dir per locale | implemented | `src/render/mod.rs:90-101` | `content/zh/**` → `/zh/**` |
| RTL (`dir`) | missing | — | no `dir` handling, experimental upstream |

## guide/asset-handling.md

| feature | status | ours | note |
|---|---|---|---|
| Referencing static assets | n/a | — | no Vite processing: no hashing, no <4kb inlining; relative asset URLs work as plain paths |
| Public directory | implemented | `src/render/mod.rs:264-280` | `static/` copied verbatim (site static over repo `static/`) |
| Base URL | implemented | `src/render/mod.rs:143-157` | all internal URLs pass through `base` |
| CDN assets (`assetsBase`) | missing | — | see site-config row |
| `withBase` | n/a | — | runtime helper |

## guide/using-vue.md

Everything here is **n/a** — markdown-as-Vue-SFC needs the Vue runtime
gen-docs deliberately has no replacement for. One exception:

| feature | status | ours | note |
|---|---|---|---|
| Escaping (`::: v-pre`) | implemented | `src/markdown/preprocess.rs:325-328` | marker stripped; content passes through (nothing to interpolate anyway) |
| `<span v-pre>`, `-vue` fence suffix, `<script setup>`, components, CSS pre-processors, teleports, `<ClientOnly>`, IntelliSense | n/a | — | no Vue runtime; `{{ }}` stays literal (README caveat) |

## guide/extending-default-theme.md

| feature | status | ours | note |
|---|---|---|---|
| Customizing CSS variables | implemented | `theme.toml` → `theme.css` (`src/render/mod.rs:111-136`) | diverged mechanism: TOML key/value instead of a CSS file (a raw CSS file can still be linked via `[[head]]`) |
| Different fonts | partial | `--vp-font-family-*` overridable via `theme.toml` | no `theme-without-fonts` entry; Inter always ships |
| Navbar theming (CSS vars) | implemented | `styles/vitepress.css` | same `--vp-nav-*` tokens |
| Navbar overflow `...` collapse | missing | — | long nav wraps/scrolls instead |
| Registering global components | n/a | — | no Vue |
| Layout slots | n/a | — | no Vue |
| Overriding internal components | n/a | — | one compiled-in design (README) |
| View Transitions on appearance toggle | missing | — | doable client-side (vanilla), not built |

## guide/data-loading.md

n/a — `*.data.js` loaders, `createContentLoader`, `watch` run on Node
at build time; no plugin ABI in the Rust binary.

## guide/sitemap-generation.md

| feature | status | ours | note |
|---|---|---|---|
| Sitemap generation | implemented | `src/render/mod.rs:289-293,392-407` | `sitemap.hostname` → `sitemap.xml`, `<lastmod>` from `lastUpdated` |
| Extra `sitemap` module options / `transformItems` | n/a | — | Node sitemap library options |

## guide/mpa-mode.md

n/a — gen-docs output is always zero-framework static HTML + one Alpine
bundle; there is no SPA/SSR mode to trade away. `<script client>` is a
VitePress-only tag for that pipeline.

## guide/ssr-compat.md

n/a — `<ClientOnly>`, `defineClientComponent`, `import.meta.env.SSR`
guarding solve SSR/hydration problems gen-docs doesn't have.

## guide/cms.md

n/a — `loadEnv` + `paths()` data fetching is Node build-time work.

## guide/custom-theme.md

n/a — no swappable theme system by design (README); the compiled-in
theme is customized only via CSS variables (`theme.toml`) and
`[[head]]`.

## reference/cli.md

| feature | status | ours | note |
|---|---|---|---|
| `build [root]` | implemented | `src/main.rs:101` | `gen-docs build <site>`; no `--base`/`--outDir` flags |
| `dev [root]` | implemented | `src/serve.rs:24` | `gen-docs serve`: rebuild-on-change + livereload SSE |
| `preview [root]` | implemented | `src/serve.rs` | same server serves the built `public/` |
| `init` wizard | missing | — | no scaffold command |

## reference/runtime-api.md

n/a — `useData`/`useRoute`/`useRouter`/`useIcon`/`withBase`/`<Content/>`/`$frontmatter`/`$params`
are Vue composables over the SPA runtime.

## reference/default-theme-*.md (dedicated pages)

| feature | status | ours | note |
|---|---|---|---|
| nav: custom `component`/`props` | n/a | — | Vue components |
| sidebar: nested `base` overrides | implemented | `src/sidebar.rs:170-233` | nearest base wins |
| edit-link: frontmatter `editLink: false` | missing | — | no per-page frontmatter toggle |
| last-updated: frontmatter `lastUpdated: false` / Date | missing | — | no per-page control |
| search: local provider | partial | `src/render/mod.rs:243-254`, `src/render/search_modal.rs` | client-side scored index + modal (Ctrl/Cmd+K, `/`); no `translations`, no `miniSearch` options, no per-page `search: false` frontmatter |
| search: Algolia / DocSearch / Ask AI | missing | — | external service; see extras for our `askAiUrl` |
| footer: inline HTML in message/copyright | partial | `src/render/layout.rs:87-98` | plain text only |
| layout: `page` | missing | — | unstyled layout not distinguished from `doc` |
| home-page: hero `image` light/dark | missing | `src/content.rs:77` | `{src, alt}` only |
| home-page: hero action `target`/`rel` | missing | finding #5 | parsed in `src/content.rs:92` but not emitted in `src/render/home.rs:129-134` |
| home-page: `markdownStyles` | missing | — | |
| team page (`VPTeamMembers` etc.) | missing | — | README caveat |
| `<Badge>` | implemented | `src/markdown/preprocess.rs:574-627`, `src/render/vpdoc.rs:192` | all 7 types, self-closing + paired |
| carbon ads | missing | — | external service |

## Audit findings (2026-09-10)

Parsed-but-dead config and other small bugs this audit surfaced — none
affect the pinned demo pages, all are fixable in an afternoon:

1. **`returnToTopLabel` never read** — `src/config.rs:153` parses it,
   `src/render/local_nav.rs:17` hardcodes `"Return to top"`.
2. **`[markdown.container] detailsLabel` never read** —
   `src/config.rs:552` parses it; `parse_details_rest` hardcodes
   `"Details"` (`src/markdown/preprocess.rs:478`), and `label_for`
   (`src/config.rs:561`) has no `details` arm.
3. **`[locales.*] title` never used** — parsed (`src/config.rs:623`) but
   `document_title` (`src/render/mod.rs:364-377`) only reads the site
   title, so a locale's `<title>`/navbar title cannot differ.
4. **`image.lazyLoading` vs upstream `image.lazyLoad`** — accept both
   or rename (`src/config.rs:522`).
5. **hero/feature `target` parsed but not emitted** —
   `src/content.rs:92,109` vs `src/render/home.rs:129-134`.
6. **`appearance: "force-dark"` rejected** — we accept `"force"`
   (`src/config.rs:435`); upstream's documented spelling fails to load.
7. **`[!DANGER]` renders as a plain blockquote** (probe above) — comrak
   knows the five GitHub alert kinds only.

## gen-docs-only surface (no upstream counterpart)

Tracking our own extras here keeps the tables above honest about
direction: upstream → us.

- `[syntax]` Helix TOML themes (218 bundled + custom file paths,
  light/dark split) — upstream colors code via Shiki `markdown` options.
- `theme.toml` palettes + builtins (`green`, `purple`, `orange`,
  `mono`) — upstream theming is CSS files.
- `[markdown] codeCopyButton` toggle — upstream has no such option.
- `askAiUrl` navbar link — upstream's Ask AI lives inside Algolia search.
- `[notFound]` title/quote/linkText — upstream 404 is slot-based.
- hero action `theme: "sponsor"` — upstream has brand/alt only.
- `gen-docs serve` with SSE livereload; the parity tooling itself.
