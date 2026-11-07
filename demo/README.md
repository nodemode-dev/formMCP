# formMCP demo

An offline, browser-runnable copy of the formMCP window. Open `demo/index.html` by double-clicking it (file://), or serve the folder with any static server. No backend, no Tauri, no network and no CDN: every script, style and image is in this folder.

## What it shows

The page is the real formMCP window, not a picture of it. `frontend/src/main.ts` and the real tokens, layers and `app.css` run unchanged, on the real family list (`core/data/family.json`, 24 apps). A small shim stands in for Tauri's `invoke`, so the window works the way the installed app does, with settings kept in memory (and in the browser's local storage when it is allowed).

It opens on the family's real pictures: every card carries its app's own icon, and a reel above the clients shows the verified demo screenshots of nine Form apps (FormCut, FormVis, FormWav, FormPDF, FormDeck, FormLit, form.md, FormEncoder, FormVFX), with the photographs, footage and recordings visible in them credited in `CREDITS.md`. Choose a tile and the Lab asks `suggest_app` for a task that app serves. Every file, its source and its licence is in `CREDITS.md`. A few apps have no icon in a sibling repository and show a plain two-letter tile.

In the window itself:

- Cards for every Form app, the MCP filter, and the On for switch per agent (Claude Code, Codex, Cursor, other agents).
- Always show tools, start policies, and the Tools panel with single tool switches (sample tools, labelled as simulated).
- Profiles: save the chosen agent's settings under a name, use, export (a download) and delete one, or import a profile file by choosing one or dropping it on the Lab.
- The Safety panel: hide secrets in results, file check, hooks, tracing address and keychain secret names.
- The HTTP bar (start, stop, copy setup, new token) and the Activity log, which records what you do.

In the Lab (button at the top right), a Flows tab and three operations ported to TypeScript from `core/src`. Flows replays three realistic tasks (a 30 second trailer, a podcast episode, a retouched poster) as the MCP calls an agent would make, one step at a time, with the app's real screenshot beside each step. The first call is the real `suggest_app` ranking; app results are simulated and labelled; a token in a reply is hidden by the Safety setting; switching an app off in the window makes the flow stop at its step, as the real server would refuse it. The three operations:

1. Suggest: the `suggest_app` ranking from `suggest.rs`, with its word lists and scoring. It skips apps switched off for the agent, so switch one off in the window and ask again.
2. Console: run `tools/list`, `family_apps`, `suggest_app`, `app_tools`, `call_app`, `app_resources` and `app_prompts` with JSON arguments and read the results in MCP's shape. Results pass through the same hiding as the real server when Safety has it on.
3. Redact: the secret shapes from `redact.rs`. Load the built-in sample (built from pieces at run time, not real credentials), paste text, or open or drop your own text file.

Drop any file on the Lab: a formMCP profile (JSON) is imported into the window, any other text file is scanned for secrets.

## What is simulated

No Form app runs in a browser. Every app reports "Not installed", and app tool lists and `call_app` results come from a small sample set, labelled simulated, that you can switch off in the Console (calls then fail the way an app that cannot start fails). The client Connect buttons only change the state shown. Nothing is written outside the page.

## Rebuild

```
cd frontend
npm ci
npm run build:demo
```

This type checks `demo/src` and runs Vite with `frontend/vite.demo.config.ts` (host off, port 5620), writing `demo/index.html` and `demo/assets/` with relative paths and a classic script so file:// works. Vite warns that the output folder is a parent of its root; the build never empties it.

`npx vite --config vite.demo.config.ts` serves the sources at `http://localhost:5620`; the pictures live in `demo/assets/media/` and load only from the built `demo/` folder, so open or serve that folder to see them.
