use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead},
    path::PathBuf,
};

use crate::error::TError;
use crate::{constants::STATE_FILE, error::TResult, log, utils::get_config_dir};

// TOOD: Evaluate when done with the settings to ensure we set a sane default here
const DEFAULT_CAPACITY: usize = 10;

#[derive(Debug)]
pub struct Persistence {
    path: PathBuf,
    values: HashMap<String, String>,
}

// TODO: implement Drop

impl Persistence {
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

    // TODO: implement get
    // TODO: implement set
    // TODO: implement save
    // TODO: implement some sort of flush behaviour to save to disk

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

        Ok(())
    }
}
