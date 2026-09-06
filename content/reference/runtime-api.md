+++
title = "Runtime API"
weight = 3
description = "Reference of VitePress runtime APIs including composables, helper functions, and built-in components."
+++

VitePress offers several built-in APIs to let you access app data. VitePress also comes with a few built-in components that can be used globally.

The helper methods are globally importable from `vitepress` and are typically used in custom theme Vue components. However, they are also usable inside `.md` pages because markdown files are compiled into Vue [Single-File Components](https://vuejs.org/guide/scaling-up/sfc.html).

Methods that start with `use*` indicates that it is a [Vue 3 Composition API](https://vuejs.org/guide/introduction.html#composition-api) function ("Composable") that can only be used inside `setup()` or `<script setup>`.

## `useData` <span class="VPBadge info">composable</span>

Returns page-specific data. The returned object has the following type:

```ts
interface VitePressData<T = any> {
  /**
   * Site-level metadata
   */
  site: Ref<SiteData<T>>
  /**
   * themeConfig from .vitepress/config.js
   */
  theme: Ref<T>
  /**
   * Page-level metadata
   */
  page: Ref<PageData>
  /**
   * Page frontmatter
   */
  frontmatter: Ref<PageData['frontmatter']>
  /**
   * Dynamic route params
   */
  params: Ref<PageData['params']>
  title: Ref<string>
  description: Ref<string>
  lang: Ref<string>
  isDark: Ref<boolean>
  dir: Ref<string>
  localeIndex: Ref<string>
  /**
   * Current location hash
   */
  hash: Ref<string>
}

interface PageData {
  title: string
  titleTemplate?: string | boolean
  description: string
  relativePath: string
  filePath: string
  headers: Header[]
  frontmatter: Record<string, any>
  params?: Record<string, any>
  isNotFound?: boolean
  lastUpdated?: number
}
```

`page.headers` is populated only when [`markdown.headers`](/reference/site-config/#markdown) is enabled. Without that option, it remains an empty array. The default theme outline reads rendered headings from the page content, so it can still appear when `page.headers` is empty.

**Example:**

```vue
<script setup>
import { useData } from 'vitepress'

const { theme } = useData()
</script>

<template>
{% raw %}  <h1>{{ theme.footer.copyright }}</h1>{% endraw %}
</template>
```

## `useRoute` <span class="VPBadge info">composable</span>

Returns the current route object with the following type:

```ts
interface Route {
  path: string
  data: PageData
  component: Component | null
}
```

## `useRouter` <span class="VPBadge info">composable</span>

Returns the VitePress router instance so you can programmatically navigate to another page.

```ts
interface Router {
  /**
   * Current route.
   */
  route: Route
  /**
   * Navigate to a new URL.
   */
  go: (to?: string) => Promise<void>
  /**
   * Called before the route changes. Return `false` to cancel the navigation.
   */
  onBeforeRouteChange?: (to: string) => Awaitable<void | boolean>
  /**
   * Called before the page component is loaded (after the history state is updated).
   * Return `false` to cancel the navigation.
   */
  onBeforePageLoad?: (to: string) => Awaitable<void | boolean>
  /**
   * Called after the page component is loaded (before the page component is updated).
   */
  onAfterPageLoad?: (to: string) => Awaitable<void>
  /**
   * Called after the route changes.
   */
  onAfterRouteChange?: (to: string) => Awaitable<void>
}
```

Assign route-change handlers on the router instance:

```ts
const router = useRouter()

router.onBeforeRouteChange = (to) => {
  console.log('navigating to', to)
}
```

For custom themes, the same router is available from [`enhanceApp`](/guide/customization/custom-theme/#theme-interface).

## `useIcon` <span class="VPBadge info">composable</span>

- **Type**: `(icon: MaybeRefOrGetter<string | { svg: string } | undefined>, el?: MaybeRefOrGetter<HTMLElement | null>) => ComputedRef<string | undefined>`

Renders an [iconify](https://iconify.design/) icon through VitePress's icon pipeline. Takes a fully qualified `collection:name` (resolved against the `@iconify-json/*` packages in your project's dependencies) and returns the class to put on the element — `vpi-<collection>-<name>`.

During SSR the name is registered on the page's [`SSGContext`](/reference/site-config/#postrender), so the build emits the icon's styles into the generated stylesheet; in dev, icons are served on demand by the dev server from the locally installed collections. No icon is ever fetched from an external service.

```vue
<script setup>
import { useIcon } from 'vitepress'
import { useTemplateRef } from 'vue'

const el = useTemplateRef('el')
const iconClass = useIcon('lucide:rocket', el)
</script>

<template>
  <span ref="el" :class="iconClass" />
</template>
```

Pass the template ref of the element carrying the class so dev mode can resolve the icon on it. The element needs the mask rules the default theme ships; in a custom theme without them, dev applies an inline equivalent and the generated stylesheet includes zero-specificity base rules for production.

When using the default theme, the `VPIcon` component from `vitepress/theme` wraps this composable (and also accepts a raw `{ svg }` string):

```vue-html
<VPIcon icon="lucide:rocket" />
```

Icons rendered only on the client (e.g. inside `<ClientOnly />`) can't be collected during the build — list them in [`icons.include`](/reference/site-config/#icons) instead.

## `withBase` <span class="VPBadge info">helper</span>

- **Type**: `(path: string) => string`

Prepends the configured [`base`](/reference/site-config/#base) to a given URL path. Also see [Base URL](/guide/writing/asset-handling/#base-url).

## `<Content />` <span class="VPBadge info">component</span>

The `<Content />` component displays the rendered markdown contents. Useful [when creating your own theme](/guide/customization/custom-theme/).

```vue
<template>
  <h1>Custom Layout!</h1>
  <Content />
</template>
```

## `<ClientOnly />` <span class="VPBadge info">component</span>

The `<ClientOnly />` component renders its slot only at client side.

Because VitePress applications are server-rendered in Node.js when generating static builds, any Vue usage must conform to the universal code requirements. In short, make sure to only access Browser / DOM APIs in beforeMount or mounted hooks.

If you are using or demoing components that are not SSR-friendly (for example, contain custom directives), you can wrap them inside the `ClientOnly` component.

```vue-html
<ClientOnly>
  <NonSSRFriendlyComponent />
</ClientOnly>
```

- Related: [SSR Compatibility](/guide/customization/ssr-compat/)

## `$frontmatter` <span class="VPBadge info">template global</span>

Directly access current page's [frontmatter](/guide/writing/frontmatter/) data in Vue expressions.

```md
---
title: Hello
---

{% raw %}# {{ $frontmatter.title }}{% endraw %}
```

## `$params` <span class="VPBadge info">template global</span>

Directly access current page's [dynamic route params](/guide/introduction/routing/#dynamic-routes) in Vue expressions.

```md
{% raw %}- package name: {{ $params.pkg }}
- version: {{ $params.version }}{% endraw %}
```
