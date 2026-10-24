use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub expose: Vec<String>,
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
    pub fn load() -> Settings {
        let mut settings: Settings = std::fs::read_to_string(path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
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
}
