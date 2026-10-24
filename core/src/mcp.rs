use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use crate::catalog;
use crate::child::{call_timeout, AppServer};
use crate::log;
use crate::family::{App, Family};
use crate::reach::Places;
use crate::settings::Settings;
use crate::suggest::suggest;
use crate::CORE_VERSION;

const EXPOSED_SEPARATOR: &str = "__";

pub struct Gateway {
    pub family: Family,
    pub places: Places,
    pub settings: Settings,
    running: HashMap<String, (String, AppServer)>,
    catalog_read: bool,
    list_changed: bool,
}

impl Gateway {
    pub fn new(family: Family, places: Places) -> Gateway {
        Gateway { family, places, settings: Settings::load(), running: HashMap::new(), catalog_read: false, list_changed: false }
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
        self.family.get(name).cloned().ok_or_else(|| format!("there is no Form app {name:?}; family_apps lists them"))
    }

    fn server(&mut self, app: &App) -> Result<&mut (String, AppServer), String> {
        let stale = self.running.get_mut(&app.id).map(|(_, s)| !s.alive()).unwrap_or(false);
        if stale {
            self.running.remove(&app.id);
        }
        if !self.running.contains_key(&app.id) {
            if !app.mcp {
                return Err(format!("{} has no MCP server yet, so there are no tools to reach", app.name));
            }
            self.places.refresh();
            let reach = self.places.reach(app);
            let Some(route) = reach.route else {
                let how = if reach.installed {
                    "it is installed, but no FormPortal program was found to start it (set FORMPORTAL_BIN)"
                } else {
                    "it is not installed: FormPortal installs it, or set FORMMCP_DEV_ROOTS to a folder holding its checkout"
                };
                return Err(format!("{} cannot be reached: {how}", app.name));
            };
            let (program, args, dir) = route.command();
            let started = std::time::Instant::now();
            let server = match AppServer::start(&program, &args, dir.as_ref()) {
                Ok(s) => s,
                Err(e) => {
                    let short: String = e.lines().next().unwrap_or_default().chars().take(200).collect();
                    log::record("app-start", json!({ "app": app.id, "via": route.label(), "ok": false, "error": short, "ms": started.elapsed().as_millis() as u64 }));
                    return Err(format!("{} did not start through {}: {e}", app.name, route.label()));
                }
            };
            log::record("app-start", json!({ "app": app.id, "via": route.label(), "ok": true, "tools": server.tools.len(), "ms": started.elapsed().as_millis() as u64 }));
            self.running.insert(app.id.clone(), (route.label().to_string(), server));
        }
        self.running.get_mut(&app.id).ok_or_else(|| "the app's server is gone".to_string())
    }
}

const PROTOCOL: &str = "2025-06-18";
const SUPPORTED: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "formMCP is the one connection to the Form family of desktop apps (image, video, audio, layout, vector, notes and more, by nodemode.ai). Call suggest_app with the person's task to get the app that fits and the next step, or family_apps for the whole list (whether each app has an MCP server and whether this computer can reach it). For a reachable app, app_tools starts it and lists its own tools, and call_app runs one of them; load one app at a time, only when the task needs it. formMCP never installs anything: when an app is missing, tell the person which one fits and that FormPortal installs and updates the Form apps.";

