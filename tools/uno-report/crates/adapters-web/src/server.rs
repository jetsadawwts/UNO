//! Local live server (tokio + axum): `/` the page, `/version` its fingerprint, `/ws` pane updates.
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Request, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, HOST, ORIGIN, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use tokio::sync::watch;

use app::ports::{Clock, ManualSource, ProjectStore, SessionProbe};

use crate::{diff_message, render_page, render_panes, PageMode, Panes};

/// Most WebSocket connections open at once; the next one gets 503.
const MAX_SOCKETS: usize = 8;
const PING_EVERY: Duration = Duration::from_secs(20);

/// Binds 127.0.0.1 only. Every route refuses a Host header that is not this machine (so another
/// site cannot read the data through DNS rebinding); `/ws` also refuses a foreign Origin.
pub struct Server<S, M, P, C, F> {
    pub port: u16,
    pub store: S,
    pub manual: M,
    pub sessions: P,
    pub clock: C,
    /// Shown in the page header as the data source.
    pub data_label: String,
    /// Fingerprint of the manual files, for `/version`.
    pub manual_stamp: F,
    /// Latest signature from the watcher; every change makes each socket send its pane diff.
    pub changes: watch::Receiver<String>,
}

struct Shared<S, M, P, C, F> {
    server: Server<S, M, P, C, F>,
    sockets: AtomicUsize,
}

/// Frees a socket slot when the connection task ends, however it ends.
struct Slot<'a>(&'a AtomicUsize);
impl Drop for Slot<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

fn plain(code: StatusCode, body: &'static str) -> Response {
    (code, [(CONTENT_TYPE, "text/plain")], body).into_response()
}

impl<S, M, P, C, F> Server<S, M, P, C, F>
where
    S: ProjectStore + Send + Sync + 'static,
    M: ManualSource + Send + Sync + 'static,
    P: SessionProbe + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
    F: Fn() -> String + Send + Sync + 'static,
{
    /// The routes and checks, without a listener (tests serve it on their own port).
    pub fn router(self) -> Router {
        let port = self.port;
        let shared = Arc::new(Shared { server: self, sockets: AtomicUsize::new(0) });
        Router::new()
            .route("/", get(page::<S, M, P, C, F>))
            .route("/version", get(version::<S, M, P, C, F>))
            .route("/ws", get(socket::<S, M, P, C, F>))
            .fallback(|| async { plain(StatusCode::NOT_FOUND, "not found") })
            .method_not_allowed_fallback(|| async { plain(StatusCode::METHOD_NOT_ALLOWED, "GET only") })
            .with_state(shared)
            .layer(middleware::from_fn(move |req: Request, next: Next| host_check(port, req, next)))
    }

    /// Listen until the process is stopped. Exits with status 1 when the port cannot be bound.
    pub async fn run(self, open: bool) {
        let port = self.port;
        let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("uno-report: cannot listen on 127.0.0.1:{port}: {e}");
                std::process::exit(1);
            }
        };
        let url = format!("http://localhost:{port}/");
        println!("uno-report: live dashboard at {url} (Ctrl+C to stop)");
        if open {
            let _ = Command::new("open").arg(&url).status();
        }
        if let Err(e) = axum::serve(listener, self.router()).await {
            eprintln!("uno-report: server stopped: {e}");
            std::process::exit(1);
        }
    }
}

