<p align="center"><img src="assets/formmcp_icon.png" width="160" alt="formMCP icon" /></p>

<h1 align="center">formMCP</h1>

<p align="center"><b>One MCP connection from Claude Code, Codex and Cursor to the whole Form family. Small on context, and it never installs anything.</b></p>

<p align="center">
  <img src="https://img.shields.io/badge/version-v0.7.4-f97316" alt="version v0.7.4" />
  <img src="https://img.shields.io/badge/MCP-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20Cursor-8b5cf6" alt="MCP for Claude Code, Codex and Cursor" />
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Rust-stable-B7410E" alt="Rust stable" />
  <img src="https://img.shields.io/badge/Node.js-20.19%2B-339933?logo=nodedotjs&logoColor=white" alt="Node.js 20.19+" />
  <img src="https://img.shields.io/badge/license-GPL--3.0-555555" alt="license GPL 3.0" />
</p>

formMCP is the one MCP server you add for the Form family of desktop apps (image, video, audio, layout, vector, notes, slides and the rest, by nodemode.ai). The agent sees every Form app in one short list (what each is for, whether it has an MCP server, whether this computer can reach it), whether or not any of them is installed, and loads one app's own tools only when the task needs that app. formMCP suggests; FormPortal installs.

**Stack:** Rust (`formmcp-core`, `formmcp-cli`) · Tauri 2 shell · vanilla TypeScript + Vite window with zero runtime dependencies · **Scale:** 22 Form apps from formcompiler's registry and the signed catalogue, 8 MCP tools, stdio and HTTP, 2 ways to reach an app, 3 agent clients

## Highlights

