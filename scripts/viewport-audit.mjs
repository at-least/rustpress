#!/usr/bin/env node
// Full-breakpoint visual/geometry audit: rustpress demo vs a local
// build of upstream VitePress (pinned ref, see parity/upstream-ref.txt).
//
// Unlike scripts/parity-viewport.mjs (a regression gate over a small
// golden set), this is a broad audit tool: for a matrix of pages ×
// breakpoint-boundary widths × states it dumps getBoundingClientRect +
// getComputedStyle for a mapped list of landmarks on BOTH sides and
// reports every property differing beyond tolerance. Screenshots for
// eyeballing land in parity/audit/ (gitignored).
//
//   node scripts/viewport-audit.mjs                 # full matrix, no screenshots
//   node scripts/viewport-audit.mjs --shots         # also save PNGs
//   node scripts/viewport-audit.mjs --pages home    # subset: home|doc|markdown|reference
//   node scripts/viewport-audit.mjs --widths 390,768
//   node scripts/viewport-audit.mjs --states light,dark
//
// Requires: demo/public (npm run check:demo) and the upstream docs
// build at ../vitepress/docs/.vitepress/dist (pnpm docs:build in the
// ../vitepress clone at the pinned ref). playwright-chromium needed.

import { createServer } from 'node:http';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join, extname, resolve, sep } from 'node:path';

const ROOT = new URL('..', import.meta.url).pathname;
const OURS_DIST = join(ROOT, 'demo/public');
const UP_DIST = resolve(ROOT, '../vitepress/docs/.vitepress/dist');
const AUDIT_DIR = join(ROOT, 'parity/audit');

for (const [label, dir, hint] of [
  ['ours', OURS_DIST, 'npm run check:demo'],
  ['upstream', UP_DIST, 'pnpm docs:build in ../vitepress at the pinned ref'],
]) {
  if (!existsSync(join(dir, 'index.html'))) {
    console.error(`viewport-audit: ${label} build missing at ${dir} — ${hint}`);
    process.exit(1);
  }
}
const gen = (html) => (html.match(/<meta name="generator" content="([^"]*)"/) || [])[1];
const upGen = gen(await readFile(join(UP_DIST, 'index.html'), 'utf8'));
console.log(`upstream build: ${upGen || 'generator meta NOT FOUND'}`);

const arg = (name, def) => {
  const i = process.argv.indexOf('--' + name);
  if (i < 0) return def;
  const v = process.argv[i + 1];
  return (!v || v.startsWith('--')) ? true : v;
};
const WANT_PAGES = arg('pages'); // comma list of page keys
const WANT_WIDTHS = arg('widths'); // comma list
const WANT_STATES = arg('states');
const SHOTS = !!arg('shots');

// Breakpoint boundaries + band middles. Both sides of every media query
// VitePress uses (640/768/960/1280/1440) plus phone/desktop extremes.
const ALL_WIDTHS = [375, 390, 639, 640, 767, 768, 959, 960, 1023, 1024, 1279, 1280, 1439, 1440, 1920];
const ALL_STATES = ['light', 'dark', 'scrolled', 'navopen'];
const ALL_PAGES = {
  home: '/',
  doc: '/guide/what-is-vitepress/',
  markdown: '/guide/markdown/',
  reference: '/reference/default-theme-config/',
};
const widths = (WANT_WIDTHS && WANT_WIDTHS !== true)
  ? WANT_WIDTHS.split(',').map(Number)
  : ALL_WIDTHS;
const states = (WANT_STATES && WANT_STATES !== true)
  ? WANT_STATES.split(',')
  : ALL_STATES;
const pageKeys = (WANT_PAGES && WANT_PAGES !== true)
  ? WANT_PAGES.split(',')
  : Object.keys(ALL_PAGES);

