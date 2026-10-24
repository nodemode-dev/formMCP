---
name: formmcp-harness
description: Find the Form family app that fits a task through formMCP, and drive it through its own tools when it is reachable. Use when the person wants something a Form app does (edit a picture, cut a video, mix audio, lay out a page, draw vector art, take notes, make slides, encode media, manage a library) and formmcp tools are available, or formMCP or the Form family is mentioned. Not for installing apps (FormPortal does that) and not for changing formMCP's own code (read CLAUDE.md).
---

# formMCP harness

formMCP is one MCP server for the whole Form family. It keeps the conversation small: an app costs one line until it is needed, and an app's own tools are loaded only for the app the task needs.

## The tools

| Tool | Changes anything | What it does |
| --- | --- | --- |
| `family_apps` | no | every Form app: `id`, `name`, `kind`, a one-line `summary`, `mcp` (it has an MCP server), `skills`, `state` (`live`, `public`, `prelaunch`), `installed` (by FormPortal on this computer) and `reach` (`dev`, `formportal`, or null: how formMCP can start its server here); `mcp_only: true` keeps the apps with a server |
| `app_tools` | no | starts one app's own server (or reuses it) and returns its tools with input schemas, its name and version, and its instructions; `refresh: true` asks again |
| `call_app` | what the app's tool changes | runs one of that app's tools with `arguments` and returns the app's own result unchanged |

## Workflow

1. Call `family_apps` once per conversation. Pick the app whose `kind` and `summary` fit the task; if two fit, name both and say how they differ.
2. If the app has `reach`, call `app_tools` for that one app, read its instructions and the tool you need, then `call_app`. Load another app only when the task moves to it.
3. Before a `call_app` that changes the person's files or computer (the app's tool is not read-only, or says so), say what will change and get their go-ahead. The app's own tool descriptions and annotations decide this, not formMCP's.
4. If the app has no `reach`: when `installed` is false, say which app fits and that FormPortal (`id` `formai`) installs and updates the Form apps; formMCP never installs. When `installed` is true but `reach` is null, FormPortal could not be found to start it, or the app's install names no launcher yet; say so plainly.
5. When `app_tools` or `call_app` fails, the message carries the app's own last output. Pass on what it asks for (for example form.md asks for `FORMMD_MCP_TOKEN`, a personal API token from its Account settings, set in the client's config for formmcp) instead of retrying.

## Without MCP

`./formmcp cli apps` (and `--mcp-only`) prints the list as JSON; `./formmcp cli app <id>` one app; `./formmcp cli reach` how each app would be started here.
