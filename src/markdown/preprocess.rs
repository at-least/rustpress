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
}

const MAX_INCLUDE_DEPTH: u8 = 8;

impl<'a> Preprocess<'a> {
    pub fn run(&self, body: &str, page_rel: &str) -> Result<String, PreprocessError> {
        let md = self.resolve_includes(body, page_rel, 0)?;
        let md = expand_containers(&md);
        let md = rewrite_fences(&md);
        Ok(rewrite_badges(&md))
    }

    /// Pass 1: `<<<` code includes and `<!--@include:-->` markdown
    /// includes. `page_rel` is the page's source-relative path.
    fn resolve_includes(&self, md: &str, page_rel: &str, depth: u8) -> Result<String, PreprocessError> {
        let mut out = String::with_capacity(md.len());
        let mut fence: Option<(char, usize)> = None;
        let page_dir = self.content_dir.join(Path::new(page_rel).parent().unwrap_or(Path::new("")));
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
            let t = bare.trim();
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
            if let Some(path) = md_include_target(t) {
                if depth >= MAX_INCLUDE_DEPTH {
                    return Err(PreprocessError::Depth { path });
                }
                let file = page_dir.join(path);
                let raw = std::fs::read_to_string(&file)
                    .map_err(|source| PreprocessError::Read { path: file.clone(), source })?;
                let inner = self.resolve_includes(&raw, page_rel, depth + 1)?;
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

    /// `<<< @/snippets/x.ts` / `<<< ../y.js` / `<<< ./z.vue [label]` → a
    /// fenced block; returns None for forms we don't support (line
    /// selectors like `{2}`), which stay literal.
    fn expand_code_include(&self, line: &str, page_dir: &Path) -> Option<String> {
        let rest = line.trim_start_matches("<<<").trim();
        let (mut target, label) = match rest.find(" [") {
            Some(i) if rest.ends_with(']') => (&rest[..i], Some(rest[i + 2..rest.len() - 1].to_string())),
            _ => (rest, None),
        };
        let mut region = None;
        if let Some(i) = target.find('#') {
            region = Some(target[i + 1..].to_string());
            target = &target[..i];
        }
        if target.contains('{') {
            return None; // line-selector form unsupported
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
        let ext = file
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_default();
        if ext == "ansi" {
            content = strip_ansi(&content);
        }
        let marker = fence_marker_for(&content);
        let mut block = String::new();
        block.push_str(&format!("{marker}{ext}"));
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

/// `<!--@include: ./file.md-->` → the target path (relative to the page).
fn md_include_target(line: &str) -> Option<PathBuf> {
    let t = line.trim();
    let inner = t
        .strip_prefix("<!--@include:")
        .and_then(|r| r.strip_suffix("-->"))?
        .trim();
    if inner.is_empty() {
        return None;
    }
    Some(PathBuf::from(inner.strip_prefix("@/").unwrap_or(inner)))
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

fn strip_ansi(s: &str) -> String {
    static ANSI: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    ANSI.get_or_init(|| Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]").unwrap())
        .replace_all(s, "")
        .into_owned()
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

/// Pass 2: `:::` containers.
pub fn expand_containers(md: &str) -> String {
    #[derive(Clone, Copy)]
    enum Open {
        Tip,
        Details,
        CodeGroup,
        VPre,
    }
    let mut out_lines: Vec<String> = Vec::new();
    let mut stack: Vec<(usize, Open)> = Vec::new();
    let mut fence: Option<(char, usize)> = None;

    let close_markup = |open: Open, out: &mut Vec<String>| match open {
        Open::Tip => out.extend(["".into(), "</div>".into(), "".into()]),
        Open::Details => out.extend(["".into(), "</details>".into(), "".into()]),
        Open::CodeGroup => out.extend(["".into(), "</div>".into(), "</div>".into(), "".into()]),
        Open::VPre => {}
    };

    for line in md.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if let Some((ch, n)) = fence {
            out_lines.push(bare.to_string());
            if is_closing_fence(bare, ch, n) {
                fence = None;
            }
            continue;
        }
        if let Some((ch, n)) = opening_fence(bare) {
            out_lines.push(bare.to_string());
            fence = Some((ch, n));
            continue;
        }
        let t = bare.trim();
        // closing `:::` (3+ colons, nothing else): closes the innermost
        // container opened with ≤ that many colons
        if let Some(colons) = closing_container(t) {
            if let Some(&(top_colons, _)) = stack.last()
                && colons >= top_colons {
                    let (_, open) = stack.pop().unwrap();
                    close_markup(open, &mut out_lines);
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
                "code-group" => {
                    stack.push((colons, Open::CodeGroup));
                    out_lines.extend([
                        "<div class=\"vp-code-group\">".into(),
                        "<div class=\"blocks\">".into(),
                        String::new(),
                    ]);
                }
                "details" => {
                    let (summary, open) = parse_details_rest(rest);
                    stack.push((colons, Open::Details));
                    let open_attr = if open { " open" } else { "" };
                    out_lines.extend([
                        format!("<details class=\"custom-block details\"{open_attr}>"),
                        format!("<summary>{}</summary>", escape_text(&summary)),
                        String::new(),
                    ]);
                }
                kind => {
                    stack.push((colons, Open::Tip));
                    let (no_title, title) = parse_tip_rest(rest);
                    out_lines.push(format!("<div class=\"custom-block {kind}\">"));
                    if !no_title {
                        let title = title.unwrap_or_else(|| kind.to_uppercase());
                        out_lines.push(format!(
                            "<p class=\"custom-block-title\">{}</p>",
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
        close_markup(open, &mut out_lines);
    }
    let mut out = out_lines.join("\n");
    out.push('\n');
    out
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
        ("Details".to_string(), false)
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
            let info = &bare[n..];
            let new_line = format!("{} {}", &bare[..n], rewrite_info(info.trim()));
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
    // :line-numbers / :no-line-numbers / :line-numbers=N — unsupported,
    // stripped (giallo-era parity)
    let mut lang = rest.split_whitespace().next().unwrap_or("").to_string();
    loop {
        let stripped = lang
            .strip_suffix(":line-numbers")
            .map(str::to_string)
            .or_else(|| lang.strip_suffix(":no-line-numbers").map(str::to_string));
        match stripped {
            Some(s) => lang = s,
            None => break,
        }
    }
    if let Some(i) = lang.find(":line-numbers=") {
        lang.truncate(i);
    }
    lang = lang.trim_matches(':').to_string();

    let mut out = format!("{FENCE_LANG} lang={lang}");
    if let Some(hl) = hl {
        out.push_str(&format!(" hl={hl}"));
    }
    if let Some(label) = label {
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

pub(crate) fn escape_text(s: &str) -> String {
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
        expand_containers(md)
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
        assert!(out.contains("<div class=\"vp-code-group\">"), "{out}");
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
        assert_eq!(rewrite_info("md:line-numbers"), "gdcode lang=md");
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
    fn ansi_and_region_extraction() {
        assert_eq!(strip_ansi("\x1b[32mok\x1b[0m\n"), "ok\n");
        let src = "// #region setup\nconst a = 1;\n// #endregion\nrest\n";
        assert_eq!(extract_region(src, "setup").unwrap(), "const a = 1;\n");
    }

    #[test]
    fn md_include_target_parses() {
        // "./"-prefixed paths join fine as-is
        assert_eq!(
            md_include_target("<!--@include: ./parts/x.md-->").unwrap(),
            PathBuf::from("./parts/x.md")
        );
        assert_eq!(
            md_include_target("<!--@include: parts/x.md-->").unwrap(),
            PathBuf::from("parts/x.md")
        );
    }
}

#[cfg(test)]
mod edge_tests {
    use super::*;

    #[test]
    fn container_directly_after_paragraph_no_blank() {
        let out = expand_containers("a paragraph\n::: tip\ninner\n:::\nafter\n");
        // the HTML block must interrupt the paragraph correctly at the
        // comrak layer; here we check the wrapper survives preprocessing
        assert!(out.contains("<div class=\"custom-block tip\">"), "{out}");
        assert!(out.contains("</div>"));
    }

    #[test]
    fn adjacent_containers() {
        let out = expand_containers("::: tip\na\n:::\n::: warning\nb\n:::\n");
        assert_eq!(out.matches("<div class=\"custom-block").count(), 2, "{out}");
        assert_eq!(out.matches("</div>").count(), 2, "{out}");
    }

    #[test]
    fn tilde_fences_protected() {
        let out = expand_containers("~~~\n::: tip\n~~~\n::: info\nreal\n:::\n");
        // ::: inside the tilde fence is literal; the real one expands
        assert!(out.contains("::: tip"), "{out}");
        assert!(out.contains("<div class=\"custom-block info\">"), "{out}");
    }

    #[test]
    fn fence_with_trailing_spaces_and_info_closer() {
        // closer with trailing spaces; opener with info string
        let out = expand_containers("```js \n::: tip\n``` \n");
        assert!(out.contains("::: tip"), "container inside fence stays literal: {out}");
    }

    #[test]
    fn unbalanced_container_closed_at_eof() {
        let out = expand_containers("::: tip\nnever closed\n");
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
        assert_eq!(rewrite_info("md:line-numbers=2"), "gdcode lang=md");
    }
}
