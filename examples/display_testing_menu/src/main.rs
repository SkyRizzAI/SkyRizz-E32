#![no_std]
#![no_main]

mod camera;

use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_hal::delay::DelayNs;
use esp_backtrace as _;
esp_bootloader_esp_idf::esp_app_desc!();
use esp_hal::{
    delay::Delay,
    dma_buffers,
    dma::DmaRxBuf,
    gpio::{Level, Output, OutputConfig},
    i2c::master::{BusTimeout, Config as I2cConfig, I2c, SoftwareTimeout},
    i2s::master::{
        Channels as I2sChannels, Config as I2sConfig, DataFormat as I2sDataFormat, I2s, I2sRx,
    },
    lcd_cam::{
        cam::{Camera, Config as CamConfig, EofMode, VhdeMode, VsyncFilterThreshold},
        LcdCam,
    },
    spi::{
        Mode,
        master::{Config as SpiConfig, Spi},
    },
    time::{Duration, Rate},
    Blocking,
};
use esp_println::println;

// XL9535 I/O expander on the shared I2C bus
const XL_ADDR: u8 = 0x20;
const XL_INPUT_0: u8 = 0x00;
const XL_INPUT_1: u8 = 0x01;
const XL_OUTPUT_0: u8 = 0x02;
const XL_OUTPUT_1: u8 = 0x03;
const XL_CONFIG_0: u8 = 0x06;
const XL_CONFIG_1: u8 = 0x07;

// XL9535 port-0 bit map (board-specific)
const BL_BIT: u8 = 1 << 0;     // P00 = LCD backlight enable
const FT_RST_BIT: u8 = 1 << 1; // P01 = FT6336U capacitive touch reset (active low)
const C_RST_BIT: u8 = 1 << 2;  // P02 = Camera reset (active low)
const SE_RST_BIT: u8 = 1 << 3; // P03 = SE050 reset (active low)
// XL9535 port-1 bit map
const IND_LED_BIT: u8 = 1 << 7; // P17 = onboard indicator LED

const LCD_W: u16 = 240;
const LCD_H: u16 = 320;

// ===========================================================================
// ILI9341 driver (SPI, write-only)
// ===========================================================================
struct Lcd<'a> {
    spi: Spi<'a, Blocking>,
    dc: Output<'a>,
    cs: Output<'a>,
    delay: Delay,
}

impl<'a> Lcd<'a> {
    fn new(spi: Spi<'a, Blocking>, dc: Output<'a>, cs: Output<'a>) -> Self {
        Self { spi, dc, cs, delay: Delay::new() }
    }

    fn cmd(&mut self, c: u8) {
        self.dc.set_low();
        self.cs.set_low();
        let _ = self.spi.write(&[c]);
        self.cs.set_high();
    }

    fn data(&mut self, d: &[u8]) {
        self.dc.set_high();
        self.cs.set_low();
        let _ = self.spi.write(d);
        self.cs.set_high();
    }

    fn cmd_data(&mut self, c: u8, d: &[u8]) {
        self.cmd(c);
        self.data(d);
    }

    fn init(&mut self) {
        self.cs.set_high();
        self.delay.delay_ms(5);

        self.cmd(0x01); // SWRESET
        self.delay.delay_ms(150);
        self.cmd(0x11); // SLPOUT
        self.delay.delay_ms(150);

        // Power / timing (Adafruit-style ILI9341 init)
        self.cmd_data(0xCB, &[0x39, 0x2C, 0x00, 0x34, 0x02]);
        self.cmd_data(0xCF, &[0x00, 0xC1, 0x30]);
        self.cmd_data(0xE8, &[0x85, 0x00, 0x78]);
        self.cmd_data(0xEA, &[0x00, 0x00]);
        self.cmd_data(0xED, &[0x64, 0x03, 0x12, 0x81]);
        self.cmd_data(0xF7, &[0x20]);
        self.cmd_data(0xC0, &[0x23]);       // Power control 1
        self.cmd_data(0xC1, &[0x10]);       // Power control 2
        self.cmd_data(0xC5, &[0x3E, 0x28]); // VCOM control 1
        self.cmd_data(0xC7, &[0x86]);       // VCOM control 2

        // Pixel format / orientation
        self.cmd_data(0x36, &[0x48]);       // MADCTL: BGR
        self.cmd_data(0x3A, &[0x55]);       // 16-bit RGB565

        self.cmd_data(0xB1, &[0x00, 0x1B]); // 70 Hz frame rate
        self.cmd_data(0xB6, &[0x08, 0x82, 0x27]);

        // Gamma
        self.cmd_data(0xF2, &[0x00]);
        self.cmd_data(0x26, &[0x01]);
        self.cmd_data(0xE0, &[
            0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1,
            0x37, 0x07, 0x10, 0x03, 0x0E, 0x09, 0x00,
        ]);
        self.cmd_data(0xE1, &[
            0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1,
            0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36, 0x0F,
        ]);

        self.cmd(0x29); // DISPON
        self.delay.delay_ms(100);
        println!("[LCD] ILI9341 initialized ({}x{})", LCD_W, LCD_H);
    }

    fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) {
        self.cmd_data(0x2A, &[(x0 >> 8) as u8, x0 as u8, (x1 >> 8) as u8, x1 as u8]);
        self.cmd_data(0x2B, &[(y0 >> 8) as u8, y0 as u8, (y1 >> 8) as u8, y1 as u8]);
        self.cmd(0x2C);
    }

    fn fill(&mut self, color: Rgb565) {
        self.fill_area(0, 0, LCD_W - 1, LCD_H - 1, color);
    }

    /// Stream a raw RGB565 buffer (bytes already in big-endian / hi,lo order
    /// per pixel) into the rectangle [x0,y0]..[x1,y1].
    fn blit_rgb565(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, data: &[u8]) {
        self.set_window(x0, y0, x1, y1);
        self.dc.set_high();
        self.cs.set_low();
        // SPI driver writes are blocking; chunk to ~512 bytes for FIFO friendliness.
        let mut i = 0usize;
        while i < data.len() {
            let n = core::cmp::min(512, data.len() - i);
            let _ = self.spi.write(&data[i..i + n]);
            i += n;
        }
        self.cs.set_high();
    }

    fn fill_area(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, color: Rgb565) {
        let raw: u16 = color.into_storage();
        let hi = (raw >> 8) as u8;
        let lo = raw as u8;
        let mut buf = [0u8; 64];
        let mut i = 0;
        while i < 64 {
            buf[i] = hi;
            buf[i + 1] = lo;
            i += 2;
        }
        self.set_window(x0, y0, x1, y1);
        self.dc.set_high();
        self.cs.set_low();
        let total = (x1 - x0 + 1) as u32 * (y1 - y0 + 1) as u32;
        let chunks = total / 32;
        let rem = (total % 32) as usize;
        for _ in 0..chunks {
            let _ = self.spi.write(&buf);
        }
        if rem > 0 {
            let _ = self.spi.write(&buf[..rem * 2]);
        }
        self.cs.set_high();
    }
}

impl DrawTarget for Lcd<'_> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0 && coord.x < LCD_W as i32 && coord.y >= 0 && coord.y < LCD_H as i32 {
                self.set_window(coord.x as u16, coord.y as u16, coord.x as u16, coord.y as u16);
                let raw: u16 = color.into_storage();
                self.data(&[(raw >> 8) as u8, raw as u8]);
            }
        }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let area = area.intersection(&self.bounding_box());
        if let Some(br) = area.bottom_right() {
            self.fill_area(
                area.top_left.x as u16, area.top_left.y as u16,
                br.x as u16, br.y as u16, color,
            );
        }
        Ok(())
    }
}

impl OriginDimensions for Lcd<'_> {
    fn size(&self) -> Size {
        Size::new(LCD_W as u32, LCD_H as u32)
    }
}

// ===========================================================================
// Menu definition and drawing helpers
// ===========================================================================
const MENU_ITEMS: [&str; 12] = [
    "1. Display Test",
    "2. RGB LEDs",
    "3. Indicator LED",
    "4. Buttons",
    "5. Accelerometer",
    "6. Audio Codec",
    "7. Touch (FT6336U)",
    "8. SE050 Secure",
    "9. XL9535 I/O",
    "10. I2C Bus Scan",
    "11. System Info",
    "12. Camera Live",
];
const VISIBLE: usize = 7;
const ITEMS_Y0: i32 = 110;
const ITEM_H: i32 = 24;

// Sensor / device addresses on the shared I2C bus
const ACC_ADDR: u8 = 0x19;        // SC7A20 accelerometer
const ACC_WHO_AM_I: u8 = 0x0F;
const ACC_CTRL_REG1: u8 = 0x20;
const ACC_OUT_X_L: u8 = 0x28;
const AUDIO_ADC_ADDR: u8 = 0x11;  // ES7243E mic ADC (typical)
const TOUCH_ADDR: u8 = 0x38;      // FT6336U capacitive touch controller
const SE050_ADDR: u8 = 0x48;      // SE050 secure element (separate I2C addr)

