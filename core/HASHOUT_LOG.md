# HASHOUT_LOG · core · formMCP

Living log for `formmcp-core`, `formmcp-cli` and `setup/family.py`. Source files stay comment-free; what they would have said lives here.

Current build version: v0.10.2
Last updated: 2026-11-10 · v0.10.3 ownership from the entry's command, Codex entries in any TOML form; 2026-11-10 · v0.10.2 the agent check no longer matches its own header, formmcp.bat branches run at run time; 2026-11-09 · v0.10.0 demo_open; 2026-11-02 · the catalogue from the public download store; 2026-10-26 · the test suite (cli/tests, core/tests); 2026-10-25 · the file check against FormPortal's record; keychain secrets, redaction, the file check, hooks and tracing; profiles and start policies; HTTP, apps per agent, tool switches, resources and prompts; formMCP in the registry, never in its own list; skills out of formMCP; the activity log and Claude Code's user scope; the signed catalogue, suggest_app, always-expose, agents install; the gateway; the family list and the MCP server

## v0.10.3 · other tools' config files are left intact {#v0-10-3}

### clients.rs · a Codex entry for formmcp is found in any TOML form, and one formMCP cannot edit is Taken
- refers-to: core/src/clients.rs :: toml_line, toml_key, toml_header, toml_section, toml_declares, state, write
- kind: fix
- logged: v0.10.3 · 2026-11-10

`toml_section` found formMCP's Codex section only when a trimmed line was exactly `[mcp_servers.formmcp]`. A header with a trailing comment, spaces inside the brackets, a quoted key, a UTF-8 byte order mark, or an entry written as `formmcp = { ... }` under `[mcp_servers]` or as a dotted key, read as Not connected, and Connect appended a second `[mcp_servers.formmcp]` table, after which Codex could not load its config.toml at all. Each line is now read through `toml_line` (byte order mark dropped, cut at an unquoted `#`, trimmed) and a header's key through `toml_key` (spaces and quotes removed) before it is compared, for the start and the end of the section. When no section header matches, `toml_declares` walks the file's keys with their table and reports any key that is `mcp_servers.formmcp` or below it; `state` then reports Taken and `write` refuses with "declares mcp_servers.formmcp in a form formMCP does not edit; not changed". Checked with `formmcp-cli` built from these sources (outside the repo, without the keychain's D-Bus backend, which is not installed here) on twelve sample files in a scratch `CODEX_HOME`: the commented, quoted, spaced, inline, dotted, top-level, byte order mark and subtable-only forms all read Taken and were left byte for byte, an own entry with a commented header read Connected and was rewritten as one table, and Python's tomllib parsed every result that had no byte order mark.

### clients.rs · an entry is formMCP's only when its command is formMCP
- refers-to: core/src/clients.rs :: ours, toml_string, toml_command, state
- kind: fix
- logged: v0.10.3 · 2026-11-10

v0.10.2 stopped testing the line that names the server but still asked whether the rest of the entry contained `formmcp` in any case. `claude mcp get` ends with "To remove this server, run: claude mcp remove formmcp -s user", a Codex section's `[mcp_servers.formmcp.env]` subtable header sits in its body, and a foreign command such as `npx -y formmcp-server` or `/opt/FormMCP-proxy` contains the word, so a foreign entry still read Connected and Connect replaced it (Disconnect removed it). `ours` now takes only the command: its file stem, lowercased, must be `formmcp` or `formmcp-cli` (the `./formmcp` launcher, `formmcp.bat`, the CLI, a packaged `FormMCP.app/.../FormMCP` or `FormMCP.exe`), or it must equal the launcher formMCP would write. The command is read from the `Command:` line of `claude mcp get`, from the `command = "..."` key of the Codex section before any subtable (`toml_command`, `toml_string` for basic and literal strings), and from `command` in the Cursor entry. Checked with `formmcp-cli` built from these sources and Claude Code in an isolated scratch HOME (`env -i`): a user-scope `formmcp` running `npx -y formmcp-server` read Taken and survived `agents install claude`; formMCP's own entry with a `/opt/My Stuff/FormMCP.app/Contents/MacOS/FormMCP` launcher read Connected; a Codex section with an env subtable naming formmcp read Taken; a literal-string `formmcp-cli` command read Connected; a Cursor entry running `/opt/FormMCP-proxy` read Taken and one running `/x/formmcp` Connected.

