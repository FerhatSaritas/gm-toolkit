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

## Install (Arch Linux, x86_64)

Build once, then copy the portable binary to any machine (desktop, laptop):

```bash
# from the repo, on the machine that has the toolchain:
APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=true npm run tauri build
```

The AppImage lands in `src-tauri/target/release/bundle/appimage/` and is fully
portable — no install step, no root:

```bash
cp "src-tauri/target/release/bundle/appimage/GM Toolkit_0.1.0_amd64.AppImage" /wherever/
chmod +x "GM Toolkit_0.1.0_amd64.AppImage"   # usually already executable
./"GM Toolkit_0.1.0_amd64.AppImage"          # double-click also works in Plasma
```

Both machines must be `x86_64` (`uname -m` to check). The NVIDIA/Wayland
rendering workaround is compiled in — no environment variables needed.

Notes:
- The `.deb` in `bundle/deb/` is for Debian/Ubuntu only — **not** for Arch.
- Optional: keep a `~/Applications` folder for AppImages and let Plasma's
  app launcher index it.
- Campaign data is a plain folder you choose (e.g. synced via git or Syncthing),
  so desktop and laptop can share the same campaign.

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