- **Cheap on context.** `family_apps` returns one line per app and no app's tools, so the whole family is about a thousand tokens (4.4 KB for 22 apps with their install state), not twenty apps' worth of tools.
- **Installed apps' own tools, on demand.** `app_tools` starts one app's own MCP server (through its checkout's launcher, or `formportal app-mcp` for a FormPortal install) and lists its tools; `call_app` runs one and returns the app's result unchanged. Nothing is copied, so the tools are always the running app's.
- **Resources and prompts too.** When an app offers documents or prompt templates, `app_resources`, `read_app_resource`, `app_prompts` and `get_app_prompt` pass them through on demand, like its tools.
- **You choose what each agent gets.** Turn an app off for Claude Code, Codex, Cursor or other agents, and turn single tools off (say, an app's delete) for every agent. Off apps leave the lists and are refused with the reason; changes apply from the next request.
- **Profiles you can share.** Save what an agent gets (apps off, tools off, start policies) as a named profile, use it for any agent, and export it as one file for a teammate to import. A profile only ever turns more off.
- **Start when it suits you.** Per app: start on demand and stop after 10 idle minutes, start in the background when an agent's session starts, or keep running while formMCP runs.
- **Safe by default.** Secrets an app needs live in the system keychain and reach only that app; secrets and token-shaped text are hidden in results and the log; an installed app whose files changed since formMCP first started it is refused. Optional hooks can allow or refuse each call, and optional tracing sends names and outcomes to an OpenTelemetry collector.
- **stdio or HTTP.** Agents start `formmcp mcp`, or reach `http://127.0.0.1:5625/mcp` while the window (or `formmcp serve`) is open: this computer only, an access token on every request, Host and Origin checked, the token renewed with one button.
- **The right app for the task.** `suggest_app` takes the task in the person's words and returns the apps that fit with the next step: load its tools, install it with FormPortal, or install FormPortal first.
- **Never hand-kept.** The family list is the signed family catalogue (checked against the same keys FormPortal trusts) over a list generated from formcompiler's registry (`./formmcp family --compiler PATH`), and every answer says which one was used and why.
- **Tools only; skills live in formskills.** formMCP lists apps and runs their MCP tools. The Form apps' released skills are viewed, downloaded and edited in formskills, not here.
- **Suggests, never installs.** There is no tool that installs, updates or removes an app; the server tells the agent that FormPortal does.
- **Headless first.** The engine and the MCP server need no window. The installed app answers `formmcp mcp` on stdio without opening one, so it is its own launcher.
- **A window that does not flicker.** Connect formMCP to Claude Code, Codex and Cursor with one button each; see every app's install state and how formMCP reaches it; switch an app's tools to always show; watch the activity (sessions, tool calls and app starts, never their arguments). Painted complete in its first frame, and nothing moves when the live state arrives.

## Quick start

Requirements: Rust (stable), Node 20.19+; on Linux libdbus for the keychain (`libdbus-1-dev` and `pkg-config`) and WebKitGTK 4.1 for the window (the installer prints the packages; the MCP server needs only libdbus).

```bash
./formmcp_oc_install        # macOS / Linux
./formmcp                   # the window on localhost:5620
./formmcp mcp               # the MCP server on stdio
./formmcp cli apps          # the family list as JSON
./formmcp cli reach         # how each app's server would be started on this computer
./formmcp cli agents install   # add formMCP to Claude Code, Codex and Cursor for your user account
./formmcp cli serve         # the MCP server over HTTP at http://127.0.0.1:5625/mcp (or Start in the window)
./formmcp cli http token    # the access token an HTTP agent sends as Authorization: Bearer <token>
./formmcp cli for codex formvis off       # FormVis not offered to Codex
./formmcp cli tool formvis erase_all off  # one tool off for every agent
./formmcp cli start formvis session       # start FormVis when an agent's session starts (demand, session, keep)
./formmcp cli profile save "Quiet desk" --from codex   # Codex's settings as a profile
./formmcp cli profile use claude "Quiet desk"          # Claude Code uses it
./formmcp cli profile export "Quiet desk"              # a file in Downloads to share; profile import <file> reads one
./formmcp cli secret set formmd FORMMD_MCP_TOKEN < token.txt   # kept in the system keychain, given to form.md's server
./formmcp cli verify formvis                           # check FormVis's installed files now
./formmcp cli hook before "python3 my_policy.py"       # allow or refuse each app tool call
./formmcp cli telemetry http://127.0.0.1:4318          # traces and metrics to an OpenTelemetry collector
```

Form app checkouts in the folder beside this one are found on their own (`FORMMCP_DEV_ROOTS` names other folders, or is set empty to turn that off); FormPortal installs are found through FormPortal. The wiki's Agents page has the details and the environment variables.

On Windows run `formmcp_oc_install.bat`, then `formmcp.bat`.

This repository is already wired for agents: `.mcp.json` (Claude Code), `.codex/config.toml` (Codex) and `.cursor/mcp.json` (Cursor) all run `./formmcp mcp`.

## Where things live

| Path | Role |
| --- | --- |
| `core/` | The engine: the family list (`core/data/family.json`, generated), finding each app's server (`reach.rs`), running it (`child.rs`), the MCP server (`mcp.rs`), and the safety parts: `secrets.rs`, `redact.rs`, `verify.rs`, `hooks.rs`, `telemetry.rs` |
| `cli/` | `formmcp-cli apps`, `app`, `reach`, `suggest`, `catalog`, `expose`, `for`, `tool`, `tools`, `start`, `profile`, `secret`, `verify`, `redact`, `hook`, `telemetry`, `settings`, `agents`, `mcp`, `serve`, `http`, `version` |
| `src-tauri/` | The shell: the window, and `formmcp mcp` with no window |
| `frontend/` | The window: tokens, layers, one list |
| `setup/family.py` | Generates `core/data/family.json` from formcompiler's registry |
| `assets/` | The owner's icon |
| `.docs/wiki/` | Getting Started, Architecture, Agents |

| Port | Service |
| --- | --- |
| 5620 | Vite dev server (localhost only) |
| 5621 | Vite preview (localhost only) |
| 5625 | formMCP over HTTP, while the window or `formmcp serve` runs (127.0.0.1 only; `FORMMCP_HTTP_PORT` or `http_port` in settings.json changes it) |

**Releases.** formMCP builds nothing in this repository. After the version bump lands on main, push a branch `compile_vX.Y.Z` cut from that commit; CI runs on it, and formcompiler (`nodemode-ai/formcompiler`) builds the bundles with `./build_app` (`build_app.bat` on Windows) and drafts the release `formMCP-vX.Y.Z` (`apps/formMCP.json` in its registry). `./build_app` also builds the app on your own machine.

## Status

v0.7.1: installed apps checked against FormPortal's record of every file (FormPortal 0.46.0), falling back to formMCP's own first-start record with an older FormPortal. Checked with a FormPortal 0.46.0 install made from formcompiler 0.4.2's `form-app.json`: a clean install started (outcome formportal); a changed and an added file refused before formMCP had ever started the app; a record for another version refused; without a record, the 0.7.0 behaviour.

v0.7.0: safety and tracing. Secrets in the system keychain given to one app's server; secrets and token-shaped text hidden in results and the log; an installed app's files checked before it starts; before and after hooks; OpenTelemetry traces and metrics over HTTP; the window's Safety panel. Checked over stdio with a scratch FormPortal install and a real GNOME Keyring on D-Bus: a secret reaching the app and hidden in its answer (shown when hiding is off), never in settings.json or the log; a keychain that cannot be reached refusing the start with the reason; a changed and an added file refusing the start, Record again, a record pointing outside FormPortal's apps folder refused, a new version recorded afresh; a before hook allowing, refusing by answer and by exit 2, failing closed when missing or too slow, an after hook given the cleaned result; traces and metrics received by a local collector and parsed with the OpenTelemetry protocol definitions; the window at 4x CPU and network throttling bridged to the real CLI (CLS 0, dark 1040 by 780, light 640 by 520) and the real window under Xvfb (a secret kept in the real keyring).

v0.6.0: profiles and start-up. Named profiles (apps off, tools off, start policies) used per agent, exported and imported as a file; a start policy per app (on demand with an idle stop, with the session, keep running). Checked over stdio with a scratch app (each policy, idle stop with `FORMMCP_IDLE_MINUTES=0`, profiles turning apps and tools off for one agent only, export, import with unknown entries dropped, bad files refused, deletion), the window at 4x CPU and network throttling bridged to the real CLI (CLS 0, dark, light, 640 by 520), and the real window (FormPortal's real server started in the background as an HTTP session began).

v0.5.0: connect (family plan Phase 6). Streamable HTTP on 127.0.0.1 with an access token; apps turned on or off per agent; single tools turned off; an app's resources and prompts passed through; the window's HTTP bar, agent picker and per-app Tools panel. Checked: every HTTP rule with curl (401, 403 for a foreign Host or Origin, 400 and 404 for sessions and versions, 405, 202, chunked bodies, batches, token rotation, loopback only); the real Claude Code connected over HTTP (`claude mcp list`: Connected) and was recognised by its own name; agent sets and tool switches over stdio and HTTP; resources and prompts through a scratch server and FormPortal's real one; the window at 4x CPU and network throttling bridged to the real engine (CLS 0, dark, light and 640 by 520) and the real window under Xvfb (HTTP started, a Cursor session over it, FormPortal's 33 tools loaded).

v0.4.2: formMCP is in formcompiler's registry (`formmcp`, `ai.nodemode.formmcp`) with `build_app` and `build_app.bat` for its bundles, the same names in FormPortal, FormTray and FormRanker, and formskills in the family list. formMCP leaves itself out of its own list (generated and from the catalogue), so no agent can start formMCP inside formMCP.

v0.4.1: skills taken out of formMCP (the window's Skills filter and badge, the `skills` field in `family_apps`, the command line and the generated list); the server tells agents that skills are in formskills. Window checked again under throttling: CLS 0.

v0.4.0: the window. Clients with Connect and Disconnect, each app's install state and route, the always-show switches, and the activity panel fed by a log the MCP server writes (tool names and outcomes only). Claude Code counts as connected only at user scope. Checked in Chromium under 4x CPU and network throttling with a stubbed bridge (CLS 0 through the live state, Connect, a switch and the filters), and in the real window under Xvfb with a real session's log; Claude Code's install and remove run against a scratch home with the real `claude` command.

v0.3.0: `suggest_app`; the signed family catalogue as the list's first source (refused with its reason and the built-in list used until the family's catalogue key exists); an app's own tools always shown when chosen (`expose`); `agents install` for Claude Code, Codex and Cursor. Checked by hand: every catalogue trust case with a key pair from formcompiler, 17 tasks through `suggest_app`, FormPortal's tools exposed and called by their prefixed names, and the client writer in a scratch home.

v0.2.0: the gateway. `app_tools` and `call_app` reach an app's own server through its checkout or a FormPortal install; `family_apps` says which apps are installed and reachable. Checked by hand through FormPortal's checkout (33 tools, calls passed through), form.md's checkout and a FormPortal install. FormPortal starts an install only when its `form-app.json` names a launcher, which the apps' bundles do not carry yet (family plan, Q9).

v0.1.0: the scaffold. `family_apps` over stdio from the bundled list, the command line, the window, and the agent wiring.

Next: v0.7.0, secrets in the system keychain, secrets kept out of calls and the log, a file check before an installed app starts, hooks and tracing. The owner's first push, `compile_v*` branch and the family's catalogue key pair are still to come.

## License

formMCP is licensed under the GNU General Public License v3.0 (`LICENSE`, SPDX `GPL-3.0-only`), as ComfyUI and formpromo are. Copyright 2026 NodeAgentc, Inc. `NOTICE.md` lists the work it includes or depends on.

## Trademarks

Claude Code is a product of Anthropic, Codex of OpenAI and Cursor of Anysphere; they are named only as the agent clients formMCP connects to. formMCP is not affiliated with or endorsed by them.
