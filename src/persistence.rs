use crate::{
    common::filesystem::{config_dir, create_file},
    constants::STATE_FILE,
    error::{AppError, AppResult},
};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufWriter, Write},
    path::PathBuf,
};

// TODO: Evaluate when done with the settings to ensure we set a sane default here
const DEFAULT_CAPACITY: usize = 10;

#[derive(Debug, Default)]
pub struct Persistence {
    path: PathBuf,
    values: HashMap<String, String>,
    dirty: bool,
}

impl Clone for Persistence {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            values: self.values.clone(),
            dirty: self.dirty,
        }
    }
}

impl Persistence {
    /// Creates a new persistence instance for storing key-value data.
    ///
    /// This initializes the persistence layer, creating the config directory if needed,
    /// and loads existing state from disk if available.
    ///
    /// # Returns
    /// Returns an `AppResult<Self>` containing the new Persistence instance.
    ///
    /// # Errors
    /// Returns an error if the config directory cannot be created or accessed.
    pub fn new() -> AppResult<Self> {
        let config_dir = config_dir()?;
        let path = config_dir.join(STATE_FILE);

        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let mut persistence = Self {
            path: path.clone(),
            values: HashMap::with_capacity(DEFAULT_CAPACITY),
            dirty: false,
        };

        if path.exists() && persistence.load().is_err() {
            eprintln!("Failed to load state file");
        }

        Ok(persistence)
    }

    /// Gets a value from the persistent state.
    ///
    /// # Arguments
    /// * `key` - The key to look up
    ///
    /// # Returns
    /// Returns `Some(&str)` if the key exists, `None` otherwise.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// Gets a value from the persistent state, or returns the default if not found.
    ///
    /// # Arguments
    /// * `key` - The key to look up
    /// * `default` - The default value to return if key is not found
    ///
    /// # Returns
    /// Returns the value if found, otherwise the default.
    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    /// Deletes a key from the persistent state.
    ///
    /// Marks the state as dirty if the key existed. Call `flush()` to save to disk.
    ///
    /// # Arguments
    /// * `key` - The key to delete
    ///
    /// # Returns
    /// Returns `true` if the key existed and was removed, `false` otherwise.
    pub fn delete(&mut self, key: &str) -> bool {
        if self.values.remove(key).is_some() {
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Returns true if there are unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Returns the number of stored key-value pairs.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if no key-value pairs are stored.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Clears all stored key-value pairs.
    ///
    /// Marks the state as dirty. Call `flush()` to save to disk.
    pub fn clear(&mut self) {
        if !self.values.is_empty() {
            self.values.clear();
            self.dirty = true;
        }
    }

    /// Sets a value in the persistent state.
    ///
    /// Marks the state as dirty. Call `flush()` to save to disk.
    ///
    /// # Arguments
    /// * `key` - The key to set
    /// * `value` - The value to store
    ///
    /// # Returns
    /// Returns `Ok(())` on success.
    pub fn set(&mut self, key: &str, value: &str) -> AppResult<()> {
        if self.values.get(key).map(|v| v.as_str()) != Some(value) {
            self.values.insert(key.to_string(), value.to_string());
            self.dirty = true;
        }
        Ok(())
    }

    /// Loads state from disk.
    ///
    /// Parses the state file in key=value format, ignoring empty lines and comments.
    ///
    /// # Returns
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or contains invalid format.
    fn load(&mut self) -> AppResult<()> {
        let file = File::open(&self.path)?;
        let reader = io::BufReader::new(file);
        let mut values = HashMap::with_capacity(DEFAULT_CAPACITY);

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
            } else {
                let err = format!("Invalid format at line {}: {line}", idx + 1);
                eprintln!("{err}");
                return Err(AppError::InvalidConfigData(err));
            }
        }
        self.values = values;
        self.dirty = false;

        Ok(())
    }

    /// Saves current state to disk.
    ///
    /// Uses atomic writes by saving to a temporary file first, then renaming.
    ///
    /// # Returns
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    /// Returns an error if writing or renaming fails.
    fn save(&mut self) -> AppResult<()> {
        let temp_path = self.path.with_extension("tmp");
        let file = create_file(&temp_path)?;
        let mut writer = BufWriter::new(file);

        for (k, v) in &self.values {
            writeln!(writer, "{k} = {v}")?;
        }

        writer.flush()?;

        fs::rename(temp_path, &self.path)?;
        self.dirty = false;

        Ok(())
    }

    /// Forces an immediate save to disk if there are unsaved changes.
    ///
    /// # Returns
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    /// Returns an error if saving fails.
    pub fn flush(&mut self) -> AppResult<()> {
        if self.dirty {
            self.save()?;
        }
        Ok(())
    }
}

