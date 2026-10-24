# NOTICE

formMCP
Copyright 2026 NodeAgentc, Inc.

Licensed under the GNU General Public License v3.0 (SPDX `GPL-3.0-only`). See `LICENSE`. The third-party work listed below keeps its own licence; all of it is compatible with GPL-3.0.

## Dependencies

formMCP's own code and art (the icon in `assets/`, `frontend/public/` and `src-tauri/icons/`) are the owner's. The family list in `core/data/family.json` is generated from formcompiler's registry (`nodemode-ai/formcompiler`, GPL-3.0-only, the same owner). The window's design tokens and layer scale come from FormTray (`nodemode-ai/formtray`, GPL-3.0-only, the same owner).

Its dependencies are MIT, Apache-2.0 or dual MIT and Apache-2.0, with some BSD-3-Clause, ISC, Zlib, Unicode-3.0 and CC0 crates, as their manifests declare (checked with `cargo metadata` against `Cargo.lock`, 386 packages in the shell's graph). The main ones:

| Package | Used for |
| --- | --- |
| serde, serde_json | the family list and MCP messages (`core`, `cli`) |
| tauri 2, tauri-build | the desktop shell |
| Vite, TypeScript, @tauri-apps/cli, @types/node (build only) | building the window and the app; not shipped |

Every new dependency, binary, font or data file gets a row here before it is used.

## Desktop shell: crates under other licences

Tauri links these crates unedited (no `[patch]` and no vendored copy):

| Crate | Licence | Why it is there |
| --- | --- | --- |
| cssparser, cssparser-macros, selectors, dtoa-short, option-ext | MPL-2.0 | Tauri's HTML handling and its folder lookups; used unmodified, source at crates.io |
| r-efi | MIT OR Apache-2.0 OR LGPL-2.1-or-later | used under MIT |

The Linux AppImage also carries the build machine's WebKitGTK, GTK and GLib (LGPL-2.1-or-later) as separate shared libraries.

## Trademarks

Claude Code is a product of Anthropic, Codex of OpenAI and Cursor of Anysphere; they are named only as the agent clients formMCP connects to. The names of the Form apps belong to their owner. formMCP is not affiliated with or endorsed by Anthropic, OpenAI or Anysphere.
