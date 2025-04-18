use std::fs::{File, OpenOptions};
use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    constants::{
        APPNAME, DAYS_PER_MONTH, DAYS_PER_YEAR, SECS_PER_DAY, SECS_PER_HOUR, SECS_PER_MIN,
    },
    error::{TError, TResult},
};

/// Grabs the intenal config dir where the logger and persitant state files live
pub fn get_config_dir() -> TResult<PathBuf> {
    let config_dir = if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/termitype
        env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(APPNAME)
        })
    } else if cfg!(target_os = "windows") {
        // Windows: %APPDATA%\termitype (who knows)
        env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join(APPNAME))
    } else {
        // Linux/Unix: XDG Base Directory Spec
        env::var_os("XDG_CONFIG_HOME")
            .map(|xdg| PathBuf::from(xdg).join(APPNAME))
            .or_else(|| {
                env::var_os("HOME").map(|home| PathBuf::from(home).join(".config").join(APPNAME))
            })
    };

    config_dir.ok_or(TError::ConfigDirNotFound)
}

#[cfg(unix)]
pub fn create_file(path: &PathBuf) -> TResult<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .map_err(TError::from)
}

#[cfg(target_os = "windows")]
pub fn create_file(path: &PathBuf) -> TResult<File> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(TError::from)
}

/// Formats the give time in y-m-dThh-mm-ss.mmm
pub fn format_timestamp(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let ms = duration.as_millis();

    let days_since_epoch = secs / SECS_PER_DAY;
    let secs_in_day = secs % SECS_PER_DAY;

    let years_since_epoch = days_since_epoch / DAYS_PER_YEAR;
    let remaining_days = days_since_epoch % DAYS_PER_YEAR;

    let month = (remaining_days / DAYS_PER_MONTH) + 1;
    let day = (remaining_days % DAYS_PER_MONTH) + 1;
    let year = 1970 + years_since_epoch;

    let hour = secs_in_day / SECS_PER_HOUR;
    let min = (secs_in_day % SECS_PER_HOUR) / SECS_PER_MIN;
    let sec = secs_in_day % SECS_PER_MIN;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}",
        year, month, day, hour, min, sec, ms
    )
}
