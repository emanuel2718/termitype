use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::{config::ModeType, error::TResult, helpers::get_config_dir, log_debug, log_info};

const DB_FILE: &str = "termitype.db";
const SCHEMA_VERSION: i32 = 1;

// TODO: add more stuff to store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingTestResult {
    pub id: i64,
    pub mode_type: ModeType,
    pub mode_value: i32,
    pub language: String,
    pub wpm: u16,
    pub create_at: DateTime<Utc>,
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

    pub fn init(&mut self) -> anyhow::Result<()> {
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
            self.create_tables()?;
            self.conn.execute(
                "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
            log_info!("DB: schema updated to version {}", SCHEMA_VERSION);
        }

        Ok(())
    }

    fn create_tables(&mut self) -> anyhow::Result<()> {
        // TODO: add more fields
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS test_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mode_type TEXT NOT NULL,
                mode_value INTEGER NOT NULL,
                language TEXT NOT NULL,
                wpm REAL NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        // TODO: add more indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_test_config ON test_results (
                mode_type, mode_value, language
            )",
            [],
        )?;

        log_debug!("DB: tables created successfully");

        Ok(())
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
}
