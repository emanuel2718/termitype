use crate::config::Config;

pub mod cli;
pub mod config;
pub mod constants;
pub mod error;
pub mod logger;
pub mod persistence;
pub mod utils;

pub mod prelude {
    #[cfg(debug_assertions)]
    pub use crate::log_debug;
    pub use crate::{log_error, log_info, log_warn};
}

pub fn start() -> anyhow::Result<()> {
    let config = Config::new()?;
    logger::init()?;

    Ok(())
}
