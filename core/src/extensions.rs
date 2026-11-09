use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use regex::Regex;
use serde_json::{json, Map, Value};

use crate::settings;
use crate::CORE_VERSION;

pub const API: &str = "1";
pub const PRODUCT: &str = "formmcp";
pub const APP_NAME: &str = "FormMCP";
pub const HTTP_BASE: &str = "/api/extensions/files";
#[cfg(any(windows, target_os = "android"))]
pub const PROTOCOL_BASE: &str = "http://formext.localhost";
#[cfg(not(any(windows, target_os = "android")))]
pub const PROTOCOL_BASE: &str = "formext:/";

const TYPES: [(&str, &str); 21] = [
    (".html", "text/html; charset=utf-8"),
    (".js", "text/javascript; charset=utf-8"),
    (".mjs", "text/javascript; charset=utf-8"),
    (".css", "text/css; charset=utf-8"),
    (".json", "application/json"),
    (".svg", "image/svg+xml"),
    (".png", "image/png"),
    (".webp", "image/webp"),
    (".jpg", "image/jpeg"),
    (".jpeg", "image/jpeg"),
    (".gif", "image/gif"),
    (".wasm", "application/wasm"),
    (".txt", "text/plain; charset=utf-8"),
    (".md", "text/markdown; charset=utf-8"),
    (".wav", "audio/wav"),
    (".ogg", "audio/ogg"),
    (".mp3", "audio/mpeg"),
    (".webm", "video/webm"),
    (".mp4", "video/mp4"),
    (".cube", "text/plain; charset=utf-8"),
    (".woff2", "font/woff2"),
];
const PERMISSIONS: [&str; 6] = ["document", "files-read", "files-write", "network", "models", "native-code"];

fn id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z0-9][a-z0-9-]{1,63}$").expect("id pattern"))
}

fn local_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z0-9][a-z0-9-]{0,31}$").expect("local id pattern"))
}

fn version_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+$").expect("version pattern"))
}

