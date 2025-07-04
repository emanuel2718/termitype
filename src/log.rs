use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::sync::Mutex;
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
    process, thread,
    time::SystemTime,
};

use once_cell::sync::OnceCell;

use crate::helpers::format_timestamp;

static LOGGER: OnceCell<Mutex<Logger>> = OnceCell::new();
static LOG_LEVEL: OnceCell<Mutex<Level>> = OnceCell::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl Level {
    fn as_str(&self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

#[derive(Debug)]
pub struct Logger {
    log_file: PathBuf,
}

impl Logger {
    fn new(log_file: PathBuf) -> io::Result<Self> {
        if let Some(dir) = log_file.parent() {
            fs::create_dir_all(dir)?;
        }
        Ok(Self { log_file })
    }

    fn write(&self, level: Level, msg: &str) {
        if let Some(level_lock) = LOG_LEVEL.get() {
            if let Ok(min_level) = level_lock.lock() {
                if level < *min_level {
                    return;
                }
            }
        }

        let now = SystemTime::now();

        let timestamp = format_timestamp(now);
        let thread_name = thread::current().name().unwrap_or("unknown").to_string();
        let pid = process::id();

        if let Some(parent) = self.log_file.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create log directory: {e}");
                return;
            }
        }

        let result = (|| -> io::Result<()> {
            let mut file = {
                #[cfg(unix)]
                {
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .mode(0o600)
                        .open(&self.log_file)?
                }
                #[cfg(windows)]
                {
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&self.log_file)?
                }
            };

            writeln!(
                file,
                "{} | {} | {:>5} | {:>5} | {}",
                timestamp,
                level.as_str(),
                pid,
                thread_name,
                msg
            )
        })();

        if let Err(e) = result {
            eprintln!("Failed to write to log file: {e}");
        }
    }

    fn write_startup_banner(&self) {
        let version = env!("CARGO_PKG_VERSION");
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        self.write(Level::Info, "\n----------------------------------------");
        self.write(Level::Info, &format!("Starting termitype v{version}"));
        self.write(Level::Info, &format!("OS: {os} ({arch})"));
        let pid = process::id();
        self.write(Level::Info, &format!("PID: {pid}"));
        self.write(Level::Info, "----------------------------------------\n");
    }
}

/// Initialize the logger
pub fn init(log_file: PathBuf, is_debug: bool) -> io::Result<()> {
    let initial_level = if is_debug { Level::Debug } else { Level::Info };

    let init_result = LOGGER.get_or_try_init(|| -> io::Result<Mutex<Logger>> {
        LOG_LEVEL.get_or_init(|| Mutex::new(initial_level));
        let logger = Logger::new(log_file)?;
        logger.write_startup_banner();
        Ok(Mutex::new(logger))
    });

    match init_result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Set the minimum log level. Messages below this level will be ignored.
pub fn set_level(level: Level) {
    if let Some(level_lock) = LOG_LEVEL.get() {
        if let Ok(mut current_level) = level_lock.lock() {
            *current_level = level;
        }
    }
}

/// Internal `write` call
pub fn write(level: Level, msg: &str) {
    if let Some(logger) = LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.write(level, msg);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn setup(log_path: PathBuf) {
        let dummy_path = PathBuf::from("target/.init_dummy.log");
        if let Some(p) = dummy_path.parent() {
            fs::create_dir_all(p).ok();
        }
        let _ = LOGGER.get_or_try_init(|| -> io::Result<Mutex<Logger>> {
            LOG_LEVEL.get_or_init(|| Mutex::new(Level::Debug));
            let logger = Logger::new(dummy_path)?;
            Ok(Mutex::new(logger))
        });

        if let Some(logger_mutex) = LOGGER.get() {
            let mut logger_guard = logger_mutex.lock().expect("Failed to lock logger mutex");
            let test_specific_logger = Logger::new(log_path).expect("Failed to create test logger");
            *logger_guard = test_specific_logger;
        } else {
            panic!("Logger failed to initialize");
        }
    }

    #[test]
    fn test_log_levels() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp log file");
        let log_path = temp_file.path().to_path_buf();

        setup(log_path.clone());

        set_level(Level::Info);

        fs::write(&log_path, "").expect("Failed to clear log file");

        write(Level::Debug, "Debug message");
        write(Level::Info, "Info message");
        write(Level::Warn, "Warn message");
        write(Level::Error, "Error message");

        let content = fs::read_to_string(&log_path).expect("Failed to read temp log file");
        assert!(
            !content.contains("Debug message"),
            "Content should not contain Debug: {content}",
        );
        assert!(
            content.contains("Info message"),
            "Content should contain Info: {content}",
        );
        assert!(
            content.contains("Warn message"),
            "Content should contain Warn: {content}",
        );
        assert!(
            content.contains("Error message"),
            "Content should contain Error: {content}",
        );
    }
}
