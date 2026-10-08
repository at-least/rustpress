---
outline: deep
description: Set up the built-in local search for your rustpress site.
---

# Search

## Local Search

rustpress supports full-text search over an index built at build time and queried in the browser, with no external service. To enable this feature, add a `[search]` section with `provider = "local"` to your `rustpress.toml`:

```toml [rustpress.toml]
[search]
provider = "local"
```

`local` is the only provider. Algolia DocSearch and the community search plugins for VitePress are not available.

### How it works

- The build writes `search-docs.json` to the output root: one entry per page with its URL, title and the plain text of its content. On a [multi-language site](../guide/i18n) each locale gets its own index (`zh/search-docs.json`, …), and a page searches only its own locale's pages.
- The navbar shows a search button; the modal opens on click, with <kbd>Ctrl</kbd>+<kbd>K</kbd> / <kbd>⌘</kbd>+<kbd>K</kbd>, or with <kbd>/</kbd> while no input field is focused. <kbd>Esc</kbd> closes it.
- Results are scored client-side per whitespace-separated query token: an exact title match scores highest, then title hits, then occurrences in the body (capped per token), and the top 20 pages are listed.
- Search runs on page titles and body text; there is no per-heading (section) result list.

CJK text is tokenized on whitespace only, so recall for languages written without spaces is limited to whole-string matches.

### Excluding pages

Add `search: false` to a page's frontmatter to leave it out of the index:

```yaml
---
search: false
---
```

### i18n {#local-search-i18n}

The strings of the search button and modal are configurable under `[search.translations]`:

```toml [rustpress.toml]
[search]
provider = "local"

[search.translations]
buttonText = "搜索"
buttonAriaLabel = "搜索"
noResultsText = "没有找到 “{q}” 的结果"
resetButtonTitle = "清除"
navigateText = "切换"
selectText = "选择"
closeText = "关闭"
backButtonTitle = "关闭搜索"
navigateUpKeyAriaLabel = "上箭头"
navigateDownKeyAriaLabel = "下箭头"
selectKeyAriaLabel = "输入"
closeKeyAriaLabel = "esc"
```

| key | default | where it shows |
| --- | --- | --- |
| `buttonText` | `Search` | navbar button label; in the modal, the search icon's tooltip and the input placeholder |
| `buttonAriaLabel` | `Search` | navbar button `aria-label` |
| `placeholder` | the `buttonText` | modal input placeholder — a rustpress extra: upstream's input always shows `buttonText` |
| `noResultsText` | `No results for "{q}"` | empty state; `{q}` marks where the query goes — without it the quoted query follows the text, as in VitePress (`没有结果` shows `没有结果 "query"`) |
| `resetButtonTitle` | `Reset search` | clear-button tooltip |
| `navigateText` | `to navigate` | footer hint after the arrow keys |
| `selectText` | `to select` | footer hint after Enter |
| `closeText` | `to close` | footer hint after Esc |
| `backButtonTitle` | `Close search` | tooltip and label of the back button that closes the modal on phones (below 768px, where the modal fills the screen) |
| `navigateUpKeyAriaLabel` | `up arrow` | accessible name of the footer's ↑ key icon |
| `navigateDownKeyAriaLabel` | `down arrow` | accessible name of the footer's ↓ key icon |
| `selectKeyAriaLabel` | `enter` | accessible name of the footer's ↵ key icon |
| `closeKeyAriaLabel` | `escape` | accessible name of the footer's `esc` key |

On a [multi-language site](../guide/i18n), each locale can set its own strings under `[search.locales.<key>.translations]` (VitePress's `search.options.locales`), with the same keys. A key a locale leaves out falls back to `[search.translations]`, then to the default. Use the key `root` for the root locale's pages (and the 404 page); it doesn't reach the other locales.

```toml [rustpress.toml]
[search]
provider = "local"

[search.locales.zh.translations]
buttonText = "搜索"
buttonAriaLabel = "搜索"
noResultsText = "没有找到 “{q}” 的结果"
resetButtonTitle = "清除"
selectText = "选择"
navigateText = "切换"
closeText = "关闭"
```

A key under `[search.locales]` that names no locale in `[locales]` (other than `root`) fails the build.

### Tuning

There are no equivalents of VitePress's `miniSearch` options, `detailedView`, custom content renderers or `exclude` callbacks: the scoring is fixed, and pages are excluded through frontmatter.

## Algolia Search

Not available. rustpress has no integration with external search services; use the local provider above.
