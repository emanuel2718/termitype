use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config, error::TResult, helpers::get_config_dir, log_debug, log_info, tracker::Tracker,
};

const DB_FILE: &str = ".termitype.db";
const SCHEMA_VERSION: i32 = 1;

// TODO: add more stuff to store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingTestResult {
    pub id: Option<i64>,
    pub mode_type: String,
    pub mode_value: i32,
    pub language: String,
    pub wpm: u16,
    pub accuracy: u8,
    pub consistency: f64,
    pub total_keystrokes: u32,
    pub correct_keystrokes: u32,
    pub backspace_count: u32,
    pub numbers: bool,
    pub punctuation: bool,
    pub symbols: bool,
    pub created_at: DateTime<Utc>,
}

pub struct TermiDB {
    conn: Connection,
}

impl TermiDB {
    pub fn new() -> TResult<Self> {
        let config_dir = get_config_dir()?;
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        let path = config_dir.join(DB_FILE);
        let conn = Connection::open(&path)?;

        let mut db = Self { conn };
        db.init()?;

        log_info!("DB: database initialized at: {}", path.display());

        Ok(db)
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.conn.execute("PRAGMA foreign_keys = ON", [])?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                            version INTEGER PRIMARY KEY
            )",
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

    fn create(&mut self) -> anyhow::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS test_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mode_type TEXT NOT NULL,
                mode_value INTEGER NOT NULL,
                language TEXT NOT NULL,
                wpm REAL NOT NULL,
                accuracy INTEGER NOT NULL,
                consistency REAL NOT NULL,
                total_keystrokes INTEGER NOT NULL,
                correct_keystrokes INTEGER NOT NULL,
                backspace_count INTEGER NOT NULL,
                numbers BOOLEAN NOT NULL,
                punctuation BOOLEAN NOT NULL,
                symbols BOOLEAN NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        // TODO: add more indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_test_config ON test_results (
                mode_type, mode_value, language, numbers, punctuation, symbols
            )",
            [],
        )?;

        log_debug!("DB: tables created successfully");

        Ok(())
    }

    pub fn write(&mut self, config: &Config, tracker: &Tracker) -> TResult<i64> {
        let result = TypingTestResult {
            id: None,
            mode_type: config.resolve_mode_type_to_str(),
            mode_value: config.current_mode().value() as i32,
            language: config.resolve_language_to_str(),
            wpm: tracker.wpm as u16,
            accuracy: tracker.accuracy,
            consistency: tracker.calculate_consistency(),
            total_keystrokes: tracker.total_keystrokes as u32,
            correct_keystrokes: tracker.correct_keystrokes as u32,
            backspace_count: tracker.backspace_count as u32,
            numbers: config.use_numbers,
            punctuation: config.use_punctuation,
            symbols: config.use_symbols,
            created_at: Utc::now(),
        };

        self.conn.execute(
            "INSERT INTO test_results (
                mode_type, mode_value, language, wpm, accuracy, consistency, total_keystrokes, correct_keystrokes, backspace_count, numbers, punctuation, symbols, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                result.mode_type,
                result.mode_value,
                result.language,
                result.wpm,
                result.accuracy,
                result.consistency,
                result.total_keystrokes,
                result.correct_keystrokes,
                result.backspace_count,
                result.numbers,
                result.punctuation,
                result.symbols,
                result.created_at
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        log_debug!("DB: saved test result with ID: {id}");
        Ok(id)
    }

    pub fn get(&self, id: i64) -> Option<TypingTestResult> {
        let result = self.conn.query_row(
            "SELECT id, mode_type, mode_value, language, wpm, accuracy, consistency,
                    total_keystrokes, correct_keystrokes, backspace_count,
                    numbers, punctuation, symbols, created_at
             FROM test_results WHERE id = ?1",
            params![id],
            |row| {
                let created_at_str: String = row.get(13)?;
                let created_at = created_at_str.parse::<DateTime<Utc>>().map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        13,
                        "datetime".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;

                Ok(TypingTestResult {
                    id: Some(row.get(0)?),
                    mode_type: row.get(1)?,
                    mode_value: row.get(2)?,
                    language: row.get(3)?,
                    wpm: row.get::<_, f64>(4)? as u16,
                    accuracy: row.get(5)?,
                    consistency: row.get(6)?,
                    total_keystrokes: row.get(7)?,
                    correct_keystrokes: row.get(8)?,
                    backspace_count: row.get(9)?,
                    numbers: row.get(10)?,
                    punctuation: row.get(11)?,
                    symbols: row.get(12)?,
                    created_at,
                })
            },
        );

        match result {
            Ok(test_result) => Some(test_result),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

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
    fn test_database_creation() {
        let (_db, _temp) = setup_db();
    }

    #[test]
    fn test_save_results() {
        let (mut db, _tmp) = setup_db();
        let config = Config::default();
        let mut tracker = Tracker::new(&config, "test".to_string());

        tracker.wpm = 50.0;
        tracker.completion_time = Some(30.0);

        let id = db.write(&config, &tracker).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_saved_result_with_id() {
        let (mut db, _tmp) = setup_db();
        let config = Config::default();
        let mut tracker = Tracker::new(&config, "test".to_string());

        tracker.wpm = 10.0;
        tracker.accuracy = 10;
        tracker.total_keystrokes = 10;
        tracker.correct_keystrokes = 10;
        tracker.backspace_count = 10;
        tracker.completion_time = Some(30.0);

        let id = db.write(&config, &tracker).unwrap();

        let result = db.get(id);
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().id, Some(id));
        assert_eq!(result.as_ref().unwrap().wpm, 10);
        assert_eq!(result.as_ref().unwrap().total_keystrokes, 10);
        assert_eq!(result.as_ref().unwrap().correct_keystrokes, 10);
        assert_eq!(result.as_ref().unwrap().backspace_count, 10);
    }
}
