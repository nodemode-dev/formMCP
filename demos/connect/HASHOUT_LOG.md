# HASHOUT_LOG · demos/connect · formMCP

Living log for demo 2. Source files stay comment-free; what they would have said lives here.

Current build version: v0.14.1
Last updated: 2026-11-20 · v0.14.1 the older Antigravity file note; v0.14.0 four agent programs and FormVis's surfaces; v0.9.1 rebuilt for the version it shows (no demo changes); v0.9.0 rebuilt with the window's Extensions button (empty list: no extension route in a file:// demo); first release of the connecting-agents demo

## v0.14.1 · the older Antigravity file note {#v0-14-1}

### world.ts, workbench.ts, connect.css · an older `antigravity-cli` file, seen and left alone
- refers-to: demos/connect/src/world.ts :: antigravityOlderPath, olderSeen, stateOf, connect, reset · demos/connect/src/workbench.ts :: buildSetup · demos/connect/src/connect.css :: .wb-older
- kind: change
- logged: v0.14.1 · 2026-11-20

Ported from `clients.rs` `antigravity_older` and `older_seen`: when `~/.gemini/antigravity-cli/mcp_config.json` exists and the documented `~/.gemini/config/mcp_config.json` does not, the Antigravity state carries the note "an older Antigravity file, [path], was seen and left alone; formMCP writes [path]", in the same words. As in the engine it is the note of a not connected state (after the "taken" note, which wins), and Connect returns it once beside the new state (`after.note.or(older)`). The real window shows a client's note as the Connect button's tip, so the card does the same here; the Agent setup pane also prints it on a wrapping line under the file's title, because the demo's purpose is to show what the agent's files hold. The virtual home now starts with that older file (the Cursor sample's content), so the note is there on opening; "No file yet" brings it back, and "Start over" restores it. The older file itself is never changed.

## v0.14.0 · four agent programs, FormVis's surfaces {#v0-14-0}

### world.ts, workbench.ts, shim.ts, agent.ts, src/index.html · Antigravity in demo 2
- refers-to: demos/connect/src/world.ts :: ClientKey, antigravityPath, fileOf, setFile · demos/connect/src/workbench.ts :: CLIENTS, AGENTS · demos/connect/src/shim.ts :: patched · demos/connect/src/agent.ts :: clientKey, initialize · demos/connect/src/index.html :: .sub, .clients, #agent
- kind: change
- logged: v0.14.0 · 2026-11-20

Antigravity is the fourth client, written as `clients.rs` writes it: the same JSON entry as Cursor, in `~/.gemini/config/mcp_config.json`. The virtual home starts without that file, so the Agent setup tab shows the "no file yet" case for it; the older `antigravity-cli` file note of `clients.rs` is not modelled. The JSON paths read and write through `fileOf` and `setFile` so Cursor and Antigravity share one code path, as `json_config` does in the engine. `clientKey` matches "antigravity" first, as `settings.rs` does; the ask-as form initializes Antigravity with the name "antigravity", a stand-in, because the name the real program gives has not been seen. The server's `serverInfo.version` is the build's version instead of a fixed 0.9.1. Rebuilt with the real tokens.css, so the page takes FormVis's surfaces from 0.13.0.

## v0.11.0 · gateway wave 1 {#v0-11-0}

### agent.ts · the source line has no commit
- refers-to: demos/connect/src/agent.ts :: sourceText
- kind: change
- logged: v0.11.0 · 2026-11-19

The family list no longer records a commit (see core/HASHOUT_LOG.md, the family spec entry), so `sourceText` returns "bundled list from <repository> <version>". The compiled page in `demos/connect/assets` still shows the old line until the demo is rebuilt for the release.

### agent.ts · the source line names the repository only
- refers-to: demos/connect/src/agent.ts :: sourceText
- kind: change
- logged: v0.11.0 · 2026-11-19

The family list no longer carries the registry version (see core/HASHOUT_LOG.md, "the source line names the repository only"), so `sourceText` returns "bundled list from <repository>". The compiled page in `demos/connect/assets` still shows the old line until the demo is rebuilt for the release.

## v0.8.4 · connecting agents {#v0-8-4}

### main.ts, isolate.ts, shim.ts · the real window, memory only
- refers-to: demos/connect/src/main.ts :: import order · src/isolate.ts · src/shim.ts :: patched
- kind: decision
- logged: v0.8.4 · 2026-11-07

Imports run in order: the three real stylesheets, this demo's css, isolate (a memory-backed `localStorage` so demo 1's saved settings are never read or written), demo 1's shim (it installs `window.__TAURI__`), then the real `frontend/src/main.ts`, then the workbench. shim.ts wraps `invoke` instead of copying demo 1's handlers: `connect`, `http_switch`, `http_token` and `http_rotate` are answered from this demo's virtual home and HTTP server, `snapshot` is patched so four apps read as installed and the three clients read from the virtual files, and everything else falls through to demo 1. After a change the workbench is told through a `world-change` event so every pane repaints from state.

### world.ts · a virtual home and the ported `clients.rs`
- refers-to: demos/connect/src/world.ts :: state, nextPlan, write, diffLines
- kind: decision
- logged: v0.8.4 · 2026-11-07

`state` decides connected, not connected, taken (another server uses the name formmcp) or no client; `nextPlan` computes the text the next Connect or Disconnect would write, with the backup, using the same section edit (`toml_section`), quoting and shell-setting copy as the Rust source. Claude Code is not a file: formMCP only runs two commands, so the pane shows the commands and the entry read back. The platform switch changes the home folder, path separator and launcher and restarts the sample files. The Codex and Cursor results were compared with the compiled `clients.rs` (a scratch crate including the source by path) on five starting files and matched byte for byte.

### httpd.ts · the ported `http.rs` handler
- refers-to: demos/connect/src/httpd.ts :: handle
- kind: decision
- logged: v0.8.4 · 2026-11-07

One function takes a request and returns a status, headers and body, in the order the engine checks: path, Host, Origin, token (constant-time compare), protocol version, session. A refusal is logged as `http-refused` with its status and reason, never with the token or body. New token drops every session, so the old token fails at once, as in the app.

### workbench.ts · three panes, built once
- refers-to: demos/connect/src/workbench.ts :: build, buildSetup, buildHttp, buildView
- kind: decision
- logged: v0.8.4 · 2026-11-07

Every pane is built once and hidden or shown by tab. A repaint only changes text, `data-` attributes and `hidden`, and the diff rows are replaced as one batch inside a fixed-height box. `booting` stays on until the app icons have decoded (1.5 s failsafe), so the first visible frame is final and nothing moves.

### Not done
- The window pictures (hero, agents, safety) that the plan put in the Agent view header were left out: the icons carry the look and the real window is on screen.
- Claude Code's own settings store is not modelled.
- Checked in Chromium over file:// only.
