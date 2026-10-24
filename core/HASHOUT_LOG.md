# HASHOUT_LOG · core · formMCP

Living log for `formmcp-core`, `formmcp-cli` and `setup/family.py`. Source files stay comment-free; what they would have said lives here.

Current build version: v0.1.0
Last updated: 2026-10-24 · the family list and the MCP server

## v0.1.0 · the family list and the MCP server {#v0-1-0}

### family.rs · the list is compiled in
- refers-to: core/src/family.rs :: BUNDLED, Family::bundled, Family::get, describe_source
- kind: decision
- logged: v0.1.0 · 2026-10-24

`core/data/family.json` is read with `include_str!`, so the engine, the command line and the installed app all carry the same list and none of them reads a file at run time. `bundled` checks the `format` and returns an error rather than panicking, so a bad regeneration shows up as a message from `formmcp mcp`, not a crash. `get` matches the catalogue id, the registry's app name or the display name, case-insensitively (`formvis`, `FormVis`, `formai` and `formportal` all work). The signed catalogue (`apps.json`) becomes the first source in a later version, with this file as the offline fallback.

### setup/family.py · generated from formcompiler's registry
- refers-to: setup/family.py :: build, main
- kind: decision
- logged: v0.1.0 · 2026-10-24

The family plan forbids hand-kept copies. The generator reads formcompiler's `apps/*.json`: the catalogue id, name, kind and summary, the repository, `state`, `enabled` (written as `released`) and the `agents` block (formcompiler 0.4.0). It records formcompiler's VERSION and commit. Output is sorted by id with a trailing newline, so regenerating from the same commit changes nothing; `--check` exits 1 while the file drifts. formMCP, formskills and formranker are not in formcompiler's registry yet (family plan, Phase 5), so they are not in the list.

### mcp.rs · one tool, stdio, no dependency
- refers-to: core/src/mcp.rs :: tools, family_apps, answer, serve
- kind: decision
- logged: v0.1.0 · 2026-10-24

The server is hand-rolled on `serde_json` in FormPortal's pattern (`cli/src/mcp.rs` there), so formMCP adds no MCP crate: newline-delimited JSON-RPC on stdin and stdout, batches answered as arrays, notifications (no `id`) answered with nothing, a parse error answered with -32700, an unknown method with -32601. It negotiates the protocol version the client asks for when it is one of the four FormPortal supports. `family_apps` returns the short fields only (no repository, no `released`), about 3.8 KB for 21 apps, both as text and as `structuredContent`. The instructions say formMCP never installs and that FormPortal does, which is the plan's rule that formMCP suggests and FormPortal installs. Checked by hand through `formmcp-cli mcp` and the shell's `formmcp mcp`: initialize, tools/list, family_apps with and without `mcp_only`, an unknown tool, a line that is not JSON.

### cli/src/main.rs · the same engine, no window
- refers-to: cli/src/main.rs :: main, HELP
- kind: rationale
- logged: v0.1.0 · 2026-10-24

`mcp` is checked before the list is loaded so the server reports its own errors on stderr; `version` and `help` need no list. `apps` adds `--mcp-only`; `app <id>` prints one app or exits 1 with the reason.
