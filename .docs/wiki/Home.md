# formMCP

formMCP is one MCP server for the whole Form family. Add it to Claude Code, Codex or Cursor once, and the agent can see every Form app, with one line on what each is for, whether it has an MCP server, and whether it ships skills, whether or not any app is installed.

**Current version:** v0.2.0 · **Stack:** Rust (`formmcp-core`, `formmcp-cli`) · Tauri 2 shell · vanilla TypeScript and Vite window · **Team:** one owner (nodemode-ai)

| Page | What it covers |
| --- | --- |
| [Getting Started](Getting-Started) | install from source, run the window, the command line and the MCP server |
| [Architecture](Architecture) | the engine, the family list and where it comes from, the shell, the window |
| [Agents](Agents) | connecting Claude Code, Codex and Cursor, and the `family_apps` tool |

## What it does

- `family_apps`: the Form family from formcompiler's registry, generated into the build, never edited by hand, with each app's install state and how formMCP can reach it on this computer (v0.2.0).
- `app_tools` and `call_app`: one app's own tools, loaded only when asked for, and its results passed through unchanged (v0.2.0).
- A window listing the same apps, filterable by MCP server and skills.
- `formmcp mcp` on stdio, from a checkout (`./formmcp mcp`) or from the installed app with no window.

## What comes next

Per the family plan (Phase 3): the signed catalogue as the list's first source and suggesting the app that fits a task with FormPortal to install it (v0.3.0); connected clients and per-app switches in the window (v0.4.0).
