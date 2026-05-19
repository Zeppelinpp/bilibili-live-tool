# HD Icons Design

## Goal
Replace all low-resolution bitmap app icons with a single SVG source, generating crisp icons for all platforms and display densities.

## Source Asset

A single `source.svg` placed at `src-tauri/icons/source.svg`.

- Canvas: 1024 x 1024
- Shape: rounded-rect body + two rounded-rect eyes + two antenna stubs
- Color: Bilibili brand blue `#00A1D6`
- Padding: ~10% on all sides so the icon does not touch the edge

## Generated Outputs

### App Icon Bundle
Run `cargo tauri icon source.svg` to regenerate the entire bundle:

- `icon.icns` — macOS app icon
- `icon.ico` — Windows app icon
- `icon.png` — generic / Linux
- `32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png`
- `Square30x30Logo.png` through `Square310x310Logo.png`
- `StoreLogo.png`
- `ios/AppIcon-*.png` (all 20 sizes)
- `android/mipmap-*/ic_launcher*.png` (all densities)

### Tray Icon (macOS)
A separate grayscale+alpha export from the same SVG:

- File: `src-tauri/icons/tray-icon-macos.png`
- Sizes: 64 x 64 and 128 x 128 (macOS picks the best match for @2x / @3x)
- Style: monochrome silhouette, no color fill, transparent background
- Template flag stays `true` in code so macOS tints it automatically

## Code Changes

None. The existing code in `main.rs` already loads `tray-icon-macos.png` and the bundle config in `tauri.conf.json` already points to the generated icon files.

## Success Criteria

- [ ] `source.svg` exists and renders the blue TV shape accurately
- [ ] `cargo tauri icon source.svg` completes without errors
- [ ] All generated PNG/ICNS/ICO files are larger than their previous versions
- [ ] Tray icon appears crisp on a Retina MacBook Pro menu bar
- [ ] macOS Dock icon and Finder "Get Info" preview appear crisp
- [ ] `npm run tauri-dev` starts without icon-related errors
