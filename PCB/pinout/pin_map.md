# SkyRizz E32 module-by-module firmware map

This version is module-centric instead of ESP32-pin-centric. Each onboard IC, flex connector, LED chain, and external connector is listed pin-by-pin so you can see exactly how firmware reaches it: directly from an ESP32-S3-WROOM-1 GPIO, through the shared buses, or through the XL9535 I/O expander.

## Quick board-level firmware view

- Only `C_P0` is a direct external ESP32 GPIO (`GPIO1`). External `P1`-`P7` come from the XL9535, so they are slower I2C-backed GPIOs rather than native ESP32 pins.
- `GPIO47` / `GPIO48` are the main shared I2C bus for nearly every sensor / control IC on the board.
- `GPIO43 / U0TXD` is consumed by the XL9535 interrupt line `BUS_INT`.
- `GPIO44 / U0RXD` is reused as SPI clock `SCLK3` for both the TF socket and `U2`.
- `GPIO0` is both the ESP32 boot-strap pin and the shared audio `BCLK` line. Treat it carefully at boot.
- `GPIO35`, `GPIO36`, and `GPIO37` are unused in this PCB netlist.

### ESP32 resource summary

| ESP32 resource | Used by |
| --- | --- |
| `GPIO47 / GPIO48` | Shared I2C bus for `HUM`, `LS`, `U5`, `U10`, `U14` control, `U18`, `U9`, `FPC2`, `FPC3`, and `C_I2C`. |
| `GPIO43 / U0TXD` | XL9535 `INT#` / `BUS_INT`. |
| `GPIO44 / U0RXD` | Shared SPI clock `SCLK3` for `TF1` and `U2`. |
| `GPIO40 / GPIO41 / GPIO42` | Shared SPI `CS/MISO/MOSI` for `TF1` and `U2`. |
| `GPIO0 / GPIO3 / GPIO38 / GPIO39 / GPIO45` | Audio interface for `U13` and `U14`. |
| `GPIO4-18` | Camera connector `FPC3` data and sync lines. |
| `GPIO12 / GPIO13 / GPIO14 / GPIO21` | LCD SPI lines on `FPC1`. |
| `GPIO19 / GPIO20` | Native USB FS `D-` / `D+`. |
| `GPIO46` | WS2812 LED chain (`RGB1` -> `RGB2`). |
| `GPIO1` | Direct external pin `P0` on `C_P0`. |
| `GPIO35 / GPIO36 / GPIO37` | No onboard connection found in this PCB netlist. |

### Module access index

| Module | Part / block | Firmware access |
| --- | --- | --- |
| `HUM` | AHT20 | I2C on `GPIO47`/`GPIO48` |
| `LS` | LTR-303ALS | I2C on `GPIO47`/`GPIO48` |
| `U5` | SC7A20 | I2C on `GPIO47`/`GPIO48` |
| `U10` | TSC2007 | I2C + `GPIO2` IRQ + XL9535 reset |
| `FPC1` | LCD flex | LCD SPI direct; backlight via XL9535 |
| `FPC2` | Touch/control flex | Shared I2C + touch IRQ/reset |
| `FPC3` | Camera flex | Direct camera GPIOs + I2C + XL9535 reset |
| `U14` | ES7243E | I2S + shared I2C control |
| `MIC1/MIC2` | Analog microphones | Via ES7243E analog front-end |
| `U13` | NS4168 | I2S-driven speaker amplifier |
| `RGB1/RGB2` | WS2812 LEDs | `GPIO46` serial LED chain |
| `TF1` | microSD socket | Shared SPI3 |
| `U2` | GT30L24A3W | Shared SPI3 with inverted `CS#` |
| `U18` | SE050 | I2C + XL9535 reset |
| `U9` | XL9535 | I2C expander at `0x20`, IRQ on `GPIO43` |
| `C_I2C` | External I2C header | Shared I2C breakout |
| `C_P0` | External P0 | Direct `GPIO1` breakout |
| `C_P1-3` | External P1-P3 | XL9535-backed external GPIOs |
| `C_P4-7` | External P4-P7 | XL9535-backed external GPIOs |
| `USB1` | USB Type-C | USB FS `GPIO19`/`GPIO20` + passive CC/VBUS |

