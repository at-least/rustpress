---
description: Generate a sitemap.xml file for your rustpress site to improve search engine discoverability.
---

# Sitemap Generation

rustpress comes with out-of-the-box support for generating a `sitemap.xml` file for your site. To enable it, add the following to your `rustpress.toml`:

```toml
[sitemap]
hostname = "https://example.com"
```

Every page gets a `<url>` entry whose `<loc>` is the hostname followed by the page's URL. Each entry also carries a `<lastmod>` date: the last git commit touching the file when [`lastUpdated`](../reference/default-theme-last-updated) is enabled and the site lives in a git repository, and the file's modification time otherwise.

## Base URL

If you're using [`base`](../reference/site-config#base) in your config, **do not** append it to `hostname`. Page URLs already include the base, so the hostname should be the origin alone:

```toml
base = "/my-site/"

[sitemap]
hostname = "https://example.com"   # entries come out as https://example.com/my-site/...
```

This is the opposite of VitePress's advice, where the base has to be repeated in the hostname.

## Options

`hostname` is the only option. There is no `transformItems` hook and no way to set `changefreq` or `priority`; the sitemap lists every built page and nothing else.
