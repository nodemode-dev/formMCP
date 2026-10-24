use std::process::ExitCode;

use formmcp_core::{mcp, Family, CORE_VERSION};
use serde_json::json;

const HELP: &str = "formmcp-cli: formMCP without the window

  apps [--mcp-only]     the Form family: id, name, kind, summary, MCP server, skills, state (JSON)
  app <id>              one app (JSON)
  mcp                   serve formMCP's MCP tools on stdin and stdout
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
