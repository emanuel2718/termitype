use clap::{ArgGroup, Parser};

use crate::constants::{DEFAULT_LANGUAGE, DEFAULT_THEME};

#[derive(Parser, Debug, Clone)]
#[command(name = "Termitype", about = "Terminal based typing game")]
#[command(group(
    ArgGroup::new("mode")
        .args(&["time", "words"])
        .required(false)
        .multiple(false)
))]
pub struct Config {
    /// The language dictionary used for the test. Defaults to English 10k words.
    #[arg(short, long, value_name = "LANG")]
    pub language: Option<String>,

    /// Duration of the test in seconds (only valid in Time mode).
    #[arg(short = 't', long = "time", group = "mode")]
    pub time: Option<u64>,

    /// Number of words used in the test (only valid in Words mode).
    #[arg(short = 'w', long = "words", group = "mode")]
    pub words: Option<usize>,

    /// Sets the theme if a valid theme is given, ignored otherwise
    #[arg(short = 'T', long = "theme")]
    pub theme: Option<String>,

    /// Lists the available themes
    #[arg(short = 'L', long = "list-themes")]
    pub list_themes: bool,

    /// Introduces symbols to the test words.
    #[arg(short = 's', long = "use-symbols")]
    pub use_symbols: bool,

    /// Introduces punctuation to the test words.
    #[arg(short = 'p', long = "use-punctuation")]
    pub use_punctuation: bool,

    /// Introduces numbers to the test words
    #[arg(short = 'n', long = "use-numbers")]
    pub use_numbers: bool,

    /// Set color support level
    #[arg(
        long = "color-mode",
        value_name = "MODE",
        help = "Overwrite color support mode: 'basic' (8 colors), 'extended' (256 colors), \
               or 'truecolor' (24-bit, default)."
    )]
    pub color_mode: Option<String>,

    /// Enable debug mode
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModeType {
    Time,
    Words,
}

/// Represents the operationlal mode of the game>
pub enum Mode {
    Time { duration: u64 },
    Words { count: usize },
}

impl Mode {
    /// Returns the value of the current mode.
    pub fn value(&self) -> usize {
        match self {
            Mode::Time { duration } => *duration as usize,
            Mode::Words { count } => *count,
        }
    }
}

impl Config {
    /// Returns a new instance of the Config struct with default values.
    pub fn default() -> Self {
        Self {
            language: Some(DEFAULT_LANGUAGE.to_string()),
            time: Some(30),
            words: None,
            use_symbols: false,
            use_numbers: false,
            use_punctuation: false,
            theme: Some(DEFAULT_THEME.to_string()),
            color_mode: None,
            list_themes: false,
            debug: false,
        }
    }

    /// Resolves the mode based onf the provided arguments
    /// Defaults to time mode with (30) seconds if no options are provided.
    /// If *both* `time` and `word` mode are passed, it will default to time mode.
    pub fn current_mode(&self) -> Mode {
        match (self.time, self.words) {
            (Some(time), None) => Mode::Time { duration: time },
            (None, Some(count)) => Mode::Words { count },
            (None, None) => Mode::Time { duration: 30 },
            _ => unreachable!("Both Time mode and Words mode cannot be used at the same time."),
        }
    }

    /// Changes the mode of the game.
    pub fn change_mode(&mut self, mode: ModeType, value: Option<usize>) {
        match mode {
            ModeType::Time => {
                self.time = Some(value.unwrap_or(30) as u64);
                self.words = None;
            }
            ModeType::Words => {
                self.words = Some(value.unwrap_or(100));
                self.time = None;
            }
        }
    }

    /// Changes the value of the current mode.
    pub fn change_mode_value(&mut self, value: usize) {
        match self.current_mode() {
            Mode::Time { .. } => self.time = Some(value as u64),
            Mode::Words { .. } => self.words = Some(value),
        }
    }

    /// Resolves the test word count based on current configuration.
    pub fn resolve_word_count(&self) -> usize {
        match (self.time, self.words) {
            (None, Some(count)) => count,
            _ => 100,
        }
    }

    /// Resolves the test duration based on current configuration.
    pub fn resolve_duration(&self) -> u64 {
        match (self.time, self.words) {
            (Some(duration), None) => duration,
            _ => 30,
        }
    }

    /// Toggles the presence of numbers in the test word pool.
    pub fn toggle_numbers(&mut self) {
        self.use_numbers = !self.use_numbers;
    }

    /// Toggles the presence of punctuation in the test word pool.
    pub fn toggle_punctuation(&mut self) {
        self.use_punctuation = !self.use_punctuation;
    }

    /// Toggles the presence of symbols in the test word pool.
    pub fn toggle_symbols(&mut self) {
        self.use_symbols = !self.use_symbols;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_config() -> Config {
        let config = Config::default();
        config
    }

    #[test]
    fn test_default_state() {
        let config = Config::default();
        assert!(config.language.is_some());
        assert!(config.time.is_some());
        assert!(config.words.is_none());
        assert_eq!(config.language, Some(DEFAULT_LANGUAGE.to_string()));
        assert_eq!(config.use_symbols, false);
        assert_eq!(config.use_punctuation, false);
        assert_eq!(config.debug, false);
    }

    fn assert_mode(config: &Config, expected_mode: ModeType, expected_value: usize) {
        match config.current_mode() {
            Mode::Time { duration } if matches!(expected_mode, ModeType::Time) => {
                assert_eq!(duration as usize, expected_value)
            }
            Mode::Words { count } if matches!(expected_mode, ModeType::Words) => {
                assert_eq!(count, expected_value)
            }
            _ => panic!("Unexpected mode"),
        }
    }

    #[test]
    fn test_config_lifecycle() {
        let mut config = create_config();

        // modes
        config.change_mode(ModeType::Time, Some(30));
        assert!(config.words.is_none());
        assert_mode(&config, ModeType::Time, 30);

        config.change_mode(ModeType::Words, Some(50));
        assert!(config.time.is_none());
        assert_mode(&config, ModeType::Words, 50);

        // toggles
        config.toggle_numbers();
        config.toggle_punctuation();
        config.toggle_symbols();

        assert!(config.use_numbers);
        assert!(config.use_punctuation);
        assert!(config.use_punctuation);

        // resolvers
        config.change_mode(ModeType::Time, Some(45));
        assert_eq!(config.resolve_duration(), 45);
        config.change_mode(ModeType::Words, Some(75));
        assert_eq!(config.resolve_word_count(), 75);
    }
}
