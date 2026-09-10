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

- The build writes `search-docs.json` to the output root: one entry per page with its URL, title and the plain text of its content.
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

The strings of the search button and modal are configurable under `[search.translations]`. They apply site-wide — there is no per-locale set.

```toml [rustpress.toml]
[search]
provider = "local"

[search.translations]
buttonText = "搜索"
buttonAriaLabel = "搜索"
placeholder = "搜索文档"
noResultsText = "没有找到 “{q}” 的结果"
resetButtonTitle = "清除"
navigateText = "切换"
selectText = "选择"
closeText = "关闭"
```

| key | default | where it shows |
| --- | --- | --- |
| `buttonText` | `Search` | navbar button label |
| `buttonAriaLabel` | `Search` | navbar button `aria-label` |
| `placeholder` | `Search docs` | modal input placeholder |
| `noResultsText` | `No results for "{q}"` | empty state; `{q}` is replaced with the query |
| `resetButtonTitle` | `Clear` | clear-button tooltip |
| `navigateText` | `to navigate` | footer hint after the arrow keys |
| `selectText` | `to select` | footer hint after Enter |
| `closeText` | `to close` | footer hint after Esc |

### Tuning

There are no equivalents of VitePress's `miniSearch` options, `detailedView`, custom content renderers or `exclude` callbacks: the scoring is fixed, and pages are excluded through frontmatter.

## Algolia Search

Not available. rustpress has no integration with external search services; use the local provider above.
