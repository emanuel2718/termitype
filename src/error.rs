use std::io;

#[derive(Debug)]
/// TermitypeError
pub enum TError {
    Io(io::Error),
    ConfigDirNotFound,
    InvalidConfigData(String),
    TermiDB(String),
    SqliteError(rusqlite::Error),
    Other(String),
}

impl std::fmt::Display for TError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::ConfigDirNotFound => write!(f, "Could not find termitype config directory"),
            Self::InvalidConfigData(msg) => write!(f, "Invalid configuration data: {}", msg),
            Self::TermiDB(err) => write!(f, "TermiDB Error: {}", err),
            Self::SqliteError(err) => write!(f, "Sqlite Error: {}", err),
            Self::Other(err) => write!(f, "Error: {}", err),
        }
    }
}

impl std::error::Error for TError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::SqliteError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for TError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

pub type TResult<T> = std::result::Result<T, TError>;

impl From<rusqlite::Error> for TError {
    fn from(err: rusqlite::Error) -> Self {
        Self::SqliteError(err)
    }
}

impl From<anyhow::Error> for TError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}
