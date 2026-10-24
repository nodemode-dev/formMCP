# HASHOUT_LOG · src-tauri · formMCP

Living log for formMCP's desktop shell. Source files stay comment-free; what they would have said lives here.

Current build version: v0.4.0
Last updated: 2026-10-24 · the window's commands; the shell, and `formmcp mcp` without a window

## v0.4.0 · the window's commands {#v0-4-0}

### lib.rs · snapshot, set_expose, connect, log_tail
- refers-to: src/lib.rs :: snapshot_now, snapshot, set_expose, connect, log_tail
- kind: decision
- logged: v0.4.0 · 2026-10-24

`snapshot` gathers, off the main thread (`spawn_blocking`, since the catalogue may wait up to 8 s on the network and `claude mcp get` starts a process): the family from the catalogue over the built-in list, each app's install state, route, version and switch, the three clients' states, whether FormPortal is on this computer, and whether `FORMMCP_EXPOSE` overrides the saved switches. `set_expose` writes only `settings.json` (read without the environment override, so a session variable never gets saved). `connect` adds or removes formMCP in one client with this program and `mcp` as the launcher, so an installed copy registers itself. `log_tail` passes the activity log through. The window is 900 by 780 (minimum 560 by 560) to hold the clients strip, the list and the activity panel. Checked under Xvfb with a scratch home and a real session's log: the window shows the clients, every app's local line, the status line, the activity rows and the source.

## v0.1.0 · the shell {#v0-1-0}

### lib.rs · `mcp` before any window code
- refers-to: src/lib.rs :: run
- kind: decision
- logged: v0.1.0 · 2026-10-24

When the first argument is `mcp`, `run` serves MCP on stdio through `formmcp_core::mcp::serve` and exits before `tauri::Builder` is touched, so no window, webview or event loop is created. That makes the installed program its own MCP launcher, which is what the family plan's Q9 recommends for every app (formcompiler's `agents.mcp_launch`). On Windows the release binary is a GUI-subsystem program; a client that starts it with pipes still gets stdin and stdout.

### tauri.conf.json · one window, no plugins
- refers-to: tauri.conf.json, capabilities/default.json
- kind: rationale
- logged: v0.1.0 · 2026-10-24

One `main` window, 820 by 640, resizable down to 520 by 420, `backgroundColor` set to the dark ground so the native window never shows white before the page paints. The page needs no Tauri command, so the capability is `core:default` only and no plugin is linked. The identifier is `ai.nodemode.formmcp` (family plan, Q5). Bundle resources carry `LICENSE` and `NOTICE.md`. Icons in `icons/` are generated from `assets/formmcp_icon.png` with `npx tauri icon` (desktop sizes kept; the Android and iOS folders it also writes were removed).

Checked: `cargo build --workspace --locked` with no warnings, and the debug window launched under Xvfb against the dev server, alive after 12 seconds, showing the list.
