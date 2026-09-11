//! [`TabContainer`] — owns the tab control and coordinates all tab pages.
//!
//! [`TabContainer`] is the single point of contact between the main window
//! and the tab UI. The main window calls [`TabContainer::resize`] on every
//! WM_SIZE and [`TabContainer::update_tab_control_titles`] /
//! [`TabContainer::update_page_contents`] on every locale change.

use winsafe::gui;
use winsafe::prelude::*;

use super::{build, layout};
use crate::ui::pages::{settings::SettingsPage, window_visual_styles::WindowVisualStylesPage};

/// Owns the tab control widget and all tab page instances.
///
/// Cloning is cheap — all inner types are reference-counted WinSafe handles.
#[derive(Clone)]
pub struct TabContainer {
    tab_control: gui::Tab,
    /// Flat list of tab pages in display order, used for resize calculations.
    tab_pages: Vec<gui::TabPage>,
    settings_page: SettingsPage,
    window_visual_styles_page: WindowVisualStylesPage,
}

impl TabContainer {
    /// Construct the tab control and all pages, wiring up their events.
    pub fn new(parent_window: &(impl GuiParent + 'static), status_bar: gui::StatusBar) -> Self {
        let settings_page = SettingsPage::new(parent_window, status_bar.clone());
        let window_visual_styles_page = WindowVisualStylesPage::new(parent_window, status_bar);

        let tab_pages = vec![
            settings_page.clone().into(),
            window_visual_styles_page.clone().into(),
        ];

        let tab_control =
            build::create_tab_control(parent_window, &settings_page, &window_visual_styles_page);

        Self {
            tab_control,
            tab_pages,
            settings_page,
            window_visual_styles_page,
        }
    }

    /// Resize the tab control and the currently visible page to fit the
    /// available client area. Called on every WM_SIZE of the main window.
    pub fn resize(&self, client_width: i32, client_height: i32) -> winsafe::AnyResult<()> {
        layout::resize_tab_control(&self.tab_control, client_width, client_height)?;
        layout::resize_current_tab_page(&self.tab_control, &self.tab_pages)?;
        Ok(())
    }

    /// Re-translate all tab title strings to the current locale.
    ///
    /// Called after a language change so the tab headers update immediately.
    pub fn update_tab_control_titles(&self) -> winsafe::AnyResult<()> {
        let tab_control_titles = build::get_tab_control_titles();
        for (tab_control_index, tab_control_title) in tab_control_titles.iter().enumerate() {
            let target_tab_control_item = self.tab_control.items().get(tab_control_index as u32);
            target_tab_control_item.set_text(tab_control_title)?;
        }
        Ok(())
    }

    /// Re-translate all text content inside each page to the current locale.
    ///
    /// Called after a language change so page labels update immediately.
    pub fn update_page_contents(&self) -> winsafe::AnyResult<()> {
        self.settings_page.update_texts()?;
        self.window_visual_styles_page.update_texts()?;
        Ok(())
    }

    /// Notify all hosted pages that the system UI font has changed.
    pub fn handle_font_changed(&self) -> winsafe::AnyResult<()> {
        self.window_visual_styles_page.handle_font_changed()?;
        Ok(())
    }
}
