use crate::constants::{
    DEFAULT_LINE_COUNT, MAX_CUSTOM_TIME, MAX_CUSTOM_WORD_COUNT, MIN_CUSTOM_TIME,
    MIN_CUSTOM_WORD_COUNT,
};
use clap::Parser;

/// The CLI arguments
#[derive(Parser, Debug, Default, Clone)]
#[command(name = "termitype", about = "Terminal-based typing game.", version)]
#[command(group(
    clap::ArgGroup::new("mode")
        .args(&["time", "words"])
        .required(false)
        .multiple(false)
))]
pub struct Cli {
    /// Test duration in seconds. Enforces Time mode.
    #[arg(short = 't', long = "time", group = "mode", value_name = "SECONDS")]
    pub time: Option<u64>,

    /// Custom words for the test. Enforces Word mode.
    #[arg(short = 'w', long = "words", group = "mode", value_name = "WORDS")]
    pub words: Option<String>,

    /// Number (count) of words to type
    #[arg(short = 'c', long = "count", group = "mode", value_name = "COUNT")]
    pub words_count: Option<usize>,

    /// Should use number in the test word pool or not
    #[arg(short = 'n', long = "numbers")]
    pub use_numbers: bool,

    /// Should use symbols in the test word pool or not
    #[arg(short = 's', long = "symbols")]
    pub use_symbols: bool,

    /// Should use punctuation in the test word pool or not
    #[arg(short = 'p', long = "punctuation")]
    pub use_punctuation: bool,

    /// Language dictionary the test will use
    #[arg(short = 'l', long, value_name = "LANG")]
    pub language: Option<String>,

    /// The theme that is going to be used
    #[arg(long = "theme")]
    pub theme: Option<String>,

    /// The ASCII art for results screen
    #[arg(long = "ascii")]
    pub ascii: Option<String>,

    /// Cursor style variant: beam, block, underline, blinking-beam, blinking-block, blinking-underline
    #[arg(long = "cursor", value_name = "STYLE")]
    pub cursor: Option<String>,

    /// Picker style variant: quake, telescope, ivy, minimal
    #[arg(long = "picker", value_name = "STYLE")]
    pub picker: Option<String>,

    /// Results style variatn: minimal, neofetch, graph
    #[arg(long = "results", value_name = "STYLE")]
    pub results: Option<String>,

    /// Number of visible text lines
    #[arg(
        long = "lines",
        default_value_t = DEFAULT_LINE_COUNT,
        value_name = "COUNT",
    )]
    pub visible_lines: u8,

    /// Enables debug mode
    #[cfg(debug_assertions)]
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Start the app showing the results screen (debug only)
    #[cfg(debug_assertions)]
    #[arg(long = "show-results")]
    pub show_results: bool,

    /// Hide live WPM counter
    #[arg(long = "hide-live-wpm")]
    pub hide_live_wpm: bool,

    /// Hide notifications
    #[arg(long = "hide-notifications")]
    pub hide_notifications: bool,

    /// Do not locally track tests results
    #[arg(long = "no-track")]
    pub no_track: bool,
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(t) = self.time {
            if t < MIN_CUSTOM_TIME as u64 || t > MAX_CUSTOM_TIME as u64 {
                return Err(format!(
                    "Time must be between {} and {} seconds",
                    MIN_CUSTOM_TIME, MAX_CUSTOM_TIME
                ));
            }
        }
        if let Some(c) = self.words_count {
            if !(MIN_CUSTOM_WORD_COUNT..=MAX_CUSTOM_WORD_COUNT).contains(&c) {
                return Err(format!(
                    "Word count must be between {} and {}",
                    MIN_CUSTOM_WORD_COUNT, MAX_CUSTOM_WORD_COUNT
                ));
            }
        }

        if let Some(c) = &self.words {
            if !(MIN_CUSTOM_WORD_COUNT..=MAX_CUSTOM_WORD_COUNT).contains(&c.len()) {
                return Err(format!(
                    "Word count must be between {} and {}",
                    MIN_CUSTOM_WORD_COUNT, MAX_CUSTOM_WORD_COUNT
                ));
            }
        }
        Ok(())
    }

    pub fn clear_custom_words_flag(&mut self) {
        if self.words.is_some() {
            self.words = None
        }
    }
    // TODO: add here the check to print to console (i.e languages, themes, etc.)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_default() {
        let cli = Cli::default();
        assert!(cli.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_time() {
        // good
        let cli = Cli {
            time: Some(30),
            words_count: None,
            ..Default::default()
        };
        assert!(cli.validate().is_ok());

        // too low
        let cli = Cli {
            time: Some(0),
            words_count: None,
            ..Default::default()
        };
        assert!(cli.validate().is_err());
        assert_eq!(
            cli.validate().unwrap_err(),
            "Time must be between 1 and 300 seconds"
        );

        // too high
        let cli = Cli {
            time: Some(301),
            words_count: None,
            ..Default::default()
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_validate_valid_count() {
        // good
        let cli = Cli {
            time: None,
            words_count: Some(100),
            ..Default::default()
        };
        assert!(cli.validate().is_ok());

        // too low
        let cli = Cli {
            time: None,
            words_count: Some(0),
            ..Default::default()
        };
        assert!(cli.validate().is_err());
        assert_eq!(
            cli.validate().unwrap_err(),
            "Word count must be between 1 and 5000"
        );

        // too high
        let cli = Cli {
            time: None,
            words_count: Some(5001),
            ..Default::default()
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_validate_words_should_respect_count_rules() {
        // good
        let normal_text = "*".repeat(MAX_CUSTOM_WORD_COUNT);
        let empty_text = "".to_string();
        let long_text = "*".repeat(MAX_CUSTOM_WORD_COUNT + 1);
        let cli = Cli {
            time: None,
            words: Some(normal_text),
            ..Default::default()
        };
        assert!(cli.validate().is_ok());

        // lower than min count
        let cli = Cli {
            time: None,
            words: Some(empty_text),
            ..Default::default()
        };
        assert!(cli.validate().is_err());
        assert_eq!(
            cli.validate().unwrap_err(),
            "Word count must be between 1 and 5000"
        );

        // higher than min count
        let cli = Cli {
            time: None,
            words: Some(long_text.clone()),
            ..Default::default()
        };
        assert!(cli.validate().is_err());
        assert_eq!(
            cli.validate().unwrap_err(),
            "Word count must be between 1 and 5000"
        );
    }
}
