# formMCP

formMCP is one MCP server for the whole Form family. Add it to Claude Code, Codex or Cursor once, and the agent can see every Form app, with one line on what each is for, and whether it has an MCP server, whether or not any app is installed.

**Current version:** v0.4.2 · **Stack:** Rust (`formmcp-core`, `formmcp-cli`) · Tauri 2 shell · vanilla TypeScript and Vite window · **Team:** one owner (nodemode-ai)

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
- `formmcp mcp` on stdio, from a checkout (`./formmcp mcp`) or from the installed app with no window.

## What comes next

formMCP is in formcompiler's registry (v0.4.2) under the same id and name as in FormPortal, FormTray and FormRanker. What remains is the owner's: the first push and `compile_v0.4.2` branch, and the family's catalogue key pair.
