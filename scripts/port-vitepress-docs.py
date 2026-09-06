#!/usr/bin/env python3
"""Port the real vitepress.dev docs (docs/en) into the vitezola demo content tree.

Single fence-aware pass converts, outside code fences only:
  - `<<< @/path` code includes -> inlined fenced blocks (regions extracted)
  - VitePress fence annotations `lang [Title]` / `lang{1,2-4}` -> `lang,name=…,hl_lines=…`
  - `:::` containers -> Tera 2 component blocks (tip/details/codegroup/v-pre)
  - `<Badge type="x" text="y" />` -> <span class="VPBadge x">y</span>
Regions containing `{{`/`{%` get inline {% raw %} markers (works inside fences
too — Zola templates markdown before rendering, fences are plain text to Tera).
The leading H1 is stripped; its text becomes the front-matter title.
"""
import re
import sys
from pathlib import Path

SRC = Path('/home/newlix/github/at-least/vitepress/docs/en')
DOCS = Path('/home/newlix/github/at-least/vitepress/docs')
DST = Path('/home/newlix/github/at-least/vitezola/content')

# dest rel path (without .md) -> src rel path; title/weight come from the H1 / order below
PAGES = {
    'guide/introduction/what-is-vitepress': 'guide/what-is-vitepress.md',
    'guide/introduction/getting-started': 'guide/getting-started.md',
    'guide/introduction/routing': 'guide/routing.md',
    'guide/introduction/deploy': 'guide/deploy.md',
    'guide/writing/markdown': 'guide/markdown.md',
    'guide/writing/asset-handling': 'guide/asset-handling.md',
    'guide/writing/frontmatter': 'guide/frontmatter.md',
    'guide/writing/using-vue': 'guide/using-vue.md',
    'guide/writing/i18n': 'guide/i18n.md',
    'guide/customization/custom-theme': 'guide/custom-theme.md',
    'guide/customization/extending-default-theme': 'guide/extending-default-theme.md',
    'guide/customization/data-loading': 'guide/data-loading.md',
    'guide/customization/ssr-compat': 'guide/ssr-compat.md',
    'guide/customization/cms': 'guide/cms.md',
    'guide/experimental/mpa-mode': 'guide/mpa-mode.md',
    'guide/experimental/sitemap-generation': 'guide/sitemap-generation.md',
    'reference/site-config': 'reference/site-config.md',
    'reference/frontmatter-config': 'reference/frontmatter-config.md',
    'reference/runtime-api': 'reference/runtime-api.md',
    'reference/cli': 'reference/cli.md',
    'reference/default-theme/config': 'reference/default-theme-config.md',
    'reference/default-theme/nav': 'reference/default-theme-nav.md',
    'reference/default-theme/sidebar': 'reference/default-theme-sidebar.md',
    'reference/default-theme/home-page': 'reference/default-theme-home-page.md',
    'reference/default-theme/footer': 'reference/default-theme-footer.md',
    'reference/default-theme/layout': 'reference/default-theme-layout.md',
    'reference/default-theme/badge': 'reference/default-theme-badge.md',
    'reference/default-theme/team-page': 'reference/default-theme-team-page.md',
    'reference/default-theme/prev-next-links': 'reference/default-theme-prev-next-links.md',
    'reference/default-theme/edit-link': 'reference/default-theme-edit-link.md',
    'reference/default-theme/last-updated': 'reference/default-theme-last-updated.md',
    'reference/default-theme/search': 'reference/default-theme-search.md',
    'reference/default-theme/carbon-ads': 'reference/default-theme-carbon-ads.md',
}

