# Extensions

FormMCP loads extensions made to the Form Extension Contract, API 1. FormPortal installs them and form.ext runs one straight from a folder; FormMCP itself never installs anything. An extension adds commands and panels to FormMCP's window and works on FormMCP's own settings (which apps each agent sees, which apps always show their tools, start policies, tools turned off, profiles), never on the other Form apps.

## Using them

- **Extensions** in the filter row, beside Profiles and Safety, opens a menu: one item per command an extension adds (`Name: Title`), then Extensions Panel, Reload Extensions and Manage in FormPortal.
- **The Extensions panel** has one row per extension: name, version, where it came from (Catalogue, File, Development), the permissions it asks for, and its state: Ready, Needs Permission, or Refused with the reason (made for another app, needs a newer FormMCP, a native extension, a broken manifest). Open shows its panels; a development extension shows its folder.
- **The first time** a version of an extension runs, FormMCP asks you to allow what it asks for (read and change FormMCP's settings, open files you choose, save files where you choose). Not Now leaves it closed, and FormMCP asks again the next time. A new version, or one that asks for more, asks again.
- **Undo**: whatever an extension changes in one go is one step. Undo in the filter row puts the settings back as they were before it, until you change something yourself.
- **Panels** open under the filter row, titled with the extension's name, or as a dialog. Close ends them.
- FormMCP checks for new or changed extensions every 2 s while its window has focus; Reload Extensions checks at once. Only the rows that changed are redrawn.

## Not open to extensions

Secrets, hooks, tracing, the HTTP server and its token, the safety switches, connecting agents, and exporting or importing profiles stay with you in the window. FormMCP's window never reaches the internet, so extensions cannot either.

## Agents

`extensions_list` lists the extensions with their state and commands; `extension_command` runs one command in the window, which must be open, with the same consent. On the command line: `./formmcp cli extensions` and `./formmcp cli extension <id> <command> [args as JSON]`.

## Making one

The API, with the document FormMCP returns and the commands it accepts, is `docs/EXTENSIONS.md` in the repository. form.ext's Run in App writes `extensions/formmcp/dev.json` in FormPortal's folder; set `FORMPORTAL_ROOT` to try one in a folder of your own. In a browser, `npm run dev` in `frontend` (or the demo, `npx vite --config vite.demo.config.ts`) serves the same list and files through `formmcp-cli`, so a development extension can be tried without the desktop app.
