//! Default Windows metrics, color schemes, and appearance registry preset data.

use std::borrow::Cow;

use winreg::enums::RegType::REG_BINARY;

/// Caption and dialog font byte payload for default Microsoft YaHei UI (9pt, regular).
pub const DEFAULT_YAHEI_FONT_BYTES: [u8; 92] = [
    0xf4, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x90, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x4d, 0x00, 0x69, 0x00,
    0x63, 0x00, 0x72, 0x00, 0x6f, 0x00, 0x73, 0x00, 0x6f, 0x00, 0x66, 0x00, 0x74, 0x00, 0x20, 0x00,
    0x59, 0x00, 0x61, 0x00, 0x48, 0x00, 0x65, 0x00, 0x69, 0x00, 0x20, 0x00, 0x55, 0x00, 0x49, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// String key-value pairs to write under `Control Panel\Appearance`.
pub const APPEARANCE_ENTRIES: [(&str, &str); 2] = [("Current", ""), ("NewCurrent", "")];

/// System colors to write under `Control Panel\Colors`.
pub const COLOR_ENTRIES: [(&str, &str); 32] = [
    ("ActiveBorder", "180 180 180"),
    ("ActiveTitle", "153 180 209"),
    ("AppWorkspace", "171 171 171"),
    ("Background", "0 0 0"),
    ("ButtonAlternateFace", "0 0 0"),
    ("ButtonDkShadow", "105 105 105"),
    ("ButtonFace", "240 240 240"),
    ("ButtonHilight", "255 255 255"),
    ("ButtonLight", "227 227 227"),
    ("ButtonShadow", "160 160 160"),
    ("ButtonText", "0 0 0"),
    ("GradientActiveTitle", "185 209 234"),
    ("GradientInactiveTitle", "215 228 242"),
    ("GrayText", "109 109 109"),
    ("Hilight", "0 120 215"),
    ("HilightText", "255 255 255"),
    ("HotTrackingColor", "0 102 204"),
    ("InactiveBorder", "244 247 252"),
    ("InactiveTitle", "191 205 219"),
    ("InactiveTitleText", "0 0 0"),
    ("InfoText", "0 0 0"),
    ("InfoWindow", "255 255 225"),
    ("Menu", "240 240 240"),
    ("MenuBar", "240 240 240"),
    ("MenuHilight", "0 120 215"),
    ("MenuText", "0 0 0"),
    ("Scrollbar", "200 200 200"),
    ("TitleText", "0 0 0"),
    ("Window", "255 255 255"),
    ("WindowFrame", "100 100 100"),
    ("WindowText", "0 0 0"),
    ("HotTracking", "0 0 128"),
];

/// Metrics string values under `Control Panel\Desktop\WindowMetrics`.
pub const WINDOW_METRICS_STRING_ENTRIES: [(&str, &str); 15] = [
    ("BorderWidth", "-15"),
    ("CaptionHeight", "-330"),
    ("CaptionWidth", "-330"),
    ("IconTitleWrap", "1"),
    ("MenuHeight", "-285"),
    ("MenuWidth", "-285"),
    ("ScrollHeight", "-255"),
    ("ScrollWidth", "-255"),
    ("Shell Icon Size", "32"),
    ("SmCaptionHeight", "-330"),
    ("SmCaptionWidth", "-330"),
    ("PaddedBorderWidth", "-60"),
    ("IconVerticalSpacing", "-1125"),
    ("IconSpacing", "-1125"),
    ("MinAnimate", "1"),
];

/// Names of binary font metrics under `Control Panel\Desktop\WindowMetrics`.
pub const WINDOW_METRICS_BINARY_FONT_KEYS: [&str; 6] = [
    "CaptionFont",
    "IconFont",
    "MenuFont",
    "MessageFont",
    "SmCaptionFont",
    "StatusFont",
];

/// Applied DPI integer value under `Control Panel\Desktop\WindowMetrics`.
pub const APPLIED_DPI_VALUE: u32 = 0x0000_0060;

/// Return the binary RegValue for the default font.
pub fn default_font_reg_value() -> winreg::RegValue<'static> {
    winreg::RegValue {
        vtype: REG_BINARY,
        bytes: Cow::Borrowed(&DEFAULT_YAHEI_FONT_BYTES),
    }
}
