---
description: Learn how to use YAML frontmatter in rustpress markdown files to control page-level metadata and behavior.
---

# Frontmatter

## Usage

rustpress supports YAML frontmatter in all markdown files. The frontmatter must be at the very top of the file and must take the form of valid YAML set between triple-dashed lines. Example:

```md
---
title: Docs with rustpress
editLink: false
---
```

Many site or default theme config options have corresponding options in frontmatter. You can use frontmatter to override specific behavior for the current page only. For details, see the [Frontmatter Config Reference](../reference/frontmatter-config).

Keys the theme does not know are tolerated and ignored, so custom metadata can live in frontmatter without breaking the build. There is however no template language to read it back: unlike VitePress, `{{ $frontmatter.title }}` in the page body stays literal text.

## Title

The page title comes from, in order:

1. the `title` frontmatter key;
2. the first `# H1` heading in the body (code fences are skipped);
3. the file name.

The title is used for the `<title>` element, the search index, and the auto-derived sidebar. Explicit sidebar entries in `rustpress.toml` carry their own `text`.

## Alternative Frontmatter Formats

Because JSON is valid YAML, a JSON object between the triple-dashed lines works as well:

```md
---
{
  "title": "Blogging Like a Hacker",
  "editLink": false
}
---
```
