---
description: Learn how to reference and serve static assets such as images, fonts and downloadable files in a rustpress site.
---

# Asset Handling

rustpress has no asset pipeline. Nothing is hashed, inlined or rewritten: files you put in the site's `static/` directory are copied to the output root as they are, and paths you write in markdown are emitted as you wrote them. That keeps asset handling predictable, but it also means you are responsible for placing every file you reference.

## The Static Directory

Put images, favicons, `robots.txt`, PDFs and any other file you want served under its original name in the `static/` directory of the site (the directory that holds `rustpress.toml`):

```
.
├─ rustpress.toml
├─ content/
│  └─ index.md
└─ static/
   ├─ logo.svg
   ├─ robots.txt
   └─ images/
      └─ screenshot.png
```

`static/` is copied verbatim to the root of the output directory (`public/`), so `static/images/screenshot.png` is served at `/images/screenshot.png`. Reference it from markdown with a root-absolute path:

```md
![A screenshot](/images/screenshot.png)
```

::: tip Linked files are checked
The dead-link checker verifies `href` and `src` values on every page against the site's pages **and** the files in the output directory. A link or image pointing at a file that is not in `static/` fails the build, unless [`ignoreDeadLinks`](../reference/site-config#ignoredeadlinks) silences it.
:::

This directory plays the role of VitePress's `public/` directory. The name differs because `public/` is where rustpress **writes** the built site.

### The built theme assets

The theme's stylesheet (`vitepress.css`), script bundle (`js/app.js`) and the Inter font files under `fonts/` are embedded in the binary and written into `public/` first on every build. Two further layers are copied over them, in this order:

1. the site's own `static/` — a file with the same name replaces the embedded one;
2. a `static/` directory in the site's **parent** directory, if there is one. This is a development convenience for the rustpress repository itself, where `demo/` and `docs/` sit next to the repo's `static/`: the freshly built assets win over the ones compiled into the binary.

A site anywhere else needs neither; the embedded files are enough. See [Getting Started](./getting-started#theme-assets).

## Relative Asset Paths

Relative paths in image references are passed through unchanged:

```md
![An image](./image.png)
```

The browser resolves `./image.png` against the page URL. Because every page is served from its own directory (`guide/x.md` becomes `/guide/x/`), a relative path resolves one level deeper than the markdown file's location, so files next to the markdown source are **not** picked up automatically and are not copied to the output. Prefer root-absolute paths into `static/`.

Relative **links** between markdown pages, on the other hand, are resolved at build time to the target page's URL; see [Routing](./routing).

## Base URL

If your site is deployed to a non-root URL, set the [`base`](../reference/site-config#base) option. For example, if you plan to deploy your site to `https://foo.github.io/bar/`, then `base` should be set to `"/bar/"`.

The base is prepended to every URL the theme generates: the navbar, sidebar, logo, hero actions, language switcher, prev/next links, and the stylesheet and script tags in `<head>`.

Root-absolute paths inside markdown get it too, as in VitePress: with `base = "/bar/"`, `![x](/image.png)` is emitted as `src="/bar/image.png"`, `[x](/guide/)` as `href="/bar/guide/"`, and a relative page link like `[Guide](./guide)` resolves to `/bar/guide/`. Write paths as if the site were at the root and let `base` do the rest.

::: warning Raw HTML and `head` entries are emitted as written
The rewrite covers markdown link and image syntax only. A raw `<a href="/x">` or `<img src="/x">` in your content, and the `href` values of [`[[head]]`](../reference/site-config#head) entries, are copied verbatim, so include the base in those yourself.
:::

## Serving Assets from a CDN

There is no equivalent of VitePress's `assetsBase`. Assets are plain files next to the pages; to serve them from another origin, write absolute URLs in your markdown.
