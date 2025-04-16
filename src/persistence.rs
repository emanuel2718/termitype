use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufWriter, Write},
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use crate::{constants::STATE_FILE, error::TResult, log, utils::get_config_dir};
use crate::{error::TError, utils::create_file};

// TOOD: Evaluate when done with the settings to ensure we set a sane default here
const DEFAULT_CAPACITY: usize = 10;
const SAVE_DEBOUNCE_MS: u64 = 1_000; // 1 second as u64 because of Duration::from_millis

#[derive(Debug)]
pub struct Persistence {
    path: PathBuf,
    values: HashMap<String, String>,
    last_save: Instant,
    has_pending_changes: AtomicBool,
}

impl Clone for Persistence {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            values: self.values.clone(),
            last_save: self.last_save,
            has_pending_changes: AtomicBool::new(self.has_pending_changes.load(Ordering::Relaxed)),
        }
    }
}

impl Drop for Persistence {
    fn drop(&mut self) {
        if self.has_pending_changes.load(Ordering::Relaxed) {
            if let Err(e) = self.save() {
                log::error(&format!("Failed to save state on drop: {}", e))
            }
        }
    }
}

impl Persistence {
    /// Creates new persistence instnace
    pub fn new() -> TResult<Self> {
        log::debug("initializing persistance system");

        let config_dir = get_config_dir()?;
        let path = config_dir.join(STATE_FILE);

        if !config_dir.exists() {
            log::debug(&format!("Creating config directory at: {:?}", config_dir));
            fs::create_dir_all(config_dir)?;
        }

        let mut persistence = Self {
            path: path.clone(),
            values: HashMap::with_capacity(DEFAULT_CAPACITY),
            last_save: Instant::now(),
            has_pending_changes: AtomicBool::new(false),
        };

        if path.exists() {
            if let Err(e) = persistence.load() {
                log::error(&format!("Failed to load state file: {}", e));
            } else {
                log::info("Successfully loaded state file");
            }
        }

        Ok(persistence)
    }

    // Gets a value from state
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    // Sets a value to the sate (may be deffered)
    pub fn set(&mut self, key: &str, value: &str) -> TResult<()> {
        match self.values.get(key) {
            Some(existing) if existing == value => {
                return Ok(());
            }
            _ => {
                self.values.insert(key.to_string(), value.to_string());
                self.has_pending_changes.store(true, Ordering::Relaxed);
            }
        }

        let now = Instant::now();
        if now.duration_since(self.last_save) >= Duration::from_millis(SAVE_DEBOUNCE_MS) {
            self.save()?;
        }

        Ok(())
    }

    /// Loads state form disk
    fn load(&mut self) -> TResult<()> {
        log::debug(&format!("Loading state from: {:?}", self.path));
        let file = File::open(&self.path)?;
        let reader = io::BufReader::new(file);
        let mut values = HashMap::with_capacity(DEFAULT_CAPACITY);
        let mut loaded_count = 0;

        for (idx, line) in reader.lines().enumerate() {
            let line = line?;
            let line = line.trim();

            // skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim();
                let value = v.trim();
                values.insert(key.to_string(), value.to_string());
                loaded_count += 1;
            } else {
                let err = format!("Invalid format at line {}: {}", idx + 1, line);
                log::error(&err);
                return Err(TError::InvalidConfigData(err));
            }
        }
        log::info(&format!("Successfully loaded {} settings", loaded_count));
        self.values = values;
        self.has_pending_changes.store(false, Ordering::Relaxed);

