use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::catalog;
use crate::child::{call_timeout, AppServer};
use crate::family::{App, Family};
use crate::reach::{Places, Route};
use crate::settings::{client_key, client_label, Settings};
use crate::suggest::suggest;
use crate::{extensions, hooks, log, redact, secrets, telemetry, toolcache, verify};
use crate::CORE_VERSION;

const EXPOSED_SEPARATOR: &str = "__";

pub struct Gateway {
    pub family: Family,
    pub places: Places,
    pub settings: Settings,
    pub client: String,
    pub transport: &'static str,
    forced_client: bool,
    running: HashMap<String, (String, AppServer)>,
    pending: HashMap<String, Receiver<Result<(String, AppServer), String>>>,
    last_used: HashMap<String, Instant>,
    catalog_read: bool,
    list_changed: bool,
}

impl Gateway {
    pub fn new(family: Family, places: Places) -> Gateway {
        let settings = Settings::load();
        secrets::remember_all(&settings);
        Gateway { family, places, settings, client: "other".into(), transport: "stdio", forced_client: false, running: HashMap::new(), pending: HashMap::new(), last_used: HashMap::new(), catalog_read: false, list_changed: false }
    }

    pub fn force_client(&mut self, name: &str) {
        self.client = client_key(name).to_string();
        self.forced_client = true;
    }

    pub fn set_session_client(&mut self, asked: Option<&str>) {
        self.forced_client = asked.is_some();
        self.client = asked.map(client_key).unwrap_or("other").to_string();
    }

    pub fn reload_settings(&mut self) {
        let fresh = Settings::load();
        if fresh.secrets != self.settings.secrets {
            secrets::remember_all(&fresh);
        }
        self.settings = fresh;
    }

    pub fn start_app(&mut self, id: &str) -> Result<usize, String> {
        self.ensure_catalog();
        let app = self.family.get(id).cloned().ok_or_else(|| format!("there is no Form app {id:?}"))?;
        let (_, server) = self.server(&app)?;
        Ok(server.tools.len())
    }

    pub fn is_off(&self, id: &str) -> bool {
        self.settings.is_off(&self.client, id)
    }

    pub fn ensure_catalog(&mut self) {
        if !self.catalog_read {
            self.catalog_read = true;
            self.family = catalog::load(self.family.clone());
        }
    }

    pub fn take_list_changed(&mut self) -> bool {
        std::mem::replace(&mut self.list_changed, false)
    }

    fn exposed_tools(&mut self) -> Vec<Value> {
        self.ensure_catalog();
        let mut out = Vec::new();
        for id in self.settings.expose.clone() {
            let Some(app) = self.family.get(&id).cloned() else { continue };
            if self.is_off(&app.id) {
                continue;
            }
            let started = self.server(&app).map(|(_, s)| {
                if s.tools_changed {
                    let _ = s.load_tools();
                }
                s.tools.clone()
            });
            match started {
                Ok(tools) => {
                    for tool in tools {
                        let Some(name) = tool.get("name").and_then(Value::as_str) else { continue };
                        if self.settings.hides_tool(&self.client, &app.id, name) {
                            continue;
                        }
                        let mut renamed = tool.clone();
                        renamed["name"] = Value::String(format!("{}{EXPOSED_SEPARATOR}{name}", app.id));
                        let said = tool.get("description").and_then(Value::as_str).unwrap_or_default();
                        renamed["description"] = Value::String(format!("[{}] {said}", app.name));
                        out.push(renamed);
                    }
                }
                Err(e) => eprintln!("formmcp: {} is set to always show its tools, but it could not be started: {e}", app.name),
            }
        }
        out
    }

