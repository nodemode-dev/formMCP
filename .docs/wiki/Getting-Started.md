# Getting Started

## Requirements

- Rust (stable) for the engine, the command line and the shell.
- Node 20.19 or newer for the window.
- On Linux, libdbus and pkg-config (`libdbus-1-dev pkg-config` on Debian or Ubuntu) for the system keychain, and WebKitGTK 4.1 and its build packages for the window (the installer names them). The MCP server and the command line need only libdbus. Keeping a secret also needs a running, unlocked Secret Service (GNOME Keyring or KWallet), which desktops have.

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

The list is generated from the family spec, the `family.json` that nodemode-dev/formadaptor publishes on its `spec` branch (public apps only; nothing is fetched by the generator):

```bash
./formmcp family --family PATH          # writes core/data/family.json; keeps the spec when the list changed
./formmcp family --family PATH --check  # writes nothing; exits 1 and names the file while it drifts
```

The file names the registry's repository as its `source` (the window's footer and `family_apps` show it) and carries no registry version, so a new registry release that changes no public app leaves it as it is. When a spec changes the list, the generator also copies that spec to `setup/family.spec.json`, beside itself, and that copy records the registry version the list came from; a spec that differs only in what the list does not use (the registry version, the generation time, a licence, an app that is not public) changes nothing, so no commit is needed for it. CI runs `--check` against the kept copy and fetches nothing.

## Ports

| Port | Service |
| --- | --- |
| 5620 | Vite dev server (localhost only) |
| 5621 | Vite preview (localhost only) |

The MCP server uses stdio, no port. Over HTTP (`./formmcp cli serve`, or Start in the window) it listens on 127.0.0.1:5625 only and needs the access token from `./formmcp cli http token`; the Agents page has the rules.
