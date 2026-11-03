# Changelog

## v0.7.6 · 2026-11-03

- The app is named FormMCP in its window, title bar and installers, and it has the unified Form family icon. The command line, the bundle identifier and the program names are unchanged.

## v0.7.5 · 2026-11-02

- The family catalogue is read from the public download store, `https://dl.formai.studio/catalog/apps.json` with its signature beside it, instead of formcompiler's private release page. `FORMMCP_CATALOG` still overrides it.
- This build trusts the family's production catalogue key (`P3Cgp37EmB2jOBtTWRNFtO8xMq8hsb+Ii3U5+UrYNtY=`), so a catalogue formcompiler signs is used instead of being refused as untrusted.
- Links to the Form repositories point at the `nodemode-dev` organisation, and the built-in family list is regenerated from formcompiler 0.6.1: it now holds 24 apps, FormPromo and FormRanker among them.

## v0.7.4 · 2026-10-26

- A test suite checks the family list and the signed catalogue, which apps this computer can reach, the MCP server over stdio and over HTTP (its access token, Host and Origin checks), and that a call to an app that cannot start fails at once with the reason; it runs on every change. Nothing in the app changes.

## v0.7.3 · 2026-10-26

- Release builds keep their symbols (`strip = false`), because macOS builds fail when the release profile strips the binary. Nothing in the app changes.

## v0.7.2 · 2026-10-25

- The window's build tooling moves to Vite 8.3.5, which clears two command-injection advisories in the development server's editor launcher. It is a development dependency only; nothing in the installed app changes.

## v0.7.1 · 2026-10-25

- The file check uses FormPortal's own record: FormPortal 0.46.0 records every installed file's SHA-256, and formMCP compares the install with it before starting the app, so a change made before formMCP first started it is caught too. With an older FormPortal, formMCP compares with what it recorded at the first start, as in 0.7.0.

## v0.7.0 · 2026-10-25

- Secrets in the system keychain: keep a secret for an app (form.md's `FORMMD_MCP_TOKEN`, say) and formMCP gives it to that app's server as an environment variable when it starts it. Only the name is written to settings.json.
- Secrets hidden: known secrets and token-shaped text (API keys, access tokens, JWTs, private keys, password and token assignments) are replaced in what apps send back before an agent sees it (on by default), and always in the activity log.
- A file check before an installed app starts: formMCP records an installed version's files the first time it starts it and refuses to start it if they change, with the files named. Record again accepts a change you made.
- Hooks: a command of yours runs before each app tool call and can refuse it (a failure or timeout refuses too), and another after each call with the outcome.
- Tracing: each call and app start can be sent to an OpenTelemetry collector over HTTP as a trace and metrics, with names and outcomes only.
- The window's Safety panel holds all of this; the command line gains `secret`, `verify`, `redact`, `hook` and `telemetry`.
- On Linux, building formMCP now needs libdbus (`libdbus-1-dev` and `pkg-config` on Debian or Ubuntu); a desktop with WebKitGTK already has it.

## v0.6.0 · 2026-10-25

- Profiles: save what an agent gets (apps off, tools off, start policies) under a name, choose a profile per agent, and export or import it as one file to share. A profile only ever turns more off, never less.
- Start policies per app: start on demand and stop after 10 idle minutes (the default), start in the background when an agent's session starts, or keep running while formMCP runs.
- When a profile turns an app off, the agent is told which profile did it.
- The activity log shows apps stopped for being idle and how each app was started.
- `formmcp start` and `formmcp profile` on the command line.

## v0.5.0 · 2026-10-25

- formMCP also answers over HTTP, on this computer only: Start in the window (or `formmcp serve`) and point an agent at `http://127.0.0.1:5625/mcp` with the access token. Copy setup gives the address, the header and a ready Claude Code command; New token replaces the token at once.
- Choose which apps each agent gets: pick Claude Code, Codex, Cursor or other agents, and switch apps on or off for it.
- Turn single tools off, for every agent, from an app's Tools panel.
- Apps' own resources and prompts come through four new tools: `app_resources`, `read_app_resource`, `app_prompts` and `get_app_prompt`.
- Changes in the window apply to running agents from their next request.
- The activity log shows HTTP starts, stops and refused requests, and which agent each session counts as.

## v0.4.2 · 2026-10-24

- formMCP is a registered Form app: formcompiler builds and releases it, and FormPortal, FormTray and FormRanker know it as `formmcp`, formMCP, `ai.nodemode.formmcp`.
- `./build_app` (macOS and Linux) and `build_app.bat` (Windows) build the desktop app and its installers.
- The family list includes formskills, and no longer could include formMCP itself.

## v0.4.1 · 2026-10-24

- Skills are no longer part of formMCP: the window's Skills filter and badge, and the `skills` field in `family_apps` and `./formmcp cli apps`, are gone. The Form apps' released skills are in formskills, and formMCP tells agents so.

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
