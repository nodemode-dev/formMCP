# HASHOUT_LOG · demo · formMCP

Living log for the offline demo. Source files stay comment-free; what they would have said lives here.

Current build version: v0.8.1
Last updated: 2026-11-07 · real media: icons, screenshot reel and Lab flows; before that the offline demo: the real window on a Tauri shim, and the Lab

## v0.8.1 · real media {#v0-8-1}

### media.ts · icons and the reel without a flash
- refers-to: demo/src/media.ts :: decorateCards, buildReel, failsafe
- kind: decision
- logged: v0.8.1 · 2026-11-07

main.ts builds every card and removes `booting` in one task. media.ts runs after it, puts `booting` back in the same task (no frame is painted between), adds the icon to each card head and builds the reel, then waits for every image to decode (with a 1.5 s failsafe) before removing `booting` again. The first visible frame therefore already holds the icons and screenshots. Icon slots have fixed width and height, the reel frames a fixed 16 by 10 box, so nothing moves. Pictures are plain relative files: they are only displayed, never read for pixels, so file:// is fine. Apps without an icon under `assets/app-icon` in a sibling repository get a two-letter tile.

### flows.ts · task flows in the Lab
- refers-to: demo/src/flows.ts :: FLOWS, execute, run, swap
- kind: decision
- logged: v0.8.1 · 2026-11-07

Six step rows are made once; a flow change or a run only changes their text and `data-state`. The screenshot is swapped only after the next one has decoded, in a fixed 16 by 10 frame. The first step calls the demo's real `suggest` port; app steps answer from `sampleTools` and a written simulated result, honour the agent's off switches and hidden tools, and pass through the same redaction. Tool names for FormCut, FormWav, FormEncoder and FormVis are the ones those apps publish; arguments and results are invented for the demo and marked simulated. The Authorization shape in the last trailer step is assembled from pieces at run time, as the Redact sample is.

### Not done
- Twelve apps have no icon in a sibling repository's `assets/app-icon`, so they show a monogram, not artwork.
- Checked in Chromium over file:// only.

## v0.8.0 · the offline demo {#v0-8-0}

### main.ts, shim.ts · the real window on a shim
- refers-to: demo/src/main.ts :: import order · demo/src/shim.ts :: handlers, invoke
- kind: decision
- logged: v0.8.0 · 2026-11-07

`frontend/src/main.ts` decides at load whether `window.__TAURI__.core.invoke` exists. ES module imports run in order, so the shim is imported first and installs it; the real module then builds the full window. The shim implements every command main.ts calls and keeps settings in memory, mirrored to local storage inside try and catch. Secret values are never stored; only the names are, and the values live in memory for hiding. A secret kept before a reload still lists as kept but its value is no longer known to the hider.

### index.html · a copy of the window markup
- refers-to: demo/src/index.html :: #demo-strip
- kind: decision
- logged: v0.8.0 · 2026-11-07

main.ts needs the window's element ids, so the markup is a copy of `frontend/index.html` with relative paths, a title for the demo and a fixed 36 px strip above the header holding the Lab button, present in the first frame. The styles come from the real tokens, layers and app.css, imported from main.ts so Vite emits one linked stylesheet in the head: with the IIFE format, per-chunk CSS was injected by the script instead, which would have shown the page unstyled before `booting` was removed.

### gateway.ts · ports of the engine
- refers-to: demo/src/gateway.ts :: suggest, score, close, words, redact, readProfile, writeProfile, familyApps
- kind: decision
- logged: v0.8.0 · 2026-11-07

The word lists, stop words, scoring (5 for a name or id, 2 for the kind, 1 for a close word of five letters), the sort (score, released first, id) and the filter for apps off for the agent follow `core/src/suggest.rs`. The secret shapes and the mark follow `redact.rs`; JavaScript has no inline case flag, so only the last shape carries `i`. The profile reader follows `settings.rs::read_profile`: format, version 1, name rule, unknown apps and tools dropped and named. The next step text is the engine's with nothing installed.

### lab.ts · the Lab drawer
- refers-to: demo/src/lab.ts :: build, run, scan, take · demo/src/demo.css :: .lab, .lab-results, .lab-out
- kind: decision
- logged: v0.8.0 · 2026-11-07

A fixed drawer on the overlay token, hidden until opened, so it moves nothing in the window. Suggest has five result slots made once and updated in place, in a list of fixed height that scrolls. The Console output and Redact output are fixed-height scrolling blocks. A dropped profile is handed to the window's own import by setting the files of its hidden input and dispatching change, so the demo uses the real import path and messages.

### vite.demo.config.ts · building into the demo folder
- refers-to: frontend/vite.demo.config.ts · frontend/package.json :: build:demo
- kind: decision
- logged: v0.8.0 · 2026-11-07

Root is `demo/src` and the output is its parent `demo/`, so `demo/index.html` is the file to open; `emptyOutDir` is false and Vite's warning about that is expected. Classic deferred script instead of a module, because Chrome refuses module scripts from file URLs. No public dir: the icon is imported by the page and emitted with a hash. Host off, port 5620, preview 5621.

### Edit discipline note
- kind: decision
- logged: v0.8.0 · 2026-11-07

One new, still uncommitted file of the demo (`demo/src/index.html`) was derived from `frontend/index.html` and then had its stylesheet links removed by a small script run once, before it had been read back or committed. Every other change to existing files was made with the Edit tool. Checked headless in Chromium over file://: no console errors, 24 cards, suggest and console and redact run, a profile saves. Not checked in Firefox or Safari, and not at phone width.
