use crate::db::{
    Db, LeaderboardColumn, LeaderboardQuery, LeaderboardResult, LeaderboardState, SortOrder,
};
use ratatui::widgets::TableState;

#[derive(Debug, Clone, PartialEq)]
pub enum SortColumn {
    Mode,
    Language,
    Wpm,
    RawWpm,
    Accuracy,
    Consistency,
    ErrorCount,
    CreatedAt,
}

impl SortColumn {
    /// Converts `SortColumn` into a `LeaderboardColumn` compatible value
    pub fn to_value(&self) -> &'static str {
        match self {
            SortColumn::Mode => "mode_kind",
            SortColumn::Language => "language",
            SortColumn::Wpm => "wpm",
            SortColumn::RawWpm => "raw_wpm",
            SortColumn::Accuracy => "accuracy",
            SortColumn::Consistency => "consistency",
            SortColumn::ErrorCount => "error_count",
            SortColumn::CreatedAt => "created_at",
        }
    }

    /// Converts a `SortColumn` into a readable title
    pub fn to_display(&self) -> &'static str {
        match self {
            SortColumn::Mode => "Mode",
            SortColumn::Language => "Language",
            SortColumn::Wpm => "WPM",
            SortColumn::RawWpm => "Raw",
            SortColumn::Accuracy => "Accuracy",
            SortColumn::Consistency => "Consistency",
            SortColumn::ErrorCount => "Errors",
            SortColumn::CreatedAt => "Date",
        }
    }

    /// Returns all the available `SortColumn`
    pub fn all() -> Vec<SortColumn> {
        vec![
            SortColumn::Mode,
            SortColumn::Language,
            SortColumn::Wpm,
            SortColumn::RawWpm,
            SortColumn::Accuracy,
            SortColumn::Consistency,
            SortColumn::ErrorCount,
            SortColumn::CreatedAt,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LeaderboardMotion {
    Up,
    Down,
    // PageUp,
    // PageDown,
    Home,
    End,
}

pub enum LoadType {
    /// First load
    Initial,
    /// Load more data
    More,
    /// Reload current data
    Refresh,
}

#[derive(Default, Debug, Clone)]
pub enum LeaderboardStatus {
    Open,
    #[default]
    Closed,
    Loading,
    Error(String),
}

#[derive(Default, Debug, Clone)]
pub struct Leaderboard {
    status: LeaderboardStatus,
    query: LeaderboardQuery,
    pub state: Option<LeaderboardState>,
    pub table: TableState,
}

impl Leaderboard {
    pub fn new() -> Self {
        let mut leaderboard = Self::default();
        leaderboard.table.select(Some(0));
        leaderboard
    }

    pub fn open(&mut self, db: &Db) {
        self.status = LeaderboardStatus::Closed;
        self.load(db, LoadType::Initial);
        self.table.select(Some(0));
    }

    pub fn close(&mut self) {
        self.clear();
    }

    pub fn toggle(&mut self, db: &Db) {
        if self.is_open() {
            self.close();
        } else {
            self.open(db);
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self.status, LeaderboardStatus::Open)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.status, LeaderboardStatus::Loading)
    }

    fn clear(&mut self) {
        self.status = LeaderboardStatus::Closed;
        self.state = None;
        self.query.offset = 0;
        self.table = TableState::default();
    }

    pub fn data(&self) -> &[LeaderboardResult] {
        if let Some(state) = &self.state {
            &state.data
        } else {
            &[]
        }
    }

    pub fn is_empty(&self) -> bool {
        self.state.as_ref().is_some_and(|s| s.data.is_empty())
    }

    pub fn has_more(&self) -> bool {
        self.state.as_ref().is_some_and(|s| s.has_more)
    }

    fn load(&mut self, db: &Db, load_type: LoadType) {
        if matches!(load_type, LoadType::More) && !self.has_more() {
            return;
        }

        self.status = LeaderboardStatus::Loading;

        match load_type {
            LoadType::Initial | LoadType::Refresh => self.query.offset = 0,
            LoadType::More => {
                if let Some(state) = &self.state {
                    self.query.offset = state.data.len();
                } else {
                    self.query.offset = 0;
                }
            }
        }

        match db.query_data(&self.query) {
            Ok(new_state) => {
                match load_type {
                    LoadType::Initial | LoadType::Refresh => {
                        self.state = Some(new_state);
                    }
                    LoadType::More => {
                        if let Some(existing) = &mut self.state {
                            existing.data.extend(new_state.data);
                            existing.has_more = new_state.has_more;
                            existing.count = new_state.count;
                        } else {
                            self.state = Some(new_state);
                        }
                    }
                }
                self.status = LeaderboardStatus::Open;
            }
            Err(e) => {
                self.status = LeaderboardStatus::Error(e.to_string());
            }
        }
    }

    pub fn navigate(&mut self, db: &Db, motion: LeaderboardMotion) {
        let data_len = self.data().len();
        if data_len == 0 {
            return;
        }

        let current = self.table.selected().unwrap_or(0);
        let new_idx = match motion {
            LeaderboardMotion::Up => current.saturating_sub(1),
            LeaderboardMotion::Down => {
                let next = current + 1;
                if next >= data_len && self.has_more() {
                    self.load(db, LoadType::More);
                    next.min(self.data().len().saturating_sub(1))
                } else {
                    next.min(data_len.saturating_sub(1))
                }
            }
            LeaderboardMotion::Home => 0,
            LeaderboardMotion::End => data_len.saturating_sub(1),
        };

        self.table.select(Some(new_idx));
    }

    pub fn sort(&mut self, col: SortColumn, db: &Db) {
        let target_column = match col {
            SortColumn::Mode => LeaderboardColumn::ModeKind,
            SortColumn::Language => LeaderboardColumn::Language,
            SortColumn::Wpm => LeaderboardColumn::Wpm,
            SortColumn::RawWpm => LeaderboardColumn::RawWpm,
            SortColumn::Accuracy => LeaderboardColumn::Accuracy,
            SortColumn::Consistency => LeaderboardColumn::Consistency,
            SortColumn::ErrorCount => LeaderboardColumn::ErrorCount,
            SortColumn::CreatedAt => LeaderboardColumn::CreatedAt,
        };

        let current_col_str = self.query.sort_by.to_value();
        let target_col_str = target_column.to_value();

        if current_col_str == target_col_str {
            self.query.sort_order = match self.query.sort_order {
                SortOrder::Descending => SortOrder::Ascending,
                SortOrder::Ascending => SortOrder::Descending,
            };
        } else {
            self.query.sort_by = target_column;
            self.query.sort_order = SortOrder::Descending;
        }

        self.load(db, LoadType::Refresh);
        self.table.select(Some(0));
    }

    pub fn current_sort(&self) -> (&LeaderboardColumn, &SortOrder) {
        (&self.query.sort_by, &self.query.sort_order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Db {
        Db::new_in_memory().expect("Failed to create test database")
    }

    #[test]
    fn test_leaderboard_setup() {
        let leaderboard = Leaderboard::new();
        assert!(!leaderboard.is_open());
        assert!(leaderboard.data().is_empty());
        assert_eq!(leaderboard.data().len(), 0);
    }

    #[test]
    fn test_leaderboard_load_initial() {
        let mut db = create_test_db();
        db.reset().unwrap();
        db.insert_test_result("time", 60, "english", 80, 95);
        db.insert_test_result("words", 25, "english", 70, 90);

        let mut leaderboard = Leaderboard::new();
        leaderboard.open(&db);

        assert!(leaderboard.is_open());
        assert_eq!(leaderboard.data().len(), 2);
    }

    #[test]
    fn test_leaderboard_load_empty() {
        let db = create_test_db();
        db.reset().unwrap();

        let mut leaderboard = Leaderboard::new();
        leaderboard.open(&db);

        assert!(leaderboard.is_open());
        assert!(leaderboard.data().is_empty());
    }

    #[test]
    fn test_load_more_no_state() {
        let mut db = create_test_db();
        db.reset().unwrap();
        db.insert_test_result("time", 60, "english", 80, 95);

        let mut leaderboard = Leaderboard::new();
        leaderboard.load(&db, LoadType::More);

        assert!(!leaderboard.is_open());
        assert!(leaderboard.data().is_empty());
    }

    #[test]
    fn test_load_more_when_no_more_data() {
        let mut db = create_test_db();
        db.reset().unwrap();
        db.insert_test_result("time", 60, "english", 80, 95);
        db.insert_test_result("words", 25, "english", 70, 90);

        let mut leaderboard = Leaderboard::new();
        leaderboard.open(&db);

        let initial_len = leaderboard.data().len();
        assert_eq!(initial_len, 2);

        leaderboard.load(&db, LoadType::More);

        assert_eq!(leaderboard.data().len(), initial_len);
        assert!(leaderboard.is_open());
    }

    #[test]
    fn test_offset_calculation() {
        let mut db = create_test_db();
        db.reset().unwrap();
        for i in 0..3 {
            db.insert_test_result("time", 60, "english", (80 + i) as u16, 95);
        }

        let mut leaderboard = Leaderboard::new();
        leaderboard.query.limit = 1;
        leaderboard.open(&db);

        assert_eq!(leaderboard.data().len(), 1);
        assert_eq!(leaderboard.query.offset, 0);
        assert!(leaderboard.has_more());

        leaderboard.load(&db, LoadType::More); // offset = 1

        assert_eq!(leaderboard.data().len(), 2);
        assert!(leaderboard.has_more());
        assert_eq!(leaderboard.query.offset, 1);

        leaderboard.load(&db, LoadType::More); // offset = 2
        assert_eq!(leaderboard.data().len(), 3);
        assert!(!leaderboard.has_more());
        assert_eq!(leaderboard.query.offset, 2);
    }
}
