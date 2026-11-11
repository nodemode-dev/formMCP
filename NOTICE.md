# NOTICE

formMCP
Copyright 2026 NodeAgentc, Inc.

Licensed under the GNU General Public License v3.0 (SPDX `GPL-3.0-only`). See `LICENSE`. The third-party work listed below keeps its own licence; all of it is compatible with GPL-3.0.

## Dependencies

formMCP's own code and art (the icon in `assets/`, `frontend/public/` and `src-tauri/icons/`) are the owner's. The family list in `core/data/family.json` is generated from formcompiler's registry (`nodemode-dev/formcompiler`, GPL-3.0-only, the same owner). The window's design tokens and layer scale come from FormTray (`nodemode-dev/formtray`, GPL-3.0-only, the same owner).

Its dependencies are MIT, Apache-2.0 or dual MIT and Apache-2.0, with some BSD-3-Clause, ISC, Zlib, Unicode-3.0, CDLA-Permissive-2.0 and CC0 crates, as their manifests declare (checked with `cargo metadata` against `Cargo.lock`, 420 packages in the shell's graph at v0.3.0). The main ones:

| Package | Used for |
| --- | --- |
| serde, serde_json | the family list and MCP messages (`core`, `cli`) |
| ureq 2 (rustls, ring, webpki-roots) | reading the family catalogue over https (`core`); ring is Apache-2.0 and ISC, webpki-roots (the root certificates as data) CDLA-Permissive-2.0 |
| ed25519-dalek 2 (curve25519-dalek, subtle), base64 | checking the catalogue's signature (`core`); the dalek crates and subtle are BSD-3-Clause |
| getrandom 0.2 | the HTTP access token and session ids, from the operating system's random source (`core`); MIT OR Apache-2.0, already in the graph through the dalek crates |
| keyring 3 (security-framework on macOS, windows-sys on Windows, dbus and dbus-secret-service with aes, cbc, hkdf, num and zeroize on Linux) | secrets such as form.md's token, kept in the system keychain (`core`, v0.7.0); all MIT OR Apache-2.0. On Linux it links the system's libdbus (AFL-2.1 OR GPL-2.0-or-later) at run time; libdbus is not shipped or built into formMCP |
| regex 1 (regex-automata, regex-syntax, aho-corasick, memchr) | finding token-shaped text to hide in results and the log (`core`, v0.7.0); MIT OR Apache-2.0, aho-corasick and memchr Unlicense OR MIT |
| sha2 0.10 | the digests of an installed app's files (`core`, v0.7.0); MIT OR Apache-2.0, already in the graph through the dalek crates |
| toml_edit 0.25 (toml_parser, toml_datetime, winnow, indexmap), parse feature only | checking that Codex's config.toml is valid TOML before and after formMCP edits it (`core`, v0.10.4); toml_edit, toml_parser and toml_datetime MIT OR Apache-2.0, winnow MIT, indexmap Apache-2.0 OR MIT; all already in Cargo.lock through the desktop shell's build crates (proc-macro-crate, tauri-build), so nothing new is downloaded; now shipped in the CLI and the app |
| tauri 2, tauri-build | the desktop shell |
| Vite, TypeScript, @tauri-apps/cli, @types/node (build only) | building the window and the app; not shipped |
| `demo/index.html` and `demo/assets/` (v0.8.0) | the offline browser demo, compiled by Vite (MIT) from this repository's own window, tokens, family list and the TypeScript in `demo/src/`; the icon is the app's own; no third-party code, font or media is added |

| `demo/assets/media/` (v0.8.1) | the demo's pictures: the Form family's own app icons from the sibling repositories and the verified demo screenshots of nine Form apps, downscaled; every file, author, source and licence, and the Pexels, NASA and public-domain media visible in the screenshots, is listed in `demo/CREDITS.md` |
| `demos/connect/assets/media/` (v0.8.4) | demo 2's pictures: the Form family's own app icons from the sibling repositories (the same files as demo 1) and formMCP's own window mark; every file, author, source and licence is listed in `demos/connect/CREDITS.md` |
| Form Extension Contract kit, Apache-2.0, nodemode (v0.9.0) | the extension host: `frontend/src/ext/host-bridge.ts` is the kit's `host/host-bridge.ts` unchanged; `core/src/extensions.rs` is a line-for-line Rust port of the kit's `host/python/formext_host.py` (resolver and safe file serving) keeping its messages. Apache-2.0 is compatible with GPL-3.0 |
| `demos/safety/assets/` (v0.8.4) | demo 3's pictures: the Form family's own app icons from the sibling repositories (the same files as demo 1) and formMCP's own window mark; every file, author, source and licence is listed in `demos/safety/CREDITS.md` |
| The three demos inside the app (v0.10.0) | `frontend/demos-bundle.ts` copies the built demo pages above (`demo/index.html`, `demos/connect/index.html`, `demos/safety/index.html` and each `assets/` folder) unchanged into the window's bundle at `demo/` and `demos/<name>/`; nothing new is added, and every shipped file stays credited in that demo's `CREDITS.md`. The Demos panel's previews are demo 1's `mcp-hero.webp`, `mcp-agents.webp` and `mcp-safety.webp`, formMCP's own screenshots |

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
