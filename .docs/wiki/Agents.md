# Agents

formMCP's MCP server is `formmcp mcp` (stdio). In this repository it is already wired:

| Client | In this repository |
| --- | --- |
| Claude Code | `.mcp.json` |
| Codex | `.codex/config.toml` |
| Cursor | `.cursor/mcp.json` |

All three run `./formmcp mcp`, which builds the command line on first use. An installed formMCP answers the same way: `<the formMCP program> mcp`.

## Tools

| Tool | Changes anything | Returns |
| --- | --- | --- |
| `family_apps` | no | every Form app (`id`, `name`, `kind`, `summary`, `mcp`, `skills`, `state`) and where the list came from; `mcp_only` keeps the apps with an MCP server |

The answer is short on purpose: one line per app and no app's tools, so adding formMCP costs little context. The server's instructions tell the agent that formMCP never installs and that FormPortal installs and updates the Form apps.

The `formmcp-harness` skill (`.claude/skills/formmcp-harness`) teaches the workflow.
