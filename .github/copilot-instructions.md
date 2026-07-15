# AI Coding Agent Instructions for Eyelash Corne ZMK Configuration

## Project Overview

This is a ZMK (Zephyr Mechanical Keyboard) firmware configuration for the **Eyelash Peripherals Corne** keyboard - a custom split keyboard that differs from the standard Corne and requires specialized firmware. The project manages custom keymaps, mouse/pointing device integration, RGB lighting, and advanced input behaviors.

## Architecture & Key Components

### Core Configuration Files

- **`config/eyelash_corne.keymap`** - Main keymap with 4 layers, homerow mods, combos, and macros
- **`config/west.yml`** - ZMK module dependencies and hardware board definitions
- **`build.yaml`** - Build targets for left/right halves, Studio support, and settings reset
- **`boards/arm/eyelash_corne/`** - Hardware-specific device tree files and configurations

### Key Architecture Patterns

#### Layer Structure (4-layer system)

```c
Layer 0: "Rizvi" (base QWERTY with homerow mods)
Layer 1: "NUMBER" (numbers, Bluetooth, media controls)
Layer 2: "SYMBOL" (symbols, mouse controls, special functions)
Layer 3: "Fn" (function keys, system controls, bootloader access)
```

#### Homerow Modifications with Positional Awareness

```c
#define KEYS_L 0 1 2 3 4 5 13 14 15 16 17 18 28 29 30 31 32 33 34  // Left hand keys (all left side positions)
#define KEYS_R 6 7 8 9 10 11 12 19 20 21 22 23 24 25 26 27 35 36 37 38 39 40 41  // Right hand keys (all right side positions)
#define THUMBS 34 36 37 38 39 40 41  // All thumb keys (including space bar at 39)
#define POINTING 6 19 20 21 35  // Joystick cluster: UP, LEFT, CENTER, RIGHT, DOWN
```

- Uses `MAKE_HRM` macro to generate position-aware hold-tap behaviors
- Left/right hand mods trigger only on opposite hand keys + thumbs + pointing device

#### Pointing Device Integration

- **Joystick positions**: UP(6), LEFT(19), CENTER/CLICK(20), RIGHT(21), DOWN(35)
- **Mouse acceleration**: Configurable via `&mmv` and `&msc` with custom scaling
- **Scroll optimization**: Enhanced with 3x scaling and reduced response times

## Critical Development Workflows

### Local Keymap Visualization

```bash
# Generate SVG keymap visualization
./generate_keymap.bat
# Uses keymap-drawer with custom Windows-focused styling in keymap_drawer.config.yaml
```

### Build Process

- **GitHub Actions**: Auto-builds on config changes via `.github/workflows/build.yml`
- **Local Testing**: Use ZMK's west build system or GitHub workflow dispatch
- **Targets**: Produces left/right firmware, Studio-enabled builds, and settings reset
- **Host-side RAW HID app**: local patched clone lives at `C:\Users\RabR\dev\qmk-hid-host`; repo backups of the patched files live under `host_patches/qmk-hid-host/` with restore script `host_patches/apply_qmk_hid_host_patches.ps1`

### Module Dependencies

- **eyelash_corne**: Hardware board definition (external GitHub module)
- **zmk-nice-oled** (`mctechnology17`): Display widget module; the **left/central** half uses the `nice_epaper` shield (`CONFIG_NICE_EPAPER_ON=y`)
- **hammerbeam-slideshow** (`GPeye`): Static image slideshow on the **right/peripheral** half
- **zmk**: Main ZMK firmware (pinned to `v0.3.0` in `config/west.yml`)

## Project-Specific Conventions

### Keymap Patterns

1. **Combo Definitions**: Use `key-positions` with physical matrix positions, not logical keys
2. **Sensor Bindings**: Different per layer (scroll, volume, brightness, workspace switching)
3. **Macro Structure**: Alt-Tab uses key toggle `&kt` for persistent modifier state
4. **RGB Controls**: Centralized in NUMBER layer with full brightness/color/effect control

### Configuration Constants

```c
#define ZMK_POINTING_DEFAULT_MOVE_VAL 1200  // Mouse movement sensitivity
#define ZMK_POINTING_DEFAULT_SCRL_VAL 40    // Scroll wheel sensitivity
#define QUICK_TAP_MS 175                    // Homerow mod quick-tap threshold
```

