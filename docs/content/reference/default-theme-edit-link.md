---
description: Display an edit link on doc pages to let users suggest changes on GitHub or GitLab.
---

# Edit Link

## Site-Level Config

Edit Link lets you display a link to edit the page on Git management services such as GitHub, or GitLab. To enable it, add an `[editLink]` section to your config.

```toml [rustpress.toml]
[editLink]
pattern = "https://github.com/vuejs/vitepress/edit/main/docs/:path"
```

The `pattern` option defines the URL structure for the link, and `:path` is going to be replaced with the page path — the source file's path relative to the content directory ([`srcDir`](./site-config#srcdir)), e.g. `guide/getting-started.md`. If your content lives in `docs/content/`, the pattern therefore needs the `content/` segment too:

```toml [rustpress.toml]
[editLink]
pattern = "https://github.com/acme/project/edit/main/docs/content/:path"
```

Only the string form is supported; VitePress's function form (computing the URL from page data) needs JavaScript at build time.

By default, this will add the link text "Edit this page on GitHub" at the bottom of the doc page. You may customize this text by defining the `text` option.

```toml [rustpress.toml]
[editLink]
pattern = "https://github.com/vuejs/vitepress/edit/main/docs/:path"
text = "Edit this page on GitHub"
```

## Frontmatter Config

This can be disabled per-page using the `editLink` option on frontmatter:

```yaml
---
editLink: false
---
```
