# formMCP

formMCP is one MCP server for the whole Form family. Add it to Claude Code, Codex or Cursor once, and the agent can see every Form app, with one line on what each is for, whether it has an MCP server, and whether it ships skills, whether or not any app is installed.

**Current version:** v0.1.0 · **Stack:** Rust (`formmcp-core`, `formmcp-cli`) · Tauri 2 shell · vanilla TypeScript and Vite window · **Team:** one owner (nodemode-ai)

| Page | What it covers |
| --- | --- |
| [Getting Started](Getting-Started) | install from source, run the window, the command line and the MCP server |
| [Architecture](Architecture) | the engine, the family list and where it comes from, the shell, the window |
| [Agents](Agents) | connecting Claude Code, Codex and Cursor, and the `family_apps` tool |

## What it does in v0.1.0

- `family_apps`: the Form family from formcompiler's registry, generated into the build, never edited by hand.
- A window listing the same apps, filterable by MCP server and skills.
- `formmcp mcp` on stdio, from a checkout (`./formmcp mcp`) or from the installed app with no window.

## What comes next

Per the family plan (Phase 3): suggesting the app that fits a task and FormPortal to install it (v0.3.0); passing installed apps' own tools through on demand (v0.2.0); connected clients and per-app switches in the window (v0.4.0).
