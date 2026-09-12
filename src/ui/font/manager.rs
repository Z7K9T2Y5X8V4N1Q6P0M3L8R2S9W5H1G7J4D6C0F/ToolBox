//! System font manager for synchronizing application controls with Windows font metrics.
//!
//! Queries [`winsafe::NONCLIENTMETRICS`] via `SystemParametersInfo(SPI_GETNONCLIENTMETRICS)`
//! and applies the configured message font (`lfMessageFont`) to the window and all of
//! its descendant controls via Win32 `WM_SETFONT`.

use rust_i18n::t;
use windows::Win32::{
    Foundation::{BOOL, FALSE, HWND as RawHwnd, LPARAM, TRUE, WPARAM},
    UI::WindowsAndMessaging::{EnumChildWindows, SendMessageW, WM_SETFONT},
};
use winsafe::{
    AnyResult, HFONT, HWND, LOGFONT, NONCLIENTMETRICS, SystemParametersInfo, co,
    guard::DeleteObjectGuard,
};

/// The outcome of synchronizing the application font with system metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSyncResult {
    /// The font attributes have changed, and the new font was applied to all controls.
    Changed,
    /// The system font matches the currently active font; no update is needed.
    Unchanged,
}

/// Manages the application's active GDI font handle and synchronizes with system metrics.
pub struct FontManager {
    /// Holds the active GDI font handle to keep it alive for all controls.
    ///
    /// When replaced, the previous font guard is automatically dropped and
    /// deleted via Win32 `DeleteObject`.
    current_font_guard: Option<DeleteObjectGuard<HFONT>>,
    /// Cached system logical font used to detect configuration changes.
    current_logfont: Option<LOGFONT>,
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FontManager {
    /// Create a new uninitialized font manager.
    pub fn new() -> Self {
        Self {
            current_font_guard: None,
            current_logfont: None,
        }
    }

    /// Synchronize the font with the current Windows system settings.
    ///
    /// Reads the system non-client metrics. If the message font has changed since
    /// the last synchronization (or on initial call), creates a new [`HFONT`],
    /// applies it to the given window and all child controls, and safely drops
    /// the previous font.
    ///
    /// Returns `true` if the font changed and was applied, or `false` if the
    /// system font is identical to the currently active font.
    pub fn sync_system_font(&mut self, main_window_hwnd: &HWND) -> AnyResult<FontSyncResult> {
        let system_non_client_metrics = fetch_system_non_client_metrics();
        let target_logfont = system_non_client_metrics.lfMessageFont;

        if let Some(current_logfont) = &self.current_logfont {
            if is_logfont_equal(current_logfont, &target_logfont) {
                return Ok(FontSyncResult::Unchanged);
            }
        }

        let new_font_guard = HFONT::CreateFontIndirect(&target_logfont)?;

        apply_font_to_window_tree(main_window_hwnd, &new_font_guard);

        self.current_font_guard = Some(new_font_guard);
        self.current_logfont = Some(target_logfont);

        Ok(FontSyncResult::Changed)
    }
}

/// Fetch system non-client metrics from Windows.
fn fetch_system_non_client_metrics() -> NONCLIENTMETRICS {
    let mut non_client_metrics = NONCLIENTMETRICS::default();
    unsafe {
        SystemParametersInfo(
            co::SPI::GETNONCLIENTMETRICS,
            size_of::<NONCLIENTMETRICS>() as u32,
            &mut non_client_metrics,
            co::SPIF::NoValue,
        )
    }
    .unwrap_or_else(|_| panic!("{}", t!("ERROR_GET_NONCLIENTMETRICS_FAILED")));
    non_client_metrics
}

/// Check whether two [`LOGFONT`] structures represent the same font configuration.
///
/// A field-by-field comparison is performed rather than a raw byte comparison (such
/// as `memcmp`) because Win32 `LOGFONT` structures contain a fixed-size `lfFaceName`
/// buffer (`[u16; 32]`). The memory bytes past the null terminator (`\0`) are
/// undefined and may contain uninitialized padding or garbage data between successive
/// `SystemParametersInfo` calls. Calling `lfFaceName()` parses the valid null-terminated
/// string for exact semantic equality.
fn is_logfont_equal(current_logfont: &LOGFONT, target_logfont: &LOGFONT) -> bool {
    current_logfont.lfHeight == target_logfont.lfHeight
        && current_logfont.lfWidth == target_logfont.lfWidth
        && current_logfont.lfWeight == target_logfont.lfWeight
        && current_logfont.lfItalic == target_logfont.lfItalic
        && current_logfont.lfUnderline == target_logfont.lfUnderline
        && current_logfont.lfStrikeOut == target_logfont.lfStrikeOut
        && current_logfont.lfCharSet == target_logfont.lfCharSet
        && current_logfont.lfFaceName() == target_logfont.lfFaceName()
}

/// Apply the given font handle to the window and all descendant child windows.
fn apply_font_to_window_tree(main_window_hwnd: &HWND, font_guard: &HFONT) {
    let raw_main_hwnd = RawHwnd(main_window_hwnd.ptr());

    let _ = unsafe {
        EnumChildWindows(
            raw_main_hwnd,
            Some(apply_font_to_child_window_callback),
            LPARAM(font_guard.ptr() as isize),
        )
    };
}

/// Callback for `EnumChildWindows` to dispatch `WM_SETFONT` to each child window.
unsafe extern "system" fn apply_font_to_child_window_callback(
    child_window_hwnd: RawHwnd,
    lparam: LPARAM,
) -> BOOL {
    let execution_result = std::panic::catch_unwind(|| {
        let font_handle_wparam = WPARAM(lparam.0 as usize);
        unsafe {
            SendMessageW(
                child_window_hwnd,
                WM_SETFONT,
                font_handle_wparam,
                LPARAM(TRUE.0 as isize),
            );
        }
    });

    match execution_result {
        Ok(_) => TRUE,
        Err(_) => FALSE,
    }
}
