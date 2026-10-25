use std::sync::Mutex;

use formmcp_core::clients::{self, Client, Launcher, CLIENTS};
use formmcp_core::settings::{self, Settings, CLIENT_KEYS};
use formmcp_core::{catalog, http, log, mcp, toolcache, Family, Places};
use serde_json::{json, Value};

#[derive(Default)]
struct Http(Mutex<Option<http::Server>>);

fn http_state_of(slot: &Http) -> Value {
    let port = Settings::load().port();
    let running = slot.0.lock().ok().and_then(|s| s.as_ref().map(|server| server.port));
    json!({ "running": running.is_some(), "port": running.unwrap_or(port), "address": http::address(running.unwrap_or(port)) })
}

fn snapshot_now() -> Result<Value, String> {
    let family = catalog::load(Family::bundled()?);
    let places = Places::from_env();
    let settings = Settings::load();
    let apps: Vec<Value> = family
        .apps
        .iter()
        .map(|a| {
            let reach = places.reach(a);
            let off_for: Vec<&str> = CLIENT_KEYS.iter().copied().filter(|c| settings.off.get(*c).is_some_and(|l| l.contains(&a.id))).collect();
            let profile_off_for: Vec<&str> = CLIENT_KEYS.iter().copied().filter(|c| settings.profile_hides_app(c, &a.id)).collect();
            let start = settings.start.get(&a.id).cloned().unwrap_or_else(|| "demand".to_string());
            json!({
                "id": a.id, "name": a.name, "kind": a.kind, "summary": a.summary, "mcp": a.mcp,
                "state": a.state, "version": a.version, "installed": reach.installed,
                "reach": reach.route.as_ref().map(|r| r.label()), "exposed": settings.expose.contains(&a.id),
                "off_for": off_for, "profile_off_for": profile_off_for, "start": start, "hidden_tools": settings.hidden_tools.get(&a.id).map(Vec::len).unwrap_or(0)
            })
        })
        .collect();
    let clients: Vec<Value> = CLIENTS.iter().map(|c| json!(clients::state(*c))).collect();
    let formportal = places.portal_program.is_some() || places.portal_root.join("installed.json").is_file();
    Ok(json!({ "source": family.describe_source(), "apps": apps, "clients": clients, "formportal": formportal, "env_expose": std::env::var("FORMMCP_EXPOSE").is_ok(), "profiles": profiles_of(&settings) }))
}

fn profiles_of(settings: &Settings) -> Value {
    let names: Vec<Value> = settings
        .profiles
        .iter()
        .map(|(name, p)| {
            let used_by: Vec<&String> = settings.use_profile.iter().filter(|(_, n)| *n == name).map(|(c, _)| c).collect();
            json!({ "name": name, "apps_off": p.apps_off.len(), "tools_off": p.hidden_tools.values().map(Vec::len).sum::<usize>(), "starts": p.start.len(), "used_by": used_by })
        })
        .collect();
    json!({ "list": names, "in_use": settings.use_profile })
}

#[tauri::command]
fn set_start(id: String, policy: String) -> Result<String, String> {
    let mut saved = Settings::saved();
    saved.set_start(&id, &policy)?;
    saved.save()?;
    Ok(saved.start.get(&id).cloned().unwrap_or_else(|| "demand".to_string()))
}

#[tauri::command]
fn profile_use(client: String, name: Option<String>) -> Result<Value, String> {
    if !CLIENT_KEYS.contains(&client.as_str()) {
        return Err(format!("there is no agent {client:?}"));
    }
    let mut saved = Settings::saved();
    saved.set_profile_for(&client, name.as_deref())?;
    saved.save()?;
    Ok(profiles_of(&saved))
}

#[tauri::command]
fn profile_save(name: String, from: String) -> Result<Value, String> {
    let name = name.trim().to_string();
    if !settings::profile_name_ok(&name) {
        return Err("a profile name has 1 to 40 letters, digits, spaces, hyphens or underscores".into());
    }
    if !CLIENT_KEYS.contains(&from.as_str()) {
        return Err(format!("there is no agent {from:?}"));
    }
    let mut saved = Settings::saved();
    let profile = saved.profile_from(&from);
    saved.profiles.insert(name, profile);
    saved.save()?;
    Ok(profiles_of(&saved))
}

#[tauri::command]
fn profile_delete(name: String) -> Result<Value, String> {
    let mut saved = Settings::saved();
    saved.delete_profile(&name)?;
    saved.save()?;
    Ok(profiles_of(&saved))
}