// FT6336U registers we care about.
const FT_REG_TD_STATUS: u8 = 0x02;
const FT_REG_TOUCH1_XH: u8 = 0x03;

// XL9535 OUTPUT_0 helpers: toggle reset lines while keeping BL on.
// Active LOW: bit cleared = chip in reset.
// `ft_run` controls the FT6336U capacitive touch reset (was "ts_run").
fn xl_set_resets(i2c: &mut I2c<'_, Blocking>, ft_run: bool, se_run: bool, c_run: bool) {
    let mut v = BL_BIT; // backlight stays on
    if ft_run { v |= FT_RST_BIT; }
    if se_run { v |= SE_RST_BIT; }
    if c_run  { v |= C_RST_BIT;  }
    let _ = i2c.write(XL_ADDR, &[XL_OUTPUT_0, v]);
}

// ---------------------------------------------------------------------------
// FT6336U capacitive touch helper.
// Returns (finger_count, x, y) in screen pixels (240x320 native landscape on
// this board). Only first finger is reported.
// ---------------------------------------------------------------------------
fn ft6336_read(i2c: &mut I2c<'_, Blocking>) -> Option<(u8, u16, u16)> {
    let mut td = [0u8; 1];
    i2c.write_read(TOUCH_ADDR, &[FT_REG_TD_STATUS], &mut td).ok()?;
    let pts = td[0] & 0x0F;
    if pts == 0 || pts > 2 {
        return Some((0, 0, 0));
    }
    let mut d = [0u8; 4];
    i2c.write_read(TOUCH_ADDR, &[FT_REG_TOUCH1_XH], &mut d).ok()?;
    let x = (((d[0] & 0x0F) as u16) << 8) | d[1] as u16;
    let y = (((d[2] & 0x0F) as u16) << 8) | d[3] as u16;
    Some((pts, x, y))
}

// ---------------------------------------------------------------------------
// ES7243E (16 kHz, 32-bit Philips I2S, stereo, mic gain ~24 dB) init script.
// Distilled from the Espressif esp-adf ES7243E driver and the SkyRizz Arduino
// reference (.ino). Returns true if all I2C writes ACK'd.
// ---------------------------------------------------------------------------
fn es7243_write(i2c: &mut I2c<'_, Blocking>, reg: u8, val: u8) -> bool {
    i2c.write(AUDIO_ADC_ADDR, &[reg, val]).is_ok()
}

fn es7243_init(i2c: &mut I2c<'_, Blocking>) -> bool {
    // (reg, value) pairs — apply in order. 24 dB analog gain (reg 20/21 = 0x18).
    const SEQ: &[(u8, u8)] = &[
        (0x01, 0x3A), (0x00, 0x80), (0xF9, 0x00),
        (0x04, 0x02), (0x04, 0x01), (0xF9, 0x01), (0x00, 0x1E),
        (0x01, 0x00), (0x02, 0x00), (0x03, 0x20), (0x04, 0x01),
        (0x0D, 0x00), (0x05, 0x00), (0x06, 0x03), (0x07, 0x00),
        (0x08, 0xFF), (0x09, 0xCA),
        (0x0A, 0x85), (0x0B, 0x00),
        (0x0E, 0xBF), (0x0F, 0x80),
        (0x14, 0x0C), (0x15, 0x0C),
        (0x17, 0x02), (0x18, 0x26), (0x19, 0x77), (0x1A, 0xF4),
        (0x1B, 0x66), (0x1C, 0x44), (0x1E, 0x00), (0x1F, 0x0C),
        // PGA gain: 0x10 | (gain_db / 3); 24/3 = 8 -> 0x18.
        (0x20, 0x18), (0x21, 0x18),
        (0x00, 0x80), (0x01, 0x3A),
        (0x16, 0x3F), (0x16, 0x00),
        // second gain commit (matches reference driver)
        (0x20, 0x18), (0x21, 0x18),
        (0x00, 0x80), (0x01, 0x3A),
        (0x16, 0x3F), (0x16, 0x00),
    ];
    let mut all_ok = true;
    for (r, v) in SEQ {
        if !es7243_write(i2c, *r, *v) {
            all_ok = false;
        }
    }
    all_ok
}

/// Holds the LCD_CAM camera peripheral + DMA buffer between frames.
struct CamState {
    camera: Option<Camera<'static>>,
    buf: Option<DmaRxBuf>,
    sensor_inited: bool,
    last_status: CamStatus,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CamStatus {
    Unknown,
    NoPeripheral,
    SensorNotFound,
    SensorBusError,
    InitFailed,
    Streaming,
}

impl CamState {
    const fn new() -> Self {
        Self {
            camera: None,
            buf: None,
            sensor_inited: false,
            last_status: CamStatus::Unknown,
        }
    }
}

/// Live state for the audio meter detail screen — keeps the previous bar
/// width so we only repaint the delta and avoid a fill-then-draw flash.
struct AudioUi { prev_l: u16, prev_r: u16 }
impl AudioUi { const fn new() -> Self { Self { prev_l: 0, prev_r: 0 } } }

/// Live state for the touch detail screen — remembers the prior marker so we
/// can erase exactly that rectangle instead of wiping the whole pad.
struct TouchUi { prev_drawn: bool, prev_x: u16, prev_y: u16 }
impl TouchUi {
    const fn new() -> Self { Self { prev_drawn: false, prev_x: 0, prev_y: 0 } }
}

fn ensure_visible(view_top: &mut usize, selected: usize) {
    if selected < *view_top {
        *view_top = selected;
    } else if selected + 1 > *view_top + VISIBLE {
        *view_top = selected + 1 - VISIBLE;
    }
}

fn draw_menu(lcd: &mut Lcd<'_>, i2c_ok: bool, bl_on: bool, devices: u32,
             selected: usize, view_top: usize) {
    let bg = Rgb565::BLACK;
    let fg = Rgb565::WHITE;
    let green = Rgb565::new(0, 63, 0);
    let amber = Rgb565::new(28, 40, 0);
    let red = Rgb565::new(31, 0, 0);
    let blue = Rgb565::new(8, 16, 31);
    let dark = Rgb565::new(4, 8, 4);

    lcd.fill(bg);

    Rectangle::new(Point::new(0, 0), Size::new(240, 32))
        .into_styled(PrimitiveStyle::with_fill(blue))
        .draw(lcd).ok();
    Text::new("SkyRizz E32 Test", Point::new(40, 22),
        MonoTextStyle::new(&FONT_9X18_BOLD, fg)).draw(lcd).ok();

    let mut buf = [0u8; 40];
    let status = fmt(&mut buf, format_args!("I2C:{} BL:{} Dev:{}",
        if i2c_ok {"OK"} else {"NO"},
        if bl_on  {"ON"} else {"OFF"},
        devices));
    let status_color = if i2c_ok && bl_on { green }
                       else if i2c_ok      { amber }
                       else                { red };
    Text::new(status, Point::new(10, 56),
        MonoTextStyle::new(&FONT_9X18_BOLD, status_color)).draw(lcd).ok();

    Text::new("Tap=open  Hold=back", Point::new(10, 80),
        MonoTextStyle::new(&FONT_9X18_BOLD, fg)).draw(lcd).ok();

    Rectangle::new(Point::new(10, 96), Size::new(220, 2))
        .into_styled(PrimitiveStyle::with_fill(dark))
        .draw(lcd).ok();

    draw_menu_items(lcd, selected, view_top);

    Text::new("SW1=UP SW3=DN SW2=OK", Point::new(20, 308),
        MonoTextStyle::new(&FONT_9X18_BOLD, dark)).draw(lcd).ok();
}

/// Redraw only the menu items area (with scrolling).
///
/// We deliberately do NOT pre-fill the strip with black: every row paints
/// its own full-width rectangle, and consecutive rows fully overwrite the
/// previous frame's pixels. Skipping the bulk fill eliminates the
/// black-flash that used to appear on every up/down keystroke.
fn draw_menu_items(lcd: &mut Lcd<'_>, selected: usize, view_top: usize) {
    let fg = Rgb565::WHITE;
    let highlight_bg = Rgb565::new(0, 32, 16);
    let highlight_fg = Rgb565::BLACK;
    let dark = Rgb565::new(4, 8, 4);

    let end = (view_top + VISIBLE).min(MENU_ITEMS.len());
    let mut row = 0i32;
    for i in view_top..end {
        let y = ITEMS_Y0 + row * ITEM_H;
        let is_sel = i == selected;
        let row_bg = if is_sel { highlight_bg } else { dark };
        let txt_fg = if is_sel { highlight_fg } else { fg };
        // 230x22 row tile: rows are 24 px tall, so each tile fully overwrites
        // the previous tile at this slot — no separate clear needed.
        Rectangle::new(Point::new(5, y - 2), Size::new(230, 22))
            .into_styled(PrimitiveStyle::with_fill(row_bg))
            .draw(lcd).ok();
        Text::new(MENU_ITEMS[i], Point::new(15, y + 14),
            MonoTextStyle::new(&FONT_9X18_BOLD, txt_fg)).draw(lcd).ok();
        row += 1;
    }
    // If the visible window is shorter than VISIBLE rows (last page), blank
    // out the trailing row slots so stale text from a longer page disappears.
    while (row as usize) < VISIBLE {
        let y = ITEMS_Y0 + row * ITEM_H;
        Rectangle::new(Point::new(5, y - 2), Size::new(230, 22))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(lcd).ok();
        row += 1;
    }
    if view_top > 0 {
        Text::new("^", Point::new(220, ITEMS_Y0 + 12),
            MonoTextStyle::new(&FONT_9X18_BOLD, fg)).draw(lcd).ok();
    }
    if end < MENU_ITEMS.len() {
        Text::new("v", Point::new(220, ITEMS_Y0 + (VISIBLE as i32 - 1) * ITEM_H + 12),
            MonoTextStyle::new(&FONT_9X18_BOLD, fg)).draw(lcd).ok();
    }
}

/// Stack-only formatter. Usage: `fmt(&mut buf, format_args!("x={}", v))`
fn fmt<'a>(buf: &'a mut [u8], args: core::fmt::Arguments) -> &'a str {
    use core::fmt::Write;
    struct W<'b> { buf: &'b mut [u8], pos: usize }
    impl<'b> Write for W<'b> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let n = (self.buf.len() - self.pos).min(bytes.len());
            self.buf[self.pos..self.pos + n].copy_from_slice(&bytes[..n]);
            self.pos += n;
            Ok(())
        }
    }
    let mut w = W { buf, pos: 0 };
    let _ = w.write_fmt(args);
    let pos = w.pos;
    core::str::from_utf8(&buf[..pos]).unwrap_or("")
}

