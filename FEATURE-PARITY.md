# Feature parity: rustpress vs upstream VitePress

This is the **feature-surface audit**: every documented upstream
capability, row by row, against what rustpress actually does (claims
verified against source, not README prose). It complements
[PARITY.md](PARITY.md), whose two mechanical gates cover only the 14
pinned demo pages — this file answers "VitePress shipped a feature we
never looked at" for the *whole* documented surface.

- **Pinned upstream:** `vuejs/vitepress` @ `3e681e2` (v2.0.0-alpha.20;
  audited 2026-09-10, statuses updated after the fill-in pass
  d6b05d2…fe1cdc9)
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
| n/a | needs a runtime rustpress doesn't have (Vue/Vite/Node/external service) |

Status summary: implemented 76, partial 17, diverged 12, missing 25,
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
tooling concerns of the `.vitepress/config.ts` ecosystem — rustpress has
a single `rustpress.toml` with `deny_unknown_fields`, so typos fail at
load (`src/config.rs:28`).

| heading | status | ours | note |
|---|---|---|---|
| Config Resolution | n/a | `rustpress.toml` | no JS config cascade; one TOML file (`src/config.rs:205`) |
| Config Intellisense | n/a | — | no TS types to IntelliSense |
| Typed Theme Config | n/a | — | single compiled-in theme; no `defineConfig` surface |
| Vite, Vue & Markdown Config | n/a | — | no Vite/Vue anywhere in the pipeline |
| Page-Level Overrides | partial | `src/content.rs` | 15 front-matter keys honored; only directory-level overrides remain |
| Directory-Level Overrides | missing | — | no per-directory front matter files |

### Site Metadata

| heading | status | ours | note |
|---|---|---|---|
| title | implemented | `src/config.rs:31` | navbar text + `<title>` default (`src/render/mod.rs:364`) |
| titleTemplate | implemented | `src/render/mod.rs` document_title | string templates + `false` (suffix dropped) |
| description | implemented | `src/config.rs:36` | meta tag; locale/frontmatter fallback (`src/render/mod.rs:183-189`) |
| head | partial | `src/render/layout.rs` | site-level + per-page front-matter tags; no per-locale head, no dedup/merge |
| lang | implemented | `src/config.rs:41` | `<html lang>`; per-locale override (`src/render/mod.rs:356-362`) |
| base | implemented | `src/config.rs:46` | validated + prepended (`src/render/mod.rs:143-157`) |

### Routing

| heading | status | ours | note |
|---|---|---|---|
| cleanUrls | diverged | `src/content.rs:286-294` | rustpress *always* emits directory URLs (`/guide/x/`); there is no `.html`-suffix mode to disable |
| rewrites | partial | `src/render/mod.rs:66-88` | static map + `:rest*` suffix capture only; no multi-param `:pkg/:slug*` patterns |

### Build

| heading | status | ours | note |
|---|---|---|---|
| srcDir | implemented | `src/config.rs:140` | default `content` (upstream default `.`) |
| srcExclude | implemented | `src/content.rs` glob_match | `*` within a segment, `**` across; matched against source-rel paths |
| outDir | diverged | `src/render/mod.rs:260` | fixed `<site>/public`, not configurable |
| assetsDir | n/a | — | no Vite asset pipeline; assets copied verbatim, no hashing |
| assetsBase | missing | — | CDN prefix for generated assets; nothing hashed to serve |
| icons | n/a | — | iconify collection pipeline needs Node; our icons are compiled-in SVG |
| cacheDir | n/a | — | no Vite cache |
| ignoreDeadLinks | partial | `src/config.rs` | true / "localhostLinks" (accepted; no-op — http targets never collected) / prefix list; no regex or function forms |
| mpa | n/a | — | rustpress is always a static build with Alpine islands (MPA-shaped by construction) |

### Theming

| heading | status | ours | note |
|---|---|---|---|
| appearance | implemented | `src/config.rs:396-443` | `true`/`false`/`"dark"`/`"force"`/`"force-auto"` (upstream spells `force-dark`; we accept `force`), anti-FOUC script `src/render/layout.rs:149-170` |
| lastUpdated | implemented | `src/render/mod.rs` | `false` hides; a date string is displayed instead of the git timestamp |

### Customization

| heading | status | ours | note |
|---|---|---|---|
| markdown | partial | `src/config.rs:491-574` | ours: `lineNumbers`, `codeCopyButton`, `math`, `image.lazyLoading`, `container.*`; missing: `anchor`, `toc`, `theme`, `headers`, `snippet.*`, `include.*`, `config()` hook. Key divergence: upstream's image option is `lazyLoad`, ours `lazyLoading` |
| vite | n/a | — | no Vite |
| vue | n/a | — | no Vue |