pub fn portal_root() -> PathBuf {
    let env = std::env::var("FORMPORTAL_ROOT").unwrap_or_default();
    if !env.trim().is_empty() {
        return PathBuf::from(env.trim());
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_default();
    if cfg!(target_os = "macos") {
        let support = home.join("Library").join("Application Support");
        let (new, old) = (support.join("studio.formai.portal"), support.join("dev.formai.portal"));
        return if old.is_dir() && !new.exists() { old } else { new };
    }
    if cfg!(windows) {
        let base = std::env::var_os("LOCALAPPDATA").filter(|b| !b.is_empty());
        return base.map(PathBuf::from).unwrap_or_else(|| home.join("AppData").join("Local")).join("FormPortal");
    }
    let base = std::env::var_os("XDG_DATA_HOME").filter(|b| !b.is_empty());
    base.map(PathBuf::from).unwrap_or_else(|| home.join(".local").join("share")).join("formportal")
}

pub fn version_tuple(v: &str) -> Vec<i64> {
    let parts: Result<Vec<i64>, _> = v.split('.').take(3).map(|p| p.trim().parse::<i64>()).collect();
    parts.unwrap_or_else(|_| vec![0, 0, 0])
}

#[derive(Debug, Clone)]
pub struct Extension {
    pub id: String,
    pub version: String,
    pub source: String,
    pub path: PathBuf,
    pub name: String,
    pub kind: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub entry_ui: String,
    pub panels: Vec<Value>,
    pub commands: Vec<Value>,
    pub state: String,
    pub reason: String,
}

impl Extension {
    fn new(id: &str, version: &str, source: &str, path: PathBuf) -> Extension {
        Extension { id: id.into(), version: version.into(), source: source.into(), path, name: String::new(), kind: String::new(), description: String::new(), permissions: Vec::new(), entry_ui: String::new(), panels: Vec::new(), commands: Vec::new(), state: "ready".into(), reason: String::new() }
    }

    pub fn public(&self, base: &str) -> Value {
        json!({
            "id": self.id,
            "name": if self.name.is_empty() { &self.id } else { &self.name },
            "version": self.version,
            "kind": self.kind,
            "source": self.source,
            "description": self.description,
            "permissions": self.permissions,
            "base": format!("{base}/{}/{}/", self.id, self.version),
            "entryUi": self.entry_ui,
            "panels": self.panels,
            "commands": self.commands,
            "path": if self.source == "development" { Some(self.path.display().to_string()) } else { None },
            "state": self.state,
            "reason": self.reason,
        })
    }
}

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

fn safe_rel(rel: &str) -> bool {
    !rel.is_empty() && !rel.starts_with('/') && !rel.contains('\\') && !rel.split('/').any(|p| p == "..") && rel.chars().count() <= 200
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

fn py_str(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "None".into(),
        Some(Value::Bool(true)) => "True".into(),
        Some(Value::Bool(false)) => "False".into(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

fn py_repr(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'")),
        other => py_str(other),
    }
}

fn cut(text: &str, n: usize) -> String {
    text.chars().take(n).collect()
}

fn or_value<'a>(v: Option<&'a Value>, fallback: &'a Value) -> &'a Value {
    match v {
        Some(x) if truthy(Some(x)) => x,
        _ => fallback,
    }
}

pub fn check(mut ext: Extension, product: &str, app_version: &str, app_name: &str) -> Extension {
    let refuse = |mut ext: Extension, reason: String| {
        ext.state = "refused".into();
        ext.reason = reason;
        ext
    };
    let Some(Value::Object(manifest)) = read_json(&ext.path.join("manifest.json")) else {
        return refuse(ext, "its manifest.json is missing or not valid JSON".into());
    };
    let one = |v: Option<&Value>| matches!(v, Some(Value::Bool(true))) || v.and_then(Value::as_f64) == Some(1.0);
    if manifest.get("format").and_then(Value::as_str) != Some("formai-package") || !one(manifest.get("format_version")) {
        return refuse(ext, "it is not a formai package".into());
    }
    ext.name = cut(&py_str(Some(or_value(manifest.get("name"), &Value::String(ext.id.clone())))), 80);
    ext.kind = if truthy(manifest.get("kind")) { py_str(manifest.get("kind")) } else { String::new() };
    ext.description = if truthy(manifest.get("description")) { cut(&py_str(manifest.get("description")), 400) } else { String::new() };
    let said = match manifest.get("version") {
        None => String::new(),
        v => py_str(v),
    };
    if ext.source == "development" && version_re().is_match(&said) {
        ext.version = said;
    }
    if manifest.get("id").and_then(Value::as_str) != Some(ext.id.as_str()) {
        let said = py_repr(manifest.get("id"));
        let id = py_repr(Some(&Value::String(ext.id.clone())));
        return refuse(ext, format!("its manifest says {said}, not {id}"));
    }
    if manifest.get("product").and_then(Value::as_str) != Some(product) {
        return refuse(ext, format!("it is made for {}, not {app_name}", py_str(manifest.get("product"))));
    }
    let said_version = match manifest.get("version") {
        None => String::new(),
        v => py_str(v),
    };
    if said_version != ext.version && ext.source != "development" {
        let said = py_str(manifest.get("version"));
        let portal = ext.version.clone();
        return refuse(ext, format!("its manifest says version {said}, FormPortal says {portal}"));
    }
    if truthy(manifest.get("version")) {
        ext.version = py_str(manifest.get("version"));
    }
    if !version_re().is_match(&ext.version) {
        return refuse(ext, "its version is not x.y.z".into());
    }
    if manifest.get("api").and_then(Value::as_str) != Some(API) {
        return refuse(ext, format!("it needs extension API {}; this {app_name} offers API {API}", py_str(manifest.get("api"))));
    }
    if truthy(manifest.get("native")) {
        return refuse(ext, format!("native extensions are not supported by {app_name}"));
    }
    let empty = Value::Array(Vec::new());
    let perms = or_value(manifest.get("permissions"), &empty);
    let Some(perms) = perms.as_array().filter(|list| list.iter().all(|p| p.as_str().is_some_and(|p| PERMISSIONS.contains(&p)))) else {
        return refuse(ext, "it asks for a permission that does not exist".into());
    };
    let mut seen: Vec<String> = Vec::new();
    for p in perms.iter().filter_map(Value::as_str) {
        if !seen.iter().any(|s| s == p) {
            seen.push(p.to_string());
        }
    }
    ext.permissions = seen;
    let minimum = manifest.get("minimum_app");
    if truthy(minimum) && version_tuple(&py_str(minimum)) > version_tuple(app_version) {
        return refuse(ext, format!("it needs {app_name} {} or newer", py_str(minimum)));
    }
    let no_object = Value::Object(Map::new());
    let object = |key: &str| manifest.get(key).filter(|v| v.is_object()).unwrap_or(&no_object);
    let entry_value = object("entry").get("ui").cloned().unwrap_or_else(|| Value::String(String::new()));
    if !entry_value.is_string() {
        return refuse(ext, "its entry.ui is not a path inside the package".into());
    }
    let contributes = object("contributes");
    let listed = |key: &str| -> Vec<Map<String, Value>> {
        or_value(contributes.get(key), &empty)
            .as_array()
            .map(|list| list.iter().filter_map(Value::as_object).filter(|p| local_re().is_match(&py_str(Some(p.get("id").unwrap_or(&Value::String(String::new()))))) && truthy(p.get("title"))).cloned().collect())
            .unwrap_or_default()
    };
    let panels = listed("panels");
    let commands = listed("commands");
    if (!panels.is_empty() || !commands.is_empty()) && !truthy(Some(&entry_value)) {
        return refuse(ext, "it contributes panels or commands but has no entry.ui".into());
    }
    let mut entry = String::new();
    if truthy(Some(&entry_value)) {
        let Some(text) = entry_value.as_str().filter(|e| safe_rel(e)) else {
            return refuse(ext, "its entry.ui is not a path inside the package".into());
        };
        let inside = match (ext.path.join(text).canonicalize(), ext.path.canonicalize()) {
            (Ok(target), Ok(root)) => target.is_file() && target != root && target.starts_with(&root),
            _ => false,
        };
        if !inside {
            return refuse(ext, format!("its entry.ui {text} is not in the package"));
        }
        entry = text.to_string();
    }
    ext.entry_ui = entry;
    ext.panels = panels
        .iter()
        .map(|p| {
            let location = p.get("location").and_then(Value::as_str).filter(|l| matches!(*l, "dock" | "dialog")).unwrap_or("dock");
            json!({ "id": p["id"], "title": cut(&py_str(p.get("title")), 60), "location": location })
        })
        .collect();
    ext.commands = commands
        .iter()
        .map(|c| {
            let mut out = json!({ "id": c["id"], "title": cut(&py_str(c.get("title")), 80) });
            if truthy(c.get("panel")) {
                out["panel"] = c["panel"].clone();
            }
            out
        })
        .collect();
    ext
}

pub fn resolve(product: &str, app_version: &str, app_name: &str, root: Option<&Path>) -> Vec<Extension> {
    let folder = root.map(Path::to_path_buf).unwrap_or_else(portal_root).join("extensions").join(product);
    let mut found: Vec<Extension> = Vec::new();
    let mut put = |ext: Extension| {
        found.retain(|e| e.id != ext.id);
        found.push(ext);
    };
    let rows_of = |file: &str, format: &str| -> Vec<Map<String, Value>> {
        match read_json(&folder.join(file)) {
            Some(Value::Object(doc)) if doc.get("format").and_then(Value::as_str) == Some(format) => doc.get("extensions").and_then(Value::as_array).map(|l| l.iter().filter_map(Value::as_object).cloned().collect()).unwrap_or_default(),
            _ => Vec::new(),
        }
    };
    for row in rows_of("enabled.json", "formportal-enabled-extensions") {
        let id = py_str(Some(row.get("id").unwrap_or(&Value::String(String::new()))));
        if !id_re().is_match(&id) {
            continue;
        }
        let source = if row.get("source").and_then(Value::as_str) == Some("sideload") { "sideload" } else { "catalogue" };
        let version = py_str(Some(row.get("version").unwrap_or(&Value::String(String::new()))));
        let path = PathBuf::from(py_str(Some(row.get("path").unwrap_or(&Value::String(String::new())))));
        put(check(Extension::new(&id, &version, source, path), product, app_version, app_name));
    }
    for row in rows_of("dev.json", "formext-dev-extensions") {
        let id = py_str(Some(row.get("id").unwrap_or(&Value::String(String::new()))));
        if !id_re().is_match(&id) {
            continue;
        }
        let path = PathBuf::from(py_str(Some(row.get("path").unwrap_or(&Value::String(String::new())))));
        put(check(Extension::new(&id, "0.0.0", "development", path), product, app_version, app_name));
    }
    found.sort_by_key(|e| if e.name.is_empty() { e.id.to_lowercase() } else { e.name.to_lowercase() });
    found
}

pub fn stamp(product: &str, root: Option<&Path>) -> (f64, f64) {
    let folder = root.map(Path::to_path_buf).unwrap_or_else(portal_root).join("extensions").join(product);
    let time = |name: &str| std::fs::metadata(folder.join(name)).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs_f64()).unwrap_or(0.0);
    (time("enabled.json"), time("dev.json"))
}

pub fn csp_for(base: &str) -> String {
    let src = if base.starts_with("http://") || base.starts_with("https://") || base.starts_with("formext://") { base } else { "'self'" };
    format!(
        "default-src 'none'; script-src {src} 'wasm-unsafe-eval'; style-src {src} 'unsafe-inline'; img-src {src} data: blob:; media-src {src} data: blob:; font-src {src}; connect-src 'none'; frame-src 'none'; form-action 'none'; base-uri 'none'"
    )
}

pub fn file_for(extensions: &[Extension], ext_id: &str, version: &str, rel: &str) -> Option<(PathBuf, &'static str)> {
    let ext = extensions.iter().find(|e| e.id == ext_id && e.version == version && e.state == "ready")?;
    if !safe_rel(rel) {
        return None;
    }
    let suffix = Path::new(rel).extension().map(|s| format!(".{}", s.to_string_lossy().to_lowercase()))?;
    let content_type = TYPES.iter().find(|(s, _)| *s == suffix).map(|(_, t)| *t)?;
    let root = ext.path.canonicalize().ok()?;
    let candidate = ext.path.join(rel);
    let linked = |p: &Path| std::fs::symlink_metadata(p).map(|m| m.file_type().is_symlink()).unwrap_or(false);
    if linked(&candidate) || candidate.ancestors().skip(1).any(|p| p.canonicalize().is_ok_and(|r| r == root || r.starts_with(&root)) && linked(p)) {
        return None;
    }
    let target = candidate.canonicalize().ok()?;
    if target == root || !target.starts_with(&root) || !target.is_file() {
        return None;
    }
    Some((target, content_type))
}

pub fn headers_for(content_type: &str, base: &str) -> Vec<(String, String)> {
    let mut h: Vec<(String, String)> = [("X-Content-Type-Options", "nosniff"), ("Cache-Control", "no-store"), ("Cross-Origin-Resource-Policy", "cross-origin"), ("Access-Control-Allow-Origin", "*")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    if content_type.starts_with("text/html") {
        h.push(("Content-Security-Policy".into(), csp_for(base)));
    }
    h
}

pub fn current() -> Vec<Extension> {
    resolve(PRODUCT, CORE_VERSION, APP_NAME, None)
}

pub fn listing(base: &str) -> Value {
    let (enabled, dev) = stamp(PRODUCT, None);
    let rows: Vec<Value> = current().iter().map(|e| e.public(base)).collect();
    json!({ "product": PRODUCT, "app": APP_NAME, "appVersion": CORE_VERSION, "api": API, "root": portal_root(), "stamp": [enabled, dev], "extensions": rows })
}

pub fn stamp_value() -> Value {
    let (enabled, dev) = stamp(PRODUCT, None);
    json!([enabled, dev])
}

fn unescape(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok().filter(|s| !s.contains('\0'))
}

pub fn parse_address(address: &str) -> Option<(String, String, String)> {
    let address = address.split(['?', '#']).next().unwrap_or("");
    let rest = if let Some(rest) = address.strip_prefix("formext://") {
        match rest.split_once('/') {
            Some(("localhost" | "formext.localhost", tail)) => tail.to_string(),
            Some((id, tail)) => format!("{id}/{tail}"),
            None => return None,
        }
    } else if let Some(rest) = address.strip_prefix("http://formext.localhost/").or_else(|| address.strip_prefix("https://formext.localhost/")) {
        rest.to_string()
    } else {
        address.strip_prefix(HTTP_BASE)?.strip_prefix('/')?.to_string()
    };
    let mut parts = rest.splitn(3, '/');
    let id = unescape(parts.next()?)?;
    let version = unescape(parts.next()?)?;
    let rel = unescape(parts.next()?)?;
    Some((id, version, rel))
}

pub struct Served {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub file: Option<PathBuf>,
}

pub fn answer(address: &str, protocol: bool) -> Served {
    let found = parse_address(address).and_then(|(id, version, rel)| {
        let extensions = current();
        file_for(&extensions, &id, &version, &rel).map(|(path, kind)| (path, kind, format!("{PROTOCOL_BASE}/{id}/{version}/")))
    });
    match found {
        Some((path, kind, base)) => {
            let mut headers = headers_for(kind, if protocol { &base } else { "" });
            headers.push(("Content-Type".into(), kind.into()));
            Served { status: 200, headers, file: Some(path) }
        }
        None => Served { status: 404, headers: headers_for("text/plain", ""), file: None },
    }
}

pub fn agent_rows() -> Vec<Value> {
    current()
        .iter()
        .map(|e| {
            let commands: Vec<Value> = e.commands.iter().map(|c| json!({ "id": c["id"], "title": c["title"] })).collect();
            json!({ "id": e.id, "name": if e.name.is_empty() { &e.id } else { &e.name }, "version": e.version, "source": e.source, "state": e.state, "reason": e.reason, "permissions": e.permissions, "commands": commands })
        })
        .collect()
}

pub fn save_download(name: &str, data: &str) -> Result<PathBuf, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(data.trim()).map_err(|_| "the file's data is not base64".to_string())?;
    let clean: String = name.chars().filter(|c| !matches!(c, '/' | '\\' | ':' | '\0' | '<' | '>' | '"' | '|' | '?' | '*') && !c.is_control()).collect();
    let clean = clean.trim().trim_start_matches('.').to_string();
    let clean = if clean.is_empty() { "extension-file".to_string() } else { cut(&clean, 120) };
    let dir = settings::downloads_dir();
    let (stem, ext) = match clean.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s.to_string(), format!(".{e}")),
        _ => (clean.clone(), String::new()),
    };
    let mut path = dir.join(&clean);
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{stem} ({n}){ext}"));
        n += 1;
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

pub fn open_portal() -> Result<(), String> {
    let address = format!("formai://extensions?product={PRODUCT}");
    let mut command = if cfg!(target_os = "macos") {
        std::process::Command::new("open")
    } else if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.args(["/c", "start", ""]);
        c
    } else {
        std::process::Command::new("xdg-open")
    };
    command.arg(&address).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    command.spawn().map(|_| ()).map_err(|e| format!("FormPortal could not be opened ({e}); install it from formai.studio"))
}

