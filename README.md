# vitezola

**The VitePress default theme, ported to Zola.** As faithful as possible a re-creation of the [VitePress](https://vitepress.dev/) default theme's layout, colors and interactions — navbar, auto sidebar, right-hand outline, dark mode, code groups, local search — all generated as pure static HTML + CSS + a little vanilla JS. No Node toolchain, no hydration.

![Zola](https://img.shields.io/badge/zola-%E2%89%A50.23-blue) (requires Zola 0.23+: uses Tera 2 components and the new highlighting settings)

## Features

- **Layout**: navbar (dropdown flyouts, social links, Ask AI button), auto-generated nested sidebar (one per top-level content section, mirroring VitePress's path-keyed sidebar), right-hand "On this page" outline with scrollspy marker (depth h2–h3 by default; front matter `outline = "deep"` expands everything), prev/next pager, edit link, last updated, footer
- **Dark mode**: CSS variables taken directly from VitePress `vars.css`; blocking script in `<head>` prevents FOUC; preference stored in localStorage
- **Markdown extensions** (Tera 2 components): `tip` / `warning` / `danger` / `note` / `info` / `important` / `caution` containers (custom titles and `{no-title}` supported), `details`, `codegroup`
- **Code blocks**: Zola 0.22+ Giallo highlighting with dual light/dark themes (defaults github-light/github-dark, same as VitePress), `name=` labels, `hl_lines=` line highlighting, copy buttons, code group tabs
- **Search**: Zola's built-in index + a hand-written vanilla JS search modal (Ctrl+K or `/`)
- **Mobile**: hamburger full-screen menu, sidebar drawer, outline dropdown in the local nav
- **Badge**: `<span class="VPBadge tip">…</span>` (equivalent of VitePress's `<Badge>` component, styles ported)
- **The demo site is a vitepress.dev content mirror**: `content/` is an automated conversion of the official vitepress docs (33 pages) via `scripts/port-vitepress-docs.py`, used for pixel-level comparison
- **Pixel calibration**: navbar details matched against the deployed vitepress.dev — zero gap between logo and title, translations button (16px option-icon structure + 17px left reservation), GitHub icon as a simple-icons mask (`vpi-simple-icons-github`), hero image carries the `VPImage` class (enables the site's custom drop-shadow)

## Install

```sh
git clone https://github.com/your-name/vitezola themes/vitezola
```

Then add to your `config.toml` (**required settings as of Zola 0.23**):

```toml
theme = "vitezola"
compile_sass = true
build_search_index = true

[search]
index_format = "fuse_json"

[markdown.highlighting]
style = "class"
light_theme = "github-light"
dark_theme = "github-dark"

[extra]
vitepress_site_title = "My Site"
vitepress_edit_link = "https://github.com/me/my-site/edit/main/content/"
vitepress_last_updated = true

# Sidebar: by default one is derived automatically per top-level content
# section (guide/, reference/, … — like VitePress's path-keyed sidebar).
# Only set this to force a single fixed sidebar:
# vitepress_sidebar_path = "docs/_index.md"

[[extra.nav]]
text = "Guide"
link = "/guide/introduction/what-is-vitepress/"
active_match = "/guide/"        # optional: URL-prefix active matching (like VitePress activeMatch)

[[extra.nav]]
text = "More"
items = [{ text = "Reference", link = "/reference/" }]

[[extra.social]]
kind = "github"                              # github | twitter | other
link = "https://github.com/me/my-site"
```

> Note: in TOML, array-of-tables like `[[extra.nav]]` / `[[extra.social]]` must come **after** all scalar `[extra]` keys; bare scalar keys must not follow an `[extra.xxx]` sub-table either.

**Sidebar group behavior** matches VitePress's tri-state `collapsed`, controlled from the group section's front matter (e.g. `content/guide/introduction/_index.md`):

```toml
[extra]
vitepress_collapsed = false   # collapsible, starts expanded (caret button shown)
# vitepress_collapsed = true  # collapsible, starts collapsed
# unset                       # not collapsible (no caret, always expanded)
```

To render the sidebar root as a titled group (like VitePress's `{ text, items }` root entry), add:

```toml
[extra]
vitepress_sidebar_title = "Reference"
```

A section's front matter also accepts `[[extra.vitepress_extra_items]]` (`text` / `link`) for plain cross-section links (like the "Config & API Reference" entry at the bottom of the Guide sidebar).

## Home page

Set `template = "index.html"` on the root section (`content/_index.md`) and add:

```toml
[extra.vitepress_home]
name = "My Project"          # brand-colored headline
text = "The tagline text"
tagline = "Longer description"
image = { src = "hero.png", alt = "" }   # optional, lives in static/

[[extra.vitepress_home.actions]]
text = "Get Started"
theme = "brand"              # brand | alt
link = "/docs/guide/"

[[extra.vitepress_home.features]]
icon = "⚡"
title = "Feature"
details = "Description"
link = "/docs/feature/"      # optional
link_text = "Learn more"     # optional
```

## Using in Markdown

Zola 0.23 replaced shortcodes with Tera 2 components. Containers (**block calls must list every parameter**):

```md
{% <tip kind="warning" title="" no_title={false}> %}
**Note** this text is rendered as markdown.
{% </tip> %}
```

Available `kind`: `tip`, `warning`, `danger`, `note`, `info`, `important`, `caution`. Custom title: `title="Server Support Required"`; hide the title row: `no_title={true}` (same as VitePress's `::: tip {no-title}`). There is also `{% <details summary=""> %}…{% </details> %}` (an empty `summary` renders DETAILS).

Code groups (tab labels come from each block's `name=` annotation):

````md
{% <codegroup> %}
```js,name=a.js
const a = 1;
```
```ts,name=b.ts
const b: number = 2;
```
{% </codegroup> %}
````

## Syntax highlighting themes

The light/dark highlight CSS is generated by Zola at build time into `public/giallo-light.css` / `giallo-dark.css`, but both use flat classes (`z-l-*` / `z-d-*`) that can't be switched with `html.dark` directly. The theme's bundled `static/syntax.css` rewrites the dark rules under `html.dark` scope. **Regenerate it after changing `[markdown.highlighting]` themes:**

```sh
zola build && python3 scripts/gen-syntax-css.py
```

## Team page and sponsors

**Team page**: create a section (e.g. `content/team/_index.md`) with `template = "team.html"`:

```toml
[extra.vitepress_team]
title = "Our Team"
lead = "The folks behind this project."

[[extra.vitepress_team.members]]
name = "Ella"
avatar = "images/avatar.svg"      # lives in static/
title = "Creator"
org = "vitezola"
org_link = "https://example.com" # optional, turns org into a link
desc = "Description with **markdown**."
links = [{ kind = "github", link = "https://github.com/you" }]
sponsor = "https://github.com/sponsors/you"   # optional, Sponsor button on the card

[[extra.vitepress_team.sections]]  # optional: grouped sections
title = "Contributors"
lead = "..."
size = "small"                     # small | medium

[[extra.vitepress_team.sections.members]]
name = "Alex"
avatar = "images/avatar-2.svg"
```

**Sponsors**:

```toml
# Home page footer section → inside [extra.vitepress_home] in content/_index.md
[extra.vitepress_home.sponsors]
message = "Special thanks to:"
action_link = "https://github.com/sponsors/you"
action_text = "Become a sponsor"

[[extra.vitepress_home.sponsors.tiers]]
tier = "Diamond"
size = "medium"                    # xmini | mini | small | medium | big (optional; auto by count)
[[extra.vitepress_home.sponsors.tiers.items]]
name = "Acme"
img = "images/sponsor-1.svg"
url = "https://example.com"
```

```toml
# Doc page right-hand aside → config.toml
[[extra.vitepress_sponsors.tiers]]
tier = "Sponsors"
size = "xmini"
[[extra.vitepress_sponsors.tiers.items]]
name = "Acme"
img = "images/sponsor-1.svg"
url = "https://example.com"
```

Grid column behavior matches VitePress: size auto-selected by item count (9+ → xmini, 7–8 → mini, 5–6 → small, 3–4 → medium, 1–2 → big); desktop columns = min(size columns, item count) with empty slots for alignment; narrow screens drop to 2 / 1 columns. Sponsor logos invert automatically in dark mode (same as VitePress).

## Language switcher

When the site has multiple languages (`[languages.fr]` etc.), a language flyout appears in the navbar and a language accordion in the mobile menu. Behavior matches VitePress: link to the translated page when it exists, otherwise to that language's root. Display names are optional:

```toml
[extra.vitepress_language_labels]
en = "English"
fr = "Français"
```

## VitePress ↔ vitezola syntax mapping

| VitePress | vitezola (Zola 0.23) |
| --- | --- |
| `::: tip` … `:::` | `{% <tip kind="tip" title="" no_title={false}> %} … {% </tip> %}` |
| `::: warning SERVER REQUIRED` | `kind="warning" title="SERVER REQUIRED"` |
| `::: tip {no-title}` | `no_title={true}` |
| `::: details` | `{% <details summary="" open={false}> %} … {% </details> %}` |
| `<Badge type="warning" text="experimental" />` | `<span class="VPBadge warning">experimental</span>` |
| <code>\`\`\`js [a.js]</code> (code group) | <code>\`\`\`js,name=a.js</code> wrapped in `{% <codegroup> %} … {% </codegroup> %}` |
| <code>\`\`\`js{1,3-4}</code> (line highlighting) | <code>\`\`\`js,hl_lines=1 3-4</code> |
| `outline: deep` (front matter) | same name supported: page/section front matter `outline = "deep"` (default depth h2–h3) |
| `themeConfig.sidebar` | derived from content section structure (one sidebar per top-level section) |
| `themeConfig.appearance` | always on (`prefers-color-scheme` + localStorage) |
| `langLabel` / locale names | `extra.vitepress_language_labels` |

## Local development

The repo root is itself a Zola demo site (root-level templates take effect directly):

```sh
zola serve
```

## Caveats

- Tested on **Zola 0.23**; Zola 0.22 and earlier (old Tera / shortcodes) are incompatible.
- Requires modern browsers with `:has()` and CSS nesting (Chrome/Edge/Firefox/Safari 2023+).
- Search uses lightweight whitespace-tokenized scoring; **CJK recall is worse than English**. For better CJK search, swap `[search]` to elasticlunr and bring your own frontend.
- The site stays readable without JS: sidebar/outline are static, code groups show the first tab, appearance follows the system `prefers-color-scheme`; the dark toggle and search are disabled.

## Differences from VitePress (design trade-offs)

- No Vue runtime: no Vue components inside markdown; all interactivity is vanilla JS
- Data loaders must be generated outside the build pipeline
- The sidebar is derived from Zola's content section structure rather than a `themeConfig.sidebar` array; collapse semantics are the tri-state described above
- Sidebar/pager titles come from the page title (= the content H1); you can't give the sidebar a separate short name like vitepress.dev does ("Deploy" vs the H1 "Deploy Your VitePress Site"). Prev/next ordering also only sorts within the same section
- Search is a hand-written lightweight scorer over Zola's fuse_json index, not minisearch/Algolia
- Shiki inline markers `[!code highlight]` / `[!code focus]` / `[!code ++]` have no effect (giallo has no transformers) and appear as literal text inside code blocks; the `:line-numbers` modifier is stripped (giallo doesn't support per-block line numbers)
- Things vitepress.dev adds on top of the default theme are only partially reproduced: no package icons in code group tabs (that's vitepress-plugin-group-icons), no Carbon Ads in the aside (requires an account), and the navbar Ask AI button is a plain link (`vitepress_ask_ai_url`) with no DocSearch side panel

## License

CSS variables, fonts (Inter) and icons are ported from VitePress (MIT). Everything else is released under MIT.
