<p align="center"><img src="assets/formmcp_icon.png" width="160" alt="formMCP icon" /></p>

<h1 align="center">formMCP</h1>

<p align="center"><b>One MCP connection from Claude Code, Codex and Cursor to the whole Form family. Small on context, and it never installs anything.</b></p>

<p align="center">
  <img src="https://img.shields.io/badge/version-v0.4.2-f97316" alt="version v0.4.2" />
  <img src="https://img.shields.io/badge/MCP-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20Cursor-8b5cf6" alt="MCP for Claude Code, Codex and Cursor" />
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Rust-stable-B7410E" alt="Rust stable" />
  <img src="https://img.shields.io/badge/Node.js-20.19%2B-339933?logo=nodedotjs&logoColor=white" alt="Node.js 20.19+" />
  <img src="https://img.shields.io/badge/license-GPL--3.0-555555" alt="license GPL 3.0" />
</p>

formMCP is the one MCP server you add for the Form family of desktop apps (image, video, audio, layout, vector, notes, slides and the rest, by nodemode.ai). The agent sees every Form app in one short list (what each is for, whether it has an MCP server, whether this computer can reach it), whether or not any of them is installed, and loads one app's own tools only when the task needs that app. formMCP suggests; FormPortal installs.

**Stack:** Rust (`formmcp-core`, `formmcp-cli`) · Tauri 2 shell · vanilla TypeScript + Vite window with zero runtime dependencies · **Scale:** 22 Form apps from formcompiler's registry and the signed catalogue, 4 MCP tools, 2 ways to reach an app, 3 agent clients

## Highlights