    fn app(&self, args: &Value) -> Result<App, String> {
        let name = args.get("app").and_then(Value::as_str).unwrap_or_default();
        if name.trim().is_empty() {
            return Err("app is required: an id from family_apps, for example formvis".into());
        }
        let app = self.family.get(name).cloned().ok_or_else(|| format!("there is no Form app {name:?}; family_apps lists them"))?;
        if self.is_off(&app.id) {
            if let Some((profile, _)) = self.settings.profile_of(&self.client).filter(|_| self.settings.profile_hides_app(&self.client, &app.id)) {
                return Err(format!("{} is turned off for {} by the profile {profile:?} in formMCP; the person can change that in formMCP's window", app.name, client_label(&self.client)));
            }
            return Err(format!("{} is turned off for {} in formMCP; the person can turn it on in formMCP's window", app.name, client_label(&self.client)));
        }
        Ok(app)
    }

    fn server(&mut self, app: &App) -> Result<&mut (String, AppServer), String> {
        let stale = self.running.get_mut(&app.id).map(|(_, s)| !s.alive()).unwrap_or(false);
        if stale {
            self.running.remove(&app.id);
        }
        if !self.running.contains_key(&app.id) {
            let early = self.pending.remove(&app.id).and_then(|rx| rx.recv().ok());
            let started = match early {
                Some(result) => result,
                None => {
                    self.places.refresh();
                    start_app_server(app, &self.places, self.settings.start_of(&self.client, &app.id), &self.settings)
                }
            };
            self.running.insert(app.id.clone(), started?);
        }
        self.last_used.insert(app.id.clone(), Instant::now());
        self.running.get_mut(&app.id).ok_or_else(|| "the app's server is gone".to_string())
    }

    pub fn start_early(&mut self) {
        let wanted = |m: &std::collections::BTreeMap<String, String>| m.values().any(|p| p == "session");
        if !wanted(&self.settings.start) && !self.settings.profile_of(&self.client).is_some_and(|(_, p)| wanted(&p.start)) {
            return;
        }
        self.ensure_catalog();
        self.places.refresh();
        for app in self.family.apps.clone() {
            if !app.mcp || self.is_off(&app.id) || self.running.contains_key(&app.id) || self.pending.contains_key(&app.id) {
                continue;
            }
            if self.settings.start_of(&self.client, &app.id) != "session" || self.places.reach(&app).route.is_none() {
                continue;
            }
            let (send, receive) = mpsc::channel();
            let places = self.places.clone();
            let settings = self.settings.clone();
            let id = app.id.clone();
            thread::spawn(move || {
                let _ = send.send(start_app_server(&app, &places, "session", &settings));
            });
            self.pending.insert(id, receive);
        }
    }

    pub fn sweep_idle(&mut self) {
        let idle = idle_limit();
        let stale: Vec<String> = self
            .running
            .keys()
            .filter(|id| self.settings.start_of(&self.client, id) == "demand" && self.last_used.get(*id).is_some_and(|t| t.elapsed() >= idle))
            .cloned()
            .collect();
        for id in stale {
            self.running.remove(&id);
            self.last_used.remove(&id);
            log::record("app-stop", json!({ "app": id, "reason": "idle", "minutes": idle.as_secs() / 60 }));
            self.list_changed |= self.settings.expose.iter().any(|e| e == &id);
        }
    }

    pub fn running_apps(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.running.keys().cloned().collect();
        ids.sort();
        ids
    }
}

pub fn idle_limit() -> Duration {
    let minutes = std::env::var("FORMMCP_IDLE_MINUTES").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(10);
    Duration::from_secs(minutes.saturating_mul(60))
}

