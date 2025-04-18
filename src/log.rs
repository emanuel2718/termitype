use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::sync::Mutex;
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
    process, thread,
    time::SystemTime,
};

use once_cell::sync::OnceCell;

use crate::utils::format_timestamp;

static LOGGER: OnceCell<Mutex<Logger>> = OnceCell::new();

#[derive(Debug, Clone, Copy)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
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

        // TODO: Windows file
        let result = (|| -> io::Result<()> {
            #[cfg(unix)]
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .mode(0o600)
                .open(&self.log_file)?;

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
        self.write(Level::Info, &format!("Starting termitype v{}", version));
        self.write(Level::Info, &format!("OS: {} ({})", os, arch));
        self.write(Level::Info, &format!("PID: {}", process::id()));
        self.write(Level::Info, "----------------------------------------\n");
    }
}

/// Initialize the logger iwth the given file path
pub fn init(log_file: PathBuf) -> io::Result<()> {
    let logger = Logger::new(log_file)?;
    logger.write_startup_banner();

    match LOGGER.get() {
        Some(_) => Ok(()), // we already have a logger
        None => LOGGER
            .set(Mutex::new(logger))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to initialize logger.")),
    }
}

/// Internal `write` call
fn write(level: Level, msg: &str) {
    if let Some(logger) = LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.write(level, msg);
        }
    }
}

#[cfg(debug_assertions)]
/// Log a debug message, only available on debug mode
pub fn debug(msg: &str) {
    write(Level::Debug, msg);
}

/// Logs a info message
pub fn info(msg: &str) {
    write(Level::Info, msg);
}

/// Logs a warn message
pub fn warn(msg: &str) {
    write(Level::Warn, msg);
}

/// Logs a error message
pub fn error(msg: &str) {
    write(Level::Error, msg);
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::TempDir;

    struct Temp {
        path: PathBuf,
    }

    impl Default for Temp {
        fn default() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let temp_path = temp_dir.path().join("logs").join("test.log");
            Self { path: temp_path }
        }
    }

    #[test]
    fn test_logger_init() {
        let temp = Temp::default();

        assert!(init(temp.path.clone()).is_ok());
        assert!(init(temp.path.clone()).is_ok()); // double initialization

        write(Level::Info, "Test");

        let content = fs::read_to_string(&temp.path).unwrap();
        assert!(content.contains("Test"));
        assert!(content.contains("termitype"));
    }
}
