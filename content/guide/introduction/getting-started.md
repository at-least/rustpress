+++
title = "Getting Started"
weight = 2
description = "Get up and running with VitePress. Learn how to install, scaffold, and start developing your documentation site."
+++

## Try It Online

You can try VitePress directly in your browser on [StackBlitz](https://vitepress.new).

## Installation

### Prerequisites

- [Node.js](https://nodejs.org/) version 22 or higher.
- Terminal for accessing VitePress via its command line interface (CLI).
- Text Editor with [Markdown](https://en.wikipedia.org/wiki/Markdown) syntax support.
  - [VSCode](https://code.visualstudio.com/) is recommended, along with the [official Vue extension](https://marketplace.visualstudio.com/items?itemName=Vue.volar).

VitePress can be used on its own, or be installed into an existing project. In both cases, you can install it with:

{% <codegroup> %}

```sh,name=npm
$ npm add -D vitepress@next
```

```sh,name=pnpm
$ pnpm add -D vitepress@next
```

```sh,name=yarn
$ yarn add -D vitepress@next vue
```

```sh,name=bun
$ bun add -D vitepress@next
```

```sh,name=deno
$ deno add -D vitepress@next
```

{% </codegroup> %}

{% <tip kind="tip" title="NOTE" no_title={false}> %}

VitePress is an ESM-only package. Don't use `require()` to import it, and make sure your nearest `package.json` contains `"type": "module"`, or change the file extension of your relevant files like `.vitepress/config.js` to `.mjs`/`.mts`. Refer to [Vite's troubleshooting guide](http://vite.dev/guide/troubleshooting.html#this-package-is-esm-only) for more details. Also, inside async CJS contexts, you can use `await import('vitepress')` instead.

{% </tip> %}

### Setup Wizard

VitePress ships with a command line setup wizard that will help you scaffold a basic project. After installation, start the wizard by running:

{% <codegroup> %}

```sh,name=npm
$ npx vitepress init
```

```sh,name=pnpm
$ pnpm vitepress init
```

```sh,name=yarn
$ yarn vitepress init
```

```sh,name=bun
$ bun vitepress init
```

{% </codegroup> %}

You will be greeted with a few simple questions:

```
┌  Welcome to VitePress!
│
◇  Where should VitePress initialize the config?
│  ./docs
│
◇  Where should VitePress look for your markdown files?
│  ./docs
│
◇  Site title:
│  My Awesome Project
│
◇  Site description:
│  A VitePress Site
│
◇  Theme:
│  Default Theme
│
◇  Use TypeScript for config and theme files?
│  Yes
│
◇  Add VitePress npm scripts to package.json?
│  Yes
│
◇  Add a prefix for VitePress npm scripts?
│  Yes
│
◇  Prefix for VitePress npm scripts:
│  docs
│
└  Done! Now run pnpm run docs:dev and start writing.
```

{% <tip kind="tip" title="Vue as Peer Dependency" no_title={false}> %}
If you intend to perform customization that uses Vue components or APIs, you should also explicitly install `vue` as a dependency.
{% </tip> %}

## File Structure

If you are building a standalone VitePress site, you can scaffold the site in your current directory (`./`). However, if you are installing VitePress in an existing project alongside other source code, it is recommended to scaffold the site in a nested directory (e.g. `./docs`) so that it is separate from the rest of the project.

Assuming you chose to scaffold the VitePress project in `./docs`, the generated file structure should look like this:

```
.
├─ docs
│  ├─ .vitepress
│  │  └─ config.js
│  ├─ api-examples.md
│  ├─ markdown-examples.md
│  └─ index.md
└─ package.json
```

The `docs` directory is considered the **project root** of the VitePress site. The `.vitepress` directory is a reserved location for VitePress' config file, dev server cache, build output, and optional theme customization code.

{% <tip kind="tip" title="" no_title={false}> %}
By default, VitePress stores its dev server cache in `.vitepress/cache`, and the production build output in `.vitepress/dist`. If using Git, you should add them to your `.gitignore` file. These locations can also be [configured](/reference/site-config/#outdir).
{% </tip> %}

### The Config File

The config file (`.vitepress/config.js`) allows you to customize various aspects of your VitePress site, with the most basic options being the title and description of the site:

```js,name=.vitepress/config.js
export default {
  // site-level options
  title: 'VitePress',
  description: 'Just playing around.',

  themeConfig: {
    // theme-level options
  }
}
```

You can also configure the behavior of the theme via the `themeConfig` option. Consult the [Config Reference](/reference/site-config/) for full details on all config options.

### Source Files

Markdown files outside the `.vitepress` directory are considered **source files**.

VitePress uses **file-based routing**: each `.md` file is compiled into a corresponding `.html` file with the same path. For example, `index.md` will be compiled into `index.html`, and can be visited at the root path `/` of the resulting VitePress site.

VitePress also provides the ability to generate clean URLs, rewrite paths, and dynamically generate pages. These will be covered in the [Routing Guide](/guide/introduction/routing/).

## Up and Running

The tool should have also injected the following npm scripts to your `package.json` if you allowed it to do so during the setup process:

```json,name=package.json
{
  ...
  "scripts": {
    "docs:dev": "vitepress dev docs",
    "docs:build": "vitepress build docs",
    "docs:preview": "vitepress preview docs"
  },
  ...
}
```

The `docs:dev` script will start a local dev server with instant hot updates. Run it with the following command:

{% <codegroup> %}

```sh,name=npm
$ npm run docs:dev
```

```sh,name=pnpm
$ pnpm run docs:dev
```

```sh,name=yarn
$ yarn docs:dev
```

```sh,name=bun
$ bun run docs:dev
```

{% </codegroup> %}

Instead of npm scripts, you can also invoke VitePress directly with:

{% <codegroup> %}

```sh,name=npm
$ npx vitepress dev docs
```

```sh,name=pnpm
$ pnpm vitepress dev docs
```

```sh,name=yarn
$ yarn vitepress dev docs
```

```sh,name=bun
$ bun vitepress dev docs
```

{% </codegroup> %}

More command line usage is documented in the [CLI Reference](/reference/cli/).

The dev server should be running at `http://localhost:5173`. Visit the URL in your browser to see your new site in action!

## What's Next?

- To better understand how markdown files are mapped to generated HTML, proceed to the [Routing Guide](/guide/introduction/routing/).

- To discover more about what you can do on the page, such as writing markdown content or using Vue Components, refer to the "Writing" section of the guide. A great place to start would be to learn about [Markdown Extensions](/guide/writing/markdown/).

- To explore the features provided by the default documentation theme, check out the [Default Theme Config Reference](/reference/default-theme/config/).

- If you want to further customize the appearance of your site, explore how to either [Extend the Default Theme](/guide/customization/extending-default-theme/) or [Build a Custom Theme](/guide/customization/custom-theme/).

- Once your documentation site takes shape, make sure to read the [Deployment Guide](/guide/introduction/deploy/).