pub fn reset_persistence() -> anyhow::Result<()> {
    let mut persistence = Persistence::new()?;
    persistence.clear();
    persistence.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // NOTE: this is to prevent concurrent ENV variables access causing tests to sometimes fail due to race conditions
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        original_home: Option<String>,
        original_appdata: Option<String>,
        original_xdg: Option<String>,
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // NOTE: Rust 2024 edition marks as unsafe `env::set_var()`, we only use this on tests so it's fine
            // https://github.com/rust-lang/rust/pull/124636
            unsafe {
                if let Some(ref h) = self.original_home {
                    std::env::set_var("HOME", h);
                }
                if let Some(ref a) = self.original_appdata {
                    std::env::set_var("APPDATA", a);
                }
                if let Some(ref x) = self.original_xdg {
                    std::env::set_var("XDG_CONFIG_HOME", x);
                }
            }
        }
    }

    fn init() -> (Persistence, TempDir, EnvGuard) {
        let _guard = ENV_MUTEX.lock().unwrap();

        let original_home = std::env::var("HOME").ok();
        let original_appdata = std::env::var("APPDATA").ok();
        let original_xdg = std::env::var("XDG_CONFIG_HOME").ok();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        let config_dir = if cfg!(target_os = "macos") {
            temp_path.join("Library/Application Support/termitype")
        } else if cfg!(target_os = "windows") {
            temp_path.join("AppData/Roaming/termitype")
        } else {
            temp_path.join(".config/termitype")
        };
        fs::create_dir_all(&config_dir).unwrap();

        let path = config_dir.join(".tmp.state");

        let persistence = Persistence {
            path,
            values: HashMap::with_capacity(DEFAULT_CAPACITY),
            dirty: false,
        };

        (
            persistence,
            temp_dir,
            EnvGuard {
                original_home,
                original_appdata,
                original_xdg,
            },
        )
    }

    #[test]
    fn test_config_dir_creation() {
        #[allow(unused_variables)]
        let (ps, tmp, guard) = init();
        assert!(ps.path.parent().unwrap().exists());
        assert!(ps.path.parent().unwrap().is_dir());
    }

    #[test]
    fn test_set_and_get() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        let config_json = r#"{"mode":"Words","language":"en","numbers":true}"#;
        ps.set("config", config_json).unwrap();
        ps.flush().unwrap();
        assert_eq!(ps.get("config"), Some(config_json))
    }

    #[test]
    fn test_set_and_get_json() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        let json_value = r#"{"mode":"Time","language":"en","numbers":false}"#;
        ps.set("config", json_value).unwrap();
        ps.flush().unwrap();
        assert_eq!(ps.get("config"), Some(json_value))
    }

    #[test]
    fn test_empty_lines() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        let config_json = r#"{"mode":"Time","language":"en"}"#;
        fs::write(&ps.path, format!("\n\n\nconfig = {}\n", config_json)).unwrap();

        ps.load().unwrap();
        assert_eq!(ps.values.len(), 1);
        assert_eq!(ps.get("config"), Some(config_json));
    }

    #[test]
    fn test_invalid_lines() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        let config_json = r#"{"mode":"Words","language":"en"}"#;
        fs::write(
            &ps.path,
            format!("\ninvalid line - random\nconfig = {}\n", config_json),
        )
        .unwrap();

        assert!(ps.load().is_err())
    }

    #[test]
    fn test_comment_lines() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        let config_json = r#"{"mode":"Time","language":"en"}"#;
        fs::write(
            &ps.path,
            format!("config = {}\n#other = ignored", config_json),
        )
        .unwrap();

        ps.load().unwrap();
        assert_eq!(ps.values.len(), 1);
        assert_eq!(ps.get("config"), Some(config_json));
    }

    #[test]
    fn test_save_and_load() {
        #[allow(unused_variables)]
        let (mut ps1, tmp, guard) = init();
        let config_json = r#"{"mode":"Time","language":"en","numbers":false,"symbols":true}"#;
        ps1.set("config", config_json).unwrap();
        ps1.set("other_key", "other_value").unwrap();
        ps1.flush().unwrap();

        let mut ps2 = Persistence {
            path: ps1.path.clone(),
            values: HashMap::with_capacity(DEFAULT_CAPACITY),
            dirty: false,
        };

        ps2.load().unwrap();

        assert_eq!(ps2.get("config"), Some(config_json));
        assert_eq!(ps2.get("other_key"), Some("other_value"));
    }

    #[test]
    fn test_get_or() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        ps.set("existing", "value").unwrap();
        assert_eq!(ps.get_or("existing", "default"), "value");
        assert_eq!(ps.get_or("nonexistent", "default"), "default");
    }

    #[test]
    fn test_delete() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        ps.set("key1", "value1").unwrap();
        ps.set("key2", "value2").unwrap();
        ps.flush().unwrap();

        assert!(ps.delete("key1"));
        assert!(ps.is_dirty());
        assert_eq!(ps.get("key1"), None);
        assert_eq!(ps.get("key2"), Some("value2"));

        assert!(!ps.delete("nonexistent"));
        assert_eq!(ps.len(), 1);
    }

    #[test]
    fn test_is_dirty_and_len() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        assert!(!ps.is_dirty());
        assert_eq!(ps.len(), 0);
        assert!(ps.is_empty());

        ps.set("key", "value").unwrap();
        assert!(ps.is_dirty());
        assert_eq!(ps.len(), 1);
        assert!(!ps.is_empty());

        ps.flush().unwrap();
        assert!(!ps.is_dirty());
    }

    #[test]
    fn test_flush_when_not_dirty() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        // should to nothing
        ps.flush().unwrap();
        assert!(!ps.is_dirty());
    }

    #[test]
    fn test_clear() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        ps.set("key1", "value1").unwrap();
        ps.set("key2", "value2").unwrap();
        assert_eq!(ps.len(), 2);

        ps.clear();
        assert!(ps.is_dirty());
        assert_eq!(ps.len(), 0);
        assert!(ps.is_empty());
        assert_eq!(ps.get("key1"), None);
    }
}
