# HASHOUT_LOG · frontend · formMCP

Living log for formMCP's window. Source files stay comment-free; what they would have said lives here.

Current build version: v0.4.0
Last updated: 2026-10-24 · clients, local state, switches and activity; the family list window

## v0.4.0 · clients, local state, switches and activity {#v0-4-0}

### index.html and main.ts · every live value has its slot from the first frame
- refers-to: index.html :: .clients, #status, .log · src/main.ts :: row (app-local, switch), showSnapshot, showClients, setText · src/styles/app.css :: .client, .btn, .status, .app-local, .log, .source
- kind: decision
- logged: v0.4.0 · 2026-10-24

The first frame is still built synchronously from the bundled list, with the body hidden until it is complete. What arrives later (the snapshot, client states, activity) only fills slots that already have their size: each client card is 56 px with a one-line state and a button at least 96 px wide (so Connect and Disconnect take the same room); the status line is one line high; each app card has a local line of at least 24 px holding its install and reach text and, for apps with a server, the switch; the activity panel is 132 px with its own scroll; the footer's source text fills a flexible box aligned right. `setText` writes only when the text changes. A first measurement found one shift of 0.0007 when the source text changed length inside a right-aligned span; making the span fill the footer removed it. Measured in Chromium at 4x CPU and a slow network with a stubbed bridge (snapshot after 900 ms, log lines every 2 s), dark and light, with real clicks on Connect, a switch and the filters: CLS 0, no page error. In a browser with no bridge the status line says to open the app, the client states read "Not known here", and the switches stay off and disabled: CLS 0.

### main.ts · actions change only their own node
- refers-to: src/main.ts :: connect and switch handlers, refresh, pollLog
- kind: rationale
- logged: v0.4.0 · 2026-10-24

Connect and Disconnect disable their button while the change runs, then show the client's new state and a line in the status; a switch saves and shows what `settings.json` now holds, and the status says clients see it when they next start formMCP. The snapshot is read at start and again when the window regains focus, at most every 10 s. The activity log is polled every 2 s from the last offset; new rows are appended at the bottom, the panel follows them only when it was already at the bottom, failed calls are in the fault colour, and past 300 rows the oldest are dropped (inside the panel's own scroll).

## v0.1.0 · the family list window {#v0-1-0}

### index.html and main.ts · the first frame is the final layout
- refers-to: index.html :: html.booting · src/main.ts :: rows, apply · src/styles/app.css :: html.booting body
- kind: decision
- logged: v0.1.0 · 2026-10-24

The list is imported from `../../core/data/family.json` at build time (`fs.allow` names `../core/data`), the same bytes the engine compiles in, so there is nothing to fetch and nothing to wait for. `html` carries `booting` and the body is `visibility: hidden` until `main.ts` has created every row, written the version, the source line and the count, and only then removes the class; the page background is the theme's ground colour throughout, so there is no white or partial frame. Sampled every 150 ms in Chromium at 4x CPU and a slow network: hidden with 0 rows, then shown with all 21, never in between; CLS 0.

### main.ts · filters toggle, never rebuild
- refers-to: src/main.ts :: apply
- kind: decision
- logged: v0.1.0 · 2026-10-24

The three chips set `hidden` on the existing rows (only when it changes) and rewrite the one count line; `aria-pressed` marks the chosen chip. No row is created or removed after boot. Checked with real clicks under throttling: CLS 0 through All, MCP server, Skills and back.

### Rows and badges
- refers-to: src/main.ts :: row, badge · src/styles/app.css :: .app, .badge
- kind: rationale
- logged: v0.1.0 · 2026-10-24

A row is the name (not translated), the kind, the summary, and a badge line: MCP and Skills filled with the accent colours when the app has them, dashed and faint (with an `aria-label` saying "not yet") when it does not, and the state (Live in the success colour, Public, or Coming for prelaunch). Rows sit in an auto-fill grid with a 280 px minimum and a 132 px minimum height, so the badge line lines up across a row.

### styles · family tokens and layers
- refers-to: src/styles/tokens.css, src/styles/layers.css
- kind: rationale
- logged: v0.1.0 · 2026-10-24

Copied from FormTray (the family's newest shell, same owner and licence) so formMCP looks like the rest of the family, dark by default and light when the system is light. FormTray's per-app hue tokens are replaced by formMCP's accents, orange `--accent` and violet `--accent-2`, from the owner's icon. The layer scale is unchanged; v0.1.0 stacks nothing, so no `z-index` is used yet.