WEIGHTS = {
    'guide/introduction/what-is-vitepress': 1,
    'guide/introduction/getting-started': 2,
    'guide/introduction/routing': 3,
    'guide/introduction/deploy': 4,
    'guide/writing/markdown': 1,
    'guide/writing/asset-handling': 2,
    'guide/writing/frontmatter': 3,
    'guide/writing/using-vue': 4,
    'guide/writing/i18n': 5,
    'guide/customization/custom-theme': 1,
    'guide/customization/extending-default-theme': 2,
    'guide/customization/data-loading': 3,
    'guide/customization/ssr-compat': 4,
    'guide/customization/cms': 5,
    'guide/experimental/mpa-mode': 1,
    'guide/experimental/sitemap-generation': 2,
    'reference/site-config': 1,
    'reference/frontmatter-config': 2,
    'reference/runtime-api': 3,
    'reference/cli': 4,
    'reference/default-theme/config': 1,
    'reference/default-theme/nav': 2,
    'reference/default-theme/sidebar': 3,
    'reference/default-theme/home-page': 4,
    'reference/default-theme/footer': 5,
    'reference/default-theme/layout': 6,
    'reference/default-theme/badge': 7,
    'reference/default-theme/team-page': 8,
    'reference/default-theme/prev-next-links': 9,
    'reference/default-theme/edit-link': 10,
    'reference/default-theme/last-updated': 11,
    'reference/default-theme/search': 12,
    'reference/default-theme/carbon-ads': 13,
}

KINDS = {'tip': 'tip', 'note': 'note', 'info': 'note', 'warning': 'warning',
         'danger': 'danger', 'important': 'important', 'caution': 'caution'}

TOML_STR = lambda s: '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'

ANSI_RE = re.compile(r'\x1b\[[0-9;]*m')
BADGE_RE = re.compile(r'<Badge type="(\w+)" text="([^"]*)"\s*/>')
FENCE_OPEN_RE = re.compile(
    r'^(`{3,}|~{3,})\s*([A-Za-z0-9_#+:.-]*)\s*(?:\{([0-9][0-9,\s-]*)\})?\s*(?:\[([^\]]+)\])?\s*$')
LINENUMBERS_RE = re.compile(r':(?:no-)?line-numbers(?:=\d+)?')
INCLUDE_RE = re.compile(r'^<<< (@?[\w./@#-]+)(?:\{[^}]*\})?(?:\s+\[([^\]]+)\])?\s*$')
CONTAINER_OPEN_RE = re.compile(r'^:{3,}\s+(.*)$')
CONTAINER_CLOSE_RE = re.compile(r'^:{3,}\s*$')

# basename -> absolute dest path, for rewriting cross-group relative links
LINK_MAP = {}
for _k, _s in PAGES.items():
    LINK_MAP[Path(_k).name] = '/' + _k + '/'
    LINK_MAP.setdefault(Path(_s).stem, '/' + _k + '/')
REL_LINK_RE = re.compile(
    r'\]\((?:\./|\.\./[a-z0-9-]+/|\.\./)([A-Za-z0-9_-]+?)(\.md)?(#[^)\s]*)?\)')


def split_front_matter(text):
    if text.startswith('---\n'):
        end = text.index('\n---\n', 4)
        return text[4:end], text[end + 5:]
    return None, text


def strip_ansi(s):
    return ANSI_RE.sub('', s)


def inline_include(path, label):
    real, _, region = path.replace('@/', '').partition('#')
    f = DOCS / real
    text = f.read_text()
    if region:
        m = re.search(
            rf'^\s*(?://|<!--|/\*)\s*#region\s+{re.escape(region)}.*?$\n(.*?)^\s*(?://|<!--|/\*)\s*#endregion.*$',
            text, re.M | re.S)
        if m:
            text = m.group(1)
        else:
            raise SystemExit(f'region {region!r} not found in {real}')
    ext = f.suffix.lstrip('.')
    lang = {'ts': 'ts', 'js': 'js', 'vue': 'vue', 'mts': 'ts'}.get(ext, '')
    if ext == 'ansi':
        text = strip_ansi(text)
        lang = ''
    info = lang
    if label:
        info += (',' if info else '') + 'name=' + label
    return f'```{info}\n{text.rstrip()}\n```'


