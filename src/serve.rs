//! `rustpress serve`: build once, watch the site for changes, rebuild on
//! them, and serve `public/` over HTTP. `/@rustpress/livereload` is an
//! SSE stream every HTML page subscribes to (one injected `<script>`),
//! so edits show up in the browser without a manual refresh.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;

use crate::render::Site;

/// Everything the server task needs.
struct ServeState {
    root: PathBuf,
    /// The site's `base`, refreshed after every rebuild so a config edit
    /// takes effect without restarting the server.
    base: std::sync::RwLock<String>,
}

impl ServeState {
    fn new(root: PathBuf, base: String) -> ServeState {
        ServeState {
            root,
            base: std::sync::RwLock::new(base),
        }
    }
}

pub async fn run(site_dir: PathBuf, port: u16) -> anyhow::Result<()> {
    // notify reports absolute paths, so the `public/` filter in the
    // watcher only works against an absolute site dir
    let site_dir = site_dir
        .canonicalize()
        .with_context(|| format!("cannot resolve site dir {}", site_dir.display()))?;
    // initial build (fail hard: a broken site should not serve stale output)
    let base = rebuild(&site_dir)?;
    let root = site_dir.join("public");

    let state = Arc::new(ServeState::new(root.clone(), base.clone()));

    // watcher: any change under the site dir (minus public/) rebuilds
    start_watcher(site_dir.clone(), Arc::clone(&state))?;

    let app = Router::new()
        .route("/@rustpress/livereload", get(livereload))
        .fallback(move |uri: axum::http::Uri| {
            let state = Arc::clone(&state);
            async move { serve_file(&state, &uri).await }
        })
        .with_state(());

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    println!(
        "rustpress: serving {} on http://{addr}{base}",
        root.display()
    );

    axum::serve(listener, app).await.context("server")
}

/// Rebuild the site into `public/`; returns the config's `base`, the
/// URL prefix the server maps onto `public/`.
fn rebuild(site_dir: &Path) -> anyhow::Result<String> {
    let site = Site::load(site_dir).context("loading site")?;
    let out = site_dir.join("public");
    site.build(site_dir, &out)?;
    Ok(site.config.base)
}

fn start_watcher(site_dir: PathBuf, state: Arc<ServeState>) -> anyhow::Result<()> {
    use notify::Watcher as _;
    let (tx, rx) = std::sync::mpsc::channel();
    // created on this thread so a failure returns Err from `serve` — a
    // panicking watcher thread would leave the server running without
    // rebuilds or live reload
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        // reads (the build itself opens content/, static/ and the
        // config) must not count as changes, or every rebuild
        // triggers the next one
        if let Ok(ev) = res
            && !matches!(ev.kind, notify::EventKind::Access(_))
        {
            let _ = tx.send(ev.paths);
        }
    })
    .context("starting the file watcher")?;
    watcher
        .watch(&site_dir, notify::RecursiveMode::Recursive)
        .with_context(|| format!("watching {}", site_dir.display()))?;
    std::thread::spawn(move || {
        // the watcher must outlive the loop or no events are produced
        let _keep_alive = watcher;
        let public = site_dir.join("public");
        let is_change = |paths: &[PathBuf]| !is_build_output(paths, &public);
        while let Ok(paths) = rx.recv() {
            // ignore the build output itself
            if !is_change(&paths) {
                continue;
            }
            // debounce: one save can arrive as several events (editors
            // that write a temp file and rename it produce ~8), and each
            // rebuild also reloads every open browser tab
            while rx.recv_timeout(DEBOUNCE).is_ok() {}
            match rebuild(&site_dir) {
                Ok(base) => *state.base.write().unwrap() = base,
                Err(e) => {
                    eprintln!("rustpress: rebuild failed: {e:#}");
                    continue;
                }
            }
            println!("rustpress: rebuilt");
            // Signal readers via a shared generation counter — the SSE
            // task polls the atomic and emits when it moves.
            GENERATION.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    });
    Ok(())
}

/// Quiet period after the last filesystem event before rebuilding.
const DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(100);

