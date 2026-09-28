#!/usr/bin/env node
// Block-flow parity gate: the sequence of content blocks (tags + first
// words + document coords) down the two code-heaviest pages, compared
// against goldens recorded from a state VERIFIED against the pinned
// upstream build.
//
// Why: landmarks compare isolated elements; this axis compares FLOW —
// the exact place where margin-collapse/spacing drift hides. It is how
// the missing `pre + pre` −0.5rem tightening rule (the audit's "residue
// between blocks") was localized, and it would have caught it the day
// it regressed.
//
// Determinism: external scripts (MathJax CDN — client-side typesetting
// is a documented feature-axis gap) are aborted so the math section
// renders raw $…$ everywhere; top/height compared at ±1px.
//
//   node scripts/parity-blockflow.mjs           # check
//   node scripts/parity-blockflow.mjs --update  # re-record after a demo
//                                               # content change; VERIFY
//                                               # a diff against upstream
//                                               # before committing
//
// Same convention as the viewport gate: no playwright → FAIL, or a
// visible skip with RUSTPRESS_ALLOW_NO_PLAYWRIGHT=1.

import { createServer } from 'node:http';
import { readFileSync, existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import { join, extname, resolve, sep } from 'node:path';

const ROOT = new URL('..', import.meta.url).pathname;
const DIST = join(ROOT, 'demo/public');
const GOLDENS = join(ROOT, 'parity/blockflow-goldens.json');
const TOL = 1;

const CASES = [
  ['markdown @375', '/guide/markdown/', 375],
  ['markdown @768', '/guide/markdown/', 768],
  ['reference @375', '/reference/default-theme-config/', 375],
  ['reference @768', '/reference/default-theme-config/', 768],
];

if (!existsSync(join(DIST, 'index.html'))) {
  console.error('parity-blockflow: demo/public missing — run npm run check:demo first');
  process.exit(1);
}

let chromium;
try {
  ({ chromium } = await import('playwright-chromium'));
} catch (e) {
  const msg = `parity-blockflow: playwright-chromium unusable (${e.message}) — SKIPPED`;
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

// leaf-block sequence in .vp-doc: tag + normalized first words + coords
const dumpFn = () => {
  const root = document.querySelector('.vp-doc');
  if (!root) return [];
  const BLOCK = /^(P|H1|H2|H3|H4|H5|H6|PRE|TABLE|UL|OL|BLOCKQUOTE|DETAILS|HR)$/;
  const isBlock = (el) => BLOCK.test(el.tagName)
    || (el.tagName === 'DIV' && ['custom-block', 'vp-code-group', 'vp-raw'].some((c) => el.classList.contains(c)));
  const out = [];
  const visit = (el) => {
    if (isBlock(el)) {
      if (getComputedStyle(el).display !== 'none') {
        const r = el.getBoundingClientRect();
        // text from <code> for PREs: our server-side lang label span
        // sits inside the pre, upstream's outside — same code, same sig
        const textEl = el.tagName === 'PRE' ? (el.querySelector('code') ?? el) : el;
        out.push([
          el.tagName + '|' + (textEl.textContent || '').replace(/\u200B/g, '').replace(/[\s\[\]]/g, '').slice(0, 48),
          Math.round(r.top + window.scrollY), Math.round(r.height),
        ]);
        return;
      }
    }
    for (const ch of el.children) visit(ch);
  };
  visit(root);
  return out;
};

const UPDATE = process.argv.includes('--update');
const goldens = UPDATE ? {} : JSON.parse(readFileSync(GOLDENS, 'utf8'));
const browser = await chromium.launch({ headless: true });
let failures = 0;
const measured = {};
try {
  for (const [label, path, width] of CASES) {
    const ctx = await browser.newContext({ viewport: { width, height: 900 } });
    const page = await ctx.newPage();
    // no network beyond localhost: MathJax must not typeset here so the
    // math section is raw text on every machine
    await page.route(/^https?:\/\/(?!127\.0\.0\.1)/, (r) => r.abort());
    await page.goto(`http://127.0.0.1:${PORT}${path}`, { waitUntil: 'networkidle' });
    await page.evaluate(() => document.fonts.ready);
    const flow = await page.evaluate(dumpFn);
    measured[label] = flow;
    if (!UPDATE) {
      const want = goldens[label];
      if (!want) {
        failures++;
        console.error(`FAIL ${label}: no golden (run --update)`);
        continue;
      }
      let bad = 0;
      if (want.length !== flow.length) {
        console.error(`FAIL ${label}: block count ${want.length} → ${flow.length}`);
        failures++;
        continue;
      }
      for (let i = 0; i < want.length; i++) {
        const [ws, wt, wh] = want[i];
        const [s, t, h] = flow[i];
        if (s !== ws) {
          console.error(`FAIL ${label} [${i}] block: ${ws} → ${s}`);
          bad++;
          break;
        } else if (Math.abs(t - wt) > TOL || Math.abs(h - wh) > TOL) {
          console.error(`FAIL ${label} [${i}] ${s}: top ${wt}→${t} height ${wh}→${h}`);
          bad++;
        }
      }
      if (!bad) console.log(`ok ${label} (${flow.length} blocks)`);
      failures += bad ? 1 : 0;
    }
    await ctx.close();
  }
} finally {
  await browser.close();
  server.close();
}

if (UPDATE) {
  await writeFile(GOLDENS, JSON.stringify(measured, null, 2) + '\n');
  console.log(`parity-blockflow: ${Object.keys(measured).length} flows recorded → parity/blockflow-goldens.json (verify against the pinned upstream build before committing)`);
} else {
  console.log(`parity-blockflow: ${CASES.length - failures}/${CASES.length} flows aligned`);
}
process.exit(failures ? 1 : 0);