fn start_app_server(app: &App, places: &Places, policy: &str, settings: &Settings) -> Result<(String, AppServer), String> {
    if !app.mcp {
        return Err(format!("{} has no MCP server yet, so there are no tools to reach", app.name));
    }
    let reach = places.reach(app);
    let Some(route) = reach.route else {
        let how = if reach.installed {
            "it is installed, but no FormPortal program was found to start it (set FORMPORTAL_BIN)"
        } else {
            "it is not installed: FormPortal installs it, or set FORMMCP_DEV_ROOTS to a folder holding its checkout"
        };
        return Err(format!("{} cannot be reached: {how}", app.name));
    };
    let (program, args, dir) = route.command();
    let started = Instant::now();
    let wall = std::time::SystemTime::now();
    let span = |ok: bool, error: Option<&str>| {
        telemetry::export(settings.telemetry.as_deref(), telemetry::Span { name: format!("app start {}", app.id), start: wall, ms: started.elapsed().as_millis() as u64, ok, error, attributes: vec![("formmcp.app", app.id.clone()), ("formmcp.via", route.label().to_string()), ("formmcp.policy", policy.to_string())] });
    };
    let failed = |stage: &str, e: String| {
        let short: String = e.lines().next().unwrap_or_default().chars().take(300).collect();
        log::record("app-start", json!({ "app": app.id, "via": route.label(), "ok": false, "error": short, "stage": stage, "ms": started.elapsed().as_millis() as u64, "policy": policy }));
        span(false, Some(stage));
        e
    };
    let checked = match (&route, settings.verify_off) {
        (Route::Formportal { .. }, false) => Some(verify::check(&places.portal_root, &app.id).map_err(|e| failed("verify", format!("{} was not started: {e}", app.name)))?),
        _ => None,
    };
    let env = secrets::env_for(settings, &app.id).map_err(|e| failed("secrets", format!("{} was not started: {e}", app.name)))?;
    let server = AppServer::start(&program, &args, dir.as_ref(), &env).map_err(|e| failed("start", format!("{} did not start through {}: {e}", app.name, route.label())))?;
    log::record("app-start", json!({ "app": app.id, "via": route.label(), "ok": true, "tools": server.tools.len(), "ms": started.elapsed().as_millis() as u64, "policy": policy, "verified": checked.as_ref().map(|c| c.outcome), "files": checked.as_ref().map(|c| c.files), "verify_ms": checked.as_ref().map(|c| c.ms), "secrets": env.len() }));
    span(true, None);
    toolcache::remember(&app.id, &server.server, &server.tools);
    Ok((route.label().to_string(), server))
}

const PROTOCOL: &str = "2025-06-18";
const SUPPORTED: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

pub fn supported(version: &str) -> bool {
    SUPPORTED.contains(&version.trim())
}

const INSTRUCTIONS: &str = "formMCP is the one connection to the Form family of desktop apps (image, video, audio, layout, vector, notes and more, by nodemode.ai). Call suggest_app with the person's task to get the app that fits and the next step, or family_apps for the whole list (whether each app has an MCP server and whether this computer can reach it). For a reachable app, app_tools starts it and lists its own tools, and call_app runs one of them; load one app at a time, only when the task needs it. An app's own resources and prompts, when it offers them, come through app_resources and read_app_resource, app_prompts and get_app_prompt. The person may have turned some apps or tools off for this agent in formMCP's window: they are left out of the lists and refused with the reason. formMCP never installs anything: when an app is missing, tell the person which one fits and that FormPortal installs and updates the Form apps. formMCP does not handle skills: the Form apps' released skills are viewed and downloaded in formskills.";

