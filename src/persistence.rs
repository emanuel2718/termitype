use crate::{
    constants::STATE_FILE,
    error::{AppError, AppResult},
    utils::filesystem::{config_dir, create_file},
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

        if path.exists() {
            if let Err(e) = persistence.load() {
                eprintln!("Failed to load state file: {e}");
            } else {
                eprintln!("Successfully loaded state file");
            }
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
                let err = format!("Invalid format at line {}: {line}", idx + 1);
                eprintln!("{err}");
                return Err(AppError::InvalidConfigData(err));
            }
        }
        eprintln!("Successfully loaded {loaded_count} settings");
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

        let path = config_dir.join(STATE_FILE);

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
        ps.set("vizquit", "test").unwrap();
        ps.flush().unwrap();
        assert_eq!(ps.get("vizquit"), Some("test"))
    }

    #[test]
    fn test_empty_lines() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        fs::write(&ps.path, "\n\n\nkey1 = value1\n").unwrap();

        ps.load().unwrap();
        assert_eq!(ps.values.len(), 1);
        assert_eq!(ps.get("key1"), Some("value1"));
    }

    #[test]
    fn test_invalid_lines() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        fs::write(&ps.path, "\ninvalid line - random\nkey1 = value1\n").unwrap();

        assert!(ps.load().is_err())
    }

    #[test]
    fn test_comment_lines() {
        #[allow(unused_variables)]
        let (mut ps, tmp, guard) = init();
        fs::write(&ps.path, "key1 = value1\n#key2 = should_be_ignored").unwrap();

        ps.load().unwrap();
        assert_eq!(ps.values.len(), 1);
        assert_eq!(ps.get("key1"), Some("value1"));
    }

    #[test]
    fn test_save_and_load() {
        #[allow(unused_variables)]
        let (mut ps1, tmp, guard) = init();
        ps1.set("key1", "value1").unwrap();
        ps1.set("key2", "value2").unwrap();
        ps1.flush().unwrap();

        let config_dir = ps1.path.parent().unwrap().to_path_buf();
        let path = config_dir.join(STATE_FILE);

        let mut ps2 = Persistence {
            path,
            values: HashMap::with_capacity(DEFAULT_CAPACITY),
            dirty: false,
        };

        ps2.load().unwrap();

        assert_eq!(ps2.get("key1"), Some("value1"));
        assert_eq!(ps2.get("key2"), Some("value2"));
    }
}
