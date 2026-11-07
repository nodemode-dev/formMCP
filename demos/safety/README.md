# formMCP demo 3: safety and activity

An offline, browser-runnable look at what formMCP hides, checks and records on every agent call. Open `demos/safety/index.html` by double-clicking it (file://), or serve the folder with any static server. No backend, no Tauri, no network and no CDN. The three demos link to each other from the strip at the top: [demo 1](../../demo/index.html), [demo 2](../connect/index.html) and this one.

## What it shows

The page is the real formMCP window with its Safety panel open (secrets, hide-secrets and file-check switches, before and after hooks, tracing address) and an Activity log already filled by a recorded session of three agents. Below the panel is the bench, with six tabs.

1. Replay. Twelve recorded calls from Claude Code, Codex and Cursor, each shown with the outcome it had when it was recorded and the outcome it would have now. Change the window's switches, the before hook or a file on the Files tab and the Now column updates at once. Pick a row to see the engine's order of work step by step (agent and app, tool switch, before-call hook, file check and start, the app runs the tool, after-call hook, hide secrets, log and trace) and, side by side, what the app sent and what the agent sees. Open or drop a `log.jsonl` (an exported log) to replay your own calls, export the demo log, or add the replay to the Activity log.
2. Try a call. Build a call (agent, call, app, tool, arguments, and what the app answers) or pick a preset: a result holding an API key, a header with a token, a delete, a path in a key folder, an app whose files changed, an unknown tool. The call goes through the same pipeline and lands in the Activity log and the tracing collector.
3. Hooks. The six hook scripts the demo models, with their source, and a runner: the event the hook gets on stdin, its exit code, stdout and stderr, and whether formMCP allows or refuses (a JSON decision, exit code 2, text that is not JSON, a hook that does not finish in 10 s). Use one as the window's before or after hook.
4. Tracing. Send spans to a simulated collector at `127.0.0.1:4318`, to an address where nothing listens (one warning in the Activity log, calls unaffected) or turn tracing off. Read the OTLP trace and metrics bodies that were sent (names and outcomes only) and save them.
5. Files. Nine installed apps and the file check: change a file in one and a call to it is refused with the engine's wording; Record again in the Safety panel accepts it, Restore undoes it.
6. Asking the person. formMCP has no consent dialog of its own. This tab lists the real equivalents (two-click confirms, the before-call hook, switches, the file check, the keychain) and quotes the agent-side rule rather than simulating it.

## What is real and what is simulated

Real: the window and the family list, and logic ported to TypeScript from `core/src`: `redact.rs` (secret shapes and kept values), `hooks.rs` (command splitting, the allow and deny protocol, exit code 2, the 10 s limit), `telemetry.rs` (OTLP trace and metrics bodies, the address check, one warning per run), `verify.rs` (FormPortal's files record, formMCP's baseline, the refusal texts), `mcp.rs` (the order of work for a call) and `log.rs` (event shapes).

Simulated and labelled: the machine (which apps are installed, with their file records), app results, hook scripts (a browser cannot run commands, so named scripts are modelled in TypeScript and unknown ones fail as a missing file would), and the collector. Secrets and tokens are built from pieces at run time and are not real credentials. The page keeps its state in memory only and does not touch the settings demo 1 keeps in local storage.

## Rebuild

```
cd frontend
npm ci
cd ../demos/safety
node ../../frontend/node_modules/typescript/bin/tsc --noEmit -p .
node ../../frontend/node_modules/vite/bin/vite.js build --config ./vite.config.mjs
```

The build writes `index.html` and `assets/` next to `src/`, with relative paths and a classic script so file:// works. Vite warns that the output folder is a parent of its root; the build never empties it. `node ../../frontend/node_modules/vite/bin/vite.js --config ./vite.config.mjs` serves the sources at `http://localhost:5623`.

Media, source and licences: [`CREDITS.md`](CREDITS.md). Rationale: [`HASHOUT_LOG.md`](HASHOUT_LOG.md).