## Module-by-module map

## Onboard sensors

### `HUM` — AHT20 humidity / temperature sensor

Firmware access: Shared I2C on `GPIO47`/`GPIO48`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `NC` | `` | Not connected | Not connected. |
| `2` | `VDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `3` | `SCL` | `SCL` | GPIO48 | Shared I2C SCL on `GPIO48`. |
| `4` | `SDA` | `SDA` | GPIO47 | Shared I2C SDA on `GPIO47`. |
| `5` | `GND` | `GND` | Ground | Ground. |
| `6` | `NC` | `` | Not connected | Not connected. |

### `LS` — LTR-303ALS ambient light sensor

Firmware access: Shared I2C on `GPIO47`/`GPIO48`; interrupt pin is not routed.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `VDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `NC` | `` | Not connected | Not connected. |
| `3` | `GND` | `GND` | Ground | Ground. |
| `4` | `SCL` | `SCL` | GPIO48 | Shared I2C SCL on `GPIO48`. |
| `5` | `INT` | `` | Not connected | Interrupt output is not routed anywhere. |
| `6` | `SDA` | `SDA` | GPIO47 | Shared I2C SDA on `GPIO47`. |

### `U5` — SC7A20 accelerometer

Firmware access: Shared I2C on `GPIO47`/`GPIO48`; `INT1`/`INT2` are not routed.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `SDO` | `` | Not connected | `SDO` is not connected. |
| `2` | `SDx` | `SDA` | GPIO47 | Shared I2C SDA on `GPIO47`. |
| `3` | `VDDIO` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `4` | `NC` | `GND` | Ground | Package `NC` pin is tied to ground on this PCB. |
| `5` | `INT1` | `` | Not connected | Accelerometer interrupt output `INT1` is not routed. |
| `6` | `INT2` | `` | Not connected | Accelerometer interrupt output `INT2` is not routed. |
| `7` | `VDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `8` | `GNDIO` | `GND` | Ground | Ground. |
| `9` | `GND` | `GND` | Ground | Ground. |
| `10` | `CS` | `` | Not connected | Not connected. |
| `11` | `NC` | `` | Not connected | Not connected. |
| `12` | `SCx` | `SCL` | GPIO48 | Shared I2C SCL on `GPIO48`. |

## Touch and display

### `U10` — TSC2007 touch controller

Firmware access: Shared I2C on `GPIO47`/`GPIO48`; `PENIRQ` on `GPIO2`; reset via XL9535 `P01`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `VDDREF` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `X+` | `X+` | Only between U10 and the FPC1 resistive touch panel | Board-specific connection; see linked net and connected parts. |
| `3` | `Y+` | `Y+` | Only between U10 and the FPC1 resistive touch panel | Board-specific connection; see linked net and connected parts. |
| `4` | `X-` | `X-` | Only between U10 and the FPC1 resistive touch panel | Board-specific connection; see linked net and connected parts. |
| `5` | `Y-` | `Y-` | Only between U10 and the FPC1 resistive touch panel | Board-specific connection; see linked net and connected parts. |
| `6` | `GND` | `GND` | Ground | Ground. |
| `7` | `NC/2` | `` | Not connected | Not connected. |
| `8` | `NC/4` | `` | Not connected | Not connected. |
| `9` | `NC` | `` | Not connected | Not connected. |
| `10` | `PENIRQ` | `TS_INT` | GPIO2 | Touch interrupt / `PENIRQ` to `GPIO2`. |
| `11` | `SDA` | `SDA` | GPIO47 | Shared I2C SDA on `GPIO47`. |
| `12` | `SCL` | `SCL` | GPIO48 | Shared I2C SCL on `GPIO48`. |
| `13` | `A1` | `GND` | Ground | Address/config strap `A1` is tied low. |
| `14` | `A0` | `GND` | Ground | Address/config strap `A0` is tied low. |
| `15` | `NC/3` | `` | Not connected | Not connected. |
| `16` | `AUX` | `` | Not connected | Not connected. |

### `FPC1` — LCD / touch main flex connector

Firmware access: LCD SPI is direct from the ESP32; raw resistive touch lines go to `U10`; backlight is switched through XL9535 `P00` and transistor `Q4`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `1` | `Y-` | Only between U10 and the FPC1 resistive touch panel | Resistive touch `Y-` line to `U10`. |
| `2` | `2` | `X-` | Only between U10 and the FPC1 resistive touch panel | Resistive touch `X-` line to `U10`. |
| `3` | `3` | `Y+` | Only between U10 and the FPC1 resistive touch panel | Resistive touch `Y+` line to `U10`. |
| `4` | `4` | `X+` | Only between U10 and the FPC1 resistive touch panel | Resistive touch `X+` line to `U10`. |
| `5` | `5` | `$4N855` | XL9535 P00 -> R19/Q4 -> R17 -> FPC1 backlight pins | One of the LCD backlight flex pins on the shared `$4N855` net. |
| `6` | `6` | `$4N855` | XL9535 P00 -> R19/Q4 -> R17 -> FPC1 backlight pins | One of the LCD backlight flex pins on the shared `$4N855` net. |
| `7` | `7` | `$4N855` | XL9535 P00 -> R19/Q4 -> R17 -> FPC1 backlight pins | One of the LCD backlight flex pins on the shared `$4N855` net. |
| `8` | `8` | `$4N855` | XL9535 P00 -> R19/Q4 -> R17 -> FPC1 backlight pins | One of the LCD backlight flex pins on the shared `$4N855` net. |
| `9` | `9` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `10` | `10` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `11` | `11` | `GND` | Ground | Ground. |
| `12` | `12` | `GND` | Ground | Ground. |
| `13` | `13` | `LCD_MOSI` | GPIO21 | LCD `MOSI` from `GPIO21`. |
| `14` | `14` | `LCD_CS` | GPIO14 | LCD `CS` from `GPIO14`. |
| `15` | `15` | `LCD_DC` | GPIO13 | LCD `D/C` from `GPIO13`. |
| `16` | `16` | `LCD_SCLK` | GPIO12 | LCD `SCLK` from `GPIO12`. |
| `17` | `17` | `$4N864` | Passive RC to 3V3 only | Auxiliary LCD pin with only an RC pull-up (`R20/C25`). |
| `18` | `18` | `GND` | Ground | Ground. |
| `19` | `19` | `GND` | Ground | Ground. |
| `20` | `20` | `GND` | Ground | Ground. |

Display-specific note: `FPC1` pads `5-8` are the shared backlight string path. The actual switching path is `U9 P00 -> R19 -> Q4 -> R17 -> FPC1 pads 5-8` rather than a direct ESP32 GPIO.

### `FPC2` — Touch / side-control flex connector

Firmware access: Breaks out the shared I2C bus plus touch interrupt/reset.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `1` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `2` | `SDA` | GPIO47 | Shared I2C SDA for the touch-side flex. |
| `3` | `3` | `SCL` | GPIO48 | Shared I2C SCL for the touch-side flex. |
| `4` | `4` | `TS_INT` | GPIO2 | Touch interrupt back to the ESP32. |
| `5` | `5` | `TS_RST` | XL9535 P01 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Touch reset via XL9535 `P01`. |
| `6` | `6` | `GND` | Ground | Ground. |
| `7` | `7` | `GND` | Ground | Ground. |
| `8` | `8` | `GND` | Ground | Ground. |

## Camera

### `FPC3` — Camera flex connector

Firmware access: Camera data/clock pins are mostly direct ESP32 GPIOs; reset is on XL9535 `P02`; SCCB uses the shared I2C bus.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `1` | `` | Not connected | No connection on the PCB. |
| `2` | `2` | `` | Not connected | No connection on the PCB. |
| `3` | `3` | `C_D4` | GPIO11 | Camera data bit `D4` to `GPIO11`. |
| `4` | `4` | `C_D3` | GPIO10 | Camera data bit `D3` to `GPIO10`. |
| `5` | `5` | `C_D5` | GPIO9 | Camera data bit `D5` to `GPIO9`. |
| `6` | `6` | `C_D2` | GPIO8 | Camera data bit `D2` to `GPIO8`. |
| `7` | `7` | `C_D6` | GPIO18 | Camera data bit `D6` to `GPIO18`. |
| `8` | `8` | `C_PCLK` | GPIO17 | Camera `PCLK` to `GPIO17`. |
| `9` | `9` | `C_D7` | GPIO16 | Camera data bit `D7` to `GPIO16`. |
| `10` | `10` | `GND` | Ground | Ground. |
| `11` | `11` | `C_D8` | GPIO15 | Camera data bit `D8` to `GPIO15`. |
| `12` | `12` | `C_XCLK` | GPIO7 | Camera `XCLK` to `GPIO7`. |
| `13` | `13` | `C_D9` | GPIO6 | Camera data bit `D9` to `GPIO6`. |
| `14` | `14` | `2V8` | 2V8 rail from U11 | Camera 2.8 V rail. |
| `15` | `15` | `1V8` | 1V8 rail from U12 | Camera 1.8 V rail. |
| `16` | `16` | `C_HREF` | GPIO5 | Camera `HREF` to `GPIO5`. |
| `17` | `17` | `$4N2005` | 0-ohm strap to GND only | Hard-strapped to ground through `R32`. |
| `18` | `18` | `C_VSYC` | GPIO4 | Camera `VSYNC` to `GPIO4`. |
| `19` | `19` | `C_RST` | XL9535 P02 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Camera reset via XL9535 `P02`. |
| `20` | `20` | `SCL` | GPIO48 | Camera SCCB / I2C `SCL`. |
| `21` | `21` | `2V8` | 2V8 rail from U11 | Camera 2.8 V rail. |
| `22` | `22` | `SDA` | GPIO47 | Camera SCCB / I2C `SDA`. |
| `23` | `23` | `GND` | Ground | Ground. |
| `24` | `24` | `` | Not connected | No connection on the PCB. |
| `25` | `25` | `GND` | Ground | Ground. |
| `26` | `26` | `GND` | Ground | Ground. |

Camera-specific notes: `FPC3 pad 17` is not a usable GPIO signal — it is strapped to ground through `R32`. `FPC3 pads 14/21` are `2V8`, and `pad 15` is `1V8` for the attached camera module.

## Audio

### `U14` — ES7243E audio ADC

Firmware access: I2S to the ESP32 on `GPIO0`, `GPIO3`, `GPIO38`, `GPIO39`; control pins are on the shared I2C bus.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `VDDP` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `TDMIN` | `` | Not connected | `TDMIN` is not connected. |
| `3` | `SDOUT/AD2` | `I2S_SDO` | GPIO39 | Audio data from the ES7243E back into the ESP32 on `GPIO39`. |
| `4` | `GNDD` | `GND` | Ground | Ground. |
| `5` | `VDDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `6` | `SCLK` | `BOOT` | GPIO0 | Shared audio `BCLK` on `GPIO0`; also the ESP32 boot-strap pin. |
| `7` | `LRCK` | `I2S_LRCK` | GPIO38 | Shared audio `LRCK/WS` on `GPIO38`. |
| `8` | `AD1` | `GND` | Ground | `AD1` strap tied low. |
| `9` | `AINLP` | `$6N1543` | Analog path through U14; digitized back to ESP32 on GPIO39/I2S | AC-coupled microphone net into `U14 AINLP` through `C27`. |
| `10` | `AINLN` | `$6N1544` | Analog path through U14; digitized back to ESP32 on GPIO39/I2S | Reference leg for `U14 AINLN`; AC-grounded through `C28`. |
| `11` | `REFQ` | `$6N1575` | Analog path through U14; digitized back to ESP32 on GPIO39/I2S | ES7243E `REFQ` decoupling node. |
| `12` | `VDDA` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `13` | `GNDA` | `GND` | Ground | Ground. |
| `14` | `REFP` | `$6N1576` | Analog path through U14; digitized back to ESP32 on GPIO39/I2S | ES7243E `REFP` decoupling node. |
| `15` | `AINRN` | `$6N1562` | Analog path through U14; digitized back to ESP32 on GPIO39/I2S | Reference leg for `U14 AINRN`; AC-grounded through `C39`. |
| `16` | `AINRP` | `$6N1563` | Analog path through U14; digitized back to ESP32 on GPIO39/I2S | AC-coupled microphone net into `U14 AINRP` through `C38`. |
| `17` | `AD0` | `3V3` | 3V3 rail | `AD0` strap tied high to 3.3 V. |
| `18` | `CDATA` | `SDA` | GPIO47 | Shared I2C SDA on `GPIO47`. |
| `19` | `CCLK` | `SCL` | GPIO48 | Shared I2C SCL on `GPIO48`. |
| `20` | `MCLK` | `I2S_MCLK` | GPIO3 | Audio master clock from `GPIO3`. |
| `21` | `EP` | `GND` | Ground | Ground. |

