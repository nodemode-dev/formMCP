use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::{json, Value};

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
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        fs::copy(path, path.with_file_name(format!("{name}.bak-{stamp}"))).map_err(|e| format!("{}: {e}", path.display()))?;
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

fn toml_section(text: &str, name: &str) -> Option<(usize, usize)> {
    let key = format!("mcp_servers.{name}");
    let sub = format!("{key}.");
    let mut offset = 0;
    let mut start = None;
    for line in text.split_inclusive('\n') {
        let header = toml_header(line);
        if let Some(s) = start {
            if header.as_ref().is_some_and(|h| !h.starts_with(&sub)) {
                return Some((s, offset));
            }
        } else if header.as_deref() == Some(key.as_str()) {
            start = Some(offset);
        }
        offset += line.len();
    }
    start.map(|s| (s, text.len()))
}

fn toml_string(value: &str) -> Option<String> {
    if let Some(rest) = value.strip_prefix('\'') {
        return rest.split_once('\'').map(|(s, _)| s.to_string());
    }
    let mut out = String::new();
    let mut chars = value.strip_prefix('"')?.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => out.push(chars.next()?),
            _ => out.push(c),
        }
    }
    None
}

fn toml_command(section: &str) -> Option<String> {
    for line in section.split_inclusive('\n').skip(1) {
        if toml_header(line).is_some() {
            break;
        }
        if let Some((k, v)) = toml_line(line).split_once('=') {
            if toml_key(k) == "command" {
                return toml_string(v.trim());
            }
        }
    }
    None
}

fn toml_declares(text: &str, name: &str) -> bool {
    let key = format!("mcp_servers.{name}");
    let sub = format!("{key}.");
    let mut table = String::new();
    for line in text.split_inclusive('\n') {
        let path = match toml_header(line) {
            Some(header) => {
                table = header;
                table.clone()
            }
            None => match toml_line(line).split_once('=') {
                Some((k, _)) if table.is_empty() => toml_key(k),
                Some((k, _)) => format!("{table}.{}", toml_key(k)),
                None => continue,
            },
        };
        if path == key || path.starts_with(&sub) {
            return true;
        }
    }
    false
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
            let (text, unread) = match read_host(&path) {
                Ok(text) => (text, None),
                Err(e) => (String::new(), Some(e)),
            };
            let s = match toml_section(&text, SERVER_NAME) {
                Some((a, b)) if toml_command(&text[a..b]).is_some_and(|c| ours(&c)) => Link::Connected,
                Some(_) => Link::Taken,
                None if toml_declares(&text, SERVER_NAME) => Link::Taken,
                None => Link::NotConnected,
            };
            ClientState { client, label, state: s, config: Some(path), note: unread.or(taken(s)) }
        }
        Client::Cursor => {
            let path = cursor_config();
            let (text, unread) = match read_host(&path) {
                Ok(text) => (text, None),
                Err(e) => (String::new(), Some(e)),
            };
            let data: Value = serde_json::from_str(&text).unwrap_or_default();
            let s = match data.get("mcpServers").and_then(|m| m.get(SERVER_NAME)) {
                Some(entry) if entry.get("command").and_then(Value::as_str).is_some_and(ours) => Link::Connected,
                Some(_) => Link::Taken,
                None => Link::NotConnected,
            };
            ClientState { client, label, state: s, config: Some(path), note: unread.or(taken(s)) }
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
            let list =launcher.args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(", ");
            let env_line = if env.is_empty() { String::new() } else { format!("env = {{ {} }}\n", env.iter().map(|(k, v)| format!("{k} = {}", quote(v))).collect::<Vec<_>>().join(", ")) };
            let block = format!("[mcp_servers.{SERVER_NAME}]\ncommand = {}\nargs = [{list}]\n{env_line}startup_timeout_sec = 600\ntool_timeout_sec = 900\n", quote(&exe));
            let next = match toml_section(&text, SERVER_NAME) {
                Some((a, b)) => format!("{}{block}\n{}", &text[..a], &text[b..]),
                None if toml_declares(&text, SERVER_NAME) => return Err(format!("{} declares mcp_servers.{SERVER_NAME} in a form formMCP does not edit; not changed", path.display())),
                None if text.trim().is_empty() => block,
                None => format!("{}\n\n{block}", text.trim_end()),
            };
            backup(&path)?;
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            fs::write(&path, next).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        Client::Cursor => {
            let path = cursor_config();
            let mut data: Value = match read_host(&path)? {
                text if !text.trim().is_empty() => serde_json::from_str(&text).map_err(|e| format!("{} is not valid JSON ({e}); not changed", path.display()))?,
                _ => json!({}),
            };
            let servers = data.as_object_mut().ok_or_else(|| format!("{} is not a JSON object; not changed", path.display()))?.entry("mcpServers").or_insert_with(|| json!({}));
            let mut entry = json!({ "command": exe, "args": launcher.args });
            if !env.is_empty() {
                entry["env"] = Value::Object(env.into_iter().map(|(k, v)| (k, Value::String(v))).collect());
            }
            servers.as_object_mut().ok_or_else(|| format!("mcpServers in {} is not an object; not changed", path.display()))?.insert(SERVER_NAME.to_string(), entry);
            backup(&path)?;
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            fs::write(&path, serde_json::to_string_pretty(&data).map_err(|e| e.to_string())? + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
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
            let text = fs::read_to_string(&path).unwrap_or_default();
            if let Some((a, b)) = toml_section(&text, SERVER_NAME) {
                backup(&path)?;
                fs::write(&path, format!("{}{}", &text[..a], &text[b..])).map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
        Client::Cursor => {
            let path = cursor_config();
            if let Ok(text) = fs::read_to_string(&path) {
                if let Ok(mut data) = serde_json::from_str::<Value>(&text) {
                    if let Some(servers) = data.get_mut("mcpServers").and_then(|m| m.as_object_mut()) {
                        if servers.remove(SERVER_NAME).is_some() {
                            backup(&path)?;
                            fs::write(&path, serde_json::to_string_pretty(&data).map_err(|e| e.to_string())? + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
                        }
                    }
                }
            }
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