### Windows-Optimized Keybindings

- VS Code shortcuts: `LC(SLASH)` (toggle comment), `F2` (rename)
- Window management: `LA(TAB)` (Alt-Tab), `LC(W)` (close tab)
- Bluetooth and output switching integrated into NUMBER layer

## Display Layout (nice-oled / nice_epaper)

The left/central `nice!view` screen is driven by the `zmk-nice-oled` module. All display config lives in [`config/eyelash_corne.conf`](../config/eyelash_corne.conf). The authoritative list of widget toggles and default positions is the module's `boards/shields/nice_oled/Kconfig.defconfig` (the module README table has at least one mismatch — trust the Kconfig).

### Coordinate system (critical)

The physical screen is **160w × 68h landscape**. The LVGL container is also **160×68** (landscape). But the canvas that canvas-drawn widgets render onto is **68w × 160h portrait**, and `rotate_canvas()` rotates it 90° to fill the landscape frame.

There are **two** coordinate spaces:

1. **Canvas-drawn widgets** (layer, battery, profile, output, WPM gauge, raw-hid, **and `MODIFIERS_INDICATORS_FIXED`**): drawn on the **68×160 portrait canvas**. `CUSTOM_X/Y` are portrait coords: `landscape_X ≈ 159 − portrait_Y`, `landscape_Y = portrait_X`. Portrait X range 0–67, portrait Y range 0–159.
2. **LVGL object widgets** (WPM bongo cat, WPM luna, HID indicators, responsive bongo cat): placed as child LVGL objects using **direct landscape coords** (X 0–159, Y 0–67).

### Key gotchas

- **`MODIFIERS_INDICATORS_FIXED` is canvas-drawn, NOT an LVGL object.** Icons are placed at portrait `(base_x + i×16, base_y)` in HOR mode. Since portrait X is 0–67, `base_x + 3×16 ≤ 67` → `base_x ≤ 19`. The module’s ePaper default of `X=62` is therefore broken for HOR (icon 2–4 all off-canvas). **Use `FIXED_VER` with CENTER alignment instead** — the source hard-codes portrait positions `(27, 62/78/94/110)` which are all within bounds and produce a physical horizontal row.
- **Module defaults all coexist poorly on 160×68.** Enabling everything overlaps widgets — this is the “everything in the wrong place” chaos, NOT a RAM/perf problem. Curate a minimal subset.
- **The layer-name font is hardcoded** (`layer.c` → `pixel_operator_mono_16`); there is NO Kconfig to enlarge it.
- **Battery labels**: `CONFIG_..._CENTRAL_SHOW_BATTERY_PERIPHERAL_ALL=y` shows both halves AND suppresses the hardcoded “SIG” label; requires `CONFIG_ZMK_SPLIT_BLE_CENTRAL_BATTERY_LEVEL_PROXY=y`.
- **BT profile number**: no disable flag; set `CONFIG_NICE_OLED_WIDGET_PROFILE_BIG=n` to shrink it.
- **Do not trust coordinate math blindly for every widget.** Small moves must be validated on hardware. On this board, the profile indicator responded empirically to `PROFILE_CUSTOM_Y` as the reliable vertical/downward adjustment, while `PROFILE_CUSTOM_X` shifted it sideways in practice.
- **HID indicators (CapsLock) do NOT work on this display.** The bongo cat and luna sub-modes both render as a solid white rectangle on the nice!view 1-bit monochrome display. The plain (no animation) mode shows nothing (confirmed from source: just `lv_label_set_text("")`). There is no working CapsLock indicator without patching the module.
- **LVGL child-image widgets remain high-risk on this hardware.** Treat WPM Luna, WPM Bongo Cat, and HID indicator animation modes as likely white-box/freeze candidates unless the user explicitly wants to re-test them.
- **`CONFIG_NICE_OLED_WIDGET_OUTPUT_BACKGROUND` must stay `n`** unless deliberately re-testing the white rectangle behind the BT/USB icon.
- **Undefined Kconfig symbols break CI.** `CONFIG_NICE_OLED_SPLIT_TOTAL_DEVICES` is a C macro, not a Kconfig option; never assign it in `.conf`.

