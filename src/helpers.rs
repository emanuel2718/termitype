use std::fs::{File, OpenOptions};
use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::config::Config;
use crate::{
    constants::{
        APPNAME, DAYS_PER_MONTH, DAYS_PER_YEAR, SECS_PER_DAY, SECS_PER_HOUR, SECS_PER_MIN,
    },
    error::{TError, TResult},
};

pub fn should_print_to_console(config: &Config) -> bool {
    if config.list_themes {
        crate::theme::print_theme_list();
        return true;
    }

    if config.list_languages {
        crate::builder::print_language_list();
        return true;
    }

    if config.list_ascii {
        crate::ascii::print_ascii_list();
        return true;
    }
    false
}

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
        // Linux/Unix: $XDG_CONFIG_HOME/termitype
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
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}.{ms:03}")
}

// TODO: maybe we can improve this to be more performant. Using the most basic fuzzy search possible for now
pub fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let text = text.chars().collect::<Vec<_>>();
    let pattern = pattern.chars().collect::<Vec<_>>();

    let mut text_idx = 0;
    let mut pattern_idx = 0;

    while text_idx < text.len() && pattern_idx < pattern.len() {
        if text[text_idx] == pattern[pattern_idx] {
            pattern_idx += 1;
        }
        text_idx += 1;
    }

    pattern_idx == pattern.len()
}
