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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub start: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub use_profile: BTreeMap<String, String>,
}

pub const START_POLICIES: [&str; 3] = ["demand", "session", "keep"];
pub const PROFILE_FORMAT: &str = "formmcp-profile";
const PROFILE_LIMIT: usize = 256 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub apps_off: Vec<String>,
    #[serde(default)]
    pub hidden_tools: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub start: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileFile {
    pub format: String,
    pub format_version: u32,
    pub name: String,
    #[serde(flatten)]
    pub profile: Profile,
}

pub fn profile_name_ok(name: &str) -> bool {
    let n = name.chars().count();
    (1..=40).contains(&n) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ') && !name.starts_with(' ') && !name.ends_with(' ')
}

pub fn read_profile(text: &str, known_app: &dyn Fn(&str) -> bool) -> Result<(String, Profile, Vec<String>), String> {
    if text.len() > PROFILE_LIMIT {
        return Err(format!("a profile file is at most {PROFILE_LIMIT} bytes"));
    }
    let file: ProfileFile = serde_json::from_str(text).map_err(|e| format!("not a formMCP profile ({e})"))?;
    if file.format != PROFILE_FORMAT || file.format_version != 1 {
        return Err(format!("not a formMCP profile (format {:?} version {})", file.format, file.format_version));
    }
    if !profile_name_ok(&file.name) {
        return Err(format!("the profile name {:?} is not allowed: 1 to 40 letters, digits, spaces, hyphens or underscores", file.name));
    }
    let mut dropped = Vec::new();
    let mut profile = file.profile;
    profile.apps_off.retain(|a| known_app(a) || {
        dropped.push(a.clone());
        false
    });
    profile.hidden_tools.retain(|a, tools| {
        tools.retain(|t| !t.is_empty() && t.len() <= 128);
        known_app(a) && !tools.is_empty() || {
            dropped.push(a.clone());
            false
        }
    });
    profile.start.retain(|a, policy| known_app(a) && START_POLICIES.contains(&policy.as_str()) || {
        dropped.push(format!("{a} ({policy})"));
        false
    });
    profile.apps_off.sort();
    profile.apps_off.dedup();
    Ok((file.name, profile, dropped))
}

pub fn downloads_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("FORMMCP_DOWNLOADS") {
        return PathBuf::from(dir);
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_default();
    let downloads = home.join("Downloads");
    if downloads.is_dir() { downloads } else { home }
}

pub fn export_profile(name: &str, profile: &Profile, to: Option<&Path>) -> Result<PathBuf, String> {
    let file = match to {
        Some(path) => path.to_path_buf(),
        None => {
            let slug: String = name.chars().map(|c| if c == ' ' { '-' } else { c }).collect();
            let dir = downloads_dir();
            let mut path = dir.join(format!("formmcp-profile-{slug}.json"));
            let mut n = 2;
            while path.exists() {
                path = dir.join(format!("formmcp-profile-{slug} ({n}).json"));
                n += 1;
            }
            path
        }
    };
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(&file, write_profile(name, profile)).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(file)
}

pub fn write_profile(name: &str, profile: &Profile) -> String {
    let file = ProfileFile { format: PROFILE_FORMAT.into(), format_version: 1, name: name.into(), profile: profile.clone() };
    serde_json::to_string_pretty(&file).unwrap_or_default() + "\n"
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
        self.off.get(client).is_some_and(|list| list.iter().any(|a| a == app)) || self.profile_hides_app(client, app)
    }

    pub fn profile_of(&self, client: &str) -> Option<(&String, &Profile)> {
        let name = self.use_profile.get(client)?;
        self.profiles.get_key_value(name)
    }

    pub fn profile_hides_app(&self, client: &str, app: &str) -> bool {
        self.profile_of(client).is_some_and(|(_, p)| p.apps_off.iter().any(|a| a == app))
    }

    pub fn hides_tool(&self, client: &str, app: &str, tool: &str) -> bool {
        self.tool_hidden(app, tool) || self.profile_of(client).is_some_and(|(_, p)| p.hidden_tools.get(app).is_some_and(|l| l.iter().any(|t| t == tool)))
    }

    pub fn start_of(&self, client: &str, app: &str) -> &str {
        self.profile_of(client).and_then(|(_, p)| p.start.get(app)).or_else(|| self.start.get(app)).map(String::as_str).filter(|p| START_POLICIES.contains(p)).unwrap_or("demand")
    }

    pub fn set_start(&mut self, app: &str, policy: &str) -> Result<(), String> {
        if !START_POLICIES.contains(&policy) {
            return Err(format!("{policy:?}: use demand, session or keep"));
        }
        if policy == "demand" {
            self.start.remove(app);
        } else {
            self.start.insert(app.to_string(), policy.to_string());
        }
        Ok(())
    }

    pub fn profile_from(&self, client: &str) -> Profile {
        Profile { apps_off: self.off.get(client).cloned().unwrap_or_default(), hidden_tools: self.hidden_tools.clone(), start: self.start.clone() }
    }

    pub fn set_profile_for(&mut self, client: &str, name: Option<&str>) -> Result<(), String> {
        match name {
            Some(n) if !self.profiles.contains_key(n) => Err(format!("there is no profile {n:?}")),
            Some(n) => {
                self.use_profile.insert(client.to_string(), n.to_string());
                Ok(())
            }
            None => {
                self.use_profile.remove(client);
                Ok(())
            }
        }
    }

    pub fn delete_profile(&mut self, name: &str) -> Result<(), String> {
        self.profiles.remove(name).ok_or_else(|| format!("there is no profile {name:?}"))?;
        self.use_profile.retain(|_, n| n != name);
        Ok(())
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