/// Host check on every request, then the same safety headers on every response.
async fn host_check(port: u16, req: Request, next: Next) -> Response {
    let host = req.headers().get(HOST).and_then(|h| h.to_str().ok()).unwrap_or("");
    let ok = host == format!("localhost:{port}") || host == format!("127.0.0.1:{port}");
    let mut res = if ok { next.run(req).await } else { plain(StatusCode::FORBIDDEN, "forbidden") };
    let h = res.headers_mut();
    h.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    h.insert(REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
    res
}

type St<S, M, P, C, F> = State<Arc<Shared<S, M, P, C, F>>>;

async fn page<S, M, P, C, F>(State(sh): St<S, M, P, C, F>) -> Response
where
    S: ProjectStore + Send + Sync + 'static,
    M: ManualSource + Send + Sync + 'static,
    P: SessionProbe + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
    F: Fn() -> String + Send + Sync + 'static,
{
    let r = tokio::task::spawn_blocking(move || {
        let s = &sh.server;
        app::build_dashboard(&s.store, &s.sessions, &s.clock)
            .map(|d| render_page(&d, &s.manual.manual(), &s.data_label, &PageMode::Live))
    })
    .await;
    match r {
        Ok(Ok(html)) => ([(CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response(),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, [(CONTENT_TYPE, "text/plain; charset=utf-8")], e).into_response(),
        Err(_) => plain(StatusCode::INTERNAL_SERVER_ERROR, "render failed"),
    }
}

async fn version<S, M, P, C, F>(State(sh): St<S, M, P, C, F>) -> Response
where
    S: ProjectStore + Send + Sync + 'static,
    M: ManualSource + Send + Sync + 'static,
    P: SessionProbe + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
    F: Fn() -> String + Send + Sync + 'static,
{
    let sig = tokio::task::spawn_blocking(move || {
        let s = &sh.server;
        app::signature(&s.store, &(s.manual_stamp)(), &s.sessions)
    })
    .await
    .unwrap_or_default();
    ([(CONTENT_TYPE, "text/plain")], sig).into_response()
}

async fn socket<S, M, P, C, F>(State(sh): St<S, M, P, C, F>, headers: HeaderMap, up: WebSocketUpgrade) -> Response
where
    S: ProjectStore + Send + Sync + 'static,
    M: ManualSource + Send + Sync + 'static,
    P: SessionProbe + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
    F: Fn() -> String + Send + Sync + 'static,
{
    let port = sh.server.port;
    if let Some(o) = headers.get(ORIGIN) {
        let o = o.to_str().unwrap_or("");
        if o != format!("http://localhost:{port}") && o != format!("http://127.0.0.1:{port}") {
            return plain(StatusCode::FORBIDDEN, "forbidden");
        }
    }
    let taken = sh.sockets.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| (n < MAX_SOCKETS).then_some(n + 1));
    if taken.is_err() {
        return plain(StatusCode::SERVICE_UNAVAILABLE, "too many live pages");
    }
    up.on_upgrade(move |ws| async move {
        let _slot = Slot(&sh.sockets);
        live(ws, sh.clone()).await;
    })
}

/// Render the panes and the current signature off the async threads.
async fn snapshot<S, M, P, C, F>(sh: &Arc<Shared<S, M, P, C, F>>) -> Option<(String, Panes)>
where
    S: ProjectStore + Send + Sync + 'static,
    M: ManualSource + Send + Sync + 'static,
    P: SessionProbe + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
    F: Fn() -> String + Send + Sync + 'static,
{
    let sh = sh.clone();
    tokio::task::spawn_blocking(move || {
        let s = &sh.server;
        let d = app::build_dashboard(&s.store, &s.sessions, &s.clock).ok()?;
        let sig = app::signature(&s.store, &(s.manual_stamp)(), &s.sessions);
        Some((sig, render_panes(&d, &s.manual.manual(), &s.data_label)))
    })
    .await
    .ok()
    .flatten()
}

/// One socket: full panes on connect, then only the panes that changed. Client messages are
/// read (so pings and closes work) and otherwise dropped.
async fn live<S, M, P, C, F>(mut ws: WebSocket, sh: Arc<Shared<S, M, P, C, F>>)
where
    S: ProjectStore + Send + Sync + 'static,
    M: ManualSource + Send + Sync + 'static,
    P: SessionProbe + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
    F: Fn() -> String + Send + Sync + 'static,
{
    let mut changes = sh.server.changes.clone();
    let mut last: Option<Panes> = None;
    let mut ping = tokio::time::interval(PING_EVERY);
    ping.tick().await;
    let mut send_now = true;
    loop {
        if send_now {
            send_now = false;
            if let Some((sig, panes)) = snapshot(&sh).await {
                let msg = diff_message(&sig, last.as_ref(), &panes);
                if ws.send(Message::Text(msg.into())).await.is_err() {
                    return;
                }
                last = Some(panes);
            }
        }
        tokio::select! {
            r = changes.changed() => {
                if r.is_err() {
                    return;
                }
                send_now = true;
            }
            _ = ping.tick() => {
                if ws.send(Message::Ping(Vec::new().into())).await.is_err() {
                    return;
                }
            }
            m = ws.recv() => match m {
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => return,
                Some(Ok(_)) => {}
            },
        }
    }
}
