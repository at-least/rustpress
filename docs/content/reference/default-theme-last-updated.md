---
description: Show the last updated timestamp on rustpress pages based on Git commit history.
---

# Last Updated

The update time of the last content will be displayed in the lower right corner of the page. To enable it, add the `lastUpdated` option to your config.

::: info
rustpress displays the "last updated" time using the timestamp of the most recent Git commit for each file. To use this, the Markdown file must be committed to Git; files with no Git history fall back to their modification time on disk.

Internally, rustpress answers every page's timestamp with a single batched `git log` walk over the content tree (a subprocess per page does not scale). If all pages show the same update time, it's likely due to shallow cloning (common in CI environments), which limits Git history.

To fix this in **GitHub Actions**, use the following in your workflow:

```yaml{4}
- name: Checkout
  uses: actions/checkout@v5
  with:
    fetch-depth: 0
```

Other CI/CD platforms have similar settings. If such options aren't available, run `git fetch --unshallow` before the build.
:::

## Site-Level Config

```toml [rustpress.toml]
lastUpdated = true
```

The label in front of the date is `lastUpdatedText` (default `Last updated`):

```toml [rustpress.toml]
lastUpdated = true
lastUpdatedText = "Updated at"
```

Dates render VitePress-style, as `MMM D, YYYY, h:mm:ss A` in UTC (e.g. `Sep 18, 2026, 4:52:09 PM`); the wrapping `<time>` element carries an ISO `YYYY-MM-DD` `datetime` attribute. VitePress's `formatOptions` has no counterpart. Enabling `lastUpdated` also adds `<lastmod>` to the [sitemap](../guide/sitemap-generation).

## Frontmatter Config

This can be disabled per-page using the `lastUpdated` option on frontmatter:

```yaml
---
lastUpdated: false
---
```

A string value is displayed verbatim instead of the Git timestamp, which is useful for pages whose content predates the repository:

```yaml
---
lastUpdated: "March 2026"
---
```
