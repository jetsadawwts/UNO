//! Local live server: `/` the page, `/version` its fingerprint. std only, one thread per request.
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::thread;
use std::time::Duration;

use app::ports::{Clock, ManualSource, ProjectStore, SessionProbe};

use crate::{render_page, PageMode};

/// Binds 127.0.0.1 only and refuses requests whose Host header is not this machine
/// (so another site cannot read the data through DNS rebinding). GET only.
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
}

impl<S, M, P, C, F> Server<S, M, P, C, F>
where
    S: ProjectStore,
    M: ManualSource,
    P: SessionProbe,
    C: Clock,
    F: Fn() -> String,
{
    /// Answer one request, given its request line and headers. Returns (status, content type, body).
    pub fn handle(&self, request_head: &str) -> (u16, &'static str, Vec<u8>) {
        let mut lines = request_head.lines();
        let first = lines.next().unwrap_or("");
        let mut host = String::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some(v) = line.split_once(':').filter(|(k, _)| k.eq_ignore_ascii_case("host")).map(|(_, v)| v.trim().to_string()) {
                host = v;
            }
        }
        let mut parts = first.split_whitespace();
        let (method, path) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
        let allowed = [format!("localhost:{}", self.port), format!("127.0.0.1:{}", self.port)];
        if !allowed.contains(&host) {
            return (403, "text/plain", b"forbidden".to_vec());
        }
        if method != "GET" {
            return (405, "text/plain", b"GET only".to_vec());
        }
        match path.split('?').next().unwrap_or("") {
            "/" => match app::build_dashboard(&self.store, &self.sessions, &self.clock) {
                Ok(d) => (200, "text/html; charset=utf-8", render_page(&d, &self.manual.manual(), &self.data_label, &PageMode::Live).into_bytes()),
                Err(e) => (500, "text/plain; charset=utf-8", e.into_bytes()),
            },
            "/version" => (200, "text/plain", app::signature(&self.store, &(self.manual_stamp)(), &self.sessions).into_bytes()),
            _ => (404, "text/plain", b"not found".to_vec()),
        }
    }
}

impl<S, M, P, C, F> Server<S, M, P, C, F>
where
    S: ProjectStore + Send + Sync + 'static,
    M: ManualSource + Send + Sync + 'static,
    P: SessionProbe + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
    F: Fn() -> String + Send + Sync + 'static,
{
    /// Listen until the process is stopped. Exits with status 1 when the port cannot be bound.
    pub fn run(self, open: bool) -> ! {
        let port = self.port;
        let listener = match TcpListener::bind(("127.0.0.1", port)) {
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
        let me = std::sync::Arc::new(self);
        for conn in listener.incoming() {
            let Ok(stream) = conn else { continue };
            let me = me.clone();
            thread::spawn(move || me.serve_one(stream));
        }
        std::process::exit(0)
    }

    fn serve_one(&self, mut stream: TcpStream) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut reader = BufReader::new(match stream.try_clone() { Ok(s) => s, Err(_) => return });
        let mut head = String::new();
        if reader.read_line(&mut head).is_err() {
            return;
        }
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" || line == "\n" {
                break;
            }
            head += &line;
        }
        let (code, ctype, body) = self.handle(&head);
        let status = match code {
            200 => "200 OK",
            403 => "403 Forbidden",
            404 => "404 Not Found",
            405 => "405 Method Not Allowed",
            _ => "500 Internal Server Error",
        };
        let h = format!("HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\
                         X-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n", body.len());
        let _ = stream.write_all(h.as_bytes());
        let _ = stream.write_all(&body);
    }
}
