# formMCP

formMCP is one MCP server for the whole Form family. Add it to Claude Code, Codex or Cursor once, and the agent can see every Form app, with one line on what each is for, and whether it has an MCP server, whether or not any app is installed.

**Current version:** v0.7.3 · **Stack:** Rust (`formmcp-core`, `formmcp-cli`) · Tauri 2 shell · vanilla TypeScript and Vite window · **Team:** one owner (nodemode-ai)

| Page | What it covers |
| --- | --- |
| [Getting Started](Getting-Started) | install from source, run the window, the command line and the MCP server |
| [Architecture](Architecture) | the engine, the family list and where it comes from, the shell, the window |
| [Agents](Agents) | connecting Claude Code, Codex and Cursor, and the `family_apps` tool |

## What it does

- `family_apps`: the Form family from formcompiler's registry, generated into the build, never edited by hand, with each app's install state and how formMCP can reach it on this computer (v0.2.0).
- `app_tools` and `call_app`: one app's own tools, loaded only when asked for, and its results passed through unchanged (v0.2.0).
- `suggest_app`: the app that fits a task in the person's words, with the next step; the signed family catalogue as the list's first source; an app's tools always shown when the person chooses; `agents install` for Claude Code, Codex and Cursor (v0.3.0).
- A window listing the same apps, filterable by MCP server; from v0.4.0 it also connects formMCP to Claude Code, Codex and Cursor, shows each app's install state and route, holds the always-show switches, and shows the activity (sessions, tool calls, app starts).
- No skills: from v0.4.1 the Form apps' skills are formskills' job (view, download, personal edits), and formMCP says so to agents.
- `formmcp mcp` on stdio, from a checkout (`./formmcp mcp`) or from the installed app with no window; or HTTP on 127.0.0.1:5625 with an access token, while the window or `formmcp serve` runs.
- Apps turned on or off per agent, single tools turned off, and each app's resources and prompts passed through.
- Safety (v0.7.0): secrets in the system keychain, secrets hidden in results and the log, an installed app's files checked before it starts, before and after hooks, and OpenTelemetry tracing.

## What comes next

formMCP is in formcompiler's registry (v0.4.2) under the same id and name as in FormPortal, FormTray and FormRanker. v0.5.0 adds HTTP, apps per agent, tool switches, and apps' resources and prompts; v0.6.0 profiles and a start-up policy per app; v0.7.0 safety and tracing. The parity plan's formMCP steps are done; what follows comes from the family plan. formMCP stays with the Form family only (owner). The owner's first push, `compile_v*` branch and the family's catalogue key pair are still to come.