### `MIC1` — Microphone 1

Firmware access: Analog microphone feeding `U14` through the passive front-end; not connected to ESP32 GPIO directly.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `VDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `OUT` | `$6N1541` | Analog path through U14; digitized back to ESP32 on GPIO39/I2S | Microphone analog output feeding the ES7243E front-end. |
| `3` | `GND` | `AGND` | Analog ground | Analog ground for the microphone / audio ADC front-end. |
| `4` | `GND` | `AGND` | Analog ground | Analog ground for the microphone / audio ADC front-end. |

### `MIC2` — Microphone 2

Firmware access: Analog microphone feeding `U14` through the passive front-end; not connected to ESP32 GPIO directly.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `VDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `OUT` | `$6N1560` | Analog path through U14; digitized back to ESP32 on GPIO39/I2S | Microphone analog output feeding the ES7243E front-end. |
| `3` | `GND` | `AGND` | Analog ground | Analog ground for the microphone / audio ADC front-end. |
| `4` | `GND` | `AGND` | Analog ground | Analog ground for the microphone / audio ADC front-end. |

### `U13` — NS4168 speaker amplifier

Firmware access: Driven from the ESP32 I2S-style audio signals on `GPIO45`, `GPIO38`, and `GPIO0`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `CTRL` | `3V3` | 3V3 rail | `CTRL` is tied high to 3.3 V, so the amplifier is always enabled. |
| `2` | `LRCLK` | `I2S_LRCK` | GPIO38 | Shared audio `LRCK/WS` on `GPIO38`. |
| `3` | `BCLK` | `BOOT` | GPIO0 | Shared audio `BCLK` on `GPIO0`; also the ESP32 boot-strap pin. |
| `4` | `SDATA` | `I2S_SDI` | GPIO45 | Audio data from the ESP32 into the speaker amplifier on `GPIO45`. |
| `5` | `VON` | `$6N1591` | Speaker output from U13 driven by ESP32 audio signals | Differential speaker output from `U13`. |
| `6` | `VDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `7` | `GND` | `GND` | Ground | Ground. |
| `8` | `VOP` | `$6N1590` | Speaker output from U13 driven by ESP32 audio signals | Differential speaker output from `U13`. |
| `9` | `EP` | `GND` | Ground | Ground. |

## Indicators and LEDs

### `RGB1` — RGB LED 1

Firmware access: WS2812-style data chain starting at `GPIO46`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `DOUT` | `$5N3637` | LED chain only: RGB1 DOUT -> RGB2 DIN | Daisy-chain output into `RGB2 DIN`. |
| `2` | `GND` | `GND` | Ground | Ground. |
| `3` | `DIN` | `RGB` | GPIO46 | First LED data input from `GPIO46`. |
| `4` | `VDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |

