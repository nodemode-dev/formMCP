# Branching

- `main` is always releasable. Work lands on `main` in small commits by `nodemode-ai <mackin@nodemode.ai>`, with no attribution trailers.
- A release is a branch `compile_vX.Y.Z` cut from `main` after the version bump commit. CI runs on it, and formcompiler (`nodemode-ai/formcompiler`) builds it with `./build_app` and drafts the release `formMCP-vX.Y.Z` from it (`apps/formMCP.json` there). This repository's CI builds nothing itself.
- Versions are `vX.Y.Z` with dots: patch for fixes, minor for a feature, major for a breaking change. The version lives in `Cargo.toml` (workspace), `Cargo.lock` (the three crates), `src-tauri/tauri.conf.json`, `frontend/package.json` and its lock, the README badge and `CHANGELOG.md`.
