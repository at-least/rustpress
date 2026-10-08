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
// Cases the demo corpus can't host (it is a verbatim upstream mirror —
// tests/upstream_sync.rs) run against the fixture site built on the fly
// from parity/probe-site/ and served under /probe/.
//
// Skipping: without a usable playwright-chromium install the dynamic
// axis FAILS by default (repo precedent, tests/common/mod.rs); set
// RUSTPRESS_ALLOW_NO_PLAYWRIGHT=1 to turn that into a visible skip.

import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join, extname, resolve, sep } from 'node:path';
import { execFileSync } from 'node:child_process';

const ROOT = new URL('..', import.meta.url).pathname;
const DIST = join(ROOT, 'demo/public');
const PROBE_SITE = join(ROOT, 'parity/probe-site');
const PROBE_DIST = join(PROBE_SITE, 'public');
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
    // counts verified against the compiled stylesheet; the hero blur
    // ladder is demo-site CSS (demo/theme.css), not part of the base
    '@media (min-width:640px)': 4,
    '@media (min-width:768px)': 3,
    '@media (min-width:960px)': 7,
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
  // doc-footer type: last-updated 500 (VPLastUpdated), outline links 400
  // (upstream .outline-link overrides the container's 500)
  ['last-updated fw @768', '/guide/what-is-vitepress/', 768, `getComputedStyle(document.querySelector('#VPContent footer .last-updated p')).fontWeight`],
  ['outline fw @1280', '/guide/what-is-vitepress/', 1280, `getComputedStyle(document.querySelector('.VPDocAsideOutline .outline-link')).fontWeight`],
  // navbar appearance switch: hidden <768 like upstream .VPNavBarAppearance
  ['appearance @767', '/guide/what-is-vitepress/', 767, `(() => { const e = document.getElementById('VPSwitchAppearance'); return e && !!e.offsetParent ? 'visible' : 'hidden'; })()`],
  ['appearance @768', '/guide/what-is-vitepress/', 768, `(() => { const e = document.getElementById('VPSwitchAppearance'); return e && !!e.offsetParent ? 'visible' : 'hidden'; })()`],
  // the 1px rule upstream's VPNavBar draws before the switch when the menu
  // precedes it (8px | rule | 16px): menu-end-to-switch gap + the rule.
  // Golden measured on the pinned build with its locale flyout and
  // Ask-AI button removed (the demo config has neither)
  ['appearance divider @1280', '/guide/what-is-vitepress/', 1280, `(() => { const sw = document.getElementById('VPSwitchAppearance'); const menu = document.querySelector('#VPNavBar nav'); const cs = getComputedStyle(sw.parentElement, '::before'); return [Math.round(sw.getBoundingClientRect().left - menu.getBoundingClientRect().right), cs.width, cs.marginLeft, cs.marginRight].join(','); })()`],
  // nav screen (hamburger opened in the probe): upstream's appearance
  // row is a labeled row on the soft surface (VPNavScreenAppearance), and
  // its social links center (VPSocialLinks) with rel="me noopener" — the
  // `me` is what Mastodon-style profile verification looks for
  ['screen appearance row @375', '/guide/what-is-vitepress/', 375, `(async () => { document.getElementById('VPNavBarHamburger').click(); await new Promise((r) => setTimeout(r, 400)); const row = document.getElementById('VPSwitchAppearanceScreen').parentElement; const cs = getComputedStyle(row); const l = getComputedStyle(row.firstElementChild); return [cs.justifyContent, cs.padding, cs.borderRadius, cs.backgroundColor, l.fontSize, l.color, l.lineHeight].join('|'); })()`],
  ['screen social @375', '/guide/what-is-vitepress/', 375, `(async () => { document.getElementById('VPNavBarHamburger').click(); await new Promise((r) => setTimeout(r, 400)); const c = document.getElementById('VPSwitchAppearanceScreen').parentElement.parentElement.getBoundingClientRect(); const a = document.querySelector('#VPNavScreen a[aria-label="github"]'); const g = a.getBoundingClientRect(); return Math.round(g.left + g.width / 2 - (c.left + c.width / 2)) + '|' + a.getAttribute('rel'); })()`],
  // code lines are inline (line-height parity): tallest pre is 24 lines
  ['code max-pre h @768', '/guide/markdown/', 768, `Math.max(...[...document.querySelectorAll('.vp-doc pre')].map(p => Math.round(p.getBoundingClientRect().height))).toString()`],
  // highlighted lines full-bleed: as wide as their pre
  ['code hl w @768', '/guide/markdown/', 768, `(() => { const e = document.querySelector('.vp-doc pre .line.hl'); return e ? Math.round(e.getBoundingClientRect().width).toString() : 'none'; })()`],
  // <<< includes with {n} keep every line ({n} highlights, not selects)
  ['include hl lines @768', '/guide/markdown/', 768, `(() => { const b = [...document.querySelectorAll('.vp-code-block-title')].find(x => x.querySelector('[data-title="snippet.js"]') && x.querySelector('.line.hl')); return b ? b.querySelectorAll('.line').length.toString() : 'missing'; })()`],
  // diff notation x line-numbers (fixture site under /probe/ — the demo
  // corpus is a verbatim upstream mirror and can't host the combo): the
  // number gutter owns ::before (an un-gated '-' rule once
  // out-specified the counter) and the removed line keeps its
  // background + 0.7 opacity (the rule was once lost in a refactor);
  // unnumbered blocks keep the +/- symbol
  ['diff ln gutter @768', '/probe/', 768, `getComputedStyle(document.querySelector('.vp-doc pre.line-numbers .line.diff.remove'), '::before').content`],
  ['diff ln bg @768', '/probe/', 768, `getComputedStyle(document.querySelector('.vp-doc pre.line-numbers .line.diff.remove')).backgroundColor`],
  ['diff ln opacity @768', '/probe/', 768, `getComputedStyle(document.querySelector('.vp-doc pre.line-numbers .line.diff.remove')).opacity`],
  ['diff remove sym @768', '/probe/', 768, `getComputedStyle(document.querySelector('.vp-doc pre:not(.line-numbers) .line.diff.remove'), '::before').content`],
  // pager: stacked <640, side-by-side >=640
  ['pager @639', '/guide/markdown/', 639, `(() => { const f = document.querySelector('#VPContent footer'); const l = [...f.querySelectorAll('a')].filter(a => /Previous|Next/i.test(a.textContent)); if (l.length < 2) return 'links:' + l.length; return Math.abs(l[0].getBoundingClientRect().top - l[1].getBoundingClientRect().top) < 2 ? 'row' : 'stacked'; })()`],
  ['pager @640', '/guide/markdown/', 640, `(() => { const f = document.querySelector('#VPContent footer'); const l = [...f.querySelectorAll('a')].filter(a => /Previous|Next/i.test(a.textContent)); if (l.length < 2) return 'links:' + l.length; return Math.abs(l[0].getBoundingClientRect().top - l[1].getBoundingClientRect().top) < 2 ? 'row' : 'stacked'; })()`],
  // hero: name font steps 32/48/56 at sm/lg
  ['hero name fs @390', '/', 390, `getComputedStyle(document.querySelector('h1.heading').children[0]).fontSize`],
  ['hero name fs @640', '/', 640, `getComputedStyle(document.querySelector('h1.heading').children[0]).fontSize`],
  ['hero name fs @960', '/', 960, `getComputedStyle(document.querySelector('h1.heading').children[0]).fontSize`],
  // hero image container: stacked above text <960 (negative-margin box),
  // right column at >=960. `.hero-image-box` is the counterpart hook of
  // upstream's `.VPHero .image` (a `[class*=...]` substring selector once
  // silently matched the text column via `lg:order-1`)
  ['hero img top @375', '/', 375, `document.querySelector('.hero-image-box').getBoundingClientRect().top.toFixed(0)`],
  ['hero img top @959', '/', 959, `document.querySelector('.hero-image-box').getBoundingClientRect().top.toFixed(0)`],
  ['hero img left @960', '/', 960, `(() => { const r = document.querySelector('.hero-image-box').getBoundingClientRect(); return Math.round(r.left) + ',' + Math.round(r.top); })()`],
  // features: 1 col <640, 2 cols 640-959, 4 cols at 960
  ['feat w @390', '/', 390, `document.querySelector('.VPFeature').closest('li').getBoundingClientRect().width.toFixed(0)`],
  ['feat w @640', '/', 640, `document.querySelector('.VPFeature').closest('li').getBoundingClientRect().width.toFixed(0)`],
  ['feat w @960', '/', 960, `document.querySelector('.VPFeature').closest('li').getBoundingClientRect().width.toFixed(0)`],
  // the local-nav outline dropdown lists the aside's headings, nested
  // like upstream's (VPDocOutlineItem without `root`: the top list padded
  // like the nested ones) — text offsets of "Return to top" + each link
  ['outline dropdown @375', '/guide/routing/', 375, `(async () => { document.getElementById('VPOutlineDropdownButton').click(); const box = document.getElementById('VPOutlineDropdownItems'); for (let i = 0; i < 100 && !box.offsetParent; i++) await new Promise((r) => setTimeout(r, 50)); const bb = box.getBoundingClientRect(); return [...box.querySelectorAll('a')].map((a) => { const g = document.createRange(); g.selectNodeContents(a); return Math.round(g.getBoundingClientRect().left - bb.left); }).join(','); })()`],
  // a page with neither outline headers nor a sidebar (fixture under
  // /probe/): upstream shows no local nav until the page has scrolled
  // past the navbar, then a fixed bar holding "Return to top"
  ['local nav without headers @375', '/probe/no-outline/', 375, `(async () => { const nav = document.getElementById('VPLocalNav'); const shown = () => getComputedStyle(nav).display !== 'none'; const top = shown(); window.scrollTo(0, 300); for (let i = 0; i < 100 && !shown(); i++) await new Promise((r) => setTimeout(r, 50)); const b = nav.getBoundingClientRect(); return [top, getComputedStyle(nav).position, Math.round(b.top), Math.round(b.height), nav.innerText.trim()].join('|'); })()`],
];

