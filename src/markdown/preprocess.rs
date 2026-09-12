//! Markdown preprocessing before comrak — everything VitePress does with
//! markdown-it plugins, done as fence-aware line passes:
//!
//! 1. `<<< @/path` / `<!--@include: file.md-->` file includes
//! 2. `:::` containers → HTML-block wrappers (the markdown-it-container
//!    trick: an HTML block ends at a blank line, so the inner content is
//!    parsed as markdown in the *same* comrak pass — footnotes, link
//!    reference definitions and TOC collection stay intact)
//! 3. fence info-string rewriting to the `gdcode` sentinel
//!    (```` ```js{1,3} [npm] ```` → ```` ```gdcode lang=js hl=1,3 label=npm ````)
//! 4. `<Badge>` → `<span class="VPBadge …">`
//!
//! All passes leave code-fence content untouched (a fence line opens on
//! a run of ≥3 backticks/tildes and closes on a run at least as long;
//! shorter runs inside are literal text).

use std::path::{Path, PathBuf};

use regex::Regex;

use super::highlight::FENCE_LANG;
use crate::config::ContainerOptions;

/// Errors during preprocessing (mostly bad includes).
#[derive(Debug, thiserror::Error)]
pub enum PreprocessError {
    #[error("cannot read include {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("include depth exceeded at {path}")]
    Depth { path: PathBuf },
}

pub struct Preprocess<'a> {
    /// The site root: `@/` includes resolve against it.
    pub site_root: &'a Path,
    /// The content directory (`../`-style includes resolve against the
    /// page's directory inside it).
    pub content_dir: &'a Path,
    /// Container labels and custom kinds ([markdown.container]).
    pub container: ContainerOptions,
    /// Site `base`, prefixed onto root-absolute URLs of `[x](/y){attrs}`
    /// links (they become raw HTML here and never reach the link rewriter).
    pub base: &'a str,
}

const MAX_INCLUDE_DEPTH: u8 = 8;

/// Info-string token marking a fence as a `::: code-group` member. A
/// token (not a substring) so a bare "```" opener — whose trimmed info
/// is exactly this marker — is still detected, and no natural info
/// string collides with it.
const GD_GROUP_TOKEN: &str = "gd-ingroup";

impl<'a> Preprocess<'a> {
    pub fn run(&self, body: &str, page_rel: &str) -> Result<String, PreprocessError> {
        let md = self.resolve_includes(body, page_rel, 0)?;
        let md = expand_inline_footnotes(&md);
        let md = rewrite_link_attrs(&md, self.base);
        let md = expand_alerts(&md, &self.container);
        let md = expand_containers(&md, &self.container);
        let md = rewrite_fences(&md);
        Ok(rewrite_badges(&md))
    }

    /// Pass 1: `<<<` code includes and `<!--@include:-->` markdown
    /// includes. `page_rel` is the page's source-relative path.
    /// Include targets take an optional `#section` (VS Code region or
    /// heading anchor) and an optional `{a,b}` line range; inside code
    /// fences the directive inserts the selected lines verbatim
    /// (upstream "Including Code Files").
    fn resolve_includes(&self, md: &str, page_rel: &str, depth: u8) -> Result<String, PreprocessError> {
        let mut out = String::with_capacity(md.len());
        let mut fence: Option<(char, usize)> = None;
        let page_dir = self.content_dir.join(Path::new(page_rel).parent().unwrap_or(Path::new("")));
        for line in md.split_inclusive('\n') {
            let bare = line.trim_end_matches(['\n', '\r']);
            let t = bare.trim();
            if let Some((ch, n)) = fence {
                if let Some(target) = md_include_target(t) {
                    // verbatim insertion — the raw selected lines join
                    // the code block content
                    out.push_str(&self.load_include(&target, &page_dir, depth)?);
                } else {
                    out.push_str(line);
                    if is_closing_fence(bare, ch, n) {
                        fence = None;
                    }
                }
                continue;
            }
            if let Some((ch, n)) = opening_fence(bare) {
                out.push_str(line);
                fence = Some((ch, n));
                continue;
            }
            if t.starts_with("<<<") {
                match self.expand_code_include(t, &page_dir) {
                    Some(block) => {
                        // The block may itself pull includes (rare);
                        // recurse on just this chunk.
                        let resolved =
                            self.resolve_includes(&block, page_rel, depth + 1)?;
                        out.push_str(&resolved);
                        if !resolved.ends_with('\n') {
                            out.push('\n');
                        }
                    }
                    None => out.push_str(line), // unsupported form: keep literal
                }
                continue;
            }
            if let Some(target) = md_include_target(t) {
                if depth >= MAX_INCLUDE_DEPTH {
                    return Err(PreprocessError::Depth { path: target.raw_path() });
                }
                let inner = self.load_include(&target, &page_dir, depth)?;
                let inner = self.resolve_includes(&inner, page_rel, depth + 1)?;
                out.push_str(&inner);
                if !inner.ends_with('\n') {
                    out.push('\n');
                }
                continue;
            }
            out.push_str(line);
        }
        Ok(out)
    }

    /// Read an include target and apply its `#section` + `{range}`
    /// selectors. `@/`-prefixed paths resolve against the site root,
    /// everything else against the including page's directory.
    fn load_include(
        &self,
        target: &IncludeTarget,
        page_dir: &Path,
        depth: u8,
    ) -> Result<String, PreprocessError> {
        if depth >= MAX_INCLUDE_DEPTH {
            return Err(PreprocessError::Depth { path: target.raw_path() });
        }
        let file = if let Some(rel) = target.path.strip_prefix("@/") {
            self.site_root.join(rel)
        } else {
            page_dir.join(&target.path)
        };
        let mut content = std::fs::read_to_string(&file).map_err(|source| PreprocessError::Read {
            path: file.clone(),
            source,
        })?;
        if let Some(section) = &target.section {
            content = extract_region(&content, section)
                .or_else(|| extract_heading_section(&content, section))
                .unwrap_or_default();
        }
        if let Some(range) = &target.range {
            content = apply_line_range(&content, range);
        }
        Ok(content)
    }

