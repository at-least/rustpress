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
}

pub async fn run(site_dir: PathBuf, port: u16) -> anyhow::Result<()> {
    // notify reports absolute paths, so the `public/` filter in the
    // watcher only works against an absolute site dir
    let site_dir = site_dir
        .canonicalize()
        .with_context(|| format!("cannot resolve site dir {}", site_dir.display()))?;
    // initial build (fail hard: a broken site should not serve stale output)
    rebuild(&site_dir)?;
    let root = site_dir.join("public");

    let state = Arc::new(ServeState { root: root.clone() });

    // watcher: any change under the site dir (minus public/) rebuilds
    start_watcher(site_dir.clone())?;

    let app = Router::new()
        .route("/@rustpress/livereload", get(livereload))
        .fallback(move |uri: axum::http::Uri| {
            let state = Arc::clone(&state);
            async move { serve_file(&state, &uri) }
        })
        .with_state(());

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    println!("rustpress: serving {} on http://{addr}", root.display());

    axum::serve(listener, app).await.context("server")
}

/// Rebuild the site; returns whether the build changed anything on disk
/// (best-effort mtime snapshot comparison is unnecessary — every build
/// bumps the generation, and the browser reload is cheap).
fn rebuild(site_dir: &Path) -> anyhow::Result<()> {
    let site = Site::load(site_dir).context("loading site")?;
    let out = site_dir.join("public");
    site.build(site_dir, &out)?;
    Ok(())
}

fn start_watcher(site_dir: PathBuf) -> anyhow::Result<()> {
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
        let is_change = |paths: &[PathBuf]| !paths.iter().any(|p| p.starts_with(&public));
        while let Ok(paths) = rx.recv() {
            // ignore the build output itself
            if !is_change(&paths) {
                continue;
            }
            // debounce: one save can arrive as several events (editors
            // that write a temp file and rename it produce ~8), and each
            // rebuild also reloads every open browser tab
            while rx.recv_timeout(DEBOUNCE).is_ok() {}
            if let Err(e) = rebuild(&site_dir) {
                eprintln!("rustpress: rebuild failed: {e:#}");
                continue;
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

/// Serves files under public/. Traversal: the path is percent-decoded
/// once, then segment-split, and any `..` segment (or a backslash, a
/// path separator on Windows) is rejected before any filesystem access;
/// symlinks inside public/ are trusted (127.0.0.1 dev server serving its
/// own build output only).
fn serve_file(state: &ServeState, uri: &axum::http::Uri) -> Response {
    let path = uri.path();
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
    if !file_path.is_file() {
        // directory-style 404: serve the custom 404 page
        let not_found = state.root.join("404.html");
        if let Ok(body) = std::fs::read(&not_found) {
            return html_response(StatusCode::NOT_FOUND, body);
        }
        return plain(StatusCode::NOT_FOUND, "not found");
    }
    match std::fs::read(&file_path) {
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
        let dir = std::env::temp_dir().join(format!("rp-serve-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("public/sub")).unwrap();
        std::fs::write(dir.join("public/index.html"), "<html></html>").unwrap();
        std::fs::write(dir.join("public/sub/page.html"), "<html></html>").unwrap();
        std::fs::write(dir.join("public/data.txt"), "hello").unwrap();
        dir
    }

    fn status_of(root: &Path, path: &'static str) -> StatusCode {
        let state = ServeState {
            root: root.to_path_buf(),
        };
        serve_file(&state, &Uri::from_static(path)).status()
    }

    #[test]
    fn serves_files_and_rejects_traversal() {
        let dir = temp_site();
        let public = dir.join("public");
        assert_eq!(status_of(&public, "/"), StatusCode::OK);
        assert_eq!(status_of(&public, "/data.txt"), StatusCode::OK);
        assert_eq!(status_of(&public, "/sub/page.html"), StatusCode::OK);
        assert_eq!(status_of(&public, "/missing.txt"), StatusCode::NOT_FOUND);
        // traversal, raw and percent-encoded
        assert_eq!(
            status_of(&public, "/../rustpress.toml"),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_of(&public, "/%2e%2e/rustpress.toml"),
            StatusCode::NOT_FOUND
        );
        // double-encoded decodes to a literal "%2e%2e" name: no second pass
        assert_eq!(
            status_of(&public, "/%252e%252e/rustpress.toml"),
            StatusCode::NOT_FOUND
        );
        // a backslash is a path separator on Windows: never let one ride
        // through a segment ("..\..\rustpress.toml")
        assert_eq!(
            status_of(&public, "/..%5C..%5Crustpress.toml"),
            StatusCode::NOT_FOUND
        );

        // Content-Type follows the DECODED path: /index%2Ehtml is
        // index.html and must be served as HTML, not octet-stream
        let state = ServeState {
            root: public.clone(),
        };
        let resp = serve_file(&state, &Uri::from_static("/index%2Ehtml"));
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        let resp = serve_file(&state, &Uri::from_static("/data.txt"));
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
        let err = start_watcher(std::env::temp_dir().join("rp-no-such-dir-for-watcher"));
        assert!(err.is_err(), "watching a nonexistent dir must return Err");
    }
}
