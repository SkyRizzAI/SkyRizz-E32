# SkyRizz E32 ESP32-S3 module pin capabilities

- Module symbol / part: `ESP32-S3-WROOM-1-N16R8`
- Generated to help build a visual pinout diagram in the style of common ESP32 reference images.
- Complementary board wiring reference: `skyrizz_e32_esp32s3_pin_map.md`

This file answers two different questions for every module pin:

1. What the ESP32-S3 silicon can do on that pin (`RTC`, `ADC`, `TOUCH`, `USB`, `UART`, `JTAG`, strapping).
2. What the SkyRizz E32 PCB already uses that pin for.

## Quick takeaways

- Public module pins in the EasyEDA symbol: `41` total, including power and ground pins.
- The board-silk external headers are **`IO 1` = `C_P0`**, **`I2C` = `C_I2C`**, **`IO 2` = `C_P1-3`**, and **`IO 3` = `C_P4-7`**.
- The only intentionally direct external native ESP32 GPIO on this PCB is **`IO 1` / module pin 39 / GPIO1 / net `P0`**.
- The main shared buses are **module pins 24-25 (`GPIO47/48`)** for I2C and **module pins 33-36 (`GPIO40/41/42/44`)** for SPI.
- Native USB is hard-wired on **module pins 13-14 (`GPIO19/20`)** to the USB-C connector.
- Module pins **28-30 (`GPIO35/36/37`)** have no board trace in this PCB netlist, so they are not broken out to any connector or onboard device.
- `GPIO0`, `GPIO3`, `GPIO45`, and `GPIO46` are strapping pins and are also used by onboard peripherals here, so treat them carefully at boot.
- `GPIO39-42` can be external JTAG pins if you switch JTAG away from USB Serial/JTAG, but this board already uses them for audio / SPI.

## Source notes

- ESP32-S3 GPIO / RTC / ADC / USB-JTAG summary: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/api-reference/peripherals/gpio.html>
- ESP32-S3 capacitive touch channel mapping: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/api-reference/peripherals/cap_touch_sens.html>
- Default UART IOMUX pins: `components/soc/esp32s3/include/soc/uart_pins.h` in `espressif/esp-idf`
- USB PHY pins: `components/soc/esp32s3/include/soc/usb_pins.h` in `espressif/esp-idf`
- External JTAG pin mapping for ESP32-S3: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/api-guides/jtag-debugging/configure-other-jtag.html>

## External breakout headers on the PCB

These are the headers you are likely to care about in firmware when connecting external hardware:

| PCB silk | EasyEDA designator | Signals exposed | Firmware path |
| --- | --- | --- | --- |
| `IO 1` | `C_P0` | `P0`, `3V3`, `GND` | `P0` is direct `GPIO1`. |
| `I2C` | `C_I2C` | `SCL`, `SDA`, `3V3`, `GND` | Direct breakout of `GPIO48` / `GPIO47`. |
| `IO 2` | `C_P1-3` | `P1`, `P2`, `P3`, `3V3`, `GND` | XL9535-backed GPIOs over I2C on `GPIO47/48`, with interrupt on `GPIO43`. |
| `IO 3` | `C_P4-7` | `P4`, `P5`, `P6`, `P7`, `3V3`, `GND` | XL9535-backed GPIOs over I2C on `GPIO47/48`, with interrupt on `GPIO43`. |

### `IO 1` / `C_P0`

| Header pad | Signal | Firmware path |
| --- | --- | --- |
| `1` | `3V3` | Power only |
| `2` | `GND` | Ground |
| `3` | `P0` | Direct `GPIO1` |
| `4` | `GND` | Ground |
| `5` | `GND` | Ground |

### `I2C` / `C_I2C`

| Header pad | Signal | Firmware path |
| --- | --- | --- |
| `1` | `3V3` | Power only |
| `2` | `GND` | Ground |
| `3` | `SCL` | Direct `GPIO48` |
| `4` | `SDA` | Direct `GPIO47` |
| `5` | `GND` | Ground |
| `6` | `GND` | Ground |

### `IO 2` / `C_P1-3`

| Header pad | Signal | Firmware path |
| --- | --- | --- |
| `1` | `3V3` | Power only |
| `2` | `GND` | Ground |
| `3` | `P1` | XL9535 `P07` via `GPIO47/48`, interrupt on `GPIO43` |
| `4` | `P2` | XL9535 `P16` via `GPIO47/48`, interrupt on `GPIO43` |
| `5` | `P3` | XL9535 `P15` via `GPIO47/48`, interrupt on `GPIO43` |
| `6` | `GND` | Ground |
| `7` | `GND` | Ground |