def convert(body):
    """Fence-aware single pass over all line-level constructs."""
    out = []
    stack = []           # open ::: containers
    fence = None         # (char, length) of the open fence, if any
    for line in body.split('\n'):
        stripped = line.strip()
        if fence:
            # a fence is only closed by a matching marker at least as long
            m = re.match(r'^(%s{%d,})\s*$' % (re.escape(fence[0]), fence[1]), stripped)
            if m:
                fence = None
            out.append(line)
            continue
        m = re.match(r'^(`{3,}|~{3,})', stripped)
        if m:
            marker = m.group(1)
            fence = (marker[0], len(marker))
            fm = FENCE_OPEN_RE.match(stripped)
            if fm and (fm.group(2) or fm.group(3) or fm.group(4)):
                # giallo has no :line-numbers modifier; drop it, keep the rest
                info = LINENUMBERS_RE.sub('', fm.group(2))
                attrs = []
                if fm.group(3):
                    attrs.append('hl_lines=' + fm.group(3).replace(',', ' ').strip())
                if fm.group(4):
                    attrs.append('name=' + fm.group(4))
                if attrs:
                    info = info + (',' if info else '') + ','.join(attrs)
                indent = line[:len(line) - len(line.lstrip())]
                out.append(indent + fm.group(1) + info)
                continue
            out.append(line)
            continue
        m = INCLUDE_RE.match(line)
        if m:
            out.append(inline_include(m.group(1), m.group(2)))
            continue
        m = CONTAINER_OPEN_RE.match(line)
        if m and m.group(1).strip():
            arg = m.group(1).strip()
            if arg == 'code-group':
                stack.append('codegroup')
                out.append('{% <codegroup> %}')
                continue
            if arg == 'v-pre':
                stack.append('v-pre')
                continue
            m2 = re.match(r'^(\w+)\s*(.*)$', arg)
            typ, rest = m2.group(1), m2.group(2).strip()
            if typ == 'details':
                stack.append('details')
                open_attr = False
                if rest.endswith('}'):
                    rest, _, opts = rest.rpartition('{')
                    rest = rest.strip()
                    open_attr = 'open' in opts
                out.append('{% <details summary=' + TOML_STR(rest) + ' open={'
                           + ('true' if open_attr else 'false') + '}> %}')
                continue
            if typ in KINDS:
                kind = KINDS[typ]
                no_title = False
                if rest.startswith('{'):
                    if 'no-title' in rest:
                        no_title = True
                    rest = ''
                # Tera 2 block calls require every declared parameter; bare
                # attrs are strings, so booleans use the {expression} form
                out.append('{% <tip kind=' + TOML_STR(kind) + ' title=' + TOML_STR(rest)
                           + ' no_title={' + ('true' if no_title else 'false') + '}> %}')
                stack.append('tip')
                continue
            out.append(line)
            continue
        if re.match(r'^:{3,}\s*$', line) and stack:
            top = stack.pop()
            if top != 'v-pre':
                out.append('{% </' + top + '> %}')
            continue
        out.append(BADGE_RE.sub(
            lambda mm: f'<span class="VPBadge {mm.group(1)}">{mm.group(2)}</span>', line))
    if fence:
        raise SystemExit(f'unterminated code fence at end of file: {fence}')
    if stack:
        raise SystemExit(f'unbalanced containers: {stack}')
    return '\n'.join(out)


def rewrite_links(body):
    body = REL_LINK_RE.sub(
        lambda m: '](' + LINK_MAP[m.group(1)] + (m.group(3) or '') + ')'
        if m.group(1) in LINK_MAP else m.group(0), body)
    return body.replace('](../)', '](/)')


