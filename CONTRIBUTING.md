# Contributing to FormMCP

Thank you for helping. FormMCP aims to be the one MCP server an agent needs for the
whole Form family, cheap on context and safe by default, and it gets there through real
use and real contributions.

## Where to start

- **Use it and report what goes wrong.** A clear bug report, with steps, the version,
  your platform and the agent client you connected, is one of the most useful things
  you can give.
- **Look at the open issues.** Comment on one before starting larger work, so nobody
  builds the same thing twice.

## Setting up

Clone the repository and run `./formmcp_oc_install`, then `./formmcp` (or
`formmcp_oc_install.bat` and `formmcp.bat` on Windows). The first checks Rust, Node
and, on Linux, the D-Bus and WebKitGTK packages; the second opens the window.
`./formmcp mcp` runs the MCP server on stdio. The README's Quick start has the details
and the requirements.

The code lives in:

| Folder | What is there |
| --- | --- |
| `core` | `formmcp-core`, the engine (Rust): the family list, reaching and running each app's server, the MCP server, secrets, redaction, verification, hooks and telemetry |
| `cli` | `formmcp-cli`, the command line (Rust) |
| `src-tauri` | the desktop shell (Rust, Tauri) |
| `frontend` | the window (TypeScript, Vite) |
| `setup` | `family.py`, which generates `core/data/family.json` from formcompiler's registry |
| `.docs/wiki` | the wiki's source |

## Before you open a pull request

Run the same checks CI runs:

```sh
(cd frontend && npm ci && npm audit && npm run typecheck && npm run build)
cargo check --locked -p formmcp-core -p formmcp-cli
cargo test --locked -p formmcp-core -p formmcp-cli
python3 -m py_compile setup/family.py
```

CI also checks the shell scripts with `bash -n` and refuses an em dash in tracked
Markdown or an emoji in any tracked text.

- Keep each pull request to one change, with tests for what it adds or fixes.
- Write commit subjects in Conventional Commits: `feat(core): ...`, `fix(cli): ...`,
  `docs(...)`. Leave the version alone; a maintainer bumps it when releasing (see
  [.docs/BRANCHING.md](.docs/BRANCHING.md)).
- `core/data/family.json` is generated: change formcompiler's registry and run
  `./formmcp family --compiler PATH`, never edit it by hand.
- FormMCP suggests and FormPortal installs: a tool that installs, updates or removes an
  app will not be accepted. FormMCP serves the Form family only, with no third-party or
  remote MCP servers.
- The HTTP server listens on 127.0.0.1 only and checks the access token, the Host and
  the Origin on every request. Secrets live only in the system keychain and never reach
  a file, the log, an error, a hook or a span.
- Every `z-index` comes from `frontend/src/styles/layers.css`, and every colour, space
  and radius from `frontend/src/styles/tokens.css`.
- Every new dependency, binary, font or data file gets a row in `NOTICE.md` before it
  is used.

## Maintainers

FormMCP is maintained by:

- the [nodemode-dev](https://github.com/nodemode-dev) organisation on GitHub
- [@nodemode-ai](https://github.com/nodemode-ai)
- [@gbarnai](https://github.com/gbarnai)

Reach them through issues and pull requests.

## How changes land

Only a maintainer pushes to `main`. Everyone else forks the repository and opens a pull request into `main`. A pull request merges when all four checks pass on an up-to-date branch and a maintainer has approved it after its last push.

## Releases

A release is a version, and only a maintainer makes one: after the version is bumped on
`main`, a maintainer cuts a `compile_vX.Y.Z` branch, and formcompiler
(`nodemode-dev/formcompiler`) builds every platform and publishes them together. No
push from a contributor starts a release. See [.docs/BRANCHING.md](.docs/BRANCHING.md).

## Licensing

FormMCP is licensed under the GNU General Public License v3.0 only
([LICENSE](LICENSE), SPDX `GPL-3.0-only`). By contributing, you agree that your
contribution is licensed under the same terms. Only add code you have the right to
contribute, and name any third-party code and its licence in the pull request.

## Patents and other people's IP

A feature may be held back or removed if it would infringe someone else's patent or
other intellectual property. The maintainers decide.

## Conduct

Everyone taking part follows the [code of conduct](CODE_OF_CONDUCT.md). Security
problems go through [SECURITY.md](SECURITY.md), never a public issue.