### `RGB2` — RGB LED 2

Firmware access: Second WS2812-style LED in the chain after `RGB1`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `DOUT` | `` | Not connected | Chain output not used further on this PCB. |
| `2` | `GND` | `GND` | Ground | Ground. |
| `3` | `DIN` | `$5N3637` | LED chain only: RGB1 DOUT -> RGB2 DIN | Input from `RGB1 DOUT`. |
| `4` | `VDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |

## Storage and SPI devices

### `TF1` — TF / microSD socket

Firmware access: Shared SPI bus on `GPIO40`/`41`/`42`/`44`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `DAT2` | `` | Not connected | `DAT2` is not connected. |
| `2` | `CD/DAT3` | `CS3` | GPIO40 | `DAT3 / CS` to `GPIO40`. |
| `3` | `CMD` | `MOSI3` | GPIO42 | `CMD / MOSI` to `GPIO42`. |
| `4` | `VDD` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `5` | `CLK` | `SCLK3` | GPIO44 / U0RXD | `CLK` to `GPIO44 / U0RXD`. |
| `6` | `VSS` | `GND` | Ground | Ground. |
| `7` | `DAT0` | `MISO3` | GPIO41 | `DAT0 / MISO` to `GPIO41`. |
| `8` | `DAT1` | `` | Not connected | `DAT1` is not connected. |
| `9` | `CD` | `` | Not connected | Card-detect pad is not connected. |
| `10` | `GND` | `GND` | Ground | Ground. |
| `11` | `GND` | `GND` | Ground | Ground. |
| `12` | `GND` | `GND` | Ground | Ground. |
| `13` | `GND` | `GND` | Ground | Ground. |

### `U2` — GT30L24A3W SPI ROM / font chip

Firmware access: Shares SPI with the TF socket. `CS#` is inverted from `GPIO40 / CS3` through `Q5`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `CS#` | `$1N44298` | U2 CS# from GPIO40 / CS3 through R28/Q5 inverter | Chip select is inverted from `GPIO40 / CS3` by `Q5`. |
| `2` | `SO` | `MISO3` | GPIO41 | Shared SPI `MISO` on `GPIO41`. |
| `3` | `NC` | `` | Not connected | Not connected. |
| `4` | `GND` | `GND` | Ground | Ground. |
| `5` | `SI` | `MOSI3` | GPIO42 | Shared SPI `MOSI` on `GPIO42`. |
| `6` | `SCLK` | `SCLK3` | GPIO44 / U0RXD | Shared SPI clock on `GPIO44 / U0RXD`. |
| `7` | `HOLD#` | `$1N44462` | Pull-up only (R31 -> 3V3) | `HOLD#` pulled high by `R31`. |
| `8` | `VCC` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |

