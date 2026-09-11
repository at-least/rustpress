/* rustpress theme gallery: paint any container with a built-in palette.
 *
 * Usage: <div x-data="themePreview('green')">…anything…</div>
 *
 * The palette definitions are inlined below (generated from rustpress's
 * palettes.rs at build time by build_theme_showcase.rs) so the gallery
 * works when served from any subpath — no fetch needed. Each preview
 * injects one <style> scoped to a generated class, so it is independent
 * of the page's own theme. Dark mode follows the site's .dark class
 * on <html>.
 */

/* START generated */
const PALETTES = {
  "green": `
:root, .dark {
  --vp-c-brand-1: var(--vp-c-green-1);
  --vp-c-brand-2: var(--vp-c-green-2);
  --vp-c-brand-3: var(--vp-c-green-3);
  --vp-c-brand-soft: var(--vp-c-green-soft);
}
`,
  "purple": `
:root, .dark {
  --vp-c-brand-1: var(--vp-c-purple-1);
  --vp-c-brand-2: var(--vp-c-purple-2);
  --vp-c-brand-3: var(--vp-c-purple-3);
  --vp-c-brand-soft: var(--vp-c-purple-soft);
}
`,
  "orange": `
:root, .dark {
  --vp-c-brand-1: var(--vp-c-orange-1);
  --vp-c-brand-2: var(--vp-c-orange-2);
  --vp-c-brand-3: var(--vp-c-orange-3);
  --vp-c-brand-soft: var(--vp-c-orange-soft);
}
`,
  "mono": `
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
`,
  "ocean": `
:root {
  --vp-c-brand-1: #0969da;
  --vp-c-brand-2: #218bff;
  --vp-c-brand-3: #0550ae;
  --vp-c-brand-soft: rgba(9, 105, 218, 0.12);
}

.dark {
  --vp-c-brand-1: #58a6ff;
  --vp-c-brand-2: #79b8ff;
  --vp-c-brand-3: #1f6feb;
  --vp-c-brand-soft: rgba(88, 166, 255, 0.14);
}
`,
  "sakura": `
:root {
  --vp-c-brand-1: #bf3989;
  --vp-c-brand-2: #d93b9d;
  --vp-c-brand-3: #a03182;
  --vp-c-brand-soft: rgba(236, 96, 174, 0.13);
}

.dark {
  --vp-c-brand-1: #f57fc0;
  --vp-c-brand-2: #ec60ae;
  --vp-c-brand-3: #c94596;
  --vp-c-brand-soft: rgba(245, 127, 192, 0.15);
}
`,
  "ember": `
:root, .dark {
  --vp-c-brand-1: var(--vp-c-red-1);
  --vp-c-brand-2: var(--vp-c-red-2);
  --vp-c-brand-3: var(--vp-c-red-3);
  --vp-c-brand-soft: var(--vp-c-red-soft);
}
`,
};
/* END generated */

document.addEventListener("alpine:init", () => {
  Alpine.data("themePreview", () => ({
    init() {
      const name = this.$el.getAttribute("x-data").match(/'([^']+)'/)?.[1];
      const css = name && PALETTES[name];
      if (!name || !css) {
        this.$el.dataset.missing = name || "?";
        return;
      }
      const cls = `rp-tp-${name.replace(/[^a-z0-9-]/gi, "")}`;
      this.$el.classList.add(cls);
      const style = document.createElement("style");
      style.textContent = css.replaceAll(":root", `.${cls}`).replaceAll(".dark", `.dark .${cls}`);
      document.head.appendChild(style);
    },
  }));
});
