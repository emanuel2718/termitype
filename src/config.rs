use clap::{ArgGroup, Parser};
use crossterm::cursor::SetCursorStyle;

use crate::constants::{DEFAULT_LANGUAGE, DEFAULT_THEME};

#[derive(Parser, Debug, Clone)]
#[command(name = "Termitype", about = "Terminal based typing game")]
#[command(group(
    ArgGroup::new("mode")
        .args(&["time", "word_count"])
        .required(false)
        .multiple(false)
))]
pub struct Config {
    /// The language dictionary used for the test. Defaults to English.
    #[arg(short, long, value_name = "LANG")]
    pub language: Option<String>,

    /// Duration of the test in seconds (only valid in Time mode).
    #[arg(short = 't', long = "time", group = "mode")]
    pub time: Option<u64>,

    /// Words used in the test (only valid in Words mode).
    #[arg(short = 'w', long = "words", group = "mode")]
    pub words: Option<String>,

    /// Number of words used in the test (only valid in Words mode).
    #[arg(long = "word-count", group = "mode")]
    pub word_count: Option<usize>,

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

    /// Sets the cursor style
    #[arg(
        long = "cursor-style",
        value_name = "CURSOR",
        help = "Sets the cursor style: 'beam', 'block', 'underline', 'blinking-beam', 'blinking-block', 'blinking-underline'"
    )]
    pub cursor_style: Option<String>,

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

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Some(DEFAULT_LANGUAGE.to_string()),
            time: Some(30),
            words: None,
            word_count: None,
            use_symbols: false,
            use_numbers: false,
            use_punctuation: false,
            theme: Some(DEFAULT_THEME.to_string()),
            cursor_style: None,
            color_mode: None,
            list_themes: false,
            debug: false,
        }
    }
}

impl Config {
    /// Returns a new instance of the Config struct with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves the mode based onf the provided arguments
    /// Defaults to time mode with (30) seconds if no options are provided.
    /// If *both* `time` and `word` mode are passed, it will default to time mode.
    pub fn current_mode(&self) -> Mode {
        if let Some(words) = self.words.clone() {
            Mode::Words {
                count: words.split_ascii_whitespace().count(),
            }
        } else {
            match (self.time, self.word_count) {
                (Some(time), None) => Mode::Time { duration: time },
                (None, Some(count)) => Mode::Words { count },
                (None, None) => Mode::Time { duration: 30 },
                _ => unreachable!("Both Time mode and Words mode cannot be used at the same time."),
            }
        }
    }

    pub fn current_mode_type(&self) -> ModeType {
        match self.current_mode() {
            Mode::Time { .. } => ModeType::Time,
            Mode::Words { .. } => ModeType::Words,
        }
    }

    /// Changes the mode of the game.
    pub fn change_mode(&mut self, mode: ModeType, value: Option<usize>) {
        match mode {
            ModeType::Time => {
                self.word_count = None;
                self.time = Some(value.unwrap_or(30) as u64);
            }
            ModeType::Words => {
                self.time = None;
                self.word_count = Some(value.unwrap_or(25));
            }
        }
    }

    /// Chages the current theme of the game.
    pub fn change_theme(&mut self, theme_name: &str) {
        self.theme = Some(theme_name.to_string())
    }

    /// Resets the words flag after a test has been run with it.
    pub fn reset_words_flag(&mut self) {
        self.words = None;
    }

    /// Changes the value of the current mode.
    pub fn change_mode_value(&mut self, value: usize) {
        match self.current_mode() {
            Mode::Time { .. } => self.time = Some(value as u64),
            Mode::Words { .. } => self.word_count = Some(value),
        }
    }

    /// Resolves the test word count based on current configuration.
    pub fn resolve_word_count(&self) -> usize {
        match (self.time, self.word_count) {
            (None, Some(count)) => count,
            _ => 100,
        }
    }

    /// Resolves the test duration based on current configuration.
    pub fn resolve_duration(&self) -> u64 {
        match (self.time, self.word_count) {
            (Some(duration), None) => duration,
            _ => 30,
        }
    }

    /// Resolves the cursor style based on current configuration.
    pub fn resolve_current_cursor_style(&self) -> SetCursorStyle {
        match self.cursor_style.as_deref() {
            Some("beam") => SetCursorStyle::SteadyBar,
            Some("block") => SetCursorStyle::DefaultUserShape,
            Some("underline") => SetCursorStyle::SteadyUnderScore,
            Some("blinking-beam") => SetCursorStyle::BlinkingBar,
            Some("blinking-block") => SetCursorStyle::BlinkingBlock,
            Some("blinking-underline") => SetCursorStyle::BlinkingUnderScore,
            _ => SetCursorStyle::BlinkingBar, // default to beam style
        }
    }

    pub fn resolve_cursor_style_from_name(&self, name: &str) -> SetCursorStyle {
        match name {
            "beam" => SetCursorStyle::SteadyBar,
            "block" => SetCursorStyle::DefaultUserShape,
            "underline" => SetCursorStyle::SteadyUnderScore,
            "blinking-beam" => SetCursorStyle::BlinkingBar,
            "blinking-block" => SetCursorStyle::BlinkingBlock,
            "blinking-underline" => SetCursorStyle::BlinkingUnderScore,
            _ => SetCursorStyle::SteadyBar,
        }
    }

    pub fn resolve_cursor_name_from_style(&self, style: &Option<SetCursorStyle>) -> &str {
        if let Some(style) = style {
            match style {
                SetCursorStyle::SteadyBar => "beam",
                SetCursorStyle::DefaultUserShape => "block",
                SetCursorStyle::SteadyUnderScore => "underline",
                SetCursorStyle::BlinkingBar => "blinking-beam",
                SetCursorStyle::BlinkingBlock => "blinking-block",
                SetCursorStyle::BlinkingUnderScore => "blinking-underline",
                _ => "beam",
            }
        } else {
            "Not found."
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
        assert!(config.word_count.is_none());
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
    fn test_config_change_mode() {
        let mut config = create_config();
        config.change_mode(ModeType::Time, Some(30));
        assert!(config.word_count.is_none());
        assert_mode(&config, ModeType::Time, 30);
    }

    #[test]
    fn test_config_change_theme() {
        let mut config = create_config();
        let theme_name = "Monokai Classic";
        config.change_theme(theme_name);
        assert_eq!(config.theme, Some(theme_name.to_string()));
    }

    #[test]
    fn test_config_toggles() {
        let mut config = create_config();
        config.toggle_numbers();
        config.toggle_punctuation();
        config.toggle_symbols();
    }

    #[test]
    fn test_config_resolvers() {
        let mut config = create_config();
        config.change_mode(ModeType::Time, Some(45));
        assert_eq!(config.resolve_duration(), 45);
        config.change_mode(ModeType::Words, Some(75));
        assert_eq!(config.resolve_word_count(), 75);
    }

    #[test]
    fn test_config_resolve_cursor_style() {
        let mut config = create_config();

        // the default
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingBar
        );

        config.cursor_style = Some("beam".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::SteadyBar
        );

        config.cursor_style = Some("block".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::DefaultUserShape
        );

        config.cursor_style = Some("underline".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::SteadyUnderScore
        );

        config.cursor_style = Some("blinking-beam".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingBar
        );

        config.cursor_style = Some("blinking-block".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingBlock
        );

        config.cursor_style = Some("blinking-underline".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingUnderScore
        );
    }
}