if (skip) {
  console.log(`axis 2 (geometry): skipped (${CASES.length} cases unmeasured)`);
  process.exit(failures ? 1 : 0);
}
if (!existsSync(join(DIST, 'index.html'))) {
  console.error('parity-viewport: demo/public missing — run `cargo run -- build demo` first');
  process.exit(1);
}
// rebuild the /probe/ fixture site so its computed-style cases always
// measure the current binary + stylesheet, never a stale public/
{
  const out = execFileSync('cargo', ['run', '--quiet', '--', 'build', PROBE_SITE], { cwd: ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] });
  process.stdout.write(out);
}
// stale-build guard: measuring an old build records or checks the
// wrong values. Sources compare by NEWEST FILE mtime — a directory's
// own mtime doesn't move when a file inside is edited (the same trap
// as index.html above, which the comment warned about)
{
  const { statSync, readdirSync } = await import('node:fs');
  const newest = (p) => {
    const st = statSync(p);
    if (st.isFile()) return st.mtimeMs;
    return readdirSync(p, { withFileTypes: true })
      .map((e) => newest(join(p, e.name)))
      .reduce((a, b) => Math.max(a, b), 0);
  };
  const distMtime = statSync(join(DIST, 'index.html')).mtimeMs;
  // vitecss by its two stylesheets, not its directory: node_modules/vitecss
  // links to the ../vitecss checkout, whose .git would make a walk see
  // every commit there as a source change
  const vitecss = ['index.css', 'fonts.css'].map((f) => join(ROOT, 'node_modules/vitecss', f));
  for (const src of [CSS_PATH, join(ROOT, 'styles/vitepress.css'), ...vitecss, join(ROOT, 'src/render')]) {
    if (existsSync(src) && newest(src) > distMtime) {
      console.error(`parity-viewport: demo/public is older than ${src} — rebuild first (npm run check:demo)`);
      process.exit(1);
    }
  }
}

const server = createServer(async (req, res) => {
  try {
    let p = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    let dist = DIST;
    if (p === '/probe' || p.startsWith('/probe/')) {
      dist = PROBE_DIST;
      p = p.slice('/probe'.length) || '/';
    }
    if (p.endsWith('/')) p += 'index.html';
    let file = resolve(dist, '.' + p);
    if (!file.startsWith(resolve(dist) + sep)) throw new Error('outside dist');
    if (!existsSync(file)) file = join(dist, p + '.html');
    if (!file.startsWith(resolve(dist) + sep)) throw new Error('outside dist');
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
