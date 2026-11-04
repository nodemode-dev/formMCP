<p align="center"><img src="assets/app-icon/formMCP-256.png" width="128" height="128" alt="FormMCP icon: a folded orange and violet ribbon shaped like the letters M and C on a dark tile"></p>

<h1 align="center">FormMCP</h1>

<p align="center">
  <b>Agent gateway; one MCP connection from Claude Code, Codex and Cursor to the Form family.</b><br>
  Every Form app in one short list, an app's own tools loaded only when a task needs them,<br>
  and it never installs anything. Local, and yours.
</p>

<p align="center">
  <img alt="version 0.7.7" src="https://img.shields.io/badge/version-0.7.7-eef0f4?style=flat-square">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-stable-B7410E?style=flat-square&logo=rust&logoColor=white">
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square">
  <img alt="macOS, Windows, Linux" src="https://img.shields.io/badge/macOS%20%C2%B7%20Windows%20%C2%B7%20Linux-native-2f7bf5?style=flat-square">
  <img alt="License: GPL-3.0" src="https://img.shields.io/badge/license-GPL--3.0-3a3a3a?style=flat-square">
  <img alt="Status: in development" src="https://img.shields.io/badge/status-in%20development-d69e2e?style=flat-square">
</p>

<p align="center">
  <a href="https://formmcp.formai.studio/"><b>Download FormMCP on formai.studio</b></a> ·
  <a href="#run-it-from-source">Run it from source</a> ·
  <a href="#the-form-family">All Form apps</a>
</p>

<br>

<p align="center">
  <img src="docs/images/formmcp-hero.png" alt="The FormMCP window: Claude Code, Codex and Cursor connect buttons, an HTTP bar, cards for FormPortal, Format, FormCut and other Form apps with per-agent switches, and an Activity log of family_apps and suggest_app calls" width="100%">
  <br><sub>The window: the Form family as agents see it, with a live log of the sessions and calls from three agents.</sub>
</p>

<p align="center">
  <a href="#features">Features</a> · <a href="#everything-in-the-box">In the box</a> · <a href="#built-for-agents">Agents</a> · <a href="#under-the-hood">Under the hood</a> · <a href="#download">Download</a> · <a href="#run-it-from-source">Run from source</a> · <a href="#the-form-family">Form family</a>
</p>

<br>

<table>
  <tr>
    <td width="25%" valign="top"><h3>Cheap on context</h3>One line per app and no app's tools, so the family list stays small instead of loading every app's tools up front.</td>
    <td width="25%" valign="top"><h3>You choose what each agent gets</h3>Turn an app or a single tool off for Claude Code, Codex or Cursor, and share the setup as a named profile.</td>
    <td width="25%" valign="top"><h3>Safe by default</h3>Secrets live in the system keychain, results are scrubbed, and an installed app whose files changed is refused.</td>
    <td width="25%" valign="top"><h3>Agent-ready</h3>The same engine runs the window, a command line and an MCP server over stdio or local HTTP.</td>
  </tr>
</table>

<br>

## Features

<table>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/formmcp-agents.webp" alt="The FormMCP window with Codex selected and FormCut switched off for Codex" width="100%">
      <br><sub>Apps per agent: FormCut is turned off for Codex only.</sub>
      <h3>Per-agent control</h3>
      Off apps leave that agent's lists and are refused with the reason. Changes apply from the next request.
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/formmcp-safety.png" alt="The Safety panel with keychain secrets, hide-secrets and file-check switches, before and after hooks and a tracing field" width="100%">
      <br><sub>The Safety panel: secrets, hooks and tracing.</sub>
      <h3>Safety you can see</h3>
      Hooks can allow or refuse each call, and tracing sends names and outcomes, never arguments, to an OpenTelemetry collector.
    </td>
  </tr>
</table>

## Everything in the box

<table>
  <tr>
    <td width="33%" valign="top"><h4>8 MCP tools</h4>family_apps · suggest_app · app_tools · call_app · app_resources · read_app_resource · app_prompts · get_app_prompt</td>
    <td width="33%" valign="top"><h4>Controls</h4>Apps per agent · single tools · profiles · start policies · hooks · secrets · tracing · activity log</td>
    <td width="33%" valign="top"><h4>Transports</h4>stdio · local HTTP at <code>127.0.0.1:5625/mcp</code> with an access token · Claude Code, Codex and Cursor</td>
  </tr>
