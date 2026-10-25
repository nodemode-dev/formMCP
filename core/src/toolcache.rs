use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::settings::dir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolLine {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Known {
    pub app: String,
    pub server: Value,
    pub loaded: u64,
    pub tools: Vec<ToolLine>,
}

fn plain(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') && !id.starts_with('.')
}

fn path(app: &str) -> Option<PathBuf> {
    plain(app).then(|| dir().join("tools").join(format!("{app}.json")))
}

pub fn remember(app: &str, server: &Value, tools: &[Value]) {
    let Some(file) = path(app) else { return };
    let lines = tools
        .iter()
        .filter_map(|t| {
            let name = t.get("name").and_then(Value::as_str)?.to_string();
            let description: String = t.get("description").and_then(Value::as_str).unwrap_or_default().chars().take(240).collect();
            let read_only = t.pointer("/annotations/readOnlyHint").and_then(Value::as_bool).unwrap_or(false);
            Some(ToolLine { name, description, read_only })
        })
        .collect();
    let loaded = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let known = Known { app: app.to_string(), server: server.clone(), loaded, tools: lines };
    let Ok(text) = serde_json::to_string_pretty(&known) else { return };
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let part = file.with_extension(format!("part-{}", std::process::id()));
    if std::fs::write(&part, text).and_then(|()| std::fs::rename(&part, &file)).is_err() {
        let _ = std::fs::remove_file(&part);
    }
}

pub fn known(app: &str) -> Option<Known> {
    serde_json::from_str(&std::fs::read_to_string(path(app)?).ok()?).ok()
}
