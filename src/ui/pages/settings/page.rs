//! [`SettingsPage`] — the public handle for the settings tab page.
//!
//! Owns all controls and exposes only the operations the rest of the
//! application needs: construction and locale-driven text updates.

use rust_i18n::t;
use winsafe::{
    AnyResult, gui,
    prelude::{GuiParent, GuiWindow},
};

use super::{CheckboxId, build, event};

/// The settings tab page.
///
/// Cloning is cheap — all inner fields are reference-counted WinSafe handles.
/// The [`From`] impl allows this to be handed directly to [`gui::Tab`] pages.
#[derive(Clone)]
pub struct SettingsPage {
    tab_page: gui::TabPage,
    group_box: gui::Button,
    scrollable_panel: gui::WindowControl,
    content_panel: gui::WindowControl,
    checkboxes: Vec<(CheckboxId, gui::Button)>,
    button_select_all_toggle: gui::Button,
    button_apply: gui::Button,
}

impl From<SettingsPage> for gui::TabPage {
    fn from(settings_page: SettingsPage) -> gui::TabPage {
        settings_page.tab_page.clone()
    }
}

impl SettingsPage {
    /// Construct the settings page, creating all controls and registering all events.
    ///
    /// Must be called before the message loop starts, on the same thread as
    /// the parent window.
    pub fn new(parent_window: &(impl GuiParent + 'static), status_bar: gui::StatusBar) -> Self {
        let tab_page = build::create_tab_page(parent_window);
        let group_box = build::create_group_box(&tab_page);
        let scrollable_panel = build::create_scrollable_panel(&tab_page);
        let content_panel = build::create_content_panel(&scrollable_panel);
        let checkboxes = build::create_checkboxes(&content_panel);
        let button_select_all_toggle = build::create_button_select_all_toggle(&tab_page);
        let button_apply = build::create_button_apply(&tab_page);

        event::setup_all_events(
            &tab_page,
            &group_box,
            &scrollable_panel,
            &content_panel,
            &checkboxes,
            &button_select_all_toggle,
            &button_apply,
            &status_bar,
        );

        Self {
            tab_page,
            group_box,
            scrollable_panel,
            content_panel,
            checkboxes,
            button_select_all_toggle,
            button_apply,
        }
    }

    /// Re-translate all visible text labels to the current locale.
    ///
    /// Called by [`crate::ui::tab::container::TabContainer::update_page_contents`]
    /// after a language change.
    pub fn update_texts(&self) -> AnyResult<()> {
        self.group_box
            .hwnd()
            .SetWindowText(&t!("GROUP_BOX_SETTINGS_TITLE"))?;

        for (checkbox_id, checkbox) in &self.checkboxes {
            checkbox.hwnd().SetWindowText(&t!(checkbox_id.i18n_key()))?;
        }

        self.button_select_all_toggle
            .hwnd()
            .SetWindowText(&t!("BUTTON_SELECT_ALL_TOGGLE"))?;

        self.button_apply
            .hwnd()
            .SetWindowText(&t!("BUTTON_APPLY"))?;

        Ok(())
    }
}
