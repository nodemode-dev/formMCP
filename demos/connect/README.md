# formMCP demo 2: connecting agents

An offline, browser-runnable look at how formMCP connects Claude Code, Codex and Cursor. Open `demos/connect/index.html` by double-clicking it (file://), or serve the folder with any static server. No backend, no Tauri, no network and no CDN. The three demos link to each other from the strip at the top: [demo 1](../../demo/index.html), this one, and [demo 3](../safety/index.html).

## What it shows

The page is the real formMCP window (`frontend/src/main.ts`, the real tokens, layers and `app.css`) on the real family list, with one new section, the workbench, above the Activity log. It has three tabs.

1. Agent setup. Pick Claude Code, Codex or Cursor. The left side is the agent's file as it is on disk in a virtual home folder (`~/.codex/config.toml`, `~/.cursor/mcp.json`; for Claude Code the two commands formMCP would run). The right side is what pressing Connect or Disconnect would change, with added and removed lines marked, the backup formMCP takes first, and the launcher and shell settings that are copied into the entry. Edit the file, open or drop your own, save the result, or start from the cases formMCP handles: no file yet, an empty file, another server already named formmcp, and invalid JSON. The window's own Connect buttons act on the same virtual home.
2. HTTP and token. The address, the access token (hidden until shown) and a request builder (method, Host, Origin, token, protocol version, session id, agent name, body). Presets for each refusal: wrong or previous token (401), a web page's Origin or a wrong Host (403), GET (405), an old protocol version (400), no session (404). Refusals and calls land in the window's Activity log, and New token in the window invalidates the old one at once.
3. What each agent sees. A matrix of apps by agent (on, off for this agent, off by profile name, some tools off) and an "ask as this agent" form that runs `family_apps`, `suggest_app`, `app_tools` and `call_app`, so a profile's refusals read as the real server words them.

It opens on a finished state: Claude Code and Codex connected, Cursor not, HTTP serving with a token, two profiles in use ("Video crew" and "Read only review"), and an Activity log of earlier sessions from the three agents over stdio and HTTP, including two refused HTTP requests.

## What is real and what is simulated

Real: the window, the family list, and logic ported line by line to TypeScript from the Rust engine: `clients.rs` (state, write, remove, TOML section edit, backup, quoting), `http.rs` (path, Host, Origin, token, protocol version and session rules, status codes), `settings.rs` (profile resolution, switches), `mcp.rs` (initialize, tool list, refusal texts) and `suggest.rs` (through demo 1's gateway). The Codex and Cursor results were compared byte for byte with the compiled `clients.rs` on five starting files (see the sidecar `DEMOS23_VERIFY.md`).

Simulated and labelled: no agent exists in a browser, so the files are a virtual home folder; Claude Code's own store is not modelled; app tool results are demo 1's sample tools; four apps are shown as installed. Nothing is written outside the page, and the page keeps its state in memory only (it does not touch the settings demo 1 keeps in local storage).

## Rebuild

```
cd frontend
npm ci
cd ../demos/connect
node ../../frontend/node_modules/typescript/bin/tsc --noEmit -p .
node ../../frontend/node_modules/vite/bin/vite.js build --config ./vite.config.mjs
```

The build writes `index.html` and `assets/` next to `src/`, with relative paths and a classic script so file:// works. Vite warns that the output folder is a parent of its root; the build never empties it. `node ../../frontend/node_modules/vite/bin/vite.js --config ./vite.config.mjs` serves the sources at `http://localhost:5622`.

Media, source and licences: [`CREDITS.md`](CREDITS.md). Rationale: [`HASHOUT_LOG.md`](HASHOUT_LOG.md).
