use formmcp_core::clients::{self, Client, Launcher, CLIENTS};
use formmcp_core::settings::Settings;
use formmcp_core::{catalog, log, mcp, Family, Places};
use serde_json::{json, Value};

fn snapshot_now() -> Result<Value, String> {
    let family = catalog::load(Family::bundled()?);
    let places = Places::from_env();
    let settings = Settings::load();
    let apps: Vec<Value> = family
        .apps
        .iter()
        .map(|a| {
            let reach = places.reach(a);
            json!({
                "id": a.id, "name": a.name, "kind": a.kind, "summary": a.summary, "mcp": a.mcp,
                "state": a.state, "version": a.version, "installed": reach.installed,
                "reach": reach.route.as_ref().map(|r| r.label()), "exposed": settings.expose.contains(&a.id)
            })
        })
        .collect();
    let clients: Vec<Value> = CLIENTS.iter().map(|c| json!(clients::state(*c))).collect();
    let formportal = places.portal_program.is_some() || places.portal_root.join("installed.json").is_file();
    Ok(json!({ "source": family.describe_source(), "apps": apps, "clients": clients, "formportal": formportal, "env_expose": std::env::var("FORMMCP_EXPOSE").is_ok() }))
}

#[tauri::command]
async fn snapshot() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(snapshot_now).await.map_err(|e| e.to_string())?
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
        .invoke_handler(tauri::generate_handler![snapshot, set_expose, connect, log_tail])
        .run(tauri::generate_context!())
        .expect("formMCP could not start its window");
}
