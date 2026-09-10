//! Window visual styles page event registration.
//!
//! [`setup_all_events`] is the single entry point called from [`WindowVisualStylesPage::new`].
//! It wires up every event handler for the window visual styles page in the correct order.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use rust_i18n::t;
use winsafe::{
    GetCursorPos, HIMAGELIST, LVHITTESTINFO, POINT, SIZE, TRACKMOUSEEVENT, TrackMouseEvent,
    WString, co, gui, msg, prelude::*,
};

use super::layout::{
    WindowVisualStylesPageLayout, calculate_listview_usable_column_width,
    calculate_process_listview_column_widths, update_listview_header_sort_indicator,
};
use super::menu::{
    IDM_VISUAL_STYLES_APPLY_BASIC, IDM_VISUAL_STYLES_APPLY_CLASSIC, show_process_context_menu,
};
use super::process::{ProcessItem, ProcessManager, SortColumn};
use crate::ui::tab::layout as tab_layout;

/// Unique Win32 timer ID for refreshing the process list.
const PROCESS_REFRESH_TIMER_ID: usize = 3001;

/// Process list auto-refresh interval in milliseconds.
const PROCESS_REFRESH_INTERVAL_MS: u32 = 1000;

/// Target row height for each item in the ListView at 96 DPI.
const LISTVIEW_ROW_HEIGHT_RAW: i32 = 30;

/// Wire up all event handlers for the window visual styles page.
///
/// Must be called once during [`WindowVisualStylesPage::new`], after all controls
/// are constructed but before the message loop starts.
pub(super) fn setup_all_events(
    tab_page: &gui::TabPage,
    edit: &gui::Edit,
    listview: &gui::ListView,
    process_manager: &Rc<RefCell<ProcessManager>>,
    status_bar: &gui::StatusBar,
) {
    tab_layout::paint_tab_page_background(tab_page);
    setup_resize_event(tab_page, edit, listview);
    setup_page_initialization_event(tab_page, edit, listview, process_manager);
    setup_edit_filter_event(edit, listview, process_manager);
    setup_column_click_event(listview, process_manager);
    setup_timer_refresh_event(tab_page, listview, process_manager);
    setup_listview_hover_event(listview, status_bar);
    setup_context_menu_event(tab_page, listview);
    setup_context_menu_command_events(tab_page, listview);
}

// ---------------------------------------------------------------------------
// Context Menu Events
// ---------------------------------------------------------------------------

/// Register the right-click event on the ListView to display the context menu.
///
/// Uses WinSafe's [`msg::LvmHitTest`] to inspect the item under the cursor. If an
/// item is hit, it is highlighted and selected, and the popup menu is displayed.
/// If the user right-clicks on an empty area, no menu is shown.
fn setup_context_menu_event(tab_page: &gui::TabPage, listview: &gui::ListView) {
    let cloned_listview = listview.clone();
    let cloned_tab_page = tab_page.clone();

    listview
        .on_subclass()
        .wm_r_button_down(move |mouse_event_params| {
            let click_position_client = mouse_event_params.coords;

            if let Some(target_item_index) =
                hit_test_listview_item(&cloned_listview, click_position_client)
            {
                select_single_listview_item(&cloned_listview, target_item_index)?;

                let cursor_screen_position = GetCursorPos()?;
                show_process_context_menu(cloned_tab_page.hwnd(), cursor_screen_position)?;
            }

            Ok(())
        });
}

/// Register command handlers for context menu options.
fn setup_context_menu_command_events(tab_page: &gui::TabPage, listview: &gui::ListView) {
    let cloned_listview_for_basic = listview.clone();
    tab_page
        .on()
        .wm_command_acc_menu(IDM_VISUAL_STYLES_APPLY_BASIC, move || {
            if let Some(selected_process_id) =
                get_currently_selected_process_id(&cloned_listview_for_basic)
            {
                // Placeholder: Apply basic visual style to the target process.
                let _ = selected_process_id;
            }
            Ok(())
        });

    let cloned_listview_for_classic = listview.clone();
    tab_page
        .on()
        .wm_command_acc_menu(IDM_VISUAL_STYLES_APPLY_CLASSIC, move || {
            if let Some(selected_process_id) =
                get_currently_selected_process_id(&cloned_listview_for_classic)
            {
                // Placeholder: Apply classic visual style to the target process.
                let _ = selected_process_id;
            }
            Ok(())
        });
}

