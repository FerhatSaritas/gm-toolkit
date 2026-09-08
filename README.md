# gm-toolkit

A simple and private GM toolkit to run TTRPGs as a desktop client. All data stored on device — plain Markdown + YAML frontmatter in a campaign folder you control.

Built with Tauri 2 (Rust) + Svelte 5. Design system: [DESIGN.md](DESIGN.md) — "Obsidian Grimoire".

## Development

```bash
npm install         # once
npm run tauri dev   # run the desktop app in dev mode
npm run tauri build # release build (.deb + AppImage into src-tauri/target/release/bundle)
                    # without FUSE: APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=true npm run tauri build
```

Frontend-only: `npm run dev` / `npm run check`. Backend: `cargo check` / `cargo test` in `src-tauri/`.

Keyboard: `Ctrl+K` global search · `Space` next turn · `J/K` select combatant · `?` shortcuts.

## Campaign layout

Point the app at any folder. Conventional subfolders:

```
campaign/
  scenes/      narrative locations & events
  encounters/  combat blueprints (combatants, tactics)
  npcs/        NPC statblocks
  statblocks/  monster statblocks
  items/       items & treasure
  party/       player characters
  .state/      live encounter state (session-only)
```
