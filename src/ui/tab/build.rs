//! Tab control construction.
//!
//! The tab control is built once during [`TabContainer::new`] and its titles
//! are refreshed on every locale change via [`get_tab_control_titles`].

use rust_i18n::t;
use winsafe::{gui, prelude::GuiParent};

use crate::ui::pages::{SettingsPage, WindowVisualStylesPage};

/// Create the [`gui::Tab`] control with all pages attached.
///
/// Page order here determines the tab order visible to the user.
/// Titles are sourced from the current locale at construction time.
pub(super) fn create_tab_control(
    parent_window: &(impl GuiParent + 'static),
    settings_page: &SettingsPage,
    window_visual_styles_page: &WindowVisualStylesPage,
) -> gui::Tab {
    let tab_control_titles = get_tab_control_titles();

    gui::Tab::new(
        parent_window,
        gui::TabOpts {
            pages: &[
                (&tab_control_titles[0], settings_page.clone().into()),
                (
                    &tab_control_titles[1],
                    window_visual_styles_page.clone().into(),
                ),
            ],
            ..Default::default()
        },
    )
}

/// Return the localized title string for each tab, in display order.
///
/// Called both during construction and on locale change so tab titles
/// always reflect the current language.
pub(super) fn get_tab_control_titles() -> Vec<String> {
    vec![
        t!("TAB_SETTINGS").to_string(),
        t!("TAB_WINDOW_VISUAL_STYLES").to_string(),
    ]
}
