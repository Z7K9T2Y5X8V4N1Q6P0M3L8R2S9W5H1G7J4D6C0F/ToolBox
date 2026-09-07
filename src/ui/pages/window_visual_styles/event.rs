//! Window visual styles page event registration.
//!
//! [`setup_all_events`] is the single entry point called from [`WindowVisualStylesPage::new`].
//! It wires up every event handler for the window visual styles page in the correct order.

use std::{cell::RefCell, rc::Rc};

use rust_i18n::t;
use winsafe::{WString, gui, msg, prelude::*};

use super::layout::{WindowVisualStylesPageLayout, calculate_process_listview_column_widths};
use super::process::{ProcessItem, ProcessManager};
use crate::ui::tab::layout as tab_layout;

/// Unique Win32 timer ID for refreshing the process list.
const PROCESS_REFRESH_TIMER_ID: usize = 3001;

/// Process list auto-refresh interval in milliseconds.
const PROCESS_REFRESH_INTERVAL_MS: u32 = 1000;

/// Wire up all event handlers for the window visual styles page.
///
/// Must be called once during [`WindowVisualStylesPage::new`], after all controls
/// are constructed but before the message loop starts.
pub(super) fn setup_all_events(
    tab_page: &gui::TabPage,
    edit: &gui::Edit,
    listview: &gui::ListView,
) {
    let process_manager = Rc::new(RefCell::new(ProcessManager::new()));

    tab_layout::paint_tab_page_background(tab_page);
    setup_resize_event(tab_page, edit, listview);
    setup_page_initialization_event(tab_page, edit, listview, &process_manager);
    setup_timer_refresh_event(tab_page, listview, &process_manager);
}

// ---------------------------------------------------------------------------
// Page Initialization (Single WM_CREATE Handler)
// ---------------------------------------------------------------------------

/// Register the unified `WM_CREATE` handler on the tab page.
///
/// All initialization steps that require a valid HWND are coordinated here in a
/// single closure to prevent multiple `wm_create` registrations from overwriting each other:
/// 1. Set the edit control's cue banner (placeholder).
/// 2. Fetch the initial snapshot of system processes and populate the ListView.
/// 3. Start the periodic 1-second Win32 timer for ongoing background refreshes.
fn setup_page_initialization_event(
    tab_page: &gui::TabPage,
    edit: &gui::Edit,
    listview: &gui::ListView,
    process_manager: &Rc<RefCell<ProcessManager>>,
) {
    let cloned_edit = edit.clone();
    let cloned_listview = listview.clone();
    let cloned_tab_page = tab_page.clone();
    let cloned_process_manager = process_manager.clone();

    tab_page.on().wm_create(move |_| {
        // Step 1: Set cue banner for the edit input field.
        unsafe {
            cloned_edit
                .hwnd()
                .SendMessage(msg::EmSetCueBanner {
                    show_even_with_focus: false,
                    text: WString::from_str(t!("COMBOBOX_CUE_BANNER")),
                })
                .ok();
        }

        // Step 2: Populate initial process list snapshot.
        let initial_processes = cloned_process_manager.borrow_mut().fetch_sorted_processes();
        apply_process_list_to_view(&cloned_listview, &initial_processes)?;

        // Step 3: Start auto-refresh timer.
        cloned_tab_page.hwnd().SetTimer(
            PROCESS_REFRESH_TIMER_ID,
            PROCESS_REFRESH_INTERVAL_MS,
            None,
        )?;

        Ok(0)
    });
}

// ---------------------------------------------------------------------------
// Periodic Timer Refresh
// ---------------------------------------------------------------------------

/// Register the `WM_TIMER` handler that periodically synchronizes the process list.
fn setup_timer_refresh_event(
    tab_page: &gui::TabPage,
    listview: &gui::ListView,
    process_manager: &Rc<RefCell<ProcessManager>>,
) {
    let cloned_listview = listview.clone();
    let cloned_process_manager = process_manager.clone();

    tab_page.on().wm_timer(PROCESS_REFRESH_TIMER_ID, move || {
        let updated_processes = cloned_process_manager.borrow_mut().fetch_sorted_processes();
        apply_process_list_to_view(&cloned_listview, &updated_processes)?;
        Ok(())
    });
}

// ---------------------------------------------------------------------------
// Resize
// ---------------------------------------------------------------------------

