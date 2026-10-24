use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use crate::child::{call_timeout, AppServer};
use crate::family::{App, Family};
use crate::reach::Places;
use crate::CORE_VERSION;

pub struct Gateway {
    pub family: Family,
    pub places: Places,
    running: HashMap<String, (String, AppServer)>,
}

impl Gateway {
    pub fn new(family: Family, places: Places) -> Gateway {
        Gateway { family, places, running: HashMap::new() }
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
            let server = AppServer::start(&program, &args, dir.as_ref()).map_err(|e| format!("{} did not start through {}: {e}", app.name, route.label()))?;
            self.running.insert(app.id.clone(), (route.label().to_string(), server));
        }
        self.running.get_mut(&app.id).ok_or_else(|| "the app's server is gone".to_string())
    }
}

const PROTOCOL: &str = "2025-06-18";
const SUPPORTED: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "formMCP is the one connection to the Form family of desktop apps (image, video, audio, layout, vector, notes and more, by nodemode.ai). Call family_apps to see which app fits the person's task, whether it has an MCP server and whether this computer can reach it. For a reachable app, app_tools starts it and lists its own tools, and call_app runs one of them; load one app at a time, only when the task needs it. formMCP never installs anything: when an app is missing, tell the person which one fits and that FormPortal installs and updates the Form apps.";

fn tools() -> Value {
    json!([
        { "name": "family_apps", "title": "The Form family",
          "description": "Every Form app: id, name, what kind of work it does, one line on what it is for, whether it has an MCP server agents can use, whether it ships skills, its release state (live, public or prelaunch), whether FormPortal installed it on this computer, and reach: how formMCP can start its server here (dev for a local checkout, formportal for an install, null when it cannot). Use it to pick the app that fits a task. Short on purpose: no app's tools are listed here.",
          "inputSchema": { "type": "object", "properties": {
              "mcp_only": { "type": "boolean", "description": "Only the apps that have an MCP server." } } },
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

fn family_apps(gateway: &mut Gateway, args: &Value) -> Value {
    let mcp_only = args.get("mcp_only").and_then(Value::as_bool).unwrap_or(false);
    gateway.places.refresh();
    let Gateway { family, places, running } = gateway;
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
    let (_, server) = gateway.server(&app)?;
    server.request("tools/call", json!({ "name": tool, "arguments": arguments }), call_timeout())
}

enum Outcome {
    Mine(Value),
    Passed(Value),
}

fn call(gateway: &mut Gateway, name: &str, args: &Value) -> Result<Outcome, String> {
    match name {
        "family_apps" => Ok(Outcome::Mine(family_apps(gateway, args))),
        "app_tools" => app_tools(gateway, args).map(Outcome::Mine),
        "call_app" => call_app(gateway, args).map(Outcome::Passed),
        other => Err(format!("no tool named {other:?}; tools/list has the tools")),
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
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "formmcp", "title": "formMCP", "version": CORE_VERSION },
                "instructions": INSTRUCTIONS
            })
        }
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools() }),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            match call(gateway, name, &args) {
                Ok(Outcome::Passed(result)) => result,
                Ok(Outcome::Mine(v)) => json!({ "content": [{ "type": "text", "text": serde_json::to_string(&v).unwrap_or_default() }], "structuredContent": v, "isError": false }),
                Err(e) => json!({ "content": [{ "type": "text", "text": e }], "isError": true }),
            }
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
    }
    Ok(())
}
