use ratatui::widgets::TableState;

use crate::{
    actions::TermiAction,
    db::{LeaderboardQuery, LeaderboardResult, SortOrder, TermiDB, TypingTestResult},
    log_debug,
};

pub enum LoadType {
    Initial, // first load
    More,    // load more data
    Refresh, // sort changes
}

#[derive(Default, Debug, Clone)]
pub struct Leaderboard {
    is_open: bool,
    is_loading: bool,
    err_msg: Option<String>,
    results: Option<LeaderboardResult>,
    items: Vec<TypingTestResult>,
    query: LeaderboardQuery,
    table: TableState,
}

impl Leaderboard {
    pub fn new() -> Self {
        let mut state = Self::default();
        state.table.select(Some(0));
        state
    }

    pub fn open(&mut self, db: &TermiDB) {
        self.is_open = true;
        self.load(db, LoadType::Initial);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.clear();
    }

    pub fn toggle(&mut self, db: &TermiDB) {
        if self.is_open {
            self.close()
        } else {
            self.open(db)
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn items(&self) -> &[TypingTestResult] {
        &self.items
    }

    pub fn table(&mut self) -> &mut TableState {
        &mut self.table
    }

    pub fn sort_col(&self) -> &str {
        &self.query.sort_col
    }

    pub fn sort_order(&self) -> &SortOrder {
        &self.query.sort_order
    }

    pub fn error_message(&self) -> Option<&str> {
        self.err_msg.as_deref()
    }

    pub fn has_results(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn count(&self) -> usize {
        self.results.as_ref().map(|r| r.total_count).unwrap_or(0)
    }

    pub fn current_sort_col_idx(&self) -> usize {
        let cols = get_sortable_columns();
        cols.iter()
            .position(|(col, _)| *col == self.query.sort_col)
            .unwrap_or(0)
    }

    pub fn handle_action(&mut self, action: TermiAction, db: &TermiDB) -> Option<TermiAction> {
        match action {
            TermiAction::LeaderboardOpen => {
                self.open(db);
                None
            }
            TermiAction::LeaderboardClose => {
                self.close();
                None
            }
            TermiAction::LeaderboardInput(leaderboard_action) => match leaderboard_action {
                crate::actions::LeaderboardAction::NavigateUp => {
                    self.up();
                    None
                }
                crate::actions::LeaderboardAction::NavigateDown => {
                    self.down(db);
                    None
                }
                crate::actions::LeaderboardAction::SortBy(idx) => {
                    self.sort_by_column(idx, db);
                    None
                }
            },
            _ => None,
        }
    }

    // TODO: this might need to be broken down
    pub fn load(&mut self, db: &TermiDB, load_type: LoadType) {
        if matches!(load_type, LoadType::More) && self.is_loading {
            return;
        }

        match load_type {
            LoadType::Initial | LoadType::Refresh => {
                self.query.offset = 0;
                self.query.limit = 25; // TODO: magic number
                self.items.clear();
            }
            LoadType::More => {
                self.is_loading = true;
                self.query.limit = 25;
                self.query.offset = self.items.len(); // QUESTION: is this correct?
            }
        }

        match db.query_leaderboard(&self.query) {
            Ok(res) => {
                let count = res.results.len();
                match load_type {
                    LoadType::Initial | LoadType::Refresh => {
                        self.items = res.results.clone();
                    }
                    LoadType::More => {
                        self.items.extend(res.results);
                    }
                }

                self.results = Some(LeaderboardResult {
                    has_more: res.has_more,
                    total_count: res.total_count,
                    results: self.items.clone(),
                });

                self.err_msg = None;

                if !matches!(load_type, LoadType::More) {
                    if self.items.is_empty() {
                        self.table.select(None);
                    } else if self.table.selected().is_none() {
                        self.table.select(Some(0))
                    }
                }
                match load_type {
                    LoadType::Initial => {
                        log_debug!("Loaded {} initial items", self.items.len())
                    }
                    LoadType::More => {
                        log_debug!("Loaded {} more items, total: {}", count, self.items.len())
                    }
                    LoadType::Refresh => log_debug!("Refreshed {} items", self.items.len()),
                }
            }
            Err(err) => {
                let msg = match load_type {
                    LoadType::Initial => "Failed to laod leaderboard",
                    LoadType::More => "Failed to load more data",
                    LoadType::Refresh => "Failed to refresh leaderboard",
                };
                self.err_msg = Some(format!("{}: {}", msg, err));

                if matches!(load_type, LoadType::Initial) {
                    self.results = None;
                    self.items.clear();
                }

                log_debug!("{}: {}", msg, err);
            }
        }

        if matches!(load_type, LoadType::More) {
            self.is_loading = false;
        }
    }

    fn up(&mut self) {
        if !self.items.is_empty() {
            let selected = self.table.selected().unwrap_or(0);
            if selected > 0 {
                self.table.select(Some(selected - 1));
            }
        }
    }

    fn down(&mut self, db: &TermiDB) {
        if !self.items.is_empty() {
            let selected = self.table.selected().unwrap_or(0);
            let max_idx = self.items.len() - 1;
            let scroll_offset = 5;
            if selected >= max_idx.saturating_sub(scroll_offset) && !self.is_loading {
                if let Some(results) = &self.results {
                    if results.has_more {
                        self.load(db, LoadType::More);
                    }
                }
            }

            if selected < max_idx {
                self.table.select(Some(selected + 1));
            }
        }
    }

    fn sort_by_column(&mut self, idx: usize, db: &TermiDB) {
        let cols = get_sortable_columns();
        if idx < cols.len() {
            let (name, _) = cols[idx];
            if self.current_sort_col_idx() == idx {
                self.toggle_sort(db);
            } else {
                self.query.sort_col = name.to_string();
                self.query.sort_order = SortOrder::Descending;
                self.reset(db);
            }
        }
    }

    fn toggle_sort(&mut self, db: &TermiDB) {
        self.query.sort_order = match self.query.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
        self.reset(db);
    }

    fn reset(&mut self, db: &TermiDB) {
        self.query.offset = 0;
        self.items.clear();
        self.table.select(Some(0));
        self.load(db, LoadType::Initial);
    }

    fn clear(&mut self) {
        self.items.clear();
        self.table = TableState::default();
        self.table.select(Some(0));
        self.query.offset = 0;
        self.results = None;
        self.err_msg = None;
        self.is_loading = false;
    }
}

pub fn get_sortable_columns() -> Vec<(&'static str, &'static str)> {
    vec![
        ("wpm", "WPM"),
        ("raw_wpm", "Raw"),
        ("accuracy", "Accuracy"),
        ("consistency", "Consistency"),
        ("total_keystrokes", "Chars"),
        ("mode_type", "Mode"),
        ("created_at", "Date"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_db() -> (TermiDB, TempDir) {
        let _guard = ENV_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        if cfg!(target_os = "macos") {
            std::env::set_var("HOME", &temp_path);
        } else if cfg!(target_os = "windows") {
            std::env::set_var("APPDATA", &temp_path);
        } else {
            std::env::set_var("XDG_CONFIG_HOME", &temp_path);
        }
        let db = TermiDB::new().expect("Failed to create test database");
        (db, temp_dir)
    }

    #[test]
    fn test_leaderboard_setup() {
        let leaderboard = Leaderboard::new();
        assert!(!leaderboard.is_open());
        assert!(leaderboard.items().is_empty());
        assert_eq!(leaderboard.count(), 0);
    }

    #[test]
    fn test_leaderboard_toggle() {
        let (db, _) = setup_db();
        let mut leaderboard = Leaderboard::new();

        assert!(!leaderboard.is_open());
        leaderboard.open(&db);
        assert!(leaderboard.is_open());
        leaderboard.close();
        assert!(!leaderboard.is_open());
        leaderboard.toggle(&db);
        assert!(leaderboard.is_open());
        leaderboard.toggle(&db);
        assert!(!leaderboard.is_open());
    }
}