fn tools() -> Value {
    json!([
        { "name": "family_apps", "title": "The Form family",
          "description": "Every Form app: id, name, what kind of work it does, one line on what it is for, whether it has an MCP server agents can use, its release state (live, public or prelaunch), whether FormPortal installed it on this computer, and reach: how formMCP can start its server here (dev for a local checkout, formportal for an install, null when it cannot). Use it to pick the app that fits a task. Short on purpose: no app's tools are listed here.",
          "inputSchema": { "type": "object", "properties": {
              "mcp_only": { "type": "boolean", "description": "Only the apps that have an MCP server." } } },
          "annotations": { "readOnlyHint": true, "openWorldHint": false } },
        { "name": "suggest_app", "title": "Which Form app fits a task",
          "description": "Given the person's task in their own words, returns up to five Form apps that fit, best first, each with whether it has an MCP server, whether it is installed and reachable here, and the next step: call app_tools, install it with FormPortal, install FormPortal first, or why an agent cannot drive it yet. It never installs anything.",
          "inputSchema": { "type": "object", "properties": {
              "task": { "type": "string", "description": "What the person wants done, for example 'cut the silences out of this interview video'." },
              "limit": { "type": "integer", "minimum": 1, "maximum": 5, "description": "How many apps at most (3 when left out)." } }, "required": ["task"] },
          "annotations": { "readOnlyHint": true, "openWorldHint": false } },
        { "name": "app_tools", "title": "One app's tools",
          "description": "Starts one Form app's own MCP server (or reuses it once started) and returns its tools with their input schemas, its name and version, and its instructions. Call it for the one app the task needs, after family_apps shows it is reachable. Fails with the reason when the app has no MCP server or cannot be reached.",
          "inputSchema": { "type": "object", "properties": {
              "app": { "type": "string", "description": "An id from family_apps, for example formvis." },
              "refresh": { "type": "boolean", "description": "Ask the app for its tools again." } }, "required": ["app"] },
          "annotations": { "readOnlyHint": true, "openWorldHint": false } },
        { "name": "call_app", "title": "Run one of an app's tools",
          "description": "Runs a tool from app_tools in that Form app and returns the app's own result unchanged. What it changes is what the app's tool changes: read that tool's description and annotations first, and confirm with the person before anything that changes their files.",
          "inputSchema": { "type": "object", "properties": {
              "app": { "type": "string", "description": "An id from family_apps." },
              "tool": { "type": "string", "description": "A tool name from app_tools." },
              "arguments": { "type": "object", "description": "The tool's arguments, as its input schema in app_tools describes." } }, "required": ["app", "tool"] },
          "annotations": { "readOnlyHint": false, "destructiveHint": true, "openWorldHint": true } },
        { "name": "app_resources", "title": "One app's resources",
          "description": "Starts one Form app's MCP server (or reuses it) and lists the resources it offers (documents, project state, references, each with a uri) and its resource templates. Empty, with a note, when the app offers none.",
          "inputSchema": { "type": "object", "properties": {
              "app": { "type": "string", "description": "An id from family_apps." } }, "required": ["app"] },
          "annotations": { "readOnlyHint": true, "openWorldHint": false } },
        { "name": "read_app_resource", "title": "Read one app resource",
          "description": "Reads one resource by the uri app_resources gave (or one filled in from its templates) and returns its contents as the app sent them.",
          "inputSchema": { "type": "object", "properties": {
              "app": { "type": "string", "description": "An id from family_apps." },
              "uri": { "type": "string", "description": "The resource's uri." } }, "required": ["app", "uri"] },
          "annotations": { "readOnlyHint": true, "openWorldHint": false } },
        { "name": "app_prompts", "title": "One app's prompts",
          "description": "Lists the prompt templates one Form app offers, with their arguments. Empty, with a note, when the app offers none.",
          "inputSchema": { "type": "object", "properties": {
              "app": { "type": "string", "description": "An id from family_apps." } }, "required": ["app"] },
          "annotations": { "readOnlyHint": true, "openWorldHint": false } },
        { "name": "get_app_prompt", "title": "Fill one app prompt",
          "description": "Fills one prompt from app_prompts with its arguments and returns the app's messages, ready to follow.",
          "inputSchema": { "type": "object", "properties": {
              "app": { "type": "string", "description": "An id from family_apps." },
              "name": { "type": "string", "description": "A prompt name from app_prompts." },
              "arguments": { "type": "object", "description": "The prompt's arguments as strings.", "additionalProperties": { "type": "string" } } }, "required": ["app", "name"] },
          "annotations": { "readOnlyHint": true, "openWorldHint": false } },
        { "name": "extensions_list", "title": "formMCP's extensions",
          "description": "The extensions installed in formMCP itself (by FormPortal, or run from a folder by form.ext): id, name, version, source (catalogue, sideload, development), state (ready or refused, with the reason), the permissions each asks for, and the commands each adds. These extend formMCP's window, not the other Form apps.",
          "inputSchema": { "type": "object", "properties": {} },
          "annotations": { "readOnlyHint": true, "openWorldHint": false } },
        { "name": "extension_command", "title": "Run an extension's command",
          "description": "Runs one command an extension adds to formMCP, in formMCP's window, which must be open. The first time an extension version runs, the window asks the person to allow what it asks for; a refusal fails the call. What it changes is what the extension's command does to formMCP's settings: say so and get the person's go-ahead first.",
          "inputSchema": { "type": "object", "properties": {
              "id": { "type": "string", "description": "An extension id from extensions_list." },
              "command": { "type": "string", "description": "A command id from that extension's commands." },
              "args": { "type": "object", "description": "Arguments for the command, when it takes any." } }, "required": ["id", "command"] },
          "annotations": { "readOnlyHint": false, "destructiveHint": false, "openWorldHint": false } }
    ])
}

