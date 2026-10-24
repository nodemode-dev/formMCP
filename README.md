<p align="center"><img src="assets/formmcp_icon.png" width="160" alt="formMCP icon" /></p>

<h1 align="center">formMCP</h1>

<p align="center"><b>One MCP connection from Claude Code, Codex and Cursor to the whole Form family. Small on context, and it never installs anything.</b></p>

<p align="center">
  <img src="https://img.shields.io/badge/version-v0.1.0-f97316" alt="version v0.1.0" />
  <img src="https://img.shields.io/badge/MCP-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20Cursor-8b5cf6" alt="MCP for Claude Code, Codex and Cursor" />
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Rust-stable-B7410E" alt="Rust stable" />
  <img src="https://img.shields.io/badge/Node.js-20.19%2B-339933?logo=nodedotjs&logoColor=white" alt="Node.js 20.19+" />
  <img src="https://img.shields.io/badge/license-GPL--3.0-555555" alt="license GPL 3.0" />
</p>

formMCP is the one MCP server you add for the Form family of desktop apps (image, video, audio, layout, vector, notes, slides and the rest, by nodemode.ai). The agent sees every Form app in one short list (what each is for, whether it has an MCP server, whether it ships skills), whether or not any of them is installed. formMCP suggests; FormPortal installs.

**Stack:** Rust (`formmcp-core`, `formmcp-cli`) · Tauri 2 shell · vanilla TypeScript + Vite window with zero runtime dependencies · **Scale:** 21 Form apps from formcompiler's registry, 1 MCP tool, 3 agent clients

## Highlights

- **Cheap on context.** `family_apps` returns one line per app and no app's tools, so the whole family is about a thousand tokens (3.8 KB for 21 apps), not twenty apps' worth of tools.
- **Never hand-kept.** The family list is generated from formcompiler's registry (`./formmcp family --compiler PATH`) and records the formcompiler version and commit it came from.
- **Suggests, never installs.** There is no tool that installs, updates or removes an app; the server tells the agent that FormPortal does.
- **Headless first.** The engine and the MCP server need no window. The installed app answers `formmcp mcp` on stdio without opening one, so it is its own launcher.
- **A window that does not flicker.** One list, filterable by MCP server and skills, painted complete in its first frame.

## Quick start

Requirements: Rust (stable), Node 20.19+, and on Linux WebKitGTK 4.1 for the window (the installer prints the packages; the MCP server does not need them).

```bash
./formmcp_oc_install        # macOS / Linux
./formmcp                   # the window on localhost:5620
./formmcp mcp               # the MCP server on stdio
./formmcp cli apps          # the family list as JSON
```

On Windows run `formmcp_oc_install.bat`, then `formmcp.bat`.

This repository is already wired for agents: `.mcp.json` (Claude Code), `.codex/config.toml` (Codex) and `.cursor/mcp.json` (Cursor) all run `./formmcp mcp`.

## Where things live

| Path | Role |
| --- | --- |
| `core/` | The engine: the family list (`core/data/family.json`, generated) and the MCP server |
| `cli/` | `formmcp-cli apps`, `app`, `mcp`, `version` |
| `src-tauri/` | The shell: the window, and `formmcp mcp` with no window |
| `frontend/` | The window: tokens, layers, one list |
| `setup/family.py` | Generates `core/data/family.json` from formcompiler's registry |
| `assets/` | The owner's icon |
| `.docs/wiki/` | Getting Started, Architecture, Agents |

| Port | Service |
| --- | --- |
| 5620 | Vite dev server (localhost only) |
| 5621 | Vite preview (localhost only) |

**Releases.** formMCP builds nothing in this repository. After the version bump lands on main, push a branch `compile_vX.Y.Z` cut from that commit; CI runs on it, and formcompiler (`nodemode-ai/formcompiler`) builds the bundles and drafts the release `formmcp-vX.Y.Z` once formMCP is in its registry.

## Status

v0.1.0: the scaffold. `family_apps` over stdio from the bundled list, the command line, the window, and the agent wiring. Next (family plan, Phase 3): installed apps' own tools passed through on demand (v0.2.0), `suggest_app` (v0.3.0), and connected clients and per-app switches in the window (v0.4.0).

## License

formMCP is licensed under the GNU General Public License v3.0 (`LICENSE`, SPDX `GPL-3.0-only`), as ComfyUI and formpromo are. Copyright 2026 NodeAgentc, Inc. `NOTICE.md` lists the work it includes or depends on.

## Trademarks

Claude Code is a product of Anthropic, Codex of OpenAI and Cursor of Anysphere; they are named only as the agent clients formMCP connects to. formMCP is not affiliated with or endorsed by them.
