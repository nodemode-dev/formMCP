# HASHOUT_LOG · frontend · formMCP

Living log for formMCP's window. Source files stay comment-free; what they would have said lives here.

Current build version: v0.1.0
Last updated: 2026-10-24 · the family list window

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