### Build Hooks

buildEnd / postRender / transformHead / transformHtml / transformPageData —
all **n/a**: JavaScript build hooks need the Node build process rustpress
replaces. The Rust binary has no plugin ABI; `[[head]]` covers the
common `transformHead` use.

## reference/default-theme-config.md

| heading | status | ours | note |
|---|---|---|---|
| i18nRouting | diverged | `src/render/mod.rs:324-353` | no toggle; language switcher always targets the same page in the other locale (upstream's default behavior), else the locale root |
| logo | implemented | `logo_html` `src/render/navbar.rs` | path, `{src, alt}`, `{light, dark}` pair |
| siteTitle | implemented | `src/render/mod.rs` navbar_site_title | string override or `false` to hide; `<title>` unaffected (upstream scopes it to the navbar) |
| nav | partial | `src/render/navbar.rs` | text/link/items/activeMatch/target/rel; `noIcon` n/a (items never show icons); overflow `...` collapse still missing |
| sidebar | implemented | `src/config.rs`, `src/sidebar.rs` | array + multi forms, collapsed, base, per-item target/rel/docFooterText |
| aside | implemented | `src/render/doc.rs` | `false`/`true`/`"left"` (site config + per-page front matter) |
| outline | implemented | `OutlineConfig` `src/config.rs` | `false` / bare level / `[a, b]` / `{level, label}` |
| socialLinks | implemented | `src/render/navbar.rs` | named icons + `{svg}` + ariaLabel/target |
| footer | implemented | `src/render/layout.rs:87-98` | `message`/`copyright` as inline HTML, hidden when a sidebar is shown |
| editLink | implemented | `src/render/doc.rs:90-97` | `pattern` (`:path`) + `text`; function pattern form n/a |
| lastUpdated | diverged | `src/config.rs:151` | ours is the flat `lastUpdatedText` key (upstream nests `lastUpdated.text`); no `formatOptions` — dates are fixed UTC format |
| algolia | missing | — | external service (README caveat) |
| carbonAds | missing | — | external service |
| docFooter | implemented | `src/render/doc.rs` | prev/next labels; `false` disables a side; per-item `docFooterText` overrides pager titles |
| darkModeSwitchLabel | implemented | `src/render/navbar.rs:256,284` | |
| lightModeSwitchTitle | partial | `src/render/navbar.rs` | static dark-mode title only — no client-side swap between the two titles |
| darkModeSwitchTitle | implemented | `src/render/navbar.rs` | static `title` on the switch |
| sidebarMenuLabel | implemented | `src/render/local_nav.rs` | the local-nav "Menu" button (our nav screen has no heading) |
| returnToTopLabel | implemented | `src/render/local_nav.rs` | was parsed-but-dead; wired in d6b05d2 |
| langMenuLabel | implemented | `src/render/navbar.rs` |  |
| navMenuLabel | implemented | `src/render/navbar.rs` | desktop nav + nav screen |
| mobileMenuLabel | implemented | `src/render/navbar.rs` |  |
| extraMenuLabel | missing | — | (and the `...` overflow menu itself is missing, see `nav`) |
| skipToContentLabel | implemented | `src/render/layout.rs:41,71` | |
| externalLinkIcon | diverged | `src/render/vpdoc.rs:108` | arrow is always on; upstream defaults off, opt-in |
| gradedContainers | implemented | `vp-graded-containers` body class | the graded palette ships in `styles/vitepress.css`; switch added in fe1cdc9 |
| useLayout | n/a | — | Vue composable |

## reference/frontmatter-config.md

| heading | status | ours | note |
|---|---|---|---|
| title | implemented | `src/content.rs:22,242-248` | falls back to first H1, then file stem |
| titleTemplate | implemented | `src/content.rs` | per-page template wins over the site's |
| description | implemented | `src/content.rs:25` | |
| head | implemented | `layout::serialize_head_tags` | appended after the site's tags |
| Default Theme Only | — | — | subsections below |
| layout | implemented | `src/render/mod.rs` | `home` / `doc` / `page` (page = no outline/edit/pager chrome); custom component names n/a |
| hero | implemented | `src/content.rs:73-106`, `src/render/home.rs` | `name`/`text`/`tagline`/`image{src,alt}`/`actions`; see findings #5 for `target` |
| features | implemented | `src/content.rs:108-123`, `src/render/home.rs:81-113` | icon as raw HTML/emoji; no `{light,dark}`/`width`/`height` icon forms |
| navbar | implemented | `src/render/layout.rs` |  |
| sidebar | implemented | `src/render/mod.rs` | per-page toggle over the sidebar resolution |
| aside | implemented | `src/render/mod.rs` | front matter wins over the site-level `aside` |
| outline | implemented | `src/content.rs` | `deep`/level/range/`false`; `false` disables the outline |
| lastUpdated | implemented | `src/render/mod.rs` | `false` hides; a date string is displayed instead of the git timestamp |
| editLink | implemented | `src/render/mod.rs` |  |
| footer | implemented | `src/render/layout.rs` |  |
| pageClass | implemented | `src/render/layout.rs` | added to the VPContent container |
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
| Footnotes | implemented | `expand_inline_footnotes` `src/markdown/preprocess.rs` | `[^1]` references + inline `^[…]` (rewritten to reference form); comrak renders |
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
| Additional Attributes | partial | `rewrite_link_attrs` | `{target=… rel=…}` on links (rewritten to raw anchors — no markdown inside the text); `{open}`/`{no-title}` on containers; not on arbitrary elements |
| raw | implemented | `expand_containers` | wraps in `<div class="vp-raw">` |
| GitHub-flavored Alerts | implemented | `expand_alerts` (preprocess) | all kinds render as containers (styling, labels, custom titles); `[!DANGER]` + registered custom kinds work |

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
| Markdown File Inclusion | partial | `load_include` `src/markdown/preprocess.rs` | recursive + `#region`/heading-anchor sections + `{a,b}` line ranges + `@/`-root resolution; relative links inside includes are NOT rebased to the included file |
| Including Code Files | implemented | fence-mode include branch | the directive inside a fence inserts the selected lines verbatim |
| Math Equations | diverged | `src/markdown/mod.rs:77-79,164-178` | `$…$`/`$$…$$` via client-side MathJax CDN injected per page; upstream typesets at build |
| Image Lazy Loading | implemented | `src/config.rs` | both `lazyLoading` and upstream's `lazyLoad` accepted |
| Advanced Configuration | n/a | — | `markdown.anchor`/`toc`/`config()` need the markdown-it JS engine |

## guide/routing.md

| feature | status | ours | note |
|---|---|---|---|
| File-based routing | implemented | `src/content.rs:286-294` | URL shape diverges (directory URLs) |
| srcDir | implemented | `src/config.rs:140` | |
| Linking between pages | implemented | `src/markdown/mod.rs:183-258` | `.md`/`.html`/relative resolve; `{target="_self"}` attribute blocks work (as raw anchors) |
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
rustpress deliberately has no replacement for. One exception:

| feature | status | ours | note |
|---|---|---|---|
| Escaping (`::: v-pre`) | implemented | `src/markdown/preprocess.rs:325-328` | marker stripped; content passes through (nothing to interpolate anyway) |
| `<span v-pre>`, `-vue` fence suffix, `<script setup>`, components, CSS pre-processors, teleports, `<ClientOnly>`, IntelliSense | n/a | — | no Vue runtime; `{{ }}` stays literal (README caveat) |

## guide/extending-default-theme.md

| feature | status | ours | note |
|---|---|---|---|
| Customizing CSS variables | implemented | `theme = "file.css"` → `theme.css` (`src/render/mod.rs`) | same mechanism: a CSS file copied verbatim and linked after `main.css` |
| Different fonts | partial | `--vp-font-family-*` overridable via the `theme` CSS file | no `theme-without-fonts` entry; Inter always ships |
| Navbar theming (CSS vars) | implemented | `styles/vitepress.css` | same `--vp-nav-*` tokens |
| Navbar overflow `...` collapse | missing | — | long nav wraps/scrolls instead |
| Registering global components | n/a | — | no Vue |
| Layout slots | n/a | — | no Vue |
| Overriding internal components | n/a | — | one compiled-in design (README) |
| View Transitions on appearance toggle | implemented | `js/alpine-entry.js` | `document.startViewTransition` around the dark flip, instant fallback |

## guide/data-loading.md

n/a — `*.data.js` loaders, `createContentLoader`, `watch` run on Node
at build time; no plugin ABI in the Rust binary.

## guide/sitemap-generation.md

| feature | status | ours | note |
|---|---|---|---|
| Sitemap generation | implemented | `src/render/mod.rs:289-293,392-407` | `sitemap.hostname` → `sitemap.xml`, `<lastmod>` from `lastUpdated` |
| Extra `sitemap` module options / `transformItems` | n/a | — | Node sitemap library options |

## guide/mpa-mode.md

n/a — rustpress output is always zero-framework static HTML + one Alpine
bundle; there is no SPA/SSR mode to trade away. `<script client>` is a
VitePress-only tag for that pipeline.

## guide/ssr-compat.md

n/a — `<ClientOnly>`, `defineClientComponent`, `import.meta.env.SSR`
guarding solve SSR/hydration problems rustpress doesn't have.

## guide/cms.md

n/a — `loadEnv` + `paths()` data fetching is Node build-time work.

## guide/custom-theme.md

n/a — no swappable theme system by design (README); the compiled-in
theme is customized only via CSS variables (`theme = "file.css"`) and
`[[head]]`.

## reference/cli.md

| feature | status | ours | note |
|---|---|---|---|
| `build [root]` | implemented | `src/main.rs:101` | `rustpress build <site>`; no `--base`/`--outDir` flags |
| `dev [root]` | implemented | `src/serve.rs:24` | `rustpress serve`: rebuild-on-change + livereload SSE |
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
| search: local provider | partial | `src/render/mod.rs`, `search_modal.rs` | translations wired (button/modal strings, `{q}` no-results); per-page `search: false`; still no `miniSearch` tuning |
| search: Algolia / DocSearch / Ask AI | missing | — | external service; see extras for our `askAiUrl` |
| footer: inline HTML in message/copyright | partial | `src/render/layout.rs:87-98` | plain text only |
| layout: `page` | missing | — | unstyled layout not distinguished from `doc` |
| home-page: hero `image` light/dark | implemented | `hero_image_html` |  |
| home-page: hero action `target`/`rel` | implemented | `hero_button` | was finding #5; fixed in d6b05d2 |
| home-page: `markdownStyles` | missing | — | |
| team page (`VPTeamMembers` etc.) | missing | — | README caveat |
| `<Badge>` | implemented | `src/markdown/preprocess.rs:574-627`, `src/render/vpdoc.rs:192` | all 7 types, self-closing + paired |
| carbon ads | missing | — | external service |

## Audit findings (2026-09-10)

Parsed-but-dead config and other small bugs this audit surfaced — all
seven were fixed in the fill-in pass (`d6b05d2`), plus an eighth found
on the way (feature.link never rendered as a link; alerts had no CSS at
all — both fixed):

1. **`returnToTopLabel` never read** *(fixed, d6b05d2)* — `src/config.rs:153` parsed it,
   `src/render/local_nav.rs:17` hardcodes `"Return to top"`.
2. **`[markdown.container] detailsLabel` never read** *(fixed, d6b05d2)* —
   `src/config.rs:552` parses it; `parse_details_rest` hardcodes
   `"Details"` (`src/markdown/preprocess.rs:478`), and `label_for`
   (`src/config.rs:561`) has no `details` arm.
3. **`[locales.*] title` never used** *(fixed, d6b05d2)* — parsed but
   `document_title` (`src/render/mod.rs:364-377`) only reads the site
   title, so a locale's `<title>`/navbar title cannot differ.
4. **`image.lazyLoading` vs upstream `image.lazyLoad`** *(fixed, d6b05d2)* — both spellings accepted.
5. **hero/feature `target` parsed but not emitted** *(fixed, d6b05d2)* —
   `src/content.rs:92,109` vs `src/render/home.rs:129-134`.
6. **`appearance: "force-dark"` rejected** *(fixed, d6b05d2)* — both spellings accepted;
   (`src/config.rs:435`); upstream's documented spelling fails to load.
7. **`[!DANGER]` rendered as a plain blockquote** *(fixed, d6b05d2)* — alerts now rewrite into containers.

## rustpress-only surface (no upstream counterpart)

Tracking our own extras here keeps the tables above honest about
direction: upstream → us.

- `[syntax]` Helix TOML themes (218 bundled + custom file paths,
  light/dark split) — upstream colors code via Shiki `markdown` options.
- `[markdown] codeCopyButton` toggle — upstream has no such option.
- `askAiUrl` navbar link — upstream's Ask AI lives inside Algolia search.
- `[notFound]` title/quote/linkText — upstream 404 is slot-based.
- hero action `theme: "sponsor"` — upstream has brand/alt only.
- `rustpress serve` with SSE livereload; the parity tooling itself.
