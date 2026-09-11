// Weekly upstream drift watch (see .github/workflows/drift-watch.yml):
// compare the deployed vitepress.dev generator version with the pinned
// parity baseline (parity/upstream.json → meta.generator). When upstream
// moved, mechanically prepare a refresh PR:
//   1. bump parity/upstream-ref.txt and re-check out the pinned clone
//   2. re-sync the verbatim corpora (demo/content, tests/fixtures/en)
//   3. npm run parity:refresh — re-pin landmark fingerprints, print the
//      old→new landmark diff
//   4. rebuild the demo and run parity check; remaining divergences go
//      into the PR body (they are the human work: fix rustpress or
//      review into parity/known-deltas.json)
// Zero-dependency on purpose (Node 22+ global fetch). DRIFT_WATCH_DRY_RUN=1
// stops before the git branch/push/PR steps for local rehearsal.
import { execFileSync, execSync } from 'node:child_process'
import { copyFile, mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import path from 'node:path'

const SITE = 'https://vitepress.dev'
const FIXTURES = 'tests/fixtures/en'
const UPSTREAM_DOCS = '../vitepress/docs/en'
const dryRun = process.env.DRIFT_WATCH_DRY_RUN === '1'
// --check: compare versions only, no side effects — the CI workflow uses
// it to skip the heavy install/build steps in the common "no drift" week
const checkOnly = process.argv.includes('--check')

const sh = (cmd) => execSync(cmd, { stdio: 'pipe', encoding: 'utf8' }).trim()
const run = (cmd) => {
  console.log(`drift-watch: + ${cmd}`)
  execSync(cmd, { stdio: 'inherit' })
}

async function walk(dir) {
  const out = []
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name)
    if (entry.isDirectory()) out.push(...(await walk(p)))
    else out.push(p)
  }
  return out
}

// 0. what does the deployed site run, and what are we pinned to?
const html = await (await fetch(SITE)).text()
const generator = html.match(/<meta name="generator" content="([^"]*)"/)?.[1]
if (!generator) {
  throw new Error('no <meta name="generator"> on the upstream site — inspect its HTML shape')
}
const baseline = JSON.parse(await readFile('parity/upstream.json', 'utf8'))
const pinned = baseline.meta?.generator
console.log(`drift-watch: deployed ${generator} / pinned ${pinned}`)
if (!pinned || generator === pinned) {
  console.log('drift-watch: upstream is at the pinned version — nothing to do')
  process.exit(0)
}
if (checkOnly) {
  console.log('drift-watch: upstream drifted — full refresh needed')
  process.exit(3)
}

// 1. pin the clone to the matching tag (if one exists yet). The tag
// comes from fetched upstream HTML and ends up in shell/git arguments:
// strictly validate it before use.
const tag = /^VitePress (v[0-9A-Za-z._-]+)$/.exec(generator)?.[1] ?? null
let refNote
if (tag && sh(`git ls-remote --tags https://github.com/vuejs/vitepress "refs/tags/${tag}"`)) {  const refFile = 'parity/upstream-ref.txt'
  const refLines = (await readFile(refFile, 'utf8')).split('\n')
  refLines[refLines.findLastIndex((l) => l.trim() && !l.trim().startsWith('#'))] = tag
  await writeFile(refFile, refLines.join('\n'))
  run('bash scripts/sync-upstream.sh')
  refNote = `Pinned the clone to ${tag} and re-synced demo/content + tests/fixtures/en from it.`
} else {
  refNote = `No matching tag for ${generator} yet — parity/upstream-ref.txt stays at its current ref; re-run once the release is tagged.`
  console.log(`drift-watch: ${refNote}`)
}

// 2. re-sync the verbatim corpora from the (possibly new) clone
run(`rsync -a --delete ${UPSTREAM_DOCS}/ demo/content/`)
for (const file of await walk(FIXTURES)) {
  const upstream = path.join(UPSTREAM_DOCS, path.relative(FIXTURES, file))
  if (existsSync(upstream)) await copyFile(upstream, file)
  else await rm(file)
}

// 3. re-pin the landmark fingerprints; parity:refresh prints the old→new diff
let landmarkDiff = ''
try {
  landmarkDiff = sh('npm run parity:refresh')
} catch (e) {
  landmarkDiff = `${e.stdout ?? ''}${e.stderr ?? ''}`
}
console.log(landmarkDiff)

// 4. rebuild and collect what still diverges after the mechanical part
run('npm run build:js')
run('npm run build:css')
run('cargo run --quiet -- build demo')
let remaining = ''
try {
  remaining = sh('cargo run --quiet -- parity check demo/public')
} catch (e) {
  remaining = `${e.stdout ?? ''}${e.stderr ?? ''}`
}

const trunc = (s, n = 40000) => (s.length > n ? `${s.slice(0, n)}\n… (truncated)` : s)
const report = [
  `Upstream moved: ${pinned} → ${generator}.`,
  '',
  refNote,
  '',
  '## Landmark diff (old → new fingerprints)',
  '',
  '```',
  trunc(landmarkDiff),
  '```',
  '',
  '## Remaining divergences after the mechanical refresh',
  '',
  '```',
  trunc(remaining) || '(none — the refresh was fully mechanical)',
  '```',
  '',
  'Fix rustpress for each remaining divergence, or review it into',
  '`parity/known-deltas.json` with a reason (see PARITY.md).',
  'Note: CI does not auto-run on PRs opened with GITHUB_TOKEN — push an',
  'empty commit to this branch (or run the CI workflow on it) to get a run.',
].join('\n')

if (dryRun) {
  console.log(`drift-watch: DRY RUN — would open a refresh PR with body:\n${report}`)
  process.exit(0)
}

// 5. branch, commit the mechanical refresh, open the PR
const branch = `drift-watch/${tag ?? generator.replace(/[^\w.-]+/g, '-')}`
run('git config user.name "rustpress-drift-watch[bot]"')
run('git config user.email "drift-watch@users.noreply.github.com"')
run(`git checkout -b ${branch}`)
run('git add parity/upstream.json parity/upstream-ref.txt demo/content tests/fixtures/en')
execFileSync('git', ['commit', '-m', `Parity refresh: upstream ${generator}`], { stdio: 'inherit' })
run(`git push origin ${branch}`)
// --body carries upstream diff text (quotes, backticks, $): pass it as a
// real argv element, not through a shell string
execFileSync('gh', ['pr', 'create', '--title', `Parity refresh: upstream ${generator}`, '--body', report], { stdio: 'inherit' })
console.log(`drift-watch: refresh PR opened on ${branch}`)
