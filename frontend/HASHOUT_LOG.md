# HASHOUT_LOG · frontend · formMCP

Living log for formMCP's window. Source files stay comment-free; what they would have said lives here.

Current build version: v0.8.4
Last updated: 2026-11-07 · v0.8.4 no window changes (demos 2 and 3 in demos/); the splash page and the ready call; 2026-10-25 · the log names FormPortal's file check; the Safety panel; profiles and start selects; the HTTP bar, apps per agent and the Tools panel; no Skills filter or badge; clients, local state, switches and activity; the family list window

## v0.8.2 · the splash page and the ready call {#v0-8-2}

### public/splash.html, src/main.ts · the card and app_ready
- refers-to: frontend/public/splash.html; src/main.ts :: end of module (booting removal, app_ready)
- kind: decision
- logged: v0.8.2 · 2026-11-07

splash.html is formvis's card with FormMCP's name, icon (formmcp.png) and done hook (`formmcpSplashDone`): a transparent html and a body that is the card (16 px radius, 1 px edge, #0a0a0a), the 72 px icon, one line, and the 180 by 3 px family-gradient bar on the 2.6 s curve. main.ts invokes `app_ready` three animation frames after the `booting` class comes off, so the shell reveals the window once the first real layout is painted; in a browser there is no invoke and nothing happens.

## v0.7.1 · the log names FormPortal's file check {#v0-7-1}

### main.ts · describe
- refers-to: src/main.ts :: describe (app-start)
- kind: decision
- logged: v0.7.1 · 2026-10-25

An app start checked against FormPortal's record reads "files as FormPortal installed them". Text only, in a row that clips to one line, so nothing moves; typecheck and build passed, the window was not run again for it.

## v0.7.0 · the Safety panel {#v0-7-0}

### index.html, main.ts, app.css · Safety
- refers-to: index.html :: #safety-toggle, #safety, #safety-note, #secret-form, #secret-app, #secret-name, #secret-value, #secret-save, #redact-switch, #verify-switch, #hook-form, #hook-before, #hook-after, #hook-save, #trace-form, #trace-address, #trace-save, #safety-list · src/main.ts :: Safety, SecretRow, showSafety, safetyCall, loadSafety, closeSafety, twoClick, appName, describe · src/styles/app.css :: .safety, .safety-body, .safety-row, .safety-form, .safety-switches, .safety-list
- kind: fix
- logged: v0.7.0 · 2026-10-25

Safety sits beside Profiles and opens a panel of fixed height (212 px, 148 px in windows under 640 px tall) whose body scrolls; opening one closes the other. The secret value is a password field, cleared after Keep, and never read back: the panel shows names and whether each is in the keychain. The first throttled run measured CLS 0.0014: the kept secrets list sat under the Secrets row and its new row pushed the rows below about 2.5 s after the click. Then 0.0005 with the lists moved to the end, because the secrets list sat above the recorded installs. Now one list at the end of the panel, recorded installs first and secrets last, so a new secret only appends: CLS 0 at dark 1040 by 780 and light 640 by 520. Record again and Remove take a second click within four seconds. The log describes hook refusals, refused starts (stage verify) and how many secrets a call hid.

## v0.6.0 · profiles and start selects {#v0-6-0}

### index.html, main.ts · the Profile picker and panel
- refers-to: index.html :: #profile-pick, #profiles-toggle, #profiles, #profile-form, #profile-name, #profile-save, #profile-import, #profile-file, #profile-note, #profile-list · src/main.ts :: showProfiles, profileAction, profileNameOk, showAgent, START_LABELS, describe
- kind: decision
- logged: v0.6.0 · 2026-10-25

The Profile picker follows the Apps for picker: it shows and sets the chosen agent's profile. The panel is a fixed 168 px section between the filter row and the cards, opened only by its button, so nothing moves while its content changes. Import reads the chosen file in the page (no path is sent to the shell, so no file dialog plugin is needed) and sends its text. Delete wants a second click within 4 s. A switch an agent's profile turns off shows off, locked, with the profile named in its title. The start select sits with the card's switches.

### app.css · four layout fixes found by the checks
- refers-to: src/styles/app.css :: .profile-row .btn, #profile-save, .app-switches, .clients, .http, .status, .tools, .profiles, .log
- kind: fix
- logged: v0.6.0 · 2026-10-25

The global `.btn` places buttons in grid column 2 (for the agent cards), so the profile row's Export and Delete landed on each other; the row resets it. The Save button's label names the agent, and its width changing moved Import (CLS 0.0002); it has a fixed width. At 640 by 520 with the panel open the fixed rows shrank and the status line slid under the HTTP bar; they no longer shrink, and the activity panel gives way to 72 px. In the real window (WebKitGTK) a card's wrapped switch row was not counted in its height, so Always show tools hung over the card's edge (Chromium did count it); the switches now stack in a grid. The filter row wraps rather than scroll sideways.

## v0.5.0 · the HTTP bar, apps per agent and the Tools panel {#v0-5-0}

### index.html, main.ts · fixed places for new controls
- refers-to: index.html :: .http, #http-state, #http-copy, #http-rotate, #http-toggle, #agent · src/main.ts :: row, makeSwitch, showAgent, showHttp, showSnapshot, renderTools, describe, AGENTS
- kind: decision
- logged: v0.5.0 · 2026-10-25

The HTTP bar is a fixed 40 px row under the agent cards and the agent picker sits in the filter row, both in the first frame. Every card gets an On for switch (all apps, since an app without a server is still listed and suggested); its label has a minimum width so naming another agent does not move the switch beside it. Cards with an MCP server move Always show tools beside it and gain a Tools button. Copy setup puts the address, the header and a Claude Code command on the clipboard (a hidden textarea when the clipboard API is refused) and never shows the token on screen; New token wants a second click within 4 s, as Discard does in formskills. The activity log names the new events; before this they read as a failed call.

### app.css · the open Tools panel grows its own card
- refers-to: src/styles/app.css :: .apps, .app, .tool-panel
- kind: fix
- logged: v0.5.0 · 2026-10-25

The panel has a fixed height (168 px) and appears on the click, so its content arriving moves nothing. At 640 by 520 the first run found the open panel drawn over the next card: the list has a definite height (it scrolls), so CSS grid leaves no free space and keeps auto rows at their base size, which the card's `min-height` (196 px) sets, instead of growing them to the content. `grid-auto-rows: max-content` sizes each row to its card; `align-items: start` keeps the cards beside an open one at their own height. Checked at 640 by 520 and 1040 by 780, dark and light: CLS 0.

## v0.4.1 · no Skills filter or badge {#v0-4-1}

### index.html, main.ts, app.css · two chips, one badge
- refers-to: index.html :: .chips · src/main.ts :: Filter, row, apply · src/styles/app.css :: .badge-skills (removed)
- kind: decision
- logged: v0.4.1 · 2026-10-24

Skills belong to formskills (owner), so the window shows All and MCP server only, each card carries the MCP badge and its state, and the count line reads "N apps · M with an MCP server". The v0.1.0 entries below that mention the Skills chip and badge describe the earlier window. Measured again in Chromium at 4x CPU and a slow network, with the stubbed bridge and without it, dark and light: CLS 0, no page error.

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

### package.json · vite 8.3.5 clears the dev server's launch-editor advisories
- refers-to: frontend/package.json :: devDependencies.vite · frontend/package-lock.json
- kind: fix
- logged: v0.7.2 · 2026-10-25

Family rule from `FAMILY_DEV_PLAN.md` v0.4.x Phase 2b (formai_dev sidecar): every frontend installs with zero `npm audit` findings. Vite 8.3.2 carries a `launch-editor` with two command-injection advisories in the dev server (ledger FB-004); 8.3.5 clears them. Dev dependency only, nothing in the shipped bundle changed. Verified after `npm install`: `npm audit` reports 0 vulnerabilities, `npm run typecheck` passes, `vite build` succeeds (outDir outside the repo), and the dev server answers 200 on `http://localhost:5620/` with no Network address printed.
