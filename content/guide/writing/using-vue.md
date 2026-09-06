+++
title = "Using Vue in Markdown"
weight = 4
description = "Use Vue components and dynamic templating features directly inside Markdown files in VitePress."
+++

In VitePress, each Markdown file is compiled into HTML and then processed as a [Vue Single-File Component](https://vuejs.org/guide/scaling-up/sfc.html). This means you can use any Vue features inside the Markdown, including dynamic templating, using Vue components, or arbitrary in-page Vue component logic by adding a `<script>` tag.

It's worth noting that VitePress leverages Vue's compiler to automatically detect and optimize the purely static parts of the Markdown content. Static contents are optimized into single placeholder nodes and eliminated from the page's JavaScript payload for initial visits. They are also skipped during client-side hydration. In short, you only pay for the dynamic parts on any given page.

{% <tip kind="tip" title="SSR Compatibility" no_title={false}> %}
All Vue usage needs to be SSR-compatible. See [SSR Compatibility](/guide/customization/ssr-compat/) for details and common workarounds.
{% </tip> %}

## Templating

### Interpolation

Each Markdown file is first compiled into HTML and then passed on as a Vue component to the Vite process pipeline. This means you can use Vue-style interpolation in text:

**Input**

```md
{% raw %}{{ 1 + 1 }}{% endraw %}
```

**Output**

{% raw %}<div class="language-text"><pre><code>{{ 1 + 1 }}</code></pre></div>{% endraw %}

### Directives

Directives also work (note that by design, raw HTML is also valid in Markdown):

**Input**

```html
{% raw %}<span v-for="i in 3">{{ i }}</span>{% endraw %}
```

**Output**

{% raw %}<div class="language-text"><pre><code><span v-for="i in 3">{{ i }} </span></code></pre></div>{% endraw %}

## `<script>` and `<style>`

Root-level `<script>` and `<style>` tags in Markdown files work just like they do in Vue SFCs, including `<script setup>`, `<style module>`, etc. The main difference here is that there is no `<template>` tag: all other root-level content is Markdown. Also note that all tags should be placed **after** the frontmatter:

```html
---
hello: world
---

<script setup>
import { ref } from 'vue'

const count = ref(0)
</script>

## Markdown Content

{% raw %}The count is: {{ count }}{% endraw %}

<button :class="$style.button" @click="count++">Increment</button>

<style module>
.button {
  color: red;
  font-weight: bold;
}
</style>
```

{% <tip kind="warning" title="Avoid `<style scoped>` in Markdown" no_title={false}> %}
When used in Markdown, `<style scoped>` requires adding special attributes to every element on the current page, which will significantly bloat the page size. `<style module>` is preferred when locally-scoped styling is needed in a page.
{% </tip> %}

You also have access to VitePress' runtime APIs such as the [`useData` helper](/reference/runtime-api/#usedata), which provides access to current page's metadata:

**Input**

```html
<script setup>
import { useData } from 'vitepress'

const { page } = useData()
</script>

{% raw %}<pre>{{ page }}</pre>{% endraw %}
```

**Output**

```json
{
  "path": "/using-vue.html",
  "title": "Using Vue in Markdown",
  "frontmatter": {},
  ...
}
```

## Using Components

You can import and use Vue components directly in Markdown files.

### Importing in Markdown

If a component is only used by a few pages, it's recommended to explicitly import them where they are used. This allows them to be properly code-split and only loaded when the relevant pages are shown:

```md
<script setup>
import CustomComponent from '../components/CustomComponent.vue'
</script>

# Docs

This is a .md using a custom component

<CustomComponent />

## More docs

...
```

### Registering Components Globally

If a component is going to be used on most of the pages, they can be registered globally by customizing the Vue app instance. See relevant section in [Extending Default Theme](/guide/customization/extending-default-theme/#registering-global-components) for an example.

{% <tip kind="warning" title="IMPORTANT" no_title={false}> %}
Make sure a custom component's name either contains a hyphen or is in PascalCase. Otherwise, it will be treated as an inline element and wrapped inside a `<p>` tag, which will lead to hydration mismatch because `<p>` does not allow block elements to be placed inside it.
{% </tip> %}

### Using Components In Headers <ComponentInHeader />

You can use Vue components in the headers, but note the difference between the following syntaxes:

| Markdown                                                | Output HTML                               | Parsed Header |
| ------------------------------------------------------- | ----------------------------------------- | ------------- |
| <pre v-pre><code> # text &lt;Tag/&gt; </code></pre>     | `<h1>text <Tag/></h1>`                    | `text`        |
| <pre v-pre><code> # text \`&lt;Tag/&gt;\` </code></pre> | `<h1>text <code>&lt;Tag/&gt;</code></h1>` | `text <Tag/>` |

The HTML wrapped by `<code>` will be displayed as-is; only the HTML that is **not** wrapped will be parsed by Vue.

{% <tip kind="tip" title="" no_title={false}> %}
The output HTML is accomplished by [Markdown-it](https://github.com/Markdown-it/Markdown-it), while the parsed headers are handled by VitePress (and used for both the sidebar and document title).
{% </tip> %}


## Escaping

You can escape Vue interpolations by wrapping them in a `<span>` or other elements with the `v-pre` directive:

**Input**

```md
{% raw %}This <span v-pre>{{ will be displayed as-is }}</span>{% endraw %}
```

**Output**

<div class="escape-demo">
{% raw %}  <p>This <span v-pre>{{ will be displayed as-is }}</span></p>{% endraw %}
</div>

Alternatively, you can wrap the entire paragraph in a `v-pre` custom container:

```md
::: v-pre
{% raw %}{{ This will be displayed as-is }}{% endraw %}
:::
```

**Output**

<div class="escape-demo">

{% raw %}{{ This will be displayed as-is }}{% endraw %}

</div>

## Unescape in Code Blocks

By default, all fenced code blocks are automatically wrapped with `v-pre`, so no Vue syntax will be processed inside. To enable Vue-style interpolation inside fences, you can append the language with the `-vue` suffix, e.g. `js-vue`:

**Input**

````md
```js-vue
{% raw %}Hello {{ 1 + 1 }}{% endraw %}
```
````

**Output**

```js-vue
{% raw %}Hello {{ 1 + 1 }}{% endraw %}
```

Note that this might prevent certain tokens from being syntax highlighted properly.

## Using CSS Pre-processors

VitePress has [built-in support](https://vite.dev/guide/features.html#css-pre-processors) for CSS pre-processors: `.scss`, `.sass`, `.less`, `.styl` and `.stylus` files. There is no need to install Vite-specific plugins for them, but the corresponding pre-processor itself must be installed:

```
# .scss and .sass
npm install -D sass

# .less
npm install -D less

# .styl and .stylus
npm install -D stylus
```

Then you can use the following in Markdown and theme components:

```vue
<style lang="sass">
.title
  font-size: 20px
</style>
```

## Using Teleports

VitePress currently has SSG support for teleports to body only. For other targets, you can wrap them inside the built-in `<ClientOnly>` component or inject the teleport markup into the correct location in your final page HTML through [`postRender` hook](/reference/site-config/#postrender).

<ModalDemo />

{% <details summary="" open={false}> %}
```vue
<script setup lang="ts">
import { ref } from 'vue'
const showModal = ref(false)
</script>

<template>
  <button class="modal-button" @click="showModal = true">Show Modal</button>

  <Teleport to="body">
    <Transition name="modal">
      <div v-show="showModal" class="modal-mask">
        <div class="modal-container">
          <p>Hello from the modal!</p>
          <div class="model-footer">
            <button class="modal-button" @click="showModal = false">
              Close
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-mask {
  position: fixed;
  z-index: 200;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: opacity 0.3s ease;
}

.modal-container {
  width: 18.75rem;
  margin: auto;
  padding: 1.25rem 1.875rem;
  background-color: var(--vp-c-bg);
  border-radius: 0.125rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.33);
  transition: all 0.3s ease;
}

.model-footer {
  margin-top: 0.5rem;
  text-align: right;
}

.modal-button {
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  border-color: var(--vp-button-alt-border);
  color: var(--vp-button-alt-text);
  background-color: var(--vp-button-alt-bg);
}

.modal-button:hover {
  border-color: var(--vp-button-alt-hover-border);
  color: var(--vp-button-alt-hover-text);
  background-color: var(--vp-button-alt-hover-bg);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(1.1);
}
</style>
```
{% </details> %}

```md
<ClientOnly>
  <Teleport to="#modal">
    <div>
      // ...
    </div>
  </Teleport>
</ClientOnly>
```

<script setup>
import ModalDemo from '../../components/ModalDemo.vue'
import ComponentInHeader from '../../components/ComponentInHeader.vue'
</script>

<style>
.escape-demo {
  border: 1px solid var(--vp-c-border);
  border-radius: 8px;
  padding: 0 20px;
}
</style>


## VS Code IntelliSense Support

<!-- Based on https://github.com/vuejs/language-tools/pull/4321 -->

Vue provides IntelliSense support out of the box via the [Vue - Official VS Code plugin](https://marketplace.visualstudio.com/items?itemName=Vue.volar). However, to enable it for `.md` files, you need to make some adjustments to the configuration files.


1. Add `.md` pattern to the `include` and `vueCompilerOptions.vitePressExtensions` options in the tsconfig/jsconfig file:

{% <codegroup> %}
```json,name=tsconfig.json
{
  "include": [
    "docs/**/*.ts",
    "docs/**/*.vue",
    "docs/**/*.md",
  ],
  "vueCompilerOptions": {
    "vitePressExtensions": [".md"],
  },
}
```
{% </codegroup> %}

2. Add `markdown` to the `vue.server.includeLanguages` option in the VS Code setting:

{% <codegroup> %}
```json,name=.vscode/settings.json
{
  "vue.server.includeLanguages": ["vue", "markdown"]
}
```
{% </codegroup> %}
