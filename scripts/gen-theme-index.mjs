// Generate docs/static/themes.json — the index the docs site's Theme
// Gallery (/themes/) renders its cards from. For every bundled
// theme in static/themes/ it extracts, from the theme file itself:
//   name  — file stem
//   desc  — the header comment's one-line description
//   bg / text / brand — light-mode identity swatches
// Run automatically by `npm run build:docs` so the gallery can never
// drift from static/themes/.
import { readdirSync, readFileSync, writeFileSync } from "node:fs";

const dir = new URL("../static/themes/", import.meta.url);
const themes = readdirSync(dir)
  .filter((f) => f.endsWith(".css"))
  .sort()
  .map((file) => {
    const css = readFileSync(new URL(file, dir)).toString();
    // first block comment, line-wraps joined, then its first sentence
    const header = (css.match(/^\/\*[\s\S]*?\*\//)?.[0] ?? "")
      .replace(/^\/\*+/, "").replace(/\*+\/$/, "")
      .split("\n").map((l) => l.replace(/^\s*\*\s?/, "").trim()).join(" ")
      .replace(/\s+/g, " ");
    const desc = header.match(/— (.+?)(?:\.\s|$)/)?.[1] ?? "";
    const root = css.match(/:root\s*\{([\s\S]*?)\}/)?.[1] ?? "";
    const token = (name) =>
      root.match(new RegExp(`--vp-c-${name}:\\s*(#[0-9a-fA-F]{6})`))?.[1] ?? "";
    return {
      name: file.replace(/\.css$/, ""),
      desc,
      bg: token("bg"),
      text: token("text-1"),
      brand: token("brand-1"),
    };
  });

const out = new URL("../docs/static/themes.json", import.meta.url);
writeFileSync(out, JSON.stringify(themes, null, 2) + "\n");
console.log(
  `theme index: ${themes.map((t) => t.name).join(", ")} → docs/static/themes.json`,
);
