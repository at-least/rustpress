#!/usr/bin/env node
// Design-token parity gate (static axis, no browser).
//
// The viewport axis guards geometry; this axis guards the token LAYER:
// every `--vp-*` custom property the compiled stylesheet applies at
// :root and .dark, compared against the pinned upstream declarations.
// It catches the drift classes that produce no structural signal — a
// rem/em swap, a recolored border token, a dropped dark override — in
// reviewable-text form instead of a screenshot.
//
// Two scopes, because upstream itself is layered the same way:
//
//   stock : upstream theme defaults = theme-default/styles/vars.css as
//           overridden by fonts.css (the default theme always ships it)
//           — compared against OUR base stylesheet (static/vitepress.css)
//   demo  : what vitepress.dev actually serves = the built docs site
//           bundle (theme + docs' own custom.css) — compared against our
//           demo site net of base + demo/public/themes/theme.css
//
// Only top-level (non-@media) `:root`/`.dark` blocks of exactly that
// selector participate. Values compare whitespace-insensitively.
//
//   node scripts/parity-tokens.mjs            # check against goldens
//   node scripts/parity-tokens.mjs --update   # re-record goldens from the
//                                             # upstream clone at the pin
//
// Reviewed, intentional differences live in parity/token-deltas.json
// (same shape as known-deltas.json: scope+token+values+reason).
//
// --update needs the vuejs/vitepress clone (bash scripts/sync-upstream.sh)
// at the pinned ref and its docs build (pnpm docs:build); the CHECK path
// only reads committed files + the compiled static/vitepress.css.