### `IO 3` / `C_P4-7`

| Header pad | Signal | Firmware path |
| --- | --- | --- |
| `1` | `3V3` | Power only |
| `2` | `GND` | Ground |
| `3` | `P4` | XL9535 `P14` via `GPIO47/48`, interrupt on `GPIO43` |
| `4` | `P5` | XL9535 `P13` via `GPIO47/48`, interrupt on `GPIO43` |
| `5` | `P6` | XL9535 `P10` via `GPIO47/48`, interrupt on `GPIO43` |
| `6` | `P7` | XL9535 `P11` via `GPIO47/48`, interrupt on `GPIO43` |
| `7` | `GND` | Ground |
| `8` | `GND` | Ground |

## Diagram edge order

These tables are arranged for drawing a module pinout. Left and right edges are shown top-to-bottom. Bottom edge is shown left-to-right.

### Left edge

| Slot | Module pin | Signal | GPIO | Board use |
| --- | --- | --- | --- | --- |
| `left-01` | `1` | `GND` | `—` | Ground |
| `left-02` | `2` | `3V3` | `—` | 3.3 V supply |
| `left-03` | `3` | `EN` | `—` | Chip enable / reset |
| `left-04` | `4` | `IO4` | `GPIO4` | Camera VSYNC |
| `left-05` | `5` | `IO5` | `GPIO5` | Camera HREF |
| `left-06` | `6` | `IO6` | `GPIO6` | Camera data D9 |
| `left-07` | `7` | `IO7` | `GPIO7` | Camera XCLK |
| `left-08` | `8` | `IO15` | `GPIO15` | Camera data D8 |
| `left-09` | `9` | `IO16` | `GPIO16` | Camera data D7 |
| `left-10` | `10` | `IO17` | `GPIO17` | Camera PCLK |
| `left-11` | `11` | `IO18` | `GPIO18` | Camera data D6 |
| `left-12` | `12` | `IO8` | `GPIO8` | Camera data D2 |
| `left-13` | `13` | `IO19` | `GPIO19` | Native USB D- |
| `left-14` | `14` | `IO20` | `GPIO20` | Native USB D+ |

### Bottom edge

| Slot | Module pin | Signal | GPIO | Board use |
| --- | --- | --- | --- | --- |
| `bottom-01` | `15` | `IO3` | `GPIO3` | Audio MCLK output |
| `bottom-02` | `16` | `IO46` | `GPIO46` | WS2812 data output |
| `bottom-03` | `17` | `IO9` | `GPIO9` | Camera data D5 |
| `bottom-04` | `18` | `IO10` | `GPIO10` | Camera data D3 |
| `bottom-05` | `19` | `IO11` | `GPIO11` | Camera data D4 |
| `bottom-06` | `20` | `IO12` | `GPIO12` | LCD SPI clock |
| `bottom-07` | `21` | `IO13` | `GPIO13` | LCD D/C |
| `bottom-08` | `22` | `IO14` | `GPIO14` | LCD chip select |
| `bottom-09` | `23` | `IO21` | `GPIO21` | LCD SPI MOSI |
| `bottom-10` | `24` | `IO47` | `GPIO47` | Shared I2C SDA |
| `bottom-11` | `25` | `IO48` | `GPIO48` | Shared I2C SCL |
| `bottom-12` | `26` | `IO45` | `GPIO45` | Audio serial data to speaker amp |

### Right edge

| Slot | Module pin | Signal | GPIO | Board use |
| --- | --- | --- | --- | --- |
| `right-01` | `41` | `GND` | `—` | Ground |
| `right-02` | `40` | `GND` | `—` | Ground |
| `right-03` | `39` | `IO1` | `GPIO1` | Direct external P0 GPIO |
| `right-04` | `38` | `IO2` | `GPIO2` | Touch interrupt input |
| `right-05` | `37` | `TXD0` | `GPIO43` | XL9535 interrupt input |
| `right-06` | `36` | `RXD0` | `GPIO44` | Shared SPI clock |
| `right-07` | `35` | `IO42` | `GPIO42` | Shared SPI MOSI |
| `right-08` | `34` | `IO41` | `GPIO41` | Shared SPI MISO |
| `right-09` | `33` | `IO40` | `GPIO40` | Shared SPI chip-select |
| `right-10` | `32` | `IO39` | `GPIO39` | Audio serial data from mic ADC |
| `right-11` | `31` | `IO38` | `GPIO38` | Shared audio LRCK / WS |
| `right-12` | `30` | `IO37` | `GPIO37` | No PCB net beyond module pad |
| `right-13` | `29` | `IO36` | `GPIO36` | No PCB net beyond module pad |
| `right-14` | `28` | `IO35` | `GPIO35` | No PCB net beyond module pad |
| `right-15` | `27` | `IO0` | `GPIO0` | BOOT strap and shared audio BCLK |

