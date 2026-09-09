// Fetch the pinned upstream pages (vitepress.dev — the deployed
// VitePress release) into parity/cache/ for `gen-docs parity snapshot`.
// Zero-dependency on purpose: Node 22+ global fetch.
// See PARITY.md for the full workflow.
import { mkdir, readFile, writeFile } from 'node:fs/promises'

const SOURCE = 'https://vitepress.dev'
const pages = (await readFile('parity/pages.txt', 'utf8'))
  .split('\n')
  .map((l) => l.trim())
  .filter((l) => l && !l.startsWith('#'))

await mkdir('parity/cache', { recursive: true })

const slug = (p) => (p === '/' ? 'home' : p.replaceAll('/', '_').replace(/^_/, ''))
const etags = {}
let generator

for (const page of pages) {
  const res = await fetch(SOURCE + page)
  if (!res.ok) throw new Error(`${page}: HTTP ${res.status}`)
  const html = await res.text()
  await writeFile(`parity/cache/${slug(page)}.html`, html)
  etags[page] = res.headers.get('etag') ?? ''
  generator ??= html.match(/<meta name="generator" content="([^"]*)"/)?.[1]
  console.log(`parity:refresh ${page} <- ${html.length} bytes`)
}

if (!generator) {
  throw new Error('no <meta name="generator"> found — the upstream site may have changed shape; inspect the fetched HTML')
}

await writeFile(
  'parity/cache/meta.json',
  JSON.stringify({ generator, fetched_at: new Date().toISOString(), source: SOURCE, etags }, null, 2) + '\n',
)
console.log(`parity:refresh pinned upstream ${generator} (${pages.length} pages)`)
