#!/usr/bin/env node
// Client-behavior gate: the Alpine.js wiring (code-group tabs, copy
// buttons, search modal + scoring, appearance toggle, mobile overlays,
// scrollspy) has no structural signature in static HTML — a broken
// bundle or a mistyped x-data passes every other axis. This script is
// the behavior axis: drive a headless Chromium against the demo build
// and assert the visible outcomes.
//
// Unlike the other axes this is a SELF-regression gate (upstream runs a
// Vue runtime; outcomes are not cross-compared here — feature-level
// differences stay on the FEATURE-PARITY axis).
//
// Same conventions as the viewport/blockflow gates: demo/public must be
// fresh (npm run check:demo), no playwright → FAIL unless
// RUSTPRESS_ALLOW_NO_PLAYWRIGHT=1 (visible skip).

import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join, extname, resolve, sep } from 'node:path';

const ROOT = new URL('..', import.meta.url).pathname;
const DIST = join(ROOT, 'demo/public');

if (!existsSync(join(DIST, 'index.html'))) {
  console.error('parity-behavior: demo/public missing — run npm run check:demo first');
  process.exit(1);
}

let chromium;
try {
  ({ chromium } = await import('playwright-chromium'));
} catch (e) {
  const msg = `parity-behavior: playwright-chromium unusable (${e.message}) — SKIPPED`;
  if (process.env.RUSTPRESS_ALLOW_NO_PLAYWRIGHT === '1') {
    console.log(msg);
    process.exit(0);
  }
  console.error(msg + '\n  (RUSTPRESS_ALLOW_NO_PLAYWRIGHT=1 turns this into a visible skip)');
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
    const types = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png', '.webp': 'image/webp', '.woff2': 'font/woff2' };
    res.writeHead(200, { 'content-type': types[extname(file)] || 'application/octet-stream' });
    res.end(body);
  } catch {
    res.writeHead(404); res.end('not found');
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const PORT = server.address().port;
const base = `http://127.0.0.1:${PORT}`;

let failures = 0;
const ok = (label, cond, detail = '') => {
  if (cond) console.log(`ok ${label}`);
  else {
    failures++;
    console.error(`FAIL ${label}${detail ? ` — ${detail}` : ''}`);
  }
};

const browser = await chromium.launch({ headless: true });
try {
  // ---- code-group tabs -------------------------------------------------
  {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
    const page = await ctx.newPage();
    await page.goto(`${base}/guide/markdown/`, { waitUntil: 'networkidle' });
    const group = page.locator('.vp-code-group[x-data="codeGroup"]').first();
    const tabs = group.locator('.tabs label');
    const pres = group.locator('.blocks > pre');
    const nTabs = await tabs.count();
    ok('group has ≥2 tabs', nTabs >= 2, `got ${nTabs}`);
    await tabs.nth(1).click();
    const vis0 = await pres.nth(0).isVisible();
    const vis1 = await pres.nth(1).isVisible();
    ok('tab click switches panes', !vis0 && vis1, `pane0 visible=${vis0}, pane1 visible=${vis1}`);
    await tabs.nth(0).click();
    ok('tab click switches back', await pres.nth(0).isVisible());
    await ctx.close();
  }

  // ---- copy button -----------------------------------------------------
  {
    const ctx = await browser.newContext({
      viewport: { width: 1280, height: 900 },
      permissions: ['clipboard-read', 'clipboard-write'],
    });
    const page = await ctx.newPage();
    await page.goto(`${base}/guide/markdown/`, { waitUntil: 'networkidle' });
    const pre = page.locator('.vp-doc pre').first();
    const want = await pre.locator('code').innerText();
    await pre.locator('.vp-copy-button').click({ force: true });
    await page.waitForTimeout(200);
    const copied = await pre.locator('.vp-copy-button').evaluate((b) => b.classList.contains('copied'));
    const clip = await page.evaluate(() => navigator.clipboard.readText());
    ok('copy button marks .copied', copied);
    ok('clipboard holds the code', clip.trim() === want.trim(), clip.trim().slice(0, 60));
    await ctx.close();
  }

  // ---- search modal ----------------------------------------------------
  {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
    const page = await ctx.newPage();
    await page.goto(`${base}/guide/what-is-vitepress/`, { waitUntil: 'networkidle' });
    const dialog = page.locator('#VPLocalSearchBox'); // role=dialog is on this root
    ok('search modal hidden initially', !(await dialog.isVisible()));
    await page.keyboard.press('Control+k');
    await page.waitForTimeout(300);
    ok('Ctrl+K opens the modal', await dialog.isVisible());
    const input = page.locator('#VPLocalSearchBox input[type="search"], #VPLocalSearchBox input').first();
    ok('input focused', await page.evaluate(() => document.activeElement?.tagName === 'INPUT'));
    await input.fill('frontmatter');
    await page.waitForTimeout(400); // fetch index + score
    const first = page.locator('#VPLocalSearchBox li.result').first();
    const url = await first.getAttribute('data-url');
    ok('frontmatter query ranks frontmatter page first', (url || '').includes('/guide/frontmatter'), url);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(250);
    ok('Escape closes the modal', !(await dialog.isVisible()));
    await ctx.close();
  }

  // ---- appearance toggle -----------------------------------------------
  {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
    const page = await ctx.newPage();
    await page.goto(`${base}/guide/what-is-vitepress/`, { waitUntil: 'networkidle' });
    ok('starts light', await page.evaluate(() => !document.documentElement.classList.contains('dark')));
    await page.locator('#VPSwitchAppearance').click();
    await page.waitForTimeout(300); // view transition applies synchronously in headless
    const dark = await page.evaluate(() => document.documentElement.classList.contains('dark'));
    const stored = await page.evaluate(() => localStorage.getItem('vitepress-theme-appearance'));
    ok('toggle flips to dark', dark, String(dark));
    ok('preference persisted', stored === 'dark', stored);
    const aria = await page.locator('#VPSwitchAppearance').getAttribute('aria-checked');
    ok('aria-checked synced', aria === 'true', aria);
    await page.reload({ waitUntil: 'networkidle' });
    ok('dark survives reload (anti-FOUC)', await page.evaluate(() => document.documentElement.classList.contains('dark')));
    await ctx.close();
  }

  // ---- mobile overlays + scroll lock ------------------------------------
  {
    const ctx = await browser.newContext({ viewport: { width: 500, height: 800 } });
    const page = await ctx.newPage();
    await page.goto(`${base}/guide/what-is-vitepress/`, { waitUntil: 'networkidle' });
    const screen = page.locator('#VPNavScreen');
    await page.locator('#VPNavBarHamburger').click();
    await page.waitForTimeout(300);
    ok('nav screen opens', await screen.isVisible());
    ok('body scroll locked', await page.evaluate(() => document.body.style.overflow === 'hidden'));
    await page.setViewportSize({ width: 1024, height: 800 });
    await page.waitForTimeout(300);
    ok('growing past the breakpoint auto-closes it', !(await screen.isVisible()));
    ok('scroll lock released', await page.evaluate(() => document.body.style.overflow !== 'hidden'));
    await page.setViewportSize({ width: 500, height: 800 });
    await page.locator('#VPLocalNavMenu').click();
    await page.waitForTimeout(300);
    const sidebar = page.locator('#VPSidebar');
    ok('sidebar drawer opens', await page.evaluate(() => document.body.style.overflow === 'hidden'));
    ok('the drawer dims the page with the backdrop', await page.locator('#VPBackdrop').isVisible());
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);
    ok('Escape closes the drawer', await page.evaluate(() => document.body.style.overflow !== 'hidden'));
    ok('focus returns to the menu button', await page.evaluate(() => document.activeElement?.id === 'VPLocalNavMenu'));
    await ctx.close();
  }

  // ---- mobile nav screen links -------------------------------------------
  {
    const ctx = await browser.newContext({ viewport: { width: 375, height: 800 } });
    const page = await ctx.newPage();
    await page.goto(`${base}/guide/what-is-vitepress/`, { waitUntil: 'networkidle' });
    await page.locator('#VPNavBarHamburger').click();
    await page.waitForTimeout(300);
    // upstream shows the backdrop for the sidebar drawer only; one over
    // the nav screen swallowed every tap (its click handler closed the
    // menu), so the mobile menu could not navigate anywhere
    ok('no backdrop over the nav screen', !(await page.locator('#VPBackdrop').isVisible()));
    // a dropdown nav item is an accordion on the screen (upstream
    // VPNavMenuGroup screen variant); the Alpine port left it inert
    const group = page.locator('#VPNavScreen .VPNavScreenMenuGroup').first();
    const groupBtn = group.locator('button').first();
    const groupLink = group.locator('a').first();
    ok('screen dropdown starts collapsed', !(await groupLink.isVisible()) && (await groupBtn.getAttribute('aria-expanded')) === 'false');
    await groupBtn.click();
    await page.waitForTimeout(300);
    ok('tapping a screen dropdown expands it', (await groupLink.isVisible()) && (await groupBtn.getAttribute('aria-expanded')) === 'true');
    await groupBtn.click();
    await page.waitForTimeout(300);
    ok('tapping it again collapses it', !(await groupLink.isVisible()) && (await groupBtn.getAttribute('aria-expanded')) === 'false');
    const box = await page.locator('#VPNavScreen a[href*="/reference/"]').first().boundingBox();
    // a real tap at the link's position, not a selector click (which
    // waits out an intercepting overlay instead of hitting it)
    await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
    await page.waitForURL(/\/reference\//, { timeout: 2000 }).catch(() => {});
    ok('tapping a nav screen link navigates', new URL(page.url()).pathname.startsWith('/reference/'), page.url());
    await ctx.close();
  }

  // ---- scroll lock keeps the scrollbar gutter ------------------------------
  {
    // classic scrollbars, as on Windows/Linux desktops (headless hides
    // them by default): hiding the page's scrollbar under an overlay must
    // not shift the layout sideways — upstream's useBodyScrollLock keeps
    // the root gutter with scrollbar-gutter: stable while locked
    const sbBrowser = await chromium.launch({ headless: true, ignoreDefaultArgs: ['--hide-scrollbars'] });
    try {
      const page = await sbBrowser.newPage({ viewport: { width: 1280, height: 800 } });
      await page.goto(`${base}/guide/what-is-vitepress/`, { waitUntil: 'networkidle' });
      const x = () => page.evaluate(() => document.getElementById('VPSwitchAppearance').getBoundingClientRect().left);
      const before = await x();
      await page.keyboard.press('Control+k');
      await page.waitForTimeout(300);
      const during = await x();
      ok('an open overlay leaves the layout in place', during === before, `navbar switch x ${before} → ${during}`);
      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);
      ok('closing it restores the root gutter', await page.evaluate(() => document.documentElement.style.scrollbarGutter === ''));
    } finally {
      await sbBrowser.close();
    }
  }

  // ---- scrollspy ---------------------------------------------------------
  {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
    const page = await ctx.newPage();
    await page.goto(`${base}/guide/markdown/`, { waitUntil: 'networkidle' });
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.waitForTimeout(150);
    ok('navbar .top at scrollTop', await page.evaluate(() => document.getElementById('VPNavBar').classList.contains('top')));
    await page.evaluate(() => {
      const h = document.getElementById('custom-containers');
      // place the heading 50px down the viewport — inside the spy's
      // 96px offset window — regardless of sticky-header behavior
      window.scrollTo(0, h.getBoundingClientRect().top + window.scrollY - 50);
    });
    await page.waitForTimeout(300);
    const active = await page.evaluate(() => {
      const l = document.querySelector('.VPDocAsideOutline a.outline-link.active');
      return l?.getAttribute('href') ?? null;
    });
    ok('outline scrollspy activates the in-view heading', active === '#custom-containers', active);
    ok('navbar loses .top when scrolled', await page.evaluate(() => !document.getElementById('VPNavBar').classList.contains('top')));
    await ctx.close();
  }
} finally {
  await browser.close();
  server.close();
}

console.log(failures ? `parity-behavior: ${failures} case(s) failed` : 'parity-behavior: all interaction cases passed');
process.exit(failures ? 1 : 0);
