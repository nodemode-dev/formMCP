# HASHOUT_LOG · core · formMCP

Living log for `formmcp-core`, `formmcp-cli` and `setup/family.py`. Source files stay comment-free; what they would have said lives here.

Current build version: v0.2.0
Last updated: 2026-10-24 · the gateway: installed apps' own tools on demand; 2026-10-24 · the family list and the MCP server

## v0.2.0 · the gateway {#v0-2-0}

### reach.rs · where an app can be started, without starting it
- refers-to: core/src/reach.rs :: Places, Route, Reach, portal_root, portal_program
- kind: decision
- logged: v0.2.0 · 2026-10-24

Two routes, tried in this order for an app whose registry entry says it has an MCP server: a dev checkout, then a FormPortal install. A checkout is `<root>/<app>` or `<root>/<id>` under any folder in `FORMMCP_DEV_ROOTS` holding the app's own launcher (`build.cli` from formcompiler's registry, now in `family.json` as `launcher`: `form-md` for form.md, `<app>.bat` on Windows); the launcher's `mcp` is the app's own stdio bridge, which already knows how to start or attach to its engine (formvis's port file, form.md's local server), so formMCP needs no per-app knowledge of ports. An install is a stable entry for the catalogue id in FormPortal's `installed.json` (FormTray's lookup of FormPortal's folder: `FORMPORTAL_ROOT`, else the per-OS folder), started as `<FormPortal program> app-mcp <id>`; the program is `FORMPORTAL_BIN`, else `formportal-cli` or `formportal` on PATH, else the macOS app bundle. Windows and Linux standard install paths of the FormPortal program are not guessed. Family plan Q9 (b), attaching to a running app through a port file, is not built: only formvis has one, set by its own packaged `mcp` mode, so a generic attach needs a family convention first; the launchers cover the case meanwhile. `installed.json` is re-read before every `family_apps` and every start, so an install made while the client runs is seen.

### child.rs · one app's server as a child process
- refers-to: core/src/child.rs :: AppServer::start, request, load_tools, Drop
- kind: decision
- logged: v0.2.0 · 2026-10-24

The app's server is spawned with stdin, stdout and stderr piped and formMCP's own environment, so anything the app needs (form.md's `FORMMD_MCP_TOKEN`) is set once in the client's config for formmcp. A reader thread turns stdout lines into JSON and a second keeps the last 20 stderr lines, which every failure message carries. `start` sends `initialize` (2025-06-18, client formmcp), `notifications/initialized`, then `tools/list` following `nextCursor`. `request` waits for its own id with a deadline (start 600 s, since a first run may build the app; calls 900 s; `FORMMCP_START_TIMEOUT`, `FORMMCP_CALL_TIMEOUT`), notes `notifications/tools/list_changed`, and answers any request the app sends (sampling, roots, elicitation) with -32601 so the app never waits on formMCP. Dropping the server kills and reaps the process, so closing the client ends every child.

### mcp.rs · Gateway, app_tools and call_app
- refers-to: core/src/mcp.rs :: Gateway, family_apps, app_tools, call_app, Outcome
- kind: decision
- logged: v0.2.0 · 2026-10-24

The plan's token rule decides the shape: formMCP's own tool list stays three tools whatever is installed, and an app's tools are fetched only when `app_tools` is called for it. `call_app` returns the app's `tools/call` result unchanged (`Outcome::Passed`), so its content, `structuredContent` and `isError` reach the agent exactly as the app wrote them; formMCP's own errors (unknown app, no server, not reachable, start failure with the app's stderr) are tool errors with the reason. A server that has exited is dropped and started again on the next call. When an app said its tools changed, the next `app_tools` re-reads them; formMCP's own list does not change, so it does not send `list_changed` itself. `family_apps` adds `installed` and `reach` (`dev`, `formportal`, or null; the route a running server was started by). Checked by hand: FormPortal's checkout through the dev route (33 tools, `status` passed through, its own unknown-tool error passed through), a stand-in install through `formportal app-mcp` (refused with FormPortal's reason without a launcher, listed with one), form.md's checkout (its own refusals relayed: no local server, then no token), an app with no server, an unreachable install.

## v0.1.0 · the family list and the MCP server {#v0-1-0}

### family.rs · the list is compiled in
- refers-to: core/src/family.rs :: BUNDLED, Family::bundled, Family::get, describe_source
- kind: decision
- logged: v0.1.0 · 2026-10-24

`core/data/family.json` is read with `include_str!`, so the engine, the command line and the installed app all carry the same list and none of them reads a file at run time. `bundled` checks the `format` and returns an error rather than panicking, so a bad regeneration shows up as a message from `formmcp mcp`, not a crash. `get` matches the catalogue id, the registry's app name or the display name, case-insensitively (`formvis`, `FormVis`, `formai` and `formportal` all work). The signed catalogue (`apps.json`) becomes the first source in a later version, with this file as the offline fallback.

### setup/family.py · generated from formcompiler's registry
- refers-to: setup/family.py :: build, main
- kind: decision
- logged: v0.1.0 · 2026-10-24

The family plan forbids hand-kept copies. The generator reads formcompiler's `apps/*.json`: the catalogue id, name, kind and summary, the repository, `state`, `enabled` (written as `released`) and the `agents` block (formcompiler 0.4.0). It records formcompiler's VERSION and commit. Output is sorted by id with a trailing newline, so regenerating from the same commit changes nothing; `--check` exits 1 while the file drifts. formMCP, formskills and formranker are not in formcompiler's registry yet (family plan, Phase 5), so they are not in the list.

### mcp.rs · one tool, stdio, no dependency
- refers-to: core/src/mcp.rs :: tools, family_apps, answer, serve
- kind: decision
- logged: v0.1.0 · 2026-10-24

The server is hand-rolled on `serde_json` in FormPortal's pattern (`cli/src/mcp.rs` there), so formMCP adds no MCP crate: newline-delimited JSON-RPC on stdin and stdout, batches answered as arrays, notifications (no `id`) answered with nothing, a parse error answered with -32700, an unknown method with -32601. It negotiates the protocol version the client asks for when it is one of the four FormPortal supports. `family_apps` returns the short fields only (no repository, no `released`), about 3.8 KB for 21 apps, both as text and as `structuredContent`. The instructions say formMCP never installs and that FormPortal does, which is the plan's rule that formMCP suggests and FormPortal installs. Checked by hand through `formmcp-cli mcp` and the shell's `formmcp mcp`: initialize, tools/list, family_apps with and without `mcp_only`, an unknown tool, a line that is not JSON.

### cli/src/main.rs · the same engine, no window
- refers-to: cli/src/main.rs :: main, HELP
- kind: rationale
- logged: v0.1.0 · 2026-10-24

`mcp` is checked before the list is loaded so the server reports its own errors on stderr; `version` and `help` need no list. `apps` adds `--mcp-only`; `app <id>` prints one app or exits 1 with the reason.
