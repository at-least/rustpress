//! Markdown preprocessing before comrak — everything VitePress does with
//! markdown-it plugins, done as fence-aware line passes:
//!
//! 1. `<<< @/path` / `<!--@include: file.md-->` file includes
//!
//! Known fence-tracking approximations (the preprocessor does not track
//! list nesting): (a) list-marker lines are always fence-eligible at
//! any indent; (b) a marker-less fence counts as inside the item while
//! it sits within 3 columns of the last marker line's content column,
//! and that state ends at the first column-0-ish paragraph that follows
//! a blank line — a resumed OUTER list item after a nested one can
//! therefore lose a fence; (c) each included file is resolved with
//! fresh list state (the eight later passes see the merged document
//! with full context).
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
//! shorter runs inside are literal text). Every pass classifies its
//! lines through one `FenceScanner` — fence state plus the list-item
//! content column — so those rules and their order live in one place.

use std::path::{Path, PathBuf};

use regex::Regex;

use super::highlight::FENCE_LANG;
use crate::config::ContainerOptions;

/// Errors during preprocessing (mostly bad includes).
#[derive(Debug, thiserror::Error)]
pub enum PreprocessError {
    #[error("cannot read include {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("include {path}: no such region or heading {section:?}")]
    Section { path: PathBuf, section: String },
    #[error("include {path}: invalid line range {{{spec}}}")]
    Range { path: PathBuf, spec: String },
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
}

const MAX_INCLUDE_DEPTH: u8 = 8;

/// Info-string token marking a fence as a `::: code-group` member. A
/// token (not a substring) so a bare "```" opener — whose trimmed info
/// is exactly this marker — is still detected, and no natural info
/// string collides with it.
const GD_GROUP_TOKEN: &str = "gd-ingroup";

/// Marks a container title element whose escaped text is rendered as
/// inline markdown after comrak's pass (upstream renders `:::` titles
/// with md.renderInline); the renderer strips it.
pub const INLINE_MARK: &str = "data-gd-inline";

impl<'a> Preprocess<'a> {
    pub fn run(&self, body: &str, page_rel: &str) -> Result<String, PreprocessError> {
        let page_dir = self
            .content_dir
            .join(Path::new(page_rel).parent().unwrap_or(Path::new("")));
        let md = self.resolve_includes(body, &page_dir, 0)?;
        let md = expand_inline_footnotes(&md);
        let md = rewrite_link_attrs(&md);
        let md = expand_alerts(&md, &self.container);
        let md = expand_containers(&md, &self.container);
        let md = rewrite_fences(&md);
        Ok(rewrite_badges(&md))
    }

    /// Pass 1: `<<<` code includes and `<!--@include:-->` markdown
    /// includes. `dir` is the directory of the file currently being
    /// resolved: relative include targets resolve against the including
    /// file's directory (upstream recurses `processIncludes` with the
    /// included file's path), so nested includes are carried by `dir`,
    /// not pinned to the page. Include targets take an optional
    /// `#section` (VS Code region or heading anchor) and an optional
    /// `{a,b}` line range; inside code fences the directive inserts the
    /// selected lines verbatim (upstream "Including Code Files").
    fn resolve_includes(&self, md: &str, dir: &Path, depth: u8) -> Result<String, PreprocessError> {
        let mut out = String::with_capacity(md.len());
        // An open fence is buffered instead of streamed: its body can
        // gain lines from a fenced `@include`, and if those lines carry
        // a closing run for the fence, the opener (and the author's own
        // closer) must be emitted longer than it — otherwise the partial
        // closes the fence and everything after it is swallowed as code
        let mut scanner = FenceScanner::default();
        // the open fence's marker, run length and content pad
        let mut fence: Option<(char, usize, String)> = None;
        let mut opener_line: Option<String> = None;
        let mut body = String::new();
        for line in md.split_inclusive('\n') {
            let bare = line.trim_end_matches(['\n', '\r']);
            let t = bare.trim();
            match scanner.step(bare) {
                LineKind::Closes => {
                    let (ch, n, _) = fence.take().expect("closer of an open fence");
                    let len = longest_closing_run(&body, ch, n).map_or(n, |run| run.max(n) + 1);
                    out.push_str(&lengthen_fence(
                        &opener_line.take().expect("open fence has an opener line"),
                        ch,
                        len,
                    ));
                    out.push_str(&body);
                    body.clear();
                    out.push_str(&lengthen_fence(line, ch, len));
                    continue;
                }
                LineKind::Inside => {
                    let (_, _, pad) = fence.as_ref().expect("content of an open fence");
                    if let Some(target) = md_include_target(t) {
                        // verbatim insertion — the raw selected lines join
                        // the code block content; a missing final newline is
                        // restored or the inserted text glues onto the next
                        // source line (typically the closing fence). Inside
                        // a list-item fence the inserted lines must carry
                        // the block's content column, or their column-0
                        // position would end the list item — and the fenced
                        // block with it
                        let resolved = self.load_include(&target, dir, depth)?;
                        for l in resolved.split_inclusive('\n') {
                            body.push_str(pad);
                            body.push_str(l);
                        }
                        if !resolved.ends_with('\n') {
                            body.push('\n');
                        }
                    } else {
                        body.push_str(line);
                    }
                    continue;
                }
                LineKind::Opens(ch, n, col) => {
                    // the block's content column is the column of the
                    // fence marker itself (for `10. ` that is 4, for `- `
                    // that is 2): the opener's own leading whitespace
                    // verbatim — a tab stays a tab, so it still reaches
                    // the tab stop comrak expands it to — plus the
                    // marker's width in spaces; content lines may be
                    // indented up to the opener's indentation without
                    // changing their content, so this is always safe
                    let ws = &bare[..bare.len() - bare.trim_start().len()];
                    let pad = format!("{ws}{}", " ".repeat(col - ws.len()));
                    fence = Some((ch, n, pad));
                    opener_line = Some(line.to_string());
                    continue;
                }
                LineKind::Outside => {}
            }
            if t.starts_with("<<<") {
                match self.expand_code_include(t, dir) {
                    Ok(Some((block, nested_dir))) => {
                        // The block may itself pull includes (rare);
                        // recurse on just this chunk, resolving against
                        // the included file's directory.
                        let resolved = self.resolve_includes(&block, &nested_dir, depth + 1)?;
                        out.push_str(&resolved);
                        if !resolved.ends_with('\n') {
                            out.push('\n');
                        }
                    }
                    // unsupported form: keep literal
                    Ok(None) => out.push_str(line),
                    // a named target that cannot be read or whose
                    // region/heading matches nothing must not silently
                    // ship the literal directive line
                    Err(e) => return Err(e),
                }
                continue;
            }
            if let Some(target) = md_include_target(t) {
                if depth >= MAX_INCLUDE_DEPTH {
                    return Err(PreprocessError::Depth {
                        path: target.raw_path(),
                    });
                }
                let inner = self.load_include(&target, dir, depth)?;
                let inner_dir = self.include_dir(&target, dir);
                // included files resolve with fresh list state: their
                // fences are judged from their own document context
                let inner = self.resolve_includes(&inner, &inner_dir, depth + 1)?;
                out.push_str(&inner);
                if !inner.ends_with('\n') {
                    out.push('\n');
                }
                continue;
            }
            out.push_str(line);
        }
        // an unclosed fence runs to EOF. The body can still carry a
        // closer smuggled in by an include, so the opener must grow
        // past it exactly like a closed fence's would
        if let (Some(opener), Some((ch, n, ..))) = (opener_line.take(), &fence) {
            let len = longest_closing_run(&body, *ch, *n).map_or(*n, |run| run.max(*n) + 1);
            out.push_str(&lengthen_fence(&opener, *ch, len));
            out.push_str(&body);
        }
        Ok(out)
    }

