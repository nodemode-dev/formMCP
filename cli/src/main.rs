use std::process::ExitCode;

use formmcp_core::clients::{self, Client, Launcher, CLIENTS};
use formmcp_core::settings::{self, Settings};
use formmcp_core::{catalog, http, mcp, suggest, toolcache, Family, Places, CORE_VERSION};
use serde_json::json;

const HELP: &str = "formmcp-cli: formMCP without the window

  apps [--mcp-only]     the Form family: id, name, kind, summary, MCP server, state (JSON)
  app <id>              one app (JSON)
  reach                 per app: installed by FormPortal, and how formMCP would start its server here (JSON)
  suggest <task...>     the apps that fit a task, best first, with the next step (JSON)
  catalog               where the family list comes from now: the signed catalogue, or the bundled list and why
  expose <id> on|off    always list that app's own tools in formMCP's tool list (saved in settings.json)
  agents [status|install|remove] [claude,codex,cursor]
                        add formMCP to Claude Code, Codex and Cursor for this user account, or take it out
  for <agent> <id> on|off   offer an app to one agent or not: claude, codex, cursor or other (saved in settings.json)
  tool <id> <tool> on|off   offer one of an app's tools or not, to every agent
  tools <id>            the app's tools formMCP last saw, and which are off (JSON)
  settings              what settings.json holds now, with the HTTP address (JSON)
  mcp [--client AGENT]  serve formMCP's MCP tools on stdin and stdout; --client names the agent (else its own name decides)
  serve [--port N]      serve formMCP over HTTP on 127.0.0.1 only, at http://127.0.0.1:5625/mcp, until stopped
  http token|rotate     print the HTTP access token, or make a new one (the old one stops working)

  FORMMCP_DEV_ROOTS     folders holding Form app checkouts (path list); a checkout's own launcher is used first
  FORMPORTAL_ROOT       FormPortal's folder, when it is not in the usual place
  FORMPORTAL_BIN        the FormPortal program that runs `app-mcp`, when it is not on PATH
  FORMMCP_START_TIMEOUT seconds an app's server has to start (600)
  FORMMCP_CALL_TIMEOUT  seconds one tool call may take (900)
  FORMMCP_CATALOG       the catalogue (https, a loopback http address, or a file); the family catalogue by default
  FORMMCP_OFFLINE       1 reads only FormPortal's kept copy of the catalogue
  FORMAI_CATALOG_PUBKEY extra trusted catalogue keys, comma separated
  FORMMCP_EXPOSE        app ids to always list, comma separated (overrides settings.json)
  FORMMCP_HOME          where settings.json, the log and the HTTP token live
  FORMMCP_HTTP_PORT     the HTTP port (5625, or settings.json's http_port)
  version               print the version
  help                  this text
";

fn print(value: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    if command == "mcp" {
        return match mcp::serve(flag("--client").as_deref()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("formmcp: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if command == "serve" {
        let port = match flag("--port") {
            Some(p) => match p.parse::<u16>() {
                Ok(p) => p,
                Err(_) => {
                    eprintln!("formmcp: --port needs a number, for example 5625");
                    return ExitCode::FAILURE;
                }
            },
            None => Settings::load().port(),
        };
        return match http::start(port) {
            Ok(server) => {
                eprintln!("formmcp: serving {} (127.0.0.1 only); send the header Authorization: Bearer <token> (`formmcp http token`). Ctrl+C stops.", http::address(server.port));
                server.wait();
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("formmcp: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if command == "http" {
        let result = match args.get(1).map(String::as_str) {
            Some("token") => settings::token(),
            Some("rotate") => settings::rotate_token(),
            _ => Err("http token or http rotate".to_string()),
        };
        return match result {
            Ok(token) => {
                println!("{token}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("formmcp: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if command == "settings" {
        let saved = Settings::saved();
        let port = Settings::load().port();
        print(&json!({ "file": settings::path(), "settings": saved, "http": { "address": http::address(port), "token_file": settings::token_path() } }));
        return ExitCode::SUCCESS;
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
        "for" | "tool" => {
            let words: Vec<&str> = args[1..].iter().map(String::as_str).collect();
            let (agent, id, tool, onoff) = match (command, words.as_slice()) {
                ("for", [agent, id, onoff]) => (Some(*agent), *id, None, *onoff),
                ("tool", [id, tool, onoff]) => (None, *id, Some(*tool), *onoff),
                _ => {
                    eprintln!("formmcp: for <agent> <id> on|off, or tool <id> <tool> on|off");
                    return ExitCode::FAILURE;
                }
            };
            let Some(app) = family.get(id) else {
                eprintln!("formmcp: no Form app {id:?}; apps lists them");
                return ExitCode::FAILURE;
            };
            if let Some(a) = agent {
                if !settings::CLIENT_KEYS.contains(&a) {
                    eprintln!("formmcp: there is no agent {a:?}; use claude, codex, cursor or other");
                    return ExitCode::FAILURE;
                }
            }
            let on = match onoff {
                "on" => true,
                "off" => false,
                other => {
                    eprintln!("formmcp: {other:?}: use on or off");
                    return ExitCode::FAILURE;
                }
            };
            let mut saved = Settings::saved();
            match (agent, tool) {
                (Some(a), _) => saved.set_app_for(a, &app.id, on),
                (_, Some(t)) => saved.set_tool(&app.id, t, on),
                _ => {}
            }
            match saved.save() {
                Ok(path) => {
                    print(&json!({ "off": saved.off, "hidden_tools": saved.hidden_tools, "settings": path, "note": "running formMCP servers use this from their next request" }));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("formmcp: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "tools" => {
            let Some(app) = args.get(1).and_then(|id| family.get(id)) else {
                eprintln!("formmcp: tools <id>; apps lists the ids");
                return ExitCode::FAILURE;
            };
            let saved = Settings::saved();
            match toolcache::known(&app.id) {
                Some(known) => {
                    let rows: Vec<_> = known.tools.iter().map(|t| json!({ "name": t.name, "on": !saved.tool_hidden(&app.id, &t.name), "read_only": t.read_only, "description": t.description })).collect();
                    print(&json!({ "app": app.id, "server": known.server, "loaded": known.loaded, "tools": rows }));
                }
                None => print(&json!({ "app": app.id, "tools": [], "note": "formMCP has not started this app yet; its tools are listed once an agent or the window loads them" })),
            }
            ExitCode::SUCCESS
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
