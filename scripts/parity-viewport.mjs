#!/usr/bin/env node
// Viewport-parity regression gate.
//
// The landmark parity check (cargo parity) covers content and
// structure, but nothing guarded the *breakpoint behavior*: a Tailwind
// refactor can silently change a leading constant or a utility variant
// and shift layout at some widths while every test stays green (this
// exact class of drift was found by the 2026-09 full-breakpoint audit:
// a last-updated line-height and rem-vs-px breakpoints).
//
// Two axes, both against goldens verified pixel-for-pixel against a
// local build of upstream VitePress v2.0.0-alpha.20 (see
// parity/upstream-ref.txt):
//
//   1. static  — the compiled CSS must contain exactly the expected px
//                media queries; any rem breakpoint or unknown query is
//                a failure.
//   2. dynamic — a static file server + headless Chromium measure the
//                demo build at every breakpoint boundary (both sides of
//                640/768/960/1280/1440) against
//                parity/viewport-goldens.json.
//
// Some goldens encode demo content (feature count decides grid columns,
// title length decides wrap): after legitimately editing demo/content,
// refresh with `node scripts/parity-viewport.mjs --update` and eyeball
// the diff before committing.
//
// Skipping: without a usable playwright-chromium install the dynamic
// axis FAILS by default (repo precedent, tests/common/mod.rs); set
// RUSTPRESS_ALLOW_NO_PLAYWRIGHT=1 to turn that into a visible skip.

import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join, extname, resolve, sep } from 'node:path';

const ROOT = new URL('..', import.meta.url).pathname;
const DIST = join(ROOT, 'demo/public');
const GOLDENS = join(ROOT, 'parity/viewport-goldens.json');

const CSS_PATH = join(ROOT, 'static/vitepress.css');
if (!existsSync(CSS_PATH)) {
  console.error('parity-viewport: static/vitepress.css missing — run `npm run build:css` first');
  process.exit(1);
}
const CSS = await readFile(CSS_PATH, 'utf8');
let failures = 0;
const fail = (msg) => { failures++; console.error('FAIL ' + msg); };