import { readFileSync, existsSync, readdirSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';

const ROOT = new URL('..', import.meta.url).pathname;
const BASE_CSS = join(ROOT, 'static/vitepress.css');
const DEMO_THEME = join(ROOT, 'demo/public/themes/theme.css');
const GOLDENS = join(ROOT, 'parity/token-goldens.json');
const DELTAS = join(ROOT, 'parity/token-deltas.json');
// the pinned upstream clone; overridable for sandboxed runs/tests
const UP = resolve(process.env.RUSTPRESS_VITEPRESS_CLONE ?? new URL('../../vitepress', import.meta.url).pathname);

// --- tiny flat-CSS reader ------------------------------------------------
function parseTopLevelBlocks(cssPath) {
  if (!existsSync(cssPath)) return [];
  let css = readFileSync(cssPath, 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
  const blocks = [];
  let i = 0;
  const n = css.length;
  while (i < n) {
    const open = css.indexOf('{', i);
    if (open < 0) break;
    const selector = css.slice(i, open).trim();
    // top-level only: @-rules nest; a selector line starting with '@' is media/keyframes/font-face
    if (selector.slice(css.lastIndexOf('}', i)).includes('@')) {
      // skip whole at-block
      let depth = 1, j = open + 1;
      while (j < n && depth > 0) {
        if (css[j] === '{') depth++;
        else if (css[j] === '}') depth--;
        j++;
      }
      i = j;
      continue;
    }
    let depth = 1, j = open + 1;
    while (j < n && depth > 0) {
      if (css[j] === '{') depth++;
      else if (css[j] === '}') depth--;
      j++;
    }
    const body = css.slice(open + 1, j - 1);
    blocks.push({ selector: selector.split(/\s+/).join(' '), body });
    i = j;
  }
  return blocks;
}

function tokensOf(cssPaths) {
  const out = { ':root': {}, '.dark': {} };
  for (const cssPath of cssPaths) {
    for (const { selector, body } of parseTopLevelBlocks(cssPath)) {
      const scope = selector === ':root' ? ':root' : selector === '.dark' ? '.dark' : null;
      if (!scope) continue;
      for (const decl of body.split(';')) {
        const idx = decl.indexOf(':');
        if (idx < 0) continue;
        const prop = decl.slice(0, idx).trim();
        const value = decl.slice(idx + 1).trim();
        if (prop.startsWith('--vp-')) out[scope][prop] = value.replace(/\s+/g, ' ').trim();
      }
    }
  }
  return out;
}

// Semantic normalization before compare: CSS has several spellings per
// value. Colors become a canonical rgba(r,g,b,a) (hex, hex8, rgba());
// numbers lose the leading zero (.875em ≡ 0.875em); string quotes
// collapse ('Inter' ≡ "Inter").
function canonColor(v) {
  let m = v.match(/^#([0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$/i);
  if (m) {
    let h = m[1];
    if (h.length === 3 || h.length === 4) h = [...h].map((c) => c + c).join('');
    const r = parseInt(h.slice(0, 2), 16);
    const g = parseInt(h.slice(2, 4), 16);
    const b = parseInt(h.slice(4, 6), 16);
    const a = h.length === 8 ? parseInt(h.slice(6, 8), 16) : 255;
    return `rgba(${r},${g},${b},${a})`;
  }
  m = v.match(/^rgba?\(([^)]*)\)$/i);
  if (m) {
    const parts = m[1].split(/[,\s/]+/).filter(Boolean);
    if (parts.length >= 3) {
      // alpha quantizes to 8-bit: rgba(…,0.14) and #…24 both mean a=36
      const a = parts[3] === undefined ? 255 : Math.round(Number(parts[3]) * 255);
      return `rgba(${Number(parts[0])},${Number(parts[1])},${Number(parts[2])},${a})`;
    }
  }
  return null;
}

function norm(v) {
  let s = (v ?? '').replace(/\s+/g, '').replace(/["']/g, '').toLowerCase();
  const color = canonColor(s);
  if (color) return color;
  // 0.875 ≡ .875, also inside var() etc.
  s = s.replace(/(^|[^\d.])0+(\.\d+)/g, '$1$2');
  // CSS color functions inside longer values (shadows): canonicalize each
  s = s.replace(/#[0-9a-f]{3,8}|rgba?\([^)]*\)/gi, (m) => canonColor(m.replace(/\s+/g, '').toLowerCase()) ?? m);
  return s;
}

function compareScope(scope, goldenMap, oursMap, deltas) {
  const problems = [];
  const keys = new Set([...Object.keys(goldenMap), ...Object.keys(oursMap)]);
  for (const token of [...keys].sort()) {
    const up = goldenMap[token];
    const ours = oursMap[token];
    if (norm(up) === norm(ours)) continue;
    const covered = deltas.some(
      (d) =>
        d.scope === scope &&
        d.token === token &&
        (d.upstream === undefined || norm(d.upstream) === norm(up)) &&
        (d.ours === undefined || norm(d.ours) === norm(ours)) &&
        d.reason,
    );
    if (!covered) {
      problems.push(`  [${scope}] ${token}: upstream=${JSON.stringify(up)} ours=${JSON.stringify(ours)}`);
    }
  }
  return problems;
}

// --- --update: regenerate golden from the pinned upstream clone ---------
if (process.argv.includes('--update')) {
  const varsCss = join(UP, 'src/client/theme-default/styles/vars.css');
  const fontsCss = join(UP, 'src/client/theme-default/styles/fonts.css');
  // the demo (deployed-truth) scope: prefer a local upstream docs build;
  // fall back to the stylesheets parity-refresh cached from vitepress.dev
  // (the drift-watch CI path never builds the clone)
  const distAssets = join(UP, 'docs/.vitepress/dist/assets');
  const cacheCssDir = join(ROOT, 'parity/cache/css');
  let demoCss;
  if (existsSync(distAssets) && readdirSync(distAssets).some((f) => f.endsWith('.css'))) {
    demoCss = readdirSync(distAssets)
      .filter((f) => f.endsWith('.css'))
      .map((f) => join(distAssets, f));
  } else if (existsSync(cacheCssDir) && readdirSync(cacheCssDir).length) {
    demoCss = readdirSync(cacheCssDir).map((f) => join(cacheCssDir, f));
  } else {
    console.error('parity-tokens: no upstream demo css — build the clone (pnpm docs:build) or run npm run parity:refresh first');
    process.exit(1);
  }
  if (!existsSync(varsCss)) {
    console.error('parity-tokens: upstream source missing: ' + varsCss + ' — sync the upstream clone: bash scripts/sync-upstream.sh');
    process.exit(1);
  }
  const distCss = demoCss;
  const stock = tokensOf([varsCss, fontsCss]);
  const demo = tokensOf(distCss);
  // no timestamp in the golden: regeneration must be a no-op diff when
  // upstream did not move (git history is the provenance)
  const golden = {
    _meta: {
      generated_from: 'vuejs/vitepress at parity/upstream-ref.txt (src vars.css+fonts.css; docs dist assets or parity/cache/css)',
    },
    scopes: {
      stock,
      demo,
    },
  };
  await writeFile(GOLDENS, JSON.stringify(golden, null, 2) + '\n');
  const count = (m) => Object.keys(m[':root']).length + Object.keys(m['.dark']).length;
  console.log(`parity-tokens: goldens written — stock ${count(stock)} + demo ${count(demo)} declarations`);
  process.exit(0);
}

// --- check ---------------------------------------------------------------
if (!existsSync(BASE_CSS)) {
  console.error('parity-tokens: static/vitepress.css missing — run npm run build:css first');
  process.exit(1);
}
if (!existsSync(GOLDENS)) {
  console.error('parity-tokens: parity/token-goldens.json missing — run node scripts/parity-tokens.mjs --update (needs the upstream clone)');
  process.exit(1);
}
if (!existsSync(DEMO_THEME)) {
  console.error('parity-tokens: demo/public/themes/theme.css missing — run npm run check:demo first');
  process.exit(1);
}
const goldens = JSON.parse(readFileSync(GOLDENS, 'utf8')).scopes;
const deltas = existsSync(DELTAS)
  ? JSON.parse(readFileSync(DELTAS, 'utf8')).deltas ?? []
  : [];
if (deltas.some((d) => !d.reason)) {
  console.error('parity-tokens: every entry in parity/token-deltas.json needs a reason (reviewed, not mute)');
  process.exit(1);
}

const oursBase = tokensOf([BASE_CSS]);
const oursDemo = tokensOf([BASE_CSS, DEMO_THEME]); // theme.css cascades last

const problems = [];
for (const scope of [':root', '.dark']) {
  problems.push(...compareScope(scope, goldens.stock[scope] ?? {}, oursBase[scope], deltas.filter((d) => d.layer === 'stock')));
  problems.push(...compareScope(scope, goldens.demo[scope] ?? {}, oursDemo[scope], deltas.filter((d) => d.layer === 'demo')));
}
// tokens upstream never defines must not leak into ours
for (const scope of [':root', '.dark']) {
  for (const token of Object.keys(oursBase[scope])) {
    if (!(token in (goldens.stock[scope] ?? {}))) {
      const covered = deltas.some((d) => d.layer === 'stock' && d.scope === scope && d.token === token && d.upstream === null && d.reason);
      if (!covered) problems.push(`  [${scope}] ${token}: only in ours (${JSON.stringify(oursBase[scope][token])})`);
    }
  }
}

if (problems.length) {
  console.error(`parity-tokens: ${problems.length} token divergence(s) from the pinned upstream:`);
  for (const p of problems) console.error(p);
  console.error('fix, or review into parity/token-deltas.json (see PARITY.md)');
  process.exit(1);
}
console.log(`parity-tokens: stock + demo declarations match the pinned upstream (${Object.keys(goldens.stock[':root']).length}+${Object.keys(goldens.stock['.dark']).length} stock tokens)`);
