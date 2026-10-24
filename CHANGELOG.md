# Changelog

## v0.2.0 · 2026-10-24

- formMCP reaches each Form app's own MCP server: `app_tools` starts it and lists its tools, `call_app` runs one and returns the app's result unchanged. Only the app the task needs is loaded.
- An app is started through its checkout's own launcher (folders in `FORMMCP_DEV_ROOTS`; `./formmcp` uses the folder beside its checkout) or through FormPortal (`app-mcp`) for an installed app. `family_apps` shows whether each app is installed and how it can be reached; `./formmcp cli reach` shows the same.
- An app's failure to start comes back with its own last output, for example form.md asking for its token.

## v0.1.0 · 2026-10-24

- First version: the Form family list generated from formcompiler's registry, the MCP server with `family_apps` over stdio (from a checkout or from the installed app with no window), the command line (`apps`, `app`, `mcp`, `version`), and a window listing the family with filters for MCP server and skills.
- Wired for Claude Code (`.mcp.json`), Codex (`.codex/config.toml`) and Cursor (`.cursor/mcp.json`), with the `formmcp-harness` skill.
