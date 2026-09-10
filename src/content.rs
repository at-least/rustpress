//! Content loading: walk the site's `content/` tree of VitePress-format
//! Markdown pages (`---` YAML front matter, H1-derived titles), map files
//! to canonical URLs (`guide/x.md` → `/guide/x/`, `index.md` → the
//! directory root), and expose the page set for sidebar derivation and
//! rendering.
//!
//! URL scheme: every page gets a trailing-slash directory URL so the
//! static output is `url + "index.html"` and no-host-rewrite hosting
//! works. The config `base` is joined at render time, not here.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Deserialize;

/// A page's `---` YAML front matter. Unknown keys are allowed (VitePress
/// exposes them to templates) and ignored.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageFrontMatter {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// `doc` (default), `home`, `page`.
    #[serde(default)]
    pub layout: Option<String>,
    /// `deep` or a level/level-pair overriding the site outline depth.
    #[serde(default)]
    pub outline: Option<OutlineSetting>,
    #[serde(default)]
    pub hero: Option<Hero>,
    #[serde(default)]
    pub features: Vec<Feature>,
}

/// `outline: deep` | `outline: 2` | `outline: [2, 3]`. Hand-written
/// `Deserialize` because an untagged unit variant only matches null, not
/// the string `"deep"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineSetting {
    Deep,
    Level(u8),
    Range((u8, u8)),
}

impl<'de> Deserialize<'de> for OutlineSetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Word(String),
            Level(u8),
            Pair((u8, u8)),
        }
        match Raw::deserialize(deserializer)? {
            Raw::Word(w) if w == "deep" => Ok(OutlineSetting::Deep),
            Raw::Word(other) => Err(D::Error::custom(format!(
                "unknown outline setting {other:?}: expected \"deep\" or a heading level"
            ))),
            Raw::Level(l) => Ok(OutlineSetting::Level(l)),
            Raw::Pair((a, b)) => Ok(OutlineSetting::Range((a, b))),
        }
    }
}

/// Home-page hero (`layout: home`).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hero {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub tagline: Option<String>,
    #[serde(default)]
    pub image: Option<HeroImage>,
    #[serde(default)]
    pub actions: Vec<HeroAction>,
}

/// `hero.image`: a path, `{ src, alt }`, or `{ light, dark, alt }`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum HeroImage {
    Simple(String),
    Detailed {
        src: String,
        #[serde(default)]
        alt: Option<String>,
    },
    Dual {
        light: String,
        dark: String,
        #[serde(default)]
        alt: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeroAction {
    /// `brand` (default) | `alt`.
    #[serde(default)]
    pub theme: Option<String>,
    pub text: String,
    pub link: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub rel: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    /// Emoji/text, `{ src, alt, width, height }`, or `{ light, dark }`.
    #[serde(default)]
    pub icon: Option<FeatureIcon>,
    pub title: String,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub link_text: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub rel: Option<String>,
}

/// `features[].icon` (VitePress `FeatureIcon`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum FeatureIcon {
    Text(String),
    Image {
        src: String,
        #[serde(default)]
        alt: Option<String>,
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
    },
    Dual {
        light: String,
        dark: String,
        #[serde(default)]
        alt: Option<String>,
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
    },
}

/// One loaded page.
#[derive(Debug, Clone)]
pub struct Page {
    /// Path relative to the content root, with `/` separators
    /// (`guide/getting-started.md`).
    pub rel: String,
    /// Canonical base-free URL with trailing slash (`/guide/getting-started/`).
    pub url: String,
    /// Front-matter title, else the first H1's text, else the file stem.
    pub title: String,
    pub front: PageFrontMatter,
    /// Markdown body with the front matter block removed.
    pub body: String,
    /// File modification time (for `lastUpdated`).
    pub modified: Option<SystemTime>,
    /// Raw source path (diagnostics).
    pub src: PathBuf,
    /// Locale key from the site config (`"root"` when unassigned).
    pub locale: String,
}

impl Page {
    /// `layout: home` pages.
    pub fn is_home(&self) -> bool {
        self.front.layout.as_deref() == Some("home")
    }
}

/// All pages of a site, ordered by a natural sort of their source paths;
/// `by_url` maps canonical URL to index into `pages`.
#[derive(Debug, Clone, Default)]
pub struct Content {
    pub pages: Vec<Page>,
    pub by_url: BTreeMap<String, usize>,
    /// Source-rel-path index (e.g. `guide/x.md`) — link resolution and
    /// locale assignment work in source space.
    pub by_rel: BTreeMap<String, usize>,
}

impl Content {
    pub fn get(&self, url: &str) -> Option<&Page> {
        self.by_url.get(url).map(|&i| &self.pages[i])
    }

    /// Load every `.md` under `content_dir` (dotfiles/dirs skipped).
    /// `excludes` are `srcExclude` glob patterns (`*` within a segment,
    /// `**` across segments) matched against source-relative paths.
    pub fn load(content_dir: &Path, excludes: &[String]) -> Result<Content, ContentError> {
        let mut content = Content::default();
        let mut seen = std::collections::HashSet::new();
        walk(content_dir, "", &mut content, &mut seen, excludes)?;
        content.pages.sort_by(|a, b| natural_cmp(&a.url, &b.url));
        content.by_url = content
            .pages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.url.clone(), i))
            .collect();
        content.by_rel = content
            .pages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.rel.clone(), i))
            .collect();
        Ok(content)
    }
}

