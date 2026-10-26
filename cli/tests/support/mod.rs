#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};

pub const BIN: &str = env!("CARGO_BIN_EXE_formmcp-cli");

pub struct Sandbox {
    pub root: PathBuf,
}

impl Sandbox {
    pub fn new(tag: &str) -> Sandbox {
        static N: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!("formmcp-test-{}-{tag}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let _ = std::fs::remove_dir_all(&root);
        for dir in ["home", "formmcp", "portal", "dev", "bin"] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        Sandbox { root }
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    pub fn command(&self) -> Command {
        let mut c = Command::new(BIN);
        c.env_clear()
            .env("HOME", self.path("home"))
            .env("USERPROFILE", self.path("home"))
            .env("XDG_CONFIG_HOME", self.path("home/.config"))
            .env("XDG_DATA_HOME", self.path("home/.local/share"))
            .env("FORMMCP_HOME", self.path("formmcp"))
            .env("FORMMCP_DOWNLOADS", self.path("home/Downloads"))
            .env("FORMPORTAL_ROOT", self.path("portal"))
            .env("FORMMCP_CATALOG", self.path("no-catalogue.json"))
            .env("PATH", self.path("bin"))
            .env("FORMMCP_IDLE_MINUTES", "1")
            .stdin(Stdio::null());
        c
    }

    pub fn json(&self, args: &[&str], extra: &[(&str, &str)]) -> Value {
        let mut c = self.command();
        for (k, v) in extra {
            c.env(k, v);
        }
        let out = c.args(args).output().unwrap();
        assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{args:?} printed no JSON ({e}): {}", String::from_utf8_lossy(&out.stdout)))
    }

    pub fn text(&self, args: &[&str], extra: &[(&str, &str)]) -> String {
        let mut c = self.command();
        for (k, v) in extra {
            c.env(k, v);
        }
        let out = c.args(args).output().unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

pub fn public(key: &SigningKey) -> String {
    base64::engine::general_purpose::STANDARD.encode(key.verifying_key().to_bytes())
}

pub fn write_catalogue(dir: &Path, body: &str, signer: Option<&SigningKey>) -> PathBuf {
    let path = dir.join("apps.json");
    std::fs::write(&path, body).unwrap();
    if let Some(k) = signer {
        std::fs::write(dir.join("apps.json.sig"), base64::engine::general_purpose::STANDARD.encode(k.sign(body.as_bytes()).to_bytes())).unwrap();
    }
    path
}

pub fn catalogue(products: &[Value]) -> String {
    json!({ "format": "formai-catalog", "format_version": 1, "generated": "2026-10-26T00:00:00Z", "origin": "test", "products": products }).to_string()
}

pub fn product(id: &str, mcp: bool, version: Option<&str>) -> Value {
    let channels = match version {
        Some(v) => json!({ "stable": { "version": v, "published": "2026-10-26T00:00:00Z", "notes": "", "artifacts": [] } }),
        None => json!({}),
    };
    json!({ "id": id, "name": format!("Name of {id}"), "kind": "test", "summary": format!("{id} from the catalogue"), "identifier": format!("dev.test.{id}"), "repository": format!("https://github.com/nodemode-ai/{id}"), "licence": "GPL-3.0-only", "state": "live", "agents": { "mcp": mcp, "skills": false }, "channels": channels })
}

pub struct Stdio2 {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
}

impl Stdio2 {
    pub fn start(sandbox: &Sandbox, extra: &[(&str, &str)]) -> Stdio2 {
        let mut c = sandbox.command();
        for (k, v) in extra {
            c.env(k, v);
        }
        let mut child = c.arg("mcp").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let out = child.stdout.take().unwrap();
        let (tx, lines) = channel();
        thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Stdio2 { child, stdin, lines }
    }

    pub fn send(&mut self, message: Value) {
        writeln!(self.stdin, "{message}").unwrap();
        self.stdin.flush().unwrap();
    }

    pub fn reply(&mut self, id: u64, within: Duration) -> Value {
        let deadline = Instant::now() + within;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let line = self.lines.recv_timeout(left).unwrap_or_else(|_| panic!("no reply to request {id} within {within:?}"));
            let Ok(value) = serde_json::from_str::<Value>(&line) else { continue };
            if value.get("id") == Some(&json!(id)) {
                return value;
            }
        }
    }

    pub fn call(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        self.reply(id, Duration::from_secs(30))
    }

    pub fn initialize(&mut self) -> Value {
        let r = self.call(1, "initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "family-test", "version": "1" } }));
        self.send(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        r
    }
}

impl Drop for Stdio2 {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct Http {
    child: Child,
    pub port: u16,
}

impl Http {
    pub fn start(sandbox: &Sandbox) -> Http {
        let port = {
            let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().port()
        };
        let child = sandbox.command().args(["serve", "--port", &port.to_string()]).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        while TcpStream::connect(("127.0.0.1", port)).is_err() {
            assert!(Instant::now() < deadline, "formmcp serve did not listen on {port}");
            thread::sleep(Duration::from_millis(50));
        }
        Http { child, port }
    }

    pub fn raw(&self, method: &str, path: &str, headers: &[(&str, String)], body: &str) -> (u16, Vec<(String, String)>, String) {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
        let mut request = format!("{method} {path} HTTP/1.1\r\nContent-Length: {}\r\nConnection: close\r\n", body.len());
        for (k, v) in headers {
            request.push_str(&format!("{k}: {v}\r\n"));
        }
        request.push_str("\r\n");
        request.push_str(body);
        stream.write_all(request.as_bytes()).unwrap();
        let mut text = String::new();
        stream.read_to_string(&mut text).unwrap();
        let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
        let mut lines = head.lines();
        let status = lines.next().unwrap_or("").split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        let headers = lines.filter_map(|l| l.split_once(':')).map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string())).collect();
        (status, headers, body.to_string())
    }
}

impl Drop for Http {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
}

pub fn bundled() -> Value {
    serde_json::from_str(include_str!("../../../core/data/family.json")).unwrap()
}
