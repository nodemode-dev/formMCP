use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const CLIENT_KEYS: [&str; 4] = ["claude", "codex", "cursor", "other"];
pub const HTTP_PORT: u16 = 5625;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub expose: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub off: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hidden_tools: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_port: Option<u16>,
}

pub fn client_key(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    CLIENT_KEYS.iter().copied().find(|k| *k != "other" && lower.contains(k)).unwrap_or("other")
}

pub fn client_label(key: &str) -> &'static str {
    match key {
        "claude" => "Claude Code",
        "codex" => "Codex",
        "cursor" => "Cursor",
        _ => "other agents",
    }
}

pub fn dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("FORMMCP_HOME") {
        return PathBuf::from(dir);
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_default();
    if cfg!(target_os = "macos") {
        home.join("Library/Application Support/ai.nodemode.formmcp")
    } else if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| home.join("AppData/Roaming")).join("formMCP")
    } else {
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".config")).join("formmcp")
    }
}

pub fn path() -> PathBuf {
    dir().join("settings.json")
}

impl Settings {
    pub fn saved() -> Settings {
        std::fs::read_to_string(path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    pub fn load() -> Settings {
        let mut settings = Settings::saved();
        if let Ok(list) = std::env::var("FORMMCP_EXPOSE") {
            settings.expose = list.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
        }
        settings
    }

    pub fn save(&self) -> Result<PathBuf, String> {
        let file = path();
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())? + "\n";
        std::fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))?;
        Ok(file)
    }

    pub fn set_exposed(&mut self, id: &str, on: bool) {
        self.expose.retain(|e| e != id);
        if on {
            self.expose.push(id.to_string());
            self.expose.sort();
        }
    }

    pub fn is_off(&self, client: &str, app: &str) -> bool {
        self.off.get(client).is_some_and(|list| list.iter().any(|a| a == app))
    }

    pub fn set_app_for(&mut self, client: &str, app: &str, on: bool) {
        let list = self.off.entry(client.to_string()).or_default();
        list.retain(|a| a != app);
        if !on {
            list.push(app.to_string());
            list.sort();
        }
        self.off.retain(|_, l| !l.is_empty());
    }

    pub fn tool_hidden(&self, app: &str, tool: &str) -> bool {
        self.hidden_tools.get(app).is_some_and(|list| list.iter().any(|t| t == tool))
    }

    pub fn set_tool(&mut self, app: &str, tool: &str, on: bool) {
        let list = self.hidden_tools.entry(app.to_string()).or_default();
        list.retain(|t| t != tool);
        if !on {
            list.push(tool.to_string());
            list.sort();
        }
        self.hidden_tools.retain(|_, l| !l.is_empty());
    }

    pub fn port(&self) -> u16 {
        std::env::var("FORMMCP_HTTP_PORT").ok().and_then(|p| p.parse().ok()).or(self.http_port).unwrap_or(HTTP_PORT)
    }
}

pub fn token_path() -> PathBuf {
    dir().join("http-token")
}

fn new_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|e| format!("no random bytes for a token ({e})"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn write_private(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let part = path.with_extension(format!("part-{}", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let written = options.open(&part).and_then(|mut f| std::io::Write::write_all(&mut f, text.as_bytes())).and_then(|()| std::fs::rename(&part, path));
    if let Err(e) = written {
        let _ = std::fs::remove_file(&part);
        return Err(format!("{}: {e}", path.display()));
    }
    Ok(())
}

pub fn token() -> Result<String, String> {
    if let Ok(text) = std::fs::read_to_string(token_path()) {
        let text = text.trim().to_string();
        if text.len() >= 32 {
            return Ok(text);
        }
    }
    rotate_token()
}

pub fn rotate_token() -> Result<String, String> {
    let fresh = new_token()?;
    write_private(&token_path(), &format!("{fresh}\n"))?;
    Ok(fresh)
}
