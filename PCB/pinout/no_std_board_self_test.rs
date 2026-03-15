//! SkyRizz E32 ESP32-S3 board self-test harness for `no_std` firmware.
//!
//! Keep this file next to `no_std_board_pins.rs` and include both files in an
//! ESP32-S3 project. It is intentionally HAL-agnostic: the test flow is fixed,
//! while the concrete GPIO/I2C/SPI/display/audio/camera operations are supplied
//! by your `Platform` implementation.
//!
//! Coverage by default:
//! - `GPIO47` / `GPIO48` and the `I2C` header via shared-bus device probes
//! - `GPIO43` via the XL9535 interrupt test
//! - `GPIO1` and the external `IO 2` / `IO 3` headers via jumper loopback
//! - `GPIO2` via the touch IRQ test
//! - `GPIO12` / `GPIO13` / `GPIO14` / `GPIO21` plus XL9535 `LCD_BLK` via LCD
//! - `GPIO3` / `GPIO0` / `GPIO38` / `GPIO39` / `GPIO45` plus ES7243E via audio
//! - `GPIO4` / `GPIO5` / `GPIO6` / `GPIO7` / `GPIO8` / `GPIO9` / `GPIO10`
//!   / `GPIO11` / `GPIO15` / `GPIO16` / `GPIO17` / `GPIO18` plus XL9535
//!   `CAM_RST` via the optional camera test
//! - `GPIO46` via the RGB LED test
//! - XL9535 local nets `P8` / `P9` / `P10` / `P11` / `P7` via switch/button
//!   tests, and `U_LED` via the user LED test
//! - `GPIO19` / `GPIO20` via a manual USB-C enumeration check
//! - Optional pogo-fixture coverage for unrouted `GPIO35` / `GPIO36` / `GPIO37`
//!
//! The existing `skyclaw/` crate in this repository targets ESP32-C3, so this
//! harness is provided as a reusable S3-side module instead of being wired into
//! that crate directly.

#![allow(dead_code)]

use core::fmt;

#[path = "no_std_board_pins.rs"]
pub mod board;

const SIGNAL_SETTLE_MS: u32 = 2;
const VISUAL_SETTLE_MS: u32 = 150;
const AUDIO_TONE_HZ: u16 = 1_000;
const AUDIO_TONE_MS: u16 = 300;
const MIC_SAMPLE_MS: u16 = 400;

pub const REQUIRED_I2C_DEVICES: [I2cDevice; 5] = [
    I2cDevice::Xl9535,
    I2cDevice::Aht20,
    I2cDevice::Ltr303als,
    I2cDevice::Sc7a20,
    I2cDevice::Tsc2007,
];

pub const EXTERNAL_LOOPBACK_TARGETS: [LoopbackTarget; 7] = [
    LoopbackTarget::ExtP1,
    LoopbackTarget::ExtP2,
    LoopbackTarget::ExtP3,
    LoopbackTarget::ExtP4,
    LoopbackTarget::ExtP5,
    LoopbackTarget::ExtP6,
    LoopbackTarget::ExtP7,
];

pub const UNROUTED_LOOPBACK_TARGETS: [LoopbackTarget; 3] = [
    LoopbackTarget::UnusedGpio35,
    LoopbackTarget::UnusedGpio36,
    LoopbackTarget::UnusedGpio37,
];

