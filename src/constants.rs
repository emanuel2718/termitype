pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const DEFAULT_LINE_COUNT: u8 = 3;
pub const DEFAULT_LANGUAGE: &str = "english";
pub const DEFAULT_THEME: &str = "tokyonight";

pub const WPS_TARGET: usize = 6; // words per second target; approx 350 / 60

pub const MIN_CUSTOM_TIME: usize = 1;
pub const MAX_CUSTOM_TIME: usize = 300; // 5 mins
pub const DEFAULT_TIME_MODE_DURATION_IN_SECS: usize = 30; // 30 seconds

pub const MIN_CUSTOM_WORD_COUNT: usize = 1;
pub const MAX_CUSTOM_WORD_COUNT: usize = 5_000;
pub const DEFAULT_WORD_MODE_COUNT: usize = 50; // 50 words

pub const STATE_FILE: &str = ".state";

/// Returns the logger file name
pub fn logger_file() -> &'static str {
    #[cfg(debug_assertions)]
    {
        ".dev.log"
    }
    #[cfg(not(debug_assertions))]
    {
        ".log"
    }
}

/// Returns the database file name
pub fn db_file() -> &'static str {
    #[cfg(debug_assertions)]
    {
        ".termitype-dev.db"
    }
    #[cfg(not(debug_assertions))]
    {
        ".termitype.db"
    }
}
