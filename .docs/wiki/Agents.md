# Agents

formMCP's MCP server is `formmcp mcp` (stdio), and from v0.5.0 also Streamable HTTP at `http://127.0.0.1:5625/mcp` while the window or `formmcp serve` runs (see HTTP below). In this repository it is already wired:

| Client | In this repository |
| --- | --- |
| Claude Code | `.mcp.json` |
| Codex | `.codex/config.toml` |
| Cursor | `.cursor/mcp.json` |

All three run `./formmcp mcp`, which builds the command line on first use. An installed formMCP answers the same way: `<the formMCP program> mcp`.

For every project, add it to your user account instead: in the formMCP window, press Connect beside Claude Code, Codex or Cursor (the installed app registers itself), or from a checkout:

```bash
./formmcp cli agents install                 # Claude Code, Codex and Cursor
./formmcp cli agents install codex,cursor    # some of them
./formmcp cli agents status                  # connected, not connected, taken by another server, or no client
./formmcp cli agents remove
```

Claude Code is set up with `claude mcp add --scope user formmcp` (only a user-scope entry counts as connected; a project's `.mcp.json`, like this repository's, is left alone); Codex gets `[mcp_servers.formmcp]` in `~/.codex/config.toml` (or `$CODEX_HOME`); Cursor gets `mcpServers.formmcp` in `~/.cursor/mcp.json`. Each file is copied to `<name>.bak-<time>` before it changes, other servers are left as they are, and a server named `formmcp` that formMCP did not write is reported `taken` and left alone. The entry starts the `./formmcp` launcher of this checkout and carries the formMCP settings that are set in your shell (`FORMMCP_DEV_ROOTS`, `FORMPORTAL_ROOT`, `FORMPORTAL_BIN`, `FORMMCP_CATALOG`, `FORMAI_CATALOG_PUBKEY`).

## Tools

| Tool | Changes anything | Returns |
| --- | --- | --- |
| `family_apps` | no | every Form app (`id`, `name`, `kind`, `summary`, `mcp`, `state`, `installed`, `reach`) and where the list came from; `mcp_only` keeps the apps with an MCP server |
| `suggest_app` | no | for a task in the person's words, up to five apps, best first, each with `installed`, `reach` and the next step (load its tools, install it with FormPortal, install FormPortal first, or why an agent cannot drive it yet) |
| `app_tools` | no | one app's own tools with their input schemas, its server name and version, and its instructions; starts its server the first time |
| `call_app` | what the app's tool changes | the app's own result for one of its tools, unchanged |
| `app_resources` | no | the resources one app offers (each with a `uri`) and its resource templates; empty with a note when the app declares no resources |
| `read_app_resource` | no | one resource's contents by `uri`, as embedded resource content |
| `app_prompts` | no | the prompt templates one app offers, with their arguments |
| `get_app_prompt` | no | one prompt filled with its `arguments`: the app's description and messages |

`app_tools` also says which of the app's tools are turned off (`hidden`) and whether it offers resources and prompts (`offers`).

## Apps per agent, and single tools

The window's Apps for picker chooses an agent (Claude Code, Codex, Cursor, or other agents); each card's On for switch offers that app to it or not. `./formmcp cli for codex formvis off` does the same. An app turned off for an agent is left out of `family_apps`, `suggest_app` and the always-shown tools, and `app_tools`, `call_app` and the resource and prompt tools refuse it with the reason. The agent is known by `--client <name>` on `formmcp mcp`, by `?client=<name>` on the HTTP address, or else by the name the agent sends when it starts (Claude Code sends `claude-code`, which counts as claude); anything else counts as other agents.

The card's Tools panel turns single tools off for every agent (`./formmcp cli tool formvis erase_all off`): left out of `app_tools` and the always-shown tools, refused by `call_app`. The panel lists the tools formMCP saw when it last started the app; Load tools starts it once to see them. Settings are read again on every request, so running servers follow a change at once (an agent that cached its tool list sees always-shown tools change the next time it lists them).

## Profiles

