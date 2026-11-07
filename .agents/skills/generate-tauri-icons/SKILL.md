---
name: generate-tauri-icons
description: Generate a Tauri app's PNG, ICO, and ICNS icons from a square SVG or PNG, then rebuild the executable that actually embeds them. Use when the user asks to generate, update, or convert Tauri icons, or when a desktop window still shows an old icon after the SVG changed.
---

# Generate Tauri icons

The window and taskbar icon are not the SVG in `public/`. They are the files in `src-tauri/icons`, embedded into the executable when Cargo compiles it.

## Generate

From the package that has `@tauri-apps/cli` and `src-tauri/tauri.conf.json`:

```bash
pnpm exec tauri icon path/to/icon.svg
```

The source must be a square SVG or PNG with transparency. For a non-square source, pass `--fit contain` or `--fit cover`. Fill colors must be hex or `rgb()`. `oklch()` fails to parse and the rasterizer falls back to black. Output defaults to the `icons` directory next to `tauri.conf.json`.

This writes `icon.png`, `32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png`, `icon.ico`, `icon.icns`, and the Windows Store `Square*Logo.png` files. It also writes `icons/ios` and `icons/android`. Delete those two directories when the app is desktop-only.

`tauri.conf.json` `bundle.icon` should keep listing `icons/32x32.png`, `icons/128x128.png`, `icons/128x128@2x.png`, `icons/icon.icns`, and `icons/icon.ico`.

## Rebuild the binary the user launches

Changing the icon files does not change a running or previously built app. `tauri dev` embeds `icon.ico` at compile time.

Build into the workspace target the dev command uses, usually `<repo>/target`. If `CARGO_TARGET_DIR` points at a sandbox or temp cache, override it for this build. Otherwise the new exe is not the one `tauri dev` starts.

```bash
cargo build
```

Run that from `src-tauri`, with `CARGO_TARGET_DIR` set to the repo `target` directory when the environment points elsewhere.

If Cargo finishes without compiling and `icon.ico` is newer than `<target>/debug/<package>.exe`, the build script fingerprint is stale. Delete `<target>/debug/build/<package>-*`, then run `cargo build` again. Do not `cargo clean -p` the whole package unless that delete fails. Confirm the exe is newer than `icon.ico`.

Restart the desktop process. A frontend reload does not reload the native icon. Windows may keep a cached taskbar image until that process starts again. The title-bar icon comes from the rebuilt exe.
