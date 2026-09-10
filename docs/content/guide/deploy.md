---
outline: deep
description: Deploy your rustpress site to GitHub Pages, Netlify, Vercel, Cloudflare Pages, nginx or any static host.
---

# Deploy Your rustpress Site

The following guides are based on some shared assumptions:

- The site is inside the `docs` directory of your project.
- You are using the default build output directory, `docs/public`.
- The `rustpress` binary is built from a checkout, so the npm step that produces the theme assets runs before `cargo build` (see [Getting Started](./getting-started#build-from-source)). A binary installed from crates.io already has them embedded.

## Build and Test Locally

1. Build the docs:

   ```sh
   $ rustpress build docs
   rustpress: 27 pages + 404 + syntax.css + static/ → docs/public (0.1s)
   ```

2. Preview locally with the dev server, which serves `docs/public` at `http://127.0.0.1:4173`:

   ```sh
   $ rustpress serve docs
   ```

   The server binds to localhost only. Change the port with `--port`:

   ```sh
   $ rustpress serve docs --port 8080
   ```

The build fails on [dead links](./routing#dead-links) and on unknown config keys, so a green build is already a basic smoke test.

## Setting a Public Base Path

By default, we assume the site is going to be deployed at the root path of a domain (`/`). If your site is going to be served at a sub-path, e.g. `https://mywebsite.com/blog/`, set the [`base`](../reference/site-config#base) option to `"/blog/"`. It must start and end with `/`.

The base is applied to the theme chrome, to generated assets and to links and images inside markdown, so pages written for the root keep working under the sub-path. See [Asset Handling](./asset-handling#base-url) for the two things that are emitted verbatim.

## HTTP Cache Headers

rustpress does not hash file names, so there is nothing that can be cached "immutably" by URL. `main.css`, `js/app.js` and `syntax.css` change their content in place; serve them with a validating cache policy (`Cache-Control: no-cache` plus ETag or Last-Modified, which every static host does by default) rather than a long `max-age`. The font files under `/fonts/` never change and can be cached for a year.

## Platform Guides

### Netlify / Vercel / Cloudflare Pages / Render {#generic}

These platforms build in a Node image, so the build command has to install Rust first. Set up a new project and change these settings using your dashboard:

- **Build Command:** `curl -sSf https://sh.rustup.rs | sh -s -- -y && . "$HOME/.cargo/env" && cargo build --release && npm ci && npm run build:js && npm run build:css && ./target/release/rustpress build docs`
- **Output Directory:** `docs/public`

Building the binary on every deploy takes a few minutes. The alternative is to build once in CI (below), commit or publish `docs/public`, and point the platform at it.

### GitHub Pages

1. Create a file named `deploy.yml` inside the `.github/workflows` directory of your project:

   ```yaml [.github/workflows/deploy.yml]
   name: Deploy rustpress site to Pages

   on:
     push:
       branches: [main]
     workflow_dispatch:

   permissions:
     contents: read
     pages: write
     id-token: write

   concurrency:
     group: pages
     cancel-in-progress: false

   jobs:
     build:
       runs-on: ubuntu-latest
       steps:
         - name: Checkout
           uses: actions/checkout@v5
           with:
             fetch-depth: 0 # Not needed if lastUpdated is not enabled
         - name: Setup Rust
           uses: dtolnay/rust-toolchain@stable
         - name: Cache cargo
           uses: Swatinem/rust-cache@v2
         - name: Setup Node
           uses: actions/setup-node@v6
           with:
             node-version: 24
             cache: npm
         - name: Build theme assets
           run: npm ci && npm run build:js && npm run build:css
         - name: Build with rustpress
           run: cargo run --release -- build docs
         - name: Setup Pages
           uses: actions/configure-pages@v4
         - name: Upload artifact
           uses: actions/upload-pages-artifact@v3
           with:
             path: docs/public

     deploy:
       environment:
         name: github-pages
         url: ${{ steps.deployment.outputs.page_url }}
       needs: build
       runs-on: ubuntu-latest
       name: Deploy
       steps:
         - name: Deploy to GitHub Pages
           id: deployment
           uses: actions/deploy-pages@v4
   ```

   This assumes the site lives inside the rustpress repository (as this documentation does). For a site in its own repository, build rustpress as a separate step: check out the rustpress repository, run its npm build, then `cargo install --path .` (or, once the crate is published, `cargo install rustpress-cli`). The binary carries the theme assets, so nothing has to be copied into the site. Note that `cargo install --git` cannot work: the npm-built `main.css` and `js/app.js` are not committed, and the build refuses to run without them.

   ::: warning
   Make sure the `base` option is properly configured when deploying to a `<user>.github.io/<repository>/` project page — see [Setting a Public Base Path](#setting-a-public-base-path).
   :::

2. In your repository's settings under "Pages", select "GitHub Actions" in "Build and deployment > Source".

3. Push your changes to the `main` branch and wait for the workflow to complete.

### GitLab Pages

1. Create a file named `.gitlab-ci.yml` in the root of your project. GitLab Pages serves the `public` directory of the artifact, so the build output has to be moved there:

   ```yaml [.gitlab-ci.yml]
   image: rust:1
   pages:
     script:
       - apt-get update && apt-get install -y nodejs npm
       - npm ci && npm run build:js && npm run build:css
       - cargo build --release
       - ./target/release/rustpress build docs
       - rm -rf public && mv docs/public public
     artifacts:
       paths:
         - public
     only:
       - main
   ```

2. Set `base` to `"/<repository>/"` if you deploy to `https://<username>.gitlab.io/<repository>/`, or leave it at `/` for a custom domain or a unique-domain setting.

### nginx

Here is an example of an nginx server block. Because every rustpress URL is a directory, `try_files $uri $uri/ =404` is all the routing that is needed:

```nginx
server {
    listen 8080;
    listen [::]:8080;
    server_name _;

    root /usr/share/nginx/html;
    index index.html;
    charset utf-8;
    server_tokens off;

    gzip on;
    gzip_vary on;
    gzip_comp_level 5;
    gzip_min_length 1024;
    gzip_types
        application/javascript
        application/json
        image/svg+xml
        text/css
        text/javascript
        text/plain;

    location /fonts/ {
        add_header Cache-Control "public, max-age=31536000, immutable" always;
    }

    location / {
        add_header Cache-Control "no-cache" always;
        try_files $uri $uri/ =404;
    }

    error_page 404 /404.html;
}
```

### Any static host

`docs/public` is a self-contained directory of HTML, CSS, JavaScript, fonts and a `404.html`. Upload it as-is with `rsync`, `scp`, `surge`, an S3 bucket behind a CDN, or whatever your host offers — no server-side rewrites are required.
