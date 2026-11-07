# HASHOUT_LOG · src-tauri · formMCP

Living log for formMCP's desktop shell. Source files stay comment-free; what they would have said lives here.

Current build version: v0.9.1
Last updated: 2026-11-07 · v0.9.1 no shell changes (Greg's README note on FormPortal updates); v0.9.0 the formext protocol and the extension commands; v0.8.4 no shell changes (demos 2 and 3 in demos/); v0.8.3 the macos-private-api feature; v0.8.2 the splash, the rounded card and the reveal; v0.8.1 no shell changes (real media in the demo); v0.8.0 no shell changes (the browser demo in demo/); 2026-11-04 · v0.7.8 no shell changes (README in the Form family template, Mackin's #6, merged with main); v0.7.7 no shell changes (README only); 2026-11-03 · v0.7.6 the bundle names the publisher and the licence; 2026-10-25 · the safety commands; profiles and start policies; HTTP inside the window, agents and tool switches; the window's commands; the shell, and `formmcp mcp` without a window

## v0.9.0 · the formext protocol and the extension commands {#v0-9-0}

### lib.rs, tauri.conf.json · formext://, window commands, frame-src
- refers-to: src-tauri/src/lib.rs :: formext_file, run (register_asynchronous_uri_scheme_protocol), extensions_list, extensions_stamp, extension_requests, extension_finish, extension_save, open_portal, settings_mark, settings_restore · src-tauri/tauri.conf.json :: app.security.csp
- kind: decision
- logged: v0.9.0 · 2026-11-07

formMCP's window has no HTTP server of its own (the MCP one needs a token a sandboxed frame cannot send), so extension files come from the custom protocol `formext`, registered asynchronously so a large file is read off the main thread. The handler is thin: `extensions::answer` resolves and checks, the shell reads the file and copies the headers; HEAD gets GET's status and headers with no body (section 4 lets both through); any other method, and any file that cannot be read, is a 404 with the safe headers and no body. The window's CSP gains `frame-src formext: http://formext.localhost` (the second is how Tauri addresses custom protocols on Windows); without it `default-src 'self'` would block the frames. Tauri's own initialisation scripts are main-frame only in 2.12 (`for_main_frame_only: true` in its webview manager), so nothing is injected into an extension's frame. The new commands need no permission entries, as before. Not compiled here (no GTK on this machine); the protocol and command signatures were read against tauri 2.12.5's `register_asynchronous_uri_scheme_protocol` and `UriSchemeResponder::respond`, and the logic they call is in core, which was built and tested.

## v0.8.3 · the macos-private-api feature {#v0-8-3}

### Cargo.toml · tauri features
- refers-to: src-tauri/Cargo.toml :: dependencies.tauri
- kind: decision
- logged: v0.8.3 · 2026-11-07

0.8.2 set `macOSPrivateApi: true` in tauri.conf.json (the transparent splash needs it) but left the tauri crate without the matching `macos-private-api` feature; tauri-build refuses a config that enables the private API when the feature is off, so the app would not have built. The feature is now on, as in formvis. Features do not change Cargo.lock. Not compiled here.

## v0.8.2 · the splash, the rounded card and the reveal {#v0-8-2}

### reveal.rs, lib.rs, tauri.conf.json, Cargo.toml, capabilities/default.json · splash window, reveal timing, macOS corner mask
- refers-to: src-tauri/src/reveal.rs :: mark_launch, conceal, app_ready, show_main, set_seen, round_splash; src-tauri/src/lib.rs :: run; src-tauri/tauri.conf.json :: app.windows, app.macOSPrivateApi
- kind: decision
- logged: v0.8.2 · 2026-11-07

Parity with formvis's shell (owner: all thirteen apps get the same splash and rounded corners). reveal.rs is formvis's, minus the traffic-light call (formMCP keeps the native title bar): the splash holds for at least 2.6 s from launch, its bar completes in 220 ms, the main window then appears whole and the splash closes; after 20 s the window shows anyway. The splash window comes from tauri.conf.json (360x200, undecorated, transparent, no shadow, hidden until `round_splash`, which on macOS makes the window non-opaque and masks its content layer to a 16 px radius). The main window moved from the config to `run`, built hidden on macOS (then on screen at alpha 0, clicks passing through) and visible elsewhere, with the family's dark ground (20, 22, 27) as its background in place of formvis's 10, 10, 10, so it matches the page's `--ground` and no white or off-colour frame shows. Size, minimum size and centring are as they were. The page calls `app_ready` three frames after it removes `booting`; the build's commands need no permission entries (build.rs lists none), and the capability now names the splash window. objc2 0.6 and objc2-app-kit 0.3 are macOS-only dependencies (MIT or Apache-2.0; already in the tree through Tauri). Not compiled: cargo check was skipped on the shared machine; the Rust was read against formvis's reveal.rs and main.rs.

## v0.7.6 · the publisher and the licence in the bundle {#publisher}

### tauri.conf.json, Cargo.toml · publisher, copyright, license, licenseFile, authors
- refers-to: src-tauri/tauri.conf.json :: bundle.publisher, bundle.copyright, bundle.license, bundle.licenseFile; Cargo.toml :: workspace.package.authors
- kind: decision
- logged: v0.7.6 · 2026-11-03

The launch punch list (2026-11-03): the About of a downloaded app says NodeAgentc, Inc., not nodemode-ai, and lists the licence. formMCP has no About of its own: on macOS the application menu's About panel reads the bundle's copyright, and Tauri's default About metadata takes its authors from `bundle.publisher`; tauri-build writes `CompanyName` from `bundle.publisher` into the Windows version resource, and the .deb's Maintainer is the crate's `authors`. The bundle now names the publisher, `licenseFile` points at the LICENSE at the root (`../LICENSE` from `src-tauri`) beside the `license` (GPL-3.0-only) it already named, and `authors` is the company. Checked here: the JSON parses; no build was run.

## v0.7.0 · the safety commands {#v0-7-0}

### lib.rs · safety, secret_set, secret_remove, safety_switch, hook_set, telemetry_set, verify_trust
- refers-to: src-tauri/src/lib.rs :: safety_now, safety_after, safety, secret_set, secret_remove, safety_switch, hook_set, telemetry_set, verify_trust
- kind: decision
- logged: v0.7.0 · 2026-10-25

Every command runs on a blocking thread (the keychain can take a moment, and on Linux asks D-Bus) and answers with the whole safety state, so the page has one render path. `secret_set` checks the app id against the family list; the value goes straight to the keychain and is not returned. The hook command is split before saving, so a bad line is refused at once. Checked in the real window under Xvfb with GNOME Keyring on the session bus: a secret kept through the form appeared in `secret-tool` and `secret list`, and not in settings.json.

## v0.6.0 · profiles and start policies {#v0-6-0}

### lib.rs · set_start, profile_use, profile_save, profile_delete, profile_export, profile_import, profiles_of
- refers-to: src-tauri/src/lib.rs :: snapshot_now, profiles_of, set_start, profile_use, profile_save, profile_delete, profile_export, profile_import
- kind: decision
- logged: v0.6.0 · 2026-10-25

`snapshot` now separates an agent's own switch (`off_for`) from what its profile turns off (`profile_off_for`), gives each app's own start policy, and lists the profiles with counts and the agents using each. `profile_import` takes the file's text from the page and checks it against the bundled family list; `profile_export` writes to the Downloads folder and returns the path. Checked in the real window: FormPortal set to start with the session, HTTP started, and an HTTP session from outside started FormPortal's real server in the background (logged "with the session", 33 tools).

## v0.5.0 · HTTP inside the window, agents and tool switches {#v0-5-0}

### lib.rs · Http state, set_for, tool_list, tool_load, set_tool, http_switch, http_token, http_rotate
- refers-to: src-tauri/src/lib.rs :: Http, http_state_of, snapshot, set_for, tool_rows, tool_list, tool_load, set_tool, http_switch, http_token, http_rotate, run
- kind: decision
- logged: v0.5.0 · 2026-10-25

The shell manages one `Http` slot holding the running `http::Server`; `http_switch` starts it on the configured port or stops it, and it ends with the window, so HTTP never outlives the app (checked: the port closed when the real window was stopped). `tool_load` builds a throwaway `Gateway` off the main thread, starts the app once to record its tool names, and drops it, which ends the app's server. `http_token` returns the token only for the clipboard. `formmcp mcp` takes `--client` here as in the CLI.

Process note: these changes were first written as a whole new `lib.rs`; that file was restored from git and the same changes made again as region edits before anything was committed.

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