## Full per-pin reference

Touch naming uses the current ESP-IDF capacitive-touch channel naming from the stable docs (`CH1`..`CH14`). Older APIs may refer to similar channels as `TOUCH_PAD_NUM1`..`TOUCH_PAD_NUM14`.

| Module pin | Signal | GPIO | Capabilities | Board net | Board use | Reuse guidance | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `1` | `GND` | `—` | Power | `GND` | Ground | N/A - power or ground pin | Reference ground for the module. |
| `2` | `3V3` | `—` | Power | `3V3` | 3.3 V supply | N/A - power or ground pin | Module supply rail. |
| `3` | `EN` | `—` | Enable / reset control | `RST` | Chip enable / reset | N/A - module reset / enable pin | Active-high EN pin on the module. Net is named RST in the PCB. |
| `4` | `IO4` | `GPIO4` | GPIO4, RTC_GPIO4, ADC1_CH3, TOUCH_CH4 | `C_VSYC` | Camera VSYNC | No - already tied to onboard hardware on this PCB | Used by the camera interface. Connected modules: FPC3 camera connector. |
| `5` | `IO5` | `GPIO5` | GPIO5, RTC_GPIO5, ADC1_CH4, TOUCH_CH5 | `C_HREF` | Camera HREF | No - already tied to onboard hardware on this PCB | Used by the camera interface. Connected modules: FPC3 camera connector. |
| `6` | `IO6` | `GPIO6` | GPIO6, RTC_GPIO6, ADC1_CH5, TOUCH_CH6 | `C_D9` | Camera data D9 | No - already tied to onboard hardware on this PCB | Used by the camera interface. Connected modules: FPC3 camera connector. |
| `7` | `IO7` | `GPIO7` | GPIO7, RTC_GPIO7, ADC1_CH6, TOUCH_CH7 | `C_XCLK` | Camera XCLK | No - already tied to onboard hardware on this PCB | Used by the camera interface. Connected modules: FPC3 camera connector. |
| `8` | `IO15` | `GPIO15` | GPIO15, RTC_GPIO15, ADC2_CH4, U0RTS | `C_D8` | Camera data D8 | No - already tied to onboard hardware on this PCB | Also has default U0RTS IOMUX role. Connected modules: FPC3 camera connector. |
| `9` | `IO16` | `GPIO16` | GPIO16, RTC_GPIO16, ADC2_CH5, U0CTS | `C_D7` | Camera data D7 | No - already tied to onboard hardware on this PCB | Also has default U0CTS IOMUX role. Connected modules: FPC3 camera connector. |
| `10` | `IO17` | `GPIO17` | GPIO17, RTC_GPIO17, ADC2_CH6, U1TXD | `C_PCLK` | Camera PCLK | No - already tied to onboard hardware on this PCB | Also has default U1TXD IOMUX role. Connected modules: FPC3 camera connector. |
| `11` | `IO18` | `GPIO18` | GPIO18, RTC_GPIO18, ADC2_CH7, U1RXD | `C_D6` | Camera data D6 | No - already tied to onboard hardware on this PCB | Also has default U1RXD IOMUX role. Connected modules: FPC3 camera connector. |
| `12` | `IO8` | `GPIO8` | GPIO8, RTC_GPIO8, ADC1_CH7, TOUCH_CH8 | `C_D2` | Camera data D2 | No - already tied to onboard hardware on this PCB | Used by the camera interface. Connected modules: FPC3 camera connector. |
| `13` | `IO19` | `GPIO19` | GPIO19, RTC_GPIO19, ADC2_CH8, U1RTS, USB_DM | `DN` | Native USB D- | No - already tied to onboard hardware on this PCB | Repurposing this pin breaks the board USB data path and disables USB Serial/JTAG usage. Connected modules: USB1 USB-C connector. |
| `14` | `IO20` | `GPIO20` | GPIO20, RTC_GPIO20, ADC2_CH9, U1CTS, USB_DP | `DP` | Native USB D+ | No - already tied to onboard hardware on this PCB | Repurposing this pin breaks the board USB data path and disables USB Serial/JTAG usage. Connected modules: USB1 USB-C connector. |
| `15` | `IO3` | `GPIO3` | GPIO3, RTC_GPIO3, ADC1_CH2, TOUCH_CH3, STRAP | `I2S_MCLK` | Audio MCLK output | No - already tied to onboard hardware on this PCB | GPIO3 is a strapping pin; this board uses it as audio master clock. Connected modules: U14 ES7243E ADC. Strapping detail: JTAG source select when STRAP_JTAG_SEL is used. |
| `16` | `IO46` | `GPIO46` | GPIO46, STRAP | `RGB` | WS2812 data output | No - already tied to onboard hardware on this PCB | GPIO46 is a strapping pin; it drives the RGB LED chain. Connected modules: RGB1, RGB2. Strapping detail: ROM boot log / strap behavior. |
| `17` | `IO9` | `GPIO9` | GPIO9, RTC_GPIO9, ADC1_CH8, TOUCH_CH9 | `C_D5` | Camera data D5 | No - already tied to onboard hardware on this PCB | Used by the camera interface. Connected modules: FPC3 camera connector. |
| `18` | `IO10` | `GPIO10` | GPIO10, RTC_GPIO10, ADC1_CH9, TOUCH_CH10 | `C_D3` | Camera data D3 | No - already tied to onboard hardware on this PCB | Used by the camera interface. Connected modules: FPC3 camera connector. |
| `19` | `IO11` | `GPIO11` | GPIO11, RTC_GPIO11, ADC2_CH0, TOUCH_CH11 | `C_D4` | Camera data D4 | No - already tied to onboard hardware on this PCB | Used by the camera interface. Connected modules: FPC3 camera connector. |
| `20` | `IO12` | `GPIO12` | GPIO12, RTC_GPIO12, ADC2_CH1, TOUCH_CH12 | `LCD_SCLK` | LCD SPI clock | No - already tied to onboard hardware on this PCB | Dedicated LCD clock on this PCB. Connected modules: FPC1 LCD flex. |
| `21` | `IO13` | `GPIO13` | GPIO13, RTC_GPIO13, ADC2_CH2, TOUCH_CH13 | `LCD_DC` | LCD D/C | No - already tied to onboard hardware on this PCB | Dedicated LCD data/command pin on this PCB. Connected modules: FPC1 LCD flex. |
| `22` | `IO14` | `GPIO14` | GPIO14, RTC_GPIO14, ADC2_CH3, TOUCH_CH14 | `LCD_CS` | LCD chip select | No - already tied to onboard hardware on this PCB | Dedicated LCD chip-select on this PCB. Connected modules: FPC1 LCD flex. |
| `23` | `IO21` | `GPIO21` | GPIO21, RTC_GPIO21 | `LCD_MOSI` | LCD SPI MOSI | No - already tied to onboard hardware on this PCB | Dedicated LCD MOSI on this PCB. Connected modules: FPC1 LCD flex. |
| `24` | `IO47` | `GPIO47` | GPIO47 | `SDA` | Shared I2C SDA | Shared only - reuse by sharing the existing bus, not by taking the pin over exclusively | Main board control/data SDA line. Add devices only if bus address/loading still works. Connected modules: HUM AHT20; LS LTR-303ALS; U5 SC7A20; U10 TSC2007; U14 ES7243E control; U18 SE050; U9 XL9535; FPC2; FPC3; C_I2C. |
| `25` | `IO48` | `GPIO48` | GPIO48 | `SCL` | Shared I2C SCL | Shared only - reuse by sharing the existing bus, not by taking the pin over exclusively | Main board control/data SCL line. Add devices only if bus timing/loading still works. Connected modules: HUM AHT20; LS LTR-303ALS; U5 SC7A20; U10 TSC2007; U14 ES7243E control; U18 SE050; U9 XL9535; FPC2; FPC3; C_I2C. |
| `26` | `IO45` | `GPIO45` | GPIO45, STRAP | `I2S_SDI` | Audio serial data to speaker amp | No - already tied to onboard hardware on this PCB | GPIO45 is a strapping pin; board net name is I2S_SDI. Connected modules: U13 NS4168. Strapping detail: VDD_SPI voltage select. |
| `27` | `IO0` | `GPIO0` | GPIO0, RTC_GPIO0, STRAP | `BOOT` | BOOT strap and shared audio BCLK | No - already tied to onboard hardware on this PCB | GPIO0 is a boot strapping pin and must not be forced low at reset. Connected modules: U13 NS4168; U14 ES7243E. Strapping detail: Boot mode select. |
| `28` | `IO35` | `GPIO35` | GPIO35 | — | No PCB net beyond module pad | No - not routed anywhere else on this PCB | No board trace found in the EasyEDA PCB netlist. On R8-class ESP32-S3 modules, GPIO33-GPIO37 may also be constrained by octal flash/PSRAM usage, so treat these especially cautiously. |
| `29` | `IO36` | `GPIO36` | GPIO36 | — | No PCB net beyond module pad | No - not routed anywhere else on this PCB | No board trace found in the EasyEDA PCB netlist. On R8-class ESP32-S3 modules, GPIO33-GPIO37 may also be constrained by octal flash/PSRAM usage, so treat these especially cautiously. |
| `30` | `IO37` | `GPIO37` | GPIO37 | — | No PCB net beyond module pad | No - not routed anywhere else on this PCB | No board trace found in the EasyEDA PCB netlist. On R8-class ESP32-S3 modules, GPIO33-GPIO37 may also be constrained by octal flash/PSRAM usage, so treat these especially cautiously. |
| `31` | `IO38` | `GPIO38` | GPIO38 | `I2S_LRCK` | Shared audio LRCK / WS | No - already tied to onboard hardware on this PCB | Audio word-select / LRCK line. Connected modules: U13 NS4168; U14 ES7243E. |
| `32` | `IO39` | `GPIO39` | GPIO39, MTCK | `I2S_SDO` | Audio serial data from mic ADC | No - already tied to onboard hardware on this PCB | Board net name is I2S_SDO. Connected modules: U14 ES7243E. External JTAG role is only relevant if you move JTAG away from the built-in USB Serial/JTAG path. |
| `33` | `IO40` | `GPIO40` | GPIO40, MTDO | `CS3` | Shared SPI chip-select | Shared only - reuse by sharing the existing bus, not by taking the pin over exclusively | GT30 chip-select is inverted from CS3 through Q5; TF uses CS3 directly. Connected modules: TF1 microSD socket; U2 GT30L24A3W. External JTAG role is only relevant if you move JTAG away from the built-in USB Serial/JTAG path. |
| `34` | `IO41` | `GPIO41` | GPIO41, MTDI | `MISO3` | Shared SPI MISO | Shared only - reuse by sharing the existing bus, not by taking the pin over exclusively | Shared SPI MISO data path. Connected modules: TF1 microSD socket; U2 GT30L24A3W. External JTAG role is only relevant if you move JTAG away from the built-in USB Serial/JTAG path. |
| `35` | `IO42` | `GPIO42` | GPIO42, MTMS | `MOSI3` | Shared SPI MOSI | Shared only - reuse by sharing the existing bus, not by taking the pin over exclusively | Shared SPI MOSI data path. Connected modules: TF1 microSD socket; U2 GT30L24A3W. External JTAG role is only relevant if you move JTAG away from the built-in USB Serial/JTAG path. |
| `36` | `RXD0` | `GPIO44` | GPIO44, U0RXD | `SCLK3` | Shared SPI clock | Shared only - reuse by sharing the existing bus, not by taking the pin over exclusively | This is default UART0 RXD (GPIO44), but the board uses it as SPI clock. Connected modules: TF1 microSD socket; U2 GT30L24A3W. |
| `37` | `TXD0` | `GPIO43` | GPIO43, U0TXD | `BUS_INT` | XL9535 interrupt input | No - already tied to onboard hardware on this PCB | This is default UART0 TXD (GPIO43), but the board uses it as BUS_INT. Connected modules: U9 XL9535. |
| `38` | `IO2` | `GPIO2` | GPIO2, RTC_GPIO2, ADC1_CH1, TOUCH_CH2 | `TS_INT` | Touch interrupt input | No - already tied to onboard hardware on this PCB | Touch controller PENIRQ line. Connected modules: U10 TSC2007; FPC2 touch/control flex. |
| `39` | `IO1` | `GPIO1` | GPIO1, RTC_GPIO1, ADC1_CH0, TOUCH_CH1 | `P0` | Direct external P0 GPIO | Yes - intended direct external native GPIO | Only direct native ESP32 GPIO intentionally broken out for external use on this PCB. Connected modules: C_P0 external header. |
| `40` | `GND` | `—` | Power | `GND` | Ground | N/A - power or ground pin | Reference ground for the module. |
| `41` | `GND` | `—` | Power | `GND` | Ground | N/A - power or ground pin | Reference ground for the module. |

## Best pins to think about first in firmware

- **Direct external native GPIO:** module pin `39` / `GPIO1` / net `P0`.
- **Shared I2C bus:** module pins `24-25` / `GPIO47-48`.
- **Board-silk external headers:** `IO 1`, `I2C`, `IO 2`, `IO 3`.
- **Shared SPI bus:** module pins `33-36` / `GPIO40/41/42/44`.
- **Touch interrupt:** module pin `38` / `GPIO2`.
- **USB:** module pins `13-14` / `GPIO19-20`.
- **Unrouted in this PCB:** module pins `28-30` / `GPIO35-37`.
