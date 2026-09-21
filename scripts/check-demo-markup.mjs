// Guard the preprocessor's fence contract at the demo level: the gdcode
// renderer emits <span class="lang"> title bars, so a bare
// <pre><code class="language-…"> means a fence escaped the preprocessor
// entirely (regression: 4-space list-item fences went unstyled).
// A plain <pre><code> (no class) is what an INDENTED code block renders
// as — demo currently has exactly two, both in guide/using-vue; a count
// change means a fence somewhere lost its info string.
import { readdirSync, readFileSync } from 'node:fs'
import path from 'node:path'

const root = 'demo/public'
const unstyled = []
let plain = 0
const walk = (dir) => {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name)
    if (entry.isDirectory()) walk(p)
    else if (p.endsWith('.html')) {
      const html = readFileSync(p, 'utf8')
      if (html.includes('<pre><code class="language-')) unstyled.push(p)
      plain += html.split('<pre><code>').length - 1
    }
  }
}
walk(root)
if (unstyled.length) {
  console.error('check:demo markup — bare unstyled code blocks (fence escaped the preprocessor):')
  for (const p of unstyled) console.error(`  ${p}`)
  process.exit(1)
}
const KNOWN_PLAIN = 2 // the indented code blocks in guide/using-vue
if (plain !== KNOWN_PLAIN) {
  console.error(`check:demo markup — plain <pre><code> count is ${plain}, expected ${KNOWN_PLAIN}; a fence lost its info string somewhere`)
  process.exit(1)
}
console.log(`check:demo markup: no bare language-fence blocks; plain <pre><code> at the known count (${KNOWN_PLAIN})`)