    /// `<<< @/snippets/x.ts` / `<<< ../y.js` / `<<< ./z.vue [label]` → a
    /// fenced block; returns None for forms we don't support, which stay
    /// literal.
    fn expand_code_include(&self, line: &str, page_dir: &Path) -> Option<String> {
        let rest = line.trim_start_matches("<<<").trim();
        let (mut target, label) = match rest.find(" [") {
            Some(i) if rest.ends_with(']') => (&rest[..i], Some(rest[i + 2..rest.len() - 1].to_string())),
            _ => (rest, None),
        };
        // the brace spec comes after any #region anchor — parse it first
        let mut hl: Option<String> = None;
        let mut lang_switch: Option<String> = None;
        let mut ln = false;
        if let Some(i) = target.find('{') {
            let j = target[i..].find('}')?;
            let spec = &target[i + 1..i + j];
            for token in spec.split_whitespace() {
                if token == ":line-numbers" {
                    ln = true;
                } else if token.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '-') {
                    // upstream semantics: `{2}` HIGHLIGHTS line 2 (the
                    // spec passes through to the fence info), it does not
                    // select a line range — `<<< @/x.js{2}` renders every
                    // line with line 2 highlighted
                    hl = Some(token.to_string());
                } else if lang_switch.is_none() {
                    // first non-numeric token is the language (`{ts
                    // twoslash}` passes a plugin flag we don't implement)
                    lang_switch = Some(token.trim_end_matches(':').to_string());
                }
            }
            target = &target[..i];
        }
        let mut region = None;
        if let Some(i) = target.find('#') {
            region = Some(target[i + 1..].to_string());
            target = &target[..i];
        }
        let file = if let Some(rel) = target.strip_prefix("@/") {
            self.site_root.join(rel)
        } else {
            page_dir.join(target)
        };
        let mut content = std::fs::read_to_string(&file).ok()?;
        if let Some(name) = region {
            content = extract_region(&content, &name)?;
        }
        let mut ext = file
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_default();
        // .ansi keeps its SGR escape sequences: the codefence renderer
        // walks them into classed spans (Shiki's `ansi` grammar upstream)
        if let Some(l) = lang_switch {
            ext = l;
        }
        let marker = fence_marker_for(&content);
        let mut block = String::new();
        block.push_str(&format!("{marker}{ext}"));
        // the suffix must ride on the lang token: rewrite_info strips
        // `:line-numbers` only there
        if ln {
            block.push_str(":line-numbers");
        }
        if let Some(spec) = &hl {
            block.push_str(&format!("{{{spec}}}"));
        }
        // "filename is used as title by default" (upstream snippet
        // includes): an unlabeled include takes the file's name as its
        // code-group tab / standalone title bar
        let label = label.or_else(|| {
            file.file_name()
                .map(|n| n.to_string_lossy().into_owned())
        });
        if let Some(label) = label {
            block.push_str(&format!(" [{label}]"));
        }
        block.push('\n');
        block.push_str(&content);
        if !content.ends_with('\n') {
            block.push('\n');
        }
        block.push_str(&marker);
        block.push('\n');
        Some(block)
    }
}

/// `<!--@include: ./file.md-->` target: `path`, optional `#section`
/// (VS Code region name or heading anchor), optional `{a,b}` line
/// range — `path#section{range}` in that order (upstream syntax).
pub struct IncludeTarget {
    pub path: String,
    pub section: Option<String>,
    pub range: Option<String>,
}

impl IncludeTarget {
    pub fn raw_path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}

fn md_include_target(line: &str) -> Option<IncludeTarget> {
    let t = line.trim();
    let inner = t
        .strip_prefix("<!--@include:")
        .and_then(|r| r.strip_suffix("-->"))?
        .trim();
    if inner.is_empty() {
        return None;
    }
    let (rest, range) = match inner.rfind('{') {
        Some(i) if inner.ends_with('}') && inner[i..].contains(',') => {
            (&inner[..i], Some(inner[i + 1..inner.len() - 1].to_string()))
        }
        _ => (inner, None),
    };
    let (path, section) = match rest.find('#') {
        Some(i) => (&rest[..i], Some(rest[i + 1..].to_string())),
        None => (rest, None),
    };
    if path.is_empty() {
        return None;
    }
    Some(IncludeTarget {
        path: path.to_string(),
        section,
        range,
    })
}

/// `{a,b}` / `{a,}` / `{,b}` / `{a,b-c}` 1-based line selection for
/// markdown includes (upstream "Markdown File Inclusion").
fn apply_line_range(content: &str, spec: &str) -> String {
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let pick = |nums: &mut dyn Iterator<Item = usize>| {
        let mut out = String::new();
        for n in nums {
            if let Some(l) = lines.get(n.saturating_sub(1)) {
                out.push_str(l);
            }
        }
        out
    };
    let (a, b) = spec.split_once(',').unwrap_or((spec, ""));
    let a: Option<usize> = a.trim().parse().ok();
    let b: Option<usize> = b.trim().parse().ok();
    let from = a.unwrap_or(1);
    let to = b.unwrap_or(lines.len()).min(lines.len());
    if from == 0 || to < from {
        return String::new();
    }
    pick(&mut (from..=to))
}

/// Heading-anchor section extraction: from the heading whose auto slug
/// (or explicit `{#id}`) matches, up to (not including) the next
/// heading of the same or higher level. Fence-aware.
fn extract_heading_section(content: &str, anchor: &str) -> Option<String> {
    fn slugify(text: &str) -> String {
        let mut out = String::new();
        for ch in text.chars() {
            match ch {
                ' ' => out.push('-'),
                c if c.is_ascii_alphanumeric() || c == '-' || c == '_' => {
                    out.push(c.to_ascii_lowercase())
                }
                _ => {}
            }
        }
        out.trim_matches('-').to_string()
    }
    let mut fence: Option<(char, usize)> = None;
    let mut level = 0usize;
    let mut started = false;
    let mut out = String::new();
    for line in content.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if let Some((ch, n)) = fence {
            if started {
                out.push_str(line);
            }
            if is_closing_fence(bare, ch, n) {
                fence = None;
            }
            continue;
        }
        if let Some((ch, n)) = opening_fence(bare) {
            if started {
                out.push_str(line);
            }
            fence = Some((ch, n));
            continue;
        }
        let t = bare.trim_start();
        if let Some(rest) = t.strip_prefix('#') {
            let hashes = rest.chars().take_while(|&c| c == '#').count() + 1;
            let text = t[hashes..].trim();
            if started {
                if hashes <= level {
                    return Some(out);
                }
                out.push_str(line);
            } else {
                let explicit = text
                    .rfind("{#")
                    .filter(|_| text.ends_with('}'))
                    .map(|i| text[i + 2..text.len() - 1].to_string());
                let plain = text.split("{#").next().unwrap_or(text).trim();
                if explicit.as_deref() == Some(anchor) || slugify(plain) == anchor {
                    level = hashes;
                    started = true;
                    out.push_str(line);
                }
            }
            continue;
        }
        if started {
            out.push_str(line);
        }
    }
    started.then_some(out)
}

