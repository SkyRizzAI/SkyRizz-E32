# SkyRizz E32

SkyRizz E32 is a Solana-oriented development board built around an `ESP32-S3-WROOM-1-N16R8` and an `SE050` secure element. This repository includes the hardware design package in `PCB/`.

## Highlights

- `ESP32-S3-WROOM-1-N16R8` main module
- `SE050C2HQ1/Z01SDZ` secure element integration
- Shared onboard I2C bus for sensors, touch, security, and GPIO expansion
- LCD, touch, camera, audio, microSD, and RGB LED support on the board
- External headers labeled `IO 1`, `I2C`, `IO 2`, and `IO 3`
- Firmware-oriented references under `PCB/pinout/`

## Hardware specs

| Area | Details |
| --- | --- |
| Main compute module | `ESP32-S3-WROOM-1-N16R8` |
| Wireless | Integrated `ESP32-S3` Wi-Fi + Bluetooth LE |
| Memory variant | `N16R8` module variant (`16 MB` flash / `8 MB` PSRAM) |
| Security | `SE050C2HQ1/Z01SDZ` secure element on the shared I2C bus |
| I/O expansion | `XL9535QF24` 16-bit I2C GPIO expander |
| Environmental sensors | `AHT20` temperature/humidity, `LTR-303ALS-01` ambient light, `SC7A20HTR` accelerometer |
| Display and touch | LCD flex connector (`FPC1`) plus `TSC2007IPWR` touch controller |
| Camera | Camera flex connector (`FPC3`) with direct ESP32 camera GPIO routing |
| Audio | `ES7243E` audio ADC, `NS4168` speaker amplifier, `2x MSM381ACP003` microphones |
| Storage | `TF1` microSD socket and `GT30L24A3W` SPI ROM / font chip |
| USB | USB Type-C with native USB FS on `GPIO19` / `GPIO20` |
| LEDs | `2x XL-0807RGBC-2812B` WS2812-style RGB LEDs plus a separate indicator LED |
| External expansion | Direct `GPIO1`, shared I2C breakout, and `XL9535`-backed `P1`-`P7` expansion headers |

## Modules and major ICs used on the board

| Category | Module / refdes | Part | Role |
| --- | --- | --- | --- |
| Main controller | `U1` | `ESP32-S3-WROOM-1-N16R8` | Main MCU and wireless module |
| Security | `U18` | `SE050C2HQ1/Z01SDZ` | Secure element |
| I/O expansion | `U9` | `XL9535QF24` | 16-bit GPIO expander for external pins, resets, buttons, and indicator LED |
| Sensors | `HUM` | `AHT20` | Temperature and humidity sensing |
| Sensors | `LS` | `LTR-303ALS-01` | Ambient light sensing |
| Sensors | `U5` | `SC7A20HTR` | Accelerometer |
| Touch | `U10` | `TSC2007IPWR` | Resistive touch controller |
| Display | `FPC1` | LCD flex connector | LCD interface and backlight path |
| Touch/control flex | `FPC2` | Touch/control flex connector | Shared I2C and touch control breakout |
| Camera | `FPC3` | Camera flex connector | Camera interface |
| Audio input | `U14` | `ES7243E` | Audio ADC |
| Audio output | `U13` | `NS4168` | Speaker amplifier |
| Audio input | `MIC1`, `MIC2` | `MSM381ACP003` | Microphones |
| Storage | `TF1` | TF / microSD socket | Removable storage |
| Storage | `U2` | `GT30L24A3W` | SPI ROM / font chip |
| LEDs | `RGB1`, `RGB2` | `XL-0807RGBC-2812B` | Addressable RGB LEDs |
| Indicator | `IND` | `KT-0603YG` | User / status LED |
| USB | `USB1` | `USB-TYPE-C-018` | USB Type-C connector |
| External headers | `C_I2C`, `C_P0`, `C_P1-3`, `C_P4-7` | JST / board connectors | I2C and external GPIO breakout |
| Power connector | `BATT` | `PH-2P` | Battery connector |

## Repository layout

| Path | Purpose |
| --- | --- |
| `PCB/` | Hardware design package, manufacturing files, imagery, and pinout references |

## `PCB/` contents

The `PCB/` directory includes the current hardware package:

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

### For firmware and board bring-up

1. Start with `PCB/pinout/pin_map.md` for the board-level wiring overview.
2. Use `PCB/pinout/pin_capabilities.md` when you need to know both ESP32-S3 capabilities and existing board assignments.
3. Reuse `PCB/pinout/no_std_board_pins.rs` and `PCB/pinout/no_std_board_self_test.rs` as bring-up references for custom firmware.

## Project status

The board is still in an alpha/prototype phase, but the repository now includes the current schematic, manufacturing data, major board module inventory, EasyEDA source archive, webpage assets, and firmware-facing pinout references for the latest documented hardware package.

## Contributing

Issues and pull requests are welcome, especially for documentation, board validation notes, and firmware bring-up references.

## License

This project is licensed under the GNU General Public License v3.0 or later. See `LICENSE` for the full text.