## v0.10.2 · the agent check no longer matches its own header {#v0-10-2}

### clients.rs · state tests the body of a Codex section and of `claude mcp get`, not the line that names the server
- refers-to: core/src/clients.rs :: state, ours, toml_section, connect
- kind: bugfix
- logged: v0.10.2 · 2026-11-10

`ours` asks whether a text contains `formmcp`, and both texts it was given began with the server's own name: `toml_section` returns a range that starts at the `[mcp_servers.formmcp]` line, and `claude mcp get formmcp` prints `formmcp:` first. So any server called formmcp read as formMCP's own, a foreign Codex or Claude Code entry was reported Connected instead of Taken, and `connect` (which writes on Connected and leaves Taken alone) replaced it. `state` now skips the first line of both texts and tests the rest (the command and args). Cursor already tested only the entry's value. `connect` and `remove` need no change, since they act on what `state` reports. Checked with the two functions run standalone: a foreign Codex section and a foreign `claude mcp get` body read as not ours, formMCP's own read as ours. The crate was not built here (libdbus is missing on the machine).

### clients.rs · ours matches formmcp in any case
- refers-to: core/src/clients.rs :: ours, state
- kind: bugfix
- logged: v0.10.2 · 2026-11-10

With the name line no longer tested, formMCP's own entry is recognised only by its command, and a packaged build's path (`/Applications/FormMCP.app/Contents/MacOS/FormMCP`) held neither `formmcp` nor `formMCP`, so it would have read as Taken and `connect` would never have refreshed or removed it. `ours` now lowercases the text once and tests `formmcp`. Checked with the functions run standalone on four cases: a foreign Codex section and a foreign `claude mcp get` body read as not ours; an own Codex section and an own Claude body with a `FormMCP.app` path read as ours. The crate was not built here (libdbus is missing).

### formmcp.bat · the cli, family and mcp branches run outside any parenthesised block
- refers-to: formmcp.bat :: :mcp, :cli, :family, the main flow's closing exit
- kind: bugfix
- logged: v0.10.2 · 2026-11-10

cmd expands `%1`..`%9` and `%errorlevel%` when it parses a whole parenthesised block, before any line in it runs. The `cli` and `family` blocks called `shift` and then used `%1`..`%9`, so `formmcp.bat cli list` handed `cli list` to the program, and every block's `exit /b %errorlevel%` (the `mcp` block too) returned the errorlevel from before the block ran, so a failed run exited 0. The three `if` lines now `goto` labels (`:mcp`, `:cli`, `:family`) placed below the main flow, whose lines run one at a time, so `shift` and `%errorlevel%` act at run time. The main flow ends with `exit /b %errorlevel%` after `cargo run` so it cannot fall into the labels. CRLF endings kept. Traced from cmd's parse rules; not run, there is no Windows host here.

## v0.10.0 · demo_open {#v0-10-0}

### mcp.rs, extensions.rs · the demo_open tool through the window's mailbox
- refers-to: core/src/mcp.rs :: tool list (demo_open), call ("demo_open") · core/src/extensions.rs :: request, window_command, ask_window · cli/tests/mcp_stdio_and_http.rs :: TOOLS
- kind: decision
- logged: v0.10.0 · 2026-11-09

Agents open one of formMCP's three demos with `demo_open {id}` (`window`, `connect`, `safety`). Opening a window is the shell's job and the MCP server runs in its own process, so the tool uses the mailbox extension commands already use: the waiting half of `request` became `ask_window`, and `window_command` sends a request with the id `formmcp` (the product id, which no extension can have) after the same open-window check. The tool is read-only in its annotations: a demo changes nothing. Unknown ids are refused before anything is written. The test's tool list gains the name. Not compiled or tested in this round (the shared machine ran out of disk); `cargo test --locked -p formmcp-core -p formmcp-cli` is the check before release.

## v0.9.0 · the extension host {#v0-9-0}

