# Agents

formMCP's MCP server is `formmcp mcp` (stdio). In this repository it is already wired:

| Client | In this repository |
| --- | --- |
| Claude Code | `.mcp.json` |
| Codex | `.codex/config.toml` |
| Cursor | `.cursor/mcp.json` |

All three run `./formmcp mcp`, which builds the command line on first use. An installed formMCP answers the same way: `<the formMCP program> mcp`.

## Tools

| Tool | Changes anything | Returns |
| --- | --- | --- |
| `family_apps` | no | every Form app (`id`, `name`, `kind`, `summary`, `mcp`, `skills`, `state`, `installed`, `reach`) and where the list came from; `mcp_only` keeps the apps with an MCP server |
| `app_tools` | no | one app's own tools with their input schemas, its server name and version, and its instructions; starts its server the first time |
| `call_app` | what the app's tool changes | the app's own result for one of its tools, unchanged |

The list is short on purpose: one line per app and no app's tools. An app's tools enter the conversation only when `app_tools` is called for it, so formMCP stays small however many apps are installed. The server's instructions tell the agent that formMCP never installs and that FormPortal installs and updates the Form apps.

## How an app's server is started

`reach` in `family_apps`, and `./formmcp cli reach`, show the route for each app:

| Route | When | Started as |
| --- | --- | --- |
| `dev` | a checkout of the app (`<folder>/<app>` holding its launcher) is in a folder listed in `FORMMCP_DEV_ROOTS` | `<checkout>/<launcher> mcp` (the app's own bridge, which starts or attaches to its engine) |
| `formportal` | FormPortal installed the app and its program is found | `<FormPortal program> app-mcp <id>` (always the installed version) |

`./formmcp` sets `FORMMCP_DEV_ROOTS` to the folder that holds the formMCP checkout when it is not set, so Form app checkouts beside it are found; set it empty to turn that off. FormPortal's program is `FORMPORTAL_BIN`, else `formportal-cli` or `formportal` on PATH, else `/Applications/FormPortal.app` on macOS. FormPortal starts an install only when the install's `form-app.json` names a launcher (`agents.mcp.command`); until the apps' bundles carry one, installed apps are reached through their checkouts.

An app's server inherits formMCP's environment, so what an app needs (form.md's `FORMMD_MCP_TOKEN`) is set once in the client's config for formmcp. `FORMMCP_START_TIMEOUT` (600 s) and `FORMMCP_CALL_TIMEOUT` (900 s) bound the waits. A started server lives as long as the client's formMCP session and ends with it.

The `formmcp-harness` skill (`.claude/skills/formmcp-harness`) teaches the workflow.