pub const LOCAL_INPUT_TARGETS: [LocalInputTarget; 5] = [
    LocalInputTarget::Sw1,
    LocalInputTarget::Sw2,
    LocalInputTarget::Pb1,
    LocalInputTarget::Pb2,
    LocalInputTarget::Sw3,
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Pull {
    None,
    Up,
    Down,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum I2cDevice {
    Xl9535,
    Aht20,
    Ltr303als,
    Sc7a20,
    Tsc2007,
    CameraSensor,
    Es7243e,
    Se050,
}

impl I2cDevice {
    pub const fn case_id(self) -> &'static str {
        match self {
            Self::Xl9535 => "i2c-xl9535",
            Self::Aht20 => "i2c-aht20",
            Self::Ltr303als => "i2c-ltr303als",
            Self::Sc7a20 => "i2c-sc7a20",
            Self::Tsc2007 => "i2c-tsc2007",
            Self::CameraSensor => "i2c-camera",
            Self::Es7243e => "i2c-es7243e",
            Self::Se050 => "i2c-se050",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Xl9535 => "XL9535",
            Self::Aht20 => "AHT20",
            Self::Ltr303als => "LTR-303ALS",
            Self::Sc7a20 => "SC7A20",
            Self::Tsc2007 => "TSC2007",
            Self::CameraSensor => "camera sensor",
            Self::Es7243e => "ES7243E",
            Self::Se050 => "SE050",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Xl9535 => "Probe the XL9535 I/O expander on the shared I2C bus",
            Self::Aht20 => "Probe the AHT20 temperature / humidity sensor",
            Self::Ltr303als => "Probe the LTR-303ALS ambient-light sensor",
            Self::Sc7a20 => "Probe the SC7A20 accelerometer",
            Self::Tsc2007 => "Probe the TSC2007 touch controller",
            Self::CameraSensor => "Probe the camera sensor over SCCB / I2C",
            Self::Es7243e => "Probe the ES7243E microphone ADC / codec",
            Self::Se050 => "Probe the SE050 secure element",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpiDevice {
    TfCard,
    Gt30l24a3w,
}

impl SpiDevice {
    pub const fn case_id(self) -> &'static str {
        match self {
            Self::TfCard => "spi-tf-card",
            Self::Gt30l24a3w => "spi-gt30",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::TfCard => "TF card",
            Self::Gt30l24a3w => "GT30L24A3W",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::TfCard => "Probe the TF card slot on the shared SPI3 bus",
            Self::Gt30l24a3w => "Probe the GT30L24A3W device on the shared SPI3 bus",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LocalInputTarget {
    Sw1,
    Sw2,
    Pb1,
    Pb2,
    Sw3,
}

impl LocalInputTarget {
    pub const fn case_id(self) -> &'static str {
        match self {
            Self::Sw1 => "local-sw1",
            Self::Sw2 => "local-sw2",
            Self::Pb1 => "local-pb1",
            Self::Pb2 => "local-pb2",
            Self::Sw3 => "local-sw3",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Sw1 => "SW1",
            Self::Sw2 => "SW2",
            Self::Pb1 => "PB1",
            Self::Pb2 => "PB2",
            Self::Sw3 => "SW3",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Sw1 => "Verify XL9535 local input P8 by pressing SW1",
            Self::Sw2 => "Verify XL9535 local input P9 by pressing SW2",
            Self::Pb1 => "Verify XL9535 local input P10 by pressing PB1",
            Self::Pb2 => "Verify XL9535 local input P11 by pressing PB2",
            Self::Sw3 => "Verify XL9535 local input / external shared pin P7 by pressing SW3",
        }
    }

    pub const fn prompt(self) -> &'static str {
        match self {
            Self::Sw1 => "Press and hold SW1 now, then answer yes.",
            Self::Sw2 => "Press and hold SW2 now, then answer yes.",
            Self::Pb1 => "Press and hold PB1 now, then answer yes.",
            Self::Pb2 => "Press and hold PB2 now, then answer yes.",
            Self::Sw3 => "Press and hold SW3 now, then answer yes.",
        }
    }

    pub const fn expander_pin(self) -> board::Xl9535Pin {
        match self {
            Self::Sw1 => board::Xl9535Pin::P8,
            Self::Sw2 => board::Xl9535Pin::P9,
            Self::Pb1 => board::Xl9535Pin::P10,
            Self::Pb2 => board::Xl9535Pin::P11,
            Self::Sw3 => board::Xl9535Pin::P7,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LoopbackTarget {
    ExtP1,
    ExtP2,
    ExtP3,
    ExtP4,
    ExtP5,
    ExtP6,
    ExtP7,
    UnusedGpio35,
    UnusedGpio36,
    UnusedGpio37,
}

impl LoopbackTarget {
    pub const fn case_id(self) -> &'static str {
        match self {
            Self::ExtP1 => "loopback-ext-p1",
            Self::ExtP2 => "loopback-ext-p2",
            Self::ExtP3 => "loopback-ext-p3",
            Self::ExtP4 => "loopback-ext-p4",
            Self::ExtP5 => "loopback-ext-p5",
            Self::ExtP6 => "loopback-ext-p6",
            Self::ExtP7 => "loopback-ext-p7",
            Self::UnusedGpio35 => "loopback-gpio35",
            Self::UnusedGpio36 => "loopback-gpio36",
            Self::UnusedGpio37 => "loopback-gpio37",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::ExtP1 => "IO 2.P1",
            Self::ExtP2 => "IO 2.P2",
            Self::ExtP3 => "IO 2.P3",
            Self::ExtP4 => "IO 3.P4",
            Self::ExtP5 => "IO 3.P5",
            Self::ExtP6 => "IO 3.P6",
            Self::ExtP7 => "IO 3.P7",
            Self::UnusedGpio35 => "GPIO35 test pad",
            Self::UnusedGpio36 => "GPIO36 test pad",
            Self::UnusedGpio37 => "GPIO37 test pad",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::ExtP1 => "Verify direct P0 to external P1 loopback through XL9535",
            Self::ExtP2 => "Verify direct P0 to external P2 loopback through XL9535",
            Self::ExtP3 => "Verify direct P0 to external P3 loopback through XL9535",
            Self::ExtP4 => "Verify direct P0 to external P4 loopback through XL9535",
            Self::ExtP5 => "Verify direct P0 to external P5 loopback through XL9535",
            Self::ExtP6 => "Verify direct P0 to external P6 loopback through XL9535",
            Self::ExtP7 => "Verify direct P0 to external P7 loopback through XL9535",
            Self::UnusedGpio35 => "Verify optional pogo-fixture access to unrouted GPIO35",
            Self::UnusedGpio36 => "Verify optional pogo-fixture access to unrouted GPIO36",
            Self::UnusedGpio37 => "Verify optional pogo-fixture access to unrouted GPIO37",
        }
    }

    pub const fn jumper_prompt(self) -> &'static str {
        match self {
            Self::ExtP1 => "Install a jumper between IO 1.P0 and IO 2.P1, then answer yes.",
            Self::ExtP2 => "Move the jumper between IO 1.P0 and IO 2.P2, then answer yes.",
            Self::ExtP3 => "Move the jumper between IO 1.P0 and IO 2.P3, then answer yes.",
            Self::ExtP4 => "Move the jumper between IO 1.P0 and IO 3.P4, then answer yes.",
            Self::ExtP5 => "Move the jumper between IO 1.P0 and IO 3.P5, then answer yes.",
            Self::ExtP6 => "Move the jumper between IO 1.P0 and IO 3.P6, then answer yes.",
            Self::ExtP7 => "Move the jumper between IO 1.P0 and IO 3.P7, then answer yes.",
            Self::UnusedGpio35 => "Bridge IO 1.P0 to GPIO35 with a pogo / solder test lead, then answer yes.",
            Self::UnusedGpio36 => "Bridge IO 1.P0 to GPIO36 with a pogo / solder test lead, then answer yes.",
            Self::UnusedGpio37 => "Bridge IO 1.P0 to GPIO37 with a pogo / solder test lead, then answer yes.",
        }
    }

    const fn endpoint(self) -> LoopbackEndpoint {
        match self {
            Self::ExtP1 => LoopbackEndpoint::Expander(board::Xl9535Pin::P1),
            Self::ExtP2 => LoopbackEndpoint::Expander(board::Xl9535Pin::P2),
            Self::ExtP3 => LoopbackEndpoint::Expander(board::Xl9535Pin::P3),
            Self::ExtP4 => LoopbackEndpoint::Expander(board::Xl9535Pin::P4),
            Self::ExtP5 => LoopbackEndpoint::Expander(board::Xl9535Pin::P5),
            Self::ExtP6 => LoopbackEndpoint::Expander(board::Xl9535Pin::P6),
            Self::ExtP7 => LoopbackEndpoint::Expander(board::Xl9535Pin::P7),
            Self::UnusedGpio35 => LoopbackEndpoint::Direct(board::PIN_UNUSED_35),
            Self::UnusedGpio36 => LoopbackEndpoint::Direct(board::PIN_UNUSED_36),
            Self::UnusedGpio37 => LoopbackEndpoint::Direct(board::PIN_UNUSED_37),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum LoopbackEndpoint {
    Expander(board::Xl9535Pin),
    Direct(u8),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CameraFrameStats {
    pub width: u16,
    pub height: u16,
    pub non_zero_bytes: u32,
}

impl CameraFrameStats {
    pub const fn looks_valid(self) -> bool {
        self.width > 0 && self.height > 0 && self.non_zero_bytes > 0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CaseOutcome {
    Passed,
    Failed,
    Skipped,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct TestSummary {
    pub passed: u16,
    pub failed: u16,
    pub skipped: u16,
}

impl TestSummary {
    pub fn record(&mut self, outcome: CaseOutcome) {
        match outcome {
            CaseOutcome::Passed => self.passed += 1,
            CaseOutcome::Failed => self.failed += 1,
            CaseOutcome::Skipped => self.skipped += 1,
        }
    }

    pub const fn is_ok(self) -> bool {
        self.failed == 0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SelfTestOptions {
    pub require_operator_confirmations: bool,
    pub require_local_button_checks: bool,
    pub require_external_loopback: bool,
    pub require_unrouted_probe_fixture: bool,
    pub require_camera_sensor: bool,
    pub require_audio_codec: bool,
    pub require_secure_element: bool,
    pub require_tf_card: bool,
    pub require_gt30: bool,
    pub require_usb_enumeration: bool,
    pub microphone_peak_threshold: u16,
}

impl Default for SelfTestOptions {
    fn default() -> Self {
        Self {
            require_operator_confirmations: true,
            require_local_button_checks: true,
            require_external_loopback: true,
            require_unrouted_probe_fixture: false,
            require_camera_sensor: false,
            require_audio_codec: true,
            require_secure_element: true,
            require_tf_card: false,
            require_gt30: true,
            require_usb_enumeration: true,
            microphone_peak_threshold: 32,
        }
    }
}

pub trait Reporter {
    fn begin_case(&mut self, id: &'static str, description: &'static str);

    fn note(&mut self, message: &'static str);

    fn note_fmt(&mut self, args: fmt::Arguments<'_>) {
        let _ = args;
    }

    fn end_case(&mut self, outcome: CaseOutcome);

    fn confirm(&mut self, prompt: &'static str) -> bool;
}

pub trait Platform {
    type Error;

    fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error>;

    /// Probe the named board device. Implementations may perform reset
    /// sequencing or other board-specific setup before checking presence.
    fn probe_i2c_device(&mut self, device: I2cDevice) -> Result<bool, Self::Error>;

    /// Probe the named SPI-side device on the board.
    fn probe_spi_device(&mut self, device: SpiDevice) -> Result<bool, Self::Error>;

    fn set_user_led(&mut self, on: bool) -> Result<(), Self::Error>;
    fn set_rgb(&mut self, r: u8, g: u8, b: u8) -> Result<(), Self::Error>;

    /// This should explicitly enable the LCD backlight through the expander.
    fn lcd_show_test_pattern(&mut self) -> Result<(), Self::Error>;

    /// Return `true` when the touch IRQ line on `GPIO2` is asserted.
    fn touch_interrupt_asserted(&mut self) -> Result<bool, Self::Error>;
    fn touch_read(&mut self) -> Result<Option<TouchPoint>, Self::Error>;

    fn play_test_tone(&mut self, hz: u16, duration_ms: u16) -> Result<(), Self::Error>;

    /// Return a normalized peak amplitude over the sampling window.
    fn microphone_peak(&mut self, sample_window_ms: u16) -> Result<u16, Self::Error>;

    fn capture_camera_frame(&mut self) -> Result<CameraFrameStats, Self::Error>;

    fn configure_direct_input(&mut self, pin: u8, pull: Pull) -> Result<(), Self::Error>;
    fn configure_direct_output(&mut self, pin: u8, initial_high: bool) -> Result<(), Self::Error>;
    fn write_direct(&mut self, pin: u8, high: bool) -> Result<(), Self::Error>;
    fn read_direct(&mut self, pin: u8) -> Result<bool, Self::Error>;

    fn configure_xl9535_input(&mut self, pin: board::Xl9535Pin) -> Result<(), Self::Error>;
    fn configure_xl9535_output(
        &mut self,
        pin: board::Xl9535Pin,
        initial_high: bool,
    ) -> Result<(), Self::Error>;
    fn write_xl9535(&mut self, pin: board::Xl9535Pin, high: bool) -> Result<(), Self::Error>;
    fn read_xl9535(&mut self, pin: board::Xl9535Pin) -> Result<bool, Self::Error>;

    /// Clear any pending XL9535 interrupt source in the platform implementation.
    fn clear_xl9535_interrupt(&mut self) -> Result<(), Self::Error>;

    /// Return `true` when the XL9535 interrupt line on `GPIO43` is asserted.
    fn xl9535_interrupt_asserted(&mut self) -> Result<bool, Self::Error>;
}

pub fn run_default<P, R>(platform: &mut P, reporter: &mut R) -> TestSummary
where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    run_with_options(platform, reporter, SelfTestOptions::default())
}

pub fn run_with_options<P, R>(
    platform: &mut P,
    reporter: &mut R,
    options: SelfTestOptions,
) -> TestSummary
where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    let mut summary = TestSummary::default();

    reporter.note("SkyRizz E32 self-test starting.");
    reporter.note("Successful shared-bus probes also validate the external I2C header because `C_I2C` is a direct breakout of GPIO47 / GPIO48.");

    for device in REQUIRED_I2C_DEVICES {
        run_i2c_probe_case(&mut summary, platform, reporter, device);
    }

    run_optional_i2c_probe_case(
        &mut summary,
        platform,
        reporter,
        I2cDevice::Es7243e,
        options.require_audio_codec,
        "Audio codec probe disabled in options.",
    );
    run_optional_i2c_probe_case(
        &mut summary,
        platform,
        reporter,
        I2cDevice::Se050,
        options.require_secure_element,
        "Secure-element probe disabled in options.",
    );
    run_optional_i2c_probe_case(
        &mut summary,
        platform,
        reporter,
        I2cDevice::CameraSensor,
        options.require_camera_sensor,
        "Camera sensor probe disabled in options.",
    );

    run_user_led_case(&mut summary, platform, reporter, options);
    run_rgb_case(&mut summary, platform, reporter, options);
    run_lcd_case(&mut summary, platform, reporter, options);
    run_touch_case(&mut summary, platform, reporter, options);

    for target in LOCAL_INPUT_TARGETS {
        run_local_input_case(&mut summary, platform, reporter, target, options);
    }

    run_speaker_case(&mut summary, platform, reporter, options);
    run_microphone_case(&mut summary, platform, reporter, options);

    run_optional_spi_probe_case(
        &mut summary,
        platform,
        reporter,
        SpiDevice::Gt30l24a3w,
        options.require_gt30,
        "GT30 probe disabled in options.",
    );
    run_optional_spi_probe_case(
        &mut summary,
        platform,
        reporter,
        SpiDevice::TfCard,
        options.require_tf_card,
        "TF card probe disabled in options.",
    );

    run_camera_capture_case(&mut summary, platform, reporter, options);
    run_usb_manual_case(&mut summary, reporter, options);
    run_xl9535_interrupt_case(&mut summary, platform, reporter, options);

    for target in EXTERNAL_LOOPBACK_TARGETS {
        run_loopback_case(&mut summary, platform, reporter, target, options);
    }

    for target in UNROUTED_LOOPBACK_TARGETS {
        run_loopback_case(&mut summary, platform, reporter, target, options);
    }

    summary
}

fn run_i2c_probe_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    device: I2cDevice,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    reporter.begin_case(device.case_id(), device.description());
    let outcome = match platform.probe_i2c_device(device) {
        Ok(true) => CaseOutcome::Passed,
        Ok(false) => {
            reporter.note("Device did not acknowledge on the expected board path.");
            CaseOutcome::Failed
        }
        Err(err) => {
            reporter.note_fmt(format_args!("Probe error for {}: {:?}", device.label(), err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_optional_i2c_probe_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    device: I2cDevice,
    enabled: bool,
    skip_message: &'static str,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    if enabled {
        run_i2c_probe_case(summary, platform, reporter, device);
    } else {
        skip_case(summary, reporter, device.case_id(), device.description(), skip_message);
    }
}

fn run_optional_spi_probe_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    device: SpiDevice,
    enabled: bool,
    skip_message: &'static str,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    if !enabled {
        skip_case(summary, reporter, device.case_id(), device.description(), skip_message);
        return;
    }

    reporter.begin_case(device.case_id(), device.description());
    let outcome = match platform.probe_spi_device(device) {
        Ok(true) => CaseOutcome::Passed,
        Ok(false) => {
            reporter.note("SPI-side device did not respond on the expected board wiring.");
            CaseOutcome::Failed
        }
        Err(err) => {
            reporter.note_fmt(format_args!("Probe error for {}: {:?}", device.label(), err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_user_led_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    reporter.begin_case("user-led", "Drive the XL9535 user / indicator LED");
    let outcome = match platform.set_user_led(true) {
        Ok(()) => match platform.delay_ms(VISUAL_SETTLE_MS) {
            Ok(()) => match platform.set_user_led(false) {
                Ok(()) => {
                    if require_confirm(
                        reporter,
                        options,
                        "Did the user / indicator LED turn on and then off?",
                    ) {
                        CaseOutcome::Passed
                    } else {
                        reporter.note("Operator did not confirm the user LED behaviour.");
                        CaseOutcome::Failed
                    }
                }
                Err(err) => {
                    reporter.note_fmt(format_args!("User LED off error: {:?}", err));
                    CaseOutcome::Failed
                }
            },
            Err(err) => {
                reporter.note_fmt(format_args!("Delay error during user LED test: {:?}", err));
                CaseOutcome::Failed
            }
        },
        Err(err) => {
            reporter.note_fmt(format_args!("User LED on error: {:?}", err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_rgb_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    reporter.begin_case("rgb-led", "Cycle the RGB LED on GPIO46");
    let outcome = if perform_rgb_cycle(platform, reporter) {
        if require_confirm(reporter, options, "Did the RGB LED cycle red, green, blue, then off?") {
            CaseOutcome::Passed
        } else {
            reporter.note("Operator did not confirm the RGB LED sequence.");
            CaseOutcome::Failed
        }
    } else {
        CaseOutcome::Failed
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn perform_rgb_cycle<P, R>(platform: &mut P, reporter: &mut R) -> bool
where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    let steps = [(64, 0, 0), (0, 64, 0), (0, 0, 64), (0, 0, 0)];
    for (r, g, b) in steps {
        if let Err(err) = platform.set_rgb(r, g, b) {
            reporter.note_fmt(format_args!("RGB write error: {:?}", err));
            return false;
        }
        if let Err(err) = platform.delay_ms(VISUAL_SETTLE_MS) {
            reporter.note_fmt(format_args!("RGB delay error: {:?}", err));
            return false;
        }
    }
    true
}

fn run_lcd_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    reporter.begin_case("lcd", "Show an LCD test pattern and enable the backlight");
    let outcome = match platform.lcd_show_test_pattern() {
        Ok(()) => {
            if require_confirm(reporter, options, "Did the LCD show the expected pattern with the backlight on?") {
                CaseOutcome::Passed
            } else {
                reporter.note("Operator did not confirm the LCD pattern.");
                CaseOutcome::Failed
            }
        }
        Err(err) => {
            reporter.note_fmt(format_args!("LCD test error: {:?}", err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_touch_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    reporter.begin_case("touch", "Verify touch IRQ on GPIO2 and read a touch sample");
    if options.require_operator_confirmations {
        let _ = reporter.confirm("Touch and hold the panel now, then answer yes.");
    } else {
        reporter.note("Operator confirmations are disabled; touch test will attempt one immediate sample.");
    }

    let outcome = match platform.delay_ms(VISUAL_SETTLE_MS) {
        Ok(()) => match platform.touch_interrupt_asserted() {
            Ok(true) => match platform.touch_read() {
                Ok(Some(point)) => {
                    reporter.note_fmt(format_args!("Touch sample: ({}, {})", point.x, point.y));
                    CaseOutcome::Passed
                }
                Ok(None) => {
                    reporter.note("Touch IRQ asserted but no touch sample was returned.");
                    CaseOutcome::Failed
                }
                Err(err) => {
                    reporter.note_fmt(format_args!("Touch read error: {:?}", err));
                    CaseOutcome::Failed
                }
            },
            Ok(false) => {
                reporter.note("Touch IRQ did not assert while the panel was pressed.");
                CaseOutcome::Failed
            }
            Err(err) => {
                reporter.note_fmt(format_args!("Touch IRQ read error: {:?}", err));
                CaseOutcome::Failed
            }
        },
        Err(err) => {
            reporter.note_fmt(format_args!("Delay error during touch test: {:?}", err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_local_input_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    target: LocalInputTarget,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    if !options.require_local_button_checks {
        skip_case(
            summary,
            reporter,
            target.case_id(),
            target.description(),
            "Local switch / button checks are disabled in options.",
        );
        return;
    }

    if !options.require_operator_confirmations {
        skip_case(
            summary,
            reporter,
            target.case_id(),
            target.description(),
            "Local switch / button checks need an operator confirmation flow.",
        );
        return;
    }

    reporter.begin_case(target.case_id(), target.description());
    let pin = target.expander_pin();
    let outcome = match platform.configure_xl9535_input(pin) {
        Ok(()) => match platform.read_xl9535(pin) {
            Ok(idle) => {
                let _ = reporter.confirm(target.prompt());
                match platform.delay_ms(VISUAL_SETTLE_MS) {
                    Ok(()) => match platform.read_xl9535(pin) {
                        Ok(active) => {
                            reporter.note_fmt(format_args!(
                                "{} state changed from {} to {}",
                                target.label(),
                                idle,
                                active
                            ));
                            if active != idle {
                                CaseOutcome::Passed
                            } else {
                                reporter.note("Input level did not change while the control was pressed.");
                                CaseOutcome::Failed
                            }
                        }
                        Err(err) => {
                            reporter.note_fmt(format_args!("Input read error: {:?}", err));
                            CaseOutcome::Failed
                        }
                    },
                    Err(err) => {
                        reporter.note_fmt(format_args!("Delay error during input test: {:?}", err));
                        CaseOutcome::Failed
                    }
                }
            }
            Err(err) => {
                reporter.note_fmt(format_args!("Idle read error: {:?}", err));
                CaseOutcome::Failed
            }
        },
        Err(err) => {
            reporter.note_fmt(format_args!("Input configuration error: {:?}", err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_speaker_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    reporter.begin_case("speaker", "Play a test tone through the speaker path");
    let outcome = match platform.play_test_tone(AUDIO_TONE_HZ, AUDIO_TONE_MS) {
        Ok(()) => {
            if require_confirm(reporter, options, "Did you hear the test tone from the speaker?") {
                CaseOutcome::Passed
            } else {
                reporter.note("Operator did not confirm the speaker output.");
                CaseOutcome::Failed
            }
        }
        Err(err) => {
            reporter.note_fmt(format_args!("Speaker test error: {:?}", err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_microphone_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    reporter.begin_case("microphones", "Measure microphone activity through the ES7243E path");
    if options.require_operator_confirmations {
        let _ = reporter.confirm("Tap or speak near the microphones now, then answer yes.");
    } else {
        reporter.note("Operator confirmations are disabled; microphone sampling starts immediately.");
    }

    let outcome = match platform.microphone_peak(MIC_SAMPLE_MS) {
        Ok(peak) => {
            reporter.note_fmt(format_args!(
                "Observed microphone peak {} (threshold {}).",
                peak,
                options.microphone_peak_threshold
            ));
            if peak >= options.microphone_peak_threshold {
                CaseOutcome::Passed
            } else {
                reporter.note("Microphone peak stayed below the configured threshold.");
                CaseOutcome::Failed
            }
        }
        Err(err) => {
            reporter.note_fmt(format_args!("Microphone test error: {:?}", err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_camera_capture_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    if !options.require_camera_sensor {
        skip_case(
            summary,
            reporter,
            "camera-capture",
            "Capture a frame from the camera data bus",
            "Camera capture is disabled in options.",
        );
        return;
    }

    reporter.begin_case("camera-capture", "Capture a frame from the camera data bus");
    let outcome = match platform.capture_camera_frame() {
        Ok(stats) => {
            reporter.note_fmt(format_args!(
                "Camera frame stats: {}x{}, non_zero_bytes={}",
                stats.width,
                stats.height,
                stats.non_zero_bytes
            ));
            if stats.looks_valid() {
                CaseOutcome::Passed
            } else {
                reporter.note("Camera frame statistics looked invalid.");
                CaseOutcome::Failed
            }
        }
        Err(err) => {
            reporter.note_fmt(format_args!("Camera capture error: {:?}", err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_usb_manual_case<R>(
    summary: &mut TestSummary,
    reporter: &mut R,
    options: SelfTestOptions,
) where
    R: Reporter,
{
    if !options.require_usb_enumeration {
        skip_case(
            summary,
            reporter,
            "usb",
            "Confirm USB-C enumeration over GPIO19 / GPIO20",
            "USB enumeration check is disabled in options.",
        );
        return;
    }

    reporter.begin_case("usb", "Confirm USB-C enumeration over GPIO19 / GPIO20");
    let outcome = if reporter.confirm(
        "With a USB-C cable attached to a host, did the board enumerate and stay powered?",
    ) {
        CaseOutcome::Passed
    } else {
        reporter.note("Operator did not confirm USB-C enumeration.");
        CaseOutcome::Failed
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_xl9535_interrupt_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    if !options.require_external_loopback {
        skip_case(
            summary,
            reporter,
            "xl9535-int",
            "Verify the XL9535 interrupt line on GPIO43",
            "External loopback tests are disabled in options.",
        );
        return;
    }

    reporter.begin_case("xl9535-int", "Verify the XL9535 interrupt line on GPIO43");
    if options.require_operator_confirmations {
        let _ = reporter.confirm("Install a jumper between IO 1.P0 and IO 2.P1, then answer yes.");
    }

    let outcome = match check_xl9535_interrupt(platform, reporter) {
        Ok(true) => CaseOutcome::Passed,
        Ok(false) => {
            reporter.note("The XL9535 interrupt line did not assert after the input transition.");
            CaseOutcome::Failed
        }
        Err(err) => {
            reporter.note_fmt(format_args!("XL9535 interrupt test error: {:?}", err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn run_loopback_case<P, R>(
    summary: &mut TestSummary,
    platform: &mut P,
    reporter: &mut R,
    target: LoopbackTarget,
    options: SelfTestOptions,
) where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    let enabled = match target {
        LoopbackTarget::ExtP1
        | LoopbackTarget::ExtP2
        | LoopbackTarget::ExtP3
        | LoopbackTarget::ExtP4
        | LoopbackTarget::ExtP5
        | LoopbackTarget::ExtP6
        | LoopbackTarget::ExtP7 => options.require_external_loopback,
        LoopbackTarget::UnusedGpio35
        | LoopbackTarget::UnusedGpio36
        | LoopbackTarget::UnusedGpio37 => options.require_unrouted_probe_fixture,
    };

    if !enabled {
        skip_case(
            summary,
            reporter,
            target.case_id(),
            target.description(),
            "This loopback target is disabled in options.",
        );
        return;
    }

    reporter.begin_case(target.case_id(), target.description());
    if options.require_operator_confirmations {
        let _ = reporter.confirm(target.jumper_prompt());
    }

    let outcome = match check_loopback(platform, target) {
        Ok(true) => CaseOutcome::Passed,
        Ok(false) => {
            reporter.note("Loopback level did not match the expected drive state.");
            CaseOutcome::Failed
        }
        Err(err) => {
            reporter.note_fmt(format_args!("Loopback test error: {:?}", err));
            CaseOutcome::Failed
        }
    };
    summary.record(outcome);
    reporter.end_case(outcome);
}

fn check_xl9535_interrupt<P, R>(platform: &mut P, reporter: &mut R) -> Result<bool, P::Error>
where
    P: Platform,
    P::Error: fmt::Debug,
    R: Reporter,
{
    let direct_pin = board::EXT_IO1.p0;
    let expander_pin = board::EXT_IO2.p1;

    platform.configure_xl9535_input(expander_pin)?;
    platform.configure_direct_output(direct_pin, false)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    platform.clear_xl9535_interrupt()?;
    platform.write_direct(direct_pin, true)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    let asserted = platform.xl9535_interrupt_asserted()?;
    platform.clear_xl9535_interrupt()?;
    platform.configure_direct_input(direct_pin, Pull::Down)?;

    if !asserted {
        reporter.note("Try checking the IO 1.P0 to IO 2.P1 jumper and the XL9535 INT# pull-up path.");
    }

    Ok(asserted)
}

fn check_loopback<P>(platform: &mut P, target: LoopbackTarget) -> Result<bool, P::Error>
where
    P: Platform,
{
    match target.endpoint() {
        LoopbackEndpoint::Expander(pin) => {
            check_direct_to_expander_loopback(platform, board::EXT_IO1.p0, pin)
        }
        LoopbackEndpoint::Direct(pin) => {
            check_direct_to_direct_loopback(platform, board::EXT_IO1.p0, pin)
        }
    }
}

fn check_direct_to_expander_loopback<P>(
    platform: &mut P,
    direct_pin: u8,
    expander_pin: board::Xl9535Pin,
) -> Result<bool, P::Error>
where
    P: Platform,
{
    platform.configure_direct_input(direct_pin, Pull::Down)?;
    platform.configure_xl9535_output(expander_pin, false)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    let low_on_direct = platform.read_direct(direct_pin)?;

    platform.write_xl9535(expander_pin, true)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    let high_on_direct = platform.read_direct(direct_pin)?;

    platform.configure_xl9535_input(expander_pin)?;
    platform.configure_direct_output(direct_pin, false)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    let low_on_expander = platform.read_xl9535(expander_pin)?;

    platform.write_direct(direct_pin, true)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    let high_on_expander = platform.read_xl9535(expander_pin)?;

    platform.configure_direct_input(direct_pin, Pull::Down)?;
    platform.configure_xl9535_input(expander_pin)?;

    Ok(!low_on_direct && high_on_direct && !low_on_expander && high_on_expander)
}

fn check_direct_to_direct_loopback<P>(
    platform: &mut P,
    first_pin: u8,
    second_pin: u8,
) -> Result<bool, P::Error>
where
    P: Platform,
{
    platform.configure_direct_input(first_pin, Pull::Down)?;
    platform.configure_direct_output(second_pin, false)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    let low_on_first = platform.read_direct(first_pin)?;

    platform.write_direct(second_pin, true)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    let high_on_first = platform.read_direct(first_pin)?;

    platform.configure_direct_input(second_pin, Pull::Down)?;
    platform.configure_direct_output(first_pin, false)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    let low_on_second = platform.read_direct(second_pin)?;

    platform.write_direct(first_pin, true)?;
    platform.delay_ms(SIGNAL_SETTLE_MS)?;
    let high_on_second = platform.read_direct(second_pin)?;

    platform.configure_direct_input(first_pin, Pull::Down)?;
    platform.configure_direct_input(second_pin, Pull::Down)?;

    Ok(!low_on_first && high_on_first && !low_on_second && high_on_second)
}

fn require_confirm<R>(reporter: &mut R, options: SelfTestOptions, prompt: &'static str) -> bool
where
    R: Reporter,
{
    if options.require_operator_confirmations {
        reporter.confirm(prompt)
    } else {
        reporter.note("Operator confirmations are disabled; accepting software-side success for this case.");
        true
    }
}

fn skip_case<R>(
    summary: &mut TestSummary,
    reporter: &mut R,
    id: &'static str,
    description: &'static str,
    reason: &'static str,
) where
    R: Reporter,
{
    reporter.begin_case(id, description);
    reporter.note(reason);
    summary.record(CaseOutcome::Skipped);
    reporter.end_case(CaseOutcome::Skipped);
}
