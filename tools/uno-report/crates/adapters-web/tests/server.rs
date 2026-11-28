//! The live server over real sockets: HTTP rules and the pane-diff WebSocket.
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use adapters_web::Server;
use app::ports::*;
use domain::{AuditRecord, Task};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};

#[derive(Clone)]
struct Store(Arc<Mutex<(usize, String)>>); // (project count, event time)
impl ProjectStore for Store {
    fn projects(&self) -> Result<Vec<ProjectFiles>, String> {
        let (n, time) = self.0.lock().unwrap().clone();
        Ok((0..n).map(|i| ProjectFiles {
            name: format!("proj{i}"),
            audit: vec![AuditRecord { event: "smoke".into(), time: time.clone(), ..Default::default() }],
            tasks: vec![Task { id: "T1".into(), title: "x".into(), status: "open".into() }],
        }).collect())
    }
    fn stamp(&self) -> String { format!("{:?}", self.0.lock().unwrap()) }
}
struct Man;
impl ManualSource for Man { fn manual(&self) -> ManualText { ManualText { en: Some("# M".into()), th: None, location: "/m".into() } } }
struct Sess;
impl SessionProbe for Sess { fn open_sessions(&self) -> Option<BTreeMap<String, usize>> { None } }
struct Clk;
impl Clock for Clk { fn now(&self) -> String { "2026-01-15 09:30".into() } }

struct Live { port: u16, store: Store, tx: watch::Sender<String> }

async fn start() -> Live {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let store = Store(Arc::new(Mutex::new((1, "2026-01-01T00:00".into()))));
    let (tx, rx) = watch::channel("s0".to_string());
    fn stamp() -> String { "m".into() }
    let s = Server { port, store: store.clone(), manual: Man, sessions: Sess, clock: Clk, data_label: "/d".into(), manual_stamp: stamp as fn() -> String, changes: rx };
    tokio::spawn(async move { axum::serve(listener, s.router()).await.unwrap() });
    Live { port, store, tx }
}