/// Perform a hit test on the ListView at the given client-relative coordinates using WinSafe.
///
/// Returns the zero-based index of the item under the cursor if hit, or `None` if
/// the click was outside any item (e.g. empty background space).
fn hit_test_listview_item(listview: &gui::ListView, client_coords: POINT) -> Option<u32> {
    let mut hit_test_info = LVHITTESTINFO::default();
    hit_test_info.pt = client_coords;

    let hit_test_index = unsafe {
        listview.hwnd().SendMessage(msg::LvmHitTest {
            info: &mut hit_test_info,
        })
    };

    let is_on_item = hit_test_info.flags.has(co::LVHT::ONITEM);
    if is_on_item { hit_test_index } else { None }
}

/// Unselect all currently selected rows and select the item at the specified index.
fn select_single_listview_item(
    listview: &gui::ListView,
    item_index: u32,
) -> winsafe::AnyResult<()> {
    for selected_item in listview.items().iter_selected() {
        selected_item.select(false)?;
    }

    let target_item = listview.items().get(item_index);
    target_item.select(true)?;
    target_item.focus()?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Page Initialization (Single WM_CREATE Handler)
// ---------------------------------------------------------------------------

/// Register the unified `WM_CREATE` handler on the tab page.
///
/// All initialization steps that require a valid HWND are coordinated here in a
/// single closure to prevent multiple `wm_create` registrations from overwriting each other:
/// 1. Configure the ListView item height by binding a custom-dimension dummy ImageList.
/// 2. Set the edit control's cue banner (placeholder).
/// 3. Fetch the initial snapshot of system processes and populate the ListView.
/// 4. Display the initial sorting arrow on the header.
/// 5. Ensure column widths are synchronized with visible vertical scrollbar.
/// 6. Start the periodic 1-second Win32 timer for ongoing background refreshes.
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
        // Step 1: Expand row height using a DPI-scaled dummy ImageList.
        apply_custom_row_height(&cloned_listview)?;

        // Step 2: Set cue banner for the edit input field.
        unsafe {
            cloned_edit
                .hwnd()
                .SendMessage(msg::EmSetCueBanner {
                    show_even_with_focus: false,
                    text: WString::from_str(t!("COMBOBOX_CUE_BANNER")),
                })
                .ok();
        }

        // Step 3: Populate initial process list snapshot.
        let mut borrowed_process_manager = cloned_process_manager.borrow_mut();
        let initial_processes = borrowed_process_manager.fetch_sorted_processes();
        apply_process_list_to_view(&cloned_listview, &initial_processes)?;

        // Step 4: Apply the initial sorting arrow on the header.
        update_listview_header_sort_indicator(
            cloned_listview.hwnd(),
            borrowed_process_manager.current_sort_config(),
        );

        // Step 5: Ensure column widths are synchronized after items are populated.
        apply_dynamic_column_widths(&cloned_listview)?;

        // Step 6: Start auto-refresh timer.
        cloned_tab_page.hwnd().SetTimer(
            PROCESS_REFRESH_TIMER_ID,
            PROCESS_REFRESH_INTERVAL_MS,
            None,
        )?;

        Ok(0)
    });
}

