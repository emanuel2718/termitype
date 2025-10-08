use crate::{
    constants::APP_NAME,
    error::{AppError, AppResult},
};
use std::{env, fs, path::PathBuf};

/// Grabs the intenal config dir where the logger and persitant state files live
pub fn config_dir() -> AppResult<PathBuf> {
    let config_dir = if cfg!(target_os = "macos") {
        // macOS: try using XDG_CONFIG_HOME, otherwise default to ~/Library/Application Support/termitype
        env::var_os("XDG_CONFIG_HOME")
            .map(|xdg| PathBuf::from(xdg).join(APP_NAME))
            .or_else(|| {
                env::var_os("HOME").map(|home| {
                    PathBuf::from(home)
                        .join("Library")
                        .join("Application Support")
                        .join(APP_NAME)
                })
            })
    } else if cfg!(target_os = "windows") {
        // Windows: %APPDATA%\termitype (who knows)
        env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join(APP_NAME))
    } else {
        // Linux/Unix: $XDG_CONFIG_HOME/termitype
        env::var_os("XDG_CONFIG_HOME")
            .map(|xdg| PathBuf::from(xdg).join(APP_NAME))
            .or_else(|| {
                env::var_os("HOME").map(|home| PathBuf::from(home).join(".config").join(APP_NAME))
            })
    };

    config_dir.ok_or(AppError::ConfigDirNotFound)
}

#[cfg(unix)]
pub fn create_file(path: &PathBuf) -> AppResult<fs::File> {
    use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .map_err(AppError::from)
}

#[cfg(target_os = "windows")]
pub fn create_file(path: &PathBuf) -> AppResult<fs::File> {
    use std::fs::OpenOptions;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(AppError::from)
}