A profile is a name for a set of apps off, tools off and start policies. Save one from an agent's current settings (Profiles, then Save, in the window; `./formmcp cli profile save <name> --from <agent>`), choose it for any agent with the Profile picker (`profile use <agent> <name>`), and share it: `profile export <name>` writes `formmcp-profile-<name>.json` to Downloads (never over an existing file), `profile import <file>` (or Import in the window) reads one. A profile in use adds to what is off for that agent and never turns anything back on; its start policies take the place of the app's own. Importing drops, and names, apps this formMCP does not know and policies it does not have; another format, a name outside 1 to 40 letters, digits, spaces, hyphens and underscores, or a file over 256 KB is refused. An app a profile turns off is refused with the profile's name.

```json
{ "format": "formmcp-profile", "format_version": 1, "name": "Quiet desk",
  "apps_off": ["formvis"], "hidden_tools": { "formvis": ["erase_all"] }, "start": { "formai": "session" } }
```

## When an app's server starts

| Policy | Starts | Stops |
| --- | --- | --- |
| `demand` (default) | the first time an agent needs the app | after `FORMMCP_IDLE_MINUTES` (10) without a call, checked at the next request; or when the session ends |
| `session` | in the background when an agent's session starts, so the first call does not wait | when the session ends |
| `keep` | the first time an agent needs it | when the session ends (or, over HTTP, when the window or `serve` stops) |

Set it on the card's start select or with `./formmcp cli start <id> demand|session|keep`. Over HTTP all sessions share the app servers, so `keep` holds an app across sessions.

## HTTP

Press Start in the window's HTTP bar, or run `./formmcp cli serve` (`--port N`, else `FORMMCP_HTTP_PORT`, `http_port` in settings.json, or 5625). The server listens on 127.0.0.1 only and implements Streamable HTTP (protocol 2025-06-18; older versions accepted):

- every request carries `Authorization: Bearer <token>`; the token is 64 hex characters in `http-token` in formMCP's folder (readable by you only), shown by `./formmcp cli http token` and Copy setup, and replaced by `./formmcp cli http rotate` or New token (the old one stops working at once)
- the `Host` must be this computer at this port and an `Origin`, when sent, a page on this computer (DNS rebinding cannot reach it)
- `initialize` returns an `Mcp-Session-Id`; later requests need it (400 without, 404 for an ended one); DELETE ends a session
- notifications get 202; GET gets 405, because formMCP sends no server stream (so no list-changed notices over HTTP)
- calls are answered one at a time, as on stdio; app servers started over HTTP are shared by all HTTP sessions and stop with the window or `serve`

For Claude Code: `claude mcp add --scope user --transport http formmcp http://127.0.0.1:5625/mcp --header "Authorization: Bearer <token>"` (Copy setup gives this line with the token). Refused requests are written to the activity log with the reason.

The list is short on purpose: one line per app and no app's tools. An app's tools enter the conversation only when `app_tools` is called for it, so formMCP stays small however many apps are installed. The server's instructions tell the agent that formMCP never installs and that FormPortal installs and updates the Form apps, and that formMCP does not handle skills: the Form apps' released skills are in formskills.

## How an app's server is started

`reach` in `family_apps`, and `./formmcp cli reach`, show the route for each app:

