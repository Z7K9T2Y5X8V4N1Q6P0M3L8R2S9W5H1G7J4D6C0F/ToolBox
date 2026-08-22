use winsafe::{POINT, SIZE, gui};

const GROUP_BOX_MARGIN: i32 = 10;
pub(super) const BUTTON_WIDTH: i32 = 75;
pub(super) const BUTTON_HEIGHT: i32 = 25;
const BUTTON_HORIZONTAL_GAP: i32 = 5;

pub(super) struct SettingsPageLayout {
    pub group_box_position: POINT,
    pub group_box_size: SIZE,
    pub button_apply_position: POINT,
    pub button_apply_size: SIZE,
    pub button_select_all_toggle_position: POINT,
    pub button_select_all_toggle_size: SIZE,
}

impl SettingsPageLayout {
    pub fn calculate(tab_page_client_width: i32, tab_page_client_height: i32) -> Self {
        let group_box_margin = gui::dpi_x(GROUP_BOX_MARGIN);
        let button_width = gui::dpi_x(BUTTON_WIDTH);
        let button_height = gui::dpi_y(BUTTON_HEIGHT);
        let button_horizontal_gap = gui::dpi_x(BUTTON_HORIZONTAL_GAP);

        let group_box_width = tab_page_client_width - 2 * group_box_margin;
        let group_box_height =
            tab_page_client_height - (2 * group_box_margin) - button_height - group_box_margin;

        let button_row_vertical_position = group_box_margin + group_box_height + group_box_margin;
        let button_apply_horizontal_position =
            tab_page_client_width - group_box_margin - button_width;
        let button_select_all_toggle_horizontal_position =
            button_apply_horizontal_position - button_horizontal_gap - button_width;

        Self {
            group_box_position: POINT {
                x: group_box_margin,
                y: group_box_margin,
            },
            group_box_size: SIZE {
                cx: group_box_width,
                cy: group_box_height,
            },
            button_apply_position: POINT {
                x: button_apply_horizontal_position,
                y: button_row_vertical_position,
            },
            button_apply_size: SIZE {
                cx: button_width,
                cy: button_height,
            },
            button_select_all_toggle_position: POINT {
                x: button_select_all_toggle_horizontal_position,
                y: button_row_vertical_position,
            },
            button_select_all_toggle_size: SIZE {
                cx: button_width,
                cy: button_height,
            },
        }
    }
}