    /// Read an include target and apply its `#section` + `{range}`
    /// selectors. `@/`-prefixed paths resolve against the site root,
    /// everything else against the including file's directory.
    fn load_include(
        &self,
        target: &IncludeTarget,
        dir: &Path,
        depth: u8,
    ) -> Result<String, PreprocessError> {
        if depth >= MAX_INCLUDE_DEPTH {
            return Err(PreprocessError::Depth {
                path: target.raw_path(),
            });
        }
        let file = if let Some(rel) = target.path.strip_prefix("@/") {
            self.site_root.join(rel)
        } else {
            dir.join(&target.path)
        };
        let mut content =
            std::fs::read_to_string(&file).map_err(|source| PreprocessError::Read {
                path: file.clone(),
                source,
            })?;
        // an included .md file's front matter belongs to the part, not
        // the including page (upstream strips it too) — except for a
        // bare `{a,b}` line range, whose numbers count the raw file
        if target.path.ends_with(".md") && (target.section.is_some() || target.range.is_none()) {
            let (_, body) = crate::content::split_front_matter(&content);
            content = body.to_string();
        }
        if let Some(section) = &target.section {
            match extract_region(&content, section)
                .or_else(|| extract_heading_section(&content, section))
            {
                Some(extracted) => content = extracted,
                None => {
                    return Err(PreprocessError::Section {
                        path: file.clone(),
                        section: section.clone(),
                    });
                }
            }
        }
        if let Some(range) = &target.range {
            content = apply_line_range(&content, range).map_err(|()| PreprocessError::Range {
                path: file.clone(),
                spec: range.clone(),
            })?;
        }
        Ok(content)
    }