fn extract_region(content: &str, name: &str) -> Option<String> {
    let mut out = String::new();
    let mut inside = false;
    for line in content.split_inclusive('\n') {
        let l = line;
        if !inside && l.contains("#region") && l.contains(name) {
            inside = true;
            continue;
        }
        if inside && l.contains("#endregion") {
            return Some(out);
        }
        if inside {
            out.push_str(l);
        }
    }
    if inside {
        Some(out)
    } else {
        None
    }
}

/// A fence marker longer than any backtick run inside the content.
fn fence_marker_for(content: &str) -> String {
    let mut longest = 2usize;
    let mut run = 0usize;
    for ch in content.chars() {
        if ch == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    "`".repeat(longest + 1)
}

/// Pass 1.6: inline footnotes `^[content]` (comrak only knows reference
/// footnotes) → `[^fni-N]` references with the definitions appended at
/// the end of the document, where comrak renders them like any other
/// footnote. Fence- and inline-code-span aware.
pub fn expand_inline_footnotes(md: &str) -> String {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\^\[([^\[\]]+)\]").unwrap());
    let mut fence: Option<(char, usize)> = None;
    let mut counter = 0usize;
    let mut defs: Vec<String> = Vec::new();
    let mut out = String::with_capacity(md.len());
    for line in md.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if let Some((ch, n)) = fence {
            out.push_str(line);
            if is_closing_fence(bare, ch, n) {
                fence = None;
            }
            continue;
        }
        if let Some((ch, n)) = opening_fence(bare) {
            out.push_str(line);
            fence = Some((ch, n));
            continue;
        }
        // skip matches inside inline code spans on this line
        let mut replaced = String::new();
        let mut rest = bare;
        while let Some(i) = rest.find('`') {
            let (before, after) = rest.split_at(i);
            let span_end = after[1..].find('`').map(|j| j + 2);
            let code = span_end.map(|e| &after[..e]);
            replaced.push_str(&replace_inline(before, re, &mut counter, &mut defs));
            match code {
                Some(c) => {
                    replaced.push_str(c);
                    rest = &after[c.len()..];
                }
                None => {
                    // unclosed backtick: the rest is literal
                    replaced.push_str(after);
                    rest = "";
                }
            }
        }
        replaced.push_str(&replace_inline(rest, re, &mut counter, &mut defs));
        out.push_str(&replaced);
        out.push('\n');
    }
    if defs.is_empty() {
        return out;
    }
    out.push('\n');
    for d in &defs {
        out.push_str(d);
        out.push('\n');
    }
    out
}

fn replace_inline(
    text: &str,
    re: &Regex,
    counter: &mut usize,
    defs: &mut Vec<String>,
) -> String {
    re.replace_all(text, |c: &regex::Captures| {
        *counter += 1;
        let label = format!("fni-{counter}");
        defs.push(format!("[^{label}]: {}", &c[1]));
        format!("[^{label}]")
    })
    .into_owned()
}

/// Pass 1.7: link attribute blocks — `[text](url){target="_self" …}`
/// (upstream's @mdit/plugin-attrs link form). Rewritten to a raw
/// anchor, so markdown inside the link text is not re-parsed. Fence-
/// and inline-code-span aware.
pub fn rewrite_link_attrs(md: &str, base: &str) -> String {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        // [text](url){k="v" k2="v2"} — attrs must contain =" to avoid
        // eating unrelated braces right after links
        Regex::new(r#"(?m)(!?)\[([^\]\n]*)\]\(([^)\s]+)\)\{([^{}]*="[^}"]*"[^{}]*)\}"#).unwrap()
    });
    fn esc(s: &str) -> String {
        s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;")
    }
    let mut fence: Option<(char, usize)> = None;
    let mut out = String::with_capacity(md.len());
    for line in md.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if let Some((ch, n)) = fence {
            out.push_str(line);
            if is_closing_fence(bare, ch, n) {
                fence = None;
            }
            continue;
        }
        if let Some((ch, n)) = opening_fence(bare) {
            out.push_str(line);
            fence = Some((ch, n));
            continue;
        }
        // leave lines whose only {…} follow code spans alone: split the
        // line at inline code spans and only rewrite outside them
        let mut replaced = String::new();
        let mut rest = bare;
        while let Some(i) = rest.find('`') {
            let (before, after) = rest.split_at(i);
            let span_end = after[1..].find('`').map(|j| j + 2);
            replaced.push_str(&re.replace_all(before, |c: &regex::Captures| {
                format!(
                    r#"<a href="{}"{}>{}</a>"#,
                    esc(&super::with_base(base, &c[3])),
                    parse_attrs(&c[4]),
                    esc(&c[2])
                )
            }));
            match span_end.map(|e| &after[..e]) {
                Some(code) => {
                    replaced.push_str(code);
                    rest = &after[code.len()..];
                }
                None => {
                    replaced.push_str(after);
                    rest = "";
                }
            }
        }
        replaced.push_str(&re.replace_all(rest, |c: &regex::Captures| {
            format!(
                r#"<a href="{}"{}>{}</a>"#,
                esc(&super::with_base(base, &c[3])),
                parse_attrs(&c[4]),
                esc(&c[2])
            )
        }));
        out.push_str(&replaced);
        out.push('\n');
    }
    out
}

