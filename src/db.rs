use crate::{
    common::filesystem::config_dir,
    config::{Config, Setting},
    error::{AppError, AppResult},
    log_debug, log_info,
    tracker::Tracker,
};
use chrono::{DateTime, Local};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: i32 = 3;
const DEFAULT_LEADERBOARD_LIMIT: usize = 25;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResult {
    pub id: Option<i64>,
    pub mode_kind: String,
    pub mode_value: i32,
    pub language: String,
    pub wpm: u16,
    pub raw_wpm: u16,
    pub accuracy: u16,
    pub consistency: u16,
    pub error_count: u32,
    pub numbers: bool,
    pub symbols: bool,
    pub punctuation: bool,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub enum LeaderboardColumn {
    ModeKind,
    ModeValue,
    Language,
    Wpm,
    RawWpm,
    Accuracy,
    Consistency,
    ErrorCount,
    Numbers,
    Symbols,
    Punctuation,
    CreatedAt,
}

impl LeaderboardColumn {
    pub fn to_value(&self) -> &'static str {
        match self {
            LeaderboardColumn::ModeKind => "mode_kind",
            LeaderboardColumn::ModeValue => "mode_value",
            LeaderboardColumn::Language => "language",
            LeaderboardColumn::Wpm => "wpm",
            LeaderboardColumn::RawWpm => "raw_wpm",
            LeaderboardColumn::Accuracy => "accuracy",
            LeaderboardColumn::Consistency => "consistency",
            LeaderboardColumn::ErrorCount => "error_count",
            LeaderboardColumn::Numbers => "numbers",
            LeaderboardColumn::Symbols => "symbols",
            LeaderboardColumn::Punctuation => "punctuation",
            LeaderboardColumn::CreatedAt => "created_at",
        }
    }
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn to_value(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "ASC",
            SortOrder::Descending => "DESC",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LeaderboardState {
    pub count: usize,
    pub has_more: bool,
    pub data: Vec<LeaderboardResult>,
}

#[derive(Debug, Clone)]
pub struct LeaderboardQuery {
    pub limit: usize,
    pub offset: usize,
    pub sort_by: LeaderboardColumn, //  TODO: was `sort_col` must be an enum
    pub sort_order: SortOrder,
}

impl Default for LeaderboardQuery {
    fn default() -> Self {
        Self {
            limit: DEFAULT_LEADERBOARD_LIMIT,
            offset: 0,
            sort_by: LeaderboardColumn::CreatedAt,
            sort_order: SortOrder::Descending,
        }
    }
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new(filename: &str) -> AppResult<Self> {
        let dir = config_dir()?;
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        let path = dir.join(filename);
        let connection = Connection::open(&path)?;
        let mut db = Self { conn: connection };

        db.init()?;

        Ok(db)
    }

    fn init(&mut self) -> AppResult<()> {
        self.conn.execute("PRAGMA foreign_keys = ON", [])?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)",
            [],
        )?;

        let current_version: i32 = self
            .conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if current_version < SCHEMA_VERSION {
            self.create()?;
            self.conn.execute(
                "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
            log_info!("DB: schema updated to version {}", SCHEMA_VERSION);
        }

        Ok(())
    }

    fn create(&mut self) -> AppResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mode_kind TEXT NOT NULL,
                mode_value INTEGER NOT NULL,
                language TEXT NOT NULL,
                wpm REAL NOT NULL,
                raw_wpm REAL DEFAULT 0,
                accuracy INTEGER NOT NULL,
                consistency INTEGER NOT NULL,
                error_count INTEGER NOT NULL,
                numbers BOOLEAN NOT NULL,
                punctuation BOOLEAN NOT NULL,
                symbols BOOLEAN NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        self.create_indexes()?;
        log_debug!("DB: tables and indexes created successfully");

        Ok(())
    }

    fn create_indexes(&mut self) -> AppResult<()> {
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_filters ON results (
                mode_kind, mode_value, language, numbers, punctuation, symbols
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wpm ON results (wpm DESC)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_accuracy ON results (accuracy DESC)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_consistency ON results (consistency DESC)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON results (created_at DESC)",
            [],
        )?;

        Ok(())
    }

    pub fn write(&mut self, config: &Config, tracker: &Tracker) -> AppResult<i64> {
        let current_mode = config.current_mode();
        let summary = tracker.summary();
        let result = LeaderboardResult {
            id: None,
            mode_kind: current_mode.kind().to_display(),
            mode_value: current_mode.value() as i32,
            language: config.current_language(),
            wpm: summary.wpm.round() as u16,
            raw_wpm: summary.raw_wpm().round() as u16,
            accuracy: (summary.accuracy * 100.0) as u16,
            consistency: summary.consistency as u16,
            error_count: summary.total_errors as u32,
            numbers: config.is_enabled(Setting::Numbers),
            symbols: config.is_enabled(Setting::Symbols),
            punctuation: config.is_enabled(Setting::Punctuation),
            created_at: Local::now(),
        };

        self.conn.execute(
            "INSERT INTO results (
                mode_kind,
                mode_value,
                language,
                wpm,
                raw_wpm,
                accuracy,
                consistency,
                error_count,
                numbers,
                symbols,
                punctuation,
                created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                result.mode_kind,
                result.mode_value,
                result.language,
                result.wpm,
                result.raw_wpm,
                result.accuracy,
                result.consistency,
                result.error_count,
                result.numbers,
                result.symbols,
                result.punctuation,
                result.created_at
            ],
        )?;

        let id = self.conn.last_insert_rowid();

        log_debug!("DB: saved test result to database with ID: '{id}'");

        Ok(id)
    }

    pub fn reset(&self) -> AppResult<usize> {
        let affected_rows = self.conn.execute("DELETE FROM results", [])?;
        log_info!("DB: reset database, deleted {affected_rows} results");
        Ok(affected_rows)
    }

    pub fn insert_dummy_result(&mut self, result: LeaderboardResult) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO results (
                mode_kind,
                mode_value,
                language,
                wpm,
                raw_wpm,
                accuracy,
                consistency,
                error_count,
                numbers,
                symbols,
                punctuation,
                created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                result.mode_kind,
                result.mode_value,
                result.language,
                result.wpm,
                result.raw_wpm,
                result.accuracy,
                result.consistency,
                result.error_count,
                result.numbers,
                result.symbols,
                result.punctuation,
                result.created_at
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    #[cfg(test)]
    pub fn insert_test_result(
        &mut self,
        mode_kind: &str,
        mode_value: i32,
        language: &str,
        wpm: u16,
        accuracy: u16,
    ) {
        let created_at_str = "2023-10-18T12:00:00+00:00";
        self.conn.execute(
            "INSERT INTO results (mode_kind, mode_value, language, wpm, raw_wpm, accuracy, consistency, error_count, numbers, symbols, punctuation, created_at)
             VALUES (?, ?, ?, ?, 0, ?, 100, 0, 0, 0, 0, ?)",
            params![mode_kind, mode_value, language, wpm, accuracy, created_at_str],
        ).unwrap();
    }

    pub fn query_data(&self, query: &LeaderboardQuery) -> AppResult<LeaderboardState> {
        if !self.is_valid_column(&query.sort_by) {
            return Err(AppError::TermiDB(format!(
                "Invalid sort column: {}",
                query.sort_by.to_value()
            )));
        }
        let sort_direction = query.sort_order.to_value();
        let sort_col = query.sort_by.to_value();
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM results", [], |row| row.get(0))?;

        let sql_payload = format!(
            "SELECT
                id,
                mode_kind,
                mode_value,
                language,
                wpm,
                raw_wpm,
                accuracy,
                consistency,
                error_count,
                numbers,
                symbols,
                punctuation,
                created_at
              FROM results
             ORDER BY {} {}
             LIMIT {} OFFSET {}",
            sort_col, sort_direction, query.limit, query.offset
        );

        let mut statement = self.conn.prepare(&sql_payload)?;

        let results: Result<Vec<LeaderboardResult>, rusqlite::Error> = statement
            .query_map([], |row| {
                let created_at: DateTime<Local> = row.get(12)?;

                Ok(LeaderboardResult {
                    id: Some(row.get(0)?),
                    mode_kind: row.get(1)?,
                    mode_value: row.get(2)?,
                    language: row.get(3)?,
                    wpm: row.get::<_, f64>(4)?.round() as u16,
                    raw_wpm: row.get::<_, f64>(5).unwrap_or(0.0).round() as u16,
                    accuracy: row.get(6)?,
                    consistency: row.get::<_, f64>(7)?.round() as u16,
                    error_count: row.get(8)?,
                    numbers: row.get(9)?,
                    symbols: row.get(10)?,
                    punctuation: row.get(11)?,
                    created_at,
                })
            })?
            .collect();
        let results = results?;
        let has_more = query.offset + results.len() < count;

        Ok(LeaderboardState {
            count,
            has_more,
            data: results,
        })
    }

    fn is_valid_column(&self, column: &LeaderboardColumn) -> bool {
        let col = column.to_value();
        let valid_cols = [
            "mode_kind",
            "mode_value",
            "language",
            "wpm",
            "raw_wpm",
            "accuracy",
            "consistency",
            "error_count",
            "numbers",
            "punctuation",
            "symbols",
            "created_at",
        ];

        valid_cols.contains(&col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::Mode;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn create_test_db() -> Db {
        let _guard = ENV_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // NOTE: Rust 2024 edition marks as unsafe `env::set_var()`, we only use this on tests.
        // https://github.com/rust-lang/rust/pull/124636
        if cfg!(target_os = "macos") {
            unsafe { std::env::set_var("HOME", &temp_path) };
        } else if cfg!(target_os = "windows") {
            unsafe { std::env::set_var("APPDATA", &temp_path) };
        } else {
            unsafe { std::env::set_var("XDG_CONFIG_HOME", &temp_path) };
        }
        Db::new(".test.db").expect("Failed to create test database")
    }

    fn insert_test_result(
        db: &mut Db,
        mode_kind: &str,
        mode_value: i32,
        language: &str,
        wpm: u16,
        accuracy: u16,
    ) {
        let created_at_str = "2023-10-18T12:00:00+00:00";
        db.conn.execute(
            "INSERT INTO results (mode_kind, mode_value, language, wpm, raw_wpm, accuracy, consistency, error_count, numbers, symbols, punctuation, created_at)
             VALUES (?, ?, ?, ?, 0, ?, 100, 0, 0, 0, 0, ?)",
            params![mode_kind, mode_value, language, wpm, accuracy, created_at_str],
        ).unwrap();
    }

    #[test]
    fn test_save_results() {
        let mut db = create_test_db();
        let config = Config::default();
        let mut tracker = Tracker::new("test".to_string(), Mode::with_words(1));
        tracker.start_typing();
        for c in "test".chars() {
            tracker.type_char(c).unwrap()
        }

        tracker.complete();

        let id = db.write(&config, &tracker).unwrap();

        assert!(id > 0)
    }

    #[test]
    fn test_query_data() {
        let mut db = create_test_db();
        db.reset().unwrap();
        insert_test_result(&mut db, "time", 60, "english", 80, 95);
        insert_test_result(&mut db, "words", 25, "english", 70, 90);

        let query = LeaderboardQuery::default();
        let state = db.query_data(&query).unwrap();

        assert_eq!(state.count, 2);
        assert_eq!(state.data.len(), 2);
        assert!(!state.has_more);
    }

    #[test]
    fn test_query_data_sorting() {
        let mut db = create_test_db();
        db.reset().unwrap();
        insert_test_result(&mut db, "words", 25, "english", 70, 90);
        insert_test_result(&mut db, "time", 60, "english", 80, 95);

        let query = LeaderboardQuery {
            sort_by: LeaderboardColumn::Wpm,
            sort_order: SortOrder::Descending,
            ..Default::default()
        };
        let state = db.query_data(&query).unwrap();

        assert_eq!(state.data[0].wpm, 80);
        assert_eq!(state.data[1].wpm, 70);
    }

    #[test]
    fn test_query_data_limit_offset() {
        let mut db = create_test_db();
        // db.conn.execute("DELETE FROM results", []).unwrap();
        db.reset().unwrap();
        for i in 0..5 {
            insert_test_result(&mut db, "time", 60, "english", (50 + i) as u16, 90);
        }

        let query = LeaderboardQuery {
            limit: 2,
            offset: 0,
            ..Default::default()
        };
        let state = db.query_data(&query).unwrap();

        assert_eq!(state.count, 5);
        assert_eq!(state.data.len(), 2);
        assert!(state.has_more);

        let query2 = LeaderboardQuery {
            limit: 2,
            offset: 2,
            ..Default::default()
        };
        let state2 = db.query_data(&query2).unwrap();
        assert_eq!(state2.data.len(), 2);
        assert!(state2.has_more);
    }
}