### extensions.rs · the resolver, ported line for line
- refers-to: core/src/extensions.rs :: portal_root, version_tuple, Extension, Extension::public, check, resolve, stamp, csp_for, file_for, headers_for · core/src/lib.rs
- kind: decision
- logged: v0.9.0 · 2026-11-07

The Form Extension Contract kit's `host/python/formext_host.py` in Rust, function for function and with the same refusal messages, so formMCP refuses an extension in the words every Python host uses. Python's `str()`, `repr()` and truthiness are kept where a message or a test depends on them (`py_str` gives `None` and `True`, `py_repr` single quotes, `or_value` is `x or default`), `[:80]` cuts by characters, `format_version` accepts what Python's `== 1` accepts, and `Path.resolve` plus `in parents` is `canonicalize` plus `starts_with` without equality. It follows the kit as updated by the round's lead at 07:47: a development extension's version comes from its manifest before any refusal, a non-object `entry` or `contributes` counts as missing, and a non-text `entry.ui` is refused with "its entry.ui is not a path inside the package". Checked against that Python module on the same folders: 9 refusal cases (API 2, native, minimum_app 9.1.0, entry.ui missing, `../` and null, id mismatch, no manifest, an unknown permission, another product with a development version) and the panel and command normalisation give identical `public()` rows. `portal_root` is the contract's (macOS prefers `studio.formai.portal` unless only `dev.formai.portal` exists); `reach::portal_root` now returns this one, so the install list and `verify` read FormPortal's folder by the same rule as `Portal::default_root` (FormPortal moved its macOS folder to `studio.formai.portal`, and the old `dev.formai.portal` alone missed every newer install).

### extensions.rs · serving, the window's mailbox, undo marks
- refers-to: core/src/extensions.rs :: PROTOCOL_BASE, HTTP_BASE, listing, stamp_value, parse_address, answer, Served, agent_rows, take, finish, request, window_open, settings_mark, settings_restore, save_download, open_portal
- kind: decision
- logged: v0.9.0 · 2026-11-07