### RAW HID host constraints

- The wired media widget is `CONFIG_NICE_OLED_WIDGET_RAW_HID_MEDIA_PLAYER_SPOTIFY_MACOS`; despite the name, it works on Windows because it listens for raw HID byte `0xAE`.
- The module's firmware parser has an off-by-one bug in `boards/shields/nice_oled/src/raw_hid/hid.c`: it copies media text from `&data[1]` even though the host format is `[type, length, chars...]`. That wastes one byte on the length field and force-nulls the last byte of an 11-byte buffer.
- Practical consequence: the media widget has roughly **9 real visible characters total**, so separate artist/title multi-line layouts are fundamentally unstable without forking `zmk-nice-oled`.
- Current stable host-side workaround is the vendored `windows.rs` patch: a single `MediaTitle` packet path with a custom leading play/pause byte and a clamped 4/4 two-row payload. If the user wants to restore that behavior, prefer the files under `host_patches/qmk-hid-host/` over re-deriving it from scratch.
- The separate `MediaArtist` packet path is intentionally not used in the current stable workaround.
- Weather on Windows is supported only because the local host app is patched. Re-downloading the stock `windows.zip` from upstream will regress weather until the vendored patches are re-applied.

### Current curated layout (left → right on the 160-wide screen)

| Zone          | Widget                               | Position                                                          |
| ------------- | ------------------------------------ | ----------------------------------------------------------------- |
| X≈13          | Layer name (largest font)            | canvas, portrait `0,146`                                          |
| X≈30          | BT profile number (small)            | `PROFILE_BIG=n`; fine-tune with tiny `PROFILE_CUSTOM_Y` nudges    |
| X=49/65/81/97 | Modifiers `⊞ ⇧ Alt ⌃` horizontal row | canvas VER CENTER (hardcoded portrait `27,62–110`), physical Y=27 |
| X≈140         | Battery `L% R%` (no labels)          | canvas, portrait `26,19`                                          |
| X≈159         | Output / signal (BT/USB)             | canvas, portrait `49,0`                                           |

### ePaper default positions (portrait canvas X,Y — from Kconfig.defconfig)

`layer 0,146` · `battery 26,19` · `profile 18,129` · `output-bt 49,0` · `output-usb 45,2` · `modifiers-HOR 62,62` (broken: X=62 clips icons 2–4 off canvas) · `modifiers-VER-CENTER hardcoded 27,62` · `hid-indicators (LVGL, broken on 1-bit display) 100,8`.

## Integration Points

### Hardware Dependencies

- **nice!view display**: Driven by the `zmk-nice-oled` module in `nice_epaper` mode (NOT `nice-view-gem`)
- **RGB underglow**: WS2812 LED strip with custom animations
- **Rotary encoder**: Context-sensitive scroll/volume/navigation per layer
- **Split communication**: Wireless via nice!nano controllers

### External Services

- **GitHub Actions**: Auto-builds firmware on config push
- **Keymap Drawer**: Auto-generates visual keymap on config changes
- **ZMK Studio**: Real-time keymap editing (left half with Studio build)

## Common Modification Patterns

### Adding New Combos

```c
combo_name {
    timeout-ms = <50>;
    key-positions = <X Y>;  // Use physical matrix positions
    bindings = <&kp SHORTCUT>;
    layers = <0 1 2>;  // Specify active layers
};
```

### Modifying Mouse Behavior

- Adjust `&mmv_input_listener` and `&msc_input_listener` for sensitivity
- Modify `acceleration-exponent` and `time-to-max-speed-ms` for feel
- Use `&mmv MOVE_*` for directional movement in keybindings

### Layer-Specific Sensor Behavior

Each layer can override encoder behavior:

```c
sensor-bindings = <&scroll_encoder>;        // Default scroll
sensor-bindings = <&inc_dec_kp C_VOL_UP C_VOL_DN>;  // Volume
sensor-bindings = <&inc_dec_kp LC(TAB) LC(LS(TAB))>; // Window switching
```

When modifying this codebase, always test with `./generate_keymap.bat` to validate syntax before committing. The keymap visualization helps verify layer layouts and combo positioning.
