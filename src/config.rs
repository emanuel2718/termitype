use clap::{ArgGroup, Parser};
use crossterm::cursor::SetCursorStyle;

use crate::{
    constants::{AMOUNT_OF_VISIBLE_LINES, DEFAULT_CURSOR_STYLE, DEFAULT_LANGUAGE},
    log,
    persistence::Persistence,
    theme::ThemeLoader,
};

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
    #[arg(short, long, default_value = DEFAULT_LANGUAGE, value_name = "LANG")]
    pub language: String,

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
    #[arg(long = "list-themes")]
    pub list_themes: bool,

    /// Lists the available languages
    #[arg(long = "list-languages")]
    pub list_languages: bool,

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
        default_value = DEFAULT_CURSOR_STYLE,
        help = "Sets the cursor style: 'beam', 'block', 'underline', 'blinking-beam', 'blinking-block', 'blinking-underline'"
    )]
    pub cursor_style: String,

    /// Number of visible lines in the test.
    #[arg(long = "lines", default_value_t = AMOUNT_OF_VISIBLE_LINES)]
    pub visible_lines: u8,

    /// Prints termitype version
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    /// Enable debug mode
    #[cfg(debug_assertions)]
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    #[arg(skip)]
    persistent: Option<Persistence>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModeType {
    Time = 0,
    Words = 1,
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
            time: Some(30),
            words: None,
            word_count: None,
            use_symbols: false,
            use_numbers: false,
            use_punctuation: false,
            theme: None,
            language: DEFAULT_LANGUAGE.to_string(),
            cursor_style: DEFAULT_CURSOR_STYLE.to_string(),
            visible_lines: AMOUNT_OF_VISIBLE_LINES,
            color_mode: None,
            list_themes: false,
            list_languages: false,
            version: false,
            #[cfg(debug_assertions)]
            debug: false,
            persistent: None,
        }
    }
}

impl Config {
    /// Returns a new instance of the Config struct with default values.
    pub fn new() -> Self {
        let mut config = Self::default();
        Self::override_with_persistence(&mut config);
        config
    }

    fn override_with_persistence(config: &mut Config) {
        if let Ok(persistence) = Persistence::new() {
            // Theme
            if let Some(theme) = persistence.get("theme") {
                if ThemeLoader::has_theme(theme) {
                    config.theme = Some(theme.to_string());
                }
            }

            // Cursor
            if let Some(cursor) = persistence.get("cursor") {
                Self::change_cursor_style(config, cursor);
            }

            // Mode
            if let Some(mode) = persistence.get("mode") {
                if let Some(mode_type) = Self::resolve_mode_from_str(config, mode) {
                    log::info("Chaning mode");
                    Self::change_mode(config, mode_type, None);
                }
            }

            // TODO: language
            // TODO: mode value (time, word_count)

            // symbols
            if let Some(use_symbols) = persistence.get("use_symbols") {
                let val = match use_symbols {
                    "false" => false,
                    "true" => true,
                    _ => false,
                };
                Self::set_symbols(config, val);
            }

            // numbers
            if let Some(use_numbers) = persistence.get("use_numbers") {
                let val = match use_numbers {
                    "false" => false,
                    "true" => true,
                    _ => false,
                };
                Self::set_numbers(config, val);
            }

            // punctuation
            if let Some(use_punctuation) = persistence.get("use_punctuation") {
                let val = match use_punctuation {
                    "false" => false,
                    "true" => true,
                    _ => false,
                };
                Self::set_punctuation(config, val);
            }

            config.persistent = Some(persistence);
        }
    }

    pub fn try_parse() -> Result<Self, clap::Error> {
        let mut config = <Self as Parser>::try_parse()?;
        Self::override_with_persistence(&mut config);
        Ok(config)
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
                if let Some(persistence) = &mut self.persistent {
                    let _ = persistence.set("mode", "Time");
                }
            }
            ModeType::Words => {
                self.time = None;
                self.word_count = Some(value.unwrap_or(25));
                if let Some(persistence) = &mut self.persistent {
                    let _ = persistence.set("mode", "Words");
                }
            }
        }
    }

    /// Chages the current theme of the game.
    pub fn change_theme(&mut self, theme_name: &str) {
        self.theme = Some(theme_name.to_string());
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("theme", theme_name);
        }
    }

    /// Chages the number of visible lines in the test.
    pub fn change_visible_lines(&mut self, lines: u8) {
        self.visible_lines = lines;
    }

    /// Chages the current style of the cursor.
    pub fn change_cursor_style(&mut self, style: &str) {
        self.cursor_style = style.to_string();
        // TODO: there must be a better way to do this
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("cursor", style);
        }
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

    fn resolve_mode_from_str(&self, mode: &str) -> Option<ModeType> {
        match mode {
            "Time" => Some(ModeType::Time),
            "Words" => Some(ModeType::Words),
            _ => None,
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
        match self.cursor_style.as_str() {
            "beam" => SetCursorStyle::SteadyBar,
            "block" => SetCursorStyle::DefaultUserShape,
            "underline" => SetCursorStyle::SteadyUnderScore,
            "blinking-beam" => SetCursorStyle::BlinkingBar,
            "blinking-block" => SetCursorStyle::BlinkingBlock,
            "blinking-underline" => SetCursorStyle::BlinkingUnderScore,
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

    fn set_numbers(&mut self, val: bool) {
        self.use_numbers = val;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("use_numbers", val.to_string().as_str());
        }
    }

    fn set_symbols(&mut self, val: bool) {
        self.use_symbols = val;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("use_symbols", val.to_string().as_str());
        }
    }

    fn set_punctuation(&mut self, val: bool) {
        self.use_punctuation = val;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("use_punctuation", val.to_string().as_str());
        }
    }

    /// Toggles the presence of numbers in the test word pool.
    pub fn toggle_numbers(&mut self) {
        let val = !self.use_numbers;
        self.use_numbers = val;
        self.set_numbers(val);
    }

    /// Toggles the presence of punctuation in the test word pool.
    pub fn toggle_punctuation(&mut self) {
        let val = !self.use_punctuation;
        self.use_punctuation = val;
        self.set_punctuation(val);
    }

    /// Toggles the presence of symbols in the test word pool.
    pub fn toggle_symbols(&mut self) {
        let val = !self.use_symbols;
        self.use_symbols = val;
        self.set_symbols(val);
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
        assert!(config.time.is_some());
        assert!(config.word_count.is_none());

        assert_eq!(config.language, DEFAULT_LANGUAGE.to_string());
        assert_eq!(config.theme, None);
        assert_eq!(config.visible_lines, AMOUNT_OF_VISIBLE_LINES);

        assert_eq!(config.use_symbols, false);
        assert_eq!(config.use_punctuation, false);
        #[cfg(debug_assertions)]
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
        assert_eq!(config.theme, None);
    }

    #[test]
    fn test_config_change_visible_lines() {
        let mut config = create_config();
        assert_eq!(config.visible_lines, AMOUNT_OF_VISIBLE_LINES);
        config.change_visible_lines(10);
        assert_eq!(config.visible_lines, 10);
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

        config.cursor_style = "beam".to_string();
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::SteadyBar
        );

        config.cursor_style = "block".to_string();
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::DefaultUserShape
        );

        config.cursor_style = "underline".to_string();
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::SteadyUnderScore
        );

        config.cursor_style = "blinking-beam".to_string();
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingBar
        );

        config.cursor_style = "blinking-block".to_string();
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingBlock
        );

        config.cursor_style = "blinking-underline".to_string();
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingUnderScore
        );
    }
}