- **Cheap on context.** `family_apps` returns one line per app and no app's tools, so the whole family is about a thousand tokens (4.4 KB for 22 apps with their install state), not twenty apps' worth of tools.
- **Installed apps' own tools, on demand.** `app_tools` starts one app's own MCP server (through its checkout's launcher, or `formportal app-mcp` for a FormPortal install) and lists its tools; `call_app` runs one and returns the app's result unchanged. Nothing is copied, so the tools are always the running app's.
- **The right app for the task.** `suggest_app` takes the task in the person's words and returns the apps that fit with the next step: load its tools, install it with FormPortal, or install FormPortal first.
- **Never hand-kept.** The family list is the signed family catalogue (checked against the same keys FormPortal trusts) over a list generated from formcompiler's registry (`./formmcp family --compiler PATH`), and every answer says which one was used and why.
- **Tools only; skills live in formskills.** formMCP lists apps and runs their MCP tools. The Form apps' released skills are viewed, downloaded and edited in formskills, not here.
- **Suggests, never installs.** There is no tool that installs, updates or removes an app; the server tells the agent that FormPortal does.
- **Headless first.** The engine and the MCP server need no window. The installed app answers `formmcp mcp` on stdio without opening one, so it is its own launcher.
- **A window that does not flicker.** Connect formMCP to Claude Code, Codex and Cursor with one button each; see every app's install state and how formMCP reaches it; switch an app's tools to always show; watch the activity (sessions, tool calls and app starts, never their arguments). Painted complete in its first frame, and nothing moves when the live state arrives.

## Quick start

Requirements: Rust (stable), Node 20.19+, and on Linux WebKitGTK 4.1 for the window (the installer prints the packages; the MCP server does not need them).

```bash
./formmcp_oc_install        # macOS / Linux
./formmcp                   # the window on localhost:5620
./formmcp mcp               # the MCP server on stdio
./formmcp cli apps          # the family list as JSON
./formmcp cli reach         # how each app's server would be started on this computer
./formmcp cli agents install   # add formMCP to Claude Code, Codex and Cursor for your user account
```

Form app checkouts in the folder beside this one are found on their own (`FORMMCP_DEV_ROOTS` names other folders, or is set empty to turn that off); FormPortal installs are found through FormPortal. The wiki's Agents page has the details and the environment variables.

On Windows run `formmcp_oc_install.bat`, then `formmcp.bat`.

This repository is already wired for agents: `.mcp.json` (Claude Code), `.codex/config.toml` (Codex) and `.cursor/mcp.json` (Cursor) all run `./formmcp mcp`.

## Where things live

| Path | Role |
| --- | --- |
| `core/` | The engine: the family list (`core/data/family.json`, generated), finding each app's server (`reach.rs`), running it (`child.rs`) and the MCP server (`mcp.rs`) |
| `cli/` | `formmcp-cli apps`, `app`, `reach`, `suggest`, `catalog`, `expose`, `agents`, `mcp`, `version` |
| `src-tauri/` | The shell: the window, and `formmcp mcp` with no window |
| `frontend/` | The window: tokens, layers, one list |
| `setup/family.py` | Generates `core/data/family.json` from formcompiler's registry |
| `assets/` | The owner's icon |
| `.docs/wiki/` | Getting Started, Architecture, Agents |

| Port | Service |
| --- | --- |
| 5620 | Vite dev server (localhost only) |
| 5621 | Vite preview (localhost only) |

**Releases.** formMCP builds nothing in this repository. After the version bump lands on main, push a branch `compile_vX.Y.Z` cut from that commit; CI runs on it, and formcompiler (`nodemode-ai/formcompiler`) builds the bundles with `./build_app` (`build_app.bat` on Windows) and drafts the release `formMCP-vX.Y.Z` (`apps/formMCP.json` in its registry). `./build_app` also builds the app on your own machine.

## Status

v0.4.2: formMCP is in formcompiler's registry (`formmcp`, `ai.nodemode.formmcp`) with `build_app` and `build_app.bat` for its bundles, the same names in FormPortal, FormTray and FormRanker, and formskills in the family list. formMCP leaves itself out of its own list (generated and from the catalogue), so no agent can start formMCP inside formMCP.

v0.4.1: skills taken out of formMCP (the window's Skills filter and badge, the `skills` field in `family_apps`, the command line and the generated list); the server tells agents that skills are in formskills. Window checked again under throttling: CLS 0.

v0.4.0: the window. Clients with Connect and Disconnect, each app's install state and route, the always-show switches, and the activity panel fed by a log the MCP server writes (tool names and outcomes only). Claude Code counts as connected only at user scope. Checked in Chromium under 4x CPU and network throttling with a stubbed bridge (CLS 0 through the live state, Connect, a switch and the filters), and in the real window under Xvfb with a real session's log; Claude Code's install and remove run against a scratch home with the real `claude` command.

v0.3.0: `suggest_app`; the signed family catalogue as the list's first source (refused with its reason and the built-in list used until the family's catalogue key exists); an app's own tools always shown when chosen (`expose`); `agents install` for Claude Code, Codex and Cursor. Checked by hand: every catalogue trust case with a key pair from formcompiler, 17 tasks through `suggest_app`, FormPortal's tools exposed and called by their prefixed names, and the client writer in a scratch home.

v0.2.0: the gateway. `app_tools` and `call_app` reach an app's own server through its checkout or a FormPortal install; `family_apps` says which apps are installed and reachable. Checked by hand through FormPortal's checkout (33 tools, calls passed through), form.md's checkout and a FormPortal install. FormPortal starts an install only when its `form-app.json` names a launcher, which the apps' bundles do not carry yet (family plan, Q9).

v0.1.0: the scaffold. `family_apps` over stdio from the bundled list, the command line, the window, and the agent wiring.

Next: the owner's first push and `compile_v0.4.2` branch, and the family's catalogue key pair, so released apps and skills are read from the signed catalogue.

## License

formMCP is licensed under the GNU General Public License v3.0 (`LICENSE`, SPDX `GPL-3.0-only`), as ComfyUI and formpromo are. Copyright 2026 NodeAgentc, Inc. `NOTICE.md` lists the work it includes or depends on.

## Trademarks

Claude Code is a product of Anthropic, Codex of OpenAI and Cursor of Anysphere; they are named only as the agent clients formMCP connects to. formMCP is not affiliated with or endorsed by them.