fn suggest_app(gateway: &mut Gateway, args: &Value) -> Result<Value, String> {
    let task = args.get("task").and_then(Value::as_str).unwrap_or_default().trim().to_string();
    if task.is_empty() {
        return Err("task is required: what the person wants done, in their words".into());
    }
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(3) as usize;
    gateway.places.refresh();
    let running: HashMap<String, String> = gateway.running.iter().map(|(k, (v, _))| (k.clone(), v.clone())).collect();
    let mut picks = suggest(&gateway.family, &gateway.places, &|id: &str| running.get(id).cloned(), &task, 5);
    picks.retain(|p| !gateway.is_off(&p.id));
    picks.truncate(limit);
    let note = if picks.is_empty() { "No app matched the words of the task; call family_apps and choose by kind and summary." } else { "Best first. formMCP never installs: FormPortal installs and updates the Form apps." };
    Ok(json!({ "task": task, "apps": picks, "note": note }))
}

fn family_apps(gateway: &mut Gateway, args: &Value) -> Value {
    let mcp_only = args.get("mcp_only").and_then(Value::as_bool).unwrap_or(false);
    gateway.places.refresh();
    let Gateway { family, places, running, settings, client, .. } = gateway;
    let off = family.apps.iter().filter(|a| settings.is_off(client, &a.id)).count();
    let apps: Vec<Value> = family
        .apps
        .iter()
        .filter(|a| !mcp_only || a.mcp)
        .filter(|a| !settings.is_off(client, &a.id))
        .map(|a| {
            let reach = places.reach(a);
            let via = running.get(&a.id).map(|(v, _)| v.clone()).or_else(|| reach.route.as_ref().map(|r| r.label().to_string()));
            json!({ "id": a.id, "name": a.name, "kind": a.kind, "summary": a.summary, "mcp": a.mcp, "state": a.state, "installed": reach.installed, "reach": via })
        })
        .collect();
    json!({
        "count": apps.len(),
        "apps": apps,
        "source": family.describe_source(),
        "turned_off": off,
        "install": "formMCP never installs. FormPortal (id formai) installs, updates and removes the Form apps."
    })
}