// Landmarks: [key, ours-selector, upstream-selector, pages…]
// Selectors are tried in order (first match wins). A landmark that
// exists on neither side is skipped for that page; exactly-one-side
// mismatches are reported.
const LANDMARKS = [
  // shared skeleton
  ['navbar', '#VPNavBar', '.VPNavBar', '*'],
  ['navbar-title', '#VPNavBar a', '.VPNavBarTitle a', '*'],
  ['navbar-menu-link', '#VPNavBar nav li a', '.VPNavBarMenuLink', '*'],
  ['sidebar', '#VPSidebar', '.VPSidebar', '*'],
  ['sidebar-item', '#VPSidebar .VPSidebarItem.level-1', '.VPSidebarItem.level-1', 'doc', 'markdown', 'reference'],
  ['localnav', '#VPLocalNav', '.VPLocalNav', '*'],
  ['content', '#VPContent', '#VPContent', '*'],
  ['doc-container', '.vp-doc', '.vp-doc', 'doc', 'markdown', 'reference'],
  ['doc-h1', '.vp-doc h1', '.vp-doc h1', 'doc', 'markdown', 'reference'],
  ['doc-h2', '.vp-doc h2', '.vp-doc h2', 'doc', 'markdown', 'reference'],
  ['doc-p', '.vp-doc > div > p, .vp-doc p', '.vp-doc p', 'doc', 'markdown', 'reference'],
  ['doc-inline-code', '.vp-doc p code', '.vp-doc p code', 'doc', 'markdown', 'reference'],
  // visual card = upstream's div[class*=language-] wrapper (our styled
  // pre carries margin/bg/radius directly); comparing pre-to-pre on both
  // sides once compared DIFFERENT roles (upstream's inner pre is bare)
  ['doc-code-pre', '.vp-doc pre', ".vp-doc div[class*='language-']", 'doc', 'markdown', 'reference'],
  ['doc-code-pre-code', '.vp-doc pre code', '.vp-doc pre code', 'doc', 'markdown', 'reference'],
  ['doc-custom-block', '.vp-doc .custom-block.tip', '.vp-doc .custom-block.tip', 'doc', 'markdown'],
  ['doc-table', '.vp-doc table', '.vp-doc table', 'reference'],
  ['aside-outline', '.VPDocAsideOutline', '.VPDocAsideOutline', 'doc', 'markdown', 'reference'],
  ['outline-link', '.VPDocAsideOutline .outline-link', '.VPDocAsideOutline .outline-link', 'doc', 'reference'],
  ['doc-footer', '#VPContent footer', '.VPDocFooter', 'doc', 'markdown', 'reference'],
  ['pager-link', '#VPContent footer .pager a', '.VPDocFooter .pager-link', 'doc', 'markdown', 'reference'],
  ['last-updated', '#VPContent footer .last-updated p', '.VPDocFooter .VPLastUpdated', 'doc', 'markdown', 'reference'],
  ['edit-link', '#VPContent footer a[href*="github.com"]', '.VPDocFooter .edit-link a', 'doc', 'markdown', 'reference'],
  // home hero + features
  ['hero-h1', 'h1.heading', '.VPHero h1', 'home'],
  ['hero-name', 'h1.heading span:first-child', '.VPHero .name', 'home'],
  ['hero-text', 'h1.heading span:nth-child(2)', '.VPHero .text', 'home'],
  ['hero-tagline', 'h1.heading + p', '.VPHero .tagline', 'home'],
  ['hero-action-btn', 'h1.heading + p + div a', '.VPHero .actions .VPButton', 'home'],
  ['hero-image-box', '.hero-image-box', '.VPHero .image', 'home'],
  ['features-item', '.VPFeature', '.VPFeature', 'home'],
  ['feature-title', '.VPFeature .title', '.VPFeature .title', 'home'],
  ['feature-details', '.VPFeature .details', '.VPFeature .details', 'home'],
  // overlays (state-dependent; upstream hydrates these client-side)
  ['nav-screen', '#VPNavScreen', '.VPNavScreen', 'navopen'],
  ['backdrop', '#VPBackdrop', '.VPBackdrop', 'navopen'],
];