// ---------- axis 1: media queries ----------
{
  const found = [...CSS.matchAll(/@media[^{]*/g)].map((m) => m[0].trim().replace(/\s+/g, ' '));
  const counts = new Map();
  for (const q of found) counts.set(q, (counts.get(q) || 0) + 1);
  const expected = {
    '@media (min-width:640px)': 5,
    '@media (min-width:768px)': 3,
    '@media (min-width:960px)': 8,
    '@media (min-width:1280px)': 3,
    '@media (min-width:1440px)': 2,
    '@media not all and (min-width:768px)': 2,
    '@media not all and (min-width:960px)': 3,
    '@media (hover:hover)': 4,
    '@media (prefers-reduced-motion:reduce)': 1,
  };
  for (const [q, want] of Object.entries(expected)) {
    const got = counts.get(q) || 0;
    if (got !== want) fail(`media query ${q}: expected exactly ${want}, found ${got}`);
    counts.delete(q);
  }
  for (const [q, n] of counts) {
    if (/\d+rem/.test(q)) fail(`rem-based breakpoint (upstream uses px): ${q} x${n}`);
    else fail(`unexpected media query (update the expected table if intentional): ${q} x${n}`);
  }
  console.log(`axis 1 (media queries): ${failures ? 'FAIL' : 'ok'}`);
}

// ---------- axis 2: geometry ----------
let chromium = null;
let skip = false;
try {
  ({ chromium } = await import('playwright-chromium'));
} catch (e) {
  const msg = `parity-viewport: playwright-chromium unusable (${e.message}) — dynamic axis SKIPPED`;
  if (process.env.RUSTPRESS_ALLOW_NO_PLAYWRIGHT === '1') {
    console.log(msg);
    skip = true;
  } else {
    console.error(msg + '\n  (RUSTPRESS_ALLOW_NO_PLAYWRIGHT=1 turns this into a visible skip)');
    process.exit(1);
  }
}

const CASES = [
  // (label, path, width, page-expression returning a string)
  // expressions run in the page; keep them structural (ids, tags,
  // .VPFeature etc.) — the demo markup has no upstream semantic classes
  // for hero/aside, so probes use the shared h1.heading shape.
  // doc page: content column position at every band
  ['doc h1 @390', '/guide/what-is-vitepress/', 390, `(() => { const r = document.querySelector('h1').getBoundingClientRect(); return [Math.round(r.left), Math.round(r.width), Math.round(r.height)].join(','); })()`],
  ['doc h1 @640', '/guide/what-is-vitepress/', 640, `(() => { const r = document.querySelector('h1').getBoundingClientRect(); return [Math.round(r.left), Math.round(r.width), Math.round(r.height)].join(','); })()`],
  ['doc h1 @767', '/guide/what-is-vitepress/', 767, `(() => { const r = document.querySelector('h1').getBoundingClientRect(); return [Math.round(r.left), Math.round(r.width), Math.round(r.height)].join(','); })()`],
  ['doc h1 @768', '/guide/what-is-vitepress/', 768, `(() => { const r = document.querySelector('h1').getBoundingClientRect(); return [Math.round(r.left), Math.round(r.width), Math.round(r.height)].join(','); })()`],
  ['doc h1 @960', '/guide/what-is-vitepress/', 960, `(() => { const r = document.querySelector('h1').getBoundingClientRect(); return [Math.round(r.left), Math.round(r.width), Math.round(r.height)].join(','); })()`],
  ['doc h1 @1920', '/guide/what-is-vitepress/', 1920, `(() => { const r = document.querySelector('h1').getBoundingClientRect(); return [Math.round(r.left), Math.round(r.width), Math.round(r.height)].join(','); })()`],
  // hamburger <768 only; sidebar drawer width <960, static 272 >=960
  ['hamburger @767', '/guide/what-is-vitepress/', 767, `(() => { const e = document.getElementById('VPNavBarHamburger'); return e && !!e.offsetParent ? 'visible' : 'hidden'; })()`],
  ['hamburger @768', '/guide/what-is-vitepress/', 768, `(() => { const e = document.getElementById('VPNavBarHamburger'); return e && !!e.offsetParent ? 'visible' : 'hidden'; })()`],
  ['sidebar @959', '/guide/what-is-vitepress/', 959, `document.getElementById('VPSidebar').getBoundingClientRect().width.toFixed(0)`],
  ['sidebar @960', '/guide/what-is-vitepress/', 960, `document.getElementById('VPSidebar').getBoundingClientRect().width.toFixed(0)`],
  // local nav: menu button <960, bar height steps 48 -> 52 at 960, gone at 1280
  ['menu btn @959', '/guide/what-is-vitepress/', 959, `(() => { const e = document.getElementById('VPLocalNavMenu'); return e && !!e.offsetParent ? 'visible' : 'hidden'; })()`],
  ['menu btn @960', '/guide/what-is-vitepress/', 960, `(() => { const e = document.getElementById('VPLocalNavMenu'); return e && !!e.offsetParent ? 'visible' : 'hidden'; })()`],
  ['localnav h @959', '/guide/what-is-vitepress/', 959, `document.getElementById('VPLocalNav').getBoundingClientRect().height.toFixed(0)`],
  ['localnav h @960', '/guide/what-is-vitepress/', 960, `document.getElementById('VPLocalNav').getBoundingClientRect().height.toFixed(0)`],
  ['localnav @1280', '/guide/what-is-vitepress/', 1280, `(() => { const e = document.getElementById('VPLocalNav'); return e && !!e.offsetParent ? 'visible' : 'hidden'; })()`],
  // right-hand outline aside sits at 1024 only from 1280
  ['aside @1279', '/guide/what-is-vitepress/', 1279, `(() => { const e = document.querySelector('.VPDocAsideOutline'); if (!e) return 'missing'; const r = e.getBoundingClientRect(); return Math.round(r.left) + ',' + Math.round(r.width); })()`],
  ['aside @1280', '/guide/what-is-vitepress/', 1280, `(() => { const e = document.querySelector('.VPDocAsideOutline'); if (!e) return 'missing'; const r = e.getBoundingClientRect(); return Math.round(r.left) + ',' + Math.round(r.width); })()`],
  // footer height steps at sm (the line-height regression this guards)
  ['footer h @639', '/guide/what-is-vitepress/', 639, `document.querySelector('#VPContent footer').getBoundingClientRect().height.toFixed(0)`],
  ['footer h @640', '/guide/what-is-vitepress/', 640, `document.querySelector('#VPContent footer').getBoundingClientRect().height.toFixed(0)`],
  // pager: stacked <640, side-by-side >=640
  ['pager @639', '/guide/markdown/', 639, `(() => { const f = document.querySelector('#VPContent footer'); const l = [...f.querySelectorAll('a')].filter(a => /Previous|Next/i.test(a.textContent)); if (l.length < 2) return 'links:' + l.length; return Math.abs(l[0].getBoundingClientRect().top - l[1].getBoundingClientRect().top) < 2 ? 'row' : 'stacked'; })()`],
  ['pager @640', '/guide/markdown/', 640, `(() => { const f = document.querySelector('#VPContent footer'); const l = [...f.querySelectorAll('a')].filter(a => /Previous|Next/i.test(a.textContent)); if (l.length < 2) return 'links:' + l.length; return Math.abs(l[0].getBoundingClientRect().top - l[1].getBoundingClientRect().top) < 2 ? 'row' : 'stacked'; })()`],
  // hero: name font steps 32/48/56 at sm/lg
  ['hero name fs @390', '/', 390, `getComputedStyle(document.querySelector('h1.heading').children[0]).fontSize`],
  ['hero name fs @640', '/', 640, `getComputedStyle(document.querySelector('h1.heading').children[0]).fontSize`],
  ['hero name fs @960', '/', 960, `getComputedStyle(document.querySelector('h1.heading').children[0]).fontSize`],
  // hero image container: below text <lg, right column at lg
  ['hero img top @959', '/', 959, `document.querySelector('[class*="order-1"]').getBoundingClientRect().top.toFixed(0)`],
  ['hero img left @960', '/', 960, `(() => { const r = document.querySelector('[class*="order-1"]').getBoundingClientRect(); return Math.round(r.left) + ',' + Math.round(r.top); })()`],
  // features: 1 col <640, 2 cols 640-959, 4 cols at 960
  ['feat w @390', '/', 390, `document.querySelector('.VPFeature').closest('li').getBoundingClientRect().width.toFixed(0)`],
  ['feat w @640', '/', 640, `document.querySelector('.VPFeature').closest('li').getBoundingClientRect().width.toFixed(0)`],
  ['feat w @960', '/', 960, `document.querySelector('.VPFeature').closest('li').getBoundingClientRect().width.toFixed(0)`],
];

if (skip) {
  console.log(`axis 2 (geometry): skipped (${CASES.length} cases unmeasured)`);
  process.exit(failures ? 1 : 0);
}
if (!existsSync(join(DIST, 'index.html'))) {
  console.error('parity-viewport: demo/public missing — run `cargo run -- build demo` first');
  process.exit(1);
}

const server = createServer(async (req, res) => {
  try {
    let p = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    if (p.endsWith('/')) p += 'index.html';
    let file = resolve(DIST, '.' + p);
    if (!file.startsWith(resolve(DIST) + sep)) throw new Error('outside dist');
    if (!existsSync(file)) file = join(DIST, p + '.html');
    if (!file.startsWith(resolve(DIST) + sep)) throw new Error('outside dist');
    const body = await readFile(file);
    const types = {
      '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript',
      '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png',
      '.webp': 'image/webp', '.woff2': 'font/woff2', '.woff': 'font/woff',
    };
    res.writeHead(200, { 'content-type': types[extname(file)] || 'application/octet-stream' });
    res.end(body);
  } catch {
    res.writeHead(404); res.end('not found');
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const PORT = server.address().port;

const UPDATE = process.argv.includes('--update');
let goldens = {};
if (!UPDATE) {
  if (!existsSync(GOLDENS)) {
    console.error(`parity-viewport: ${GOLDENS} missing — run with --update to record goldens`);
    process.exit(1);
  }
  goldens = JSON.parse(await readFile(GOLDENS, 'utf8'));
}

const browser = await chromium.launch({ headless: true });
let dynamicFail = 0;
const measured = {};
try {
  for (const [label, path, width, expr] of CASES) {
    const ctx = await browser.newContext({ viewport: { width, height: 900 } });
    const page = await ctx.newPage();
    try {
      await page.goto(`http://127.0.0.1:${PORT}${path}`, { waitUntil: 'networkidle' });
      await page.evaluate(() => document.fonts.ready);
      const actual = String(await page.evaluate(expr));
      measured[label] = actual;
      if (!UPDATE) {
        const want = goldens[label];
        if (want === undefined) fail(`${label}: no golden recorded (run --update)`);
        else if (want !== actual) { fail(`${label}: expected ${want}, got ${actual}`); dynamicFail++; }
        else console.log(`ok ${label} (${actual})`);
      }
    } catch (e) {
      fail(`${label}: probe error ${e.message}`); dynamicFail++;
    } finally { await ctx.close(); }
  }
} finally {
  await browser.close();
  server.close();
}

if (UPDATE) {
  const { writeFile } = await import('node:fs/promises');
  await writeFile(GOLDENS, JSON.stringify(measured, null, 2) + '\n');
  console.log(`axis 2 (geometry): ${CASES.length} goldens written to parity/viewport-goldens.json`);
} else {
  console.log(`axis 2 (geometry): ${CASES.length - dynamicFail}/${CASES.length} passed`);
}
process.exit(failures ? 1 : 0);