// ===========================================================================
// WS2812 bit-bang on GPIO46 (RGB1 -> RGB2)
// ===========================================================================
fn ws2812_send(pin: &mut Output<'_>, delay: &mut Delay, colors: &[(u8, u8, u8)]) {
    for &(r, g, b) in colors {
        for byte in [g, r, b] {
            for i in (0..8).rev() {
                if (byte >> i) & 1 == 1 {
                    pin.set_high();
                    delay.delay_ns(550);
                    pin.set_low();
                    delay.delay_ns(200);
                } else {
                    pin.set_high();
                    core::hint::black_box(0u32);
                    core::hint::black_box(0u32);
                    pin.set_low();
                    delay.delay_ns(600);
                }
            }
        }
    }
    delay.delay_us(300);
}

// ===========================================================================
// Entry point
// ===========================================================================
#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut delay = Delay::new();

    println!("========================================");
    println!("  SkyRizz E32 - Display Menu (Rust)");
    println!("========================================");

    // ---- Status LEDs (yellow = booting) ----
    let mut rgb = Output::new(peripherals.GPIO46, Level::Low, OutputConfig::default());
    ws2812_send(&mut rgb, &mut delay, &[(40, 40, 0), (40, 40, 0)]);

    // ---- LCD SPI ----
    println!("[SPI] SCLK=12 MOSI=21 CS=14 DC=13 @8MHz (slow for cable tolerance)");
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(8))
            .with_mode(Mode::_0),
    )
    .expect("SPI init")
    .with_sck(peripherals.GPIO12)
    .with_mosi(peripherals.GPIO21);

    let dc = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());

    let mut lcd = Lcd::new(spi, dc, cs);
    lcd.init();
    lcd.fill(Rgb565::BLACK);

    // ---- I2C on GPIO47 (SDA) / GPIO48 (SCL) ----
    println!("[I2C] SDA=GPIO47  SCL=GPIO48 @100kHz");
    let mut i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default()
            .with_frequency(Rate::from_khz(100))
            .with_timeout(BusTimeout::BusCycles(20))
            .with_software_timeout(SoftwareTimeout::Transaction(Duration::from_millis(250))),
    )
    .expect("I2C init")
    .with_sda(peripherals.GPIO47)
    .with_scl(peripherals.GPIO48);

    // ---- I2C bus scan ----
    println!("[I2C] Scanning bus...");
    let mut device_count: u32 = 0;
    for addr in 0x08u8..=0x77u8 {
        let mut buf = [0u8; 1];
        if i2c.read(addr, &mut buf).is_ok() {
            println!("[I2C]   0x{:02X} ACK", addr);
            device_count += 1;
        }
    }
    println!("[I2C] {} device(s) found", device_count);

    // ---- XL9535 init ----
    let mut i2c_ok = false;
    let mut bl_on = false;

    let mut probe = [0u8; 1];
    if i2c.write_read(XL_ADDR, &[XL_INPUT_0], &mut probe).is_ok() {
        println!("[XL] Found at 0x{:02X} (Input0=0x{:02X})", XL_ADDR, probe[0]);
        i2c_ok = true;

        // Configure all of port 0 as output (P00..P03 used, P04..P07 spare).
        // Outputs: BL=1 (on), FT_RST=1, C_RST=1, SE_RST=1 -> 0x0F
        // Drive output BEFORE switching to output direction to avoid glitches.
        let _ = i2c.write(XL_ADDR, &[XL_OUTPUT_0, BL_BIT | FT_RST_BIT | C_RST_BIT | SE_RST_BIT]);
        let _ = i2c.write(XL_ADDR, &[XL_CONFIG_0, 0xF0]); // P00..P03 = output, P04..P07 = input

        // Port 1: only P17 (IND LED) is an output for us
        let _ = i2c.write(XL_ADDR, &[XL_OUTPUT_1, IND_LED_BIT]);
        let _ = i2c.write(XL_ADDR, &[XL_CONFIG_1, 0x7F]); // P17 = output, rest input

        // Read back configs
        let mut r = [0u8; 1];
        if i2c.write_read(XL_ADDR, &[XL_CONFIG_0], &mut r).is_ok() {
            println!("[XL] Cfg0=0x{:02X} (want 0xF0) {}", r[0],
                if r[0] == 0xF0 { "OK" } else { "MISMATCH" });
        }
        if i2c.write_read(XL_ADDR, &[XL_OUTPUT_0], &mut r).is_ok() {
            println!("[XL] Out0=0x{:02X}", r[0]);
            if r[0] & BL_BIT != 0 {
                bl_on = true;
            }
        }
        println!("[XL] Backlight {}", if bl_on { "ON" } else { "OFF" });
    } else {
        println!("[XL] NOT responding at 0x{:02X}", XL_ADDR);
    }

    // ===================================================================
    //  Interactive menu - exercises every onboard module
    // ===================================================================

    // Init SC7A20 accelerometer (100 Hz, all axes enabled, normal mode).
    if i2c_ok {
        let _ = i2c.write(ACC_ADDR, &[ACC_CTRL_REG1, 0x57]);
    }

    // ---- FT6336U capacitive touch reset pulse (XL9535 P01, active LOW) ----
    // Sequence: HIGH (default) -> LOW 50ms -> HIGH wait 400ms.
    if i2c_ok {
        xl_set_resets(&mut i2c, false, true, true); // FT in reset
        delay.delay_ms(50);
        xl_set_resets(&mut i2c, true,  true, true); // FT released
        delay.delay_ms(400);
        let mut probe = [0u8; 1];
        if i2c.read(TOUCH_ADDR, &mut probe).is_ok() {
            println!("[TOUCH] FT6336U ACK at 0x{:02X}", TOUCH_ADDR);
        } else {
            println!("[TOUCH] FT6336U not responding (0x{:02X})", TOUCH_ADDR);
        }
    }

    // ---- ES7243E mic ADC: I2C config first, I2S RX next ----
    let mut audio_inited = false;
    if i2c_ok {
        // Quick presence probe before sending the long init script.
        let mut probe = [0u8; 1];
        if i2c.read(AUDIO_ADC_ADDR, &mut probe).is_ok() {
            println!("[ES7243E] ACK at 0x{:02X}, running init script", AUDIO_ADC_ADDR);
            audio_inited = es7243_init(&mut i2c);
            println!("[ES7243E] init {}", if audio_inited { "OK" } else { "FAILED" });
        } else {
            println!("[ES7243E] no ACK at 0x{:02X}", AUDIO_ADC_ADDR);
        }
    }

    // Build the I2S0 RX channel (16-bit data on 32-bit slots, stereo, 16 kHz).
    // MCLK GPIO3 / BCLK GPIO0 / WS=LRCK GPIO38 / DIN GPIO39.
    let mut i2s_rx: Option<I2sRx<'static, Blocking>> = None;
    if audio_inited {
        const I2S_BUF_BYTES: usize = 4096;
        let (_rx_buf, rx_descriptors, _, _) = dma_buffers!(I2S_BUF_BYTES, 0);
        let i2s_cfg = I2sConfig::new_tdm_philips()
            .with_sample_rate(Rate::from_hz(16_000))
            .with_data_format(I2sDataFormat::Data32Channel32)
            .with_channels(I2sChannels::STEREO);
        match I2s::new(peripherals.I2S0, peripherals.DMA_CH1, i2s_cfg) {
            Ok(i2s) => {
                let i2s = i2s.with_mclk(peripherals.GPIO3);
                let rx = i2s.i2s_rx
                    .with_bclk(peripherals.GPIO0)
                    .with_ws(peripherals.GPIO38)
                    .with_din(peripherals.GPIO39)
                    .build(rx_descriptors);
                i2s_rx = Some(rx);
                println!("[I2S] RX ready: MCLK=GPIO3 BCLK=GPIO0 WS=GPIO38 DIN=GPIO39");
            }
            Err(_) => println!("[I2S] init FAILED"),
        }
    }

    // ---- Camera (GC2145 over DVP / LCD_CAM) ----
    // Build the camera peripheral; sensor SCCB init happens lazily on first
    // entry to the Camera screen so an unplugged FPC3 doesn't slow boot.
    let mut cam_state = CamState::new();
    let mut audio_ui = AudioUi::new();
    let mut touch_ui = TouchUi::new();
    if i2c_ok {
        // 320x240 RGB565 = 153,600 bytes in DRAM.
        let (rx_buffer, rx_descriptors, _, _) = dma_buffers!(camera::FRAME_BYTES, 0);
        match DmaRxBuf::new(rx_descriptors, rx_buffer) {
            Ok(dma_rx_buf) => {
                let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
                let cam_cfg = CamConfig::default()
                    .with_frequency(Rate::from_mhz(20))
                    // GC2145 (and all DVP cams) drive HREF as a level signal
                    // that gates valid pixel clocks. That is DE behavior, not
                    // a real HSYNC pulse, so the LCD_CAM peripheral must use
                    // De mode — otherwise no pixels are latched at all and
                    // the DMA buffer keeps whatever (often 0xFF) it had.
                    .with_vh_de_mode(VhdeMode::De)
                    .with_vsync_filter_threshold(VsyncFilterThreshold::Three)
                    .with_eof_mode(EofMode::VsyncSignal);
                match Camera::new(lcd_cam.cam, peripherals.DMA_CH0, cam_cfg) {
                    Ok(camera) => {
                        let camera = camera
                            .with_master_clock(peripherals.GPIO7)
                            .with_pixel_clock(peripherals.GPIO17)
                            .with_vsync(peripherals.GPIO4)
                            .with_h_enable(peripherals.GPIO5)
                            .with_data0(peripherals.GPIO8)
                            .with_data1(peripherals.GPIO10)
                            .with_data2(peripherals.GPIO11)
                            .with_data3(peripherals.GPIO9)
                            .with_data4(peripherals.GPIO18)
                            .with_data5(peripherals.GPIO16)
                            .with_data6(peripherals.GPIO15)
                            .with_data7(peripherals.GPIO6);
                        println!("[CAM] LCD_CAM ready, XCLK on GPIO7 @20 MHz");

                        // XCLK is now active. Pulse C_RST (XL9535 P02, active
                        // LOW) so the GC2145 boots with its master clock alive
                        // — SCCB will not ACK otherwise.
                        xl_set_resets(&mut i2c, true, true, false);
                        delay.delay_ms(50);
                        xl_set_resets(&mut i2c, true, true, true);
                        delay.delay_ms(300);

                        cam_state.camera = Some(camera);
                        cam_state.buf = Some(dma_rx_buf);
                    }
                    Err(_) => println!("[CAM] Camera::new failed"),
                }
            }
            Err(_) => println!("[CAM] DmaRxBuf::new failed"),
        }
    }

    println!("[UI] Drawing menu...");
    let mut selected: usize = 0;
    let mut view_top: usize = 0;
    let mut in_detail = false;
    let mut detail_init = false;
    let mut detail_tick: u32 = 0;
    draw_menu(&mut lcd, i2c_ok, bl_on, device_count, selected, view_top);

    // Final WS2812 status
    let color = if i2c_ok && bl_on {
        (0, 40, 0)
    } else if i2c_ok {
        (40, 30, 0)
    } else {
        (40, 0, 0)
    };
    ws2812_send(&mut rgb, &mut delay, &[color, color]);

    println!("========================================");
    println!("  I2C={}  BL={}  Devices={}",
        if i2c_ok { "OK" } else { "FAIL" },
        if bl_on { "ON" } else { "OFF" },
        device_count);
    println!("  SW1=UP  SW3=DOWN  SW2=SELECT (hold=BACK)");
    println!("========================================");

    // Active-low button masks.
    const SW2_MASK_P0: u8 = 1 << 4; // port0
    const SW3_MASK_P1: u8 = 1 << 1; // port1
    const SW1_MASK_P1: u8 = 1 << 2; // port1

    let mut prev_p0: u8 = 0xFF;
    let mut prev_p1: u8 = 0xFF;

    const POLL_MS: u32 = 20;
    const HOLD_TICKS: u32 = 600 / POLL_MS;
    const REPEAT_FIRST: u32 = 400 / POLL_MS;
    const REPEAT_INTERVAL: u32 = 120 / POLL_MS;

    let mut sw2_press_tick: u32 = 0;
    let mut sw2_back_fired: bool = false;
    let mut sw1_held_since: u32 = 0;
    let mut sw3_held_since: u32 = 0;
    let mut sw1_next_repeat: u32 = 0;
    let mut sw3_next_repeat: u32 = 0;

    let mut tick: u32 = 0;
    let mut led_on = true;
    let mut prev_touch_pts: u8 = 0;
    // Touch-drag scroll state for the menu screen.
    let mut touch_start_y: i32 = -1;     // y at touch-down
    let mut touch_last_y:  i32 = -1;     // last y seen during drag
    let mut touch_acc:     i32 = 0;      // unscrolled pixel delta
    let mut touch_dragged: bool = false; // any motion > tap threshold

    loop {
        delay.delay_ms(POLL_MS);
        tick = tick.wrapping_add(1);

        if !i2c_ok {
            continue;
        }

        let mut r0 = [0u8; 1];
        let mut r1 = [0u8; 1];
        if i2c.write_read(XL_ADDR, &[XL_INPUT_0], &mut r0).is_err() { continue; }
        if i2c.write_read(XL_ADDR, &[XL_INPUT_1], &mut r1).is_err() { continue; }
        let p0 = r0[0];
        let p1 = r1[0];

        let sw1_down = (p1 & SW1_MASK_P1) == 0;
        let sw2_down = (p0 & SW2_MASK_P0) == 0;
        let sw3_down = (p1 & SW3_MASK_P1) == 0;

        let pressed_p0 = prev_p0 & !p0;
        let pressed_p1 = prev_p1 & !p1;
        let released_p0 = !prev_p0 & p0;
        prev_p0 = p0;
        prev_p1 = p1;

        let mut want_redraw_items = false;
        let mut want_full_redraw = false;
        let mut do_select = false;
        let mut do_back = false;

        // SW1 / SW3 navigate only on the menu screen.
        if !in_detail {
            if pressed_p1 & SW1_MASK_P1 != 0 {
                move_up(&mut selected);
                ensure_visible(&mut view_top, selected);
                println!("[BTN] SW1 UP    -> selected={}", selected);
                want_redraw_items = true;
                sw1_held_since = tick;
                sw1_next_repeat = tick + REPEAT_FIRST;
            } else if sw1_down && tick >= sw1_next_repeat && sw1_held_since != 0 {
                move_up(&mut selected);
                ensure_visible(&mut view_top, selected);
                want_redraw_items = true;
                sw1_next_repeat = tick + REPEAT_INTERVAL;
            } else if !sw1_down {
                sw1_held_since = 0;
            }

            if pressed_p1 & SW3_MASK_P1 != 0 {
                move_down(&mut selected);
                ensure_visible(&mut view_top, selected);
                println!("[BTN] SW3 DOWN  -> selected={}", selected);
                want_redraw_items = true;
                sw3_held_since = tick;
                sw3_next_repeat = tick + REPEAT_FIRST;
            } else if sw3_down && tick >= sw3_next_repeat && sw3_held_since != 0 {
                move_down(&mut selected);
                ensure_visible(&mut view_top, selected);
                want_redraw_items = true;
                sw3_next_repeat = tick + REPEAT_INTERVAL;
            } else if !sw3_down {
                sw3_held_since = 0;
            }
        } else {
            sw1_held_since = 0;
            sw3_held_since = 0;
        }

        // SW2 tap = SELECT, hold (>= 600 ms) = BACK. Works on both screens.
        if pressed_p0 & SW2_MASK_P0 != 0 {
            sw2_press_tick = tick;
            sw2_back_fired = false;
        }
        if sw2_down && !sw2_back_fired && sw2_press_tick != 0
            && tick - sw2_press_tick >= HOLD_TICKS
        {
            sw2_back_fired = true;
            do_back = true;
            println!("[BTN] SW2 HOLD  -> BACK");
        }
        if released_p0 & SW2_MASK_P0 != 0 && sw2_press_tick != 0 {
            if !sw2_back_fired {
                do_select = true;
                println!("[BTN] SW2 TAP   -> SELECT '{}'", MENU_ITEMS[selected]);
            }
            sw2_press_tick = 0;
            sw2_back_fired = false;
        }

        // ---- Touch navigation (FT6336U) -----------------------------------
        // Track press / drag / release for smooth scrolling on the menu and
        // tap-to-back on detail screens. Skip while inside the touch detail
        // screen so its test owns the I2C reads.
        if !(in_detail && selected == 6) && tick % 2 == 0 {
            if let Some((pts, _tx, ty)) = ft6336_read(&mut i2c) {
                let down = pts > 0;
                let prev_down = prev_touch_pts > 0;
                if down {
                    if !prev_down {
                        // Touch DOWN.
                        touch_start_y = ty as i32;
                        touch_last_y  = ty as i32;
                        touch_acc     = 0;
                        touch_dragged = false;
                    } else if !in_detail {
                        // Touch MOVE on menu screen → scroll by full rows
                        // whenever the accumulated drag exceeds ITEM_H.
                        let dy = ty as i32 - touch_last_y;
                        touch_last_y = ty as i32;
                        touch_acc += dy;
                        if (ty as i32 - touch_start_y).abs() > 8 {
                            touch_dragged = true;
                        }
                        let max_items = MENU_ITEMS.len() as i32;
                        let visible = VISIBLE as i32;
                        while touch_acc >= ITEM_H {
                            // Drag down → list scrolls down (view_top--).
                            if (view_top as i32) > 0 {
                                view_top -= 1;
                                if (selected as i32) > view_top as i32 + visible - 1 {
                                    selected = view_top + VISIBLE - 1;
                                }
                                want_redraw_items = true;
                            }
                            touch_acc -= ITEM_H;
                        }
                        while touch_acc <= -ITEM_H {
                            // Drag up → list scrolls up (view_top++).
                            if (view_top as i32) + visible < max_items {
                                view_top += 1;
                                if (selected as i32) < view_top as i32 {
                                    selected = view_top;
                                }
                                want_redraw_items = true;
                            }
                            touch_acc += ITEM_H;
                        }
                    }
                } else if prev_down {
                    // Touch UP.
                    if !touch_dragged {
                        // It was a tap.
                        let yi = touch_last_y;
                        if !in_detail {
                            if yi >= ITEMS_Y0 - 4
                                && yi < ITEMS_Y0 + (VISIBLE as i32) * ITEM_H
                            {
                                let row = ((yi - ITEMS_Y0).max(0) / ITEM_H) as usize;
                                let target = view_top + row;
                                if target < MENU_ITEMS.len() {
                                    if target != selected {
                                        selected = target;
                                        want_redraw_items = true;
                                    }
                                    do_select = true;
                                    println!("[TOUCH] tap row {} -> SELECT '{}'",
                                        row, MENU_ITEMS[selected]);
                                }
                            }
                        } else if (yi as u16) < (DETAIL_BODY_Y0 as u16) {
                            do_back = true;
                            println!("[TOUCH] header tap -> BACK");
                        }
                    }
                    touch_start_y = -1;
                    touch_last_y  = -1;
                    touch_acc     = 0;
                    touch_dragged = false;
                }
                prev_touch_pts = pts;
            }
        }

        if do_select && !in_detail {
            in_detail = true;
            detail_init = false;
            detail_tick = 0;
            ws2812_send(&mut rgb, &mut delay, &[(0, 0, 40), (0, 0, 40)]);
        }
        if do_back && in_detail {
            in_detail = false;
            want_full_redraw = true;
            ws2812_send(&mut rgb, &mut delay, &[(40, 20, 0), (40, 20, 0)]);
            delay.delay_ms(120);
            ws2812_send(&mut rgb, &mut delay, &[color, color]);
        }

        if want_full_redraw {
            draw_menu(&mut lcd, i2c_ok, bl_on, device_count, selected, view_top);
        } else if want_redraw_items && !in_detail {
            draw_menu_items(&mut lcd, selected, view_top);
        }

        // Live detail rendering: chrome on first tick, body updates each tick.
        if in_detail {
            let just_entered = !detail_init;
            if just_entered {
                draw_detail_chrome(&mut lcd, selected);
                detail_init = true;
            }
            match selected {
                0 => detail_display_test(&mut lcd, detail_tick, just_entered),
                1 => detail_rgb_leds(&mut lcd, &mut rgb, &mut delay, detail_tick, just_entered),
                2 => detail_ind_led(&mut lcd, &mut i2c, detail_tick, just_entered, &mut led_on),
                3 => detail_buttons(&mut lcd, p0, p1, detail_tick, just_entered),
                4 => detail_accel(&mut lcd, &mut i2c, detail_tick, just_entered),
                5 => detail_audio_codec(&mut lcd, &mut i2c, i2s_rx.as_mut(), &mut audio_ui, just_entered),
                6 => detail_touch(&mut lcd, &mut i2c, &mut touch_ui, detail_tick, just_entered),
                7 => detail_se050(&mut lcd, &mut i2c, just_entered),
                8 => detail_xl9535(&mut lcd, p0, p1, detail_tick, just_entered),
                9 => detail_i2c_scan(&mut lcd, &mut i2c, just_entered),
                10 => detail_sysinfo(&mut lcd, just_entered),
                11 => detail_camera(&mut lcd, &mut i2c, &mut cam_state, just_entered),
                _ => {}
            }
            detail_tick = detail_tick.wrapping_add(1);
        }

        // Heartbeat: blink IND LED every 1 s while on the menu screen.
        if !in_detail && tick % 50 == 0 {
            led_on = !led_on;
            let v1 = if led_on { IND_LED_BIT } else { 0 };
            let _ = i2c.write(XL_ADDR, &[XL_OUTPUT_1, v1]);
            if tick % 250 == 0 {
                println!("[TICK] t={}s  screen=MENU  sel={}  P0=0x{:02X} P1=0x{:02X}",
                    tick / 50, selected, p0, p1);
            }
        }
    }
}

