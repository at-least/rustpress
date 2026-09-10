---
description: Choose between the doc, page, and home layouts in rustpress.
---

# Layout

You may choose the page layout by setting `layout` option to the page [frontmatter](./frontmatter-config). There are 3 layout options, `doc`, `page`, and `home`. If nothing is specified, then the page is treated as `doc` page.

```yaml
---
layout: doc
---
```

## Doc Layout

Option `doc` is the default layout and it styles the whole Markdown content into "documentation" look. It works by wrapping whole content within `vp-doc` css class, and applying styles to elements underneath it.

Almost all generic elements such as `p`, or `h2` get special styling. Therefore, keep in mind that if you add any custom HTML inside a Markdown content, those will get affected by those styles as well.

It also provides documentation specific features listed below. These features are only enabled in this layout.

- [Edit Link](./default-theme-edit-link)
- [Last Updated](./default-theme-last-updated)
- Outline (the right-hand "On this page" aside)

[Prev / Next Links](./default-theme-prev-next-links) render in both the `doc` and the `page` layout.

## Page Layout

Option `page` is treated as a "blank page". The Markdown is still parsed and all of the [Markdown Extensions](../guide/markdown) work the same as in the `doc` layout, but the page renders without most of the doc chrome: no aside/outline, no edit link and no last-updated timestamp. The [prev/next pager](./default-theme-prev-next-links) still renders; hide it per page with `prev: false` and `next: false` in the frontmatter.

Note that even in this layout, the navbar, the sidebar (if the page has a matching sidebar config) and the footer still show up. The `navbar` and `footer` frontmatter keys hide those two; see the [`sidebar`](./frontmatter-config#sidebar) frontmatter entry for why that key does not currently remove the sidebar panel.

## Home Layout

Option `home` will generate templated "Homepage". In this layout, you can set extra options such as `hero` and `features` to customize the content further. Please visit [Default Theme: Home Page](./default-theme-home-page) for more details.

## No Layout and Custom Layouts

Not available. VitePress's `layout: false` and custom Vue layout components need a Vue runtime; rustpress renders one compiled-in theme with the three layouts above. Any other `layout` value is treated as `doc`.
