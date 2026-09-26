// Guard the preprocessor's fence contract at the demo level: the gdcode
// renderer emits <span class="lang"> title bars, so a bare
// <pre><code class="language-…"> means a fence escaped the preprocessor
// entirely (regression: 4-space list-item fences went unstyled).
// A plain <pre><code> (no class) is what an INDENTED code block renders
// as; the pages known to carry them are listed below with their counts,
// every other page must have none. Per page, not site-wide: a fence that
// loses its info string on one page and a new indented example on
// another must not cancel out.
import { readdirSync, readFileSync } from 'node:fs'
import path from 'node:path'

const root = 'demo/public'
const unstyled = []
const plainByPage = new Map()
const walk = (dir) => {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name)
    if (entry.isDirectory()) walk(p)
    else if (p.endsWith('.html')) {
      const html = readFileSync(p, 'utf8')
      if (html.includes('<pre><code class="language-')) unstyled.push(p)
      const plain = html.split('<pre><code>').length - 1
      if (plain) plainByPage.set(path.relative(root, p).split(path.sep).join('/'), plain)
    }
  }
}
walk(root)
if (unstyled.length) {
  console.error('check:demo markup — bare unstyled code blocks (fence escaped the preprocessor):')
  for (const p of unstyled) console.error(`  ${p}`)
  process.exit(1)
}
const KNOWN_PLAIN = new Map([['guide/using-vue/index.html', 2]]) // the indented code blocks documented there
const mismatches = []
for (const page of new Set([...KNOWN_PLAIN.keys(), ...plainByPage.keys()])) {
  const expected = KNOWN_PLAIN.get(page) ?? 0
  const actual = plainByPage.get(page) ?? 0
  if (actual !== expected) mismatches.push(`  ${page}: ${actual} plain <pre><code>, expected ${expected}`)
}
if (mismatches.length) {
  console.error('check:demo markup — plain <pre><code> count changed (a fence lost its info string, or a new indented block needs listing):')
  for (const m of mismatches) console.error(m)
  process.exit(1)
}
console.log(`check:demo markup: no bare language-fence blocks; plain <pre><code> only on the ${KNOWN_PLAIN.size} known page(s)`)
