// Generate docs/static/themes.json — the index of bundled themes the
// docs site's live theme switcher (/js/theme-demo.js) renders its
// buttons from. Run automatically by `npm run build:docs` so the
// gallery can never drift from static/themes/.
import { readdirSync, writeFileSync } from "node:fs";

const themes = readdirSync(new URL("../static/themes", import.meta.url))
  .filter((f) => f.endsWith(".css"))
  .map((f) => f.replace(/\.css$/, ""))
  .sort();

const out = new URL("../docs/static/themes.json", import.meta.url);
writeFileSync(out, JSON.stringify(themes, null, 2) + "\n");
console.log(`theme index: ${themes.join(", ")} → docs/static/themes.json`);