fn move_up(selected: &mut usize) {
    *selected = if *selected == 0 { MENU_ITEMS.len() - 1 } else { *selected - 1 };
}
fn move_down(selected: &mut usize) {
    *selected = (*selected + 1) % MENU_ITEMS.len();
}

// ===========================================================================
// Detail screens - one per menu item
// ===========================================================================

const DETAIL_BODY_Y0: u16 = 40;
const DETAIL_BODY_Y1: u16 = 290;

fn draw_detail_chrome(lcd: &mut Lcd<'_>, selected: usize) {
    let bg = Rgb565::BLACK;
    let fg = Rgb565::WHITE;
    let blue = Rgb565::new(8, 16, 31);
    let dark = Rgb565::new(4, 8, 4);

    lcd.fill(bg);
    Rectangle::new(Point::new(0, 0), Size::new(240, 32))
        .into_styled(PrimitiveStyle::with_fill(blue))
        .draw(lcd).ok();
    Text::new(MENU_ITEMS[selected], Point::new(10, 22),
        MonoTextStyle::new(&FONT_9X18_BOLD, fg)).draw(lcd).ok();
    Text::new("Hold SW2 = BACK", Point::new(40, 308),
        MonoTextStyle::new(&FONT_9X18_BOLD, dark)).draw(lcd).ok();
}

