---
description: Customize the previous and next page links displayed at the bottom of doc pages in rustpress.
---

# Prev Next Links

You can customize the text and link for the previous and next pages (shown at doc footer). This is helpful if you want a different text there than what you have on your sidebar. Additionally, you may find it useful to disable the footer or link to a page that is not included in your sidebar.

By default the pager follows the active sidebar: the previous and next leaves in sidebar order, labelled with their sidebar text (or the item's [`docFooterText`](./default-theme-sidebar#pager-text)). Pages without a sidebar have no pager.

## prev

- Type: `string | false | { text: string; link: string }`

- Details:

  Specifies the text/link to show on the link to the previous page. If you don't set this in frontmatter, the text/link will be inferred from the sidebar config.

- Examples:

  - To customize only the text:

    ```yaml
    ---
    prev: 'Get Started | Markdown'
    ---
    ```

  - To customize both text and link:

    ```yaml
    ---
    prev:
      text: 'Markdown'
      link: '/guide/markdown'
    ---
    ```

    In this form both `text` and `link` are required. `target` and `rel` keys are accepted but not emitted on the rendered link.

  - To hide previous page:

    ```yaml
    ---
    prev: false
    ---
    ```

## next

Same as `prev` but for the next page.

## Site-Level Labels

The small "Previous page" / "Next page" captions above the links come from `[docFooter]`; set a side to `false` to remove that side of the pager on every page.

```toml [rustpress.toml]
[docFooter]
prev = "Pagina prior"
next = "Proxima pagina"
```
