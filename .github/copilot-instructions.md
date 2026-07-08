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

The physical screen is **160w × 68h landscape**. There are TWO coordinate spaces:

1. **Canvas-drawn widgets** (layer, battery, profile, output, WPM gauge, raw-hid): drawn on a **68w × 160h portrait canvas rotated 90°**. So `CUSTOM_X/Y` are portrait coords, and on-screen `landscape_X ≈ 159 − portrait_Y`, `landscape_Y ≈ portrait_X`. Higher `_Y` = further LEFT on screen.
2. **LVGL object widgets** (bongo cat, luna, HID indicators, modifier icons): use **direct landscape coords** (`X` 0–159, `Y` 0–67).
   - Modifier icons render as a row offset from `MODIFIERS_CUSTOM_X/Y`: win/alt/shf/ctl at `X+37/+51/+65/+78`, all at `Y−44`.

### Key gotchas

- **Module defaults are per-widget and are NOT designed to all coexist on 160×68.** Enabling everything (the module default) overlaps widgets — this is the "everything in the wrong place" chaos, NOT a RAM/perf problem. Fix by curating a subset and positioning explicitly.
- **The layer-name font is hardcoded** (`layer.c` → `pixel_operator_mono_16`); there is NO Kconfig to enlarge it. It's already the largest font shipped.
- **Battery labels**: `CONFIG_..._CENTRAL_SHOW_BATTERY_PERIPHERAL_ALL=y` shows both halves AND suppresses the hardcoded "SIG" label; requires `CONFIG_ZMK_SPLIT_BLE_CENTRAL_BATTERY_LEVEL_PROXY=y` for the right half to report.
- **BT profile number**: no disable flag; set `CONFIG_NICE_OLED_WIDGET_PROFILE_BIG=n` to shrink it.
- **CapsLock as a pet**: the HID-indicators widget has `LUNA` and `BONGO_CAT` sub-modes. Setting `CONFIG_NICE_OLED_WIDGET_HID_INDICATORS_BONGO_CAT=y` + `..._BONGO_CAT_ONLY_CAPSLOCK=y` makes ONE widget act as a CapsLock indicator drawn as a bongo cat (no `ZMK_WPM` dependency — that's the WPM bongo cat, a different widget). Position via `..._HID_INDICATORS_CUSTOM_X/Y` (direct landscape coords; ePaper default `100,8` overlaps the battery).

### Current curated layout (left → right on the 160-wide screen)

| Zone     | Widget                         | Position                          |
| -------- | ------------------------------ | --------------------------------- |
| X≈13     | Layer name (largest font)      | canvas, portrait `0,146`          |
| X≈30     | BT profile number (small)      | `PROFILE_BIG=n`                   |
| X≈60–101 | Modifiers `⊞ Alt ⇧ ⌃` icon row | `MODIFIERS_CUSTOM_X=23,Y=62`      |
| X≈55–95  | CapsLock indicator (bongo cat) | `HID_INDICATORS_CUSTOM_X=55,Y=33` |
| X≈140    | Battery `L% R%` (no labels)    | canvas, portrait `26,19`          |
| X≈159    | Output / signal (BT/USB)       | canvas, portrait `49,0`           |

### ePaper default positions (portrait canvas X,Y — from Kconfig.defconfig)

`layer 0,146` · `battery 26,19` · `profile 18,129` · `output-bt 49,0` · `output-usb 45,2` · `modifiers 62,62` · `hid-indicators (landscape) 100,8` · `bongo-cat (landscape) 100,8` · `luna (landscape) 100,15`. When adding a widget, place it in a free landscape-X zone and avoid the battery cluster (landscape X≈140–159).

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
