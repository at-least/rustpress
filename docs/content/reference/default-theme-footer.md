---
description: Configure the global footer displayed at the bottom of rustpress pages.
---

# Footer

rustpress will display global footer at the bottom of the page when `[footer]` is present.

```toml [rustpress.toml]
[footer]
message = "Released under the MIT License."
copyright = "Copyright © 2019-present Evan You"
```

| key | description |
| --- | --- |
| `message` | The message shown right before copyright. |
| `copyright` | The actual copyright text. |

Both keys are optional. The configuration also supports HTML strings — the values are written into the page as-is. So, for example, if you want to configure footer text to have some links, you can adjust the configuration as follows:

```toml [rustpress.toml]
[footer]
message = 'Released under the <a href="https://github.com/vuejs/vitepress/blob/main/LICENSE">MIT License</a>.'
copyright = 'Copyright © 2019-present <a href="https://github.com/yyx990803">Evan You</a>'
```

::: warning
Only inline elements can be used in `message` and `copyright` as they are rendered inside a `<p>` element. Because the strings are not escaped, keep them under your own control.
:::

Note that footer will not be displayed when the [Sidebar](./default-theme-sidebar) is visible.

## Frontmatter Config

This can be disabled per-page using the `footer` option on frontmatter:

```yaml
---
footer: false
---
```