</table>

## Built for agents

FormMCP suggests; FormPortal installs. The agent sees one short list and loads one app's own tools on demand.

```sh
./formmcp cli apps                        # the family list as JSON
./formmcp cli reach                       # how each app's server would be started on this computer
./formmcp cli agents install              # add FormMCP to Claude Code, Codex and Cursor
./formmcp mcp                             # MCP server on stdio for Claude Code, Codex and Cursor
```

`.mcp.json`, `.codex/config.toml` and `.cursor/mcp.json` are already wired in this repo. See [`AGENTS.md`](AGENTS.md).

## Under the hood

- **Headless first:** the engine and the MCP server need no window, and the installed app answers `formmcp mcp` without opening one.
- **Family list:** the signed family catalogue over a list generated from formcompiler's registry, and every answer says which was used.
- **Pass-through:** `call_app` returns an app's own result unchanged, so the tools are always the running app's.
- **Window:** vanilla TypeScript and Vite with zero runtime dependencies, painted complete in its first frame.
- **Local only:** HTTP binds 127.0.0.1, checks Host and Origin, and needs a token on every request.
- **Tested:** 30 Rust tests (family list, catalogue keys, reach, the MCP server over stdio and HTTP).

## Download

**Get the app:** download FormMCP from [formai.studio](https://formmcp.formai.studio/). That is the easiest way to install it.

**Keep it updated:** [FormPortal](https://github.com/nodemode-dev/formportal) installs, updates and rolls back every Form app from the same signed catalogue.

| Platform | Builds |
|---|---|
| macOS | Apple silicon, Intel |
| Linux | x64 |
| Windows | x64 |

All FormMCP builds are compiled and released from [`nodemode-dev/formcompiler`](https://github.com/nodemode-dev/formcompiler/releases), not from this repo. This repo holds the source.

## Run it from source

Prefer it in your browser, or want to contribute? Run the web UI straight from this repo.

**Requires:** Rust stable, Node 20.19 or newer. On Linux, `libdbus-1-dev` and `pkg-config` for the keychain, and WebKitGTK 4.1 for the window.

```sh
git clone git@github.com:nodemode-dev/formMCP.git
cd formMCP
./formmcp_oc_install   # checks the tools and installs the window's packages
./formmcp              # the window at http://localhost:5620
```

On Windows run `formmcp_oc_install.bat`, then `formmcp.bat`. Form app checkouts in the folder beside this one are found on their own.

| Service | URL | Notes |
|---|---|---|
| Window (Vite) | http://localhost:5620 | localhost only; preview is on 5621 |
| MCP over HTTP | http://127.0.0.1:5625/mcp | while the window or `formmcp serve` runs; `FORMMCP_HTTP_PORT` changes it |

| Variable | Purpose |
|---|---|
| `FORMMCP_DEV_ROOTS` | Folders holding Form app checkouts (empty turns the search off) |
| `FORMMCP_CATALOG` | Catalogue address to read instead of the public store |
| `FORMMCP_HOME` | Where settings, the activity log and the access token are kept |

> [!IMPORTANT]
> **Status:** in development. In a plain browser the window shows the family list only; its live panels need the desktop shell. See [`.docs/wiki/Architecture.md`](.docs/wiki/Architecture.md).

**Contributing:** branch from `main`, keep changes to source here, and leave packaging and releases to formcompiler. See [`.docs/BRANCHING.md`](.docs/BRANCHING.md) and [`AGENTS.md`](AGENTS.md).

## Repository

| Path | What it is |
|---|---|
| `core/` | the engine: family list, reach, running apps, the MCP server and the safety parts |
| `cli/` | `formmcp-cli`, the command line and MCP server |
| `src-tauri/` | the Tauri 2 shell: the window, and `formmcp mcp` with no window |
| `frontend/` | the window: tokens, layers, one list |
| `setup/` | `family.py`, which generates the family list from formcompiler |

Docs: [`.docs/wiki`](.docs/wiki) · Agents: [`CLAUDE.md`](CLAUDE.md), [`AGENTS.md`](AGENTS.md)

## The Form family

FormMCP is one of the **Form apps**: native creative tools that install, update and roll back through FormPortal and can all be driven by agents.

| | App | What it's for | Download | Code |
|:-:|---|---|---|---|
| <img src=".github/assets/family/form.md.png" alt="" width="32" height="32"> | **form.md** | Documents, databases and realtime editing together | [formai.studio](https://formmd.formai.studio/) | [GitHub](https://github.com/nodemode-dev/form.md) |
| <img src=".github/assets/family/format.png" alt="" width="32" height="32"> | **FormAT** | Chat and agent workspace on your key | [formai.studio](https://format.formai.studio/) | [GitHub](https://github.com/nodemode-dev/format) |
| <img src=".github/assets/family/formCAD.png" alt="" width="32" height="32"> | **FormCAD** | Drafting, parametric parts and NURBS surfaces | | [GitHub](https://github.com/nodemode-dev/formCAD) |
| <img src=".github/assets/family/formcel.png" alt="" width="32" height="32"> | **FormCel** | Spreadsheets agents can read and edit | [formai.studio](https://formcel.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formcel) |
| <img src=".github/assets/family/formcraft.png" alt="" width="32" height="32"> | **FormCraft** | Generate images, video, audio, 3D and worlds | | [GitHub](https://github.com/nodemode-dev/formcraft) |
| <img src=".github/assets/family/formcut.png" alt="" width="32" height="32"> | **FormCut** | Multi-track video editing with no upload | [formai.studio](https://formcut.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formcut) |
| <img src=".github/assets/family/formDAM.png" alt="" width="32" height="32"> | **FormDAM** | Photo library and asset manager | [formai.studio](https://formdam.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formDAM) |
| <img src=".github/assets/family/formdeck.png" alt="" width="32" height="32"> | **FormDeck** | Presentations and pitch decks | [formai.studio](https://formdeck.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formdeck) |
| <img src=".github/assets/family/formdesign.png" alt="" width="32" height="32"> | **FormDesign** | Page layout for print and the web | [formai.studio](https://formdesign.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formdesign) |
| <img src=".github/assets/family/formencoder.png" alt="" width="32" height="32"> | **FormEncoder** | Media encoding queue with presets | [formai.studio](https://formencoder.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formencoder) |
| <img src=".github/assets/family/formfabric.png" alt="" width="32" height="32"> | **FormFabric** | Sewing patterns draped with cloth simulation | | [GitHub](https://github.com/nodemode-dev/formfabric) |
| <img src=".github/assets/family/formlit.png" alt="" width="32" height="32"> | **FormLit** | Photo library and raw development | [formai.studio](https://formlit.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formlit) |
| <img src=".github/assets/family/formmorpheus.png" alt="" width="32" height="32"> | **FormMorpheus** | Frame-by-frame and performed animation | | [GitHub](https://github.com/nodemode-dev/formmorpheus) |
| <img src=".github/assets/family/formNET.png" alt="" width="32" height="32"> | **FormNET** | See what your computer sends online | | [GitHub](https://github.com/nodemode-dev/formNET) |
| <img src=".github/assets/family/formpath.png" alt="" width="32" height="32"> | **FormPath** | Vector illustration with artboards and type | [formai.studio](https://formpath.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formpath) |
| <img src=".github/assets/family/formpdf.png" alt="" width="32" height="32"> | **FormPDF** | Edit, redact, sign and fill PDFs | [formai.studio](https://formpdf.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formpdf) |
| <img src=".github/assets/family/formpromo.png" alt="" width="32" height="32"> | **FormPromo** | Promo films from your own media | [formai.studio](https://formpromo.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formpromo) |
| <img src=".github/assets/family/formref.png" alt="" width="32" height="32"> | **FormRef** | Reference boards for creative work | [formai.studio](https://formref.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formref) |
| <img src=".github/assets/family/formrig.png" alt="" width="32" height="32"> | **FormRig** | Rig, skin and retarget any mesh | | [GitHub](https://github.com/nodemode-dev/formrig) |
| <img src=".github/assets/family/formsculpt.png" alt="" width="32" height="32"> | **FormSculpt** | 3D sculpting, remeshing and painting | [formai.studio](https://formsculpt.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formsculpt) |
| <img src=".github/assets/family/formsurface.png" alt="" width="32" height="32"> | **FormSurface** | Texture painting with PBR export | [formai.studio](https://formsurface.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formsurface) |
| <img src=".github/assets/family/formtools.png" alt="" width="32" height="32"> | **FormTools** | Viewers, batch tools and file utilities | | [GitHub](https://github.com/nodemode-dev/formtools) |
| <img src=".github/assets/family/formtxt.png" alt="" width="32" height="32"> | **FormTxt** | Word processing with .docx round trips | [formai.studio](https://formtxt.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formtxt) |
| <img src=".github/assets/family/formvfx.png" alt="" width="32" height="32"> | **FormVFX** | Motion graphics and VFX compositing | [formai.studio](https://formvfx.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formvfx) |
| <img src=".github/assets/family/formvis.png" alt="" width="32" height="32"> | **FormVis** | Layered image editing with local AI | [formai.studio](https://formvis.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formvis) |
| <img src=".github/assets/family/formwav.png" alt="" width="32" height="32"> | **FormWav** | Multitrack audio editing and mixing | [formai.studio](https://formwav.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formwav) |
| <img src=".github/assets/family/formweb.png" alt="" width="32" height="32"> | **FormWeb** | Visual website building with a CMS | [formai.studio](https://formweb.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formweb) |
| <img src=".github/assets/family/formportal.png" alt="" width="32" height="32"> | **FormPortal** | Install, update and roll back Form apps | [formai.studio](https://formportal.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formportal) |
| <img src=".github/assets/family/formtray.png" alt="" width="32" height="32"> | **FormTray** | One dock icon for every app | [formai.studio](https://formtray.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formtray) |
| <img src=".github/assets/family/formMCP.png" alt="" width="32" height="32"> | **FormMCP** | One MCP connection to the family | [formai.studio](https://formmcp.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formMCP) |
| <img src=".github/assets/family/formskills.png" alt="" width="32" height="32"> | **FormSkills** | Every app's agent skills in one place | [formai.studio](https://formskill.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formskills) |
| <img src=".github/assets/family/formbench.png" alt="" width="32" height="32"> | **FormBench** | Parity, testing and rankings for the family | [formai.studio](https://formranker.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formbench) |
| <img src=".github/assets/family/formslot.png" alt="" width="32" height="32"> | **FormSlot** | Codebase memory for coding agents | | [GitHub](https://github.com/nodemode-dev/formslot) |
| <img src=".github/assets/family/subform.png" alt="" width="32" height="32"> | **SubForm** | Workbench for parallel Claude Code agents | | [GitHub](https://github.com/nodemode-dev/subform) |
| <img src=".github/assets/family/formcompiler.png" alt="" width="32" height="32"> | **FormCompiler** | Builds, signs and releases every app | [formai.studio](https://formcompiler.formai.studio/) | [GitHub](https://github.com/nodemode-dev/formcompiler) |
| <img src=".github/assets/family/formpipe.png" alt="" width="32" height="32"> | **FormPipe** | Release pipeline and update feeds | | [GitHub](https://github.com/nodemode-dev/formpipe) |

---

## Maintainers

| Maintainer | Role |
|---|---|
| [@nodemode-ai](https://github.com/nodemode-ai) | Owner |
| [@gbarnai](https://github.com/gbarnai) | Co-owner |

Code owners are set in [`.github/CODEOWNERS`](.github/CODEOWNERS).

## License

FormMCP is licensed under the GNU General Public License v3.0 ([`LICENSE`](LICENSE), SPDX `GPL-3.0-only`). Copyright 2026 NodeAgentc, Inc. Third-party notices: [`NOTICE.md`](NOTICE.md).

<sub>Claude Code is a product of Anthropic, Codex of OpenAI and Cursor of Anysphere; they are named only as the agent clients FormMCP connects to. FormMCP is not affiliated with or endorsed by them.</sub>
