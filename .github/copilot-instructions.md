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
- **nice-view-gem**: Display module for the nice!view screen
- **zmk**: Main ZMK firmware (tracks latest main branch)

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

## Integration Points

### Hardware Dependencies

- **nice!view display**: Requires `nice-view-gem` shield
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
