# gm-toolkit

Local-first GM toolkit for running TTRPGs on Linux (Tauri 2 + Svelte 5 + TypeScript, Rust backend). All campaign data is plain Markdown + YAML frontmatter stored in a user-chosen campaign folder. Design system: see `DESIGN.md` ("Obsidian Grimoire") — all UI must use the CSS tokens in `src/lib/tokens.css`.

## Commands

```bash
npm run dev          # Vite dev server (http://localhost:5173, strict port)
npm run check        # svelte-check + TS typecheck
npm run build        # svelte-check + vite production build
npm run tauri dev    # full desktop app in dev mode (runs npm run dev first)
npm run tauri build  # release build (.deb + AppImage into src-tauri/target/release/bundle)
                     # on hosts without FUSE: APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=true npm run tauri build

cargo check          # backend typecheck (run in src-tauri/)
cargo test           # backend unit tests (run in src-tauri/)
```

Rust toolchain lives in `~/.cargo` (rustup). If `cargo` is not on PATH, use `~/.cargo/bin/cargo`.

## Layout

- `src/` — Svelte 5 frontend (runes: `$state`, `$derived`, `$props`; stores in `src/lib/stores/*.svelte.ts`)
- `src/lib/tokens.css` — design tokens, do not hardcode colors/fonts
- `src-tauri/` — Rust backend; commands in `src-tauri/src/lib.rs`, campaign FS logic in `campaign.rs`, app config in `config.rs`
- Campaign folder (user-chosen) holds plain files: `scenes/`, `encounters/`, `npcs/`, `statblocks/`, `items/` + `party/`; live encounter state goes to `.state/` and must never be mixed into the Markdown blueprints.

## Conventions

- Markdown files are the source of truth; the app never rewrites them during play (only explicit user edits).
- All Tauri IPC commands must validate that file paths stay inside the campaign root (see `resolve_in_campaign`).
- Verify with `npm run check` and `cargo check` before finishing work.
