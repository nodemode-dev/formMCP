# HASHOUT_LOG · core · formMCP

Living log for `formmcp-core`, `formmcp-cli` and `setup/family.py`. Source files stay comment-free; what they would have said lives here.

Current build version: v0.4.1
Last updated: 2026-10-24 · skills out of formMCP; the activity log and Claude Code's user scope; the signed catalogue, suggest_app, always-expose, agents install; the gateway; the family list and the MCP server

## v0.4.1 · skills out of formMCP {#v0-4-1}

### family.rs, catalog.rs, mcp.rs, setup/family.py · no `skills` field
- refers-to: core/src/family.rs :: App · core/src/catalog.rs :: Agents, merge · core/src/mcp.rs :: INSTRUCTIONS, family_apps · setup/family.py :: build · cli/src/main.rs :: HELP
- kind: decision
- logged: v0.4.1 · 2026-10-24

The owner: "the skills need to be formskills and not in formMCP". formMCP is tools only: the family list no longer carries whether an app ships skills, the catalogue's `agents.skills` is not read (serde ignores the field), and `family.json` was regenerated from formcompiler 0.4.0 (30943ac) without it. The server's instructions end by saying formMCP does not handle skills and that the Form apps' released skills are viewed and downloaded in formskills, so an agent asked for a skill is sent there instead of guessing. formMCP's own agent skill (`.claude/skills/formmcp-harness`) stays in the checkout: it is the source of formMCP's released skills asset, the same as every Form app's, and formskills is where it is shown.

## v0.4.0 · the activity log {#v0-4-0}

### log.rs · what the window shows, without arguments
- refers-to: core/src/log.rs :: record, tail · core/src/mcp.rs :: answer (session, call), Gateway::server (app-start)
- kind: decision
- logged: v0.4.0 · 2026-10-24

The window and the MCP server are different processes (each client starts its own server), so the server appends one JSON line per event to `log.jsonl` in formMCP's settings folder and the window reads it. Events: `session` (the client's name and version, the exposed apps), `call` (formMCP's tool name, the app and the app's tool when there is one, ok, milliseconds) and `app-start` (app, route, ok, tool count or the first line of the reason, milliseconds), each with time and process id. Tool arguments and results are never written: they can hold the person's files, text and paths. Past 1 MB the file is renamed to `log.1.jsonl` and a new one begun. `tail(from)` returns the complete lines after a byte offset and the offset to ask from next, at most the last 64 KB, starting over when the file shrank (after a rename); a partial last line is left for the next read.

### clients.rs · Claude Code: user scope only
- refers-to: core/src/clients.rs :: state (Client::Claude)
- kind: fix
- logged: v0.4.0 · 2026-10-24

`claude mcp get formmcp` also finds a project's `.mcp.json` (this repository has one), so a window started inside the checkout reported Claude Code "Connected" with nothing at user scope. The check now runs in the home folder and counts an entry only when its output says `Scope: User`; a project entry is left out of the state and never touched. Checked with the real `claude` command against a scratch home: not connected, install (connected, user scope), remove (not connected). FormPortal's `client_state` has the same blind spot (logged in the family plan's notes for its next release).

## v0.3.0 · catalogue, suggestions, exposure, clients {#v0-3-0}

### catalog.rs · FormPortal's trust model, unchanged
- refers-to: core/src/catalog.rs :: load, read_source, verify, keys, merge · core/catalog_keys.txt
- kind: decision
- logged: v0.3.0 · 2026-10-24

formMCP trusts a catalogue exactly as FormPortal does (`core/src/signature.rs` and `portal.rs` there): an Ed25519 signature over the body, checked against keys compiled in from `core/catalog_keys.txt` plus `FORMAI_CATALOG_PUBKEY`, the same file contract and variable name, so one key pair from formcompiler's `catalog.py keygen` serves both. Neither file holds a key yet, so today every catalogue is refused with "this build trusts no catalogue key yet" and the bundled list is used; that is the intended state until the owner generates the pair. Order: the catalogue at `FORMMCP_CATALOG` (default formcompiler's `catalog` release; https, loopback http or a file) with `<url>.sig`; when that cannot be fetched, FormPortal's kept copy (`<FormPortal>/catalog/apps.json` and `.sig`), checked the same way; `FORMMCP_OFFLINE=1` reads only the kept copy; `FORMMCP_ALLOW_UNSIGNED=1` admits an unsigned one, as FormPortal's `--allow-unsigned` does. Fetches time out after 8 s. A product in the catalogue replaces the bundled entry's name, kind, summary, state, MCP and skills flags and adds its newest version; a product the bundled list lacks is added; bundled apps the catalogue lacks (not released yet) stay. Every outcome is named in `source`, so the agent and the person see which list they got and why. The catalogue is read once per session, on the first tool call, not at `initialize`, so a slow network never delays the client's start. Checked: signed (accepted, versions shown), no trusted key, tampered body, unsigned, unsigned with the override, FormPortal's kept copy offline; the default URL here fails TLS because this sandbox's proxy has its own certificate authority, and falls back as designed.

