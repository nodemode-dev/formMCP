use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::settings;

const MAX_FILES: usize = 200_000;
const SHOWN: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub version: String,
    pub recorded: String,
    pub path: PathBuf,
    pub files: BTreeMap<String, String>,
    pub at: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Checked {
    pub app: String,
    pub version: String,
    pub outcome: &'static str,
    pub files: usize,
    pub ms: u64,
}

#[derive(Deserialize)]
struct PortalState {
    #[serde(default)]
    installs: Vec<PortalInstall>,
}

#[derive(Deserialize)]
struct PortalInstall {
    product: String,
    channel: String,
    current: PortalVersion,
}

#[derive(Deserialize)]
struct PortalVersion {
    version: String,
    path: PathBuf,
    sha256: String,
}

pub fn path() -> PathBuf {
    settings::dir().join("verified.json")
}

fn load() -> BTreeMap<String, Baseline> {
    fs::read_to_string(path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn save(all: &BTreeMap<String, Baseline>) -> Result<(), String> {
    let file = path();
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let part = file.with_extension(format!("part-{}", std::process::id()));
    let text = serde_json::to_string_pretty(all).map_err(|e| e.to_string())? + "\n";
    fs::write(&part, text).and_then(|()| fs::rename(&part, &file)).map_err(|e| format!("{}: {e}", file.display()))
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn portal_install(portal_root: &Path, app: &str) -> Result<PortalVersion, String> {
    let file = portal_root.join("installed.json");
    let text = fs::read_to_string(&file).map_err(|e| format!("FormPortal's record {} could not be read ({e})", file.display()))?;
    let state: PortalState = serde_json::from_str(&text).map_err(|e| format!("FormPortal's record {} is not readable ({e})", file.display()))?;
    state.installs.into_iter().find(|i| i.product == app && i.channel == "stable").map(|i| i.current).ok_or_else(|| format!("FormPortal has no stable install of {app} on record"))
}

fn digest(file: &Path) -> Result<String, String> {
    let mut handle = fs::File::open(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let n = handle.read(&mut buffer).map_err(|e| format!("{}: {e}", file.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

fn walk(root: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path.strip_prefix(root).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
            let kind = entry.file_type().map_err(|e| format!("{}: {e}", path.display()))?;
            if kind.is_symlink() {
                let target = fs::read_link(&path).map(|t| t.to_string_lossy().into_owned()).unwrap_or_default();
                out.insert(rel, format!("link:{target}"));
            } else if kind.is_dir() {
                stack.push(path);
            } else {
                out.insert(rel, digest(&path)?);
            }
            if out.len() > MAX_FILES {
                return Err(format!("{} holds more than {MAX_FILES} files", root.display()));
            }
        }
    }
    Ok(out)
}

fn inside(path: &Path, portal_root: &Path, app: &str) -> bool {
    let apps = portal_root.join("apps").join(app);
    match (path.canonicalize(), apps.canonicalize()) {
        (Ok(p), Ok(a)) => p.starts_with(a),
        _ => false,
    }
}

fn differences(before: &BTreeMap<String, String>, now: &BTreeMap<String, String>) -> Vec<String> {
    let mut out = Vec::new();
    for (name, hash) in now {
        match before.get(name) {
            None => out.push(format!("{name} (new)")),
            Some(old) if old != hash => out.push(format!("{name} (changed)")),
            _ => {}
        }
    }
    for name in before.keys().filter(|n| !now.contains_key(*n)) {
        out.push(format!("{name} (gone)"));
    }
    out
}

pub fn check(portal_root: &Path, app: &str) -> Result<Checked, String> {
    let started = Instant::now();
    let install = portal_install(portal_root, app)?;
    if !inside(&install.path, portal_root, app) {
        return Err(format!("FormPortal's record puts {app} at {}, outside FormPortal's own apps folder, so formMCP will not start it", install.path.display()));
    }
    let files = walk(&install.path)?;
    if files.is_empty() {
        return Err(format!("{} is empty", install.path.display()));
    }
    let mut all = load();
    let outcome = match all.get(app) {
        Some(base) if base.version == install.version && base.recorded == install.sha256 && base.path == install.path => {
            let changed = differences(&base.files, &files);
            if !changed.is_empty() {
                let more = if changed.len() > SHOWN { format!(" and {} more", changed.len() - SHOWN) } else { String::new() };
                return Err(format!(
                    "{app} {} changed on disk since formMCP first checked it: {}{more}. formMCP will not start it. Reinstall it with FormPortal, or if you made the change yourself, run formmcp-cli verify --trust {app}",
                    install.version,
                    changed.iter().take(SHOWN).cloned().collect::<Vec<_>>().join(", ")
                ));
            }
            "match"
        }
        _ => {
            all.insert(app.to_string(), Baseline { version: install.version.clone(), recorded: install.sha256.clone(), path: install.path.clone(), files: files.clone(), at: now() });
            save(&all)?;
            "baseline"
        }
    };
    Ok(Checked { app: app.to_string(), version: install.version, outcome, files: files.len(), ms: started.elapsed().as_millis() as u64 })
}

pub fn trust(portal_root: &Path, app: &str) -> Result<Checked, String> {
    let mut all = load();
    all.remove(app);
    save(&all)?;
    check(portal_root, app)
}

pub fn baselines() -> BTreeMap<String, (String, usize, u64)> {
    load().into_iter().map(|(k, b)| (k, (b.version, b.files.len(), b.at))).collect()
}