#[tauri::command]
fn profile_export(name: String) -> Result<String, String> {
    let saved = Settings::saved();
    let profile = saved.profiles.get(&name).ok_or_else(|| format!("there is no profile {name:?}"))?;
    settings::export_profile(&name, profile, None).map(|p| p.display().to_string())
}

#[tauri::command]
fn profile_import(text: String) -> Result<Value, String> {
    let family = Family::bundled()?;
    let (name, profile, dropped) = settings::read_profile(&text, &|id| family.get(id).is_some())?;
    let mut saved = Settings::saved();
    let replaced = saved.profiles.insert(name.clone(), profile).is_some();
    saved.save()?;
    Ok(json!({ "name": name, "replaced": replaced, "dropped": dropped, "profiles": profiles_of(&saved) }))
}

#[tauri::command]
async fn snapshot(slot: tauri::State<'_, Http>) -> Result<Value, String> {
    let http = http_state_of(&slot);
    let mut found = tauri::async_runtime::spawn_blocking(snapshot_now).await.map_err(|e| e.to_string())??;
    found["http"] = http;
    Ok(found)
}

#[tauri::command]
fn set_for(client: String, id: String, on: bool) -> Result<bool, String> {
    if !CLIENT_KEYS.contains(&client.as_str()) {
        return Err(format!("there is no agent {client:?}"));
    }
    let mut saved = Settings::saved();
    saved.set_app_for(&client, &id, on);
    saved.save()?;
    Ok(!saved.is_off(&client, &id))
}

fn tool_rows(id: &str) -> Value {
    let saved = Settings::saved();
    match toolcache::known(id) {
        Some(known) => {
            let rows: Vec<Value> = known.tools.iter().map(|t| json!({ "name": t.name, "description": t.description, "read_only": t.read_only, "on": !saved.tool_hidden(id, &t.name) })).collect();
            json!({ "app": id, "loaded": known.loaded, "server": known.server, "tools": rows })
        }
        None => json!({ "app": id, "loaded": null, "tools": [] }),
    }
}

#[tauri::command]
fn tool_list(id: String) -> Value {
    tool_rows(&id)
}

#[tauri::command]
async fn tool_load(id: String) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut gateway = mcp::Gateway::new(Family::bundled()?, Places::from_env());
        gateway.start_app(&id)?;
        Ok(tool_rows(&id))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn set_tool(id: String, tool: String, on: bool) -> Result<bool, String> {
    let mut saved = Settings::saved();
    saved.set_tool(&id, &tool, on);
    saved.save()?;
    Ok(!saved.tool_hidden(&id, &tool))
}

#[tauri::command]
fn http_switch(on: bool, slot: tauri::State<'_, Http>) -> Result<Value, String> {
    {
        let mut held = slot.0.lock().map_err(|_| "the HTTP server is busy".to_string())?;
        if on && held.is_none() {
            *held = Some(http::start(Settings::load().port())?);
        } else if !on {
            if let Some(server) = held.take() {
                server.stop();
            }
        }
    }
    Ok(http_state_of(&slot))
}

#[tauri::command]
fn http_token() -> Result<String, String> {
    settings::token()
}

#[tauri::command]
fn http_rotate() -> Result<String, String> {
    settings::rotate_token()
}

#[tauri::command]
fn set_expose(id: String, on: bool) -> Result<Vec<String>, String> {
    let mut saved = Settings::saved();
    saved.set_exposed(&id, on);
    saved.save()?;
    Ok(saved.expose)
}

#[tauri::command]
async fn connect(client: String, on: bool) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let which = Client::parse(&client).ok_or_else(|| format!("there is no client {client:?}"))?;
        let launcher = Launcher::current()?;
        match clients::connect(&[which], &launcher, on).into_iter().next() {
            Some(Ok(state)) => Ok(json!(state)),
            Some(Err((_, e))) => Err(e),
            None => Err("nothing was changed".to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn log_tail(from: u64) -> Value {
    let (events, next) = log::tail(from);
    json!({ "events": events, "next": next })
}

pub fn run() {
    if std::env::args().nth(1).as_deref() == Some("mcp") {
        let args: Vec<String> = std::env::args().collect();
        let client = args.iter().position(|a| a == "--client").and_then(|i| args.get(i + 1)).cloned();
        let code = match mcp::serve(client.as_deref()) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("formmcp: {e}");
                1
            }
        };
        std::process::exit(code);
    }
    tauri::Builder::default()
        .manage(Http::default())
        .invoke_handler(tauri::generate_handler![snapshot, set_expose, set_for, tool_list, tool_load, set_tool, http_switch, http_token, http_rotate, connect, log_tail, set_start, profile_use, profile_save, profile_delete, profile_export, profile_import])
        .run(tauri::generate_context!())
        .expect("formMCP could not start its window");
}
