use std::io;

/// Application error types for termitype.
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    ConfigDirNotFound,
    ThemesNotFound,
    InvalidConfigData(String),
    TermiDB(String),
    InvalidLanguage(String),
    SqliteError(rusqlite::Error),
    Other(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::ConfigDirNotFound => write!(f, "Could not find termitype config directory"),
            Self::ThemesNotFound => write!(f, "No themes available"),
            Self::InvalidConfigData(msg) => write!(f, "Invalid configuration data: {msg}"),
            Self::InvalidLanguage(lang) => write!(f, "Invalid language: {lang}"),
            Self::TermiDB(err) => write!(f, "TermiDB Error: {err}"),
            Self::SqliteError(err) => write!(f, "Sqlite Error: {err}"),
            Self::Other(err) => write!(f, "Error: {err}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::SqliteError(err) => Some(err),
            _ => None,
        }
    }
}

impl AppError {
    /// Creates a new Other error with a message.
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Adds context to an existing error.
    pub fn with_context(self, context: &str) -> Self {
        match self {
            Self::Other(msg) => Self::Other(format!("{context}: {msg}")),
            other => other,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        Self::SqliteError(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::Other(err.to_string())
    }
}
