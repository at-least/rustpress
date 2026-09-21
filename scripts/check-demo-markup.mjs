// Guard the preprocessor's fence contract at the demo level: the gdcode
// renderer emits <span class="lang"> title bars, so a bare
// <pre><code class="language-…"> means a fence escaped the preprocessor
// entirely (regression: 4-space list-item fences went unstyled).
import { readdirSync, readFileSync } from 'node:fs'
import path from 'node:path'

const root = 'demo/public'
const bare = []
const walk = (dir) => {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name)
    if (entry.isDirectory()) walk(p)
    else if (p.endsWith('.html') && readFileSync(p, 'utf8').includes('<pre><code class="language-'))
      bare.push(p)
  }
}
walk(root)
if (bare.length) {
  console.error('check:demo markup — bare unstyled code blocks (fence escaped the preprocessor):')
  for (const p of bare) console.error(`  ${p}`)
  process.exit(1)
}
console.log('check:demo markup: no bare <pre><code class="language-…"> blocks')
