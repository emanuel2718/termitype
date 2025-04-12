use std::io;

#[derive(Debug)]
/// TermitypeError
pub enum TError {
    Io(io::Error),
    ConfigDirNotFound,
}

impl std::fmt::Display for TError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::ConfigDirNotFound => write!(f, "Could not find termitype config directory"),
        }
    }
}

impl std::error::Error for TError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
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
