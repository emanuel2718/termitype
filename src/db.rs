use crate::{
    common::filesystem::config_dir,
    config::{Config, Setting},
    error::AppResult,
    log_debug, log_info,
    tracker::Tracker,
};
use chrono::{DateTime, Local};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: i32 = 1;
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

#[derive(Debug, Clone)]
pub enum SortOrder {
    Ascending,
    Descending,
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
                "SELECT version FROM schema_version ORDER_BY version DESC LIMIT 1",
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
                consistency REAL NOT NULL,
                error_count INTEGER NOT NULL,
                numbers BOOLEAN NOT NULL,
                punctuation BOOLEAN NOT NULL,
                symbols BOOLEAN NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idexes ON results (
                mode_kind, mode_value, language, numbers, punctuation, symbols
            )",
            [],
        )?;

        log_debug!("DB: tables created successfully");

        Ok(())
    }

    pub fn write(&mut self, config: &Config, tracker: &Tracker) -> AppResult<i64> {
        let current_mode = config.current_mode();
        let summary = tracker.summary();
        let result = LeaderboardResult {
            id: None,
            mode_kind: current_mode.kind().to_string(),
            mode_value: current_mode.value() as i32,
            language: config.current_language(),
            wpm: summary.wpm.round() as u16,
            raw_wpm: summary.raw_wpm().round() as u16,
            accuracy: summary.accuracy as u16,
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
                result.punctuation,
                result.symbols,
                result.created_at
            ],
        )?;

        let id = self.conn.last_insert_rowid();

        log_debug!("DB: saved test result to database with ID: '{id}'");

        Ok(id)
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
}
