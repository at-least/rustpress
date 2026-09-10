//! Built-in UI palettes for the `[theme]` section of rustpress.toml.
//!
//! Each palette re-points the brand aliases (`--vp-c-brand-*`) at one of
//! the palette ramps already defined in styles/vitepress.css (or, for
//! `mono`, at literal neutrals) in both `:root` and `.dark` — the ramp
//! values themselves are mode-aware, so dark mode resolves automatically.

pub const DEFAULT_NAME: &str = "vitepress";

pub struct Palette {
    pub name: &'static str,
    pub css: &'static str,
}

pub const PALETTES: &[Palette] = &[
    Palette {
        name: "green",
        css: r#"
:root, .dark {
  --vp-c-brand-1: var(--vp-c-green-1);
  --vp-c-brand-2: var(--vp-c-green-2);
  --vp-c-brand-3: var(--vp-c-green-3);
  --vp-c-brand-soft: var(--vp-c-green-soft);
}
"#,
    },
    Palette {
        name: "purple",
        css: r#"
:root, .dark {
  --vp-c-brand-1: var(--vp-c-purple-1);
  --vp-c-brand-2: var(--vp-c-purple-2);
  --vp-c-brand-3: var(--vp-c-purple-3);
  --vp-c-brand-soft: var(--vp-c-purple-soft);
}
"#,
    },
    Palette {
        name: "orange",
        css: r#"
:root, .dark {
  --vp-c-brand-1: var(--vp-c-orange-1);
  --vp-c-brand-2: var(--vp-c-orange-2);
  --vp-c-brand-3: var(--vp-c-orange-3);
  --vp-c-brand-soft: var(--vp-c-orange-soft);
}
"#,
    },
    Palette {
        name: "mono",
        css: r#"
:root {
  --vp-c-brand-1: #242424;
  --vp-c-brand-2: #4a4a4a;
  --vp-c-brand-3: #6f6f6f;
  --vp-c-brand-soft: rgba(128, 128, 128, 0.14);
}

.dark {
  --vp-c-brand-1: #e8e8e8;
  --vp-c-brand-2: #c7c7c7;
  --vp-c-brand-3: #9a9a9a;
  --vp-c-brand-soft: rgba(160, 160, 160, 0.16);
}
"#,
    },
];

/// The override CSS for a built-in palette name.
pub fn css(name: &str) -> Option<&'static str> {
    PALETTES.iter().find(|p| p.name == name).map(|p| p.css)
}

/// Human-readable list for error messages.
pub fn available() -> String {
    let mut names: Vec<String> = PALETTES.iter().map(|p| p.name.to_string()).collect();
    names.insert(0, format!("{DEFAULT_NAME} (default)"));
    names.join(", ")
}
