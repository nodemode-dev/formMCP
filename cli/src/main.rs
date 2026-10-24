use std::process::ExitCode;

use formmcp_core::{mcp, Family, Places, CORE_VERSION};
use serde_json::json;

const HELP: &str = "formmcp-cli: formMCP without the window

  apps [--mcp-only]     the Form family: id, name, kind, summary, MCP server, skills, state (JSON)
  app <id>              one app (JSON)
  reach                 per app: installed by FormPortal, and how formMCP would start its server here (JSON)
  mcp                   serve formMCP's MCP tools on stdin and stdout

  FORMMCP_DEV_ROOTS     folders holding Form app checkouts (path list); a checkout's own launcher is used first
  FORMPORTAL_ROOT       FormPortal's folder, when it is not in the usual place
  FORMPORTAL_BIN        the FormPortal program that runs `app-mcp`, when it is not on PATH
  FORMMCP_START_TIMEOUT seconds an app's server has to start (600)
  FORMMCP_CALL_TIMEOUT  seconds one tool call may take (900)
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
    match command {
        "apps" => {
            let mcp_only = args.iter().any(|a| a == "--mcp-only");
            let apps: Vec<_> = family.apps.iter().filter(|a| !mcp_only || a.mcp).collect();
            print(&json!({ "source": family.describe_source(), "count": apps.len(), "apps": apps }));
            ExitCode::SUCCESS
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