pub fn settings_mark() -> String {
    serde_json::to_string(&settings::Settings::saved()).unwrap_or_default()
}

pub fn settings_restore(text: &str) -> Result<(), String> {
    let back: settings::Settings = serde_json::from_str(text).map_err(|e| format!("the settings to put back could not be read ({e})"))?;
    back.save().map(|_| ())
}

const ALIVE: &str = "window.alive";
const ALIVE_FOR: Duration = Duration::from_secs(5);
const WAIT: Duration = Duration::from_secs(300);

fn mailbox() -> PathBuf {
    settings::dir().join("extensions")
}

fn rid_ok(rid: &str) -> bool {
    !rid.is_empty() && rid.len() <= 64 && rid.bytes().all(|b| b.is_ascii_hexdigit())
}

fn new_rid() -> String {
    let mut bytes = [0u8; 12];
    if getrandom::getrandom(&mut bytes).is_err() {
        let t = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        bytes.copy_from_slice(&t.to_le_bytes()[..12]);
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let part = path.with_extension("part");
    std::fs::write(&part, text).map_err(|e| format!("{}: {e}", part.display()))?;
    std::fs::rename(&part, path).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn window_open() -> bool {
    std::fs::metadata(mailbox().join(ALIVE)).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age < ALIVE_FOR)
}

pub fn take() -> Vec<Value> {
    let dir = mailbox().join("requests");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(mailbox().join(ALIVE), CORE_VERSION);
    let mut names: Vec<PathBuf> = std::fs::read_dir(&dir).map(|r| r.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "json")).collect()).unwrap_or_default();
    names.sort();
    names
        .into_iter()
        .filter_map(|p| {
            let request = read_json(&p);
            let _ = std::fs::remove_file(&p);
            request.filter(|r| r.get("rid").and_then(Value::as_str).is_some_and(rid_ok))
        })
        .collect()
}

