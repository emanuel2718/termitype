use std::str::FromStr;

use clap::{ArgGroup, Parser};
use crossterm::cursor::SetCursorStyle;

use crate::{
    builder::Builder,
    constants::{
        DEFAULT_CURSOR_STYLE, DEFAULT_LANGUAGE, DEFAULT_LINE_COUNT, DEFAULT_THEME,
        DEFAULT_TIME_MODE_DURATION, DEFAULT_WORD_MODE_COUNT,
    },
    persistence::Persistence,
    theme::{ColorSupport, Theme, ThemeLoader},
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
        help = "Sets the cursor style: 'beam', 'block', 'underline', 'blinking-beam', 'blinking-block', 'blinking-underline'"
    )]
    pub cursor_style: Option<String>,

    /// Number of visible lines in the test.
    #[arg(long = "lines", default_value_t = DEFAULT_LINE_COUNT)]
    pub visible_lines: u8,

    /// Prints termitype version
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    /// Enable debug mode
    #[cfg(debug_assertions)]
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Shows the current frames per second (FPS).
    #[arg(long = "show-fps")]
    pub show_fps: bool,

    /// Stores the persistence of the game. Set automatically.
    #[arg(skip)]
    persistent: Option<Persistence>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
            language: Some(DEFAULT_LANGUAGE.to_string()),
            cursor_style: Some(DEFAULT_CURSOR_STYLE.to_string()),
            visible_lines: DEFAULT_LINE_COUNT,
            color_mode: None,
            list_themes: false,
            list_languages: false,
            version: false,
            show_fps: false,
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

    fn override_with_persistence(&mut self) {
        if let Ok(persistence) = Persistence::new() {
            // Theme
            if self.theme.is_none() {
                if let Some(theme) = persistence.get("theme") {
                    if self.theme.is_none() && ThemeLoader::has_theme(theme) {
                        self.theme = Some(theme.to_string());
                    }
                } else {
                    self.theme = Some(DEFAULT_THEME.to_string())
                }
            }

            // Cursor
            if self.cursor_style.is_none() {
                if let Some(cursor) = persistence.get("cursor") {
                    self.change_cursor_style(cursor);
                } else {
                    self.cursor_style = Some(DEFAULT_CURSOR_STYLE.to_string())
                }
            }

            if self.time.is_none() && self.words.is_none() {
                // Mode and its value
                let mode_type = persistence
                    .get("mode")
                    .and_then(|mode| self.resolve_mode_from_str(mode));
                let mode_value = persistence
                    .get("mode_value")
                    .and_then(|val| val.parse::<usize>().ok());

                if let Some(mode_type) = mode_type {
                    self.change_mode(mode_type, mode_value);
                }
            }

            // Language
            if self.language.is_none() {
                if let Some(lang) = persistence.get("language") {
                    if Builder::has_language(lang) {
                        self.language = Some(lang.to_string());
                    }
                } else {
                    self.language = Some(DEFAULT_LANGUAGE.to_string())
                }
            }

            // symbols
            if !self.use_symbols {
                if let Some(use_symbols) = persistence.get("use_symbols") {
                    let val = match use_symbols {
                        "false" => false,
                        "true" => true,
                        _ => false,
                    };
                    self.set_symbols(val);
                }
            }

            // numbers
            if !self.use_numbers {
                if let Some(use_numbers) = persistence.get("use_numbers") {
                    let val = match use_numbers {
                        "false" => false,
                        "true" => true,
                        _ => false,
                    };
                    self.set_numbers(val);
                }
            }

            // punctuation
            if !self.use_punctuation {
                if let Some(use_punctuation) = persistence.get("use_punctuation") {
                    let val = match use_punctuation {
                        "false" => false,
                        "true" => true,
                        _ => false,
                    };
                    self.set_punctuation(val);
                }
            }

            self.persistent = Some(persistence);
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
                self.time = Some(value.unwrap_or(DEFAULT_TIME_MODE_DURATION) as u64);
                if let Some(persistence) = &mut self.persistent {
                    let _ = persistence.set("mode", "Time");
                    let _ = persistence.set("mode_value", &value.unwrap_or(30).to_string());
                }
            }
            ModeType::Words => {
                self.time = None;
                self.word_count = Some(value.unwrap_or(DEFAULT_WORD_MODE_COUNT));
                if let Some(persistence) = &mut self.persistent {
                    let _ = persistence.set("mode", "Words");
                    let _ = persistence.set(
                        "mode_value",
                        &value.unwrap_or(DEFAULT_WORD_MODE_COUNT).to_string(),
                    );
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

    /// Changes the language if available.
    pub fn change_language(&mut self, lang: &str) -> bool {
        if Builder::has_language(lang) {
            self.language = Some(lang.to_string());
            if let Some(persistence) = &mut self.persistent {
                let _ = persistence.set("language", lang);
            }
            true
        } else {
            false
        }
    }

    /// Chages the number of visible lines in the test.
    pub fn change_visible_lines(&mut self, lines: u8) {
        self.visible_lines = lines;
    }

    /// Chages the current style of the cursor.
    pub fn change_cursor_style(&mut self, style: &str) {
        self.cursor_style = Some(style.to_string());
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
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("mode_value", &value.to_string());
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
            _ => DEFAULT_WORD_MODE_COUNT,
        }
    }

    /// Resolves the test duration based on current configuration.
    pub fn resolve_duration(&self) -> u64 {
        match (self.time, self.word_count) {
            (Some(duration), None) => duration,
            _ => DEFAULT_TIME_MODE_DURATION as u64,
        }
    }

    /// Resolves the cursor style based on current configuration.
    pub fn resolve_current_cursor_style(&self) -> SetCursorStyle {
        if let Some(style) = &self.cursor_style {
            match style.as_str() {
                "beam" => SetCursorStyle::SteadyBar,
                "block" => SetCursorStyle::DefaultUserShape,
                "underline" => SetCursorStyle::SteadyUnderScore,
                "blinking-beam" => SetCursorStyle::BlinkingBar,
                "blinking-block" => SetCursorStyle::BlinkingBlock,
                "blinking-underline" => SetCursorStyle::BlinkingUnderScore,
                _ => SetCursorStyle::BlinkingBar, // default to beam style
            }
        } else {
            SetCursorStyle::BlinkingBar
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

    /// Chesk if the current terminal has proper color support. Mainly used for themes
    pub fn term_has_color_support(&self) -> bool {
        let color_support = self
            .color_mode
            .as_deref()
            .and_then(|s| ColorSupport::from_str(s).ok())
            .unwrap_or_else(Theme::detect_color_support);
        color_support.supports_themes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_default_state() {
        let config = Config::default();
        assert!(config.time.is_some());
        assert!(config.word_count.is_none());

        assert_eq!(config.language, Some(DEFAULT_LANGUAGE.to_string()));
        assert_eq!(config.theme, None);
        assert_eq!(config.visible_lines, DEFAULT_LINE_COUNT);

        assert!(!config.use_symbols);
        assert!(!config.use_punctuation);
        #[cfg(debug_assertions)]
        assert!(!config.debug);
        assert!(!config.show_fps);
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
        assert_eq!(config.theme, Some("Monokai Classic".to_string()));
    }

    #[test]
    fn test_config_change_visible_lines() {
        let mut config = create_config();
        assert_eq!(config.visible_lines, DEFAULT_LINE_COUNT);
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