| Route | When | Started as |
| --- | --- | --- |
| `dev` | a checkout of the app (`<folder>/<app>` holding its launcher) is in a folder listed in `FORMMCP_DEV_ROOTS` | `<checkout>/<launcher> mcp` (the app's own bridge, which starts or attaches to its engine) |
| `formportal` | FormPortal installed the app and its program is found | `<FormPortal program> app-mcp <id>` (always the installed version) |

`./formmcp` sets `FORMMCP_DEV_ROOTS` to the folder that holds the formMCP checkout when it is not set, so Form app checkouts beside it are found; set it empty to turn that off. FormPortal's program is `FORMPORTAL_BIN`, else `formportal-cli` or `formportal` on PATH, else `/Applications/FormPortal.app` on macOS. FormPortal starts an install only when the install's `form-app.json` names a launcher (`agents.mcp.command`); until the apps' bundles carry one, installed apps are reached through their checkouts.

An app's server inherits formMCP's environment, so what an app needs (form.md's `FORMMD_MCP_TOKEN`) is set once in the client's config for formmcp. `FORMMCP_START_TIMEOUT` (600 s) and `FORMMCP_CALL_TIMEOUT` (900 s) bound the waits. A started server lives as long as the client's formMCP session and ends with it.

## Always showing one app's tools

For an app you use all the time, turn on "Always show tools" on its card in the window, or run `./formmcp cli expose formvis on` (saved in formMCP's `settings.json`; `FORMMCP_EXPOSE=formvis,formai` does the same for one session) makes formMCP start it when the client lists tools and show its tools directly, as `formvis__<tool>`, described with `[FormVis]` in front. The client then sees them without `app_tools`; formMCP tells it when the app's tools change. Off for every app by default, because each exposed app costs its whole tool list in every conversation. `expose formvis off` takes it out.

## Secrets, the file check, hooks and tracing

Open Safety in the window, or use the command line:

```bash
./formmcp cli secret set formmd FORMMD_MCP_TOKEN < token.txt   # read from stdin, kept in the system keychain
./formmcp cli secret list                                      # names and whether each is in the keychain
./formmcp cli secret remove formmd FORMMD_MCP_TOKEN
./formmcp cli redact off                                       # results as apps send them (the log is still cleaned)
./formmcp cli verify formvis                                   # check its installed files now
./formmcp cli verify --trust formvis                           # accept the files as they are, after a change you made
./formmcp cli hook before "python3 /path/policy.py"            # off: hook before off
./formmcp cli hook after "python3 /path/audit.py"
./formmcp cli telemetry http://127.0.0.1:4318                  # off: telemetry off
```

A secret is given to that app's server only, as the environment variable named, when formMCP starts it; a running server keeps the value it started with. If the keychain cannot be read (on Linux, no Secret Service running), the app is not started and the agent is told why.

A before hook gets one JSON object on stdin: `event` ("before"), `app`, `tool`, `arguments` (secrets hidden), `client`, `transport` and `formmcp`. To refuse, print `{"decision":"deny","reason":"..."}` or exit with 2 (the first line of stderr is the reason); exit 0 with nothing printed, or `{"decision":"allow"}`, lets the call run. If the hook cannot start, exits otherwise, prints something that is not JSON or takes longer than 10 s (`FORMMCP_HOOK_TIMEOUT`), the call is refused. The after hook gets the same fields plus `ok`, `ms` and `result`; what it prints is ignored. Hooks apply to app tool calls (`call_app` and always-shown tools).

## Activity

Every formMCP server appends one line per event to `log.jsonl` in formMCP's settings folder: a session starting (the client's name, the agent it counts as, stdio or HTTP), HTTP starting, stopping and refusing a request, each tool call (formMCP's tool, the app and the app's tool, whether it worked, how long it took, how many secrets were hidden) and each app server started (route, tool count, the file check, or why not), and each hook's decision. Arguments and results are never recorded, and every line is passed through the same hiding of secrets. The window's Activity panel shows it as it happens; the file rolls over at 1 MB.

## Where the list comes from

`family_apps` and `suggest_app` read the signed family catalogue once per session, on the first tool call, and fall back to the list built into formMCP. The catalogue is formcompiler's `catalog` release (or `FORMMCP_CATALOG`: an https or loopback address, or a file), with its `.sig`; when it cannot be fetched, FormPortal's kept copy is used. It must be signed with a key formMCP trusts: the keys in `core/catalog_keys.txt`, the same as FormPortal's, plus `FORMAI_CATALOG_PUBKEY`. `FORMMCP_OFFLINE=1` reads only FormPortal's copy; `FORMMCP_ALLOW_UNSIGNED=1` admits an unsigned one, for development. The answer's `source` names what was used and why, and `./formmcp cli catalog` shows the same. Until the family's catalogue key pair exists, no key is trusted and the built-in list is used.

The `formmcp-harness` skill (`.claude/skills/formmcp-harness`) teaches the workflow.
