# Agents

formMCP's MCP server is `formmcp mcp` (stdio). In this repository it is already wired:

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

## Activity

Every formMCP server appends one line per event to `log.jsonl` in formMCP's settings folder: a session starting (the client's name), each tool call (formMCP's tool, the app and the app's tool, whether it worked, how long it took) and each app server started (route, tool count, or why not). Arguments and results are never recorded. The window's Activity panel shows it as it happens; the file rolls over at 1 MB.

## Where the list comes from

`family_apps` and `suggest_app` read the signed family catalogue once per session, on the first tool call, and fall back to the list built into formMCP. The catalogue is formcompiler's `catalog` release (or `FORMMCP_CATALOG`: an https or loopback address, or a file), with its `.sig`; when it cannot be fetched, FormPortal's kept copy is used. It must be signed with a key formMCP trusts: the keys in `core/catalog_keys.txt`, the same as FormPortal's, plus `FORMAI_CATALOG_PUBKEY`. `FORMMCP_OFFLINE=1` reads only FormPortal's copy; `FORMMCP_ALLOW_UNSIGNED=1` admits an unsigned one, for development. The answer's `source` names what was used and why, and `./formmcp cli catalog` shows the same. Until the family's catalogue key pair exists, no key is trusted and the built-in list is used.

The `formmcp-harness` skill (`.claude/skills/formmcp-harness`) teaches the workflow.
