//! DPI-aware layout constants and calculators for the settings page.
//!
//! All raw pixel constants are defined at 96 DPI (100% scaling).
//! [`gui::dpi_x`] and [`gui::dpi_y`] scale them to the actual display DPI
//! at runtime, so the layout looks correct at any scaling factor.

use rust_i18n::t;
use winsafe::{NONCLIENTMETRICS, POINT, SIZE, SystemParametersInfo, co, gui};

// ---------------------------------------------------------------------------
// Raw pixel constants at 96 DPI
// ---------------------------------------------------------------------------

const GROUP_BOX_MARGIN: i32 = 10;
const GROUP_BOX_TITLE_BAR_HEIGHT: i32 = 20;
const GROUP_BOX_INTERNAL_PADDING: i32 = 6;

const CHECKBOX_HEIGHT: i32 = 22;
const CHECKBOX_VERTICAL_GAP: i32 = 10;
const CHECKBOX_LEFT_MARGIN: i32 = 20;
const CHECKBOX_RIGHT_MARGIN: i32 = 20;
const CHECKBOX_TOP_MARGIN: i32 = 10;
const CHECKBOX_BOTTOM_MARGIN: i32 = 16;

/// Raw button width at 96 DPI. Exposed so [`super::build`] can pass it
/// to [`gui::ButtonOpts`] at construction time.
pub(super) const BUTTON_WIDTH: i32 = 75;

/// Raw button height at 96 DPI.
pub(super) const BUTTON_HEIGHT: i32 = 25;

const BUTTON_HORIZONTAL_GAP: i32 = 5;

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
// CheckboxLayoutCalculator
// ---------------------------------------------------------------------------

/// Calculates DPI-scaled positions and sizes for checkboxes in the content panel.
///
/// Construct once per layout pass. All fields are pre-scaled on construction
/// so individual calculations are simple arithmetic with no repeated DPI calls.
pub(super) struct CheckboxLayoutCalculator {
    checkbox_height: i32,
    checkbox_vertical_gap: i32,
    checkbox_left_margin: i32,
    checkbox_right_margin: i32,
    checkbox_top_margin: i32,
    checkbox_bottom_margin: i32,
}

impl CheckboxLayoutCalculator {
    /// Create a new calculator with all values scaled to current DPI and active system font height.
    pub fn new() -> Self {
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

        let font_pixel_height = non_client_metrics.lfMessageFont.lfHeight.abs();
        let dynamic_checkbox_height =
            (font_pixel_height + gui::dpi_y(6)).max(gui::dpi_y(CHECKBOX_HEIGHT));

        Self {
            checkbox_height: dynamic_checkbox_height,
            checkbox_vertical_gap: gui::dpi_y(CHECKBOX_VERTICAL_GAP),
            checkbox_left_margin: gui::dpi_x(CHECKBOX_LEFT_MARGIN),
            checkbox_right_margin: gui::dpi_x(CHECKBOX_RIGHT_MARGIN),
            checkbox_top_margin: gui::dpi_y(CHECKBOX_TOP_MARGIN),
            checkbox_bottom_margin: gui::dpi_y(CHECKBOX_BOTTOM_MARGIN),
        }
    }

    /// Return the top-left position for the checkbox at the given zero-based index.
    pub fn calculate_checkbox_position(&self, index: usize) -> POINT {
        POINT {
            x: self.checkbox_left_margin,
            y: self.checkbox_top_margin
                + (index as i32) * (self.checkbox_height + self.checkbox_vertical_gap),
        }
    }

    /// Return the checkbox width that fills the content panel minus side margins.
    ///
    /// Enforces a minimum of 100px so the text is never completely clipped.
    pub fn calculate_checkbox_width(&self, content_panel_width: i32) -> i32 {
        (content_panel_width - self.checkbox_left_margin - self.checkbox_right_margin).max(100)
    }

    /// Return the DPI-scaled checkbox height.
    pub fn checkbox_height(&self) -> i32 {
        self.checkbox_height
    }

    /// Return the total pixel height needed to display all checkboxes,
    /// including top and bottom margins and inter-checkbox gaps.
    pub fn calculate_total_content_height(&self, checkbox_count: usize) -> i32 {
        if checkbox_count == 0 {
            return 0;
        }

        self.checkbox_top_margin
            + (checkbox_count as i32) * (self.checkbox_height + self.checkbox_vertical_gap)
            - self.checkbox_vertical_gap
            + self.checkbox_bottom_margin
    }
}

// ---------------------------------------------------------------------------
// SettingsPageLayout
// ---------------------------------------------------------------------------

/// Pre-calculated positions and sizes for every control on the settings page.
///
/// Computed once per WM_SIZE event from the current tab page client dimensions.
/// All values are DPI-scaled pixels relative to the tab page's client origin.
pub(super) struct SettingsPageLayout {
    pub group_box_position: POINT,
    pub group_box_size: SIZE,
    pub scrollable_panel_position: POINT,
    pub scrollable_panel_size: SIZE,
    pub button_apply_position: POINT,
    pub button_apply_size: SIZE,
    pub button_select_all_toggle_position: POINT,
    pub button_select_all_toggle_size: SIZE,
}

impl SettingsPageLayout {
    /// Calculate the full page layout from the tab page's current client size.
    pub fn calculate(tab_page_client_width: i32, tab_page_client_height: i32) -> Self {
        let group_box_margin = gui::dpi_x(GROUP_BOX_MARGIN);
        let button_width = gui::dpi_x(BUTTON_WIDTH);
        let button_height = gui::dpi_y(BUTTON_HEIGHT);
        let button_horizontal_gap = gui::dpi_x(BUTTON_HORIZONTAL_GAP);
        let group_box_title_bar_height = gui::dpi_y(GROUP_BOX_TITLE_BAR_HEIGHT);
        let group_box_internal_padding = gui::dpi_x(GROUP_BOX_INTERNAL_PADDING);

        let group_box_width = non_negative(tab_page_client_width - 2 * group_box_margin);
        let group_box_height = non_negative(
            tab_page_client_height - (2 * group_box_margin) - button_height - group_box_margin,
        );

        let scrollable_panel_position = POINT {
            x: group_box_margin + group_box_internal_padding,
            y: group_box_margin + group_box_title_bar_height,
        };
        let scrollable_panel_size = SIZE {
            cx: non_negative(group_box_width - 2 * group_box_internal_padding),
            cy: non_negative(
                group_box_height - group_box_title_bar_height - group_box_internal_padding,
            ),
        };

        let button_vertical_position = group_box_margin + group_box_height + group_box_margin;
        let button_apply_horizontal_position =
            (tab_page_client_width - group_box_margin - button_width).max(group_box_margin);
        let button_select_all_toggle_horizontal_position =
            (button_apply_horizontal_position - button_horizontal_gap - button_width)
                .max(group_box_margin);

        Self {
            group_box_position: POINT {
                x: group_box_margin,
                y: group_box_margin,
            },
            group_box_size: SIZE {
                cx: group_box_width,
                cy: group_box_height,
            },
            scrollable_panel_position,
            scrollable_panel_size,
            button_apply_position: POINT {
                x: button_apply_horizontal_position,
                y: button_vertical_position,
            },
            button_apply_size: SIZE {
                cx: button_width,
                cy: button_height,
            },
            button_select_all_toggle_position: POINT {
                x: button_select_all_toggle_horizontal_position,
                y: button_vertical_position,
            },
            button_select_all_toggle_size: SIZE {
                cx: button_width,
                cy: button_height,
            },
        }
    }
}
