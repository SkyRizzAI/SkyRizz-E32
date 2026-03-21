# SkyRizz E32

SkyRizz E32 is a Solana-oriented development board built around an `ESP32-S3-WROOM-1-N16R8` and an `SE050` secure element. This repository currently centers on the hardware design package in `PCB/`, including manufacturing outputs, prototype imagery, the EasyEDA source archive, and firmware-oriented pinout references.

## Highlights

- `ESP32-S3-WROOM-1-N16R8` main module
- `SE050` secure element integration
- Shared onboard I2C bus for sensors and control ICs
- External headers labeled `IO 1`, `I2C`, `IO 2`, and `IO 3`
- Firmware bring-up references under `PCB/pinout/`

## `PCB/` contents

The `PCB/` directory now includes the full current hardware package:

| Path | Description |
| --- | --- |
| `SKYRIZZ-E32-Schematic-v1.0.pdf` | Versioned schematic export for the board |
| `PCB-design-alpha.pdf` | PCB layout/design export |
| `BOM-SKYRIZZ-E32-v1.0.xlsx` | Bill of materials for sourcing parts |
| `PickAndPlace_E32-v1.0.xlsx` | Placement data for assembly |
| `skyrizz_e32_se050.epro` | EasyEDA Pro project source archive |
| `pcb-prototype-alpha-front.jpeg` | Front-side prototype photo |
| `pcb-prototype-alpha-back.jpeg` | Back-side prototype photo |
| `assets/` | Logos, board renders, and supporting images |
| `pinout/pin_map.md` | Module-by-module wiring map from board parts to firmware paths |
| `pinout/pin_capabilities.md` | ESP32-S3 pin capabilities cross-referenced with this PCB |
| `pinout/no_std_board_pins.rs` | Reusable Rust pin definitions for board bring-up |
| `pinout/no_std_board_self_test.rs` | HAL-agnostic self-test harness for board validation |

## External headers at a glance

The updated pinout references document the board-silk connector names and how firmware reaches them:

- `IO 1` / `C_P0`: exposes `P0` on native `GPIO1`
- `I2C` / `C_I2C`: direct breakout of `GPIO48` (`SCL`) and `GPIO47` (`SDA`)
- `IO 2` / `C_P1-3`: `P1`-`P3` through the onboard `XL9535` I/O expander
- `IO 3` / `C_P4-7`: `P4`-`P7` through the onboard `XL9535` I/O expander

Only `IO 1` is a direct native ESP32 GPIO breakout. `IO 2` and `IO 3` are I2C-backed GPIOs exposed through the `XL9535`, which is useful to know when planning latency-sensitive firmware or external attachments.

## Getting started

### For hardware builders

1. Review `SKYRIZZ-E32-Schematic-v1.0.pdf` and `PCB-design-alpha.pdf`.
2. Use `BOM-SKYRIZZ-E32-v1.0.xlsx` and `PickAndPlace_E32-v1.0.xlsx` for sourcing and assembly.
3. Open `skyrizz_e32_se050.epro` in EasyEDA Pro if you want to inspect or modify the original design.
4. Compare assembly and routing details against the prototype photos and assets in `PCB/`.

### For firmware bring-up

1. Start with `PCB/pinout/pin_map.md` for the board-level wiring overview.
2. Use `PCB/pinout/pin_capabilities.md` when you need to know both ESP32-S3 capabilities and existing board assignments.
3. Reuse `PCB/pinout/no_std_board_pins.rs` and `PCB/pinout/no_std_board_self_test.rs` as bring-up references for custom firmware.

## Project status

The board is still in an alpha/prototype phase, but the repository now includes the current schematic, manufacturing data, EasyEDA source archive, and firmware-facing pinout references for the latest documented hardware package.

## Contributing

Issues and pull requests are welcome, especially for documentation, board validation notes, and firmware bring-up references.

## License

This project is licensed under the GNU General Public License v3.0 or later. See `LICENSE` for the full text.