fn body_clear(lcd: &mut Lcd<'_>) {
    lcd.fill_area(0, DETAIL_BODY_Y0, 239, DETAIL_BODY_Y1, Rgb565::BLACK);
}

/// Clear a 24 px tall strip and draw text at (x, y).
// Draw a single text line, flicker-free. The glyph cells are rendered with
// an opaque black background so the row is repainted in a single SPI pass —
// no fill-then-draw flash. Callers must use stable-width formats for live
// fields; an initial `body_clear()` wipes leftover content from prior screens.
fn line(lcd: &mut Lcd<'_>, x: i32, y: i32, color: Rgb565, s: &str) {
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(color)
        .background_color(Rgb565::BLACK)
        .build();
    Text::new(s, Point::new(x, y), style).draw(lcd).ok();
}

// 1) Display test: cycle full-screen colors, ~1 s each.
fn detail_display_test(lcd: &mut Lcd<'_>, tick: u32, just_entered: bool) {
    let phase = (tick / 50) % 6;
    let prev_phase = if tick == 0 { 99 } else { ((tick - 1) / 50) % 6 };
    if just_entered || phase != prev_phase {
        let col = match phase {
            0 => Rgb565::RED,
            1 => Rgb565::GREEN,
            2 => Rgb565::BLUE,
            3 => Rgb565::WHITE,
            4 => Rgb565::new(31, 40, 0),
            _ => Rgb565::BLACK,
        };
        lcd.fill_area(0, DETAIL_BODY_Y0, 239, DETAIL_BODY_Y1, col);
        let label = match phase {
            0 => "RED", 1 => "GREEN", 2 => "BLUE",
            3 => "WHITE", 4 => "AMBER", _ => "BLACK",
        };
        let txt_color = if phase == 5 { Rgb565::WHITE } else { Rgb565::BLACK };
        Text::new(label, Point::new(95, 165),
            MonoTextStyle::new(&FONT_9X18_BOLD, txt_color)).draw(lcd).ok();
    }
}