Important `U2` quirk: the GT30 chip is selected when `GPIO40 / CS3` is **high**, because `Q5` inverts the signal. The TF card sees the normal active-low `CS3` directly.

## Secure element

### `U18` — SE050 secure element

Firmware access: Shared I2C on `GPIO47`/`GPIO48`; reset via XL9535 `P03`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `ISO14443LB` | `GND` | Ground | Ground. |
| `2` | `n.c.` | `` | Not connected | Not connected. |
| `3` | `ISO7816IO1` | `` | Not connected | Not connected. |
| `4` | `n.c.` | `` | Not connected | Not connected. |
| `5` | `n.c.` | `` | Not connected | Not connected. |
| `6` | `n.c.` | `` | Not connected | Not connected. |
| `7` | `n.c.` | `` | Not connected | Not connected. |
| `8` | `n.c.` | `` | Not connected | Not connected. |
| `9` | `l2C_SDA` | `SDA` | GPIO47 | Shared I2C SDA on `GPIO47`. |
| `10` | `l2C_SCL` | `SCL` | GPIO48 | Shared I2C SCL on `GPIO48`. |
| `11` | `ENA` | `$3N4515` | Pull-up / enable node only (R43 -> 3V3) | SE050 enable node pulled up to 3.3 V through `R43`. |
| `12` | `VIN` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `13` | `ISO7816CLK` | `` | Not connected | Not connected. |
| `14` | `ISO7816RST_N` | `SE_RST` | XL9535 P03 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Reset input driven by XL9535 `P03`. |
| `15` | `VOUT` | `$3N4430` | Local SE050 supply node only | Local SE050 supply / decoupling node shared by `VOUT` and `VCC`. |
| `16` | `ISO7816IO2` | `` | Not connected | Not connected. |
| `17` | `ISO14443LA` | `GND` | Ground | Ground. |
| `18` | `VCC` | `$3N4430` | Local SE050 supply node only | Local SE050 supply / decoupling node shared by `VOUT` and `VCC`. |
| `19` | `VSS` | `GND` | Ground | Ground. |
| `20` | `n.c.` | `` | Not connected | Not connected. |
| `21` | `EP` | `GND` | Ground | Ground. |

