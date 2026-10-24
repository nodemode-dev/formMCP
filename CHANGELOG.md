# Changelog

## v0.4.0 · 2026-10-24

- The window connects formMCP to Claude Code, Codex and Cursor with one button each, and shows whether each is connected.
- Each app's card shows whether it is installed, its version and how formMCP reaches it, and, for apps with an MCP server, an "Always show tools" switch.
- An Activity panel shows sessions, tool calls and app starts as they happen, from a log every formMCP server writes; arguments and results are never recorded.
- Claude Code counts as connected only when formMCP is in your user settings; a project's `.mcp.json` no longer reads as connected.

## v0.3.0 · 2026-10-24

- `suggest_app`: the task in the person's words in, the Form apps that fit out, best first, each with the next step (load its tools, install it with FormPortal, install FormPortal first, or why an agent cannot drive it yet).
- The family list comes from the signed family catalogue, checked against the same keys FormPortal trusts, with FormPortal's kept copy when offline and the built-in list as the fallback; every answer says which list was used and why.
- One app's own tools can be shown directly in formMCP's tool list (`./formmcp cli expose <id> on`).
- `./formmcp cli agents install` adds formMCP to Claude Code, Codex and Cursor for your user account (`status`, `remove`), backing up each file and leaving other servers alone.

## v0.2.0 · 2026-10-24

- formMCP reaches each Form app's own MCP server: `app_tools` starts it and lists its tools, `call_app` runs one and returns the app's result unchanged. Only the app the task needs is loaded.
- An app is started through its checkout's own launcher (folders in `FORMMCP_DEV_ROOTS`; `./formmcp` uses the folder beside its checkout) or through FormPortal (`app-mcp`) for an installed app. `family_apps` shows whether each app is installed and how it can be reached; `./formmcp cli reach` shows the same.
- An app's failure to start comes back with its own last output, for example form.md asking for its token.

## v0.1.0 · 2026-10-24

- First version: the Form family list generated from formcompiler's registry, the MCP server with `family_apps` over stdio (from a checkout or from the installed app with no window), the command line (`apps`, `app`, `mcp`, `version`), and a window listing the family with filters for MCP server and skills.
- Wired for Claude Code (`.mcp.json`), Codex (`.codex/config.toml`) and Cursor (`.cursor/mcp.json`), with the `formmcp-harness` skill.
