//! `rustpress serve`: build once, watch the site for changes, rebuild on
//! them, and serve `public/` over HTTP. `/@rustpress/livereload` is an
//! SSE stream every HTML page subscribes to (one injected `<script>`),
//! so edits show up in the browser without a manual refresh.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::Response;
use axum::routing::get;
use axum::Router;

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
    // dev convenience: when serving a site nested under a repo whose
    // root has a static/ dir (the dogfood layout), layer it on top so
    // the committed built assets (app.js, main.css) resolve
    if let Some(root) = site_dir.parent() {
        let root_static = root.join("static");
        if root_static.is_dir() {
            let _ = copy_over(&root_static, &out);
        }
    }
    Ok(())
}

fn copy_over(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_over(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn start_watcher(site_dir: PathBuf) -> anyhow::Result<()> {
    let _ = site_dir; // used via recursive watcher below
    std::thread::spawn(move || {
        use notify::Watcher as _;
        let (tx, rx) = std::sync::mpsc::channel();
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
        .unwrap_or_else(|e| panic!("rustpress: cannot start file watcher: {e}"));
        watcher
            .watch(&site_dir, notify::RecursiveMode::Recursive)
            .unwrap_or_else(|e| panic!("rustpress: cannot watch {}: {e}", site_dir.display()));
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

async fn livereload() -> Sse<impl futures_core::Stream<Item = Result<Event, std::convert::Infallible>>> {
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
/// once, then segment-split, and any `..` segment is rejected before any
/// filesystem access; symlinks inside public/ are trusted (127.0.0.1 dev
/// server serving its own build output only).
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
            if path.ends_with(".html") {
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
        && let Some(i) = s.rfind("</body>") {
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
            b'%' if i + 2 < bytes.len() + 1 && i + 2 < bytes.len() + 1 => {
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