    /// `<<< @/snippets/x.ts` / `<<< ../y.js` / `<<< ./z.vue [label]` → a
    /// fenced block. `Ok(None)` marks forms we don't support, which stay
    /// literal; a named target that fails to read — or whose
    /// region/heading matches nothing — is an error, not a silently
    /// published directive.
    fn expand_code_include(
        &self,
        line: &str,
        dir: &Path,
    ) -> Result<Option<(String, PathBuf)>, PreprocessError> {
        let rest = line.trim_start_matches("<<<").trim();
        let (mut target, label) = match rest.find(" [") {
            Some(i) if rest.ends_with(']') => {
                (&rest[..i], Some(rest[i + 2..rest.len() - 1].to_string()))
            }
            _ => (rest, None),
        };
        // the brace spec comes after any #region anchor — parse it first
        let mut hl: Option<String> = None;
        let mut lang_switch: Option<String> = None;
        let mut ln = false;
        if let Some(i) = target.find('{') {
            let j = match target[i..].find('}') {
                Some(j) => j,
                None => return Ok(None), // unsupported form: keep literal
            };
            let spec = &target[i + 1..i + j];
            for token in spec.split_whitespace() {
                if token == ":line-numbers" {
                    ln = true;
                } else if token
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == ',' || c == '-')
                {
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
            dir.join(target)
        };
        let mut content =
            std::fs::read_to_string(&file).map_err(|source| PreprocessError::Read {
                path: file.clone(),
                source,
            })?;
        if let Some(name) = region {
            match extract_region(&content, &name) {
                // snippet regions are dedented (upstream snippet.ts);
                // markdown includes keep the region's indentation
                Some(extracted) => content = dedent(&extracted),
                None => {
                    return Err(PreprocessError::Section {
                        path: file.clone(),
                        section: name,
                    });
                }
            }
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
        let label = label.or_else(|| file.file_name().map(|n| n.to_string_lossy().into_owned()));
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
        let nested_dir = file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| dir.to_path_buf());
        Ok(Some((block, nested_dir)))
    }

    /// The directory nested includes inside `target`'s content resolve
    /// against: the included file's own directory (`@/` targets live
    /// under the site root, everything else under the including file).
    fn include_dir(&self, target: &IncludeTarget, base: &Path) -> PathBuf {
        if let Some(rel) = target.path.strip_prefix("@/") {
            self.site_root
                .join(Path::new(rel).parent().unwrap_or(Path::new("")))
        } else {
            base.join(Path::new(&target.path).parent().unwrap_or(Path::new("")))
        }
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
    // upstream's directive is <!--\s*@include:\s*…-->; the space after
    // the comment opener is accepted too
    let inner = t
        .strip_prefix("<!--")
        .and_then(|r| r.trim_start().strip_prefix("@include:"))?
        .strip_suffix("-->")?
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
/// markdown includes. Upstream "Markdown File Inclusion" defines the
/// contiguous slices `{3,}`, `{,10}` and `{1,10}`; a hyphenated second
/// part (`{a,b-c}`) selects line `a` and then the inclusive range
/// `b..=c`, the same convention as snippet line highlighting. Any other
/// shape — an unparsable bound, a dangling hyphen (`{2,5-}`), a zero or
/// reversed or out-of-file range — is an error: it must fail the build
/// naming the spec, not silently ship the wrong lines.
fn apply_line_range(content: &str, spec: &str) -> Result<String, ()> {
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let last = lines.len();
    let pick = |from: usize, to: usize, out: &mut String| -> Result<(), ()> {
        // upstream fails a range whose end is out of bounds; anything
        // out of order or out of file is a spec mistake, not a clamp
        if from == 0 || to == 0 || from > to || from > last || to > last {
            return Err(());
        }
        for n in from..=to {
            if let Some(l) = lines.get(n - 1) {
                out.push_str(l);
            }
        }
        Ok(())
    };
    let parse = |s: &str, default: usize| -> Result<usize, ()> {
        let t = s.trim();
        if t.is_empty() {
            return Ok(default);
        }
        t.parse().map_err(|_| ())
    };
    let (a, b) = spec.split_once(',').unwrap_or((spec, ""));
    let from = parse(a, 1)?;
    if from == 0 {
        return Err(());
    }
    let mut out = String::new();
    let b = b.trim();
    match b.split_once('-') {
        Some((lo, hi)) => {
            pick(from, from, &mut out)?;
            // both halves of the hyphenated form are explicit — an empty
            // half (`{2,5-}`, `{2,-4}`) is a typo, not an open end
            let lo = parse(lo, 0)?;
            let hi = parse(hi, 0)?;
            if lo == 0 || hi == 0 {
                return Err(());
            }
            pick(lo, hi, &mut out)?;
        }
        None => {
            let to = parse(b, last)?;
            pick(from, to, &mut out)?;
        }
    }
    Ok(out)
}

/// Heading-anchor section extraction: from the heading whose auto slug
/// (or explicit `{#id}`) matches, up to (not including) the next
/// heading of the same or higher level. Fence-aware.
fn extract_heading_section(content: &str, anchor: &str) -> Option<String> {
    // the same Anchorizer the renderer uses (`collect_headings` feeds it
    // the raw heading text, `{#id}` literal included), fed every heading
    // in document order: the anchor an author copies off the rendered
    // page matches, including the `-1`, `-2` suffixes of repeated
    // headings and the marks/connector punctuation comrak keeps
    let mut anchorizer = comrak::Anchorizer::new();
    let mut scanner = FenceScanner::default();
    let mut level = 0usize;
    let mut started = false;
    let mut out = String::new();
    for line in content.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if scanner.step(bare) != LineKind::Outside {
            if started {
                out.push_str(line);
            }
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
                let slug = anchorizer.anchorize(text);
                if explicit.as_deref() == Some(anchor) || slug == anchor {
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

/// `// #region name` marker check: the name must be the exact token
/// after `#region`, so `demo` does not open `#region demo2`.
fn region_opens(line: &str, name: &str) -> bool {
    let Some(i) = line.find("#region") else {
        return false;
    };
    line[i + "#region".len()..]
        .split_whitespace()
        .next()
        .is_some_and(|token| token == name)
}

/// Common leading whitespace stripped from every non-blank line.
/// Upstream applies this to `<<<` snippet regions only (snippet.ts);
/// a `<!--@include:-->` region keeps its indentation (include.ts
/// slices the raw lines), so `extract_region` returns the body as-is
/// and the snippet path dedents.
fn dedent(text: &str) -> String {
    let common = text
        .split_inclusive('\n')
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    if common == 0 {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        if line.trim().is_empty() {
            out.push_str(line);
        } else {
            out.push_str(line.get(common..).unwrap_or(line));
        }
    }
    out
}

fn extract_region(content: &str, name: &str) -> Option<String> {
    let mut out = String::new();
    let mut depth = 0usize;
    for line in content.split_inclusive('\n') {
        let t = line.trim_start();
        if depth == 0 {
            if region_opens(t, name) {
                depth = 1;
            }
            continue;
        }
        // a nested #region must not end the selection; its own
        // #endregion closes it
        if t.contains("#region") {
            depth += 1;
            continue;
        }
        if t.contains("#endregion") {
            depth -= 1;
            if depth == 0 {
                return Some(out);
            }
            continue;
        }
        out.push_str(line);
    }
    (depth > 0).then_some(out)
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
    let mut scanner = FenceScanner::default();
    let mut counter = 0usize;
    let mut defs: Vec<String> = Vec::new();
    let mut out = String::with_capacity(md.len());
    for line in md.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if scanner.step(bare) != LineKind::Outside {
            out.push_str(line);
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

fn replace_inline(text: &str, re: &Regex, counter: &mut usize, defs: &mut Vec<String>) -> String {
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
/// anchor, so markdown inside the link text is not re-parsed. The href
/// carries a `data-gd-mdlink` marker: these anchors bypass comrak's
/// link nodes entirely, so `.md` → canonical-URL resolution and base
/// prefixing happen in a later pass over the rendered HTML
/// (`resolve_marked_anchors`). Fence- and inline-code-span aware.
pub fn rewrite_link_attrs(md: &str) -> String {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        // [text](url){k="v" k2="v2"} — attrs must contain =" to avoid
        // eating unrelated braces right after links
        Regex::new(r#"(?m)(!?)\[([^\]\n]*)\]\(([^)\s]+)\)\{([^{}]*="[^}"]*"[^{}]*)\}"#).unwrap()
    });
    let mut scanner = FenceScanner::default();
    let mut out = String::with_capacity(md.len());
    for line in md.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if scanner.step(bare) != LineKind::Outside {
            out.push_str(line);
            continue;
        }
        // leave lines whose only {…} follow code spans alone: split the
        // line at inline code spans and only rewrite outside them
        let mut replaced = String::new();
        let mut rest = bare;
        while let Some(i) = rest.find('`') {
            let (before, after) = rest.split_at(i);
            let span_end = after[1..].find('`').map(|j| j + 2);
            replaced.push_str(&re.replace_all(before, render_attrs_element));
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
        replaced.push_str(&re.replace_all(rest, render_attrs_element));
        out.push_str(&replaced);
        out.push('\n');
    }
    out
}

/// One `[text](url){attrs}` / `![alt](src){attrs}` match → raw element.
/// An image stays an image (the attrs land on the `<img>`, as
/// @mdit/plugin-attrs does) instead of turning into a link; both carry
/// the `data-gd-mdlink` marker for the post-pass URL resolution.
fn render_attrs_element(c: &regex::Captures) -> String {
    if &c[1] == "!" {
        format!(
            r#"<img src="{}" alt="{}" data-gd-mdlink{}>"#,
            escape_text(&c[3]),
            escape_text(&c[2]),
            parse_attrs(&c[4])
        )
    } else {
        format!(
            r#"<a href="{}" data-gd-mdlink{}>{}</a>"#,
            escape_text(&c[3]),
            parse_attrs(&c[4]),
            escape_text(&c[2])
        )
    }
}

/// `k="v" k2="v2"` → ` k="v" k2="v2"` with sanitized names/values.
/// Values are fully entity-escaped (`& < > "`): the marked-anchor
/// post-pass matches `<a …>` with a first-`>` regex, and a raw `>`
/// inside a quoted value would truncate the tag there.
fn parse_attrs(raw: &str) -> String {
    static PAIR: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let pair = PAIR.get_or_init(|| Regex::new(r#"([a-zA-Z-]+)="([^"]*)""#).unwrap());
    let mut out = String::new();
    for c in pair.captures_iter(raw) {
        out.push_str(&format!(r#" {}="{}""#, &c[1], escape_attr_value(&c[2])));
    }
    out
}

/// Entity-escape an attribute value, but leave an `&` alone when it
/// already opens a well-formed entity reference — author-written
/// `&amp;` must not double-escape into `&amp;amp;`.
fn escape_attr_value(s: &str) -> String {
    static ENT: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let ent = ENT.get_or_init(|| {
        Regex::new(r#"&(?:[a-zA-Z][a-zA-Z0-9]*|#[0-9]+|#[xX][0-9a-fA-F]+);"#).unwrap()
    });
    fn escape_plain(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
    let mut out = String::with_capacity(s.len());
    let mut last = 0;
    for m in ent.find_iter(s) {
        out.push_str(&escape_plain(&s[last..m.start()]));
        out.push_str(m.as_str());
        last = m.end();
    }
    out.push_str(&escape_plain(&s[last..]));
    out
}
/// Pass 1.5: `> [!KIND] [title]` GitHub-flavored alerts → `::: kind`
/// containers, which is exactly what VitePress does (alerts ARE
/// containers there): same styling, same `[markdown.container]` labels,
/// and `[!DANGER]` plus registered custom kinds work. Fence-aware —
/// example alerts inside code fences stay literal.
pub fn expand_alerts(md: &str, opts: &ContainerOptions) -> String {
    let mut out_lines: Vec<String> = Vec::new();
    let mut scanner = FenceScanner::default();
    let lines: Vec<&str> = md.split_inclusive('\n').collect();
    let mut i = 0usize;
    while i < lines.len() {
        let bare = lines[i].trim_end_matches(['\n', '\r']);
        if scanner.step(bare) != LineKind::Outside {
            out_lines.push(bare.to_string());
            i += 1;
            continue;
        }
        if let Some((indent, kind, title)) = alert_opener(bare, opts) {
            // upstream renders alert titles as plain text, not inline
            // markdown: backslash-escaping each ASCII punctuation char
            // (CommonMark) makes the title pass render it verbatim
            let title_suffix = if title.is_empty() {
                String::new()
            } else {
                let literal: String = title
                    .chars()
                    .flat_map(|c| {
                        let esc = c.is_ascii_punctuation().then_some('\\');
                        esc.into_iter().chain(std::iter::once(c))
                    })
                    .collect();
                format!(" {literal}")
            };
            out_lines.push(format!("{indent}::: {kind}{title_suffix}"));
            i += 1;
            alert_quote_lines(&lines, &mut i, opts, &indent, 1, &mut out_lines);
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

/// One alert's body: the blockquote's `>`-prefixed lines (a `>`-only
/// line is a blank line *inside* the quote and continues it). A
/// non-`>` line that is paragraph continuation text stays in the quote
/// too — GFM lazy continuation, what GitHub renders for a wrapped
/// alert; constructs that could interrupt a paragraph (headings,
/// fences, lists, …) still end it.
///
/// A body line that is itself an alert opener is a nested alert
/// (GitHub quotes nest): it becomes one container deeper, and the
/// deeper `>`-prefixed lines are its body — without this, the inner
/// `[!KIND]` marker survives to comrak, whose built-in alerts
/// extension renders it with `markdown-alert` classes nothing styles.
///
/// `depth` is this quote's nesting level: every body line sheds that
/// many `>` markers, so a nested body arrives marker-free and a
/// deeper `> > > [!KIND]` opener is still recognized. A line with
/// fewer markers is lazy continuation when it continues a paragraph
/// (CommonMark lets that reach through every nesting level), and
/// otherwise ends this quote for the enclosing level to handle.
fn alert_quote_lines(
    lines: &[&str],
    i: &mut usize,
    opts: &ContainerOptions,
    ind: &str,
    depth: usize,
    out_lines: &mut Vec<String>,
) {
    let mut in_paragraph = false;
    while *i < lines.len() {
        let b = lines[*i].trim_end_matches(['\n', '\r']);
        let t = b.trim_start();
        let line_ind = &b[..b.len() - t.len()];
        let (layers, body) = strip_quote_markers(t, depth);
        if layers < depth {
            if in_paragraph && is_lazy_continuation(body) {
                out_lines.push(format!("{line_ind}{body}"));
                *i += 1;
                continue;
            }
            break;
        }
        if let Some((_, inner_kind, inner_title)) = alert_opener(body, opts) {
            // the nested container keeps this container's indent: one
            // level deeper per nesting would put a third-level opener
            // at four spaces, which comrak reads as an indented code
            // block instead of an HTML block
            let title_suffix = if inner_title.is_empty() {
                String::new()
            } else {
                format!(" {inner_title}")
            };
            out_lines.push(format!("{ind}::: {inner_kind}{title_suffix}"));
            *i += 1;
            alert_quote_lines(lines, i, opts, ind, depth + 1, out_lines);
            out_lines.push(format!("{ind}:::"));
            out_lines.push(String::new());
            in_paragraph = false;
            continue;
        }
        out_lines.push(format!("{line_ind}{body}"));
        in_paragraph = !body.trim().is_empty();
        *i += 1;
    }
}

/// Shed up to `depth` blockquote markers (`>` plus one optional space,
/// each allowed leading spaces) from `t`: (markers shed, remainder).
fn strip_quote_markers(t: &str, depth: usize) -> (usize, &str) {
    let mut rest = t;
    let mut layers = 0;
    while layers < depth {
        let Some(after) = rest.trim_start_matches(' ').strip_prefix('>') else {
            break;
        };
        rest = after.strip_prefix(' ').unwrap_or(after);
        layers += 1;
    }
    (layers, rest)
}

/// Is a `>`-less line paragraph continuation text inside a quote?
/// CommonMark's lazy continuation: yes when non-blank and not a
/// construct that could interrupt a paragraph (ATX heading, fenced
/// code, blockquote, list item, thematic break, container marker,
/// HTML block).
fn is_lazy_continuation(t: &str) -> bool {
    if t.is_empty() {
        return false;
    }
    let starts = |p: &str| t.starts_with(p);
    if starts(">")
        || starts("#")
        || starts("```")
        || starts("~~~")
        || starts(":::")
        || (starts("<")
            && t[1..].starts_with(|c: char| c.is_ascii_alphabetic() || c == '/' || c == '!'))
    {
        return false;
    }
    // bullet list item (`- `, `+ `, `* `)
    if matches!(t.chars().next(), Some('-' | '+' | '*'))
        && t.chars().nth(1).is_some_and(char::is_whitespace)
    {
        return false;
    }
    // ordered list item (`1. `, `12) `)
    let digits = t.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits > 0
        && digits <= 9
        && matches!(t.as_bytes().get(digits), Some(b'.' | b')'))
        && t.as_bytes()
            .get(digits + 1)
            .is_some_and(|b| b.is_ascii_whitespace())
    {
        return false;
    }
    !is_thematic_break(t)
}

/// Thematic break (`---`, `* * *`, `___`): three or more of one of
/// `-`, `_`, `*` with nothing but whitespace between.
fn is_thematic_break(t: &str) -> bool {
    let stripped = t.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    stripped.len() >= 3
        && matches!(stripped.as_bytes()[0], b'-' | b'_' | b'*')
        && stripped.bytes().all(|b| b == stripped.as_bytes()[0])
}

/// `> [!KIND] [title]` → (indent, kind, title). Recognized kinds: the
/// builtin container kinds (incl. VitePress's `[!DANGER]` extension)
/// and registered custom containers — upstream lets registered
/// containers appear as alerts too.
fn alert_opener(line: &str, opts: &ContainerOptions) -> Option<(String, String, String)> {
    let t = line.trim_start();
    let indent = &line[..line.len() - t.len()];
    // the space after ">" is optional (GitHub accepts ">[!NOTE]")
    let rest = t.strip_prefix('>')?.trim_start();
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
    ) || opts
        .custom
        .iter()
        .any(|c| c.name.eq_ignore_ascii_case(&kind));
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
    let mut scanner = FenceScanner::default();
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
        match scanner.step(bare) {
            LineKind::Opens(..) => {
                // fences inside ::: code-group get a group marker: it
                // tells the fence renderer their label is a tab name,
                // not a standalone block title
                if stack.iter().any(|(_, o)| matches!(o, Open::CodeGroup)) {
                    out_lines.push(format!("{bare} {GD_GROUP_TOKEN}"));
                } else {
                    out_lines.push(bare.to_string());
                }
                continue;
            }
            LineKind::Inside | LineKind::Closes => {
                out_lines.push(bare.to_string());
                continue;
            }
            LineKind::Outside => {}
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
                && colons >= top_colons
            {
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
                    out_lines.extend([format!("{indent}<div class=\"vp-raw\">"), String::new()]);
                }
                "code-group" => {
                    stack.push((colons, Open::CodeGroup));
                    group_counter += 1;
                    let tabs = code_group_tabs(&lines, idx, group_counter, scanner.base());
                    out_lines.extend([
                        format!("{indent}<div class=\"vp-code-group\" x-data=\"codeGroup\">"),
                        format!("{indent}{tabs}"),
                        format!("{indent}<div class=\"blocks\">"),
                        String::new(),
                    ]);
                }
                "details" => {
                    let (summary, open) = parse_details_rest(rest);
                    let summary = if summary.is_empty() {
                        opts.label_for("details")
                    } else {
                        summary
                    };
                    stack.push((colons, Open::Details));
                    let open_attr = if open { " open" } else { "" };
                    out_lines.extend([
                        format!("{indent}<details class=\"custom-block details\"{open_attr}>"),
                        format!(
                            "{indent}<summary {INLINE_MARK}>{}</summary>",
                            escape_text(&summary)
                        ),
                        String::new(),
                    ]);
                }
                kind => {
                    // custom containers reuse a builtin kind's styling
                    let (class, default_label) = if let Some(cc) =
                        opts.custom.iter().find(|c| c.name == kind)
                    {
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
                            "{indent}<p class=\"custom-block-title\" {INLINE_MARK}>{}</p>",
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
fn code_group_tabs(lines: &[&str], idx: usize, group_no: usize, base: usize) -> String {
    let mut labels: Vec<String> = Vec::new();
    // continue the caller's list state: a group inside a `1. ` item has
    // its fences at four spaces, which only count as fences while the
    // item's content column is known — a fresh base would drop every
    // tab of such a group
    let mut scanner = FenceScanner::from_base(base);
    for line in lines.iter().skip(idx + 1) {
        let bare = line.trim_end_matches(['\n', '\r']);
        let n = match scanner.step(bare) {
            LineKind::Opens(_, n, _) => n,
            LineKind::Inside | LineKind::Closes => continue,
            LineKind::Outside => {
                // stop at the container closer (any ::: line)
                if closing_container(bare.trim()).is_some()
                    || opening_container(bare.trim()).is_some()
                {
                    break;
                }
                continue;
            }
        };
        {
            let info = bare[fence_info_offset(bare, n)..].trim();
            let label = info.find('[').and_then(|i| {
                info[i + 1..]
                    .find(']')
                    .map(|j| info[i + 1..i + 1 + j].to_string())
            });
            let lang = info.split_whitespace().next().unwrap_or("").to_string();
            labels.push(label.filter(|l| !l.is_empty()).unwrap_or(lang));
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
fn opening_container(t: &str) -> Option<(usize, &str, &str)> {
    let colons = t.chars().take_while(|&c| c == ':').count();
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
/// `{open}` with spaces inside the braces; anything else (`{reopen}`,
/// `{no-open}`) is not the open flag and is ignored.
fn parse_details_rest(rest: &str) -> (String, bool) {
    if rest.ends_with('}')
        && rest.contains('{')
        && let Some(i) = rest.rfind('{')
    {
        let attr = rest[i + 1..rest.len() - 1].trim();
        if attr == "open" {
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
    let mut scanner = FenceScanner::default();
    for line in md.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        match scanner.step(bare) {
            LineKind::Opens(_, n, _) => {
                let cut = fence_info_offset(bare, n);
                let info = &bare[cut..];
                let new_line = format!("{}{}", &bare[..cut], rewrite_info(info.trim()));
                out.push_str(&new_line);
                out.push('\n');
                continue;
            }
            LineKind::Inside | LineKind::Closes => {
                out.push_str(bare);
                out.push('\n');
                continue;
            }
            LineKind::Outside => {}
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
        && let Some(j) = rest[i..].find(']')
    {
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
        && let Some(j) = rest[i..].find('}')
    {
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
    for (suffix, token) in [(":no-line-numbers", "false"), (":line-numbers", "true")] {
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
    let self_closing =
        SELF_CLOSING.get_or_init(|| Regex::new(r#"<Badge\s+([^<>]*?)\s*/>"#).unwrap());
    let paired =
        PAIRED.get_or_init(|| Regex::new(r#"(?s)<Badge\s+([^<>]*?)>(.*?)</Badge>"#).unwrap());

    let mut out = String::with_capacity(md.len());
    let mut scanner = FenceScanner::default();
    for line in md.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if scanner.step(bare) != LineKind::Outside {
            out.push_str(line);
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

/// One line's place in the fence structure, as `FenceScanner::step`
/// classifies it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LineKind {
    /// Opens a fenced block: marker char, run length, marker column.
    Opens(char, usize, usize),
    /// Content of an open fenced block.
    Inside,
    /// Closes the open fenced block.
    Closes,
    /// Ordinary markdown outside any fence.
    Outside,
}

/// The fence-and-list state every line pass keeps: which lines are
/// fenced-block content, and the content column of the most recent
/// list item (`ListBase`), which decides how far an indented opener may
/// sit. One implementation of the rules (`opening_fence`,
/// `is_closing_fence`, `ListBase::observe`) in one order, so no pass
/// can skip a step or seed the list state differently from the others.
#[derive(Default)]
struct FenceScanner {
    list_base: ListBase,
    fence: Option<(char, usize, usize)>,
}

impl FenceScanner {
    /// Continue from a caller's list state — for a scan that starts
    /// mid-document, like `code_group_tabs`.
    fn from_base(base: usize) -> Self {
        Self {
            list_base: ListBase {
                base,
                prev_blank: false,
            },
            fence: None,
        }
    }

    /// Classify `bare` (a line without its newline) and advance.
    fn step(&mut self, bare: &str) -> LineKind {
        if let Some((ch, n, col)) = self.fence {
            if is_closing_fence(bare, ch, n, col) {
                self.fence = None;
                return LineKind::Closes;
            }
            return LineKind::Inside;
        }
        self.list_base.observe(bare);
        match opening_fence(bare, self.list_base.base) {
            Some((ch, n, col)) => {
                self.fence = Some((ch, n, col));
                LineKind::Opens(ch, n, col)
            }
            None => LineKind::Outside,
        }
    }

    /// The current list item's content column.
    fn base(&self) -> usize {
        self.list_base.base
    }
}

/// Strip a list-item marker (`- `, `* `, `+ `, `2. `, `3) `) from the
/// front of a list line, so a fence sharing the line with its list
/// marker ("- ```md", CommonMark-legal) is still recognized as a fence.
fn strip_list_marker(t: &str) -> &str {
    let after_bullet = match t.strip_prefix(['-', '*', '+']) {
        Some(rest) => Some(rest),
        None => {
            let digits = t.len() - t.trim_start_matches(|c: char| c.is_ascii_digit()).len();
            if digits > 0 {
                t[digits..].strip_prefix(['.', ')'])
            } else {
                None
            }
        }
    };
    match after_bullet {
        Some(rest) if rest.starts_with(' ') => rest.trim_start_matches(' '),
        _ => t,
    }
}

/// The content column of the most recent list marker line: fences
/// indented near it belong to the item even without a marker on their
/// own line (`1. ` sets 3, so a 4-space fence is the item's block).
/// A non-blank line left of the base ends the item — but only after a
/// blank line: CommonMark lazy continuation lets item text sit at
/// column 0 when no blank intervened, and that text must not kill the
/// state a following item fence still needs.
#[derive(Default)]
struct ListBase {
    base: usize,
    prev_blank: bool,
}

impl ListBase {
    fn observe(&mut self, line: &str) {
        let raw = line.trim_start();
        if raw.is_empty() {
            self.prev_blank = true;
            return;
        }
        let indent = line.len() - raw.len();
        // `* * *` / `- - -` is a thematic break, not a `*`/`-` item: it
        // must not set a content column that turns the indented code
        // block after it into a fence
        let after = if is_thematic_break(raw) {
            raw
        } else {
            strip_list_marker(raw)
        };
        let was_blank = self.prev_blank;
        self.prev_blank = false;
        if after.len() != raw.len() {
            self.base = indent + (raw.len() - after.len());
        } else if was_blank && indent < self.base {
            self.base = 0;
        }
    }
}

fn opening_fence(line: &str, list_base: usize) -> Option<(char, usize, usize)> {
    let raw = line.trim_start();
    let marker = strip_list_marker(raw);
    let indent = line.len() - raw.len();
    // ≤3 spaces of indentation may open a fence; 4+ is an indented code
    // block — except inside a list item, whose content column the last
    // marker line set: CommonMark allows the item's fence up to 3 past
    // it. Marker-carrying lines are always eligible (the preprocessor
    // does not track list nesting).
    let in_item = list_base > 0 && indent + 3 >= list_base && indent <= list_base + 3;
    if raw.len() == marker.len() && indent >= 4 && !in_item {
        return None;
    }
    let first = marker.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let n = marker.chars().take_while(|&c| c == first).count();
    // a bare marker may be opener or closer (the caller decides via
    // is_closing_fence); an info string means definitely opening —
    // but per CommonMark only a BACKTICK fence's info string may not
    // contain a backtick (else the line is inline span text); a tilde
    // fence's info may contain tildes
    let info = &marker[n..];
    let forbidden = first == '`' && info.contains('`');
    // the fence marker's column; closers may sit up to 3 spaces past it
    let col = indent + (raw.len() - marker.len());
    (n >= 3 && !forbidden).then_some((first, n, col))
}

fn is_closing_fence(line: &str, ch: char, n: usize, opener_col: usize) -> bool {
    let raw = line.trim_start();
    let t = strip_list_marker(raw);
    // a closing fence may be indented up to 3 spaces past the opener's
    // column (CommonMark); a deeper fence run is content. List-marker
    // lines stay eligible — see opening_fence.
    if raw.len() == t.len() && line.len() - raw.len() > opener_col + 3 {
        return false;
    }
    let count = t.chars().take_while(|&c| c == ch).count();
    count >= n && t[count..].trim().is_empty()
}

/// Longest fence-shaped closing run in `body` (line-initial run of
/// `ch`, `>= n`, nothing but whitespace after) — the runs an included
/// partial could use to close a fence opened with `n` markers.
fn longest_closing_run(body: &str, ch: char, n: usize) -> Option<usize> {
    body.split_inclusive('\n')
        .filter_map(|line| {
            let bare = line.trim_end_matches(['\n', '\r']);
            // no indent cap here, deliberately: this guards fences in
            // list items too, where valid closers sit at the item's
            // content column; over-lengthening is always safe
            let t = bare.trim_start();
            let run = t.chars().take_while(|&c| c == ch).count();
            (run >= n && t[run..].trim().is_empty()).then_some(run)
        })
        .max()
}

/// Re-emit `line` with its first `ch` run grown to `new_len` markers
/// (a no-op when it already is that long). Used to out-grow closing
/// runs that included content smuggled into a fence body.
fn lengthen_fence(line: &str, ch: char, new_len: usize) -> String {
    let Some(i) = line.find(ch) else {
        return line.to_string();
    };
    let run = line[i..].chars().take_while(|&c| c == ch).count();
    if run >= new_len {
        return line.to_string();
    }
    let mut out = String::with_capacity(line.len() + (new_len - run) * ch.len_utf8());
    out.push_str(&line[..i]);
    for _ in 0..new_len {
        out.push(ch);
    }
    out.push_str(&line[i + run * ch.len_utf8()..]);
    out
}

/// Byte offset just past the opening fence marker in `line` (leading
/// indent, any list marker, then the marker run). Slicing the raw line
/// by the marker count alone is only correct at column 0 — inside a
/// list item the indent (or the "- " of a marker-line fence) would
/// swallow the backticks and corrupt the fence.
fn fence_info_offset(line: &str, count: usize) -> usize {
    let t = line.trim_start();
    let indent = line.len() - t.len();
    let after_list = strip_list_marker(t);
    indent + (t.len() - after_list.len()) + count
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
        assert!(out.contains("<p class=\"custom-block-title\" data-gd-inline>TIP</p>"));
        assert!(out.contains("hi **bold**"));
        let out = containers("::: warning SERVER REQUIRED\nx\n:::\n");
        assert!(
            out.contains("<p class=\"custom-block-title\" data-gd-inline>SERVER REQUIRED</p>"),
            "{out}"
        );
        let out = containers("::: tip {no-title}\nx\n:::\n");
        assert!(!out.contains("custom-block-title"), "{out}");
    }

    #[test]
    fn details_and_code_group() {
        let out = containers("::: details Click me {open}\nx\n:::\n");
        assert!(
            out.contains("<details class=\"custom-block details\" open>"),
            "{out}"
        );
        assert!(out.contains("<summary data-gd-inline>Click me</summary>"));
        let out = containers("::: code-group\n```js [a]\n1\n```\n:::\n");
        assert!(
            out.contains("<div class=\"vp-code-group\" x-data=\"codeGroup\">"),
            "{out}"
        );
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
        assert_eq!(
            rewrite_info("ts:no-line-numbers"),
            "gdcode lang=ts ln=false"
        );
        assert_eq!(rewrite_info("md:line-numbers=4"), "gdcode lang=md ln=4");
        assert_eq!(
            rewrite_info("vue [Layout.vue]"),
            "gdcode lang=vue label=Layout.vue"
        );
        assert_eq!(rewrite_info(""), "gdcode");
        // non-numeric braces are not line specs: left alone
        assert_eq!(rewrite_info("jsonc { \"n\": 2 }"), "gdcode lang=jsonc");
    }

    #[test]
    fn alert_opener_accepts_no_space_after_the_marker() {
        // GitHub accepts ">[!NOTE]" without a space after ">"; the alert
        // becomes a ::: container (pass 1.5), then a custom block (pass 2)
        let opts = ContainerOptions::default();
        let out = expand_containers(&expand_alerts(">[!NOTE]\n> body line\n", &opts), &opts);
        assert!(out.contains("<div class=\"custom-block note\">"), "{out}");
        assert!(out.contains("body line"), "{out}");
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
        assert!(
            out.contains("<span class=\"VPBadge info\">composable</span>"),
            "{out}"
        );
        let out = rewrite_badges("<Badge type=\"warning\">careful</Badge>\n");
        assert!(
            out.contains("<span class=\"VPBadge warning\">careful</span>"),
            "{out}"
        );
        let out = rewrite_badges("<Badge text=\"plain\" />\n");
        assert!(
            out.contains("<span class=\"VPBadge tip\">plain</span>"),
            "{out}"
        );
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
        let out = expand_containers(
            "a paragraph\n::: tip\ninner\n:::\nafter\n",
            &ContainerOptions::default(),
        );
        // the HTML block must interrupt the paragraph correctly at the
        // comrak layer; here we check the wrapper survives preprocessing
        assert!(out.contains("<div class=\"custom-block tip\">"), "{out}");
        assert!(out.contains("</div>"));
    }

    #[test]
    fn adjacent_containers() {
        let out = expand_containers(
            "::: tip\na\n:::\n::: warning\nb\n:::\n",
            &ContainerOptions::default(),
        );
        assert_eq!(out.matches("<div class=\"custom-block").count(), 2, "{out}");
        assert_eq!(out.matches("</div>").count(), 2, "{out}");
    }

    #[test]
    fn tilde_fences_protected() {
        let out = expand_containers(
            "~~~\n::: tip\n~~~\n::: info\nreal\n:::\n",
            &ContainerOptions::default(),
        );
        // ::: inside the tilde fence is literal; the real one expands
        assert!(out.contains("::: tip"), "{out}");
        assert!(out.contains("<div class=\"custom-block info\">"), "{out}");
    }

    #[test]
    fn tilde_fence_info_may_contain_tildes() {
        // CommonMark restricts info strings for BACKTICK fences only;
        // a tilde fence's info may contain its own char
        let out = expand_containers(
            "~~~a~b\n::: tip\nliteral\n:::\n~~~\n::: info\nreal\n:::\n",
            &ContainerOptions::default(),
        );
        assert!(out.contains("::: tip"), "{out}");
        assert!(out.contains("<div class=\"custom-block info\">"), "{out}");
    }

    #[test]
    fn four_space_indented_lines_are_not_fences() {
        let opts = ContainerOptions::default();
        // inside a fence, a 4-space-indented ``` is CONTENT per
        // CommonMark (only ≤3 spaces may close): the ::: after it must
        // stay inside the still-open code block instead of expanding
        let md = "```\ncode\n    ```\n::: tip\ninner\n:::\n";
        let out = expand_containers(&expand_alerts(md, &opts), &opts);
        assert!(!out.contains("custom-block"), "{out}");
        assert!(out.contains("::: tip"), "{out}");

        // a 4+-space-indented ```js line is an indented code block, so
        // its info string must not be rewritten into a gdcode sentinel
        let md = "text\n\n     ```js{1}\nvar x = 1\n     ```\n";
        let out = rewrite_fences(md);
        assert!(!out.contains("gdcode"), "{out}");
    }
    #[test]
    fn nested_alert_bodies_shed_every_quote_marker() {
        // depth-2 body lines carry two `>`; stripping one left `> inner`
        // (a blockquote inside the container) and a depth-3 opener was
        // never seen at all
        let opts = ContainerOptions::default();
        let out = expand_alerts(
            "> [!NOTE]\n> outer\n>\n> > [!TIP]\n> > inner\n>\n> back\n",
            &opts,
        );
        let tip = out.find("::: tip").unwrap();
        let tip_end = tip + out[tip..].find("\n:::").unwrap();
        let tip_body = &out[tip..tip_end];
        assert!(tip_body.contains("\ninner"), "{out}");
        assert!(!tip_body.contains("> inner"), "{out}");
        assert!(!tip_body.contains("back"), "outer line stays outer: {out}");

        let out = expand_alerts(
            "> [!NOTE]\n> > [!TIP]\n> > > [!WARNING]\n> > > deep\n",
            &opts,
        );
        assert!(out.contains("::: warning"), "{out}");
        assert!(!out.contains("[!WARNING]"), "{out}");
        // no per-level indentation: a third-level `:::` at four spaces
        // would be an indented code block to comrak
        for line in out.lines() {
            assert!(!line.starts_with("    :::"), "{out}");
        }
    }

    #[test]
    fn code_group_tabs_see_four_space_item_fences() {
        // the tab strip scan must continue the caller's list state or
        // fences at the `1. ` item's four-space column are invisible
        // to it and the group renders without tabs
        let out = expand_containers(
            "1. Step\n\n    ::: code-group\n\n    ```sh [npm]\n    npm i\n    ```\n\n    ```sh [yarn]\n    yarn\n    ```\n\n    :::\n",
            &ContainerOptions::default(),
        );
        assert!(out.contains(">npm<"), "{out}");
        assert!(out.contains(">yarn<"), "{out}");
    }

    #[test]
    fn thematic_break_does_not_open_a_list_item() {
        // `* * *` is a thematic break, not a `*` item: the indented code
        // block after it must stay one
        let out = rewrite_fences("* * *\n\n    ```js\n    x\n    ```\n");
        assert!(!out.contains("gdcode"), "{out}");
    }

    #[test]
    fn fence_with_trailing_spaces_and_info_closer() {
        // closer with trailing spaces; opener with info string
        let out = expand_containers("```js \n::: tip\n``` \n", &ContainerOptions::default());
        assert!(
            out.contains("::: tip"),
            "container inside fence stays literal: {out}"
        );
    }

    #[test]
    fn unbalanced_container_closed_at_eof() {
        let out = expand_containers("::: tip\nnever closed\n", &ContainerOptions::default());
        assert_eq!(
            out.matches("<div").count(),
            out.matches("</div>").count(),
            "{out}"
        );
    }

    #[test]
    fn info_rewrites_edge_forms() {
        assert_eq!(rewrite_info("vue{1-2}"), "gdcode lang=vue hl=1-2");
        assert_eq!(rewrite_info("sh [npm]"), "gdcode lang=sh label=npm");
        // label containing ']' — first bracket group only
        assert_eq!(rewrite_info("js [a[b]]"), "gdcode lang=js label=a[b");
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
        assert!(
            out.contains("<div class=\"vp-code-group\" x-data=\"codeGroup\">"),
            "{out}"
        );
        assert!(out.contains("<div class=\"tabs\">"), "{out}");
        assert!(out.contains("name=\"group-1\""), "{out}");
        assert!(
            out.contains("<label for=\"group-1-0\">npm</label>"),
            "{out}"
        );
        assert!(
            out.contains("<label for=\"group-1-1\">pnpm</label>"),
            "{out}"
        );
        assert!(out.contains("checked=\"checked\""), "{out}");
        // labels without [label] fall back to the fence language
        let out2 = expand_containers(
            "::: code-group\n```js\n1\n```\n:::\n",
            &ContainerOptions::default(),
        );
        assert!(
            out2.contains("<label for=\"group-1-0\">js</label>"),
            "{out2}"
        );
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
        assert!(
            out.starts_with("```gdcode lang=js hl=2 label=snippet.js\n"),
            "{out}"
        );
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
            .run(
                "<<< @/snippets/snippet-with-region.js#snippet{1 ts:line-numbers}\n",
                "g/x.md",
            )
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
            .run(
                "<<< @/snippets/snippet.js{1,2 :line-numbers}\n",
                "guide/x.md",
            )
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
        let opts = ContainerOptions {
            tip_label: Some("提示".into()),
            ..Default::default()
        };
        let out = expand_containers("::: tip\nhi\n:::\n", &opts);
        assert!(out.contains("提示"), "{out}");
        // unknown kinds stay literal
        let out2 = expand_containers("::: mystery\nx\n:::\n", &ContainerOptions::default());
        assert!(!out2.contains("custom-block"), "{out2}");
        assert!(out2.contains("::: mystery"), "{out2}");
    }

    #[test]
    fn details_open_attr_requires_an_exact_match() {
        // `{reopen}` contains the substring "open" but is not `{open}`:
        // unknown brace suffixes must be ignored, like upstream
        let out = expand_containers(
            "::: details Summary {reopen}\nx\n:::\n",
            &ContainerOptions::default(),
        );
        assert!(
            !out.contains("<details class=\"custom-block details\" open>"),
            "an attr merely containing 'open' expanded the details: {out}"
        );
        // `{ open }` with spaces inside the braces still opens
        let out = expand_containers(
            "::: details Summary { open }\nx\n:::\n",
            &ContainerOptions::default(),
        );
        assert!(
            out.contains("<details class=\"custom-block details\" open>"),
            "{out}"
        );
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
