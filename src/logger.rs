use std::fs::File;
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::sync::Mutex;
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
    process,
    time::SystemTime,
};

use once_cell::sync::OnceCell;

use crate::{
    constants,
    utils::{filesystem, strings::format_timestamp},
};

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
    file: Mutex<File>,
}

impl Logger {
    fn new(log_file: PathBuf) -> io::Result<Self> {
        if let Some(dir) = log_file.parent() {
            fs::create_dir_all(dir)?;
        }
        let file = {
            #[cfg(unix)]
            {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .mode(0o600)
                    .open(&log_file)?
            }
            #[cfg(windows)]
            {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file)?
            }
        };
        Ok(Self {
            file: Mutex::new(file),
        })
    }

    fn write(&self, level: Level, file: Option<&str>, line: Option<u32>, msg: &str) {
        if let Some(level_lock) = LOG_LEVEL.get() {
            if let Ok(min_level) = level_lock.lock() {
                if level < *min_level {
                    return;
                }
            }
        }

        let now = SystemTime::now();

        let timestamp = format_timestamp(now);

        let file_info = if let (Some(file), Some(line)) = (file, line) {
            format!("[{}:{}] ", file, line)
        } else {
            "".to_string()
        };

        let result = (|| -> io::Result<()> {
            let mut file_handle = self
                .file
                .lock()
                .map_err(|_| io::Error::other("Mutex poisoned"))?;
            writeln!(
                file_handle,
                "[{}] {} {}{}",
                timestamp,
                level.as_str(),
                file_info,
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
        let pid = process::id();
        let now = SystemTime::now();
        let timestamp = format_timestamp(now);

        self.write(Level::Info, None, None, "=== TermiType Startup ===");
        self.write(Level::Info, None, None, &format!("Version: {}", version));
        self.write(Level::Info, None, None, &format!("OS: {} ({})", os, arch));
        self.write(Level::Info, None, None, &format!("PID: {}", pid));
        self.write(
            Level::Info,
            None,
            None,
            &format!("Started at: {}", timestamp),
        );
        self.write(Level::Info, None, None, "==========================");
    }
}

/// Initialize the logger
pub fn init() -> io::Result<()> {
    let initial_level = if cfg!(debug_assertions) {
        Level::Debug
    } else {
        Level::Info
    };
    let config_dir = filesystem::config_dir().map_err(std::io::Error::other)?;
    let log_file = config_dir.join(constants::logger_file());

    LOG_LEVEL.get_or_init(|| Mutex::new(initial_level));

    LOGGER.get_or_try_init(|| -> io::Result<Mutex<Logger>> {
        let logger = Logger::new(log_file)?;
        logger.write_startup_banner();
        Ok(Mutex::new(logger))
    })?;

    Ok(())
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
pub fn write(level: Level, file: Option<&str>, line: Option<u32>, msg: &str) {
    if let Some(logger) = LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.write(level, file, line, msg);
        }
    }
}

/// Logs a debug message. Only enabled in debug builds.
///
/// # Examples
///
/// ```
/// use termitype::log_debug;
/// let item = "example_item";
/// let status = "running";
/// log_debug!("Termitype artifact {}", item);
/// log_debug!("Status: {status}");
/// ```
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        $crate::logger::write(
            $crate::logger::Level::Debug,
            Some(file!()),
            Some(line!()),
            &format!($($arg)*)
        )
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{}};
}

//
/// Logs an info message.
///
/// # Examples
///
/// ```
/// use termitype::log_info;
/// let duration = 123;
/// log_info!("Termitype started");
/// log_info!("Test took {}ms", duration);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        $crate::logger::write(
            $crate::logger::Level::Info,
            Some(file!()),
            Some(line!()),
            &format!($($arg)*)
        )
    }};
}

/// Logs a warning message.
///
/// # Examples
///
/// ```
/// use termitype::log_warn;
/// let id = 456;
/// let reason = "timeout";
/// log_warn!("Failed to process item {}, retrying...", id);
/// log_warn!("Performance degraded: {reason}");
/// ```
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        $crate::logger::write(
            $crate::logger::Level::Warn,
            Some(file!()),
            Some(line!()),
            &format!($($arg)*)
        )
    }};
}

/// Logs an error message.
///
/// # Examples
///
/// ```
/// use termitype::log_error;
/// let err = "permission denied";
/// let msg = "Failed to open resource";
/// log_error!("Failed to save file: {}", err);
/// log_error!("Critical error occurred: {msg}");
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        $crate::logger::write(
            $crate::logger::Level::Error,
            Some(file!()),
            Some(line!()),
            &format!($($arg)*)
        )
    }};
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

        write(Level::Debug, None, None, "Debug message");
        write(Level::Info, None, None, "Info message");
        write(Level::Warn, None, None, "Warn message");
        write(Level::Error, None, None, "Error message");

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