// 2) RGB LEDs: cycle WS2812 colors.
fn detail_rgb_leds(lcd: &mut Lcd<'_>, rgb: &mut Output<'_>, delay: &mut Delay,
                   tick: u32, just_entered: bool) {
    if just_entered {
        body_clear(lcd);
        line(lcd, 10, 70, Rgb565::WHITE, "WS2812 chain GPIO46");
        line(lcd, 10, 95, Rgb565::WHITE, "RGB1 -> RGB2");
    }
    if tick % 4 == 0 {
        let phase = (tick / 4) % 6;
        let c = match phase {
            0 => (40, 0, 0),
            1 => (40, 20, 0),
            2 => (40, 40, 0),
            3 => (0, 40, 0),
            4 => (0, 0, 40),
            _ => (20, 0, 40),
        };
        ws2812_send(rgb, delay, &[c, c]);
        let mut buf = [0u8; 40];
        let s = fmt(&mut buf, format_args!("R={:>3} G={:>3} B={:>3}", c.0, c.1, c.2));
        line(lcd, 10, 160, Rgb565::WHITE, s);
        // Color swatch box
        let swatch = Rgb565::new((c.0 >> 3).min(31), (c.1 >> 2).min(63), (c.2 >> 3).min(31));
        lcd.fill_area(10, 200, 230, 250, swatch);
    }
}

// 3) Indicator LED: fast blink XL9535 P17.
fn detail_ind_led(lcd: &mut Lcd<'_>, i2c: &mut I2c<'_, Blocking>,
                  tick: u32, just_entered: bool, led_on: &mut bool) {
    if just_entered {
        body_clear(lcd);
        line(lcd, 10, 70, Rgb565::WHITE, "XL9535 P17 blink");
        line(lcd, 10, 95, Rgb565::WHITE, "Net U_LED via R16");
    }
    if tick % 10 == 0 {
        *led_on = !*led_on;
        let v = if *led_on { IND_LED_BIT } else { 0 };
        let _ = i2c.write(XL_ADDR, &[XL_OUTPUT_1, v]);
        let (s, c) = if *led_on {
            ("LED: ON ", Rgb565::new(0, 63, 0))
        } else {
            ("LED: OFF", Rgb565::new(31, 0, 0))
        };
        line(lcd, 10, 165, c, s);
    }
}

// 4) Buttons: live state of all 5 onboard switches (active LOW).
fn detail_buttons(lcd: &mut Lcd<'_>, p0: u8, p1: u8, tick: u32, just_entered: bool) {
    if just_entered {
        body_clear(lcd);
        line(lcd, 10, 60, Rgb565::WHITE, "Press any button:");
    }
    if !just_entered && tick % 5 != 0 { return; } // ~10 Hz redraw
    let on  = Rgb565::new(0, 63, 0);
    let off = Rgb565::new(15, 15, 15);
    let states = [
        ("SW1 P12", (p1 & (1 << 2)) == 0),
        ("SW2 P04", (p0 & (1 << 4)) == 0),
        ("SW3 P11", (p1 & (1 << 1)) == 0),
        ("PB1 P05", (p0 & (1 << 5)) == 0),
        ("PB2 P06", (p0 & (1 << 6)) == 0),
    ];
    for (i, &(name, pressed)) in states.iter().enumerate() {
        let y = 100 + i as i32 * 30;
        let mut buf = [0u8; 32];
        let s = fmt(&mut buf, format_args!("{}  {}", name, if pressed {"DOWN"} else {"up  "}));
        line(lcd, 10, y, if pressed { on } else { off }, s);
    }
}

// 5) SC7A20 accelerometer: WHO_AM_I + live X/Y/Z.
fn detail_accel(lcd: &mut Lcd<'_>, i2c: &mut I2c<'_, Blocking>,
                tick: u32, just_entered: bool) {
    if just_entered {
        body_clear(lcd);
        line(lcd, 10, 95, Rgb565::WHITE, "SC7A20  addr 0x19");
    }
    if !just_entered && tick % 5 != 0 { return; } // ~10 Hz
    let mut who = [0u8; 1];
    let who_ok = i2c.write_read(ACC_ADDR, &[ACC_WHO_AM_I], &mut who).is_ok();

    let mut buf = [0u8; 40];
    if !who_ok {
        line(lcd, 10, 70, Rgb565::new(31, 0, 0), "0x19 not responding");
        return;
    }
    line(lcd, 10, 70, Rgb565::new(0, 63, 0),
        fmt(&mut buf, format_args!("WHO_AM_I = 0x{:02X}", who[0])));

    let mut data = [0u8; 6];
    if i2c.write_read(ACC_ADDR, &[ACC_OUT_X_L | 0x80], &mut data).is_ok() {
        let x = i16::from_le_bytes([data[0], data[1]]) >> 4;
        let y = i16::from_le_bytes([data[2], data[3]]) >> 4;
        let z = i16::from_le_bytes([data[4], data[5]]) >> 4;
        line(lcd, 10, 145, Rgb565::WHITE,
            fmt(&mut buf, format_args!("X = {:>5}", x)));
        line(lcd, 10, 175, Rgb565::WHITE,
            fmt(&mut buf, format_args!("Y = {:>5}", y)));
        line(lcd, 10, 205, Rgb565::WHITE,
            fmt(&mut buf, format_args!("Z = {:>5}", z)));
        // Tilt magnitude bar
        let mag = ((x as i32).abs() + (y as i32).abs() + (z as i32).abs()) as u32;
        let mag = mag.min(3000);
        lcd.fill_area(10, 245, 230, 260, Rgb565::new(4, 8, 4));
        let w = (mag * 220 / 3000) as u16;
        if w > 0 {
            lcd.fill_area(10, 245, 10 + w, 260, Rgb565::new(0, 32, 31));
        }
    }
}

