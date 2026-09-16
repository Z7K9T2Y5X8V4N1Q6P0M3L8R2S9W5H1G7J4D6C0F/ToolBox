//! [`CheckboxId`] — the canonical identifier for each settings checkbox.
//!
//! Each variant carries three pieces of metadata, all accessed via methods:
//! - A Win32 control ID used when creating the [`winsafe::gui::Button`]
//! - An i18n key for the checkbox label text
//! - An i18n key for the status bar description shown on hover

/// Identifies a single checkbox on the settings page.
///
/// Used as a typed key throughout the settings module to avoid passing
/// raw integers or strings. The ordering of [`CheckboxId::all`] determines
/// the top-to-bottom display order in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxId {
    DisableWindowsUpdate,
    PauseWindowsUpdate,
    HideWindowsUpdate,
    DisableTaskbarSearchAds,
    EnableModernExplorer,
    DisableWindowsSpotlight,
    DisableExplorerSearchBar,
    DisableExplorerContextMenu,
    DisableFolderTypeDiscovery,
    RemoveWindowsDefender,
    DisableCoreIsolation,
    DisableSpectreMeltdownPatches,
    DisableSmartScreen,
}

impl CheckboxId {
    /// Returns every [`CheckboxId`] variant in display order.
    ///
    /// This is the single source of truth for which checkboxes exist
    /// and the order in which they appear in the UI.
    pub fn all() -> &'static [CheckboxId] {
        &[
            CheckboxId::DisableWindowsUpdate,
            CheckboxId::PauseWindowsUpdate,
            CheckboxId::HideWindowsUpdate,
            CheckboxId::DisableTaskbarSearchAds,
            CheckboxId::EnableModernExplorer,
            CheckboxId::DisableWindowsSpotlight,
            CheckboxId::DisableExplorerSearchBar,
            CheckboxId::DisableExplorerContextMenu,
            CheckboxId::DisableFolderTypeDiscovery,
            CheckboxId::RemoveWindowsDefender,
            CheckboxId::DisableCoreIsolation,
            CheckboxId::DisableSpectreMeltdownPatches,
            CheckboxId::DisableSmartScreen,
        ]
    }

    /// Returns the i18n key for the status bar description shown when
    /// the mouse hovers over this checkbox.
    pub fn description_i18n_key(&self) -> &'static str {
        match self {
            CheckboxId::DisableWindowsUpdate => "CHECKBOX_DISABLE_WINDOWS_UPDATE_DESCRIPTION",
            CheckboxId::PauseWindowsUpdate => "CHECKBOX_PAUSE_WINDOWS_UPDATE_DESCRIPTION",
            CheckboxId::HideWindowsUpdate => "CHECKBOX_HIDE_WINDOWS_UPDATE_DESCRIPTION",
            CheckboxId::DisableTaskbarSearchAds => {
                "CHECKBOX_DISABLE_TASKBAR_SEARCH_ADS_DESCRIPTION"
            }
            CheckboxId::EnableModernExplorer => "CHECKBOX_ENABLE_MODERN_EXPLORER_DESCRIPTION",
            CheckboxId::DisableWindowsSpotlight => "CHECKBOX_DISABLE_WINDOWS_SPOTLIGHT_DESCRIPTION",
            CheckboxId::DisableExplorerSearchBar => {
                "CHECKBOX_DISABLE_EXPLORER_SEARCH_BAR_DESCRIPTION"
            }
            CheckboxId::DisableExplorerContextMenu => {
                "CHECKBOX_DISABLE_EXPLORER_CONTEXT_MENU_DESCRIPTION"
            }
            CheckboxId::DisableFolderTypeDiscovery => {
                "CHECKBOX_DISABLE_FOLDER_TYPE_DISCOVERY_DESCRIPTION"
            }
            CheckboxId::RemoveWindowsDefender => "CHECKBOX_REMOVE_WINDOWS_DEFENDER_DESCRIPTION",
            CheckboxId::DisableCoreIsolation => "CHECKBOX_DISABLE_CORE_ISOLATION_DESCRIPTION",
            CheckboxId::DisableSpectreMeltdownPatches => {
                "CHECKBOX_DISABLE_SPECTRE_MELTDOWN_PATCHES_DESCRIPTION"
            }
            CheckboxId::DisableSmartScreen => "CHECKBOX_DISABLE_SMARTSCREEN_DESCRIPTION",
        }
    }

    /// Returns the i18n key for this checkbox's label text.
    pub fn i18n_key(&self) -> &'static str {
        match self {
            CheckboxId::DisableWindowsUpdate => "CHECKBOX_DISABLE_WINDOWS_UPDATE",
            CheckboxId::PauseWindowsUpdate => "CHECKBOX_PAUSE_WINDOWS_UPDATE",
            CheckboxId::HideWindowsUpdate => "CHECKBOX_HIDE_WINDOWS_UPDATE",
            CheckboxId::DisableTaskbarSearchAds => "CHECKBOX_DISABLE_TASKBAR_SEARCH_ADS",
            CheckboxId::EnableModernExplorer => "CHECKBOX_ENABLE_MODERN_EXPLORER",
            CheckboxId::DisableWindowsSpotlight => "CHECKBOX_DISABLE_WINDOWS_SPOTLIGHT",
            CheckboxId::DisableExplorerSearchBar => "CHECKBOX_DISABLE_EXPLORER_SEARCH_BAR",
            CheckboxId::DisableExplorerContextMenu => "CHECKBOX_DISABLE_EXPLORER_CONTEXT_MENU",
            CheckboxId::DisableFolderTypeDiscovery => "CHECKBOX_DISABLE_FOLDER_TYPE_DISCOVERY",
            CheckboxId::RemoveWindowsDefender => "CHECKBOX_REMOVE_WINDOWS_DEFENDER",
            CheckboxId::DisableCoreIsolation => "CHECKBOX_DISABLE_CORE_ISOLATION",
            CheckboxId::DisableSpectreMeltdownPatches => {
                "CHECKBOX_DISABLE_SPECTRE_MELTDOWN_PATCHES"
            }
            CheckboxId::DisableSmartScreen => "CHECKBOX_DISABLE_SMARTSCREEN",
        }
    }

    /// Returns the Win32 control ID for this checkbox.
    ///
    /// Control IDs are used by Win32 to identify controls in WM_COMMAND
    /// and other messages. Each value must be unique within the same parent window.
    pub fn window_control_id(&self) -> u16 {
        match self {
            CheckboxId::DisableWindowsUpdate => 2001,
            CheckboxId::PauseWindowsUpdate => 2002,
            CheckboxId::HideWindowsUpdate => 2003,
            CheckboxId::DisableTaskbarSearchAds => 2004,
            CheckboxId::EnableModernExplorer => 2005,
            CheckboxId::DisableWindowsSpotlight => 2006,
            CheckboxId::DisableExplorerSearchBar => 2007,
            CheckboxId::DisableExplorerContextMenu => 2008,
            CheckboxId::DisableFolderTypeDiscovery => 2009,
            CheckboxId::RemoveWindowsDefender => 2010,
            CheckboxId::DisableCoreIsolation => 2011,
            CheckboxId::DisableSpectreMeltdownPatches => 2012,
            CheckboxId::DisableSmartScreen => 2013,
        }
    }
}
