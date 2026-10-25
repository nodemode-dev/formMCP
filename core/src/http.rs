use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::{json, Value};

use crate::family::Family;
use crate::log;
use crate::mcp::{self, Gateway};
use crate::reach::Places;
use crate::settings;

const PATH: &str = "/mcp";
const HEADERS_MAX: usize = 64 * 1024;
const BODY_MAX: usize = 8 * 1024 * 1024;
const SESSIONS_MAX: usize = 64;

struct Request {
    method: String,
    path: String,
    query: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

struct Reply {
    status: u16,
    reason: &'static str,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Reply {
    fn empty(status: u16, reason: &'static str) -> Reply {
        Reply { status, reason, headers: Vec::new(), body: Vec::new() }
    }

    fn text(status: u16, reason: &'static str, text: &str) -> Reply {
        Reply { status, reason, headers: vec![("Content-Type".into(), "text/plain; charset=utf-8".into())], body: text.as_bytes().to_vec() }
    }

    fn json(value: &Value) -> Reply {
        Reply { status: 200, reason: "OK", headers: vec![("Content-Type".into(), "application/json".into())], body: value.to_string().into_bytes() }
    }
}

struct State {
    gateway: Gateway,
    sessions: HashMap<String, String>,
    order: VecDeque<String>,
}

pub struct Server {
    pub port: u16,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Server {
    pub fn stop(mut self) {
        self.halt();
    }

    fn halt(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    pub fn wait(mut self) {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.halt();
    }
}

pub fn address(port: u16) -> String {
    format!("http://127.0.0.1:{port}{PATH}")
}

fn read_line(reader: &mut BufReader<&TcpStream>, used: &mut usize) -> Result<String, Reply> {
    let mut line = String::new();
    let n = reader.read_line(&mut line).map_err(|_| Reply::empty(400, "Bad Request"))?;
    *used += n;
    if n == 0 || *used > HEADERS_MAX {
        return Err(Reply::empty(400, "Bad Request"));
    }
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

fn read_request(stream: &TcpStream) -> Result<Request, Reply> {
    let mut reader = BufReader::new(stream);
    let mut used = 0;
    let first = read_line(&mut reader, &mut used)?;
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let target = parts.next().unwrap_or_default().to_string();
    let (path, query) = target.split_once('?').map(|(p, q)| (p.to_string(), q.to_string())).unwrap_or((target, String::new()));
    let mut headers = HashMap::new();
    loop {
        let line = read_line(&mut reader, &mut used)?;
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    let mut body = Vec::new();
    if headers.get("transfer-encoding").is_some_and(|t| t.to_ascii_lowercase().contains("chunked")) {
        loop {
            let size_line = read_line(&mut reader, &mut used)?;
            let size = usize::from_str_radix(size_line.split(';').next().unwrap_or("").trim(), 16).map_err(|_| Reply::empty(400, "Bad Request"))?;
            if size == 0 {
                let _ = read_line(&mut reader, &mut used);
                break;
            }
            if body.len() + size > BODY_MAX {
                return Err(Reply::empty(413, "Payload Too Large"));
            }
            let mut chunk = vec![0; size + 2];
            reader.read_exact(&mut chunk).map_err(|_| Reply::empty(400, "Bad Request"))?;
            body.extend_from_slice(&chunk[..size]);
        }
    } else if let Some(length) = headers.get("content-length") {
        let length: usize = length.parse().map_err(|_| Reply::empty(400, "Bad Request"))?;
        if length > BODY_MAX {
            return Err(Reply::empty(413, "Payload Too Large"));
        }
        body = vec![0; length];
        reader.read_exact(&mut body).map_err(|_| Reply::empty(400, "Bad Request"))?;
    }
    Ok(Request { method, path, query, headers, body })
}

fn write_reply(mut stream: &TcpStream, reply: &Reply) {
    let mut head = format!("HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n", reply.status, reply.reason, reply.body.len());
    for (k, v) in &reply.headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str("\r\n");
    let _ = stream.write_all(head.as_bytes()).and_then(|()| stream.write_all(&reply.body)).and_then(|()| stream.flush());
}

fn local_host(value: &str, port: Option<u16>) -> bool {
    let rest = value.trim().trim_start_matches("http://").trim_start_matches("https://");
    let host_port = rest.split('/').next().unwrap_or("");
    let (name, p) = if host_port.starts_with('[') {
        match host_port.find(']') {
            Some(i) => (&host_port[..=i], host_port[i + 1..].strip_prefix(':').unwrap_or("")),
            None => return false,
        }
    } else {
        host_port.rsplit_once(':').unwrap_or((host_port, ""))
    };
    matches!(name, "127.0.0.1" | "localhost" | "[::1]") && port.is_none_or(|want| p.is_empty() || p == want.to_string())
}

fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').filter_map(|kv| kv.split_once('=')).find(|(k, _)| *k == key).map(|(_, v)| v)
}

fn session_id() -> String {
    let mut bytes = [0u8; 16];
    if getrandom::getrandom(&mut bytes).is_err() {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        bytes.copy_from_slice(&t.to_le_bytes());
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn refuse(reason: &str, status: u16, why: &'static str) -> Reply {
    log::record("http-refused", json!({ "reason": reason, "status": status }));
    let mut reply = Reply::text(status, why, reason);
    if status == 401 {
        reply.headers.push(("WWW-Authenticate".into(), "Bearer realm=\"formmcp\"".into()));
    }
    reply
}

fn handle(req: Request, state: &Mutex<State>, port: u16) -> Reply {
    if req.path != PATH {
        return Reply::text(404, "Not Found", "formMCP answers at /mcp");
    }
    if !req.headers.get("host").is_some_and(|host| local_host(host, Some(port))) {
        return refuse("host is not this computer", 403, "Forbidden");
    }
    if req.headers.get("origin").is_some_and(|origin| !local_host(origin, None)) {
        return refuse("origin is not this computer", 403, "Forbidden");
    }
    let token = match settings::token() {
        Ok(t) => t,
        Err(e) => return Reply::text(500, "Internal Server Error", &e),
    };
    let given = req.headers.get("authorization").and_then(|a| a.strip_prefix("Bearer ").or_else(|| a.strip_prefix("bearer "))).unwrap_or("").trim();
    if !same(given, &token) {
        return refuse("missing or wrong access token", 401, "Unauthorized");
    }
    if let Some(version) = req.headers.get("mcp-protocol-version") {
        if !mcp::supported(version) {
            return Reply::text(400, "Bad Request", "unsupported MCP-Protocol-Version");
        }
    }
    let session = req.headers.get("mcp-session-id").cloned();
    match req.method.as_str() {
        "GET" => {
            let mut reply = Reply::text(405, "Method Not Allowed", "formMCP sends no server stream; POST to /mcp");
            reply.headers.push(("Allow".into(), "POST, DELETE".into()));
            reply
        }
        "DELETE" => {
            let Some(id) = session else { return Reply::text(400, "Bad Request", "Mcp-Session-Id is required") };
            let Ok(mut st) = state.lock() else { return Reply::empty(500, "Internal Server Error") };
            if st.sessions.remove(&id).is_some() {
                st.order.retain(|s| s != &id);
                Reply::empty(200, "OK")
            } else {
                Reply::empty(404, "Not Found")
            }
        }
        "POST" => {
            let Ok(message) = serde_json::from_slice::<Value>(&req.body) else {
                return Reply { status: 400, reason: "Bad Request", headers: vec![("Content-Type".into(), "application/json".into())], body: json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "Parse error" } }).to_string().into_bytes() };
            };
            let messages: Vec<Value> = match &message {
                Value::Array(list) => list.clone(),
                one => vec![one.clone()],
            };
            let initializing = messages.iter().any(|m| m.get("method").and_then(Value::as_str) == Some("initialize"));
            let Ok(mut st) = state.lock() else { return Reply::empty(500, "Internal Server Error") };
            let mut new_session = None;
            if initializing {
                st.gateway.set_session_client(param(&req.query, "client"));
            } else {
                match &session {
                    None => return Reply::text(400, "Bad Request", "Mcp-Session-Id is required after initialize"),
                    Some(id) => match st.sessions.get(id).cloned() {
                        Some(client) => st.gateway.client = client,
                        None => return Reply::text(404, "Not Found", "this session is over; initialize again"),
                    },
                }
            }
            st.gateway.transport = "http";
            let replies: Vec<Value> = messages.iter().filter_map(|m| mcp::answer(&mut st.gateway, m)).collect();
            st.gateway.take_list_changed();
            if initializing {
                let id = session_id();
                let client = st.gateway.client.clone();
                st.sessions.insert(id.clone(), client);
                st.order.push_back(id.clone());
                while st.order.len() > SESSIONS_MAX {
                    if let Some(old) = st.order.pop_front() {
                        st.sessions.remove(&old);
                    }
                }
                new_session = Some(id);
            }
            drop(st);
            if replies.is_empty() {
                return Reply::empty(202, "Accepted");
            }
            let body = if matches!(message, Value::Array(_)) { Value::Array(replies) } else { replies.into_iter().next().unwrap_or(Value::Null) };
            let mut reply = Reply::json(&body);
            if let Some(id) = new_session {
                reply.headers.push(("Mcp-Session-Id".into(), id));
            }
            reply
        }
        _ => {
            let mut reply = Reply::text(405, "Method Not Allowed", "POST to /mcp");
            reply.headers.push(("Allow".into(), "POST, DELETE".into()));
            reply
        }
    }
}

pub fn start(port: u16) -> Result<Server, String> {
    let family = Family::bundled()?;
    settings::token()?;
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)).map_err(|e| format!("127.0.0.1:{port} could not be opened ({e}); another program may use it, or set another port"))?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let state = Arc::new(Mutex::new(State { gateway: Gateway::new(family, Places::from_env()), sessions: HashMap::new(), order: VecDeque::new() }));
    let stop = Arc::new(AtomicBool::new(false));
    let halt = Arc::clone(&stop);
    log::record("http-start", json!({ "port": port }));
    let handle = thread::spawn(move || {
        while !halt.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let state = Arc::clone(&state);
                    thread::spawn(move || {
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
                        let reply = match read_request(&stream) {
                            Ok(req) => handle(req, &state, port),
                            Err(reply) => reply,
                        };
                        write_reply(&stream, &reply);
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(50)),
                Err(_) => thread::sleep(Duration::from_millis(200)),
            }
        }
        log::record("http-stop", json!({ "port": port }));
    });
    Ok(Server { port, stop, handle: Some(handle) })
}