fn walk(
    dir: &Path,
    rel_dir: &str,
    out: &mut Content,
    seen: &mut std::collections::HashSet<String>,
    excludes: &[String],
) -> Result<(), ContentError> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|source| ContentError::Read {
            path: dir.to_path_buf(),
            source,
        })?
        .collect::<Result<_, _>>()
        .map_err(|source| ContentError::Read {
            path: dir.to_path_buf(),
            source,
        })?;
    entries.sort_by(|a, b| natural_cmp(&a.file_name().to_string_lossy(), &b.file_name().to_string_lossy()));
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let rel = if rel_dir.is_empty() { name.clone() } else { format!("{rel_dir}/{name}") };
        let path = entry.path();
        if path.is_dir() {
            walk(&path, &rel, out, seen, excludes)?;
        } else if name.ends_with(".md")
            && !excludes.iter().any(|p| glob_match(p, &rel))
        {
            let page = load_page(&path, &rel)?;
            if !seen.insert(page.url.clone()) {
                return Err(ContentError::Duplicate { url: page.url });
            }
            out.pages.push(page);
        }
    }
    Ok(())
}

/// `srcExclude` glob: `*` matches within a path segment, `**` across
/// segments, everything else matches literally (whole path).
pub fn glob_match(pattern: &str, path: &str) -> bool {
    fn go(p: &[char], s: &[char]) -> bool {
        if p.is_empty() {
            return s.is_empty();
        }
        match p[0] {
            '*' if p.len() > 1 && p[1] == '*' => {
                (0..=s.len()).any(|i| go(&p[2..], &s[i..]))
            }
            '*' => {
                // stop at the segment boundary
                (0..=s.len()).take_while(|i| *i == 0 || s[*i - 1] != '/').any(|i| go(&p[1..], &s[i..]))
            }
            c => !s.is_empty() && s[0] == c && go(&p[1..], &s[1..]),
        }
    }
    go(&pattern.chars().collect::<Vec<_>>(), &path.chars().collect::<Vec<_>>())
}

fn load_page(path: &Path, rel: &str) -> Result<Page, ContentError> {
    let raw = std::fs::read_to_string(path).map_err(|source| ContentError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let (front, body) = split_front_matter(&raw);
    let front = match front {
        Some(yaml) => serde_norway::from_str(yaml).map_err(|source| ContentError::FrontMatter {
            path: path.to_path_buf(),
            source,
        })?,
        None => PageFrontMatter::default(),
    };
    let stem = rel.trim_end_matches(".md").trim_end_matches("index").trim_end_matches('/');
    let title = front
        .title
        .clone()
        .or_else(|| extract_h1(body))
        .unwrap_or_else(|| {
            stem.rsplit('/').next().unwrap_or("Untitled").to_string()
        });
    let url = page_url(rel);
    let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
    Ok(Page {
        rel: rel.to_string(),
        url,
        title,
        front,
        body: body.to_string(),
        modified,
        src: path.to_path_buf(),
        locale: "root".to_string(),
    })
}

/// Split a leading `---` YAML front matter block; returns (yaml, body).
pub fn split_front_matter(raw: &str) -> (Option<&str>, &str) {
    let mut lines = raw.split_inclusive('\n');
    let first = lines.next();
    if first.map(|l| l.trim_end()) != Some("---") {
        return (None, raw);
    }
    let after_first = &raw[first.unwrap().len()..];
    let mut offset = 0usize;
    for line in after_first.split_inclusive('\n') {
        if line.trim_end() == "---" {
            let yaml = &after_first[..offset];
            let body = &after_first[offset + line.len()..];
            return (Some(yaml.trim()), body);
        }
        offset += line.len();
    }
    (None, raw)
}

/// The page's canonical URL from its source-relative path:
/// `index.md` → `/`, `guide/index.md` → `/guide/`, `guide/x.md` →
/// `/guide/x/`.
pub fn page_url(rel: &str) -> String {
    let no_ext = rel.strip_suffix(".md").unwrap_or(rel);
    let trimmed = no_ext.strip_suffix("/index").unwrap_or(no_ext);
    if trimmed == "index" || trimmed.is_empty() {
        "/".to_string()
    } else {
        format!("/{trimmed}/")
    }
}

/// First H1's display text: scans before any code fence, strips inline
/// HTML tags (e.g. `<Badge … />` in headings) and trailing `{#anchor}`
/// attributes.
pub fn extract_h1(body: &str) -> Option<String> {
    for line in body.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            break; // headings after the first fence belong to examples
        }
        let Some(rest) = t.strip_prefix("# ") else { continue };
        let rest = rest.trim();
        if rest.is_empty() {
            return None;
        }
        let no_attr = strip_heading_attrs(rest);
        let plain = strip_html_tags(no_attr);
        return Some(plain.trim().to_string());
    }
    None
}