/// Enlarge the row height of the ListView control to a modern visual spacing.
///
/// Win32 ListView items in report mode derive their vertical height from either the
/// font line-height or the attached `LVSIL_SMALL` ImageList. Attaching a 1-pixel-wide
/// ImageList with a DPI-scaled height of 30px stretches the entire row without
/// requiring custom drawing.
fn apply_custom_row_height(listview: &gui::ListView) -> winsafe::AnyResult<()> {
    let row_height = gui::dpi_y(LISTVIEW_ROW_HEIGHT_RAW);
    let dummy_image_size = SIZE::with(1, row_height);

    // Create the dummy ImageList with the specified DPI-scaled height and leak the guard
    // so the control can take permanent ownership of the handle.
    let image_list = HIMAGELIST::Create(dummy_image_size, co::ILC::COLOR32, 0, 0)?.leak();

    unsafe {
        listview.hwnd().SendMessage(msg::LvmSetImageList {
            kind: co::LVSIL::SMALL,
            himagelist: Some(image_list.raw_copy()),
        });
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Edit Filter Change Event
// ---------------------------------------------------------------------------

/// Register the `EN_CHANGE` event on the edit control to filter the ListView.
///
/// When the user inputs or clears text:
/// 1. The input string is read from the edit control.
/// 2. The search filter is updated in [`ProcessManager`].
/// 3. The process snapshot is filtered and refreshed immediately in the ListView.
fn setup_edit_filter_event(
    edit: &gui::Edit,
    listview: &gui::ListView,
    process_manager: &Rc<RefCell<ProcessManager>>,
) {
    let cloned_edit = edit.clone();
    let cloned_listview = listview.clone();
    let cloned_process_manager = process_manager.clone();

    edit.on().en_change(move || {
        let filter_keyword = cloned_edit.hwnd().GetWindowText()?;
        let mut borrowed_process_manager = cloned_process_manager.borrow_mut();
        borrowed_process_manager.set_search_filter(&filter_keyword);

        let filtered_processes = borrowed_process_manager.fetch_sorted_processes();
        apply_process_list_to_view(&cloned_listview, &filtered_processes)?;

        Ok(())
    });
}

// ---------------------------------------------------------------------------
// Column Click Sorting
// ---------------------------------------------------------------------------

/// Register the `LVN_COLUMNCLICK` notification handler on the ListView control.
///
/// When the user clicks a column header:
/// 1. The sort configuration is toggled (switches column or inverts sort direction).
/// 2. The processes are re-sorted according to the new configuration.
/// 3. The ListView rows are immediately updated to reflect the new order.
/// 4. The header indicator arrow is updated to reflect the active sort column and direction.
fn setup_column_click_event(
    listview: &gui::ListView,
    process_manager: &Rc<RefCell<ProcessManager>>,
) {
    let cloned_listview = listview.clone();
    let cloned_process_manager = process_manager.clone();

    listview.on().lvn_column_click(move |column_click_info| {
        let clicked_column_index = column_click_info.iSubItem as usize;

        if let Some(sort_column) = SortColumn::from_column_index(clicked_column_index) {
            let mut borrowed_process_manager = cloned_process_manager.borrow_mut();
            borrowed_process_manager.toggle_sort_by_column(sort_column);

            let current_sort_config = borrowed_process_manager.current_sort_config();
            let updated_processes = borrowed_process_manager.fetch_sorted_processes();

            apply_process_list_to_view(&cloned_listview, &updated_processes)?;
            update_listview_header_sort_indicator(cloned_listview.hwnd(), current_sort_config);
        }

        Ok(())
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
// Hover Description (Status Bar)
// ---------------------------------------------------------------------------

/// Register mouse hover and leave events on the ListView to update the status bar text.
fn setup_listview_hover_event(listview: &gui::ListView, status_bar: &gui::StatusBar) {
    let is_mouse_inside = Rc::new(Cell::new(false));

    let cloned_status_bar_for_mouse_move = status_bar.clone();
    let cloned_listview_for_mouse_move = listview.clone();
    let cloned_is_mouse_inside_for_mouse_move = is_mouse_inside.clone();

    listview.on_subclass().wm_mouse_move(move |_| {
        if !cloned_is_mouse_inside_for_mouse_move.get() {
            cloned_is_mouse_inside_for_mouse_move.set(true);

            let hover_description_text = t!("STATUS_BAR_LISTVIEW_HINT");
            cloned_status_bar_for_mouse_move
                .parts()
                .set_texts(&[Some(hover_description_text.as_ref())]);

            // Request WM_MOUSELEAVE notification to reset status bar when mouse exits control bounds.
            let mut track_mouse_event_info = TRACKMOUSEEVENT::default();
            track_mouse_event_info.dwFlags = co::TME::LEAVE;
            track_mouse_event_info.hwndTrack =
                unsafe { winsafe::HWND::from_ptr(cloned_listview_for_mouse_move.hwnd().ptr()) };
            TrackMouseEvent(&mut track_mouse_event_info)?;
        }

        Ok(())
    });

    let cloned_is_mouse_inside_for_mouse_leave = is_mouse_inside;
    let cloned_status_bar_for_mouse_leave = status_bar.clone();
    listview.on_subclass().wm_mouse_leave(move || {
        cloned_is_mouse_inside_for_mouse_leave.set(false);
        cloned_status_bar_for_mouse_leave
            .parts()
            .set_texts(&[Some("")]);
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

        apply_dynamic_column_widths(&cloned_listview)?;

        Ok(())
    });
}

/// Apply dynamically computed column widths to the ListView.
fn apply_dynamic_column_widths(listview: &gui::ListView) -> winsafe::AnyResult<()> {
    let usable_column_width = calculate_listview_usable_column_width(listview.hwnd());
    let column_widths = calculate_process_listview_column_widths(usable_column_width);
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
/// Uses in-place cell updates to prevent screen flickering during update and preserves
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
    let target_process_row_index = processes
        .iter()
        .position(|process| process.process_id == target_process_id);

    for selected_item in listview.items().iter_selected() {
        selected_item.select(false)?;
    }

    if let Some(target_process_row_index) = target_process_row_index {
        let item_handle = listview.items().get(target_process_row_index as u32);
        item_handle.select(true)?;
        item_handle.focus()?;
    }

    Ok(())
}