/// Whether every changed path is the build writing its own output:
/// the output dir itself, or one of the swap siblings a managed build
/// moves it through (`public.rustpress-tmp` staging,
/// `public.rustpress-old` retirement). Their events must be ignored or
/// every rebuild would trigger the next one.
fn is_build_output(paths: &[PathBuf], public: &Path) -> bool {
    let swap_prefix = format!(
        "{}.rustpress-",
        public
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
    );
    // an empty batch (notify's occasional pathless events) must count
    // as a change — `all` on an empty iterator would swallow it
    !paths.is_empty()
        && paths.iter().all(|p| {
            if p.starts_with(public) {
                return true;
            }
            p.ancestors().any(|a| {
                a.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(&swap_prefix))
            })
        })
}

static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

async fn livereload()
-> Sse<impl futures_core::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = async_stream::stream! {
        let mut last = GENERATION.load(std::sync::atomic::Ordering::SeqCst);
        loop {
            let now = GENERATION.load(std::sync::atomic::Ordering::SeqCst);
            if now != last {
                last = now;
                yield Ok(Event::default().event("reload").data("1"));
            }
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Serves files under public/, mounted at the site's `base`. Traversal:
/// the path is percent-decoded once, then segment-split, and any `..`
/// segment (or a backslash, a path separator on Windows) is rejected
/// before any filesystem access; symlinks inside public/ are trusted
/// (127.0.0.1 dev server serving its own build output only).
async fn serve_file(state: &ServeState, uri: &axum::http::Uri) -> Response {
    let base = state.base.read().unwrap().clone();
    // `base` starts and ends with '/' (config validation), so dropping
    // its trailing slash leaves the prefix every served path starts with
    let mount = base.trim_end_matches('/');
    let Some(path) = uri
        .path()
        .strip_prefix(mount)
        .filter(|rest| rest.starts_with('/'))
    else {
        return outside_base(&base, uri);
    };
    let path = match path {
        "/" => "/index.html",
        p if p.ends_with('/') => &format!("{p}index.html"),
        p => p,
    };
    // resolve under root, rejecting traversal
    let Some(rel) = percent_decode(path) else {
        return plain(StatusCode::BAD_REQUEST, "bad encoding");
    };
    if rel.contains('\\') {
        return plain(StatusCode::NOT_FOUND, "not found");
    }
    let mut file_path = state.root.clone();
    for seg in rel.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return plain(StatusCode::NOT_FOUND, "not found");
        }
        file_path.push(seg);
    }
    let meta = tokio::fs::metadata(&file_path).await.ok();
    if !meta.as_ref().is_some_and(|m| m.is_file()) {
        // VitePress's dev server sends a slash-less directory URL to the
        // canonical trailing-slash form instead of a bare 404
        if meta.is_some_and(|m| m.is_dir()) {
            let mut location = format!("{mount}{path}");
            if !location.ends_with('/') {
                location.push('/');
            }
            if let Some(q) = uri.query() {
                location.push('?');
                location.push_str(q);
            }
            return redirect(&location);
        }
        // directory-style 404: serve the custom 404 page
        let not_found = state.root.join("404.html");
        if let Ok(body) = tokio::fs::read(&not_found).await {
            return html_response(StatusCode::NOT_FOUND, body);
        }
        return plain(StatusCode::NOT_FOUND, "not found");
    }
    match tokio::fs::read(&file_path).await {
        Ok(body) => {
            // livereload injection follows the decoded path: a
            // percent-encoded dot would otherwise skip it
            if rel.ends_with(".html") {
                html_response(StatusCode::OK, inject_livereload(body))
            } else {
                let mime = mime_of(&file_path);
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, mime)
                    .body(Body::from(body))
                    .unwrap()
            }
        }
        Err(_) => plain(StatusCode::NOT_FOUND, "not found"),
    }
}

/// A request outside a non-root `base`, answered like Vite's base
/// middleware (what `vitepress dev` runs): the bare origin redirects
/// into the base, anything else is a 404 naming the based URL.
fn outside_base(base: &str, uri: &axum::http::Uri) -> Response {
    let path = uri.path();
    let query = uri.query().map(|q| format!("?{q}")).unwrap_or_default();
    if path == "/" || path == "/index.html" {
        return redirect(&format!("{base}{query}"));
    }
    let suggestion = if format!("{path}/") == base {
        format!("{base}{query}")
    } else {
        format!("{}{path}{query}", base.trim_end_matches('/'))
    };
    plain(
        StatusCode::NOT_FOUND,
        &format!(
            "The server is configured with a public base URL of {base} - did you mean to visit {suggestion} instead?"
        ),
    )
}

