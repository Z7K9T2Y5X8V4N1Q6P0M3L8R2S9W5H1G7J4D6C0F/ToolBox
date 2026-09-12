//! DPI-aware layout constants, calculators, and header visual indicator utilities.
//!
//! All raw pixel constants are defined at 96 DPI (100% scaling).
//! [`gui::dpi_x`] and [`gui::dpi_y`] scale them to the actual display DPI
//! at runtime, so the layout looks correct at any scaling factor.

use rust_i18n::t;

use windows::Win32::{
    Foundation::{HWND as RawHwnd, LPARAM, WPARAM},
    UI::{
        Controls::{
            HDF_SORTDOWN, HDF_SORTUP, HDI_FORMAT, HDITEMW, HDM_GETITEMCOUNT, HDM_GETITEMW,
            HDM_SETITEMW, LVM_GETHEADER,
        },
        WindowsAndMessaging::SendMessageW,
    },
};
use winsafe::{
    GetSystemMetrics, HFONT, HWND, NONCLIENTMETRICS, POINT, SIZE, SystemParametersInfo, co,
    guard::DeleteObjectGuard, gui, msg,
};

use super::process::{ProcessSortConfig, SortDirection};

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

/// Check whether the ListView currently has a native vertical scrollbar attached.
fn is_vertical_scrollbar_visible(listview_hwnd: &HWND) -> bool {
    let window_style_bits = listview_hwnd.GetWindowLongPtr(co::GWLP::STYLE) as u32;
    let window_style_flags = unsafe { co::WS::from_raw(window_style_bits) };
    window_style_flags.has(co::WS::VSCROLL)
}

// ---------------------------------------------------------------------------
// ListView Column Width Calculation
// ---------------------------------------------------------------------------

/// Calculated column widths for the process ListView control.
pub(super) struct ProcessListViewColumnWidths {
    pub process_name_column_width: i32,
    pub process_id_column_width: i32,
}

/// Calculate the usable client width for ListView columns without triggering horizontal scrollbars.
///
/// Deducts the vertical scrollbar width (if not already excluded by Win32 client rect calculations)
/// and a small safety margin to prevent rounding artifacts or grid line borders from causing
/// a horizontal scrollbar.
pub(super) fn calculate_listview_usable_column_width(listview_hwnd: &HWND) -> i32 {
    let client_rect = match listview_hwnd.GetClientRect() {
        Ok(client_rect) => client_rect,
        Err(_) => return 0,
    };

    let mut available_width = client_rect.right - client_rect.left;

    // Win32 behavior: If a window has WS_VSCROLL active, GetClientRect() already
    // automatically subtracts the vertical scrollbar width (SM_CXVSCROLL).
    // However, during initial layout or before items are populated, the scrollbar
    // is not yet visible, meaning GetClientRect() returns the full un-deducted width.
    // Because the process list will inevitably exceed visible rows, we must proactively
    // reserve space here to prevent columns from overflowing once the scrollbar appears.
    if !is_vertical_scrollbar_visible(listview_hwnd) {
        let vertical_scrollbar_width = GetSystemMetrics(co::SM::CXVSCROLL);
        available_width -= vertical_scrollbar_width;
    }

    non_negative(available_width)
}

/// Calculate proportional column widths for the process ListView.
///
/// Allocates 70% of the total usable width to the process name column and the remaining
/// 30% to the process ID column. Subtracts the first column's width from the total
/// to avoid pixel rounding gaps.
pub(super) fn calculate_process_listview_column_widths(
    usable_width: i32,
) -> ProcessListViewColumnWidths {
    let usable_width = non_negative(usable_width);
    let process_name_column_width = (usable_width * 70) / 100;
    let process_id_column_width = non_negative(usable_width - process_name_column_width);

    ProcessListViewColumnWidths {
        process_name_column_width,
        process_id_column_width,
    }
}

// ---------------------------------------------------------------------------
// Header Sort Arrow Indicator
// ---------------------------------------------------------------------------

/// Update the visual sort arrow indicators (▲ / ▼) on the ListView column headers.
///
/// Sends `HDM_SETITEMW` to the child Header control of the ListView to apply
/// `HDF_SORTUP` or `HDF_SORTDOWN` to the currently active sort column, while clearing
/// any sort flags from non-active columns.
pub(super) fn update_listview_header_sort_indicator(
    listview_hwnd: &HWND,
    sort_config: ProcessSortConfig,
) {
    let Some(listview_header_hwnd) = retrieve_listview_header_hwnd(listview_hwnd) else {
        return;
    };

    let total_column_count = fetch_header_column_count(listview_header_hwnd);
    let active_column_index = sort_config.column.to_column_index();
    for column_index in 0..total_column_count {
        let target_sort_direction =
            (column_index == active_column_index).then_some(sort_config.direction);
        update_column_header_sort_indicator(
            listview_header_hwnd,
            column_index,
            target_sort_direction,
        );
    }
}

/// Retrieve the child Header control handle from the ListView control.
fn retrieve_listview_header_hwnd(listview_hwnd: &HWND) -> Option<RawHwnd> {
    let raw_listview_hwnd = RawHwnd(listview_hwnd.ptr());
    let listview_header_hwnd_result =
        unsafe { SendMessageW(raw_listview_hwnd, LVM_GETHEADER, WPARAM(0), LPARAM(0)) };

    if listview_header_hwnd_result.0 == 0 {
        None
    } else {
        Some(RawHwnd(
            listview_header_hwnd_result.0 as *mut std::ffi::c_void,
        ))
    }
}

/// Query the total number of columns currently present in the Header control.
fn fetch_header_column_count(listview_header_hwnd: RawHwnd) -> usize {
    let column_count_result =
        unsafe { SendMessageW(listview_header_hwnd, HDM_GETITEMCOUNT, WPARAM(0), LPARAM(0)) };
    (column_count_result.0 as usize).max(0)
}

/// Update or clear the sort direction arrow (▲ / ▼) on a single column header item.
fn update_column_header_sort_indicator(
    listview_header_hwnd: RawHwnd,
    column_index: usize,
    target_sort_direction: Option<SortDirection>,
) {
    let mut header_item = HDITEMW {
        mask: HDI_FORMAT,
        ..Default::default()
    };

    let fetch_format_result = unsafe {
        SendMessageW(
            listview_header_hwnd,
            HDM_GETITEMW,
            WPARAM(column_index),
            LPARAM(std::ptr::from_mut(&mut header_item) as isize),
        )
    };
    if fetch_format_result.0 == 0 {
        return;
    }

    // Strip both sort arrow flags before applying new state.
    let stripped_format_flags = header_item.fmt.0 & !(HDF_SORTUP.0 | HDF_SORTDOWN.0);
    let new_sort_flag = match target_sort_direction {
        Some(SortDirection::Ascending) => HDF_SORTUP.0,
        Some(SortDirection::Descending) => HDF_SORTDOWN.0,
        None => 0,
    };

    header_item.fmt.0 = stripped_format_flags | new_sort_flag;

    unsafe {
        SendMessageW(
            listview_header_hwnd,
            HDM_SETITEMW,
            WPARAM(column_index),
            LPARAM(std::ptr::from_ref(&header_item) as isize),
        );
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
    let explicit_font = unsafe { edit_hwnd.SendMessage(msg::WmGetFont {}) };
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
    let edge_height = GetSystemMetrics(co::SM::CYEDGE);

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
