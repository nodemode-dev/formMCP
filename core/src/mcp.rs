use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use crate::family::Family;
use crate::CORE_VERSION;

const PROTOCOL: &str = "2025-06-18";
const SUPPORTED: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "formMCP is the one connection to the Form family of desktop apps (image, video, audio, layout, vector, notes and more, by nodemode.ai). Call family_apps to see which app fits the person's task and whether it has an MCP server. formMCP never installs anything: when an app is missing, tell the person which one fits and that FormPortal installs and updates the Form apps.";

fn tools() -> Value {
    json!([
        { "name": "family_apps", "title": "The Form family",
          "description": "Every Form app: id, name, what kind of work it does, one line on what it is for, whether it has an MCP server agents can use, whether it ships skills, and its release state (live, public or prelaunch). Use it to pick the app that fits a task. Short on purpose: no app's tools are listed here.",
          "inputSchema": { "type": "object", "properties": {
              "mcp_only": { "type": "boolean", "description": "Only the apps that have an MCP server." } } },
          "annotations": { "readOnlyHint": true, "openWorldHint": false } }
    ])
}

fn family_apps(family: &Family, args: &Value) -> Value {
    let mcp_only = args.get("mcp_only").and_then(Value::as_bool).unwrap_or(false);
    let apps: Vec<Value> = family
        .apps
        .iter()
        .filter(|a| !mcp_only || a.mcp)
        .map(|a| json!({ "id": a.id, "name": a.name, "kind": a.kind, "summary": a.summary, "mcp": a.mcp, "skills": a.skills, "state": a.state }))
        .collect();
    json!({
        "count": apps.len(),
        "apps": apps,
        "source": family.describe_source(),
        "install": "formMCP never installs. FormPortal (id formai) installs, updates and removes the Form apps."
    })
}

fn call(family: &Family, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "family_apps" => Ok(family_apps(family, args)),
        other => Err(format!("no tool named {other:?}; tools/list has the tools")),
    }
}

pub fn answer(family: &Family, msg: &Value) -> Option<Value> {
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
            match call(family, name, &args) {
                Ok(v) => json!({ "content": [{ "type": "text", "text": serde_json::to_string(&v).unwrap_or_default() }], "structuredContent": v, "isError": false }),
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
    let family = Family::bundled().map_err(io::Error::other)?;
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(Value::Array(batch)) => {
                let replies: Vec<Value> = batch.iter().filter_map(|m| answer(&family, m)).collect();
                if replies.is_empty() { None } else { Some(Value::Array(replies)) }
            }
            Ok(msg) => answer(&family, &msg),
            Err(_) => Some(json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "Parse error" } })),
        };
        if let Some(reply) = reply {
            writeln!(out, "{reply}")?;
            out.flush()?;
        }
    }
    Ok(())
}