fn inject_livereload(body: Vec<u8>) -> Vec<u8> {
    // One EventSource per origin, not per tab: browsers allow only six
    // HTTP/1.1 connections per host, so a seventh tab would hang while
    // six tabs each hold a live-reload stream. The tab holding the Web
    // Lock connects and rebroadcasts; the others listen on the channel
    // and take the lock over when it closes.
    const SCRIPT: &[u8] = b"<script>(function(){var ch='BroadcastChannel' in window?new BroadcastChannel('rustpress-livereload'):null;if(ch)ch.onmessage=function(){location.reload()};function connect(){new EventSource('/@rustpress/livereload').addEventListener('reload',function(){if(ch)ch.postMessage('reload');location.reload()})}if(ch&&navigator.locks)navigator.locks.request('rustpress-livereload',function(){connect();return new Promise(function(){})});else connect()})();</script>";
    if let Ok(s) = std::str::from_utf8(&body)
        && let Some(i) = s.rfind("</body>")
    {
        let mut out = body;
        out.splice(i..i, SCRIPT.iter().copied());
        return out;
    }
    let mut out = body;
    out.extend_from_slice(SCRIPT);
    out
}

fn html_response(status: StatusCode, body: Vec<u8>) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .unwrap()
}

fn plain(status: StatusCode, msg: &str) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_owned()))
        .unwrap()
}