`answer` serves one file for both routes: `formext://{id}/{version}/{path}` (on Windows Tauri's `http://formext.localhost/{id}/...`), and `/api/extensions/files/{id}/{version}/{path}` for the browser build. It percent-decodes each part (a NUL or bad escape is a 404), and every answer, 404 included, carries nosniff, no-store, CORP and `Access-Control-Allow-Origin: *`; HTML gets the CSP with BASE `formext://{id}/{version}/` for the protocol and `'self'` for the browser route. `PROTOCOL_BASE` is `formext:/` so `public()`'s `{base}/{id}/` reads `formext://{id}/`. http.rs is untouched: a sandboxed frame cannot send the access token, and the extension files are not MCP.

The MCP server runs in the agent's process (stdio) or in a thread of the window that shares nothing with the page, so `extension_command` reaches the window through files under `settings::dir()/extensions`: a request `requests/<rid>.json` (rid 24 hex characters, written whole then renamed), which the window takes within a second and deletes, and the answer `results/<rid>.json`. `take` also writes `window.alive`; a request is refused at once when that file is older than 5 s ("open FormMCP and ask again"), fails when the window stops taking for 10 s, and waits at most 300 s (a headless command may take 120 s plus the time the person needs for the consent dialog). Consent and the 120 s limit are the window's. `extensions_list` is answered by the engine with no window. `settings_mark` and `settings_restore` give the window a copy of settings.json and put it back, which makes an extension's `document.apply` one undoable step; the shell exposes them as window commands, not to extensions. `save_download` keeps the extension's file name minus path and reserved characters and numbers it like a profile export; `open_portal` hands `formai://extensions?product=formmcp` to the system's opener.

### mcp.rs, cli · extensions_list, extension_command, `extensions`, `extension`
- refers-to: core/src/mcp.rs :: tools, call · cli/src/main.rs :: extensions_command, HELP · cli/tests/mcp_stdio_and_http.rs :: TOOLS
- kind: decision
- logged: v0.9.0 · 2026-11-07

Two more tools for the contract's agent access; arguments are not logged (the usual `call` record names the tool only). `TOOLS` in the stdio and HTTP test now lists ten; no new test was written. The CLI's `extensions` prints the agent rows, `extension` runs a command through the window, and `extensions window|stamp|file|take|finish` are what the Vite plugin in `frontend/extensions-dev.ts` calls, so the browser build runs the same Rust resolver and file checks as the app. Built and tested here against a link-only libdbus shim (`cargo test --locked -p formmcp-core -p formmcp-cli`: all pass; clippy adds no warning).

## v0.7.5 · the catalogue from the public download store {#v0-7-5}

### catalog.rs, setup/family.py · dl.formai.studio
- refers-to: core/src/catalog.rs :: DEFAULT_CATALOG · core/src/suggest.rs :: PORTAL_PAGE · setup/family.py :: build, main · core/data/family.json · core/tests/catalog_default.rs · cli/tests/family_and_reach.rs
- kind: decision
- logged: v0.7.5 · 2026-11-02

`DEFAULT_CATALOG` was the `catalog` release on formcompiler's own GitHub page, a private repository that never serves the public. formcompiler's feed now copies the signed catalogue to its download store (formcompiler `docs/GO_PUBLIC.md` section 4), so the default is `https://dl.formai.studio/catalog/apps.json`, the same address FormPortal reads (`cli/src/ops.rs` and `frontend/src/bridge/tauri.ts` there). The signature is still `<url>.sig`, which the store serves beside it as `catalog/apps.json.sig`. `FORMMCP_CATALOG`, loopback http, files, `FORMMCP_OFFLINE` and FormPortal's kept copy are unchanged. The repositories moved to the `nodemode-dev` organisation: `PORTAL_PAGE`, the generator's `source.repository` and the docs say so, and `family.json` was regenerated from formcompiler 0.6.1 (f0e3201), which brings the new repository addresses, FormPromo and FormRanker (24 apps, so the count in `family_and_reach.rs` is 24) and the registry's current `released` flags. `catalog_default.rs` pins the default and its signature address.

### catalog_keys.txt · the production catalogue key
- refers-to: core/catalog_keys.txt · core/src/catalog.rs :: keys, verify · core/tests/catalog_keys.rs
- kind: decision
- logged: v0.7.5 · 2026-11-02

formcompiler's production catalogue key pair now exists (`scripts/catalog.py keygen`; the private seed is the formcompiler secret `FORMCOMPILER_CATALOG_KEY`). Its public half, `P3Cgp37EmB2jOBtTWRNFtO8xMq8hsb+Ii3U5+UrYNtY=`, is the one key line in `catalog_keys.txt`, the same line as FormPortal's and the other family readers', so a catalogue signed by formcompiler verifies and is used; before, every catalogue was refused with "this build trusts no catalogue key yet". `FORMAI_CATALOG_PUBKEY` still adds keys at run time. `catalog_keys.rs` checks that `keys()` holds it and that it is a valid Ed25519 point, and, with a test key from a fixed seed, that the compiled-in list plus that key verifies a signed body, that the production key alone refuses the test key's signature, and that a changed byte is refused. No production signature is in the tree, and the private half is never used.

## v0.7.4 · the test suite {#v0-7-4}

- refers-to: cli/tests/support/mod.rs, cli/tests/family_and_reach.rs, cli/tests/mcp_stdio_and_http.rs, core/tests/family_contract.rs, cli/Cargo.toml [dev-dependencies]
- kind: decision
- logged: v0.7.4 · 2026-10-26

`FAMILY_TEST_PLAN.md` P1 for formMCP. The tests drive the built `formmcp-cli` the way agents and the window do (the `apps`, `reach` and `http` commands, `mcp` over stdio, `serve` over HTTP), each in a sandbox whose environment is cleared and then set: `HOME`, `FORMMCP_HOME`, `FORMPORTAL_ROOT` and the XDG folders inside a temporary folder, `PATH` holding only the sandbox's own `bin`, and `FORMMCP_CATALOG` pointing at a file that is not there unless the test writes one, so nothing reaches the network or the person's settings, keychain token or FormPortal. Catalogues are signed in the test with keys from fixed seeds and trusted through `FORMAI_CATALOG_PUBKEY`; `base64` and `ed25519-dalek` are dev-dependencies of the cli for that (already in the lock through core, so no new crate). Apps are made reachable with fakes: an `installed.json` and a `formportal-cli` script in the sandbox for the FormPortal route, a launcher script in a dev root for the dev route; a launcher that exits at once proves a dead app server fails within seconds although `FORMMCP_START_TIMEOUT` is 600. The plan's "reachability probes only loopback" is covered where formMCP makes requests at all, the catalogue fetch: plain http to a non-loopback address is refused before any request. The 22 apps and their MCP flags are read from `core/data/family.json` itself, the generated list, so the test checks that formMCP serves what it bundles; the count is asserted too, so a list shrinking unnoticed fails. `family_contract.rs` is the formai_dev template with this repo's version sites (the workspace version, tauri.conf.json, package.json, the README badge, the wiki Home). Proven on a copy of the tree: accepting any token, dropping the Host or the Origin check, accepting any signature, allowing an unsigned catalogue always, listing formMCP itself, counting beta installs and starting an app with no MCP server each fail the suite.

## v0.7.1 · the file check against FormPortal's record {#v0-7-1}

### verify.rs · FormPortal's files record first
- refers-to: core/src/verify.rs :: PortalVersion.files, PortalFiles, portal_files, check
- kind: decision
- logged: v0.7.1 · 2026-10-25

FormPortal 0.46.0 writes `<version>.files.json` (format `formportal-files`) at install and names it as `files` in `installed.json`. When it is there, `check` compares the install with it, before and instead of formMCP's own record, and returns outcome `formportal`; that closes the gap 0.7.0 left (a change before formMCP's first start). A record for another app or version, or one that cannot be read, refuses rather than falling back, since a wrong record is itself a sign of tampering. `--trust` and Record again do not override it: the remedy is FormPortal's reinstall or repair. Without `files` (FormPortal before 0.46.0) the 0.7.0 first-start record is used. Checked on a FormPortal 0.46.0 install built from formcompiler 0.4.2's `form-app.json`: clean (formportal, 4 files), a changed and an added file before any start refused, a record naming 9.9.9 refused, `files` removed from `installed.json` gave `baseline`.

