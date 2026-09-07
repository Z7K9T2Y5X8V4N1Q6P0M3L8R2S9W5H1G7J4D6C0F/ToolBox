//! DPI-aware layout constants and calculators for the window visual styles page.
//!
//! All raw pixel constants are defined at 96 DPI (100% scaling).
//! [`gui::dpi_x`] and [`gui::dpi_y`] scale them to the actual display DPI
//! at runtime, so the layout looks correct at any scaling factor.

use rust_i18n::t;
use winsafe::{
    HFONT, HWND, NONCLIENTMETRICS, POINT, SIZE, SystemParametersInfo, co, guard::DeleteObjectGuard,
    gui,
};

// ---------------------------------------------------------------------------
// Raw pixel constants at 96 DPI
// ---------------------------------------------------------------------------

const PAGE_MARGIN: i32 = 10;
const CONTROL_VERTICAL_GAP: i32 = 10;

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------
/// Clamp a dimension to zero if negative.
///
/// Used throughout layout calculations to ensure widths and heights
/// never become negative when the window is resized very small.
#[inline]
fn non_negative(value: i32) -> i32 {
    value.max(0)
}

// ---------------------------------------------------------------------------
// ListView Column Width Calculation
// ---------------------------------------------------------------------------

/// Calculated column widths for the process ListView control.
pub(super) struct ProcessListViewColumnWidths {
    pub process_name_column_width: i32,
    pub process_id_column_width: i32,
}

/// Calculate proportional column widths for the process ListView.
///
/// Allocates 70% of the total width to the process name column and the remaining
/// 30% to the process ID column. Subtracts the first column's width from the total
/// to avoid pixel rounding gaps.
pub(super) fn calculate_process_listview_column_widths(
    listview_width: i32,
) -> ProcessListViewColumnWidths {
    let listview_width = non_negative(listview_width);
    let process_name_column_width = (listview_width * 70) / 100;
    let process_id_column_width = listview_width - process_name_column_width;

    ProcessListViewColumnWidths {
        process_name_column_width,
        process_id_column_width,
    }
}

// ---------------------------------------------------------------------------
// WindowVisualStylesPageLayout
// ---------------------------------------------------------------------------

/// Pre-calculated positions and sizes for every control on the window visual styles page.
///
/// Computed once per WM_SIZE event from the current tab page client dimensions.
/// All values are DPI-scaled pixels relative to the tab page's client origin.
pub(super) struct WindowVisualStylesPageLayout {
    pub edit_position: POINT,
    pub edit_size: SIZE,
    pub listview_position: POINT,
    pub listview_size: SIZE,
}

impl WindowVisualStylesPageLayout {
    pub fn calculate(
        tab_page_client_width: i32,
        tab_page_client_height: i32,
        edit_height: i32,
    ) -> Self {
        let page_margin = gui::dpi_x(PAGE_MARGIN);
        let control_vertical_gap = gui::dpi_y(CONTROL_VERTICAL_GAP);

        let edit_position = POINT {
            x: page_margin,
            y: page_margin,
        };
        let edit_size = SIZE {
            cx: non_negative(tab_page_client_width - page_margin * 2),
            cy: edit_height,
        };

        let listview_position = POINT {
            x: page_margin,
            y: page_margin + edit_height + control_vertical_gap,
        };

        let listview_height = non_negative(
            tab_page_client_height
                - (page_margin + edit_height + control_vertical_gap + page_margin),
        );

        let listview_size = SIZE {
            cx: non_negative(tab_page_client_width - page_margin * 2),
            cy: listview_height,
        };

        Self {
            edit_position,
            edit_size,
            listview_position,
            listview_size,
        }
    }
}

/// Calculates the ideal vertical dimension for a single-line Edit control.
pub(super) fn calculate_edit_ideal_height(edit_hwnd: &HWND) -> i32 {
    let explicit_font = unsafe { edit_hwnd.SendMessage(winsafe::msg::WmGetFont {}) };
    let fallback_font_guard = explicit_font.is_none().then(create_fallback_font);

    // If an explicit font is present, borrow it directly; otherwise dereference the guard to &HFONT.
    let active_font: &HFONT = match &explicit_font {
        Some(font) => font,
        None => fallback_font_guard.as_ref().unwrap(),
    };

    let device_context = edit_hwnd
        .GetDC()
        .unwrap_or_else(|_| panic!("{}", t!("ERROR_GET_DEVICE_CONTEXT_FAILED")));

    let _font_selection_guard = device_context
        .SelectObject(active_font)
        .unwrap_or_else(|_| panic!("{}", t!("ERROR_SELECT_FONT_FAILED")));

    let text_metric = device_context
        .GetTextMetrics()
        .unwrap_or_else(|_| panic!("{}", t!("ERROR_GET_TEXT_METRICS_FAILED")));

    let font_height = text_metric.tmHeight + text_metric.tmExternalLeading;
    let edge_height = winsafe::GetSystemMetrics(co::SM::CYEDGE);

    (font_height + (edge_height * 2)).max(gui::dpi_y(25))
}

/// Retrieves the system default non-client message font as a fallback.
fn create_fallback_font() -> DeleteObjectGuard<HFONT> {
    let mut non_client_metrics = NONCLIENTMETRICS::default();
    unsafe {
        SystemParametersInfo(
            co::SPI::GETNONCLIENTMETRICS,
            std::mem::size_of::<NONCLIENTMETRICS>() as u32,
            &mut non_client_metrics,
            co::SPIF::NoValue,
        )
    }
    .unwrap_or_else(|_| panic!("{}", t!("ERROR_GET_NONCLIENTMETRICS_FAILED")));

    HFONT::CreateFontIndirect(&non_client_metrics.lfMessageFont)
        .unwrap_or_else(|_| panic!("{}", t!("ERROR_CREATE_FONT_FAILED")))
}