### suggest.rs · words, not a model
- refers-to: core/src/suggest.rs :: suggest, score, next_step, KIND_WORDS, STOP
- kind: decision
- logged: v0.3.0 · 2026-10-24

The task's words (lowercased, stop words dropped) are scored against each app: its id or name 5, its kind 2, any word of its name, kind, summary or the kind's word list 1 (a word matches when equal, or when both have at least four letters and share the first five). FormPortal is suggested only when the task is about installing or updating. Ties go to released apps. The word lists are formMCP's own heuristics, not a copy of any app's data. Each suggestion carries the next step: `app_tools` when reachable; no MCP server yet; installed but no launcher; not released; install with FormPortal; or install FormPortal first (its repository page). The agent still decides; when nothing matches it is told to use `family_apps`. Checked on 17 tasks (background removal, raw photos, interview video, podcast, logo, pitch deck, PDF signing, notes wiki, budget, 3D, landing page and others); a filler-word false match (FormCel and FormDAM on "out" and "own") and a kind-word overweight (FormLit above FormVis for background removal) were fixed by the stop list, the kind weight 3 to 2 and two image words.

### mcp.rs · always expose
- refers-to: core/src/mcp.rs :: Gateway::exposed_tools, call (EXPOSED_SEPARATOR) · core/src/settings.rs
- kind: decision
- logged: v0.3.0 · 2026-10-24

For an app listed in `settings.json` `expose` (or `FORMMCP_EXPOSE`), `tools/list` starts it and adds its tools as `<id>__<tool>`, described with `[<Name>]` in front, and a call to such a name goes through `call_app`. Off by default, per the plan's token rule; it is for people who live in one app. `capabilities.tools.listChanged` is true only when something is exposed; when an exposed app says its tools changed during a call, formMCP sends `notifications/tools/list_changed` after its answer. An exposed app that cannot start is skipped with a line on stderr, so formMCP still starts. Settings live in `FORMMCP_HOME`, else Application Support (`ai.nodemode.formmcp`) on macOS, `%APPDATA%\formMCP` on Windows, `~/.config/formmcp` elsewhere.

### clients.rs · FormPortal's client writer, for formMCP
- refers-to: core/src/clients.rs :: state, connect, write, remove, Launcher
- kind: decision
- logged: v0.3.0 · 2026-10-24

Ported from FormPortal's `core/src/agents.rs` (same owner and licence) with formMCP's name: Claude Code through `claude mcp add --scope user formmcp`, Codex as `[mcp_servers.formmcp]` in `$CODEX_HOME/config.toml` (section replaced or appended; a `[mcp_servers.formmcp.env]` subtable under it counts as part of the section and is replaced with it, not left behind; FormPortal's version stops at the subtable), Cursor as `mcpServers.formmcp` in `~/.cursor/mcp.json`; each file copied to `.bak-<time>` before a write; an entry that does not mention formmcp is reported `taken` and never touched. The program written is `FORMMCP_LAUNCHER` (the `./formmcp` script sets its own path) or the running executable, with `mcp`; formMCP's own settings that are set (`FORMMCP_DEV_ROOTS`, `FORMPORTAL_ROOT`, `FORMPORTAL_BIN`, `FORMMCP_CATALOG`, `FORMAI_CATALOG_PUBKEY`) go into the entry's env, because clients do not start servers with the person's shell. Codex gets `startup_timeout_sec = 600` and `tool_timeout_sec = 900` to match the child timeouts. Checked in a scratch home with existing entries: install, a second install (one section, a backup), remove (the other entries intact), a foreign `formmcp` entry left alone, an unknown client refused; Claude Code was kept off PATH so this machine's real configuration was never touched.

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