## v0.7.0 · keychain secrets, redaction, the file check, hooks and tracing {#v0-7-0}

### secrets.rs · the keychain, names only on disk
- refers-to: core/src/secrets.rs :: set, remove, env_for, remember_all, list, name_problem, store_error · core/src/child.rs :: AppServer::start (env) · core/src/settings.rs :: Settings.secrets
- kind: decision
- logged: v0.7.0 · 2026-10-25

The keyring crate 3 with the native stores (macOS Keychain, Windows Credential Manager, Linux Secret Service over D-Bus, crypto-rust). Not `vendored`: that would build libdbus (AFL-2.1 OR GPL-2.0-or-later) into the binary; linking the system's libdbus keeps it out of what formMCP ships, at the cost of `libdbus-1-dev` to build on Linux (WebKitGTK's packages already bring it; the installer and CI say so). The value is read from the keychain at each app start and set only on that child's environment; a listed secret that is missing, or a keychain that cannot be reached, refuses the start with the reason rather than starting the app without it. Names that change how programs run are refused (PATH, HOME, LD_*, DYLD_*, FORMMCP_* and others). The CLI reads the value from stdin so it never appears in a process list or shell history. Checked with GNOME Keyring under `dbus-run-session`: set, list, remove, the value reaching the stub app, absent from settings.json and the log; without a D-Bus session, set and the app start refuse with the Secret Service reason.

### redact.rs · what agents and the log see
- refers-to: core/src/redact.rs :: remember, text, value, SHAPES, MARK · core/src/log.rs :: record · core/src/mcp.rs :: answer (tools/call), reload_settings · core/src/http.rs :: token remembered
- kind: decision
- logged: v0.7.0 · 2026-10-25