/// Remove a trailing `{#id}` / `{#id .class}` attribute block.
fn strip_heading_attrs(text: &str) -> &str {
    match text.rfind('{') {
        Some(i) if text.ends_with('}') && text[i + 1..].starts_with('#') => &text[..i],
        _ => text,
    }
}

fn strip_html_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Case-insensitive natural sort ("a2" < "a10").
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut ai = a.chars().peekable();
    let mut bi = b.chars().peekable();
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(x), Some(y)) => {
                if x.is_ascii_digit() && y.is_ascii_digit() {
                    let an: String = ai.clone().take_while(|c| c.is_ascii_digit()).collect();
                    let bn: String = bi.clone().take_while(|c| c.is_ascii_digit()).collect();
                    let ord = an
                        .trim_start_matches('0')
                        .len()
                        .cmp(&bn.trim_start_matches('0').len())
                        .then_with(|| an.trim_start_matches('0').cmp(bn.trim_start_matches('0')))
                        .then_with(|| an.len().cmp(&bn.len()));
                    if ord != std::cmp::Ordering::Equal {
                        return ord;
                    }
                    for _ in 0..an.len() {
                        ai.next();
                    }
                    for _ in 0..bn.len() {
                        bi.next();
                    }
                } else {
                    let ord = x.to_lowercase().cmp(y.to_lowercase());
                    if ord != std::cmp::Ordering::Equal {
                        return ord;
                    }
                    ai.next();
                    bi.next();
                }
            }
        }
    }
}

/// Errors loading content.
#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("cannot read {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("invalid front matter in {path}: {source}")]
    FrontMatter {
        path: PathBuf,
        source: serde_norway::Error,
    },
    #[error("duplicate page URL {url:?}")]
    Duplicate { url: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_split_basic() {
        let raw = "---\ndescription: hi\n---\n\n# T\n";
        let (yaml, body) = split_front_matter(raw);
        assert_eq!(yaml, Some("description: hi"));
        assert!(body.starts_with("\n# T"));
    }

    #[test]
    fn front_matter_split_absent_and_unterminated() {
        assert_eq!(split_front_matter("# No FM\n"), (None, "# No FM\n"));
        assert_eq!(split_front_matter("---\nx: 1\n"), (None, "---\nx: 1\n"));
    }

    #[test]
    fn home_front_matter_parses() {
        let fm: PageFrontMatter = serde_norway::from_str(
            "description: d\nlayout: home\nhero:\n  name: V\n  text: T\n  actions:\n    - theme: brand\n      text: Start\n      link: ./guide/\nfeatures:\n  - icon: <span class=\"i\"></span>\n    title: F\n    details: D\n    link: /f/\n    linkText: More\noutline: deep\n",
        )
        .unwrap();
        assert_eq!(fm.layout.as_deref(), Some("home"));
        assert_eq!(fm.outline, Some(OutlineSetting::Deep));
        assert_eq!(fm.hero.as_ref().unwrap().actions[0].text, "Start");
        assert_eq!(fm.features[0].link_text.as_deref(), Some("More"));
        assert!(matches!(
            &fm.features[0].icon,
            Some(FeatureIcon::Text(t)) if t.contains("span")
        ));
    }

    #[test]
    fn outline_setting_forms() {
        assert_eq!(serde_norway::from_str::<OutlineSetting>("deep").unwrap(), OutlineSetting::Deep);
        assert_eq!(serde_norway::from_str::<OutlineSetting>("2").unwrap(), OutlineSetting::Level(2));
        assert_eq!(
            serde_norway::from_str::<OutlineSetting>("[2, 3]").unwrap(),
            OutlineSetting::Range((2, 3))
        );
    }

    #[test]
    fn unknown_front_matter_keys_are_tolerated() {
        let fm: PageFrontMatter = serde_norway::from_str("custom: whatever\n").unwrap();
        assert_eq!(fm, PageFrontMatter::default());
    }

    #[test]
    fn url_mapping() {
        assert_eq!(page_url("index.md"), "/");
        assert_eq!(page_url("guide/index.md"), "/guide/");
        assert_eq!(page_url("guide/x.md"), "/guide/x/");
        assert_eq!(page_url("reference/default-theme/config.md"), "/reference/default-theme/config/");
    }

    #[test]
    fn h1_extraction() {
        assert_eq!(extract_h1("# Plain\n"), Some("Plain".into()));
        assert_eq!(extract_h1("## Not H1\n\n# Real\n"), Some("Real".into()));
        // fence first: no title
        assert_eq!(extract_h1("```md\n# In fence\n```\n# After\n"), None);
        // badge + heading attrs stripped
        assert_eq!(
            extract_h1("# MPA Mode <Badge type=\"warning\" text=\"experimental\" />\n"),
            Some("MPA Mode".into())
        );
        assert_eq!(extract_h1("# Anchored {#custom-id}\n"), Some("Anchored".into()));
        assert_eq!(extract_h1("no heading here\n"), None);
    }

    #[test]
    fn natural_order() {
        let mut v = vec!["a10", "a2", "B1", "a1b"];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(v, vec!["a1b", "a2", "a10", "B1"]);
    }
}