pub fn finish(rid: &str, outcome: &Value) -> Result<(), String> {
    if !rid_ok(rid) {
        return Err("that is not a request formMCP made".into());
    }
    write_atomic(&mailbox().join("results").join(format!("{rid}.json")), &outcome.to_string())
}

pub fn request(id: &str, command: &str, args: &Value, client: &str) -> Result<Value, String> {
    let ext = current().into_iter().find(|e| e.id == id).ok_or_else(|| format!("there is no extension {id:?} for {APP_NAME}; extensions_list has them"))?;
    if ext.state != "ready" {
        return Err(format!("{} is refused: {}", ext.name, ext.reason));
    }
    if !ext.commands.iter().any(|c| c["id"].as_str() == Some(command)) {
        return Err(format!("{} has no command {command:?}", ext.name));
    }
    if !window_open() {
        return Err(format!("extension commands run in the {APP_NAME} window; open {APP_NAME} and ask again"));
    }
    ask_window(id, &ext.name, command, args, client)
}

pub fn window_command(command: &str, args: &Value, client: &str) -> Result<Value, String> {
    if !window_open() {
        return Err(format!("this runs in the {APP_NAME} window; open {APP_NAME} and ask again"));
    }
    ask_window(PRODUCT, APP_NAME, command, args, client)
}

fn ask_window(id: &str, name: &str, command: &str, args: &Value, client: &str) -> Result<Value, String> {
    let rid = new_rid();
    write_atomic(&mailbox().join("requests").join(format!("{rid}.json")), &json!({ "rid": rid, "id": id, "command": command, "args": args, "client": client }).to_string())?;
    let result = mailbox().join("results").join(format!("{rid}.json"));
    let started = Instant::now();
    while started.elapsed() < WAIT {
        if let Some(outcome) = read_json(&result) {
            let _ = std::fs::remove_file(&result);
            return match outcome.get("error") {
                Some(e) => Err(e.get("message").and_then(Value::as_str).unwrap_or("the extension failed").to_string()),
                None => Ok(outcome.get("result").cloned().unwrap_or(Value::Null)),
            };
        }
        if started.elapsed() > ALIVE_FOR * 2 && !window_open() {
            let _ = std::fs::remove_file(mailbox().join("requests").join(format!("{rid}.json")));
            return Err(format!("the {APP_NAME} window closed before {name} ran"));
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    Err(format!("{name} did not finish within {} s", WAIT.as_secs()))
}
