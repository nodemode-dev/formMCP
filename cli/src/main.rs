use std::process::ExitCode;

use formmcp_core::clients::{self, Client, Launcher, CLIENTS};
use formmcp_core::settings::{self, Settings};
use formmcp_core::{catalog, mcp, suggest, Family, Places, CORE_VERSION};
use serde_json::json;

const HELP: &str = "formmcp-cli: formMCP without the window

  apps [--mcp-only]     the Form family: id, name, kind, summary, MCP server, skills, state (JSON)
  app <id>              one app (JSON)
  reach                 per app: installed by FormPortal, and how formMCP would start its server here (JSON)
  suggest <task...>     the apps that fit a task, best first, with the next step (JSON)
  catalog               where the family list comes from now: the signed catalogue, or the bundled list and why
  expose <id> on|off    always list that app's own tools in formMCP's tool list (saved in settings.json)
  agents [status|install|remove] [claude,codex,cursor]
                        add formMCP to Claude Code, Codex and Cursor for this user account, or take it out
  mcp                   serve formMCP's MCP tools on stdin and stdout

  FORMMCP_DEV_ROOTS     folders holding Form app checkouts (path list); a checkout's own launcher is used first
  FORMPORTAL_ROOT       FormPortal's folder, when it is not in the usual place
  FORMPORTAL_BIN        the FormPortal program that runs `app-mcp`, when it is not on PATH
  FORMMCP_START_TIMEOUT seconds an app's server has to start (600)
  FORMMCP_CALL_TIMEOUT  seconds one tool call may take (900)
  FORMMCP_CATALOG       the catalogue (https, a loopback http address, or a file); the family catalogue by default
  FORMMCP_OFFLINE       1 reads only FormPortal's kept copy of the catalogue
  FORMAI_CATALOG_PUBKEY extra trusted catalogue keys, comma separated
  FORMMCP_EXPOSE        app ids to always list, comma separated (overrides settings.json)
  FORMMCP_HOME          where settings.json lives
  version               print the version
  help                  this text
";

fn print(value: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");
    if command == "mcp" {
        return match mcp::serve() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("formmcp: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if matches!(command, "version" | "--version" | "-V") {
        println!("formMCP {CORE_VERSION}");
        return ExitCode::SUCCESS;
    }
    if matches!(command, "help" | "--help" | "-h") {
        print!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let family = match Family::bundled() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("formmcp: {e}");
            return ExitCode::FAILURE;
        }
    };
    let family = if matches!(command, "apps" | "app" | "suggest" | "catalog") { catalog::load(family) } else { family };
    match command {
        "apps" => {
            let mcp_only = args.iter().any(|a| a == "--mcp-only");
            let apps: Vec<_> = family.apps.iter().filter(|a| !mcp_only || a.mcp).collect();
            print(&json!({ "source": family.describe_source(), "count": apps.len(), "apps": apps }));
            ExitCode::SUCCESS
        }
        "suggest" => {
            let task = args[1..].join(" ");
            if task.trim().is_empty() {
                eprintln!("formmcp: suggest needs a task, for example: suggest \"remove the background from a photo\"");
                return ExitCode::FAILURE;
            }
            let places = Places::from_env();
            print(&json!({ "source": family.describe_source(), "apps": suggest::suggest(&family, &places, &|_| None, &task, 3) }));
            ExitCode::SUCCESS
        }
        "catalog" => {
            print(&json!({ "source": family.describe_source(), "apps": family.apps.len(), "trusted_keys": catalog::keys().len() }));
            ExitCode::SUCCESS
        }
        "expose" => {
            let (Some(id), Some(onoff)) = (args.get(1), args.get(2)) else {
                eprintln!("formmcp: expose <id> on|off");
                return ExitCode::FAILURE;
            };
            let Some(app) = family.get(id) else {
                eprintln!("formmcp: no Form app {id:?}; apps lists them");
                return ExitCode::FAILURE;
            };
            let on = matches!(onoff.as_str(), "on" | "true" | "yes");
            let mut saved: Settings = std::fs::read_to_string(settings::path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
            saved.set_exposed(&app.id, on);
            match saved.save() {
                Ok(path) => {
                    print(&json!({ "expose": saved.expose, "settings": path, "note": "clients see the change the next time they start formMCP" }));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("formmcp: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "agents" => {
            let action = args.get(1).map(String::as_str).unwrap_or("status");
            let chosen: Vec<Client> = match args.get(2) {
                Some(list) => match list.split(',').map(|c| Client::parse(c).ok_or_else(|| c.to_string())).collect::<Result<Vec<_>, _>>() {
                    Ok(c) => c,
                    Err(bad) => {
                        eprintln!("formmcp: there is no client {bad:?}; use claude, codex or cursor");
                        return ExitCode::FAILURE;
                    }
                },
                None => CLIENTS.to_vec(),
            };
            match action {
                "status" => {
                    print(&json!({ "clients": chosen.iter().map(|c| clients::state(*c)).collect::<Vec<_>>() }));
                    ExitCode::SUCCESS
                }
                "install" | "remove" => {
                    let launcher = match Launcher::current() {
                        Ok(l) => l,
                        Err(e) => {
                            eprintln!("formmcp: {e}");
                            return ExitCode::FAILURE;
                        }
                    };
                    let mut failed = false;
                    let rows: Vec<_> = clients::connect(&chosen, &launcher, action == "install")
                        .into_iter()
                        .map(|r| match r {
                            Ok(state) => json!(state),
                            Err((client, e)) => {
                                failed = true;
                                json!({ "client": client, "error": e })
                            }
                        })
                        .collect();
                    print(&json!({ "program": launcher.program, "args": launcher.args, "clients": rows }));
                    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
                }
                other => {
                    eprintln!("formmcp: agents {other:?}: use status, install or remove");
                    ExitCode::FAILURE
                }
            }
        }
        "reach" => {
            let places = Places::from_env();
            let rows: Vec<_> = family.apps.iter().map(|a| json!({ "id": a.id, "mcp": a.mcp, "reach": places.reach(a) })).collect();
            print(&json!({
                "dev_roots": places.dev_roots,
                "formportal_root": places.portal_root,
                "formportal_program": places.portal_program,
                "apps": rows
            }));
            ExitCode::SUCCESS
        }
        "app" => match args.get(1).and_then(|id| family.get(id)) {
            Some(app) => {
                print(&json!(app));
                ExitCode::SUCCESS
            }
            None => {
                eprintln!("formmcp: no Form app {:?}; apps lists them", args.get(1).cloned().unwrap_or_default());
                ExitCode::FAILURE
            }
        },
        other => {
            eprintln!("formmcp: no command {other:?}\n\n{HELP}");
            ExitCode::FAILURE
        }
    }
}