def wrap_raw(body):
    """Insert inline {% raw %}…{% endraw %} around line runs containing
    template syntax so Tera leaves the braces in the output."""
    lines = body.split('\n')
    ours = re.compile(r'^\{%\s*</?(tip|details|codegroup)')
    runs = []
    i = 0
    while i < len(lines):
        if ('{{' in lines[i] or '{%' in lines[i]) and not ours.match(lines[i].strip()):
            j = i
            while j < len(lines) and (('{{' in lines[j] or '{%' in lines[j])
                                      and not ours.match(lines[j].strip())):
                j += 1
            runs.append((i, j - 1))
            i = j
        else:
            i += 1
    for start, end in reversed(runs):
        lines[start] = '{% raw %}' + lines[start]
        lines[end] = lines[end] + '{% endraw %}'
    return '\n'.join(lines)


H1_RE = re.compile(r'^#\s+(.*?)\s*$')


def port_page(dest_rel, src_rel):
    fm_src, body = split_front_matter((SRC / src_rel).read_text())
    desc = ''
    outline_deep = False
    if fm_src:
        m = re.search(r'^description:\s*(.+)$', fm_src, re.M)
        if m:
            desc = m.group(1).strip().strip('"')
        m = re.search(r'^outline:\s*(.+)$', fm_src, re.M)
        if m and m.group(1).strip() == 'deep':
            outline_deep = True
    body = convert(body)
    body = rewrite_links(body)
    body = wrap_raw(body)

    # strip the first top-level H1; its text is the page title (badges removed).
    # Only scan BEFORE the first fence so a fenced `# …` example can never
    # shadow the real heading. Some pages (team-page) carry a script preamble.
    lines = body.lstrip('\n').split('\n')
    title = None
    for idx, line in enumerate(lines):
        if re.match(r'^(`{3,}|~{3,})', line.strip()):
            break
        m = H1_RE.match(line)
        if m:
            title = re.sub(r'<[^>]+>', '', m.group(1)).strip()
            del lines[idx]
            break
    if not title:
        raise SystemExit(f'no H1 in {src_rel}')

    out = ['+++', 'title = ' + TOML_STR(title),
           'weight = ' + str(WEIGHTS[dest_rel])]
    if desc:
        out.append('description = ' + TOML_STR(desc))
    if outline_deep:
        out.append('[extra]\noutline = "deep"')
    out += ['+++', '']
    out.append('\n'.join(lines).strip() + '\n')
    dest = DST / (dest_rel + '.md')
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text('\n'.join(out))


def port_section(dest_rel, title, weight, raw_extra=''):
    dest = DST / (dest_rel + '.md')
    dest.parent.mkdir(parents=True, exist_ok=True)
    out = ['+++', 'title = ' + TOML_STR(title), 'weight = ' + str(weight),
           'sort_by = "weight"', 'template = "section.html"']
    if raw_extra:
        out += [raw_extra]
    out += ['+++', '']
    dest.write_text('\n'.join(out))


def main():
    for rel in list(PAGES):
        dest = DST / (rel + '.md')
        if dest.exists():
            dest.unlink()
    for rel, src in PAGES.items():
        port_page(rel, src)
    port_section('guide/_index', 'Guide', 1,
                 '[[extra.vitepress_extra_items]]\n'
                 'text = "Config & API Reference"\n'
                 'link = "/reference/site-config/"')
    port_section('guide/introduction/_index', 'Introduction', 1,
                 '[extra]\nvitepress_collapsed = false')
    port_section('guide/writing/_index', 'Writing', 2,
                 '[extra]\nvitepress_collapsed = false')
    port_section('guide/customization/_index', 'Customization', 3,
                 '[extra]\nvitepress_collapsed = false')
    port_section('guide/experimental/_index', 'Experimental', 4,
                 '[extra]\nvitepress_collapsed = false')
    port_section('reference/_index', 'Reference', 2,
                 '[extra]\nvitepress_sidebar_title = "Reference"')
    port_section('reference/default-theme/_index', 'Default Theme', 5)
    print(f'ported {len(PAGES)} pages + 7 sections')


if __name__ == '__main__':
    sys.exit(main())
