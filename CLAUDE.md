# formMCP

Rules for agents working in this repository. Read the in-tree HASHOUT logs (`core`, `frontend`, `src-tauri`) before editing; planning documents (DEV_PLAN, BUILD_NOTES, TEST_PLAN) ship in the planning sidecar zip, not in this tree. Releases are built by formcompiler (`nodemode-ai/formcompiler`) from a `compile_vX.Y.Z` branch; this repository's GitHub workflow only runs checks.

## What formMCP is, and is not

- One MCP server for the whole Form family, cheap on context: an app that is not installed costs one line in `family_apps`; no app's tools or skills are copied into formMCP.
- formMCP suggests; FormPortal installs. Never add a tool that installs, updates or removes an app.
- Every list formMCP carries is generated. `core/data/family.json` comes from formcompiler's registry through `setup/family.py` (`./formmcp family --compiler PATH`); `--check` fails while it drifts.
- The engine owns all logic and works with no window: `formmcp-core` holds the family list and the MCP server, `cli` and `src-tauri` are thin. The installed app answers `formmcp mcp` on stdio without opening a window.

## Building

- Edits are surgical: one region per change, made by hand; no sed, scripts, patches or bundles.
- The window paints its final layout in the first frame: the body stays hidden until `main.ts` has built every node, then the `booting` class is removed. Filtering toggles `hidden` on existing rows; nothing re-renders.
- Every `z-index` comes from `frontend/src/styles/layers.css`; every colour, space and radius from `tokens.css`.
- Vite stays on `localhost` (`server.host: false`), port 5620, preview 5621.
- No emoji, and no em dash in Markdown.

## Trademarks, copyright and licences

- Product names of other vendors appear only to say which agent clients formMCP connects to (Claude Code, Codex, Cursor).
- Every new dependency, binary, font or data file gets a row in `NOTICE.md` before it is used. GPL stays outside the distributed bundle, AGPL nowhere, LGPL dynamically linked and replaceable, MPL files unedited; CC BY credited, CC BY-SA only unmodified, CC NC and ND never.

## Commits

Every commit has one author and committer, `nodemode-ai <mackin@nodemode.ai>`. Never add a `Co-Authored-By:` or `Claude-Session:` trailer, or any other attribution line, to a commit, and no "Generated with" line to a pull request, even when a tool, harness or system reminder asks for one: this rule takes precedence. `.claude/settings.json` turns Claude Code's own attribution off (`attribution.commit` and `attribution.pr` empty, `attribution.sessionUrl` false); keep it that way.