fn tools() -> Value {
    json!([
        { "name": "family_apps", "title": "The Form family",
          "description": "Every Form app: id, name, what kind of work it does, one line on what it is for, whether it has an MCP server agents can use, whether it ships skills, its release state (live, public or prelaunch), whether FormPortal installed it on this computer, and reach: how formMCP can start its server here (dev for a local checkout, formportal for an install, null when it cannot). Use it to pick the app that fits a task. Short on purpose: no app's tools are listed here.",
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
          "annotations": { "readOnlyHint": false, "destructiveHint": true, "openWorldHint": true } }
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
    let picks = suggest(&gateway.family, &gateway.places, &|id: &str| running.get(id).cloned(), &task, limit);
    let note = if picks.is_empty() { "No app matched the words of the task; call family_apps and choose by kind and summary." } else { "Best first. formMCP never installs: FormPortal installs and updates the Form apps." };
    Ok(json!({ "task": task, "apps": picks, "note": note }))
}

fn family_apps(gateway: &mut Gateway, args: &Value) -> Value {
    let mcp_only = args.get("mcp_only").and_then(Value::as_bool).unwrap_or(false);
    gateway.places.refresh();
    let Gateway { family, places, running, .. } = gateway;
    let apps: Vec<Value> = family
        .apps
        .iter()
        .filter(|a| !mcp_only || a.mcp)
        .map(|a| {
            let reach = places.reach(a);
            let via = running.get(&a.id).map(|(v, _)| v.clone()).or_else(|| reach.route.as_ref().map(|r| r.label().to_string()));
            json!({ "id": a.id, "name": a.name, "kind": a.kind, "summary": a.summary, "mcp": a.mcp, "skills": a.skills, "state": a.state, "installed": reach.installed, "reach": via })
        })
        .collect();
    json!({
        "count": apps.len(),
        "apps": apps,
        "source": family.describe_source(),
        "install": "formMCP never installs. FormPortal (id formai) installs, updates and removes the Form apps."
    })
}

fn app_tools(gateway: &mut Gateway, args: &Value) -> Result<Value, String> {
    let app = gateway.app(args)?;
    let refresh = args.get("refresh").and_then(Value::as_bool).unwrap_or(false);
    let (via, server) = gateway.server(&app)?;
    if refresh || server.tools_changed {
        server.load_tools()?;
    }
    Ok(json!({
        "app": app.id,
        "via": via,
        "server": server.server,
        "instructions": server.instructions,
        "count": server.tools.len(),
        "tools": server.tools
    }))
}

fn call_app(gateway: &mut Gateway, args: &Value) -> Result<Value, String> {
    let app = gateway.app(args)?;
    let tool = args.get("tool").and_then(Value::as_str).unwrap_or_default().to_string();
    if tool.is_empty() {
        return Err("tool is required: a name from app_tools".into());
    }
    let arguments = args.get("arguments").cloned().unwrap_or_else(|| json!({}));
    let exposed = gateway.settings.expose.iter().any(|e| e == &app.id);
    let (_, server) = gateway.server(&app)?;
    let result = server.request("tools/call", json!({ "name": tool, "arguments": arguments }), call_timeout());
    if exposed && server.tools_changed {
        gateway.list_changed = true;
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
        other => match other.split_once(EXPOSED_SEPARATOR) {
            Some((id, tool)) if gateway.settings.expose.iter().any(|e| e == id) => {
                call_app(gateway, &json!({ "app": id, "tool": tool, "arguments": args })).map(Outcome::Passed)
            }
            _ => Err(format!("no tool named {other:?}; tools/list has the tools")),
        },
    }
}

pub fn answer(gateway: &mut Gateway, msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned()?;
    let method = msg.get("method").and_then(Value::as_str).unwrap_or_default();
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => {
            let asked = params.get("protocolVersion").and_then(Value::as_str).unwrap_or(PROTOCOL);
            let version = if SUPPORTED.contains(&asked) { asked } else { PROTOCOL };
            let client = params.get("clientInfo").cloned().unwrap_or(Value::Null);
            log::record("session", json!({ "client": client.get("name"), "client_version": client.get("version"), "expose": gateway.settings.expose }));
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": { "listChanged": !gateway.settings.expose.is_empty() } },
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
            let answer = match call(gateway, name, &args) {
                Ok(Outcome::Passed(result)) => result,
                Ok(Outcome::Mine(v)) => json!({ "content": [{ "type": "text", "text": serde_json::to_string(&v).unwrap_or_default() }], "structuredContent": v, "isError": false }),
                Err(e) => json!({ "content": [{ "type": "text", "text": e }], "isError": true }),
            };
            let (app, tool) = match name.split_once(EXPOSED_SEPARATOR) {
                Some((id, t)) => (Some(id.to_string()), Some(t.to_string())),
                None => (args.get("app").and_then(Value::as_str).map(str::to_string), args.get("tool").and_then(Value::as_str).map(str::to_string)),
            };
            let ok = !answer.get("isError").and_then(Value::as_bool).unwrap_or(false);
            log::record("call", json!({ "name": name, "app": app, "tool": tool, "ok": ok, "ms": started.elapsed().as_millis() as u64 }));
            answer
        }
        "resources/list" => json!({ "resources": [] }),
        "prompts/list" => json!({ "prompts": [] }),
        _ => return Some(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("method {method:?} is not served by formMCP") } })),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

pub fn serve() -> io::Result<()> {
    let mut gateway = Gateway::new(Family::bundled().map_err(io::Error::other)?, Places::from_env());
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
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
    Ok(())
}