fn app_tools(gateway: &mut Gateway, args: &Value) -> Result<Value, String> {
    let app = gateway.app(args)?;
    let refresh = args.get("refresh").and_then(Value::as_bool).unwrap_or(false);
    let (settings, client) = (gateway.settings.clone(), gateway.client.clone());
    let (via, server) = gateway.server(&app)?;
    if refresh || server.tools_changed {
        server.load_tools()?;
        toolcache::remember(&app.id, &server.server, &server.tools);
    }
    let named = |t: &Value| t.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
    let shown: Vec<Value> = server.tools.iter().filter(|t| !settings.hides_tool(&client, &app.id, &named(t))).cloned().collect();
    let hidden: Vec<String> = server.tools.iter().map(named).filter(|n| settings.hides_tool(&client, &app.id, n)).collect();
    Ok(json!({
        "app": app.id,
        "via": via,
        "server": server.server,
        "instructions": server.instructions,
        "count": shown.len(),
        "tools": shown,
        "hidden": hidden,
        "offers": { "resources": server.offers("resources"), "prompts": server.offers("prompts") }
    }))
}

fn app_resources(gateway: &mut Gateway, args: &Value) -> Result<Value, String> {
    let app = gateway.app(args)?;
    let (via, server) = gateway.server(&app)?;
    if !server.offers("resources") {
        return Ok(json!({ "app": app.id, "via": via, "count": 0, "resources": [], "templates": [], "note": format!("{} offers no resources", app.name) }));
    }
    let resources = server.list_all("resources/list", "resources")?;
    let templates = server.list_all("resources/templates/list", "resourceTemplates").unwrap_or_default();
    Ok(json!({ "app": app.id, "via": via, "count": resources.len(), "resources": resources, "templates": templates }))
}

fn read_app_resource(gateway: &mut Gateway, args: &Value) -> Result<Value, String> {
    let app = gateway.app(args)?;
    let uri = args.get("uri").and_then(Value::as_str).unwrap_or_default().to_string();
    if uri.trim().is_empty() {
        return Err("uri is required: one from app_resources".into());
    }
    let (_, server) = gateway.server(&app)?;
    if !server.offers("resources") {
        return Err(format!("{} offers no resources", app.name));
    }
    let read = server.request("resources/read", json!({ "uri": uri }), call_timeout())?;
    let contents: Vec<Value> = read.get("contents").and_then(Value::as_array).cloned().unwrap_or_default();
    let blocks: Vec<Value> = contents.into_iter().map(|c| json!({ "type": "resource", "resource": c })).collect();
    Ok(json!({ "content": blocks, "isError": false }))
}

fn app_prompts(gateway: &mut Gateway, args: &Value) -> Result<Value, String> {
    let app = gateway.app(args)?;
    let (via, server) = gateway.server(&app)?;
    if !server.offers("prompts") {
        return Ok(json!({ "app": app.id, "via": via, "count": 0, "prompts": [], "note": format!("{} offers no prompts", app.name) }));
    }
    let prompts = server.list_all("prompts/list", "prompts")?;
    Ok(json!({ "app": app.id, "via": via, "count": prompts.len(), "prompts": prompts }))
}

fn get_app_prompt(gateway: &mut Gateway, args: &Value) -> Result<Value, String> {
    let app = gateway.app(args)?;
    let name = args.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
    if name.trim().is_empty() {
        return Err("name is required: a prompt name from app_prompts".into());
    }
    let arguments = args.get("arguments").cloned().unwrap_or_else(|| json!({}));
    let (_, server) = gateway.server(&app)?;
    if !server.offers("prompts") {
        return Err(format!("{} offers no prompts", app.name));
    }
    let filled = server.request("prompts/get", json!({ "name": name, "arguments": arguments }), call_timeout())?;
    Ok(json!({ "app": app.id, "prompt": name, "description": filled.get("description"), "messages": filled.get("messages") }))
}

