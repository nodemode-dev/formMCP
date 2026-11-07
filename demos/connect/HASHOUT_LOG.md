# HASHOUT_LOG · demos/connect · formMCP

Living log for demo 2. Source files stay comment-free; what they would have said lives here.

Current build version: v0.9.1
Last updated: 2026-11-07 · v0.9.1 rebuilt for the version it shows (no demo changes); v0.9.0 rebuilt with the window's Extensions button (empty list: no extension route in a file:// demo); first release of the connecting-agents demo

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