        Ok(())
    }

    /// Saves current state to disk
    fn save(&mut self) -> TResult<()> {
        log::debug(&format!("Saving state to: {:?}", self.path));

        let temp_path = self.path.with_extension("tmp");
        let file = create_file(&temp_path)?;
        let mut writer = BufWriter::new(file);

        for (k, v) in &self.values {
            writeln!(writer, "{} = {}", k, v)?;
            log::debug(&format!("Saving setting: {} = {}", k, v));
        }

        writer.flush()?;

        fs::rename(temp_path, &self.path)?;
        self.last_save = Instant::now();
        self.has_pending_changes.store(false, Ordering::Relaxed);

        Ok(())
    }

    /// Forces an immediate save to disk if there are pending changes
    pub fn flush(&mut self) -> TResult<()> {
        if self.has_pending_changes.load(Ordering::Relaxed) {
            self.save()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // NOTE: thsi is to prevent concurrent ENV variables access causing tests to somtimes fail due to race conditions
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn init() -> (Persistence, TempDir) {
        let _guard = ENV_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        if cfg!(target_os = "macos") {
            std::env::remove_var("HOME");
        } else if cfg!(target_os = "windows") {
            std::env::remove_var("APPDATA");
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
            std::env::remove_var("HOME");
        }

        // fake config dir
        if cfg!(target_os = "macos") {
            std::env::set_var("HOME", temp_path);
        } else if cfg!(target_os = "windows") {
            std::env::set_var("APPDATA", temp_path);
        } else {
            std::env::set_var("XDG_CONFIG_HOME", temp_path);
        }

        let config_dir = get_config_dir().unwrap();
        fs::create_dir_all(&config_dir).unwrap();

        let path = config_dir.join(STATE_FILE);

        let persistence = Persistence {
            path,
            values: HashMap::with_capacity(DEFAULT_CAPACITY),
            last_save: Instant::now(),
            has_pending_changes: AtomicBool::new(false),
        };

        (persistence, temp_dir)
    }

    #[test]
    fn test_config_dir_creation() {
        #[allow(unused_variables)]
        let (ps, tmp) = init();
        assert!(ps.path.parent().unwrap().exists());
        assert!(ps.path.parent().unwrap().is_dir());
    }

    #[test]
    fn test_set_and_get() {
        #[allow(unused_variables)]
        let (mut ps, tmp) = init();
        ps.set("vizquit", "test").unwrap();
        ps.flush().unwrap();
        assert_eq!(ps.get("vizquit"), Some("test"))
    }

    #[test]
    fn test_empty_lines() {
        #[allow(unused_variables)]
        let (mut ps, tmp) = init();
        fs::write(&ps.path, "\n\n\nkey1 = value1\n").unwrap();

        ps.load().unwrap();
        assert_eq!(ps.values.len(), 1);
        assert_eq!(ps.get("key1"), Some("value1"));
    }

    #[test]
    fn test_invalid_lines() {
        #[allow(unused_variables)]
        let (mut ps, tmp) = init();
        fs::write(&ps.path, "\ninvalid line - random\nkey1 = value1\n").unwrap();

        assert!(ps.load().is_err())
    }

    #[test]
    fn test_comment_lines() {
        #[allow(unused_variables)]
        let (mut ps, tmp) = init();
        fs::write(&ps.path, "key1 = value1\n#key2 = should_be_ignored").unwrap();

        ps.load().unwrap();
        assert_eq!(ps.values.len(), 1);
        assert_eq!(ps.get("key1"), Some("value1"));
    }

    #[test]
    fn test_save_and_load() {
        #[allow(unused_variables)]
        let (mut ps1, tmp) = init();
        ps1.set("key1", "value1").unwrap();
        ps1.set("key2", "value2").unwrap();
        ps1.flush().unwrap();

        let config_dir = ps1.path.parent().unwrap().to_path_buf();
        let path = config_dir.join(STATE_FILE);

        let mut ps2 = Persistence {
            path,
            values: HashMap::with_capacity(DEFAULT_CAPACITY),
            last_save: Instant::now(),
            has_pending_changes: AtomicBool::new(false),
        };

        ps2.load().unwrap();

        assert_eq!(ps2.get("key1"), Some("value1"));
        assert_eq!(ps2.get("key2"), Some("value2"));
    }
}