## I/O expansion

### `U9` — XL9535 I/O expander

Firmware access: I2C I/O expander on the shared bus; `INT#` returns to `GPIO43 / U0TXD`; address pins are grounded so the address is `0x20`.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `P00` | `LCD_BLK` | XL9535 P00 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Expander output `P00` used for LCD backlight control. |
| `2` | `P01` | `TS_RST` | XL9535 P01 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Expander output `P01` used for touch reset. |
| `3` | `P02` | `C_RST` | XL9535 P02 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Expander output `P02` used for camera reset. |
| `4` | `P03` | `SE_RST` | XL9535 P03 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Expander output `P03` used for secure-element reset. |
| `5` | `P04` | `P9` | XL9535 P04 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Local net `P9`, also wired to `SW2`. |
| `6` | `P05` | `P10` | XL9535 P05 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Local net `P10`, also wired to `PB1` with RC parts. |
| `7` | `P06` | `P11` | XL9535 P06 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Local net `P11`, also wired to `PB2` with RC parts. |
| `8` | `P07` | `P1` | XL9535 P07 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External connector net `P1`. |
| `9` | `GND` | `GND` | Ground | Ground. |
| `10` | `P10` | `P6` | XL9535 P10 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External connector net `P6`. |
| `11` | `P11` | `P7` | XL9535 P11 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External connector net `P7`; also shared with `SW3`. |
| `12` | `P12` | `P8` | XL9535 P12 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | Local net `P8`, also wired to `SW1`. |
| `13` | `P13` | `P5` | XL9535 P13 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External connector net `P5`. |
| `14` | `P14` | `P4` | XL9535 P14 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External connector net `P4`. |
| `15` | `P15` | `P3` | XL9535 P15 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External connector net `P3`. |
| `16` | `P16` | `P2` | XL9535 P16 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External connector net `P2`. |
| `17` | `P17` | `U_LED` | XL9535 P17 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | User / indicator LED drive through `R16`. |
| `18` | `A0` | `GND` | Ground | Address strap `A0` tied low. |
| `19` | `SCL` | `SCL` | GPIO48 | Shared I2C SCL on `GPIO48`. |
| `20` | `DA` | `SDA` | GPIO47 | Shared I2C SDA on `GPIO47`. |
| `21` | `VCC` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `22` | `INT#` | `BUS_INT` | GPIO43 / U0TXD | `INT#` back to `GPIO43 / U0TXD`. |
| `23` | `A1` | `GND` | Ground | Address strap `A1` tied low. |
| `24` | `A2` | `GND` | Ground | Address strap `A2` tied low. |
| `25` | `EP` | `GND` | Ground | Exposed pad tied to ground. |