/// Raw HTTP/1.1 request; returns (status code, head, body).
async fn http(port: u16, method: &str, path: &str, host: &str) -> (u16, String, String) {
    let mut c = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    c.write_all(format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n").as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    c.read_to_end(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf).to_string();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    (head[9..12].parse().unwrap(), head.to_string(), body.to_string())
}

async fn ws(port: u16, origin: Option<&str>) -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>, u16> {
    let mut req = format!("ws://127.0.0.1:{port}/ws").into_client_request().unwrap();
    if let Some(o) = origin {
        req.headers_mut().insert("Origin", o.parse().unwrap());
    }
    match tokio_tungstenite::connect_async(req).await {
        Ok((s, _)) => Ok(s),
        Err(tokio_tungstenite::tungstenite::Error::Http(r)) => Err(r.status().as_u16()),
        Err(e) => panic!("{e}"),
    }
}

async fn next_json<S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin>(s: &mut S) -> serde_json::Value {
    loop {
        let m = tokio::time::timeout(Duration::from_secs(3), s.next()).await.expect("no message").unwrap().unwrap();
        if let Message::Text(t) = m {
            return serde_json::from_str(&t).unwrap();
        }
    }
}

#[tokio::test]
async fn root_ok_and_headers() {
    let l = start().await;
    let (code, head, body) = http(l.port, "GET", "/", &format!("localhost:{}", l.port)).await;
    assert_eq!(code, 200);
    let h = head.to_lowercase();
    for want in ["cache-control: no-store", "x-content-type-options: nosniff", "referrer-policy: no-referrer", "content-type: text/html; charset=utf-8"] {
        assert!(h.contains(want), "missing {want}\n{head}");
    }
    assert!(body.contains("window.UNO_LIVE=true;"));
}

#[tokio::test]
async fn version_is_16_hex_with_headers() {
    let l = start().await;
    let (code, head, body) = http(l.port, "GET", "/version?x", &format!("127.0.0.1:{}", l.port)).await;
    assert_eq!(code, 200);
    assert!(head.to_lowercase().contains("cache-control: no-store"));
    assert_eq!(body.len(), 16);
    assert!(body.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn foreign_host_403() {
    let l = start().await;
    for path in ["/", "/version", "/ws", "/nope"] {
        let (code, head, _) = http(l.port, "GET", path, "evil.example").await;
        assert_eq!(code, 403, "{path}");
        assert!(head.to_lowercase().contains("x-content-type-options: nosniff"));
    }
    assert_eq!(http(l.port, "GET", "/", "localhost:1").await.0, 403);
}

#[tokio::test]
async fn post_405_unknown_404() {
    let l = start().await;
    let host = format!("localhost:{}", l.port);
    assert_eq!(http(l.port, "POST", "/", &host).await.0, 405);
    assert_eq!(http(l.port, "POST", "/version", &host).await.0, 405);
    assert_eq!(http(l.port, "GET", "/etc/passwd", &host).await.0, 404);
}

#[tokio::test]
async fn ws_foreign_origin_403() {
    let l = start().await;
    assert_eq!(ws(l.port, Some("https://evil.example")).await.err(), Some(403));
    assert_eq!(ws(l.port, Some(&format!("http://localhost:{}.evil.example", l.port))).await.err(), Some(403));
    assert!(ws(l.port, Some(&format!("http://localhost:{}", l.port))).await.is_ok());
    assert!(ws(l.port, Some(&format!("http://127.0.0.1:{}", l.port))).await.is_ok());
}

#[tokio::test]
async fn ws_no_origin_ok() {
    let l = start().await;
    assert!(ws(l.port, None).await.is_ok());
}

#[tokio::test]
async fn first_message_has_all_panes() {
    let l = start().await;
    let mut s = ws(l.port, None).await.unwrap();
    let v = next_json(&mut s).await;
    let keys: Vec<_> = v["panes"].as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys, ["all", "manual", "ofc-data", "p0", "psel"]);
    assert_eq!(v["sig"].as_str().unwrap().len(), 16);
}

#[tokio::test]
async fn second_message_has_only_changed() {
    let l = start().await;
    let mut s = ws(l.port, None).await.unwrap();
    next_json(&mut s).await;
    l.store.0.lock().unwrap().1 = "2026-01-02T00:00".into();
    l.tx.send("s1".into()).unwrap();
    let v = next_json(&mut s).await;
    let keys: Vec<_> = v["panes"].as_object().unwrap().keys().cloned().collect();
    assert!(keys.contains(&"all".into()) && keys.contains(&"p0".into()), "{keys:?}");
    assert!(!keys.contains(&"manual".into()));
}

#[tokio::test]
async fn project_added_sends_reload() {
    let l = start().await;
    let mut s = ws(l.port, None).await.unwrap();
    next_json(&mut s).await;
    l.store.0.lock().unwrap().0 = 2;
    l.tx.send("s1".into()).unwrap();
    assert_eq!(next_json(&mut s).await["reload"], true);
}

#[tokio::test]
async fn ninth_socket_503() {
    let l = start().await;
    let mut open = Vec::new();
    for _ in 0..8 {
        open.push(ws(l.port, None).await.unwrap());
    }
    assert_eq!(ws(l.port, None).await.err(), Some(503));
    drop(open.pop());
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(ws(l.port, None).await.is_ok(), "a closed socket frees its slot");
}

#[tokio::test]
async fn client_text_is_ignored() {
    let l = start().await;
    let mut s = ws(l.port, None).await.unwrap();
    next_json(&mut s).await;
    s.send(Message::text("hello")).await.unwrap();
    let r = tokio::time::timeout(Duration::from_millis(500), s.next()).await;
    assert!(r.is_err(), "server must not answer client text");
    l.tx.send("s1".into()).unwrap();
    assert!(next_json(&mut s).await["panes"].is_object(), "socket still open");
}

#[tokio::test]
async fn root_500_on_store_error() {
    struct Bad;
    impl ProjectStore for Bad {
        fn projects(&self) -> Result<Vec<ProjectFiles>, String> { Err("cannot read /x: gone".into()) }
        fn stamp(&self) -> String { String::new() }
    }
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (_tx, rx) = watch::channel(String::new());
    fn stamp() -> String { String::new() }
    let s = Server { port, store: Bad, manual: Man, sessions: Sess, clock: Clk, data_label: String::new(), manual_stamp: stamp as fn() -> String, changes: rx };
    tokio::spawn(async move { axum::serve(listener, s.router()).await.unwrap() });
    let (code, _, body) = http(port, "GET", "/", &format!("localhost:{port}")).await;
    assert_eq!((code, body.as_str()), (500, "cannot read /x: gone"));
}