/// Register the `WM_SIZE` handler that repositions all controls on the page.
///
/// Both controls are repositioned and resized dynamically whenever the window size changes.
/// The edit stays at the top with fixed height, and the listview fills all remaining
/// vertical space below it, maintaining consistent margins on all sides.
fn setup_resize_event(tab_page: &gui::TabPage, edit: &gui::Edit, listview: &gui::ListView) {
    let cloned_edit = edit.clone();
    let cloned_listview = listview.clone();

    tab_page.on().wm_size(move |size_info| {
        let ideal_edit_height = super::layout::calculate_edit_ideal_height(cloned_edit.hwnd());
        let window_visual_styles_page_layout = WindowVisualStylesPageLayout::calculate(
            size_info.client_area.cx,
            size_info.client_area.cy,
            ideal_edit_height,
        );

        tab_layout::reposition_and_resize_control(
            cloned_edit.hwnd(),
            window_visual_styles_page_layout.edit_position,
            window_visual_styles_page_layout.edit_size,
        )?;

        tab_layout::reposition_and_resize_control(
            cloned_listview.hwnd(),
            window_visual_styles_page_layout.listview_position,
            window_visual_styles_page_layout.listview_size,
        )?;

        apply_dynamic_column_widths(
            &cloned_listview,
            window_visual_styles_page_layout.listview_size.cx,
        )?;

        Ok(())
    });
}

/// Apply dynamically computed column widths to the ListView.
fn apply_dynamic_column_widths(
    listview: &gui::ListView,
    listview_width: i32,
) -> winsafe::AnyResult<()> {
    let column_widths = calculate_process_listview_column_widths(listview_width);
    listview
        .cols()
        .get(0)
        .set_width(column_widths.process_name_column_width)?;
    listview
        .cols()
        .get(1)
        .set_width(column_widths.process_id_column_width)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Process View Update Helpers
// ---------------------------------------------------------------------------

/// Apply a sorted process snapshot list to the ListView control smoothly.
///
/// Uses `WM_SETREDRAW` to prevent screen flickering during update and preserves
/// the previously selected process ID across refreshes.
fn apply_process_list_to_view(
    listview: &gui::ListView,
    processes: &[ProcessItem],
) -> winsafe::AnyResult<()> {
    let selected_process_id = get_currently_selected_process_id(listview);

    sync_listview_items(listview, processes)?;

    if let Some(target_process_id) = selected_process_id {
        restore_process_selection(listview, processes, target_process_id)?;
    }

    Ok(())
}

/// Retrieve the PID of the currently selected row in the ListView, if any.
fn get_currently_selected_process_id(listview: &gui::ListView) -> Option<u32> {
    let selected_item = listview.items().iter_selected().next()?;
    let pid_text = selected_item.text(1);
    pid_text.parse::<u32>().ok()
}

/// Synchronize the items in the ListView with the new list of processes.
///
/// Updates existing rows in place, appends new rows if the process count increased,
/// or truncates excess rows if the count decreased.
fn sync_listview_items(
    listview: &gui::ListView,
    processes: &[ProcessItem],
) -> winsafe::AnyResult<()> {
    let current_item_count = listview.items().count() as usize;
    let target_item_count = processes.len();

    let reusable_row_count = current_item_count.min(target_item_count);

    // 1. Update existing rows in place.
    for index in 0..reusable_row_count {
        let item_handle = listview.items().get(index as u32);
        let process_item = &processes[index];

        if item_handle.text(0) != process_item.process_name {
            item_handle.set_text(0, &process_item.process_name)?;
        }

        let process_id = process_item.process_id.to_string();
        if item_handle.text(1) != process_id {
            item_handle.set_text(1, &process_id)?;
        }
    }

    // 2. Add extra rows if new processes were spawned.
    if target_item_count > current_item_count {
        for process_item in &processes[current_item_count..] {
            let process_id = process_item.process_id.to_string();
            listview
                .items()
                .add(&[&process_item.process_name, &process_id], None, ())?;
        }
    }

    // 3. Delete trailing rows if processes exited.
    if target_item_count < current_item_count {
        for index in (target_item_count..current_item_count).rev() {
            let item_handle = listview.items().get(index as u32);
            item_handle.delete()?;
        }
    }

    Ok(())
}

/// Restore row selection by matching against the preserved PID.
fn restore_process_selection(
    listview: &gui::ListView,
    processes: &[ProcessItem],
    target_process_id: u32,
) -> winsafe::AnyResult<()> {
    let target_index = processes
        .iter()
        .position(|process| process.process_id == target_process_id);

    for selected_item in listview.items().iter_selected() {
        selected_item.select(false)?;
    }

    if let Some(target_index) = target_index {
        let item_handle = listview.items().get(target_index as u32);
        item_handle.select(true)?;
    }

    Ok(())
}