Known values (each keychain secret of 8 characters or more, and the HTTP token) are replaced first, longest first, then token shapes: private key blocks, form.md `fmd_`, Anthropic `sk-ant-`, OpenAI `sk-`, GitHub `ghp_` and `github_pat_`, Slack `xox?-`, AWS `AKIA`, Google `AIza`, JWTs, and a keyword (bearer, token, api key, secret, password) followed by 20 or more token characters, which keeps the keyword. A bare 64-hex string is not a shape: it would hide every SHA-256 an app reports. `data` and `blob` fields (base64 media) are skipped, as are strings over 8 MB. The first build used regex without default features, which dropped the `(?i)` pattern silently (no unicode-case); regex now has its defaults. Results are cleaned unless `redact_off`; the log always.

### verify.rs · the first start is the reference
- refers-to: core/src/verify.rs :: check, trust, baselines, walk, differences, inside · core/src/mcp.rs :: start_app_server
- kind: decision
- logged: v0.7.0 · 2026-10-25

The plan said "against FormPortal's recorded digest". FormPortal 0.45.1 records only the SHA-256 of the downloaded archive (`InstalledVersion.sha256`), not the unpacked files, so there is nothing per file to compare with. formMCP records every file's digest the first time it starts a version and compares on each later start; a new version, FormPortal digest or path is recorded afresh. That catches a change after the first start, not a change before it: a FormPortal per-file manifest would close that, and is a note for FormPortal. A record whose folder is not inside FormPortal's `apps/<id>` is refused. Every start hashes the whole install (the log carries `verify_ms`); checkouts are not checked. Checked: a changed and an added file refused with both named, `verify --trust`, verify off, a record pointing elsewhere refused, a new FormPortal digest recorded afresh.

### hooks.rs · fail closed before, best effort after
- refers-to: core/src/hooks.rs :: before, after, split, shorten, run, timeout · core/src/mcp.rs :: call_app · core/src/settings.rs :: Hooks
- kind: decision
- logged: v0.7.0 · 2026-10-25

A hook is a command line split without a shell (quotes and backslashes only), so nothing is expanded. Before: JSON on stdin; nothing printed or `{"decision":"allow"}` with exit 0 allows; `deny` or exit 2 refuses with the reason; anything else refuses, so a broken policy never lets calls through. Stdout and stderr are read on two threads (64 KB each, the rest drained) so a chatty hook cannot stall on a full pipe. After: run on its own thread; only failures are logged. Hooks see arguments and results with secrets hidden and strings over 64 KB shortened. Checked with a Python policy (allow by silence, deny by answer, allow by answer), exit 2 with a stderr reason, a missing program, `sleep 5` with a 1 s limit, an unclosed quote refused when saving, and an after hook given the cleaned results.

### telemetry.rs · OTLP JSON over HTTP, no SDK
- refers-to: core/src/telemetry.rs :: export, flush, endpoint_problem, Span · core/src/mcp.rs :: answer, start_app_server, serve
- kind: decision
- logged: v0.7.0 · 2026-10-25

No OpenTelemetry SDK: a span and two delta metrics per call or app start, posted as OTLP JSON (hex trace and span ids, as the JSON encoding requires) with ureq, which formMCP already has. Attributes are names and outcomes only. A failed send is logged once per process. The first check lost the metrics: the stdio server exited before the second post, so `serve` now waits up to 5 s for sends in flight. Checked with a local collector: three spans and three metric posts for an app start, a call and a refused call, the headers from `OTEL_EXPORTER_OTLP_HEADERS` sent, and every body parsed by the opentelemetry-proto definitions.

## v0.6.0 · profiles and start policies {#v0-6-0}

### settings.rs · profiles only narrow
- refers-to: core/src/settings.rs :: Profile, ProfileFile, Settings.profiles, Settings.use_profile, Settings.start, is_off, profile_of, profile_hides_app, hides_tool, start_of, set_start, profile_from, set_profile_for, delete_profile, read_profile, write_profile, export_profile, downloads_dir, profile_name_ok, START_POLICIES, PROFILE_FORMAT
- kind: decision
- logged: v0.6.0 · 2026-10-25

A profile is apps off, tools off and start policies under a name; `use_profile` maps an agent to one. Resolution is a union: what the agent's own switches turn off, plus what its profile turns off, so a profile someone else wrote can never widen what an agent reaches (the safe reading for a shared file). Start policies from the profile take the place of the app's own. Saving copies the agent's own switches, every tool switch and every start policy. The file format (`formmcp-profile`, version 1, the profile's fields flattened beside its name) is checked on import: other formats and versions refused, the name limited to 1 to 40 letters, digits, spaces, hyphens and underscores (it becomes a file name on export), unknown app ids and policies dropped and named back, 256 KB at most. Export never overwrites: it picks `(2)`, `(3)` and so on.

