---
description: rustpress's built-in Markdown extensions — the VitePress dialect — including custom containers, GitHub alerts, code blocks with syntax highlighting, line highlighting, code groups, snippets, includes and math.
outline: deep
---

# Markdown Extensions

rustpress renders the VitePress Markdown dialect. Everything on this page is handled at build time by the Rust renderer: [comrak](https://github.com/kivikakk/comrak) parses GitHub-flavored Markdown, and a fence-aware preprocessor expands the VitePress-specific syntax around it. There is no markdown-it, no plugins and no Vue — what is listed here is exactly what the renderer understands.

## Header Anchors

Headers automatically get GitHub-style anchor ids and a `#` permalink.

### Custom anchors

To specify a custom anchor tag for a heading instead of using the auto-generated one, add a suffix to the heading:

```
# Using custom anchors {#my-anchor}
```

This allows you to link to the heading as `#my-anchor` instead of the default `#using-custom-anchors`. The suffix is removed from the rendered heading text, and the right-hand outline uses the custom id too.

## Links

Both internal and external links get special treatment.

### Internal Links

Internal links are resolved to the final page URLs at build time. Every `index.md` contained in each sub-directory becomes that directory's root page, with the URL `/`-terminated.

For example, given the following directory structure:

```
.
├─ index.md
├─ foo
│  ├─ index.md
│  ├─ one.md
│  └─ two.md
└─ bar
   ├─ index.md
   ├─ three.md
   └─ four.md
```

And providing you are in `foo/one.md`:

```md
[Home](/) <!-- sends the user to the root index.md -->
[foo](/foo/) <!-- sends the user to index.html of directory foo -->
[foo heading](./#heading) <!-- anchors user to a heading in the foo index file -->
[bar - three](../bar/three) <!-- you can omit extension -->
[bar - three](../bar/three.md) <!-- you can append .md -->
[bar - four](../bar/four.html) <!-- or you can append .html -->
```

Relative links are resolved against the *source* location of the current page, `.md` and `.html` suffixes are stripped, and the result is the target page's canonical URL. A relative link that does not point at any page is left exactly as written — and reported by the dead-link checker, unless [`ignoreDeadLinks`](../reference/site-config#ignoredeadlinks) says otherwise.

### Page Suffix

Pages are always generated as directories: `guide/markdown.md` becomes `/guide/markdown/index.html` and is linked as `/guide/markdown/`. There is no `.html`-suffix mode, and no server-side rewrite is needed for clean URLs. See [Routing](./routing).

### External Links

Outbound links are marked with an external-link icon:

- [vuejs.org](https://vuejs.org)
- [rustpress on GitHub](https://github.com/at-least/rustpress)

Unlike VitePress, no `target="_blank"` is added — external links open in the same tab. Use a [link attribute block](#additional-attributes) when you want one to open elsewhere.

## Frontmatter

[YAML frontmatter](https://jekyllrb.com/docs/front-matter/) is supported out of the box:

```yaml
---
title: Blogging Like a Hacker
description: A page about writing
---
```

Recognized keys control the page's title, layout, outline, sidebar and more. Unknown keys are tolerated and ignored. For more details, see [Frontmatter](../reference/frontmatter-config).

## GitHub-Style Tables

**Input**

```md
| Tables        |      Are      |  Cool |
| ------------- | :-----------: | ----: |
| col 3 is      | right-aligned | $1600 |
| col 2 is      |   centered    |   $12 |
| zebra stripes |   are neat    |    $1 |
```

**Output**

| Tables        |      Are      |   Cool |
| ------------- | :-----------: | -----: |
| col 3 is      | right-aligned | \$1600 |
| col 2 is      |   centered    |   \$12 |
| zebra stripes |   are neat    |    \$1 |

## Task Lists

**Input**

```md
- [ ] Write the press release
- [x] Update the website
```

**Output**

- [ ] Write the press release
- [x] Update the website

## Footnotes

**Input**

```md
Footnotes are supported[^1], including inline ones^[This is an inline footnote.].

[^1]: Definitions can contain **markdown** and are rendered at the end of the page.
```

**Output**

Footnotes are supported[^1], including inline ones^[This is an inline footnote.].

[^1]: Definitions can contain **markdown** and are rendered at the end of the page.

## Emoji :tada:

**Input**

```
:tada: :100:
```

**Output**

:tada: :100:

Shortcodes follow the GitHub emoji names.

## Table of Contents

**Input**

```
[[toc]]
```

**Output**

[[toc]]

The table of contents lists the page's `h2` and `h3` headings. It is not configurable.

## Custom Containers

Custom containers can be defined by their types, titles, and contents.

### Default Title

**Input**

```md
::: info
This is an info box.
:::

::: tip
This is a tip.
:::

::: warning
This is a warning.
:::

::: danger
This is a dangerous warning.
:::

::: details
This is a details block.
:::
```

**Output**

::: info
This is an info box.
:::

::: tip
This is a tip.
:::

::: warning
This is a warning.
:::

::: danger
This is a dangerous warning.
:::

::: details
This is a details block.
:::

The `note`, `important` and `caution` types are available too, with the same styling as the corresponding [GitHub-flavored alerts](#github-flavored-alerts).

### Custom Title

You may set custom title by appending the text right after the "type" of the container.

**Input**

````md
::: danger STOP
Danger zone, do not proceed
:::

::: details Click me to toggle the code
```js
console.log('Hello, rustpress!')
```
:::
````

**Output**

::: danger STOP
Danger zone, do not proceed
:::

::: details Click me to toggle the code
```js
console.log('Hello, rustpress!')
```
:::

Also, you may set custom titles globally by adding the following content in `rustpress.toml`, helpful if not writing in English:

```toml [rustpress.toml]
[markdown.container]
tipLabel = "提示"
warningLabel = "警告"
dangerLabel = "危险"
infoLabel = "信息"
noteLabel = "注意"
importantLabel = "重要"
cautionLabel = "小心"
detailsLabel = "详细信息"
```

These labels are site-wide; there is no per-locale override.

### Registering New Containers

Beyond the built-in types, you can register additional containers. Each one reuses the styling of a built-in `kind` (default: `tip`) and gets its own default title (`label`, default: the name uppercased):

```toml [rustpress.toml]
[[markdown.container.custom]]
name = "success"
kind = "tip"
label = "SUCCESS"
```

Registered names work like the built-in ones — including custom titles, attributes, and the [GitHub-style alert syntax](#github-flavored-alerts):

```md
::: success
You have completed the walkthrough!
:::

> [!SUCCESS] Custom title
> This renders the same way.
```

Unlike VitePress, a registered container is not an unstyled `.custom-block.success` waiting for your CSS; it renders with `kind`'s classes and colors.

### Nesting

The `:::` markers follow the same rules as fenced code blocks (` ``` `): a fence is only closed by a matching fence that is **at least as long** as the one that opened it. To nest containers (or to mix them with [code groups](#code-groups)) make the outer fence longer than the ones inside it.

**Input**

`````md
:::: info Outer container
This box contains another container.

::: details Inner container
```js
console.log('Hello, rustpress!')
```
:::
::::
`````

**Output**

:::: info Outer container
This box contains another container.

::: details Inner container
```js
console.log('Hello, rustpress!')
```
:::
::::

Containers opened inside a list item stay inside the item.

### Additional Attributes

A small set of attribute blocks is understood. On a `details` container, `{open}` makes the block open by default:

**Input**

````md
::: details Click me to toggle the code {open}
```js
console.log('Hello, rustpress!')
```
:::
````

**Output**

::: details Click me to toggle the code {open}
```js
console.log('Hello, rustpress!')
```
:::

The special `no-title` attribute renders a container without a title element (it has no effect on `details`, which always needs its summary):

**Input**

```md
::: tip {no-title}
Just want to try it out? Skip to the [Quickstart](./getting-started).
:::
```

**Output**

::: tip {no-title}
Just want to try it out? Skip to the [Quickstart](./getting-started).
:::

On links, an attribute block of `key="value"` pairs is copied onto the anchor — for example to open a link in the same tab or a new one:

```md
[Link to pure.html](/pure.html){target="_self"}
[Open in a new tab](https://example.com){target="_blank" rel="noreferrer"}
```

The link text is emitted as plain text (Markdown inside it is not parsed). Attribute blocks are not supported on other elements.

### `raw`

This is a special container that wraps its content in a `<div class="vp-raw">`, so that embedded markup can opt out of the theme's content styling.

**Syntax**

```md
::: raw
Wraps in a `<div class="vp-raw">`
:::
```

`vp-raw` class can be directly used on elements too. Nothing else happens — there is no PostCSS step to configure.

## GitHub-flavored Alerts

rustpress also supports [GitHub-flavored alerts](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#alerts) to render as callouts. They will be rendered the same as the [custom containers](#custom-containers). Unlike on GitHub, text placed right after the marker becomes the title of the alert (`> [!NOTE] Custom Title`), and [containers you registered yourself](#registering-new-containers) work here too.

```md
> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.
```

> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.

By default, alert colors match GitHub's, with caution and danger both rendering in red. Enable [`gradedContainers`](../reference/default-theme-config#gradedcontainers) to use a graded severity scale: danger (red), warning (orange), and caution (yellow). Note that `[!DANGER]` is a VitePress extension and will render as a regular blockquote on GitHub.

## Syntax Highlighting in Code Blocks

rustpress uses [tree-sitter](https://tree-sitter.github.io/) to highlight language syntax in Markdown code blocks. The grammars are compiled into the binary, so no download or configuration is needed. All you need to do is append a valid language alias to the beginning backticks for the code block:

**Input**

````
```js
export default {
  name: 'MyComponent',
  // ...
}
```
````

````
```html
<ul>
  <li v-for="todo in todos" :key="todo.id">
    {{ todo.text }}
  </li>
</ul>
```
````

**Output**

```js
export default {
  name: 'MyComponent'
  // ...
}
```

```html
<ul>
  <li v-for="todo in todos" :key="todo.id">
    {{ todo.text }}
  </li>
</ul>
```

The bundled grammars and the aliases that select them:

| language   | aliases                              |
| ---------- | ------------------------------------ |
| Bash       | `bash`, `sh`, `shell`, `zsh`         |
| JavaScript | `javascript`, `js`, `jsx`, `mjs`, `cjs` |
| TypeScript | `typescript`, `ts`                   |
| TSX        | `tsx`                                |
| JSON       | `json`, `jsonc`                      |
| YAML       | `yaml`, `yml`                        |
| TOML       | `toml`                               |
| HTML       | `html`                               |
| CSS        | `css`                                |
| Python     | `python`, `py`                       |
| Rust       | `rust`, `rs`                         |
| Go         | `go`, `golang`                       |
| C          | `c`                                  |
| C++        | `cpp`, `c++`                         |

Any other language (`vue`, `md`, `nginx`, …) is accepted, keeps its name as the block label, and renders as plain escaped text.

The colors come from a pair of Helix TOML themes (light/dark), selected in the `[syntax]` section of `rustpress.toml`; all 218 themes bundled with the Helix editor are available by name. See [Theming](./coming-from-vitepress#theming).

## Line Highlighting in Code Blocks

**Input**

````
```js{4}
export default {
  data () {
    return {
      msg: 'Highlighted!'
    }
  }
}
```
````

**Output**

```js{4}
export default {
  data () {
    return {
      msg: 'Highlighted!'
    }
  }
}
```

In addition to a single line, you can also specify multiple single lines, ranges, or both:

- Line ranges: for example `{5-8}`, `{3-10}`, `{10-17}`
- Multiple single lines: for example `{4,7,9}`
- Line ranges and single lines: for example `{4,7-13,16,23-27,40}`

**Input**

````
```js{1,4,6-8}
export default { // Highlighted
  data () {
    return {
      msg: `Highlighted!
      This line isn't highlighted,
      but this and the next 2 are.`,
      motd: 'rustpress is awesome',
      lorem: 'ipsum'
    }
  }
}
```
````

**Output**

```js{1,4,6-8}
export default { // Highlighted
  data () {
    return {
      msg: `Highlighted!
      This line isn't highlighted,
      but this and the next 2 are.`,
      motd: 'rustpress is awesome',
      lorem: 'ipsum',
    }
  }
}
```

Alternatively, it's possible to highlight directly in the line by using the `// [!code highlight]` comment.

**Input**

````
```js
export default {
  data () {
    return {
      msg: 'Highlighted!' // [!!code highlight]
    }
  }
}
```
````

**Output**

```js
export default {
  data() {
    return {
      msg: 'Highlighted!' // [!code highlight]
    }
  }
}
```

::: tip Showing the notation literally
To show a `[!code …]` marker as text instead of applying it — as the *Input* blocks on this page do — write it with two exclamation marks: `[!!code highlight]`. The extra `!` is removed and the notation is left in the output untouched.
:::

## Focus in Code Blocks

Adding the `// [!code focus]` comment on a line will focus it and blur the other parts of the code.

Additionally, you can define a number of lines to focus using `// [!code focus:<lines>]`.

**Input**

````
```js
export default {
  data () {
    return {
      msg: 'Focused!' // [!!code focus]
    }
  }
}
```
````

**Output**

```js
export default {
  data() {
    return {
      msg: 'Focused!' // [!code focus]
    }
  }
}
```

## Colored Diffs in Code Blocks

Adding the `// [!code --]` or `// [!code ++]` comments on a line will create a diff of that line, while keeping the colors of the codeblock.

**Input**

````
```js
export default {
  data () {
    return {
      msg: 'Removed' // [!!code --]
      msg: 'Added' // [!!code ++]
    }
  }
}
```
````

**Output**

```js
export default {
  data () {
    return {
      msg: 'Removed' // [!code --]
      msg: 'Added' // [!code ++]
    }
  }
}
```

## Errors and Warnings in Code Blocks

Adding the `// [!code warning]` or `// [!code error]` comments on a line will color it accordingly.

**Input**

````
```js
export default {
  data () {
    return {
      msg: 'Error', // [!!code error]
      msg: 'Warning' // [!!code warning]
    }
  }
}
```
````

**Output**

```js
export default {
  data() {
    return {
      msg: 'Error', // [!code error]
      msg: 'Warning' // [!code warning]
    }
  }
}
```

## Line Numbers

You can enable line numbers for each code blocks via config:

```toml [rustpress.toml]
[markdown]
lineNumbers = true
```

Please see the [`markdown` options](../reference/site-config#markdown) for more details.

You can add `:line-numbers` / `:no-line-numbers` mark in your fenced code blocks to override the value set in config.

You can also customize the starting line number by adding `=` after `:line-numbers`. For example, `:line-numbers=2` means the line numbers in code blocks will start from `2`.

**Input**

````md
```ts {1}
// line-numbers is disabled by default
const line2 = 'This is line 2'
const line3 = 'This is line 3'
```

```ts:line-numbers {1}
// line-numbers is enabled
const line2 = 'This is line 2'
const line3 = 'This is line 3'
```

```ts:line-numbers=2 {1}
// line-numbers is enabled and start from 2
const line3 = 'This is line 3'
const line4 = 'This is line 4'
```
````

**Output**

```ts {1}
// line-numbers is disabled by default
const line2 = 'This is line 2'
const line3 = 'This is line 3'
```

```ts:line-numbers {1}
// line-numbers is enabled
const line2 = 'This is line 2'
const line3 = 'This is line 3'
```

```ts:line-numbers=2 {1}
// line-numbers is enabled and start from 2
const line3 = 'This is line 3'
const line4 = 'This is line 4'
```

## Import Code Snippets

You can import code snippets from existing files via following syntax:

```md
<<< @/filepath
```

It also supports [line highlighting](#line-highlighting-in-code-blocks):

```md
<<< @/filepath{highlightLines}
```

**Input**

```md
<<< @/snippets/snippet.js{2}
```

**Code file**

<<< @/snippets/snippet.js

**Output**

<<< @/snippets/snippet.js{2}

::: tip
The value of `@` corresponds to the **site directory** — the directory containing `rustpress.toml` — regardless of `srcDir`. Alternatively, you can also import from paths relative to the current page's source file:

```md
<<< ../snippets/snippet.js
```

:::

You can also use a [VS Code region](https://code.visualstudio.com/docs/editor/codebasics#_folding) to only include the corresponding part of the code file. You can provide a custom region name after a `#` following the filepath:

**Input**

```md
<<< @/snippets/snippet-with-region.js#snippet{1}
```

**Code file**

<<< @/snippets/snippet-with-region.js

**Output**

<<< @/snippets/snippet-with-region.js#snippet{1}

The first region with that name is imported (in any comment style); its `#region` / `#endregion` marker lines are removed.

::: warning
A `<<<` directive whose file (or region) does not exist is not an error: the line is left in the page as literal text, so check the rendered output.
:::

You can also specify the language inside the braces (`{}`) like this:

```md
<<< @/snippets/snippet.cs{c#}

<!-- with line highlighting: -->

<<< @/snippets/snippet.cs{1,2,4-6 c#}

<!-- with line numbers: -->

<<< @/snippets/snippet.cs{1,2,4-6 c#:line-numbers}
```

This is helpful if source language cannot be inferred from your file extension. Files with the `.ansi` extension have their ANSI escape sequences stripped before rendering — handy for pasted terminal output.

## Code Groups

You can group multiple code blocks like this:

**Input**

````md
::: code-group

```toml [rustpress.toml]
title = "My Docs"

[search]
provider = "local"
```

```yaml [index.md]
---
layout: home
hero:
  name: My Docs
---
```

:::
````

**Output**

::: code-group

```toml [rustpress.toml]
title = "My Docs"

[search]
provider = "local"
```

```yaml [index.md]
---
layout: home
hero:
  name: My Docs
---
```

:::

The tab strip is rendered at build time from the `[label]` of each fence (falling back to the language name); switching tabs is the only part that needs JavaScript.

You can also [import snippets](#import-code-snippets) in code groups:

**Input**

```md
::: code-group

<!-- the language is used as the tab title by default -->

<<< @/snippets/snippet.js

<!-- you can provide a custom one too -->

<<< @/snippets/snippet-with-region.js#snippet{1,2 ts:line-numbers} [snippet with region]

:::
```

**Output**

::: code-group

<<< @/snippets/snippet.js

<<< @/snippets/snippet-with-region.js#snippet{1,2 ts:line-numbers} [snippet with region]

:::

## Markdown File Inclusion

You can include a markdown file in another markdown file, even nested.

::: tip
You can also prefix the markdown path with `@`, and it will act as the site directory (where `rustpress.toml` lives). Other paths resolve relative to the including page's source file.
:::

::: warning About the examples below
The *Input* blocks in this section are written with `@@include` so that the directive is **not** expanded while rendering this very page (the include directive is also processed inside code fences — see [Including Code Files](#including-code-files)). The real syntax has a single `@`: `<!--@include: ./parts/basics.md-->`.
:::

For example, you can include a relative markdown file using this:

**Input**

```md
# Docs

## Basics

<!--@@include: ./parts/basics.md-->
```

**Part file** (`parts/basics.md`)

```md
Some getting started stuff.

### Configuration

Can be created using `.foorc.json`.
```

**Equivalent code**

```md
# Docs

## Basics

Some getting started stuff.

### Configuration

Can be created using `.foorc.json`.
```

Note that every `.md` file under the source directory becomes a page of its own, so keep part files outside it — next to `rustpress.toml` and referenced with `@/`, for instance — or exclude them with [`srcExclude`](../reference/site-config#srcexclude).

It also supports selecting a line range:

**Input**

```md:line-numbers
# Docs

## Basics

<!--@@include: ./parts/basics.md{3,}-->
```

**Part file** (`parts/basics.md`)

```md:line-numbers
Some getting started stuff.

### Configuration

Can be created using `.foorc.json`.
```

**Equivalent code**

```md:line-numbers
# Docs

## Basics

### Configuration

Can be created using `.foorc.json`.
```

The format of the selected line range can be: `{3,}`, `{,10}`, `{1,10}`

You can also use a [VS Code region](https://code.visualstudio.com/docs/editor/codebasics#_folding) to only include the corresponding part of the code file. You can provide a custom region name after a `#` following the filepath:

**Input**

```md:line-numbers
# Docs

## Basics

<!--@@include: ./parts/basics.md#basic-usage{,2}-->
<!--@@include: ./parts/basics.md#basic-usage{5,}-->
```

**Part file** (`parts/basics.md`)

```md:line-numbers
<!-- #region basic-usage -->
## Usage Line 1

## Usage Line 2

## Usage Line 3
<!-- #endregion basic-usage -->
```

**Equivalent code**

```md:line-numbers
# Docs

## Basics

## Usage Line 1

## Usage Line 3
```

::: warning
Including a missing file throws a build error. A region or heading anchor that does not exist in the file includes nothing (an empty selection), silently.
:::

Instead of VS Code regions, you can also use header anchors to include a specific section of the file. For example, if you have a header in your markdown file like this:

```md
## My Base Section

Some content here.

### My Sub Section

Some more content here.

## Another Section

Content outside `My Base Section`.
```

You can include the `My Base Section` section like this:

```md
## My Extended Section
<!--@@include: ./parts/basics.md#my-base-section-->
```

**Equivalent code**

```md
## My Extended Section

Some content here.

### My Sub Section

Some more content here.
```

Here, `my-base-section` is the generated id of the heading element. In case it's not easily guessable, you can open the part file in your browser and click on the heading anchor (`#` symbol left to the heading when hovered) to see the id in the URL bar. Or use browser dev tools to inspect the element. Alternatively, you can also specify the id to the part file like this:

```md
## My Base Section {#custom-id}
```

and include it like this:

```md
<!--@@include: ./parts/basics.md#custom-id-->
```

Relative links and images inside included files are **not** rebased: they resolve relative to the including page, not the part file. Write links in part files with that in mind (root-absolute `/…` links are the safe choice).

### Including Code Files {#including-code-files}

Since inclusion happens before code blocks are parsed, the directive also works inside fences. Combined with a line range, this lets you show only part of a code file — an alternative to [importing snippets](#import-code-snippets) when regions are not an option:

**Input**

````md
```js
<!--@@include: @/snippets/snippet-with-region.js{2,4}-->
```
````

**Output**

```js
<!--@include: @/snippets/snippet-with-region.js{2,4}-->
```

Note that the included lines are inserted verbatim (indentation is preserved), and content containing backticks needs a longer outer fence.

## Math Equations

This is opt-in. Set `markdown.math` to `true` in your config file:

```toml [rustpress.toml]
[markdown]
math = true
```

**Input**

```md
When $a \ne 0$, there are two solutions to $(ax^2 + bx + c = 0)$ and they are
$$ x = {-b \pm \sqrt{b^2-4ac} \over 2a} $$

**Maxwell's equations:**

| equation                                                                                                                                                                  | description                                                                            |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| $\nabla \cdot \vec{\mathbf{B}}  = 0$                                                                                                                                      | divergence of $\vec{\mathbf{B}}$ is zero                                               |
| $\nabla \times \vec{\mathbf{E}}\, +\, \frac1c\, \frac{\partial\vec{\mathbf{B}}}{\partial t}  = \vec{\mathbf{0}}$                                                          | curl of $\vec{\mathbf{E}}$ is proportional to the rate of change of $\vec{\mathbf{B}}$ |
| $\nabla \times \vec{\mathbf{B}} -\, \frac1c\, \frac{\partial\vec{\mathbf{E}}}{\partial t} = \frac{4\pi}{c}\vec{\mathbf{j}}    \nabla \cdot \vec{\mathbf{E}} = 4 \pi \rho$ | _wha?_                                                                                 |
```

**Output**

When $a \ne 0$, there are two solutions to $(ax^2 + bx + c = 0)$ and they are
$$ x = {-b \pm \sqrt{b^2-4ac} \over 2a} $$

**Maxwell's equations:**

| equation                                                                                                                                                                  | description                                                                            |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| $\nabla \cdot \vec{\mathbf{B}}  = 0$                                                                                                                                      | divergence of $\vec{\mathbf{B}}$ is zero                                               |
| $\nabla \times \vec{\mathbf{E}}\, +\, \frac1c\, \frac{\partial\vec{\mathbf{B}}}{\partial t}  = \vec{\mathbf{0}}$                                                          | curl of $\vec{\mathbf{E}}$ is proportional to the rate of change of $\vec{\mathbf{B}}$ |
| $\nabla \times \vec{\mathbf{B}} -\, \frac1c\, \frac{\partial\vec{\mathbf{E}}}{\partial t} = \frac{4\pi}{c}\vec{\mathbf{j}}    \nabla \cdot \vec{\mathbf{E}} = 4 \pi \rho$ | _wha?_                                                                                 |

::: warning Typeset in the browser
Unlike VitePress, rustpress does not typeset math at build time. Pages that contain math load [MathJax](https://www.mathjax.org/) (tex-svg) from a CDN and render the formulas client-side; pages without math load nothing extra.
:::

## Image Lazy Loading

You can enable lazy loading for each image added via markdown by setting `lazyLoading` to `true` in your config file (VitePress's `lazyLoad` spelling is accepted too):

```toml [rustpress.toml]
[markdown.image]
# image lazy loading is disabled by default
lazyLoading = true
```

## Badges

The `<Badge>` tag renders an inline status badge, useful next to headings:

**Input**

```md
Title <Badge type="info" text="default" />
Title <Badge type="tip" text="^1.9.0" />
Title <Badge type="warning" text="beta" />
Title <Badge type="danger" text="caution" />
```

**Output**

Title <Badge type="info" text="default" /><br>
Title <Badge type="tip" text="^1.9.0" /><br>
Title <Badge type="warning" text="beta" /><br>
Title <Badge type="danger" text="caution" />

It is rewritten at build time into a plain `<span class="VPBadge …">`. See [Badge](../reference/default-theme-badge) for all types.

## Configuration

There is no markdown-it instance to extend and no plugin hook. Everything configurable about the renderer lives in the `[markdown]` section of `rustpress.toml`:

```toml [rustpress.toml]
[markdown]
lineNumbers = false      # number every code block (per-fence :line-numbers overrides)
codeCopyButton = true    # hover copy button on code blocks
math = false             # $…$ / $$…$$ via client-side MathJax

[markdown.image]
lazyLoading = false      # loading="lazy" on content images

[markdown.container]
tipLabel = "TIP"         # …and warningLabel, dangerLabel, infoLabel, noteLabel,
                         # importantLabel, cautionLabel, detailsLabel

[[markdown.container.custom]]
name = "success"
kind = "tip"
label = "SUCCESS"
```

See the full list in [Config Reference: `markdown`](../reference/site-config#markdown).
