//! Default Windows metrics, color schemes, and appearance registry preset data.
//!
//! Replaces raw binary magic numbers with structured, semantic GDI definitions
//! while providing safe byte-slice views for Windows registry persistence.

use std::{borrow::Cow, mem};

use windows::Win32::Graphics::Gdi::{DEFAULT_CHARSET, LOGFONTW};
use winreg::enums::RegType::REG_BINARY;

/// String key-value pairs to write under `Control Panel\Appearance`.
pub const APPEARANCE_ENTRIES: &[(&str, &str)] = &[("Current", ""), ("NewCurrent", "")];

/// System colors to write under `Control Panel\Colors`.
pub const COLOR_ENTRIES: &[(&str, &str)] = &[
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
pub const WINDOW_METRICS_STRING_ENTRIES: &[(&str, &str)] = &[
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
pub const WINDOW_METRICS_BINARY_FONT_KEYS: &[&str] = &[
    "CaptionFont",
    "IconFont",
    "MenuFont",
    "MessageFont",
    "SmCaptionFont",
    "StatusFont",
];

/// Return the binary RegValue for the default Microsoft YaHei UI font.
///
/// Encapsulates the GDI logical font specification into an owned binary payload
/// suitable for storing inside `Control Panel\Desktop\WindowMetrics`.
pub fn default_font_reg_value() -> winreg::RegValue<'static> {
    let font_bytes = build_default_yahei_font_bytes();
    winreg::RegValue {
        vtype: REG_BINARY,
        bytes: Cow::Owned(font_bytes),
    }
}

/// Construct the default Microsoft YaHei UI (9pt, regular) logical font and return its raw byte representation.
///
/// Converts a fully semantic [`LOGFONTW`] representation into a 92-byte buffer
/// required by Windows metrics registry entries.
fn build_default_yahei_font_bytes() -> Vec<u8> {
    let default_logical_font = create_default_yahei_logfont();
    convert_logfont_to_bytes(&default_logical_font)
}

/// Create a zero-initialized [`LOGFONTW`] for Microsoft YaHei UI (9pt, regular).
fn create_default_yahei_logfont() -> LOGFONTW {
    // Zero-initialize the entire struct so all unused array slots and padding
    // bytes contain deterministic zero bits rather than uninitialized memory.
    let mut logical_font: LOGFONTW = unsafe { mem::zeroed() };

    // -12 corresponds to 9pt regular at standard 96 DPI: -MulDiv(9, 96, 72) = -12
    logical_font.lfHeight = -12;
    // Weight 400 corresponds to FW_NORMAL
    logical_font.lfWeight = 400;
    // DEFAULT_CHARSET = 1
    logical_font.lfCharSet = DEFAULT_CHARSET;
    // 0x05 (0b0000_0101): Native Windows dump value combining FIXED_PITCH (1)
    // with the TrueType vector font flag TMPF_TRUETYPE (4), matching the OS default.
    logical_font.lfPitchAndFamily = 5;

    copy_face_name(&mut logical_font.lfFaceName, "Microsoft YaHei UI");

    logical_font
}

/// Safely populate the fixed-size UTF-16 face name buffer without unbounded indexing.
fn copy_face_name(destination_buffer: &mut [u16; 32], font_name: &str) {
    let utf16_units = font_name.encode_utf16();
    // Leave at least the final element zeroed for null-termination.
    let writable_destination = destination_buffer
        .split_last_mut()
        .map(|(_, prefix)| prefix);

    if let Some(destination_slice) = writable_destination {
        for (destination_cell, source_unit) in destination_slice.iter_mut().zip(utf16_units) {
            *destination_cell = source_unit;
        }
    }
}

/// Safely convert a fully-initialized [`LOGFONTW`] reference into an owned byte vector.
fn convert_logfont_to_bytes(logical_font: &LOGFONTW) -> Vec<u8> {
    let structure_byte_size = mem::size_of::<LOGFONTW>();
    let structure_pointer = std::ptr::from_ref(logical_font).cast::<_>();

    // SAFETY: `logical_font` is zero-initialized and fully valid for reads of `size_of::<LOGFONTW>()` bytes.
    let byte_slice = unsafe { std::slice::from_raw_parts(structure_pointer, structure_byte_size) };
    byte_slice.to_vec()
}
