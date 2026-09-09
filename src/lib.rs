//! gen-docs: a standalone docs site generator that reads VitePress-format
//! content (YAML front matter, `:::` containers, labeled code fences) and
//! renders a static site in Rust — hypertext `rsx!` templates, Alpine.js
//! interactivity, Tailwind CSS styling.

pub mod config;
pub mod content;
pub mod markdown;
pub mod palettes;
pub mod render;
pub mod serve;
pub mod sidebar;
