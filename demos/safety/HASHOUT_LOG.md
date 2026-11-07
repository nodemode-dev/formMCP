# HASHOUT_LOG · demos/safety · formMCP

Living log for demo 3. Source files stay comment-free; what they would have said lives here.

Current build version: v0.8.4
Last updated: 2026-11-07 · first release of the safety and activity demo

## v0.8.4 · safety and activity {#v0-8-4}

### main.ts, isolate.ts, scenario.ts · the real window with a recorded session
- refers-to: demos/safety/src/main.ts :: import order · src/isolate.ts · src/scenario.ts :: patched, seed
- kind: decision
- logged: v0.8.4 · 2026-11-07

Imports run in order: the real stylesheets, this demo's css, isolate (a throwing `localStorage` getter, so nothing is shared with demo 1), scenario, the real `frontend/src/main.ts`, then the bench. scenario.ts imports demo 1's shim and wraps `invoke`: `safety` and every command that changes it also return the `checked` list (the installs' file records, changed ones first) and tell the bench through a `safety-change` event; `verify_trust` clears an install's changed files; `snapshot` marks the nine installed apps. It then sets the Safety state the demo opens with (both hooks, tracing to the simulated collector, two secret names whose values are known to the hider, one start policy) and seeds the Activity log, synchronously, before the real window starts polling: it connects the clients, clears the entries that makes, runs the twelve recorded calls through the engine at back-dated times and keeps the label each had.

### engine.ts · the pipeline as the engine orders it
- refers-to: demos/safety/src/engine.ts :: evaluate, commit, otlp, exportSpan, fileProblem
- kind: decision
- logged: v0.8.4 · 2026-11-07

`evaluate` returns a trace: eight steps with a state and a note each, the outcome label the Activity log shows, what the app sent and what the agent sees, the log lines and the span. It follows `mcp.rs`: app off, tool off, before hook, start and file check, the app runs the tool, after hook, hiding, log and span. It never mutates the log, so the Replay can run it freely on every change; `commit` writes its lines (through the same redaction) and spans. Refusal wording is the engine's, including the profile name when a profile turned an app off.

### hooks.ts · named scripts, not a shell
- refers-to: demos/safety/src/hooks.ts :: split, runLine, judge, SCRIPTS
- kind: decision
- logged: v0.8.4 · 2026-11-07

`split` and `judge` are ports of the engine's command splitting and decision rules. A browser cannot run a command, so six named scripts (guard.py, read-only.py, no-deletes.sh, chatty.sh, slow.sh, audit.py) are modelled in TypeScript beside their source text, `echo`, `true` and `false` are understood, an interpreter given an unknown file fails the way a missing file does, and an unknown program fails to start. The page says so.

### bench.ts · six tabs, built once
- refers-to: demos/safety/src/bench.ts :: build, buildReplay, buildTry, buildHooks, buildTracing, buildFiles, buildAsking
- kind: decision
- logged: v0.8.4 · 2026-11-07

The window's own controls stay the source of truth: the bench changes settings by clicking the real switches and submitting the real hook and tracing forms, so the Safety panel, its note line and the Activity log react as in the app. Panes are built once; a repaint changes text and `data-` attributes in place. The Replay list, the step rows and the Files rows have fixed slots; the tool select in Try a call, the span list and a loaded log are rebuilt only on the person's own action. `booting` stays until the Safety panel has opened and read its state (and the icons decoded, 1.5 s failsafe), so the first frame is final. The Safety panel and the bench sit above the app list through CSS `order`; the markup stays the window's.

### Not done
- No real consent prompt exists in formMCP, so none is simulated (see the last tab).
- Hook scripts are modelled, not run.
- Checked in Chromium over file:// only.