/// 302, not 301: RFC 9110 §15.4.2 lets a cache reuse a 301 without
/// revalidation, and a dev server's directory layout changes under the
/// author, so a browser must ask again next time (§15.4.3: a 302 is
/// cacheable only with explicit freshness information, which this
/// response does not carry).
fn redirect(location: &str) -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, location)
        .body(Body::empty())
        .unwrap()
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 3 <= bytes.len() => {
                let hex = bytes.get(i + 1..i + 3)?;
                let hex_str = std::str::from_utf8(hex).ok()?;
                let byte = u8::from_str_radix(hex_str, 16).ok()?;
                out.push(byte);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

fn mime_of(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml",
        Some("webmanifest") => "application/manifest+json",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Uri;

    fn temp_site() -> PathBuf {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "rp-serve-test-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("public/sub")).unwrap();
        std::fs::write(dir.join("public/index.html"), "<html></html>").unwrap();
        std::fs::write(dir.join("public/sub/page.html"), "<html></html>").unwrap();
        std::fs::write(dir.join("public/data.txt"), "hello").unwrap();
        dir
    }

    async fn status_of(root: &Path, path: &'static str) -> StatusCode {
        let state = ServeState::new(root.to_path_buf(), "/".into());
        serve_file(&state, &Uri::from_static(path)).await.status()
    }

    #[tokio::test]
    async fn serves_files_and_rejects_traversal() {
        let dir = temp_site();
        let public = dir.join("public");
        assert_eq!(status_of(&public, "/").await, StatusCode::OK);
        assert_eq!(status_of(&public, "/data.txt").await, StatusCode::OK);
        assert_eq!(status_of(&public, "/sub/page.html").await, StatusCode::OK);
        assert_eq!(
            status_of(&public, "/missing.txt").await,
            StatusCode::NOT_FOUND
        );
        // traversal, raw and percent-encoded
        assert_eq!(
            status_of(&public, "/../rustpress.toml").await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_of(&public, "/%2e%2e/rustpress.toml").await,
            StatusCode::NOT_FOUND
        );
        // double-encoded decodes to a literal "%2e%2e" name: no second pass
        assert_eq!(
            status_of(&public, "/%252e%252e/rustpress.toml").await,
            StatusCode::NOT_FOUND
        );
        // a backslash is a path separator on Windows: never let one ride
        // through a segment ("..\..\rustpress.toml")
        assert_eq!(
            status_of(&public, "/..%5C..%5Crustpress.toml").await,
            StatusCode::NOT_FOUND
        );

        // Content-Type follows the DECODED path: /index%2Ehtml is
        // index.html and must be served as HTML, not octet-stream
        let state = ServeState::new(public.clone(), "/".into());
        let resp = serve_file(&state, &Uri::from_static("/index%2Ehtml")).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        let resp = serve_file(&state, &Uri::from_static("/data.txt")).await;
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/plain; charset=utf-8"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn watcher_failures_return_err() {
        // startup must fail loudly instead of panicking inside the
        // watcher thread while the server keeps running without reloads
        let state = Arc::new(ServeState::new(PathBuf::new(), "/".into()));
        let err = start_watcher(
            std::env::temp_dir().join("rp-no-such-dir-for-watcher"),
            state,
        );
        assert!(err.is_err(), "watching a nonexistent dir must return Err");
    }

    #[test]
    fn rebuild_swaps_never_count_as_content_changes() {
        // a managed build swaps public/ through two siblings — the
        // staging dir and the retired dir. Events from either must be
        // classified as build output, or every rebuild triggers the
        // next one and serve never settles
        let site = Path::new("/site");
        let public = site.join("public");
        let output = |rel: &str| vec![site.join(rel)];
        assert!(is_build_output(&output("public/index.html"), &public));
        assert!(is_build_output(
            &output("public.rustpress-tmp/x.html"),
            &public
        ));
        assert!(
            is_build_output(&output("public.rustpress-old/404.html"), &public),
            "the retired dir's removal events must be filtered or serve rebuild-storms"
        );
        assert!(!is_build_output(&output("content/index.md"), &public));
        assert!(!is_build_output(&output("rustpress.toml"), &public));
        assert!(!is_build_output(&output("public-other/x.md"), &public));
    }

    #[test]
    fn pathless_watcher_events_still_count_as_changes() {
        // notify emits the occasional pathless event (rescans, other);
        // an empty batch must count as a change, not be swallowed
        assert!(!is_build_output(&[], Path::new("/site/public")));
    }

    #[tokio::test]
    async fn slash_less_directory_urls_redirect_to_the_trailing_slash() {
        // VitePress's dev server sends /sub to the canonical /sub/
        // instead of a bare 404; hand-typed or external links rely on it
        let dir = temp_site();
        std::fs::write(dir.join("public/sub/index.html"), "<html></html>").unwrap();
        let public = dir.join("public");
        let state = ServeState::new(public.clone(), "/".into());
        let resp = serve_file(&state, &Uri::from_static("/sub")).await;
        // temporary: a cached 301 would outlive the next rebuild
        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(
            resp.headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some("/sub/")
        );
        // a slash-less path with no directory behind it still 404s
        assert_eq!(
            status_of(&public, "/no-such-dir").await,
            StatusCode::NOT_FOUND
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn location(resp: &Response) -> Option<&str> {
        resp.headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
    }

    async fn body_text(resp: Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn serves_the_build_under_its_base() {
        // a `base = "/docs/"` build links /docs/vitepress.css and
        // /docs/guide/…: the server answers under the base the way
        // `vitepress dev` (Vite's base middleware) does
        let dir = temp_site();
        std::fs::write(dir.join("public/404.html"), "<html>custom 404</html>").unwrap();
        let state = ServeState::new(dir.join("public"), "/docs/".into());
        let get = |p: &'static str| {
            let state = &state;
            async move { serve_file(state, &Uri::from_static(p)).await }
        };

        assert_eq!(get("/docs/").await.status(), StatusCode::OK);
        assert_eq!(get("/docs/data.txt").await.status(), StatusCode::OK);
        assert_eq!(get("/docs/sub/page.html").await.status(), StatusCode::OK);
        // a miss inside the base gets the site's own 404 page
        let resp = get("/docs/missing/").await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(body_text(resp).await, "<html>custom 404</html>");
        // the slash-less directory redirect keeps the base
        let resp = get("/docs/sub").await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(location(&resp), Some("/docs/sub/"));
        // traversal is still rejected after the base is stripped
        assert_eq!(
            get("/docs/../rustpress.toml").await.status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            get("/docs/%2e%2e/rustpress.toml").await.status(),
            StatusCode::NOT_FOUND
        );

        // the bare origin redirects into the base, query intact
        let resp = get("/?q=1").await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(location(&resp), Some("/docs/?q=1"));
        let resp = get("/index.html").await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(location(&resp), Some("/docs/"));
        // anything else outside the base is a 404 naming the based URL,
        // never a file from the root of public/
        let resp = get("/data.txt?x=1").await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert!(
            body_text(resp)
                .await
                .contains("did you mean to visit /docs/data.txt?x=1 instead?")
        );
        let resp = get("/docs").await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert!(
            body_text(resp)
                .await
                .contains("did you mean to visit /docs/ instead?")
        );

        // a rebuild that changes `base` moves the served tree with it
        *state.base.write().unwrap() = "/v2/".into();
        assert_eq!(get("/v2/data.txt").await.status(), StatusCode::OK);
        assert_eq!(get("/docs/data.txt").await.status(), StatusCode::NOT_FOUND);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