// 6) ES7243E audio codec — show live mic levels for left + right channels.
fn detail_audio_codec(
    lcd: &mut Lcd<'_>,
    i2c: &mut I2c<'_, Blocking>,
    i2s_rx: Option<&mut I2sRx<'static, Blocking>>,
    ui: &mut AudioUi,
    just_entered: bool,
) {
    if just_entered {
        body_clear(lcd);
        ui.prev_l = 0;
        ui.prev_r = 0;
        let mut probe = [0u8; 1];
        let ack = i2c.read(AUDIO_ADC_ADDR, &mut probe).is_ok();
        let (msg, color) = if ack {
            ("ES7243E ACK at 0x11", Rgb565::new(0, 63, 0))
        } else {
            ("ES7243E NOT FOUND  ", Rgb565::new(31, 0, 0))
        };
        line(lcd, 10, 60,  color,         msg);
        line(lcd, 10, 85,  Rgb565::WHITE, "I2S 16 kHz, 32-bit stereo");
        line(lcd, 10, 110, Rgb565::WHITE, "MCLK=3 BCLK=0 WS=38 DIN=39");
        line(lcd, 10, 150, Rgb565::WHITE, "L:");
        line(lcd, 10, 200, Rgb565::WHITE, "R:");
        // Bar tracks (drawn once).
        lcd.fill_area(40, 150, 230, 178, Rgb565::new(4, 8, 4));
        lcd.fill_area(40, 200, 230, 228, Rgb565::new(4, 8, 4));
        line(lcd, 10, 245, Rgb565::WHITE, "L=  0%  R=  0%");
    }

    let i2s_rx = match i2s_rx {
        Some(rx) => rx,
        None => {
            if just_entered {
                line(lcd, 10, 280, Rgb565::new(31, 16, 0), "I2S not initialized");
            }
            return;
        }
    };

    // Read ~512 stereo frames (4 KB) of 32-bit samples.
    let mut samples = [0i32; 1024];
    if i2s_rx.read_words(&mut samples).is_err() {
        line(lcd, 10, 280, Rgb565::new(31, 16, 0), "I2S read error    ");
        return;
    }

    // Peak-detect per channel (interleaved L,R,L,R,...).
    let mut peak_l: u32 = 0;
    let mut peak_r: u32 = 0;
    for chunk in samples.chunks_exact(2) {
        let l = (chunk[0] >> 14).unsigned_abs();
        let r = (chunk[1] >> 14).unsigned_abs();
        if l > peak_l { peak_l = l; }
        if r > peak_r { peak_r = r; }
    }
    let map = |v: u32| -> u16 {
        (v.saturating_mul(100) / 2000).min(100) as u16
    };
    let pl = map(peak_l);
    let pr = map(peak_r);

    fn bar_color(pct: u16) -> Rgb565 {
        if pct < 60 { Rgb565::new(0, 50, 0) }
        else if pct < 85 { Rgb565::new(31, 50, 0) }
        else { Rgb565::new(31, 0, 0) }
    }
    // Update bar without wipe: paint new fill, erase only shrink delta.
    let track_bg = Rgb565::new(4, 8, 4);
    let lw = (190u16 * pl) / 100;
    let rw = (190u16 * pr) / 100;
    // L channel
    if lw > ui.prev_l {
        lcd.fill_area(40 + ui.prev_l, 152, (40 + lw).max(40 + ui.prev_l + 1), 176, bar_color(pl));
    } else if lw < ui.prev_l {
        lcd.fill_area(40 + lw, 152, 40 + ui.prev_l, 176, track_bg);
        if lw > 0 {
            lcd.fill_area(40, 152, 40 + lw, 176, bar_color(pl));
        }
    } else if lw > 0 {
        // Same width: refresh color (in case crossed threshold).
        lcd.fill_area(40, 152, 40 + lw, 176, bar_color(pl));
    }
    ui.prev_l = lw;
    // R channel
    if rw > ui.prev_r {
        lcd.fill_area(40 + ui.prev_r, 202, (40 + rw).max(40 + ui.prev_r + 1), 226, bar_color(pr));
    } else if rw < ui.prev_r {
        lcd.fill_area(40 + rw, 202, 40 + ui.prev_r, 226, track_bg);
        if rw > 0 {
            lcd.fill_area(40, 202, 40 + rw, 226, bar_color(pr));
        }
    } else if rw > 0 {
        lcd.fill_area(40, 202, 40 + rw, 226, bar_color(pr));
    }
    ui.prev_r = rw;

    // Numeric readout — fixed-width so the opaque text bg covers fully.
    let mut buf = [0u8; 40];
    line(lcd, 10, 245, Rgb565::WHITE,
        fmt(&mut buf, format_args!("L={:>3}%  R={:>3}%", pl, pr)));
}

// 7) FT6336U capacitive touch — show finger count + raw X/Y.
fn detail_touch(lcd: &mut Lcd<'_>, i2c: &mut I2c<'_, Blocking>,
                ui: &mut TouchUi, tick: u32, just_entered: bool) {
    if just_entered {
        body_clear(lcd);
        line(lcd, 10, 60, Rgb565::WHITE, "FT6336U @ 0x38");
        line(lcd, 10, 85, Rgb565::new(28, 40, 0), "Tap anywhere on screen");
        line(lcd, 10, 290, Rgb565::new(15, 15, 15), "released         ");
        // Touch pad indicator area (drawn once).
        lcd.fill_area(10, 110, 230, 285, Rgb565::new(4, 8, 4));
        ui.prev_drawn = false;
    }
    if !just_entered && tick % 2 != 0 { return; } // ~25 Hz

    let r = ft6336_read(i2c);
    let (pts, x, y) = match r {
        Some(t) => t,
        None => return,
    };

    let pad_bg = Rgb565::new(4, 8, 4);
    let marker_bg = Rgb565::new(0, 63, 31);
    let mut buf = [0u8; 40];

    if pts > 0 && x < 240 && y < 320 {
        let px = (10u16 + (x.min(239) as u32 * 220 / 240) as u16).clamp(14, 226);
        let py = (110u16 + ((y.saturating_sub(40)).min(239) as u32 * 175 / 240) as u16).clamp(114, 281);
        // Erase prior marker only if it moved.
        if ui.prev_drawn && (ui.prev_x != px || ui.prev_y != py) {
            let lx = ui.prev_x.saturating_sub(4);
            let ly = ui.prev_y.saturating_sub(4);
            lcd.fill_area(lx, ly, ui.prev_x + 4, ui.prev_y + 4, pad_bg);
        }
        lcd.fill_area(px.saturating_sub(4), py.saturating_sub(4),
                      px + 4, py + 4, marker_bg);
        ui.prev_drawn = true;
        ui.prev_x = px;
        ui.prev_y = py;
        line(lcd, 10, 290, Rgb565::new(0, 63, 0),
            fmt(&mut buf, format_args!("F={} X={:>3} Y={:>3}", pts, x, y)));
    } else if ui.prev_drawn {
        let lx = ui.prev_x.saturating_sub(4);
        let ly = ui.prev_y.saturating_sub(4);
        lcd.fill_area(lx, ly, ui.prev_x + 4, ui.prev_y + 4, pad_bg);
        ui.prev_drawn = false;
        line(lcd, 10, 290, Rgb565::new(15, 15, 15), "released         ");
    }
}

// 8) SE050 secure element. Always reachable at 0x48 (FT6336U is at 0x38).
fn detail_se050(lcd: &mut Lcd<'_>, i2c: &mut I2c<'_, Blocking>, just_entered: bool) {
    if !just_entered { return; }
    body_clear(lcd);
    line(lcd, 10, 60, Rgb565::WHITE,          "SE050 secure element");
    line(lcd, 10, 85, Rgb565::WHITE,          "I2C 0x48");

    // Simple ACK probe at 0x48.
    let mut probe = [0u8; 1];
    let ack = i2c.read(SE050_ADDR, &mut probe).is_ok();
    let (msg, c) = if ack {
        ("0x48 ACK (SE050)", Rgb565::new(0, 63, 0))
    } else {
        ("0x48 NO ACK", Rgb565::new(31, 0, 0))
    };
    line(lcd, 10, 120, c, msg);

    // Try reading 16 bytes. SE050 in T=1 will return frame bytes; on idle
    // it may NACK or return 0xFF. Show whatever we got so the user has
    // visual confirmation that the chip is electrically alive.
    let mut buf16 = [0u8; 16];
    let read_ok = i2c.read(SE050_ADDR, &mut buf16).is_ok();
    let mut s = [0u8; 40];
    if read_ok {
        line(lcd, 10, 155, Rgb565::WHITE, "Read 16 bytes:");
        line(lcd, 10, 185, Rgb565::WHITE,
            fmt(&mut s, format_args!("{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                buf16[0], buf16[1], buf16[2], buf16[3],
                buf16[4], buf16[5], buf16[6], buf16[7])));
        line(lcd, 10, 210, Rgb565::WHITE,
            fmt(&mut s, format_args!("{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                buf16[8], buf16[9], buf16[10], buf16[11],
                buf16[12], buf16[13], buf16[14], buf16[15])));
    } else {
        line(lcd, 10, 155, Rgb565::new(28, 40, 0), "16-byte read NACK");
        line(lcd, 10, 180, Rgb565::WHITE, "(probe-only result");
        line(lcd, 10, 200, Rgb565::WHITE, " above is enough)");
    }

    line(lcd, 10, 250, Rgb565::WHITE, "Hold SW2 to leave");
}

