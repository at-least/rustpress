---
description: Configure the sidebar navigation in rustpress — automatic sidebars, explicit groups, collapsible sections, multiple sidebars and path prefixes.
---

# Sidebar

The sidebar is the main navigation block for your documentation. You can configure the sidebar menu in [`sidebar`](./default-theme-config#sidebar).

```toml [rustpress.toml]
[[sidebar]]
text = "Guide"

  [[sidebar.items]]
  text = "Introduction"
  link = "/introduction/"

  [[sidebar.items]]
  text = "Getting Started"
  link = "/getting-started/"
```

## Automatic Sidebar

When `sidebar` is not configured at all, rustpress derives one sidebar per top-level content directory:

- Each directory directly under the content root (`guide/`, `reference/`, …) becomes one sidebar, shown on every page under that path.
- If the directory has an `index.md`, its title becomes the group title and the group heading links to it; otherwise the directory name is used.
- Every other page in the directory — including pages in nested subdirectories, which are flattened into the same group — becomes a leaf, labelled with its page title and ordered by natural file order (`page2.md` before `page10.md`).
- Pages at the content root (`index.md`, `prologue.md`) get no sidebar.

This is enough for small sites. Any explicit `sidebar` value replaces it entirely.

## The Basics

The simplest form of the sidebar menu is passing in a single array of links. The first level item defines the "section" for the sidebar. It should contain `text`, which is the title of the section, and `items` which are the actual navigation links.

```toml [rustpress.toml]
[[sidebar]]
text = "Section Title A"

  [[sidebar.items]]
  text = "Item A"
  link = "/item-a/"

  [[sidebar.items]]
  text = "Item B"
  link = "/item-b/"

[[sidebar]]
text = "Section Title B"

  [[sidebar.items]]
  text = "Item C"
  link = "/item-c/"
```

Each `link` should specify the path to the actual file starting with `/`. Links are canonicalized against the pages that exist: `/guide/intro`, `/guide/intro.md` and `/guide/intro/` all resolve to the page's real URL. A link ending with a slash shows the `index.md` of the corresponding directory.

```toml [rustpress.toml]
[[sidebar]]
text = "Guide"

  [[sidebar.items]]
  # This shows `/guide/index.md` page.
  text = "Introduction"
  link = "/guide/"
```

Links to external sites (`http://`, `https://`, `mailto:`) pass through untouched; add `target` and `rel` on the item when needed.

You may nest the sidebar items further: an item with `items` is a group, and groups can contain groups. There is no depth limit in rustpress, though the styling is designed for the same few levels VitePress uses.

```toml [rustpress.toml]
[[sidebar]]
text = "Level 1"

  [[sidebar.items]]
  text = "Level 2"

    [[sidebar.items.items]]
    text = "Level 3"

      [[sidebar.items.items.items]]
      text = "Level 4 leaf"
      link = "/deep/"
```

## Multiple Sidebars

You may show different sidebar depending on the page path. For example, as shown on this site, you might want to create separate sections of content in your documentation like "Guide" pages and "Config" pages.

To do so, first organize your pages into directories for each desired section:

```
.
├─ guide/
│  ├─ index.md
│  ├─ one.md
│  └─ two.md
└─ config/
   ├─ index.md
   ├─ three.md
   └─ four.md
```

Then, update your configuration to define your sidebar for each section. This time, key the `sidebar` table by path prefix instead of writing an array.

```toml [rustpress.toml]
# This sidebar gets displayed when a user
# is on `guide` directory.
[sidebar."/guide/"]

  [[sidebar."/guide/".items]]
  text = "Guide"

    [[sidebar."/guide/".items.items]]
    text = "Index"
    link = "/guide/"

    [[sidebar."/guide/".items.items]]
    text = "One"
    link = "/guide/one/"

    [[sidebar."/guide/".items.items]]
    text = "Two"
    link = "/guide/two/"

# This sidebar gets displayed when a user
# is on `config` directory.
[sidebar."/config/"]

  [[sidebar."/config/".items]]
  text = "Config"

    [[sidebar."/config/".items.items]]
    text = "Index"
    link = "/config/"

    [[sidebar."/config/".items.items]]
    text = "Three"
    link = "/config/three/"

    [[sidebar."/config/".items.items]]
    text = "Four"
    link = "/config/four/"
```

When several prefixes match a page, the longest one wins. Pages under no configured prefix get no sidebar.

## Collapsible Sidebar Groups

By adding `collapsed` option to the sidebar group, it shows a toggle button to hide/show each section.

```toml [rustpress.toml]
[[sidebar]]
text = "Section Title A"
collapsed = false

  [[sidebar.items]]
  # ...
```

All sections are "open" by default. If you would like them to be "closed" on initial page load, set `collapsed` option to `true`.

```toml [rustpress.toml]
[[sidebar]]
text = "Section Title A"
collapsed = true

  [[sidebar.items]]
  # ...
```

A group with no `collapsed` key at all is not collapsible and shows no toggle.

## Path Prefix

When your documentation structure has deep directories or groups located under the same subdirectory, you can use the `base` option to automatically prepend a path prefix to all nested `items` inside that group. This avoids repeating the same path prefix for every `link`.

The `base` option is supported in both multiple sidebar configurations and nested sidebar groups.

### In Multiple Sidebars

You can define `base` at the root of a sidebar section configuration:

```toml [rustpress.toml]
[sidebar."/guide/"]
base = "/guide/"

  # This link is resolved to `/guide/introduction/`
  [[sidebar."/guide/".items]]
  text = "Introduction"
  link = "introduction"

  # This link is resolved to `/guide/getting-started/`
  [[sidebar."/guide/".items]]
  text = "Getting Started"
  link = "getting-started"
```

### In Nested Groups

You can also use `base` inside nested sidebar groups. It applies to the group's own link and to every item below it until a nested group sets its own `base`; the nearest `base` wins.

```toml [rustpress.toml]
[[sidebar]]
text = "Reference"
base = "/reference/"

  # This link is resolved to `/reference/site-config/`
  [[sidebar.items]]
  text = "Site Config"
  link = "site-config"

  [[sidebar.items]]
  text = "Default Theme"
  # Nested base overrides the parent path prefix
  base = "/reference/default-theme-"

    # This link is resolved to `/reference/default-theme-nav/`
    [[sidebar.items.items]]
    text = "Nav"
    link = "nav"

    # This link is resolved to `/reference/default-theme-sidebar/`
    [[sidebar.items.items]]
    text = "Sidebar"
    link = "sidebar"
```

## Pager Text

The prev/next links at the bottom of a page show the sidebar label of the neighboring page. Set `docFooterText` on an item to show a different text there; see [Prev / Next Links](./default-theme-prev-next-links).

```toml [rustpress.toml]
[[sidebar.items]]
text = "Nav"
link = "nav"
docFooterText = "Configuring the navbar"
```