Local firmware-relevant nets hanging off the XL9535:

| XL9535 net | Physical destination | What it means in firmware |
| --- | --- | --- |
| `P8` | `SW1` + pull-up parts | Local switch net, only reachable through the expander. |
| `P9` | `SW2` + pull-up parts | Local switch net, only reachable through the expander. |
| `P10` | `PB1` + RC parts | Local button net, only reachable through the expander. |
| `P11` | `PB2` + RC parts | Local button net, only reachable through the expander. |
| `P7` | `SW3` and external connector `C_P4-7 pad 6` | Shared local/external expander GPIO. |
| `U_LED` | LED drive through `R16` | Expander-controlled user / indicator LED. |

## External connectors

PCB silkscreen labels for the external headers are:

- `IO 1` = `C_P0`
- `I2C` = `C_I2C`
- `IO 2` = `C_P1-3`
- `IO 3` = `C_P4-7`

### `C_I2C` — External I2C connector (`I2C` on PCB silk)

Firmware access: Direct breakout of 3V3, GND, and the shared `GPIO47`/`GPIO48` I2C bus.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `1` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `2` | `GND` | Ground | Ground. |
| `3` | `3` | `SCL` | GPIO48 | External I2C `SCL`. |
| `4` | `4` | `SDA` | GPIO47 | External I2C `SDA`. |
| `5` | `5` | `GND` | Ground | Ground. |
| `6` | `6` | `GND` | Ground | Ground. |

### `C_P0` — External P0 connector (`IO 1` on PCB silk)

Firmware access: Direct external GPIO on `GPIO1` plus 3V3/GND.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `1` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `2` | `GND` | Ground | Ground. |
| `3` | `3` | `P0` | GPIO1 | Direct ESP32 `GPIO1` breakout. |
| `4` | `4` | `GND` | Ground | Ground. |
| `5` | `5` | `GND` | Ground | Ground. |

### `C_P1-3` — External P1-P3 connector (`IO 2` on PCB silk)

