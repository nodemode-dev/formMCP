use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::{json, Value};

use crate::host_json;

pub const SERVER_NAME: &str = "formmcp";
const PASSED_ENV: [&str; 5] = ["FORMMCP_DEV_ROOTS", "FORMPORTAL_ROOT", "FORMPORTAL_BIN", "FORMMCP_CATALOG", "FORMAI_CATALOG_PUBKEY"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Client {
    Claude,
    Codex,
    Cursor,
}

pub const CLIENTS: [Client; 3] = [Client::Claude, Client::Codex, Client::Cursor];

impl Client {
    pub fn label(self) -> &'static str {
        match self {
            Client::Claude => "Claude Code",
            Client::Codex => "Codex",
            Client::Cursor => "Cursor",
        }
    }

    pub fn parse(text: &str) -> Option<Client> {
        match text.trim().to_lowercase().as_str() {
            "claude" | "claude-code" | "claude code" => Some(Client::Claude),
            "codex" => Some(Client::Codex),
            "cursor" => Some(Client::Cursor),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Link {
    Connected,
    NotConnected,
    Taken,
    NoClient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClientState {
    pub client: Client,
    pub label: &'static str,
    pub state: Link,
    pub config: Option<PathBuf>,
    pub note: Option<String>,
}

pub struct Launcher {
    pub program: PathBuf,
    pub args: Vec<String>,
}

impl Launcher {
    pub fn current() -> Result<Launcher, String> {
        let program = match std::env::var_os("FORMMCP_LAUNCHER") {
            Some(p) => PathBuf::from(p),
            None => std::env::current_exe().map_err(|e| e.to_string())?,
        };
        Ok(Launcher { program, args: vec!["mcp".to_string()] })
    }
}

fn home() -> PathBuf {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

fn codex_config() -> PathBuf {
    std::env::var_os("CODEX_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".codex")).join("config.toml")
}

fn cursor_config() -> PathBuf {
    home().join(".cursor").join("mcp.json")
}

fn which(program: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let names: Vec<String> = if cfg!(windows) { [".exe", ".cmd", ".bat"].iter().map(|e| format!("{program}{e}")).collect() } else { vec![program.to_string()] };
        for name in names {
            let candidate = dir.join(&name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn backup(path: &Path) -> Result<(), String> {
    if path.exists() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let mut target = path.with_file_name(format!("{name}.bak-{stamp}"));
        let mut n = 0;
        while target.exists() {
            n += 1;
            target = path.with_file_name(format!("{name}.bak-{stamp}-{n}"));
        }
        fs::copy(path, target).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

fn save(path: &Path, text: &str) -> Result<(), String> {
    let real = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if let Some(dir) = real.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    static WRITES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let name = real.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let part = real.with_file_name(format!("{name}.part-{}-{}", std::process::id(), WRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    let written = fs::write(&part, text)
        .and_then(|()| match fs::metadata(&real) {
            Ok(meta) if cfg!(unix) => fs::set_permissions(&part, meta.permissions()),
            _ => Ok(()),
        })
        .and_then(|()| fs::rename(&part, &real));
    if let Err(e) = written {
        let _ = fs::remove_file(&part);
        return Err(format!("{}: {e}", path.display()));
    }
    Ok(())
}

fn read_host(path: &Path) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("{}: could not be read ({e}); not changed", path.display())),
    }
}

fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

fn ours(command: &str) -> bool {
    let path = Path::new(command);
    let stem = path.file_stem().map(|s| s.to_string_lossy().to_lowercase());
    matches!(stem.as_deref(), Some("formmcp" | "formmcp-cli")) || Launcher::current().is_ok_and(|l| l.program == path)
}

fn toml_line(line: &str) -> &str {
    let line = line.trim_start_matches('\u{feff}');
    let mut quote = None;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        match quote {
            Some(q) => {
                if escaped {
                    escaped = false;
                } else if c == '\\' && q == '"' {
                    escaped = true;
                } else if c == q {
                    quote = None;
                }
            }
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '#' => return line[..i].trim(),
            None => {}
        }
    }
    line.trim()
}

fn toml_key(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace() && *c != '"' && *c != '\'').collect()
}

fn toml_header(line: &str) -> Option<String> {
    let line = toml_line(line);
    line.starts_with('[').then(|| toml_key(line.trim_start_matches('[').trim_end_matches(']')))
}

fn toml_close(line: &[u8], mut i: usize, delim: &[u8]) -> Option<usize> {
    while i < line.len() {
        if line[i] == b'\\' && delim[0] == b'"' {
            i += 2;
            continue;
        }
        if line[i..].starts_with(delim) {
            let mut j = i + 3;
            while j < line.len() && j < i + 5 && line[j] == delim[0] {
                j += 1;
            }
            return Some(j);
        }
        i += 1;
    }
    None
}

fn toml_scan(line: &str, mut open: Option<&'static [u8]>) -> Option<&'static [u8]> {
    let b = line.as_bytes();
    let mut i = 0;
    loop {
        if let Some(delim) = open {
            i = match toml_close(b, i, delim) {
                Some(after) => after,
                None => return open,
            };
            open = None;
            continue;
        }
        while i < b.len() && !matches!(b[i], b'#' | b'"' | b'\'') {
            i += 1;
        }
        let &c = b.get(i)?;
        if c == b'#' {
            return None;
        }
        let delim: &'static [u8] = if c == b'"' { b"\"\"\"" } else { b"'''" };
        if b[i..].starts_with(delim) {
            open = Some(delim);
            i += 3;
            continue;
        }
        i += 1;
        while i < b.len() && b[i] != c && b[i] != b'\n' {
            i += if c == b'"' && b[i] == b'\\' { 2 } else { 1 };
        }
        i += 1;
    }
}

fn toml_lines(text: &str) -> Vec<(usize, &str, bool)> {
    let mut out = Vec::new();
    let mut open = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        out.push((offset, line, open.is_some()));
        open = toml_scan(line, open);
        offset += line.len();
    }
    out
}

fn toml_section(text: &str, name: &str) -> Option<(usize, usize)> {
    let key = format!("mcp_servers.{name}");
    let sub = format!("{key}.");
    let mut start = None;
    let mut end = 0;
    for (offset, line, inside) in toml_lines(text) {
        let header = if inside { None } else { toml_header(line) };
        if let Some(s) = start {
            if header.as_ref().is_some_and(|h| !h.starts_with(&sub)) {
                return Some((s, end));
            }
            if inside || !toml_line(line).is_empty() {
                end = offset + line.len();
            }
        } else if header.as_deref() == Some(key.as_str()) {
            start = Some(offset);
            end = offset + line.len();
        }
    }
    start.map(|s| (s, end))
}

fn toml_doc(path: &Path, text: &str) -> Result<toml_edit::DocumentMut, String> {
    let body = text.trim_start_matches('\u{feff}');
    body.parse::<toml_edit::DocumentMut>().map_err(|e| {
        let line = e.span().map_or(1, |s| body[..s.start.min(body.len())].matches('\n').count() + 1);
        format!("{} is not valid TOML (line {line}: {}); not changed", path.display(), e.message().trim())
    })
}

fn toml_entry(doc: &toml_edit::DocumentMut) -> Option<&toml_edit::Item> {
    doc.get("mcp_servers").and_then(|m| m.get(SERVER_NAME))
}

fn toml_scalar(value: &toml_edit::Value) -> Value {
    use toml_edit::Value as T;
    match value {
        T::String(s) => Value::String(s.value().clone()),
        T::Integer(n) => json!(*n.value()),
        T::Float(f) => serde_json::Number::from_f64(*f.value()).map(Value::Number).unwrap_or_else(|| Value::String(f.value().to_string())),
        T::Boolean(b) => Value::Bool(*b.value()),
        T::Datetime(d) => json!({ "datetime": d.value().to_string() }),
        T::Array(a) => Value::Array(a.iter().map(toml_scalar).collect()),
        T::InlineTable(t) => toml_table(t),
    }
}

fn toml_table(table: &dyn toml_edit::TableLike) -> Value {
    Value::Object(table.iter().map(|(k, v)| (k.to_string(), toml_json(v))).collect())
}

fn toml_json(item: &toml_edit::Item) -> Value {
    match item {
        toml_edit::Item::None => Value::Null,
        toml_edit::Item::Value(v) => toml_scalar(v),
        toml_edit::Item::Table(t) => toml_table(t),
        toml_edit::Item::ArrayOfTables(a) => Value::Array(a.iter().map(|t| toml_table(t)).collect()),
    }
}

fn toml_rest(doc: &toml_edit::DocumentMut) -> Value {
    let mut all = toml_table(doc.as_table());
    if let Some(top) = all.as_object_mut() {
        if let Some(servers) = top.get_mut("mcp_servers").and_then(Value::as_object_mut) {
            servers.remove(SERVER_NAME);
            if servers.is_empty() {
                top.remove("mcp_servers");
            }
        }
    }
    all
}

fn toml_checked(path: &Path, before: &toml_edit::DocumentMut, next: &str, command: Option<&str>) -> Result<(), String> {
    let after = toml_doc(path, next).map_err(|_| format!("{}: formMCP's change would not leave valid TOML; not changed", path.display()))?;
    let entry = toml_entry(&after).map(|e| e.get("command").and_then(toml_edit::Item::as_str));
    if toml_rest(before) != toml_rest(&after) || entry != command.map(Some) {
        return Err(format!("{}: formMCP's change would alter other settings; not changed", path.display()));
    }
    Ok(())
}

fn cursor_unparsed(path: &Path, e: &serde_json::Error) -> String {
    format!("{} is not plain JSON ({e}); formMCP edits only plain JSON and leaves a file with comments or trailing commas alone; not changed", path.display())
}

fn cursor_data(path: &Path, text: &str) -> Result<Value, String> {
    let body = text.strip_prefix('\u{feff}').unwrap_or(text);
    if body.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(body).map_err(|e| cursor_unparsed(path, &e))
}

fn cursor_checked(path: &Path, next: &str, expected: &Value) -> Result<(), String> {
    match cursor_data(path, next) {
        Ok(data) if &data == expected => Ok(()),
        _ => Err(format!("{}: formMCP's change would alter other settings; not changed", path.display())),
    }
}

fn passed_env() -> Vec<(String, String)> {
    PASSED_ENV.iter().filter_map(|k| std::env::var(k).ok().filter(|v| !v.trim().is_empty()).map(|v| (k.to_string(), v))).collect()
}

pub fn state(client: Client) -> ClientState {
    let label = client.label();
    let taken = |s: Link| (s == Link::Taken).then(|| format!("another server named {SERVER_NAME} is set up; left alone"));
    match client {
        Client::Claude => match which("claude") {
            None => ClientState { client, label, state: Link::NoClient, config: None, note: Some("the claude command is not installed on this computer".into()) },
            Some(claude) => match Command::new(claude).args(["mcp", "get", SERVER_NAME]).current_dir(home()).output() {
                Ok(out) if out.status.success() && String::from_utf8_lossy(&out.stdout).contains("Scope: User") => {
                    let shown = String::from_utf8_lossy(&out.stdout);
                    let s = if shown.lines().find_map(|l| l.trim().strip_prefix("Command:")).is_some_and(|c| ours(c.trim())) { Link::Connected } else { Link::Taken };
                    ClientState { client, label, state: s, config: None, note: taken(s) }
                }
                _ => ClientState { client, label, state: Link::NotConnected, config: None, note: None },
            },
        },
        Client::Codex => {
            let path = codex_config();
            let (s, note) = match read_host(&path).and_then(|text| toml_doc(&path, &text).map(|doc| (text, doc))) {
                Err(e) => (Link::NotConnected, Some(e)),
                Ok((text, doc)) => {
                    let s = match toml_entry(&doc) {
                        None => Link::NotConnected,
                        Some(entry) if toml_section(&text, SERVER_NAME).is_some() && entry.get("command").and_then(toml_edit::Item::as_str).is_some_and(ours) => Link::Connected,
                        Some(_) => Link::Taken,
                    };
                    (s, taken(s))
                }
            };
            ClientState { client, label, state: s, config: Some(path), note }
        }
        Client::Cursor => {
            let path = cursor_config();
            let (s, note) = match read_host(&path).and_then(|text| cursor_data(&path, &text)) {
                Err(e) => (Link::NotConnected, Some(e)),
                Ok(data) => {
                    let s = match data.get("mcpServers").and_then(|m| m.get(SERVER_NAME)) {
                        Some(entry) if entry.get("command").and_then(Value::as_str).is_some_and(ours) => Link::Connected,
                        Some(_) => Link::Taken,
                        None => Link::NotConnected,
                    };
                    (s, taken(s))
                }
            };
            ClientState { client, label, state: s, config: Some(path), note }
        }
    }
}

fn write(client: Client, launcher: &Launcher) -> Result<(), String> {
    let exe = launcher.program.to_string_lossy().to_string();
    let env = passed_env();
    match client {
        Client::Claude => {
            let claude = which("claude").ok_or("the claude command is not installed")?;
            let _ = Command::new(&claude).args(["mcp", "remove", SERVER_NAME, "--scope", "user"]).output();
            let mut command = Command::new(&claude);
            command.args(["mcp", "add", "--scope", "user", SERVER_NAME]);
            for (k, v) in &env {
                command.arg("-e").arg(format!("{k}={v}"));
            }
            command.arg("--").arg(&exe).args(&launcher.args);
            let out = command.output().map_err(|e| e.to_string())?;
            if !out.status.success() {
                return Err(format!("claude mcp add failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
            }
        }
        Client::Codex => {
            let path = codex_config();
            let text = read_host(&path)?;
            let doc = toml_doc(&path, &text)?;
            let list = launcher.args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(", ");
            let env_line = if env.is_empty() { String::new() } else { format!("env = {{ {} }}\n", env.iter().map(|(k, v)| format!("{k} = {}", quote(v))).collect::<Vec<_>>().join(", ")) };
            let block = format!("[mcp_servers.{SERVER_NAME}]\ncommand = {}\nargs = [{list}]\n{env_line}startup_timeout_sec = 600\ntool_timeout_sec = 900\n", quote(&exe));
            let next = match toml_section(&text, SERVER_NAME) {
                Some((a, b)) => {
                    let rest = &text[b..];
                    let gap = if rest.is_empty() || rest.starts_with('\n') || rest.starts_with("\r\n") { "" } else { "\n" };
                    format!("{}{block}{gap}{rest}", &text[..a])
                }
                None if toml_entry(&doc).is_some() => return Err(format!("{} declares mcp_servers.{SERVER_NAME} in a form formMCP does not edit; not changed", path.display())),
                None if text.trim().is_empty() => block,
                None => format!("{}\n\n{block}", text.trim_end()),
            };
            if next == text {
                return Ok(());
            }
            toml_checked(&path, &doc, &next, Some(&exe))?;
            backup(&path)?;
            save(&path, &next)?;
        }
        Client::Cursor => {
            let path = cursor_config();
            let text = read_host(&path)?;
            let mut data = match cursor_data(&path, &text)? {
                Value::Null => json!({}),
                data => data,
            };
            let fresh = text.trim_start_matches('\u{feff}').trim().is_empty();
            let servers = data.as_object_mut().ok_or_else(|| format!("{} is not a JSON object; not changed", path.display()))?.entry("mcpServers").or_insert_with(|| json!({}));
            let mut entry = json!({ "command": exe, "args": launcher.args });
            if !env.is_empty() {
                entry["env"] = Value::Object(env.into_iter().map(|(k, v)| (k, Value::String(v))).collect());
            }
            servers.as_object_mut().ok_or_else(|| format!("mcpServers in {} is not an object; not changed", path.display()))?.insert(SERVER_NAME.to_string(), entry.clone());
            let next = if fresh {
                serde_json::to_string_pretty(&data).map_err(|e| e.to_string())? + "\n"
            } else {
                host_json::set(&text, SERVER_NAME, &entry).ok_or_else(|| format!("{}: formMCP could not place its entry in this file; not changed", path.display()))?
            };
            if next == text {
                return Ok(());
            }
            cursor_checked(&path, &next, &data)?;
            backup(&path)?;
            save(&path, &next)?;
        }
    }
    Ok(())
}

fn remove(client: Client) -> Result<(), String> {
    match client {
        Client::Claude => {
            if let Some(claude) = which("claude") {
                let _ = Command::new(claude).args(["mcp", "remove", SERVER_NAME, "--scope", "user"]).output();
            }
        }
        Client::Codex => {
            let path = codex_config();
            let text = read_host(&path)?;
            let doc = toml_doc(&path, &text)?;
            if let Some((a, b)) = toml_section(&text, SERVER_NAME) {
                let next = format!("{}{}", &text[..a], &text[b..]);
                toml_checked(&path, &doc, &next, None)?;
                backup(&path)?;
                save(&path, &next)?;
            }
        }
        Client::Cursor => {
            let path = cursor_config();
            let text = read_host(&path)?;
            let mut data = cursor_data(&path, &text)?;
            let Some(servers) = data.get_mut("mcpServers").and_then(|m| m.as_object_mut()) else {
                return Ok(());
            };
            if servers.remove(SERVER_NAME).is_none() {
                return Ok(());
            }
            let emptied = servers.is_empty();
            let next = host_json::remove(&text, SERVER_NAME).ok_or_else(|| format!("{}: formMCP could not find its entry in this file; not changed", path.display()))?;
            if emptied && cursor_data(&path, &next).is_ok_and(|d| d.get("mcpServers").is_none()) {
                if let Some(top) = data.as_object_mut() {
                    top.remove("mcpServers");
                }
            }
            cursor_checked(&path, &next, &data)?;
            backup(&path)?;
            save(&path, &next)?;
        }
    }
    Ok(())
}

pub fn connect(clients: &[Client], launcher: &Launcher, on: bool) -> Vec<Result<ClientState, (Client, String)>> {
    clients
        .iter()
        .map(|&client| {
            let before = state(client);
            let done = match (on, before.state) {
                (true, Link::NotConnected) | (true, Link::Connected) => write(client, launcher),
                (false, Link::Connected) => remove(client),
                _ => Ok(()),
            };
            done.map(|_| state(client)).map_err(|e| (client, e))
        })
        .collect()
}