fn call_app(gateway: &mut Gateway, args: &Value) -> Result<Value, String> {
    let app = gateway.app(args)?;
    let tool = args.get("tool").and_then(Value::as_str).unwrap_or_default().to_string();
    if tool.is_empty() {
        return Err("tool is required: a name from app_tools".into());
    }
    if gateway.settings.hides_tool(&gateway.client, &app.id, &tool) {
        return Err(format!("{tool} in {} is turned off in formMCP; the person can turn it on in formMCP's window", app.name));
    }
    let arguments = args.get("arguments").cloned().unwrap_or_else(|| json!({}));
    let exposed = gateway.settings.expose.iter().any(|e| e == &app.id);
    let hook_set = gateway.settings.hooks.clone();
    let event = |kind: &str| {
        let mut shown = arguments.clone();
        redact::value(&mut shown);
        hooks::shorten(&mut shown);
        json!({ "event": kind, "app": app.id, "tool": tool, "arguments": shown, "client": gateway.client, "transport": gateway.transport, "formmcp": CORE_VERSION })
    };
    let before = event("before");
    let mut after = event("after");
    hooks::before(&hook_set.before, &before)?;
    let started = Instant::now();
    let (_, server) = gateway.server(&app)?;
    let result = server.request("tools/call", json!({ "name": tool, "arguments": arguments }), call_timeout());
    if exposed && server.tools_changed {
        gateway.list_changed = true;
    }
    if !hook_set.after.trim().is_empty() {
        let mut shown = result.clone().unwrap_or_else(|e| json!({ "error": e }));
        redact::value(&mut shown);
        hooks::shorten(&mut shown);
        let ok = result.as_ref().is_ok_and(|r| !r.get("isError").and_then(Value::as_bool).unwrap_or(false));
        after["ok"] = json!(ok);
        after["ms"] = json!(started.elapsed().as_millis() as u64);
        after["result"] = shown;
        hooks::after(&hook_set.after, after);
    }
    result
}

enum Outcome {
    Mine(Value),
    Passed(Value),
}

fn call(gateway: &mut Gateway, name: &str, args: &Value) -> Result<Outcome, String> {
    gateway.ensure_catalog();
    match name {
        "family_apps" => Ok(Outcome::Mine(family_apps(gateway, args))),
        "suggest_app" => suggest_app(gateway, args).map(Outcome::Mine),
        "app_tools" => app_tools(gateway, args).map(Outcome::Mine),
        "call_app" => call_app(gateway, args).map(Outcome::Passed),
        "app_resources" => app_resources(gateway, args).map(Outcome::Mine),
        "read_app_resource" => read_app_resource(gateway, args).map(Outcome::Passed),
        "app_prompts" => app_prompts(gateway, args).map(Outcome::Mine),
        "get_app_prompt" => get_app_prompt(gateway, args).map(Outcome::Mine),
        "extensions_list" => Ok(Outcome::Mine(json!({ "extensions": extensions::agent_rows() }))),
        "extension_command" => {
            let text = |key: &str| args.get(key).and_then(Value::as_str).map(str::to_string).ok_or_else(|| format!("{key} is required"));
            let (id, command) = (text("id")?, text("command")?);
            let given = args.get("args").cloned().unwrap_or_else(|| json!({}));
            extensions::request(&id, &command, &given, &gateway.client).map(|result| Outcome::Mine(json!({ "id": id, "command": command, "result": result })))
        }
        other => match other.split_once(EXPOSED_SEPARATOR) {
            Some((id, tool)) if gateway.settings.expose.iter().any(|e| e == id) && !gateway.is_off(id) => {
                call_app(gateway, &json!({ "app": id, "tool": tool, "arguments": args })).map(Outcome::Passed)
            }
            _ => Err(format!("no tool named {other:?}; tools/list has the tools")),
        },
    }
}