Firmware access: External GPIOs through the XL9535 expander (`P07`, `P16`, `P15`).

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `1` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `2` | `GND` | Ground | Ground. |
| `3` | `3` | `P1` | XL9535 P07 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External `P1` through XL9535 `P07`. |
| `4` | `4` | `P2` | XL9535 P16 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External `P2` through XL9535 `P16`. |
| `5` | `5` | `P3` | XL9535 P15 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External `P3` through XL9535 `P15`. |
| `6` | `6` | `GND` | Ground | Ground. |
| `7` | `7` | `GND` | Ground | Ground. |

### `C_P4-7` — External P4-P7 connector (`IO 3` on PCB silk)

Firmware access: External GPIOs through the XL9535 expander (`P14`, `P13`, `P10`, `P11`).

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `1` | `1` | `3V3` | 3V3 rail | 3.3 V supply rail. Not a GPIO. |
| `2` | `2` | `GND` | Ground | Ground. |
| `3` | `3` | `P4` | XL9535 P14 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External `P4` through XL9535 `P14`. |
| `4` | `4` | `P5` | XL9535 P13 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External `P5` through XL9535 `P13`. |
| `5` | `5` | `P6` | XL9535 P10 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External `P6` through XL9535 `P10`. |
| `6` | `6` | `P7` | XL9535 P11 via I2C (GPIO47/GPIO48), IRQ on GPIO43 | External `P7` through XL9535 `P11` and shared with `SW3`. |
| `7` | `7` | `GND` | Ground | Ground. |
| `8` | `8` | `GND` | Ground | Ground. |

## USB and power

### `USB1` — USB Type-C connector

Firmware access: Native USB FS D+/D- go to `GPIO20`/`GPIO19` through `U3`; CC pins are passive 5.1k pulldowns; VBUS is power only.

| Pad | Signal / pin label | Net | ESP32 / firmware path | Notes |
| --- | --- | --- | --- | --- |
| `A1` | `A1` | `GND` | Ground | Ground. |
| `A4` | `A4` | `VBUS` | USB 5 V rail | USB 5 V input rail. Not a usable ESP32 GPIO. |
| `A5` | `A5` | `CC1` | No ESP32 GPIO; passive USB-C CC termination | USB Type-C `CC1` pin. |
| `A6` | `A6` | `DP` | GPIO20 | USB Type-C `D+`. |
| `A7` | `A7` | `DN` | GPIO19 | USB Type-C `D-`. |
| `A8` | `A8` | `` | Not connected | SuperSpeed pin not used on this USB 2.0 design. |
| `A9` | `A9` | `VBUS` | USB 5 V rail | USB 5 V input rail. Not a usable ESP32 GPIO. |
| `A12` | `A12` | `GND` | Ground | Ground. |
| `B1` | `B1` | `GND` | Ground | Ground. |
| `B4` | `B4` | `VBUS` | USB 5 V rail | USB 5 V input rail. Not a usable ESP32 GPIO. |
| `B5` | `B5` | `CC2` | No ESP32 GPIO; passive USB-C CC termination | USB Type-C `CC2` pin. |
| `B6` | `B6` | `DP` | GPIO20 | USB Type-C `D+`. |
| `B7` | `B7` | `DN` | GPIO19 | USB Type-C `D-`. |
| `B8` | `B8` | `` | Not connected | SuperSpeed pin not used on this USB 2.0 design. |
| `B9` | `B9` | `VBUS` | USB 5 V rail | USB 5 V input rail. Not a usable ESP32 GPIO. |
| `B12` | `B12` | `GND` | Ground | Ground. |
| `0` | `SH0` | `GND` | Ground | Shield / shell tied to ground. |
| `1` | `SH1` | `GND` | Ground | Shield / shell tied to ground. |
| `2` | `SH2` | `GND` | Ground | Shield / shell tied to ground. |
| `3` | `SH3` | `GND` | Ground | Shield / shell tied to ground. |

USB note: only `D+` / `D-` go to the ESP32 (`GPIO20` / `GPIO19`). `CC1` and `CC2` are passive pulldowns, and `VBUS` is power-path only.

## Existing paste-ready code outputs

- `no_std_board_pins.rs` provides the same grouping for `no_std` Rust / `esp-hal` firmware.
