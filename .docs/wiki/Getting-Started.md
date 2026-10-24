# Getting Started

## Requirements

- Rust (stable) for the engine, the command line and the shell.
- Node 20.19 or newer for the window.
- On Linux, WebKitGTK 4.1 and its build packages for the window (the installer names them). The MCP server and the command line do not need them.

## From source

```bash
./formmcp_oc_install      # checks the tools, installs the window's packages, fetches the crates
./formmcp                 # the window: Vite on localhost:5620 and the Tauri shell
./formmcp mcp             # the MCP server on stdio (built on first use)
./formmcp cli apps        # the family list as JSON; --mcp-only for the apps with a server
./formmcp cli app formvis # one app
./formmcp cli reach       # how each app's server would be started on this computer
```

To build the desktop app and its installers on your own machine: `./build_app` (macOS and Linux; `--no-install`, `--install`, `--open`, `--clean`, `--bundles deb,rpm,appimage`; the log is `build_app.log`) or `build_app.bat` on Windows. formcompiler runs the same scripts for releases.

Keep Form app checkouts beside this one (for example `~/code/formMCP` and `~/code/formvis`) and `./formmcp` finds them; `FORMMCP_DEV_ROOTS` names other folders. See [Agents](Agents) for every route and setting.

On Windows: `formmcp_oc_install.bat`, then `formmcp.bat`, `formmcp.bat mcp`, `formmcp.bat cli apps`.

## Refreshing the family list

The list is generated from formcompiler's registry (`apps/<app>.json`, its `catalog` and `agents` blocks):

```bash
./formmcp family --compiler ../formcompiler          # writes core/data/family.json
./formmcp family --compiler ../formcompiler --check  # fails while it drifts
```

The file records formcompiler's version and commit it came from; the window's footer and `family_apps` show it.

## Ports

| Port | Service |
| --- | --- |
| 5620 | Vite dev server (localhost only) |
| 5621 | Vite preview (localhost only) |

The MCP server uses stdio, no port.
