use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::CORE_VERSION;

const PROTOCOL: &str = "2025-06-18";
const STDERR_TAIL: usize = 20;

pub fn start_timeout() -> Duration {
    Duration::from_secs(std::env::var("FORMMCP_START_TIMEOUT").ok().and_then(|v| v.parse().ok()).unwrap_or(600))
}

pub fn call_timeout() -> Duration {
    Duration::from_secs(std::env::var("FORMMCP_CALL_TIMEOUT").ok().and_then(|v| v.parse().ok()).unwrap_or(900))
}

pub struct AppServer {
    process: Child,
    stdin: ChildStdin,
    lines: Receiver<Value>,
    stderr: Arc<Mutex<VecDeque<String>>>,
    next_id: u64,
    pub server: Value,
    pub capabilities: Value,
    pub instructions: Option<String>,
    pub tools: Vec<Value>,
    pub tools_changed: bool,
}

impl AppServer {
    pub fn start(program: &PathBuf, args: &[String], dir: Option<&PathBuf>) -> Result<AppServer, String> {
        let mut command = Command::new(program);
        command.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        if let Some(dir) = dir {
            command.current_dir(dir);
        }
        let mut process = command.spawn().map_err(|e| format!("could not start {} ({e})", program.display()))?;
        let stdin = process.stdin.take().ok_or("no stdin for the app's server")?;
        let stdout = process.stdout.take().ok_or("no stdout from the app's server")?;
        let stderr = process.stderr.take().ok_or("no stderr from the app's server")?;
        let (send, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Ok(value) = serde_json::from_str::<Value>(line.trim()) {
                    if send.send(value).is_err() {
                        break;
                    }
                }
            }
        });
        let tail = Arc::new(Mutex::new(VecDeque::with_capacity(STDERR_TAIL)));
        let keep = Arc::clone(&tail);
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Ok(mut t) = keep.lock() {
                    if t.len() == STDERR_TAIL {
                        t.pop_front();
                    }
                    t.push_back(line);
                }
            }
        });
        let mut app = AppServer { process, stdin, lines, stderr: tail, next_id: 1, server: Value::Null, capabilities: Value::Null, instructions: None, tools: Vec::new(), tools_changed: false };
        let init = app.request(
            "initialize",
            json!({ "protocolVersion": PROTOCOL, "capabilities": {}, "clientInfo": { "name": "formmcp", "title": "formMCP", "version": CORE_VERSION } }),
            start_timeout(),
        )?;
        app.server = init.get("serverInfo").cloned().unwrap_or(Value::Null);
        app.capabilities = init.get("capabilities").cloned().unwrap_or(Value::Null);
        app.instructions = init.get("instructions").and_then(Value::as_str).map(str::to_string);
        app.notify("notifications/initialized", json!({}))?;
        app.load_tools()?;
        Ok(app)
    }

    pub fn list_all(&mut self, method: &str, key: &str) -> Result<Vec<Value>, String> {
        let mut found = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..50 {
            let params = match &cursor {
                Some(c) => json!({ "cursor": c }),
                None => json!({}),
            };
            let page = self.request(method, params, call_timeout())?;
            found.extend(page.get(key).and_then(Value::as_array).cloned().unwrap_or_default());
            cursor = page.get("nextCursor").and_then(Value::as_str).map(str::to_string);
            if cursor.is_none() {
                break;
            }
        }
        Ok(found)
    }

    pub fn offers(&self, capability: &str) -> bool {
        self.capabilities.get(capability).is_some_and(|c| !c.is_null())
    }

    pub fn load_tools(&mut self) -> Result<(), String> {
        self.tools = self.list_all("tools/list", "tools")?;
        self.tools_changed = false;
        Ok(())
    }

    pub fn alive(&mut self) -> bool {
        matches!(self.process.try_wait(), Ok(None))
    }

    fn stderr_tail(&self) -> String {
        self.stderr.lock().map(|t| t.iter().cloned().collect::<Vec<_>>().join("\n")).unwrap_or_default()
    }

    fn write(&mut self, message: &Value) -> Result<(), String> {
        writeln!(self.stdin, "{message}").and_then(|_| self.stdin.flush()).map_err(|e| format!("the app's server stopped reading ({e}){}", self.detail()))
    }

    fn detail(&self) -> String {
        let tail = self.stderr_tail();
        if tail.trim().is_empty() { String::new() } else { format!("; its last output:\n{tail}") }
    }

    pub fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.write(&json!({ "jsonrpc": "2.0", "method": method, "params": params }))
    }

    pub fn request(&mut self, method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.write(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))?;
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let message = match self.lines.recv_timeout(left) {
                Ok(m) => m,
                Err(RecvTimeoutError::Timeout) => return Err(format!("the app did not answer {method} within {} s{}", timeout.as_secs(), self.detail())),
                Err(RecvTimeoutError::Disconnected) => return Err(format!("the app's server exited while answering {method}{}", self.detail())),
            };
            if let Some(asked) = message.get("method").and_then(Value::as_str) {
                if asked == "notifications/tools/list_changed" {
                    self.tools_changed = true;
                }
                if let Some(their_id) = message.get("id").cloned() {
                    let refusal = json!({ "jsonrpc": "2.0", "id": their_id, "error": { "code": -32601, "message": format!("formMCP does not answer {asked}") } });
                    self.write(&refusal)?;
                }
                continue;
            }
            if message.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = message.get("error") {
                let text = error.get("message").and_then(Value::as_str).unwrap_or("error");
                return Err(format!("the app answered {method} with an error: {text}"));
            }
            return Ok(message.get("result").cloned().unwrap_or(Value::Null));
        }
    }
}

impl Drop for AppServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