pub fn answer(gateway: &mut Gateway, msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned()?;
    let method = msg.get("method").and_then(Value::as_str).unwrap_or_default();
    gateway.reload_settings();
    gateway.sweep_idle();
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => {
            let asked = params.get("protocolVersion").and_then(Value::as_str).unwrap_or(PROTOCOL);
            let version = if SUPPORTED.contains(&asked) { asked } else { PROTOCOL };
            let client = params.get("clientInfo").cloned().unwrap_or(Value::Null);
            if !gateway.forced_client {
                gateway.client = client_key(client.get("name").and_then(Value::as_str).unwrap_or_default()).to_string();
            }
            log::record("session", json!({ "client": client.get("name"), "client_version": client.get("version"), "as": gateway.client, "transport": gateway.transport, "expose": gateway.settings.expose, "profile": gateway.settings.use_profile.get(&gateway.client) }));
            gateway.start_early();
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": { "listChanged": !gateway.settings.expose.is_empty() && gateway.transport == "stdio" } },
                "serverInfo": { "name": "formmcp", "title": "formMCP", "version": CORE_VERSION },
                "instructions": INSTRUCTIONS
            })
        }
        "ping" => json!({}),
        "tools/list" => {
            let mut list = tools().as_array().cloned().unwrap_or_default();
            list.extend(gateway.exposed_tools());
            json!({ "tools": list })
        }
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            let started = std::time::Instant::now();
            let wall = std::time::SystemTime::now();
            let mut answer = match call(gateway, name, &args) {
                Ok(Outcome::Passed(result)) => result,
                Ok(Outcome::Mine(v)) => json!({ "content": [{ "type": "text", "text": serde_json::to_string(&v).unwrap_or_default() }], "structuredContent": v, "isError": false }),
                Err(e) => json!({ "content": [{ "type": "text", "text": e }], "isError": true }),
            };
            let redacted = if gateway.settings.redact_off { 0 } else { redact::value(&mut answer) };
            let (app, tool) = match name.split_once(EXPOSED_SEPARATOR) {
                Some((id, t)) => (Some(id.to_string()), Some(t.to_string())),
                None => (args.get("app").and_then(Value::as_str).map(str::to_string), args.get("tool").and_then(Value::as_str).map(str::to_string)),
            };
            let ok = !answer.get("isError").and_then(Value::as_bool).unwrap_or(false);
            let ms = started.elapsed().as_millis() as u64;
            log::record("call", json!({ "name": name, "app": app, "tool": tool, "ok": ok, "ms": ms, "as": gateway.client, "transport": gateway.transport, "redacted": (redacted > 0).then_some(redacted) }));
            let mut attributes = vec![("formmcp.tool", name.to_string()), ("formmcp.client", gateway.client.clone()), ("formmcp.transport", gateway.transport.to_string())];
            attributes.extend(app.map(|a| ("formmcp.app", a)));
            attributes.extend(tool.map(|t| ("formmcp.app_tool", t)));
            telemetry::export(gateway.settings.telemetry.as_deref(), telemetry::Span { name: format!("tools/call {name}"), start: wall, ms, ok, error: (!ok).then_some("tool error"), attributes });
            answer
        }
        "resources/list" => json!({ "resources": [] }),
        "prompts/list" => json!({ "prompts": [] }),
        _ => return Some(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("method {method:?} is not served by formMCP") } })),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

pub fn serve(client: Option<&str>) -> io::Result<()> {
    let mut gateway = Gateway::new(Family::bundled().map_err(io::Error::other)?, Places::from_env());
    if let Some(name) = client {
        gateway.force_client(name);
    }
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    for raw in stdin.lock().split(b'\n') {
        let raw = raw?;
        let line = String::from_utf8_lossy(&raw);
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(Value::Array(batch)) => {
                let replies: Vec<Value> = batch.iter().filter_map(|m| answer(&mut gateway, m)).collect();
                if replies.is_empty() { None } else { Some(Value::Array(replies)) }
            }
            Ok(msg) => answer(&mut gateway, &msg),
            Err(_) => Some(json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "Parse error" } })),
        };
        if let Some(reply) = reply {
            writeln!(out, "{reply}")?;
            out.flush()?;
        }
        if gateway.take_list_changed() {
            writeln!(out, "{}", json!({ "jsonrpc": "2.0", "method": "notifications/tools/list_changed" }))?;
            out.flush()?;
        }
    }
    telemetry::flush(Duration::from_secs(5));
    Ok(())
}
