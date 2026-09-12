---
description: Configure the rustpress home page layout with a hero section and feature cards.
---

# Home Page

rustpress provides a homepage layout, which you can also see used on [the homepage of this site](../). You may use it on any of your pages by specifying `layout: home` in the [frontmatter](./frontmatter-config).

```yaml
---
layout: home
---
```

However, this option alone wouldn't do much. You can add several different pre templated "sections" to the homepage by setting additional other options such as `hero` and `features`.

## Hero Section

The Hero section comes at the top of the homepage. Here's how you can configure the Hero section.

```yaml
---
layout: home

hero:
  name: rustpress
  text: VitePress-format docs from one Rust binary.
  tagline: Lorem ipsum...
  image:
    src: /logo.png
    alt: rustpress
  actions:
    - theme: brand
      text: Get Started
      link: /guide/what-is-rustpress
    - theme: alt
      text: View on GitHub
      link: https://github.com/at-least/rustpress
---
```

The hero keys are:

| key | description |
| --- | --- |
| `name` | The string shown on top of `text`. Comes with brand color and expected to be short, such as product name. |
| `text` | The main text for the hero section. Rendered inside the page's `h1`. |
| `tagline` | Tagline displayed below `text`. |
| `image` | Image displayed next to the text and tagline area: a path, `{ src, alt }`, or `{ light, dark, alt }` for a pair switched by the color scheme. |
| `actions` | Action buttons to display in the hero section. |

Each action has:

| key | description |
| --- | --- |
| `theme` | Color theme of the button: `brand` (default), `alt`, or `sponsor`. |
| `text` | Label of the button. |
| `link` | Destination link. Full URLs are used as-is. A path naming a page (`/guide/intro`, `./guide/intro`, `guide/intro.md`, all resolved from the site root) becomes that page's URL, with [`base`](./site-config#base) applied; any other path is emitted as written. |
| `target` | Link `target` attribute. |
| `rel` | Link `rel` attribute. |

### Customizing the name color

rustpress uses the brand color (`--vp-c-brand-1`) for the `name`. However, you may customize this color by overriding `--vp-home-hero-name-color` variable in your [theme CSS](../guide/coming-from-vitepress#theming).

```css
:root {
  --vp-home-hero-name-color: blue;
}
```

Also you may customize it further by combining `--vp-home-hero-name-background` to give the `name` gradient color.

```css
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #bd34fe, #41d1ff);
}
```

## Features Section

In Features section, you can list any number of features you would like to show right after the Hero section. To configure it, pass `features` option to the frontmatter.

You can provide an icon for each feature, which can be an emoji or any type of image. When the configured icon is an image (svg, png, jpeg...), you should provide the icon with the proper width and height; you can also provide the description, its intrinsic size as well as its variants for dark and light theme when required.

```yaml
---
layout: home

features:
  - icon: 🛠️
    title: Simple and minimal, always
    details: Lorem ipsum...
  - icon:
      src: /cool-feature-icon.svg
      width: 48
      height: 48
    title: Another cool feature
    details: Lorem ipsum...
  - icon:
      dark: /dark-feature-icon.svg
      light: /light-feature-icon.svg
    title: Another cool feature
    details: Lorem ipsum...
---
```

::: warning Icons are text, not HTML
A string `icon` is rendered as escaped text. Emoji and plain characters work; an HTML snippet such as `<span class="my-icon"></span>` is shown literally. Use the image forms for anything that is not text.
:::

Each feature has:

| key | description |
| --- | --- |
| `icon` | A string (emoji/text), `{ src, alt, width, height }`, or `{ light, dark, alt, width, height }`. |
| `title` | Title of the feature. |
| `details` | Details of the feature. |
| `link` | Link when clicked on feature card. The whole card becomes the link. It can be both internal (`/guide/intro/`) or external (`https://example.com`). |
| `linkText` | Link text to be shown inside feature card, with an arrow. Best used with `link` option. E.g. `Learn more`, `Visit page`. |
| `rel` | Link `rel` attribute for the `link` option. |
| `target` | Link `target` attribute for the `link` option. |

The grid adapts to the number of features: two per row for two, three per row for three or a multiple of three, otherwise four per row on wide screens.

## Markdown Content

Not supported. A `layout: home` page renders only its hero and features; any markdown written below the `---` frontmatter divider is ignored. If you need prose on the landing page, put it on a `layout: doc` or `layout: page` page and link to it from a hero action.