const PROPS = {
  num: ['fontSize', 'lineHeight', 'paddingTop', 'paddingRight',
    'paddingBottom', 'paddingLeft', 'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'borderTopWidth', 'borderRadius', 'fontWeight'],
  raw: ['color', 'backgroundColor', 'position', 'display'],
};

function dumpLandmarks({ landmarks, props, side }) {
  const out = {};
  for (const [key, oursSel, upSel] of landmarks) {
    const sel = side === 'ours' ? oursSel : upSel;
    let el = null;
    for (const s of sel.split('||')) {
      const cand = document.querySelector(s.trim());
      if (cand) { el = cand; break; }
    }
    if (!el) { out[key] = null; continue; }
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    const rec = {};
    for (const p of props.num) rec[p] = Math.round(parseFloat(cs[p]) * 10) / 10;
    // rect is document coords (stable across content differences); the
    // per-property recs are intrinsic (fonts, spacing, colors)
    rec.rect = [Math.round(r.left), Math.round(r.top + window.scrollY), Math.round(r.width), Math.round(r.height)].join(',');
    for (const p of props.raw) {
      const v = cs[p];
      rec[p] = v.startsWith('rgb') ? v.replace(/\s+/g, '') : v;
    }
    rec.visible = !!el.offsetParent || cs.position === 'fixed';
    out[key] = rec;
  }
  return out;
}

function makeServer(dist) {
  const server = createServer(async (req, res) => {
    try {
      let p = decodeURIComponent(new URL(req.url, 'http://x').pathname);
      if (p.endsWith('/')) p += 'index.html';
      const tries = [p, p.replace(/\/index\.html$/, '') + '.html', p + '.html'];
      let body = null, file = null;
      for (const t of tries) {
        const f = resolve(dist, '.' + t);
        if (!f.startsWith(resolve(dist) + sep)) continue;
        if (existsSync(f) && !extname(f)) continue; // directory, not an asset
        try { body = await readFile(f); file = f; break; } catch {}
      }
      if (!body) throw new Error('not found: ' + p);
      const types = {
        '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript',
        '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png',
        '.webp': 'image/webp', '.woff2': 'font/woff2', '.woff': 'font/woff',
      };
      res.writeHead(200, { 'content-type': types[extname(file)] || 'application/octet-stream' });
      res.end(body);
    } catch (e) {
      res.writeHead(404); res.end('not found');
    }
  });
  return new Promise((r) => server.listen(0, '127.0.0.1', () => r(server)));
}

const TOL = 1; // px / unit tolerance
const COLOR_TOL = 0; // colors must match exactly after normalization

// Reviewed artifacts of the deliberate DOM divergence (not gaps): the
// landmark is still compared, these property-level differences are the
// mechanism being different, not the outcome. Keep the list tight and
// reasoned — it is a ledger, not a mute button.
const KNOWN_ARTIFACTS = [
  // upstream pads .VPContent for the navbar on home and pulls .VPHero
  // back up with a negative margin; ours leaves the content box at 0.
  // Children (hero, features) measure identical on both sides (see the
  // viewport gate's hero/feature goldens).
  ['content', 'rect.docTop', (cell) => cell.startsWith('home@'), 'home shell: upstream pads #VPContent by the nav height and -1rem-margins .VPHero; children align to the px'],
  ['content', 'rect.height', (cell) => cell.startsWith('home@'), 'same home shell mechanism'],
  // upstream's docs navbar carries Ask-AI + translations + a "more" menu
  // (site config surface); the first menu link's x-position shifts
  ['navbar-menu-link', 'rect.left', () => true, 'navbar right-side composition is demo config surface (Ask-AI, locales), not CSS'],
  // ±2px: self-hosted Inter subset rasterization vs. upstream's
  ['last-updated', 'rect.left', () => true, 'font metric residue (±2px), Inter subsetting/hinting'],
  ['last-updated', 'rect.width', () => true, 'font metric residue (±2px), Inter subsetting/hinting'],
  // upstream hydrates the outline client-side; on the scrolled probe its
  // active-link color races hydration (the CSS truth matches)
  ['outline-link', 'color', (cell) => cell.includes('scrolled'), 'upstream hydration race on the active outline link (see Flake note)'],
  // upstream sets .vp-doc position:relative ONLY at runtime (Vue injects
  // an inline style); the SSG HTML has none — no geometry impact
  ['doc-container', 'position', () => true, 'upstream .vp-doc is position:relative after hydration only; not in the source HTML, no geometry impact'],
  // the code visual card: ours carries padding on the styled pre,
  // upstream puts it on the inner pre inside the wrapper. Margin, bg,
  // radius and geometry compare across the wrapper boundary; padding is
  // where the layers split (20px on both sides, different elements)
  ['doc-code-pre', 'paddingTop', () => true, 'padding lives one layer deeper in ours (styled pre vs upstream inner pre); the card measures equal'],
  ['doc-code-pre', 'paddingBottom', () => true, 'same padding-layer split'],
  // hero buttons center the label with flex instead of upstream's fixed
  // line-height; measured identical at every width incl. the 375 wrap
  ['hero-action-btn', 'lineHeight', () => true, 'label centering via flex instead of a fixed 38px line-height; box geometry equal'],
  ['hero-action-btn', 'display', () => true, 'same flex-vs-inline-block mechanism'],
  // feature cards: we split box visuals onto the li>div wrapper and put
  // padding on .VPFeature; upstream has bg+radius on .VPFeature and
  // padding on .box. Same visual card, different layer assignment
  ['features-item', 'paddingTop', () => true, 'feature card layer split: ours pads the article, upstream pads an inner .box'],
  ['features-item', 'paddingRight', () => true, 'same card layer split'],
  ['features-item', 'paddingBottom', () => true, 'same card layer split'],
  ['features-item', 'paddingLeft', () => true, 'same card layer split'],
  ['features-item', 'borderRadius', () => true, 'same card layer split (radius lives on our wrapper)'],
  ['features-item', 'backgroundColor', () => true, 'same card layer split (bg lives on our wrapper)'],
  ['features-item', 'display', () => true, 'same card layer split'],
  // the grid cell (li) geometry is what matters visually and is pinned
  // exactly by the viewport gate's `feat w` goldens; this landmark's box
  // rides one layer on the split
  ['features-item', 'rect.left', () => true, 'measured box sits one layer into the split; li-level geometry is gated'],
  ['features-item', 'rect.docTop', () => true, 'same as features-item rect.left'],
  ['features-item', 'rect.width', () => true, 'same as features-item rect.left'],
  ['features-item', 'rect.height', () => true, 'same as features-item rect.left'],
  // everything below the math section on /guide/markdown/ shifts by the
  // client-side vs build-time MathJax height delta — the documented
  // feature-axis gap (FEATURE-PARITY.md), page-bottom landmarks share it
  ['content', 'rect.height', (cell) => cell.startsWith('markdown@'), 'math section height: client-side MathJax vs upstream build-time typesetting (feature-axis gap)'],
  ['doc-container', 'rect.height', (cell) => cell.startsWith('markdown@'), 'same math-section height delta'],
  ['doc-footer', 'rect.docTop', (cell) => cell.startsWith('markdown@'), 'page-bottom offset = the math-section delta'],
  ['pager-link', 'rect.docTop', (cell) => cell.startsWith('markdown@'), 'page-bottom offset = the math-section delta'],
  ['last-updated', 'rect.docTop', (cell) => cell.startsWith('markdown@'), 'page-bottom offset = the math-section delta'],
  ['edit-link', 'rect.docTop', (cell) => cell.startsWith('markdown@'), 'page-bottom offset = the math-section delta'],
];

const suppressed = [];
function isKnownArtifact(cell, landmark, issue) {
  const prop = issue.split(':')[0];
  const hit = KNOWN_ARTIFACTS.find(
    ([lm, p, when]) => lm === landmark && p === prop && when(cell),
  );
  if (hit) {
    suppressed.push({ cell, landmark, issue, reason: hit[3] });
    return true;
  }
  return false;
}

function diffSide(key, ours, up) {
  const issues = [];
  if (!ours && !up) return { skip: true };
  if (!ours || !up) {
    issues.push(`presence: ours=${ours ? 'yes' : 'MISSING'} upstream=${up ? 'yes' : 'MISSING'}`);
    return { issues };
  }
  if (ours.visible !== up.visible) {
    issues.push(`visibility: ours=${ours.visible} upstream=${up.visible}`);
  }
  if (!ours.visible && !up.visible) return { issues };
  for (const p of PROPS.num) {
    const d = Math.abs(ours[p] - up[p]);
    if (d > TOL) issues.push(`${p}: ours=${ours[p]} upstream=${up[p]} (Δ${d.toFixed(1)})`);
  }
  if (ours.rect !== up.rect) {
    const a = ours.rect.split(',').map(Number), b = up.rect.split(',').map(Number);
    const names = ['left', 'docTop', 'width', 'height'];
    for (let i = 0; i < 4; i++) {
      // geometry gets the same ±1 tolerance as the numeric props:
      // rounding a sub-pixel font metric into each rect is not drift
      if (Math.abs(a[i] - b[i]) > TOL) issues.push(`rect.${names[i]}: ours=${a[i]} upstream=${b[i]} (Δ${Math.abs(a[i] - b[i])})`);
    }
  }
  for (const p of PROPS.raw) {
    const a = ours[p], b = up[p];
    if (a === b) continue;
    if (a?.startsWith('rgb') && b?.startsWith('rgb')) {
      const pa = a.match(/[\d.]+/g).map(Number), pb = b.match(/[\d.]+/g).map(Number);
      const cdiff = pa.slice(0, 3).every((v, i) => Math.abs(v - pb[i]) <= COLOR_TOL)
        && Math.abs((pa[3] ?? 1) - (pb[3] ?? 1)) * 255 <= COLOR_TOL;
      if (!cdiff) issues.push(`${p}: ours=${a} upstream=${b}`);
    } else if ((a === 'none' || a === 'static' || a === 'block') && (b === 'none' || b === 'static' || b === 'block')) {
      if (a !== b) issues.push(`${p}: ours=${a} upstream=${b}`);
    } else {
      issues.push(`${p}: ours=${a} upstream=${b}`);
    }
  }
  return { issues };
}

const { chromium } = await import('playwright-chromium');
const browser = await chromium.launch({ headless: true });

if (SHOTS) await mkdir(join(AUDIT_DIR, 'shots'), { recursive: true });

const oursSrv = await makeServer(OURS_DIST);
const upSrv = await makeServer(UP_DIST);
const oursBase = `http://127.0.0.1:${oursSrv.address().port}`;
const upBase = `http://127.0.0.1:${upSrv.address().port}`;

// one upstream sanity probe: pages must exist (no 404 body)
{
  const ctx = await browser.newContext();
  const page = await ctx.newPage();
  let bad = 0;
  for (const [k, path] of Object.entries(ALL_PAGES)) {
    const resp = await page.goto(upBase + path, { waitUntil: 'domcontentloaded' });
    if (!resp.ok()) { console.error(`viewport-audit: upstream ${path} → HTTP ${resp.status()}`); bad++; }
  }
  await ctx.close();
  if (bad) process.exit(1);
  console.log(`upstream pages ok (${Object.keys(ALL_PAGES).length})`);
}

const dump = { ours: {}, up: {} };
const findings = [];

async function capture(side, base, path, width, state) {
  const ctx = await browser.newContext({ viewport: { width, height: 900 } });
  const page = await ctx.newPage();
  if (state === 'dark') {
    await page.addInitScript(() => localStorage.setItem('vitepress-theme-appearance', 'dark'));
  } else {
    await page.addInitScript(() => localStorage.removeItem('vitepress-theme-appearance'));
  }
  await page.goto(base + path, { waitUntil: 'networkidle' });
  await page.evaluate(() => document.fonts.ready);
  // settle: upstream hydrates outline/nav widgets post-networkidle, and
  // a first read right after fonts.ready has measured half-hydrated DOM
  await page.waitForTimeout(500);

  if (state === 'dark') {
    const isDark = await page.evaluate(() => document.documentElement.classList.contains('dark'));
    if (!isDark) console.error(`  warn: dark state did not apply on ${side} ${path}@${width}`);
  }
  if (state === 'scrolled') {
    await page.evaluate(() => window.scrollTo(0, 300));
    await page.waitForTimeout(700); // navbar transition-colors duration-500
  }
  if (state === 'navopen') {
    if (width >= 768) { await ctx.close(); return null; } // hamburger hidden ≥768 on both sides
    const sel = side === 'ours' ? '#VPNavBarHamburger' : '.VPNavBarHamburger';
    try {
      await page.click(sel, { timeout: 5000 });
    } catch (e) {
      console.error(`  warn: navopen click failed on ${side} ${path}@${width}: ${e.message.split('\n')[0]}`);
      await ctx.close();
      return null;
    }
    await page.waitForTimeout(700); // overlay transition duration-500
  }
  const data = await page.evaluate(dumpLandmarks, { landmarks: LANDMARKS, props: PROPS, side });
  if (SHOTS) {
    const name = `${side}-${path === '/' ? 'home' : path.replaceAll('/', '_').replace(/^_|_$/g, '')}-${width}-${state}.png`;
    await page.screenshot({ path: join(AUDIT_DIR, 'shots', name), fullPage: true });
  }
  await ctx.close();
  return data;
}

for (const pk of pageKeys) {
  const path = ALL_PAGES[pk];
  if (!path) { console.error(`unknown page key: ${pk}`); process.exit(1); }
  for (const width of widths) {
    for (const state of states) {
      const ours = await capture('ours', oursBase, path, width, state);
      const up = await capture('up', upBase, path, width, state);
      if (!ours || !up) continue;
      const cell = `${pk}@${width}/${state}`;
      dump.ours[cell] = ours; dump.up[cell] = up;
      let cellIssues = 0;
      for (const [key, , , ...pages] of LANDMARKS) {
        if (key === 'nav-screen' || key === 'backdrop') {
          if (state !== 'navopen') continue;
        } else if (state === 'navopen') continue;
        if (pages.length && !pages.includes(pk) && !pages.includes('*')) continue;
        const { skip, issues } = diffSide(key, ours[key], up[key]);
        if (skip || !issues?.length) continue;
        const remaining = issues.filter((i) => !isKnownArtifact(cell, key, i));
        if (!remaining.length) continue;
        cellIssues++;
        findings.push({ cell, landmark: key, issues: remaining });
      }
      console.log(`${cellIssues ? `${String(cellIssues).padStart(3)} diff` : '  ok'} ${cell}`);
    }
  }
}

await browser.close();
oursSrv.close(); upSrv.close();

await mkdir(AUDIT_DIR, { recursive: true });
await writeFile(join(AUDIT_DIR, 'dump.json'), JSON.stringify(dump, null, 1));
await writeFile(join(AUDIT_DIR, 'findings.json'), JSON.stringify(findings, null, 1));

console.log(`\nviewport-audit: ${findings.length} landmark-level findings across ${Object.keys(dump.ours).length} cells`);
console.log(`details: parity/audit/findings.json`);
if (suppressed.length) {
  // suppression is visible, not silent: reviewed artifacts in
  // KNOWN_ARTIFACTS, each carrying a reason
  const byReason = {};
  for (const s of suppressed) byReason[s.reason] = (byReason[s.reason] || 0) + 1;
  console.log(`suppressed as reviewed artifacts: ${suppressed.length}`);
  for (const [r, n] of Object.entries(byReason)) console.log(`  ${n}×  ${r}`);
}
if (findings.length) {
  // summary by landmark
  const byLandmark = {};
  for (const f of findings) byLandmark[f.landmark] = (byLandmark[f.landmark] || 0) + 1;
  console.log('by landmark:', Object.entries(byLandmark).sort((a, b) => b[1] - a[1])
    .map(([k, n]) => `${k}:${n}`).join('  '));
}
