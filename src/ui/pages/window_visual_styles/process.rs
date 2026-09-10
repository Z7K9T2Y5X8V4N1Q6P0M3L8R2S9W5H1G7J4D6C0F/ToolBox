//! System process information retrieval, filtering, and sorting.
//!
//! Owns the [`sysinfo::System`] instance and provides utilities to fetch,
//! filter by name or PID, sort, and format the current list of running
//! processes for display in the UI.

use std::cmp::Ordering;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

/// The column by which the process list can be sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    /// Sort processes by their executable name.
    ProcessName,
    /// Sort processes by their process identifier (PID).
    ProcessId,
}

impl SortColumn {
    /// Convert a zero-based column index from a ListView header click into a [`SortColumn`].
    ///
    /// Returns `None` if the column index is out of bounds.
    pub fn from_column_index(column_index: usize) -> Option<Self> {
        match column_index {
            0 => Some(Self::ProcessName),
            1 => Some(Self::ProcessId),
            _ => None,
        }
    }

    /// Return the zero-based column index corresponding to this column variant.
    pub fn to_column_index(&self) -> usize {
        match self {
            Self::ProcessName => 0,
            Self::ProcessId => 1,
        }
    }
}

/// The direction of the sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order (A to Z or smaller PID to larger PID).
    Ascending,
    /// Descending order (Z to A or larger PID to smaller PID).
    Descending,
}

impl SortDirection {
    /// Toggle the current sort direction to its inverse.
    pub fn inverted(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// Active sort configuration for the process list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessSortConfig {
    pub column: SortColumn,
    pub direction: SortDirection,
}

impl Default for ProcessSortConfig {
    fn default() -> Self {
        Self {
            column: SortColumn::ProcessName,
            direction: SortDirection::Ascending,
        }
    }
}

impl ProcessSortConfig {
    /// Switch or toggle sort order when a column header is clicked.
    ///
    /// If the clicked column is already the active sort column, its direction is toggled.
    /// If a different column is clicked, it becomes active and resets to [`SortDirection::Ascending`].
    pub fn toggle_column(&mut self, clicked_column: SortColumn) {
        if self.column == clicked_column {
            self.direction = self.direction.inverted();
        } else {
            self.column = clicked_column;
            self.direction = SortDirection::Ascending;
        }
    }
}

/// A snapshot of a single running system process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessItem {
    pub process_id: u32,
    pub process_name: String,
}

/// Manages system process querying with persistent state for incremental updates.
pub struct ProcessManager {
    system_monitor: System,
    sort_config: ProcessSortConfig,
    search_filter: String,
}

impl ProcessManager {
    /// Create a new process manager with an initialized system monitor and default sorting.
    pub fn new() -> Self {
        Self {
            system_monitor: System::new(),
            sort_config: ProcessSortConfig::default(),
            search_filter: String::new(),
        }
    }

    /// Retrieve the current sort configuration.
    pub fn current_sort_config(&self) -> ProcessSortConfig {
        self.sort_config
    }

    /// Update the active search filter query.
    pub fn set_search_filter(&mut self, search_query: &str) {
        self.search_filter = search_query.trim().to_lowercase();
    }

    /// Update the sort configuration based on a clicked column.
    pub fn toggle_sort_by_column(&mut self, clicked_column: SortColumn) {
        self.sort_config.toggle_column(clicked_column);
    }

    /// Refresh the current process list and return the sorted and filtered snapshots.
    pub fn fetch_sorted_processes(&mut self) -> Vec<ProcessItem> {
        self.system_monitor.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing(),
        );

        let is_filter_active = !self.search_filter.is_empty();

        let mut process_list: Vec<ProcessItem> = self
            .system_monitor
            .processes()
            .iter()
            .filter_map(|(process_id, process)| {
                let process_id_number = process_id.as_u32();
                let process_name_string = process.name().to_string_lossy().to_string();

                if is_filter_active {
                    let is_name_matched = process_name_string
                        .to_lowercase()
                        .contains(&self.search_filter);
                    let is_pid_matched =
                        process_id_number.to_string().contains(&self.search_filter);

                    if !is_name_matched && !is_pid_matched {
                        return None;
                    }
                }

                Some(ProcessItem {
                    process_id: process_id_number,
                    process_name: process_name_string,
                })
            })
            .collect();

        sort_process_items(&mut process_list, self.sort_config);

        process_list
    }
}

/// Sort the given list of process items according to the provided configuration.
fn sort_process_items(process_items: &mut [ProcessItem], sort_config: ProcessSortConfig) {
    process_items.sort_by(|first_process, second_process| {
        let comparison_result = match sort_config.column {
            SortColumn::ProcessName => compare_by_process_name(first_process, second_process),
            SortColumn::ProcessId => compare_by_process_id(first_process, second_process),
        };

        match sort_config.direction {
            SortDirection::Ascending => comparison_result,
            SortDirection::Descending => comparison_result.reverse(),
        }
    });
}

/// Compare two processes by their name case-insensitively, breaking ties by process ID.
fn compare_by_process_name(first_process: &ProcessItem, second_process: &ProcessItem) -> Ordering {
    let name_comparison = first_process
        .process_name
        .to_lowercase()
        .cmp(&second_process.process_name.to_lowercase());

    if name_comparison == Ordering::Equal {
        first_process.process_id.cmp(&second_process.process_id)
    } else {
        name_comparison
    }
}

/// Compare two processes by their numeric ID ascending.
///
/// Process IDs are guaranteed to be unique across all running processes at any given instant.
fn compare_by_process_id(first_process: &ProcessItem, second_process: &ProcessItem) -> Ordering {
    first_process.process_id.cmp(&second_process.process_id)
}
