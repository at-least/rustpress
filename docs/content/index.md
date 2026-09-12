---
description: rustpress is a standalone documentation site generator written in Rust that reads VitePress's content format and needs no Node runtime to build.
layout: home

hero:
  name: rustpress
  text: VitePress-format docs from one Rust binary
  tagline: Write the markdown you already know. Build it without Node.
  actions:
    - theme: brand
      text: What is rustpress?
      link: ./guide/what-is-rustpress
    - theme: alt
      text: Quickstart
      link: ./guide/getting-started
    - theme: alt
      text: GitHub
      link: https://github.com/at-least/rustpress
  image:
    src: /logo.svg
    alt: rustpress

features:
  - icon: 📝
    title: Focus on your content
    details: "YAML front matter, ::: containers, code groups, line highlighting, snippets, includes, GitHub alerts — the VitePress markdown dialect, rendered by comrak and tree-sitter."
  - icon: 🦀
    title: One static binary
    details: "No Node at render time. The whole default theme is compiled in; a build is a single command that finishes in well under a second for a typical site."
  - icon: 🎨
    title: The VitePress default theme
    details: Navbar, sidebar, outline, local search, dark mode, home hero and features, 404 — the same look, the same CSS variables, the same themeConfig keys in TOML.
  - icon: 🖌️
    title: Complete built-in themes
    details: "GitHub Primer, Catppuccin, Nord, Rosé Pine — full designs, light and dark, held to completeness and WCAG-contrast tests. Try them live in the Theme gallery."
    link: /themes/
  - icon: 🔁
    title: Drop-in for existing docs
    details: Point rustpress at a VitePress docs folder. Pages, links and front matter work as they are; the config moves from TypeScript to rustpress.toml.
---