// 8) XL9535 expander live port state.
fn detail_xl9535(lcd: &mut Lcd<'_>, p0: u8, p1: u8,
                 tick: u32, just_entered: bool) {
    if just_entered {
        body_clear(lcd);
        line(lcd, 10, 60, Rgb565::WHITE, "U9 XL9535 @ 0x20");
    }
    if !just_entered && tick % 5 != 0 { return; }
    let mut buf = [0u8; 40];
    line(lcd, 10, 95,  Rgb565::WHITE,
        fmt(&mut buf, format_args!("Port0: 0x{:02X}", p0)));
    line(lcd, 10, 120, Rgb565::WHITE,
        fmt(&mut buf, format_args!("Port1: 0x{:02X}", p1)));
    let labels0 = ["BL", "FT", "CM", "SE", "S2", "B1", "B2", "P1"];
    let labels1 = ["P6", "S3", "S1", "P5", "P4", "P3", "P2", "LD"];
    // Clear bit-strip area
    lcd.fill_area(0, 145, 239, 245, Rgb565::BLACK);
    for i in 0..8 {
        let x = 10 + i as i32 * 28;
        let bit = (p0 >> i) & 1;
        let c = if bit == 1 { Rgb565::new(0, 63, 0) } else { Rgb565::new(15, 15, 15) };
        Text::new(labels0[i], Point::new(x, 160),
            MonoTextStyle::new(&FONT_9X18_BOLD, c)).draw(lcd).ok();
        let s = fmt(&mut buf, format_args!("{}", bit));
        Text::new(s, Point::new(x + 6, 180),
            MonoTextStyle::new(&FONT_9X18_BOLD, c)).draw(lcd).ok();
    }
    for i in 0..8 {
        let x = 10 + i as i32 * 28;
        let bit = (p1 >> i) & 1;
        let c = if bit == 1 { Rgb565::new(0, 63, 0) } else { Rgb565::new(15, 15, 15) };
        Text::new(labels1[i], Point::new(x, 215),
            MonoTextStyle::new(&FONT_9X18_BOLD, c)).draw(lcd).ok();
        let s = fmt(&mut buf, format_args!("{}", bit));
        Text::new(s, Point::new(x + 6, 235),
            MonoTextStyle::new(&FONT_9X18_BOLD, c)).draw(lcd).ok();
    }
}

// 9) Live I2C bus rescan.
fn detail_i2c_scan(lcd: &mut Lcd<'_>, i2c: &mut I2c<'_, Blocking>, just_entered: bool) {
    if !just_entered { return; }
    body_clear(lcd);
    line(lcd, 10, 60, Rgb565::WHITE, "Scanning 0x08..0x77");
    let mut found = [0u8; 16];
    let mut n = 0usize;
    for addr in 0x08u8..=0x77u8 {
        let mut b = [0u8; 1];
        if i2c.read(addr, &mut b).is_ok() && n < found.len() {
            found[n] = addr;
            n += 1;
        }
    }
    let mut buf = [0u8; 40];
    line(lcd, 10, 90, Rgb565::new(0, 63, 0),
        fmt(&mut buf, format_args!("{} device(s) found", n)));
    for i in 0..n {
        let y = 120 + (i as i32 / 2) * 24;
        let x = if i % 2 == 0 { 15 } else { 130 };
        let label = match found[i] {
            0x11 => "ES7243E",
            0x19 => "SC7A20",
            0x20 => "XL9535",
            0x38 => "AHT20",
            0x48 => "TSC/SE",
            _    => "????",
        };
        let s = fmt(&mut buf, format_args!("0x{:02X} {}", found[i], label));
        Text::new(s, Point::new(x, y),
            MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE)).draw(lcd).ok();
    }
}

// 10) System info (static).
fn detail_sysinfo(lcd: &mut Lcd<'_>, just_entered: bool) {
    if !just_entered { return; }
    body_clear(lcd);
    line(lcd, 10, 70,  Rgb565::WHITE, "ESP32-S3-WROOM-1");
    line(lcd, 10, 95,  Rgb565::WHITE, "  N16R8");
    line(lcd, 10, 130, Rgb565::WHITE, "Flash : 16 MB");
    line(lcd, 10, 155, Rgb565::WHITE, "PSRAM : 8 MB");
    line(lcd, 10, 185, Rgb565::WHITE, "LCD   : ILI9341");
    line(lcd, 10, 210, Rgb565::WHITE, "        240x320");
    line(lcd, 10, 240, Rgb565::WHITE, "I2C   : 100 kHz");
    line(lcd, 10, 265, Rgb565::WHITE, "SPI   : 8 MHz");
}

// 11) Camera: live OV2640 RGB565 preview. Center-crops 320x240 -> 240x240
//     and blits into y range 40..280 on the portrait 240x320 LCD.
fn detail_camera(
    lcd: &mut Lcd<'_>,
    i2c: &mut I2c<'_, Blocking>,
    state: &mut CamState,
    just_entered: bool,
) {
    use camera::{Probe, FRAME_BYTES, FRAME_W};

    if just_entered {
        body_clear(lcd);
        line(lcd, 10, 60, Rgb565::WHITE, "GC2145 DVP capture");

        if state.camera.is_none() || state.buf.is_none() {
            state.last_status = CamStatus::NoPeripheral;
            line(lcd, 10, 100, Rgb565::new(31, 0, 0), "LCD_CAM not ready");
            line(lcd, 10, 125, Rgb565::WHITE, "(see boot log)");
            return;
        }

        if !state.sensor_inited {
            // Show a probe / init progress message (single shot).
            match camera::probe(i2c) {
                Probe::NotFound => {
                    state.last_status = CamStatus::SensorNotFound;
                    line(lcd, 10, 100, Rgb565::new(31, 0, 0), "GC2145 NOT FOUND");
                    line(lcd, 10, 125, Rgb565::WHITE, "Check FPC3 cable");
                    line(lcd, 10, 150, Rgb565::WHITE, "and 0x3C ACK on bus");
                    return;
                }
                Probe::BusError => {
                    state.last_status = CamStatus::SensorBusError;
                    line(lcd, 10, 100, Rgb565::new(31, 0, 0), "I2C bus error");
                    return;
                }
                Probe::Found { id_h, id_l } => {
                    let mut buf = [0u8; 40];
                    line(lcd, 10, 100, Rgb565::new(0, 63, 0),
                        fmt(&mut buf, format_args!("ID {:02X}{:02X}", id_h, id_l)));
                    line(lcd, 10, 130, Rgb565::WHITE, "Initialising...");
                    if camera::init_gc2145_rgb565(i2c).is_err() {
                        state.last_status = CamStatus::InitFailed;
                        line(lcd, 10, 160, Rgb565::new(31, 0, 0), "SCCB init failed");
                        return;
                    }
                    state.sensor_inited = true;
                    state.last_status = CamStatus::Streaming;
                    // Allow GC2145 AEC/AWB to settle before streaming.
                    let mut delay = Delay::new();
                    delay.delay_ms(200);
                    line(lcd, 10, 160, Rgb565::WHITE, "Streaming...");
                }
            }
        }

        // Clear the preview window border once.
        lcd.fill_area(0, 40, 239, 279, Rgb565::BLACK);
    }

    // Streaming path — only run when sensor is up.
    if state.last_status != CamStatus::Streaming {
        return;
    }

    // Capture one frame: hand the buffer to DMA, poll until the LCD_CAM auto-
    // stops (cam_stop_en triggers at end-of-frame). While polling, read SW2 on
    // the XL9535 — if the user holds it down we abort the capture so the main
    // loop can exit the camera screen without waiting for the next VSYNC.
    let cam = match state.camera.take() {
        Some(c) => c,
        None => return,
    };
    let buf = match state.buf.take() {
        Some(b) => b,
        None => {
            state.camera = Some(cam);
            return;
        }
    };

    let (cam, buf_back) = match cam.receive(buf) {
        Ok(transfer) => {
            // Poll for completion. ~150 ms ceiling so even a stalled sensor
            // releases control back to the main loop within one button check.
            const SW2_MASK_P0: u8 = 1 << 4;
            let mut iters: u32 = 0;
            let mut abort = false;
            while !transfer.is_done() {
                iters = iters.wrapping_add(1);
                // Every ~5 ms (rough) glance at SW2 to honour hold-back.
                if iters % 4000 == 0 {
                    let mut r0 = [0u8; 1];
                    if i2c.write_read(XL_ADDR, &[XL_INPUT_0], &mut r0).is_ok()
                        && (r0[0] & SW2_MASK_P0) == 0
                    {
                        abort = true;
                        break;
                    }
                }
                if iters > 80_000 { abort = true; break; } // ~150 ms safety
            }
            if abort {
                let (cam, finalbuf) = transfer.stop();
                (cam, finalbuf)
            } else {
                let (_res, cam, finalbuf) = transfer.wait();
                (cam, finalbuf)
            }
        }
        Err((_e, cam, b)) => (cam, b),
    };

    // Pull bytes out and blit center 240x240 into LCD y=40..280.
    let bytes = buf_back.as_slice();
    if bytes.len() >= FRAME_BYTES {
        const CROP_X: u16 = (FRAME_W - 240) / 2; // = 40
        // For each of the 240 displayed rows, push that row's 240 pixels.
        for row in 0..240u16 {
            let row_start = (row as usize) * (FRAME_W as usize) * 2
                + (CROP_X as usize) * 2;
            let row_end = row_start + 240 * 2;
            lcd.blit_rgb565(0, 40 + row, 239, 40 + row, &bytes[row_start..row_end]);
        }
    }

    state.camera = Some(cam);
    state.buf = Some(buf_back);
}
