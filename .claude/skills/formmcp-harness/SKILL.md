---
name: formmcp-harness
description: Find the Form family app that fits a task through formMCP's family_apps tool, and say how to get it. Use when the person wants something a Form app does (edit a picture, cut a video, mix audio, lay out a page, draw vector art, take notes, make slides, encode media, manage a library) and formmcp tools are available, or formMCP or the Form family is mentioned. Not for installing apps (FormPortal does that) and not for changing formMCP's own code (read CLAUDE.md).
---

# formMCP harness

formMCP is one MCP server for the whole Form family. It keeps the conversation small: an app costs one line until it is needed, and formMCP never copies any app's tools or skills.

## The tool

| Tool | Changes anything | What it returns |
| --- | --- | --- |
| `family_apps` | no | every Form app: `id`, `name`, `kind`, a one-line `summary`, `mcp` (it has an MCP server), `skills` (it ships skills), `state` (`live`, `public`, `prelaunch`); `mcp_only: true` keeps the apps with an MCP server |

## Workflow

1. Call `family_apps` once per conversation, not per message. Pick the app whose `kind` and `summary` fit the task; if two fit, name both and say how they differ.
2. Say what the person gets: the app's name, whether an agent can drive it (`mcp`), and whether it is out (`live` or `public`) or still coming (`prelaunch`).
3. To get an app, point to FormPortal (`id` `formai`): it installs, updates and rolls back every Form app. formMCP never installs, updates or removes anything, and has no tool that could.
4. Never claim an app's own tools from this list. In this version formMCP lists the family only; reaching an installed app's own tools through formMCP comes in a later version. Until then, an installed app with `mcp: true` is connected on its own, and FormPortal's `connect_agents` adds it to Claude Code, Codex and Cursor.

## Without MCP

`./formmcp cli apps` (and `--mcp-only`) prints the same list as JSON; `./formmcp cli app <id>` prints one app.
