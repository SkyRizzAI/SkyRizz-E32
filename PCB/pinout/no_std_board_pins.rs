//! SkyRizz E32 ESP32-S3 board pin constants for `no_std` `esp-hal` / Embassy projects.
//!
//! Target module: `U1` / `ESP32-S3-WROOM-1-N16R8`.
//! Derived from the EasyEDA project `skyrizz_e32_se050.epro`.
//! The EasyEDA symbol title is `ESP32-S3-WROOM-1(N16R8)`, while the actual
//! device / BOM part metadata resolves to `ESP32-S3-WROOM-1-N16R8`.
//! Important board-level notes:
//! - `GPIO43` / `U0TXD` is wired to the XL9535 `INT#` line.
//! - `GPIO44` / `U0RXD` is reused as the shared SPI clock for TF + GT30L24A3W.
//! - `GPIO0` is both the BOOT strap and the shared audio `BCLK` line.
//! - GT30L24A3W `CS#` is inverted from the TF `CS` signal through `Q5`.
//! - PCB external headers are silked as `IO 1` (`C_P0`), `I2C` (`C_I2C`),
//!   `IO 2` (`C_P1-3`), and `IO 3` (`C_P4-7`).

#![allow(dead_code)]

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct I2cBusPins {
    pub sda: u8,
    pub scl: u8,
    pub int_n: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ExternalIo1Pins {
    pub p0: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ExternalI2cHeaderPins {
    pub sda: u8,
    pub scl: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RgbLedPins {
    pub data: u8,
    pub led_count: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UserLedPins {
    pub drive_iox: Xl9535Pin,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LocalInputPins {
    pub sw1: Xl9535Pin,
    pub sw2: Xl9535Pin,
    pub sw3: Xl9535Pin,
    pub pb1: Xl9535Pin,
    pub pb2: Xl9535Pin,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CameraPins {
    pub vsync: u8,
    pub href: u8,
    pub xclk: u8,
    pub pclk: u8,
    pub d2: u8,
    pub d3: u8,
    pub d4: u8,
    pub d5: u8,
    pub d6: u8,
    pub d7: u8,
    pub d8: u8,
    pub d9: u8,
    pub siod: u8,
    pub sioc: u8,
    pub reset_iox: Xl9535Pin,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LcdPins {
    pub sclk: u8,
    pub mosi: u8,
    pub cs: u8,
    pub dc: u8,
    pub backlight_iox: Xl9535Pin,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TouchPins {
    pub irq: u8,
    pub sda: u8,
    pub scl: u8,
    pub reset_iox: Xl9535Pin,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AudioPins {
    pub mclk: u8,
    pub bclk: u8,
    pub lrck: u8,
    pub dout: u8,
    pub din: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Spi3Pins {
    pub tf_cs: u8,
    pub miso: u8,
    pub mosi: u8,
    pub sclk: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ExternalIo2Pins {
    pub p1: Xl9535Pin,
    pub p2: Xl9535Pin,
    pub p3: Xl9535Pin,
    pub sda: u8,
    pub scl: u8,
    pub int_n: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ExternalIo3Pins {
    pub p4: Xl9535Pin,
    pub p5: Xl9535Pin,
    pub p6: Xl9535Pin,
    pub p7: Xl9535Pin,
    pub sda: u8,
    pub scl: u8,
    pub int_n: u8,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Xl9535Pin {
    LcdBlk = 0,
    TsRst = 1,
    CamRst = 2,
    SeRst = 3,
    P9 = 4,
    P10 = 5,
    P11 = 6,
    P1 = 7,
    P6 = 10,
    P7 = 11,
    P8 = 12,
    P5 = 13,
    P4 = 14,
    P3 = 15,
    P2 = 16,
    UserLed = 17,
}

pub const PIN_USB_DN: u8 = 19;
pub const PIN_USB_DP: u8 = 20;

pub const PIN_I2C_SDA: u8 = 47;
pub const PIN_I2C_SCL: u8 = 48;
pub const PIN_XL9535_INT: u8 = 43;

pub const PIN_TOUCH_INT: u8 = 2;
pub const PIN_RGB_LED: u8 = 46;
pub const RGB_LED_COUNT: u8 = 2;
pub const PIN_EXT_P0: u8 = 1;

pub const PIN_LCD_SCLK: u8 = 12;
pub const PIN_LCD_MOSI: u8 = 21;
pub const PIN_LCD_CS: u8 = 14;
pub const PIN_LCD_DC: u8 = 13;

pub const PIN_CAM_VSYNC: u8 = 4;
pub const PIN_CAM_HREF: u8 = 5;
pub const PIN_CAM_XCLK: u8 = 7;
pub const PIN_CAM_PCLK: u8 = 17;
pub const PIN_CAM_D2: u8 = 8;
pub const PIN_CAM_D3: u8 = 10;
pub const PIN_CAM_D4: u8 = 11;
pub const PIN_CAM_D5: u8 = 9;
pub const PIN_CAM_D6: u8 = 18;
pub const PIN_CAM_D7: u8 = 16;
pub const PIN_CAM_D8: u8 = 15;
pub const PIN_CAM_D9: u8 = 6;

pub const PIN_AUDIO_MCLK: u8 = 3;
pub const PIN_AUDIO_BCLK: u8 = 0;
pub const PIN_AUDIO_LRCK: u8 = 38;
pub const PIN_AUDIO_DOUT: u8 = 45;
pub const PIN_AUDIO_DIN: u8 = 39;

pub const PIN_TF_CS: u8 = 40;
pub const PIN_SPI3_MISO: u8 = 41;
pub const PIN_SPI3_MOSI: u8 = 42;
pub const PIN_SPI3_SCLK: u8 = 44;

pub const PIN_UNUSED_35: u8 = 35;
pub const PIN_UNUSED_36: u8 = 36;
pub const PIN_UNUSED_37: u8 = 37;

pub const XL9535_I2C_ADDRESS: u8 = 0x20;
pub const PIN_USER_LED_IOX: Xl9535Pin = Xl9535Pin::UserLed;
pub const PIN_SW1_IOX: Xl9535Pin = Xl9535Pin::P8;
pub const PIN_SW2_IOX: Xl9535Pin = Xl9535Pin::P9;
pub const PIN_SW3_IOX: Xl9535Pin = Xl9535Pin::P7;
pub const PIN_PB1_IOX: Xl9535Pin = Xl9535Pin::P10;
pub const PIN_PB2_IOX: Xl9535Pin = Xl9535Pin::P11;

pub const I2C_BUS: I2cBusPins = I2cBusPins {
    sda: PIN_I2C_SDA,
    scl: PIN_I2C_SCL,
    int_n: PIN_XL9535_INT,
};

/// PCB silk: `IO 1` / connector `C_P0`
pub const EXT_IO1: ExternalIo1Pins = ExternalIo1Pins { p0: PIN_EXT_P0 };

/// PCB silk: `I2C` / connector `C_I2C`
pub const EXT_I2C: ExternalI2cHeaderPins = ExternalI2cHeaderPins {
    sda: PIN_I2C_SDA,
    scl: PIN_I2C_SCL,
};

pub const RGB_LEDS: RgbLedPins = RgbLedPins {
    data: PIN_RGB_LED,
    led_count: RGB_LED_COUNT,
};

pub const USER_LED: UserLedPins = UserLedPins {
    drive_iox: PIN_USER_LED_IOX,
};

pub const LOCAL_INPUTS: LocalInputPins = LocalInputPins {
    sw1: PIN_SW1_IOX,
    sw2: PIN_SW2_IOX,
    sw3: PIN_SW3_IOX,
    pb1: PIN_PB1_IOX,
    pb2: PIN_PB2_IOX,
};

pub const CAMERA: CameraPins = CameraPins {
    vsync: PIN_CAM_VSYNC,
    href: PIN_CAM_HREF,
    xclk: PIN_CAM_XCLK,
    pclk: PIN_CAM_PCLK,
    d2: PIN_CAM_D2,
    d3: PIN_CAM_D3,
    d4: PIN_CAM_D4,
    d5: PIN_CAM_D5,
    d6: PIN_CAM_D6,
    d7: PIN_CAM_D7,
    d8: PIN_CAM_D8,
    d9: PIN_CAM_D9,
    siod: PIN_I2C_SDA,
    sioc: PIN_I2C_SCL,
    reset_iox: Xl9535Pin::CamRst,
};

pub const LCD: LcdPins = LcdPins {
    sclk: PIN_LCD_SCLK,
    mosi: PIN_LCD_MOSI,
    cs: PIN_LCD_CS,
    dc: PIN_LCD_DC,
    backlight_iox: Xl9535Pin::LcdBlk,
};

pub const TOUCH: TouchPins = TouchPins {
    irq: PIN_TOUCH_INT,
    sda: PIN_I2C_SDA,
    scl: PIN_I2C_SCL,
    reset_iox: Xl9535Pin::TsRst,
};

pub const AUDIO: AudioPins = AudioPins {
    mclk: PIN_AUDIO_MCLK,
    bclk: PIN_AUDIO_BCLK,
    lrck: PIN_AUDIO_LRCK,
    dout: PIN_AUDIO_DOUT,
    din: PIN_AUDIO_DIN,
};

pub const SPI3: Spi3Pins = Spi3Pins {
    tf_cs: PIN_TF_CS,
    miso: PIN_SPI3_MISO,
    mosi: PIN_SPI3_MOSI,
    sclk: PIN_SPI3_SCLK,
};

/// PCB silk: `IO 2` / connector `C_P1-3`
pub const EXT_IO2: ExternalIo2Pins = ExternalIo2Pins {
    p1: Xl9535Pin::P1,
    p2: Xl9535Pin::P2,
    p3: Xl9535Pin::P3,
    sda: PIN_I2C_SDA,
    scl: PIN_I2C_SCL,
    int_n: PIN_XL9535_INT,
};

/// PCB silk: `IO 3` / connector `C_P4-7`
pub const EXT_IO3: ExternalIo3Pins = ExternalIo3Pins {
    p4: Xl9535Pin::P4,
    p5: Xl9535Pin::P5,
    p6: Xl9535Pin::P6,
    p7: Xl9535Pin::P7,
    sda: PIN_I2C_SDA,
    scl: PIN_I2C_SCL,
    int_n: PIN_XL9535_INT,
};

pub const UNUSED_ONBOARD: [u8; 3] = [PIN_UNUSED_35, PIN_UNUSED_36, PIN_UNUSED_37];