/// `k="v" k2="v2"` → ` k="v" k2="v2"` with sanitized names/values.
fn parse_attrs(raw: &str) -> String {
    static PAIR: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let pair = PAIR.get_or_init(|| Regex::new(r#"([a-zA-Z-]+)="([^"]*)""#).unwrap());
    let mut out = String::new();
    for c in pair.captures_iter(raw) {
        out.push_str(&format!(r#" {}="{}""#, &c[1], c[2].replace('"', "&quot;")));
    }
    out
}
/// Pass 1.5: `> [!KIND] [title]` GitHub-flavored alerts → `::: kind`
/// containers, which is exactly what VitePress does (alerts ARE
/// containers there): same styling, same `[markdown.container]` labels,
/// and `[!DANGER]` plus registered custom kinds work. Fence-aware —
/// example alerts inside code fences stay literal.
pub fn expand_alerts(md: &str, opts: &ContainerOptions) -> String {
    let mut out_lines: Vec<String> = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let lines: Vec<&str> = md.split_inclusive('\n').collect();
    let mut i = 0usize;
    while i < lines.len() {
        let bare = lines[i].trim_end_matches(['\n', '\r']);
        if let Some((ch, n)) = fence {
            out_lines.push(bare.to_string());
            if is_closing_fence(bare, ch, n) {
                fence = None;
            }
            i += 1;
            continue;
        }
        if let Some((ch, n)) = opening_fence(bare) {
            out_lines.push(bare.to_string());
            fence = Some((ch, n));
            i += 1;
            continue;
        }
        if let Some((indent, kind, title)) = alert_opener(bare, opts) {
            let title_suffix = if title.is_empty() {
                String::new()
            } else {
                format!(" {title}")
            };
            out_lines.push(format!("{indent}::: {kind}{title_suffix}"));
            i += 1;
            // Body: the blockquote's `>`-prefixed lines (a `>`-only line
            // is a blank line *inside* the quote and continues it; any
            // non-`>` line ends the quote).
            while i < lines.len() {
                let b = lines[i].trim_end_matches(['\n', '\r']);
                let t = b.trim_start();
                let ind = &b[..b.len() - t.len()];
                if let Some(rest) = t.strip_prefix('>') {
                    let rest = rest.strip_prefix(' ').unwrap_or(rest);
                    out_lines.push(format!("{ind}{rest}"));
                    i += 1;
                } else {
                    break;
                }
            }
            out_lines.push(format!("{indent}:::"));
            out_lines.push(String::new());
            continue;
        }
        out_lines.push(bare.to_string());
        i += 1;
    }
    let mut out = out_lines.join("\n");
    out.push('\n');
    out
}

/// `> [!KIND] [title]` → (indent, kind, title). Recognized kinds: the
/// builtin container kinds (incl. VitePress's `[!DANGER]` extension)
/// and registered custom containers — upstream lets registered
/// containers appear as alerts too.
fn alert_opener(line: &str, opts: &ContainerOptions) -> Option<(String, String, String)> {
    let t = line.trim_start();
    let indent = &line[..line.len() - t.len()];
    let rest = t.strip_prefix('>')?.strip_prefix(' ').unwrap_or("").trim();
    if !rest.starts_with("[!") {
        return None;
    }
    let end = rest.find(']')?;
    let kind = rest[2..end].trim().to_lowercase();
    if kind.is_empty() || !kind.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return None;
    }
    let known = matches!(
        kind.as_str(),
        "note" | "tip" | "important" | "warning" | "caution" | "danger" | "info"
    ) || opts.custom.iter().any(|c| c.name.eq_ignore_ascii_case(&kind));
    if !known {
        return None;
    }
    let title = rest[end + 1..].trim();
    Some((indent.to_string(), kind, title.to_string()))
}

/// Pass 2: `:::` containers.
pub fn expand_containers(md: &str, opts: &ContainerOptions) -> String {
    #[derive(Clone, Copy)]
    enum Open {
        Tip,
        Details,
        CodeGroup,
        VPre,
        Raw,
    }
    let mut out_lines: Vec<String> = Vec::new();
    let mut stack: Vec<(usize, Open)> = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut group_counter = 0usize;

    // Containers opened inside a list item keep the item's indent on
    // every emitted line — at column 0 the HTML block would interrupt
    // the list (markdown-it keeps the container inside the item).
    let close_markup = |open: Open, indent: &str, out: &mut Vec<String>| match open {
        Open::Tip => out.extend(["".into(), format!("{indent}</div>"), "".into()]),
        Open::Details => out.extend(["".into(), format!("{indent}</details>"), "".into()]),
        Open::CodeGroup => out.extend([
            "".into(),
            format!("{indent}</div>"),
            format!("{indent}</div>"),
            "".into(),
        ]),
        Open::Raw => out.extend(["".into(), format!("{indent}</div>"), "".into()]),
        Open::VPre => {}
    };

    let lines: Vec<&str> = md.split_inclusive('\n').collect();
    for (idx, line) in lines.iter().enumerate() {
        let bare = line.trim_end_matches(['\n', '\r']);
        let trimmed = bare.trim_start();
        let indent = &bare[..bare.len() - trimmed.len()];
        if let Some((ch, n)) = fence {
            out_lines.push(bare.to_string());
            if is_closing_fence(bare, ch, n) {
                fence = None;
            }
            continue;
        }
        if let Some((ch, n)) = opening_fence(bare) {
            // fences inside ::: code-group get a group marker: it tells
            // the fence renderer their label is a tab name, not a
            // standalone block title
            if stack.iter().any(|(_, o)| matches!(o, Open::CodeGroup)) {
                out_lines.push(format!("{bare} {GD_GROUP_TOKEN}"));
            } else {
                out_lines.push(bare.to_string());
            }
            fence = Some((ch, n));
            continue;
        }
        let t = trimmed.trim_end();
        if t == "[[toc]]" {
            out_lines.push("<!--gd-toc-->".into());
            continue;
        }
        // closing `:::` (3+ colons, nothing else): closes the innermost
        // container opened with ≤ that many colons
        if let Some(colons) = closing_container(t) {
            if let Some(&(top_colons, _)) = stack.last()
                && colons >= top_colons {
                    let (_, open) = stack.pop().unwrap();
                    close_markup(open, indent, &mut out_lines);
                    continue;
                }
            out_lines.push(bare.to_string());
            continue;
        }
        if let Some((colons, kind, rest)) = opening_container(t) {
            let rest = rest.trim();
            match kind {
                "v-pre" => {
                    stack.push((colons, Open::VPre));
                    continue; // markers vanish; content passes through
                }
                "raw" => {
                    // upstream's style-isolation wrapper for embedded
                    // component demos (vp-raw class)
                    stack.push((colons, Open::Raw));
                    out_lines.extend([
                        format!("{indent}<div class=\"vp-raw\">"),
                        String::new(),
                    ]);
                }
                "code-group" => {
                    stack.push((colons, Open::CodeGroup));
                    group_counter += 1;
                    let tabs = code_group_tabs(&lines, idx, group_counter);
                    out_lines.extend([
                        format!("{indent}<div class=\"vp-code-group\" x-data=\"codeGroup\">"),
                        format!("{indent}{tabs}"),
                        format!("{indent}<div class=\"blocks\">"),
                        String::new(),
                    ]);
                }
                "details" => {
                    let (summary, open) = parse_details_rest(rest);
                    let summary =
                        if summary.is_empty() { opts.label_for("details") } else { summary };
                    stack.push((colons, Open::Details));
                    let open_attr = if open { " open" } else { "" };
                    out_lines.extend([
                        format!("{indent}<details class=\"custom-block details\"{open_attr}>"),
                        format!("{indent}<summary>{}</summary>", escape_text(&summary)),
                        String::new(),
                    ]);
                }
                kind => {
                    // custom containers reuse a builtin kind's styling
                    let (class, default_label) =
                        if let Some(cc) = opts.custom.iter().find(|c| c.name == kind) {
                            (
                                cc.kind.clone().unwrap_or_else(|| "tip".into()),
                                cc.label.clone().unwrap_or_else(|| kind.to_uppercase()),
                            )
                        } else if matches!(
                            kind,
                            "tip" | "warning" | "danger" | "note" | "info" | "important" | "caution"
                        ) {
                            (kind.to_string(), opts.label_for(kind))
                        } else {
                            out_lines.push(bare.to_string());
                            continue;
                        };
                    stack.push((colons, Open::Tip));
                    let (no_title, title) = parse_tip_rest(rest);
                    out_lines.push(format!("{indent}<div class=\"custom-block {class}\">"));
                    if !no_title {
                        let title = title.unwrap_or(default_label);
                        out_lines.push(format!(
                            "{indent}<p class=\"custom-block-title\">{}</p>",
                            escape_text(&title)
                        ));
                    }
                    out_lines.push(String::new());
                }
            }
            continue;
        }
        out_lines.push(bare.to_string());
    }
    while let Some((_, open)) = stack.pop() {
        close_markup(open, "", &mut out_lines);
    }
    let mut out = out_lines.join("\n");
    out.push('\n');
    out
}

/// Server-side tab strip for a `::: code-group` opened at `lines[idx]`:
/// scan ahead for the group's fenced blocks and build radio inputs +
/// labels from their `[label]` annotations (falling back to the fence
/// language). Label click wiring is the `codeGroup` Alpine component.
fn code_group_tabs(lines: &[&str], idx: usize, group_no: usize) -> String {
    let mut labels: Vec<String> = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for line in lines.iter().skip(idx + 1) {
        let bare = line.trim_end_matches(['\n', '\r']);
        if let Some((ch, n)) = fence {
            if is_closing_fence(bare, ch, n) {
                fence = None;
            }
            continue;
        }
        if let Some((ch, n)) = opening_fence(bare) {
            let info = bare[fence_info_offset(bare, n)..].trim();
            let label = info
                .find('[')
                .and_then(|i| info[i + 1..].find(']').map(|j| info[i + 1..i + 1 + j].to_string()));
            let lang = info.split_whitespace().next().unwrap_or("").to_string();
            labels.push(label.filter(|l| !l.is_empty()).unwrap_or(lang));
            fence = Some((ch, n));
            continue;
        }
        // stop at the container closer (any ::: line)
        if closing_container(bare.trim()).is_some() || opening_container(bare.trim()).is_some() {
            break;
        }
    }
    let mut tabs = String::from("<div class=\"tabs\">");
    for (i, label) in labels.iter().enumerate() {
        let checked = if i == 0 { " checked=\"checked\"" } else { "" };
        tabs.push_str(&format!(
            "<input type=\"radio\" name=\"group-{group_no}\" id=\"group-{group_no}-{i}\"{checked}>"
        ));
        tabs.push_str(&format!(
            "<label for=\"group-{group_no}-{i}\">{}</label>",
            escape_text(label)
        ));
    }
    tabs.push_str("</div>");
    tabs
}

/// `::::` / `::: ` opener: (colons, kind, rest-after-kind).
fn opening_container(t: &str) -> Option<(usize, &str, &str)> {    let colons = t.chars().take_while(|&c| c == ':').count();
    if colons < 3 {
        return None;
    }
    let rest = t[colons..].trim();
    if rest.is_empty() {
        return None;
    }
    let (kind, tail) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    Some((colons, kind, tail))
}

fn closing_container(t: &str) -> Option<usize> {
    let colons = t.chars().take_while(|&c| c == ':').count();
    if colons >= 3 && t[colons..].trim().is_empty() {
        Some(colons)
    } else {
        None
    }
}

/// `TITLE {no-title}` → (no_title, Some(custom title)).
fn parse_tip_rest(rest: &str) -> (bool, Option<String>) {
    if let Some(stripped) = rest.strip_suffix("{no-title}") {
        let title = stripped.trim();
        return (true, (!title.is_empty()).then(|| title.to_string()));
    }
    (false, (!rest.is_empty()).then(|| rest.to_string()))
}

/// `SUMMARY [{open}]` → (summary, open). VitePress also tolerates
/// `{open}` with spaces inside the braces.
fn parse_details_rest(rest: &str) -> (String, bool) {
    if rest.ends_with('}') && rest.contains('{')
        && let Some(i) = rest.rfind('{') {
            let attr = rest[i + 1..rest.len() - 1].trim();
            if attr == "open" || attr.contains("open") {
                return (rest[..i].trim().to_string(), true);
            }
        }
    if rest.is_empty() {
        (String::new(), false)
    } else {
        (rest.to_string(), false)
    }
}

/// Pass 3: fence info strings → `gdcode lang=… [hl=…] [label=…]`.
pub fn rewrite_fences(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    let mut fence: Option<(char, usize)> = None; // marker, count
    for line in md.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if let Some((ch, n)) = fence {
            out.push_str(bare);
            out.push('\n');
            if is_closing_fence(bare, ch, n) {
                fence = None;
            }
            continue;
        }
        if let Some((ch, n)) = opening_fence(bare) {
            let cut = fence_info_offset(bare, n);
            let info = &bare[cut..];
            let new_line = format!("{}{}", &bare[..cut], rewrite_info(info.trim()));
            out.push_str(&new_line);
            out.push('\n');
            fence = Some((ch, n));
            continue;
        }
        out.push_str(bare);
        out.push('\n');
    }
    out
}

/// `` js{1,3-4} [npm] `` → `` gdcode lang=js hl=1,3-4 label=npm ``.
fn rewrite_info(info: &str) -> String {
    if info.is_empty() {
        return FENCE_LANG.to_string();
    }
    let mut label: Option<String> = None;
    let mut hl: Option<String> = None;
    let mut rest = info.to_string();

    // [label] — first bracket group
    if let Some(i) = rest.find('[')
        && let Some(j) = rest[i..].find(']') {
            label = Some(rest[i + 1..i + j].to_string());
            rest.replace_range(i..i + j + 1, " ");
        }
    // GD_GROUP_TOKEN — the containers pass tags fences inside :::
    // code-group. Token equality, not substring: a bare "```" opener in
    // a group trims to exactly the token, with no leading space.
    let mut group = false;
    if rest.split_whitespace().any(|t| t == GD_GROUP_TOKEN) {
        group = true;
        rest = rest
            .split_whitespace()
            .filter(|t| *t != GD_GROUP_TOKEN)
            .collect::<Vec<_>>()
            .join(" ");
    }
    // {spec} — first brace group that looks like a line spec
    if let Some(i) = rest.find('{')
        && let Some(j) = rest[i..].find('}') {
            let inner = &rest[i + 1..i + j];
            let compact: String = inner.chars().filter(|c| !c.is_whitespace()).collect();
            if !compact.is_empty()
                && compact
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == ',' || c == '-')
            {
                hl = Some(compact);
                rest.replace_range(i..i + j + 1, " ");
            }
        }
    // :line-numbers / :no-line-numbers / :line-numbers=N → ln= token
    let mut lang = rest.split_whitespace().next().unwrap_or("").to_string();
    let mut ln: Option<String> = None;
    for (suffix, token) in [
        (":no-line-numbers", "false"),
        (":line-numbers", "true"),
    ] {
        if let Some(stripped) = lang.strip_suffix(suffix) {
            lang = stripped.to_string();
            ln = Some(token.to_string());
        }
    }
    if let Some(i) = lang.find(":line-numbers=") {
        ln = Some(lang[i + ":line-numbers=".len()..].to_string());
        lang.truncate(i);
    }
    lang = lang.trim_matches(':').to_string();

    // an empty lang (a bare "```" fence, possibly with a group marker)
    // stays absent: parse_meta defaults to "" and the renderer emits
    // upstream's `language-` class
    let mut out = if lang.is_empty() {
        FENCE_LANG.to_string()
    } else {
        format!("{FENCE_LANG} lang={lang}")
    };
    if group {
        out.push_str(" group=1");
    }
    if let Some(hl) = hl {
        out.push_str(&format!(" hl={hl}"));
    }
    if let Some(v) = ln {
        out.push_str(&format!(" ln={v}"));
    }
    // an empty label is no label (upstream's group tabs filter it too)
    if let Some(label) = label.filter(|l| !l.is_empty()) {
        out.push_str(&format!(" label={label}"));
    }
    out
}

/// Pass 4: `<Badge type="warning" text="experimental" />` and
/// `<Badge type="info">children</Badge>` → `<span class="VPBadge …">`.
pub fn rewrite_badges(md: &str) -> String {
    static SELF_CLOSING: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    static PAIRED: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let self_closing = SELF_CLOSING.get_or_init(|| Regex::new(r#"<Badge\s+([^<>]*?)\s*/>"#).unwrap());
    let paired = PAIRED.get_or_init(|| Regex::new(r#"(?s)<Badge\s+([^<>]*?)>(.*?)</Badge>"#).unwrap());

    let mut out = String::with_capacity(md.len());
    let mut fence: Option<(char, usize)> = None;
    for line in md.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if let Some((ch, n)) = fence {
            out.push_str(line);
            if is_closing_fence(bare, ch, n) {
                fence = None;
            }
            continue;
        }
        if let Some((ch, n)) = opening_fence(bare) {
            out.push_str(line);
            fence = Some((ch, n));
            continue;
        }
        let self_done = self_closing.replace_all(bare, |c: &regex::Captures| {
            let (ty, text) = badge_attrs(&c[1]);
            format!("<span class=\"VPBadge {ty}\">{}</span>", escape_text(&text))
        });
        let replaced = paired
            .replace_all(&self_done, |c: &regex::Captures| {
                let (ty, _) = badge_attrs(&c[1]);
                format!("<span class=\"VPBadge {ty}\">{}</span>", c[2].trim())
            })
            .into_owned();
        out.push_str(&replaced);
        out.push('\n');
    }
    out
}

fn badge_attrs(attr_str: &str) -> (String, String) {
    static ATTR: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = ATTR.get_or_init(|| Regex::new(r#"(\w+)\s*=\s*"([^"]*)""#).unwrap());
    let mut ty = String::from("tip");
    let mut text = String::new();
    for cap in re.captures_iter(attr_str) {
        match &cap[1] {
            "type" => ty = cap[2].to_string(),
            "text" => text = cap[2].to_string(),
            _ => {}
        }
    }
    (ty, text)
}

// ── fence scanning helpers ─────────────────────────────────────────────

fn opening_fence(line: &str) -> Option<(char, usize)> {
    let t = line.trim_start();
    let first = t.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let n = t.chars().take_while(|&c| c == first).count();
    if n >= 3 && t[n..].trim() == "" {
        // bare fence (opening or closing — caller decides); an info
        // string means definitely opening
        Some((first, n))
    } else if n >= 3 {
        Some((first, n))
    } else {
        None
    }
}

fn is_closing_fence(line: &str, ch: char, n: usize) -> bool {
    let t = line.trim();
    let count = t.chars().take_while(|&c| c == ch).count();
    count >= n && t[count..].trim().is_empty()
}

/// Byte offset just past the opening fence marker in `line` (leading
/// indent + marker run). Slicing the raw line by the marker count alone
/// is only correct at column 0 — inside a list item the indent would
/// swallow the backticks and corrupt the fence.
fn fence_info_offset(line: &str, count: usize) -> usize {
    line.len() - line.trim_start().len() + count
}

pub fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn containers(md: &str) -> String {
        expand_containers(md, &ContainerOptions::default())
    }

    #[test]
    fn tip_container_with_default_and_custom_title() {
        let out = containers("::: tip\nhi **bold**\n:::\n");
        assert!(out.contains("<div class=\"custom-block tip\">"), "{out}");
        assert!(out.contains("<p class=\"custom-block-title\">TIP</p>"));
        assert!(out.contains("hi **bold**"));
        let out = containers("::: warning SERVER REQUIRED\nx\n:::\n");
        assert!(out.contains("<p class=\"custom-block-title\">SERVER REQUIRED</p>"), "{out}");
        let out = containers("::: tip {no-title}\nx\n:::\n");
        assert!(!out.contains("custom-block-title"), "{out}");
    }

    #[test]
    fn details_and_code_group() {
        let out = containers("::: details Click me {open}\nx\n:::\n");
        assert!(out.contains("<details class=\"custom-block details\" open>"), "{out}");
        assert!(out.contains("<summary>Click me</summary>"));
        let out = containers("::: code-group\n```js [a]\n1\n```\n:::\n");
        assert!(out.contains("<div class=\"vp-code-group\" x-data=\"codeGroup\">"), "{out}");
        assert!(out.contains("<div class=\"blocks\">"));
        assert!(out.contains("</div>\n</div>"));
    }

    #[test]
    fn nested_containers_stack() {
        let out = containers(":::: info\nouter\n\n::: tip\ninner\n:::\n\n::::\n");
        let outer = out.find("<div class=\"custom-block info\">").unwrap();
        let inner = out.find("<div class=\"custom-block tip\">").unwrap();
        let inner_close = out.find("</div>").unwrap();
        assert!(outer < inner && inner < inner_close, "{out}");
        assert_eq!(out.matches("</div>").count(), 2);
    }

    #[test]
    fn containers_inside_fences_untouched() {
        let out = containers("````md\n::: tip\nliteral\n:::\n````\n");
        assert!(!out.contains("custom-block"), "{out}");
        assert!(out.contains("::: tip"));
    }

    #[test]
    fn fence_info_rewrites() {
        assert_eq!(rewrite_info("js"), "gdcode lang=js");
        assert_eq!(rewrite_info("js [npm]"), "gdcode lang=js label=npm");
        assert_eq!(rewrite_info("js{1,3-4}"), "gdcode lang=js hl=1,3-4");
        assert_eq!(rewrite_info("js {4}"), "gdcode lang=js hl=4");
        assert_eq!(rewrite_info("md:line-numbers"), "gdcode lang=md ln=true");
        assert_eq!(rewrite_info("ts:no-line-numbers"), "gdcode lang=ts ln=false");
        assert_eq!(rewrite_info("md:line-numbers=4"), "gdcode lang=md ln=4");
        assert_eq!(rewrite_info("vue [Layout.vue]"), "gdcode lang=vue label=Layout.vue");
        assert_eq!(rewrite_info(""), "gdcode");
        // non-numeric braces are not line specs: left alone
        assert_eq!(rewrite_info("jsonc { \"n\": 2 }"), "gdcode lang=jsonc");
    }

    #[test]
    fn fenced_example_fences_not_rewritten() {
        let md = "````md\n```js [x]\n1\n```\n````\n";
        let out = rewrite_fences(md);
        // the OUTER fence is real and gets the sentinel; the example
        // fence inside stays literal
        assert!(out.contains("```js [x]"), "{out}");
        assert_eq!(out.matches("gdcode").count(), 1, "{out}");
    }

    #[test]
    fn indented_fences_keep_their_marker() {
        // fences inside list items are indented; the marker must survive
        // the rewrite (regression: the indent+count mixup dropped the
        // backticks and turned the fence into paragraph text)
        let md = "1. step:\n\n   ```json [package.json]\n   { \"a\": 1 }\n   ```\n";
        let out = rewrite_fences(md);
        assert!(
            out.contains("   ```gdcode lang=json label=package.json"),
            "{out}"
        );
        assert!(out.lines().any(|l| l == "   ```"), "{out}");
    }

    #[test]
    fn badge_rewrites() {
        let out = rewrite_badges("## `useData` <Badge type=\"info\" text=\"composable\" />\n");
        assert!(out.contains("<span class=\"VPBadge info\">composable</span>"), "{out}");
        let out = rewrite_badges("<Badge type=\"warning\">careful</Badge>\n");
        assert!(out.contains("<span class=\"VPBadge warning\">careful</span>"), "{out}");
        let out = rewrite_badges("<Badge text=\"plain\" />\n");
        assert!(out.contains("<span class=\"VPBadge tip\">plain</span>"), "{out}");
        // inside fences: literal
        let out = rewrite_badges("```md\n<Badge type=\"info\" text=\"x\" />\n```\n");
        assert!(out.contains("<Badge"), "{out}");
    }

    #[test]
    fn ansi_fences_keep_their_escapes() {
        // the renderer walks SGR sequences into classed spans; stripping
        // them here would render the block monochrome
        let out = Preprocess {
            site_root: Path::new("tests/fixtures"),
            content_dir: Path::new("tests/fixtures/en"),
            container: crate::config::ContainerOptions::default(),
            base: "/",
        }
        .run("```ansi\n\x1b[32mok\x1b[0m\n```\n", "guide/x.md")
        .unwrap();
        assert!(out.contains("\x1b[32m"), "{out}");
    }

    #[test]
    fn ansi_and_region_extraction() {
        let src = "// #region setup\nconst a = 1;\n// #endregion\nrest\n";
        assert_eq!(extract_region(src, "setup").unwrap(), "const a = 1;\n");
    }


    #[test]
    fn md_include_target_parses() {
        // "./"-prefixed paths join fine as-is
        let t = md_include_target("<!--@include: ./parts/x.md-->").unwrap();
        assert_eq!(
            (t.path.as_str(), t.section.is_none(), t.range.is_none()),
            ("./parts/x.md", true, true)
        );
        let t = md_include_target("<!--@include: parts/x.md-->").unwrap();
        assert_eq!(t.path, "parts/x.md");
        // section + range in upstream order: path#section{range}
        let t = md_include_target("<!--@include: parts/x.md#basic-usage{,2}-->").unwrap();
        assert_eq!(t.path, "parts/x.md");
        assert_eq!(t.section.as_deref(), Some("basic-usage"));
        assert_eq!(t.range.as_deref(), Some(",2"));
        let t = md_include_target("<!--@include: parts/x.md{3,}-->").unwrap();
        assert_eq!(t.range.as_deref(), Some("3,"));
    }
}

#[cfg(test)]
mod edge_tests {
    use super::*;

    #[test]
    fn container_directly_after_paragraph_no_blank() {
        let out = expand_containers("a paragraph\n::: tip\ninner\n:::\nafter\n", &ContainerOptions::default());
        // the HTML block must interrupt the paragraph correctly at the
        // comrak layer; here we check the wrapper survives preprocessing
        assert!(out.contains("<div class=\"custom-block tip\">"), "{out}");
        assert!(out.contains("</div>"));
    }

    #[test]
    fn adjacent_containers() {
        let out = expand_containers("::: tip\na\n:::\n::: warning\nb\n:::\n", &ContainerOptions::default());
        assert_eq!(out.matches("<div class=\"custom-block").count(), 2, "{out}");
        assert_eq!(out.matches("</div>").count(), 2, "{out}");
    }

    #[test]
    fn tilde_fences_protected() {
        let out = expand_containers("~~~\n::: tip\n~~~\n::: info\nreal\n:::\n", &ContainerOptions::default());
        // ::: inside the tilde fence is literal; the real one expands
        assert!(out.contains("::: tip"), "{out}");
        assert!(out.contains("<div class=\"custom-block info\">"), "{out}");
    }

    #[test]
    fn fence_with_trailing_spaces_and_info_closer() {
        // closer with trailing spaces; opener with info string
        let out = expand_containers("```js \n::: tip\n``` \n", &ContainerOptions::default());
        assert!(out.contains("::: tip"), "container inside fence stays literal: {out}");
    }

    #[test]
    fn unbalanced_container_closed_at_eof() {
        let out = expand_containers("::: tip\nnever closed\n", &ContainerOptions::default());
        assert_eq!(out.matches("<div").count(), out.matches("</div>").count(), "{out}");
    }

    #[test]
    fn info_rewrites_edge_forms() {
        assert_eq!(rewrite_info("vue{1-2}"), "gdcode lang=vue hl=1-2");
        assert_eq!(rewrite_info("sh [npm]"), "gdcode lang=sh label=npm");
        // label containing ']' — first bracket group only
        assert_eq!(
            rewrite_info("js [a[b]]"),
            "gdcode lang=js label=a[b"
        );
        // lang with dashes/dots survives
        assert_eq!(rewrite_info("objective-c++"), "gdcode lang=objective-c++");
        // :line-numbers=N mid-token
        assert_eq!(rewrite_info("md:line-numbers=2"), "gdcode lang=md ln=2");
    }
}

#[cfg(test)]
mod code_group_tests {
    use super::*;

    #[test]
    fn code_group_tabs_built_from_fence_labels() {
        let out = expand_containers(
            "::: code-group\n```sh [npm]\n1\n```\n```sh [pnpm]\n2\n```\n:::\n",
            &ContainerOptions::default(),
        );
        assert!(out.contains("<div class=\"vp-code-group\" x-data=\"codeGroup\">"), "{out}");
        assert!(out.contains("<div class=\"tabs\">"), "{out}");
        assert!(out.contains("name=\"group-1\""), "{out}");
        assert!(out.contains("<label for=\"group-1-0\">npm</label>"), "{out}");
        assert!(out.contains("<label for=\"group-1-1\">pnpm</label>"), "{out}");
        assert!(out.contains("checked=\"checked\""), "{out}");
        // labels without [label] fall back to the fence language
        let out2 = expand_containers("::: code-group\n```js\n1\n```\n:::\n", &ContainerOptions::default());
        assert!(out2.contains("<label for=\"group-1-0\">js</label>"), "{out2}");
    }
}

#[cfg(test)]
mod include_and_container_tests {
    use super::*;
    use crate::config::{ContainerOptions, CustomContainer};
    use std::path::Path;

    fn pre() -> Preprocess<'static> {
        Preprocess {
            site_root: Path::new("tests/fixtures"),
            content_dir: Path::new("tests/fixtures/en"),
            container: ContainerOptions::default(),
            base: "/",
        }
    }

    #[test]
    fn include_with_line_highlight() {
        let out = pre()
            .run("<<< @/snippets/snippet.js{2}\n", "guide/x.md")
            .unwrap();
        // upstream semantics: {2} highlights line 2 and every line is
        // rendered; the unlabeled include takes the filename as its
        // title/tab label
        assert!(out.starts_with("```gdcode lang=js hl=2 label=snippet.js\n"), "{out}");
        let body: Vec<&str> = out.trim().lines().skip(1).collect();
        assert_eq!(body.len(), 4, "{out}"); // 3 content lines + closer
        assert_eq!(body[1], "  // ..");
    }

    #[test]
    fn include_with_highlight_and_lang_switch() {
        let out = pre()
            .run("<<< @/snippets/snippet.js{1-2 ansi}\n", "guide/x.md")
            .unwrap();
        assert!(out.contains("```gdcode lang=ansi hl=1-2"), "{out}");
        let body = out.lines().count() - 2; // minus open/close fences
        assert_eq!(body, 3, "{out}"); // all lines kept, 1-2 highlighted
    }

    #[test]
    fn include_with_region_and_ln() {
        let out = pre()
            .run("<<< @/snippets/snippet-with-region.js#snippet{1 ts:line-numbers}\n", "g/x.md")
            .unwrap();
        assert!(out.contains("```gdcode lang=ts hl=1 ln=true"), "{out}");
        assert!(out.contains("function foo()"), "region line: {out}");
        assert_eq!(out.lines().count() - 2, 3, "all three region lines: {out}");
    }

    #[test]
    fn include_with_hl_and_ln_suffix_ordering() {
        // the generated fence must keep :line-numbers on the lang token —
        // rewrite_info strips the suffix only there
        let out = pre()
            .run("<<< @/snippets/snippet.js{1,2 :line-numbers}\n", "guide/x.md")
            .unwrap();
        assert!(out.contains("```gdcode lang=js hl=1,2 ln=true"), "{out}");
    }

    #[test]
    fn custom_container_renders_with_kind_styling() {
        let opts = ContainerOptions {
            custom: vec![CustomContainer {
                name: "success".into(),
                kind: Some("tip".into()),
                label: Some("成功".into()),
            }],
            ..Default::default()
        };
        let out = expand_containers("::: success\nnice\n:::\n", &opts);
        assert!(out.contains("<div class=\"custom-block tip\">"), "{out}");
        assert!(out.contains("成功"), "{out}");
    }

    #[test]
    fn container_labels_overridable() {
        let opts = ContainerOptions { tip_label: Some("提示".into()), ..Default::default() };
        let out = expand_containers("::: tip\nhi\n:::\n", &opts);
        assert!(out.contains("提示"), "{out}");
        // unknown kinds stay literal
        let out2 = expand_containers("::: mystery\nx\n:::\n", &ContainerOptions::default());
        assert!(!out2.contains("custom-block"), "{out2}");
        assert!(out2.contains("::: mystery"), "{out2}");
    }
}

#[cfg(test)]
mod group_marker_tests {
    use super::*;

    #[test]
    fn group_marker_is_a_token_not_a_substring() {
        // a bare "```" opener in a code group trims to exactly the marker:
        // token equality must still flag it, and the token must not leak
        // into the language
        let meta = rewrite_info(GD_GROUP_TOKEN);
        assert!(meta.contains(" group=1"), "{meta}");
        assert!(!meta.contains(GD_GROUP_TOKEN), "{meta}");
        assert_eq!(meta, "gdcode group=1");
        // a real label alongside the marker survives
        let meta = rewrite_info(&format!("js [x] {GD_GROUP_TOKEN}"));
        assert!(meta.contains("label=x"), "{meta}");
        assert!(meta.contains("group=1"), "{meta}");
    }
}