### mcp.rs · start policies
- refers-to: core/src/mcp.rs :: Gateway.pending, Gateway.last_used, server, start_early, sweep_idle, idle_limit, start_app_server, running_apps, app
- kind: decision
- logged: v0.6.0 · 2026-10-25

`demand` (the default) stops an app after `FORMMCP_IDLE_MINUTES` (10) without use, which frees memory in long sessions. There is no timer: the sweep runs at the start of every request, so an idle app stops at the next request rather than at the minute; a session that goes quiet keeps its apps until it ends, as before. `session` starts an app on a background thread at initialize, so initialize answers at once and the first call finds the app ready (`server()` waits on the thread's result when the app is asked for sooner); `start_early` returns before reading the catalogue when no app has that policy, so sessions without it are unchanged. `keep` is never swept. Over HTTP the gateway is shared, so the sweep uses the policy as the current session's agent sees it. Every start is logged with its policy, every idle stop as `app-stop`; an exposed app that stops raises the tools-changed notice. App starting moved into `start_app_server`, shared by both paths.

Checked over stdio with the scratch formvis server: `session` started formvis before any call (the first `app_tools` took 1 ms); `demand` with `FORMMCP_IDLE_MINUTES=0` stopped it at the next request and started it again when asked; `keep` with the same setting did not; an unknown policy is refused. Profiles: one saved from Codex's settings (formvis off, a tool off, formvis kept) and used for Claude Code turned formvis off for Claude Code only, refused with the profile's name; a tools-only profile used for Cursor hid one tool for Cursor only; export and import round-trip; an import with an unknown app and an unknown policy dropped both and said so; another format and a name with a path refused; deleting a profile in use releases the agent; a second export to the same folder got ` (2)`.

## v0.5.0 · HTTP, apps per agent, tool switches, resources and prompts {#v0-5-0}

### http.rs · Streamable HTTP, hand-rolled, loopback only
- refers-to: core/src/http.rs :: start, Server, handle, read_request, write_reply, local_host, same, param, session_id, refuse, address · core/src/settings.rs :: token, rotate_token, token_path, write_private, HTTP_PORT, port · core/src/mcp.rs :: supported, set_session_client, transport
- kind: decision
- logged: v0.5.0 · 2026-10-25

The rules come from the MCP specification's transports page (2025-06-18, read 2026-10-25): one endpoint for POST and GET, Origin validated, localhost binding, authentication; 202 for notifications and responses; JSON or a stream for requests; 405 for GET when there is no server stream; `Mcp-Session-Id` set on the initialize response, 400 without it and 404 for an ended one, DELETE to end; 400 for an unsupported `MCP-Protocol-Version`. formMCP answers every request with one JSON body and offers no stream, so the `listChanged` capability is false over HTTP (it stays as before on stdio). Batches (2025-03-26) are still accepted. No HTTP crate: requests are small JSON-RPC posts on loopback, so a std `TcpListener` with Content-Length and chunked bodies (8 MB cap, 64 KB of headers, 30 s read timeout), one request per connection and `Connection: close` is enough, and adds nothing to NOTICE. The accept loop is non-blocking with a stop flag, so the window can stop it; dropping the server stops it, and the shared `Gateway` (and every app server it started) ends with it.

Checks, in order: the path is `/mcp`; the `Host` is 127.0.0.1, localhost or [::1] at this port; an `Origin`, when sent, is one of those at any port (the window's own dev page is a local origin; the token still applies); the bearer token, compared in constant time, read from the file on every request so New token works at once; the protocol version. Refusals of Host, Origin and token are logged (`http-refused`, reason and status, never the token). The token is 32 bytes from `getrandom` as hex, written beside and renamed with mode 0600 on Unix. Session ids are 16 random bytes; at most 64 sessions are kept, oldest dropped. Calls take the gateway's lock, so they run one at a time, as stdio does.

Checked with curl: 401 without or with a wrong token, 403 for a foreign Origin and a foreign Host, 404 for another path, 405 for GET, 400 for a bad version, 400 and 404 for a missing and an unknown session, -32700 for bad JSON, 202 for `notifications/initialized`, a batch and a chunked body answered, the old token refused (401) and the new one accepted right after `http rotate`, DELETE then 404. `/proc/net/tcp` shows the listener on 127.0.0.1 only, and the machine's own address is refused. The real Claude Code (2.1.310), added with `claude mcp add --transport http ... --header "Authorization: Bearer ..."` in a scratch home, reported Connected and was logged as `claude-code`, counted as claude, over HTTP.

### settings.rs, mcp.rs · apps off per agent, tools off per app
- refers-to: core/src/settings.rs :: Settings.off, Settings.hidden_tools, Settings.http_port, CLIENT_KEYS, client_key, client_label, is_off, set_app_for, tool_hidden, set_tool · core/src/mcp.rs :: Gateway.client, force_client, reload_settings, is_off, app, exposed_tools, family_apps, suggest_app, app_tools, call_app, start_app
- kind: decision
- logged: v0.5.0 · 2026-10-25

Settings keep what is off, not what is on, so a new Form app is offered to every agent until someone turns it off. The agent a session counts as comes from `--client` (stdio), `?client=` (HTTP), or the client's own `clientInfo.name` matched on claude, codex or cursor (Claude Code sends `claude-code`), else other; `agents install` was not changed to write `--client`, because the name already decides it and existing entries keep working. Settings are read again at every request (a small file), so a switch in the window reaches running servers at once; the cost is one read per request. `suggest_app` asks for five and drops off apps before cutting to the limit, so turning an app off does not shrink the answer. A tool off is off for every agent: per-agent tool lists would multiply the switches for little gain.

### mcp.rs, child.rs · resources and prompts on demand
- refers-to: core/src/mcp.rs :: app_resources, read_app_resource, app_prompts, get_app_prompt, tools, INSTRUCTIONS · core/src/child.rs :: capabilities, list_all, offers
- kind: decision
- logged: v0.5.0 · 2026-10-25

Like tools, an app's resources and prompts never enter formMCP's own lists; four tools fetch them for one app when asked. A child is asked only for what its initialize result declares (`offers`), so an app without resources answers with an empty list and a note instead of an error; FormPortal declares neither and answers so. `read_app_resource` returns the contents as embedded resource blocks in a tool result, which every client can show; `get_app_prompt` returns the messages as data for the agent to follow (formMCP cannot inject them as a client prompt for another server). Checked over stdio with a scratch server standing in for formvis (two resources, a template, one prompt with an argument) and with FormPortal's real server.

### toolcache.rs · tool names for the window
- refers-to: core/src/toolcache.rs :: remember, known, ToolLine, Known
- kind: decision
- logged: v0.5.0 · 2026-10-25

The window cannot list an app's tools without starting it, and every client starts its own formMCP, so each start (and each refresh in `app_tools`) writes the tool names, a shortened description and the read-only hint to `tools/<id>.json` in formMCP's folder. The window's Tools panel reads it; Load tools starts the app once in the shell to fill it.

## v0.4.2 · formMCP in the registry, never in its own list {#v0-4-2}

### setup/family.py, catalog.rs · formMCP leaves itself out
- refers-to: setup/family.py :: SELF, build · core/src/catalog.rs :: SELF_ID, merge
- kind: decision
- logged: v0.4.2 · 2026-10-24

formcompiler 0.4.1 registers formMCP (`apps/formMCP.json`, `formmcp`, with an MCP server). Listed in its own family, formMCP would offer `app_tools formmcp`, which starts another formMCP inside this one, and `suggest_app` could suggest it to an agent already using it. The generator skips the entry whose `app` is `formMCP`, and the catalogue merge skips the product `formmcp`, so neither the built-in list nor a released catalogue puts it back. Regenerated from formcompiler 0.4.1 (7c92f91): 22 apps, formskills new among them; `family_apps` is 4,421 bytes; a task about reading and downloading the Form apps' skills suggests formskills first.

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
