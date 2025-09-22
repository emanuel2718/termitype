use crate::{
    cli::Cli,
    constants::{
        DEFAULT_LANGUAGE, DEFAULT_LINE_COUNT, DEFAULT_THEME, DEFAULT_TIME_MODE_DURATION_IN_SECS,
        DEFAULT_WORD_MODE_COUNT,
    },
    error::AppError,
    persistence::Persistence,
    theme::Theme,
    variants::{CursorVariant, PickerVariant, ResultsVariant},
};
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fmt, num::NonZeroUsize, time::Duration};

/// General settings that are toggleable
#[derive(Debug, Clone, PartialEq)]
pub enum Setting {
    Symbols,
    Numbers,
    Punctuation,
    LiveWPM,
}

/// Represents a typing test mode, either time-based or word-count based.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    Time(NonZeroUsize),
    Words(NonZeroUsize),
}

// TODO: maybe `duration` and `count()` are not needed anymore?
//      maybe with `is_time_mode()` and `value()` is enough?

impl Mode {
    /// Returns the duration of the test in seconds if is a time-limited test.
    pub fn duration(&self) -> Option<Duration> {
        if let Mode::Time(t) = self {
            Some(Duration::from_secs(t.get() as u64))
        } else {
            None
        }
    }

    /// Returns the number of words in the test word pool if is a word based test.
    pub fn count(&self) -> Option<usize> {
        if let Mode::Words(w) = self {
            Some(w.get())
        } else {
            None
        }
    }

    /// Returns true if this is a time-based mode.
    pub fn is_time_mode(&self) -> bool {
        matches!(self, Mode::Time(_))
    }

    /// Returns true if this is a word-count based mode.
    pub fn is_words_mode(&self) -> bool {
        matches!(self, Mode::Words(_))
    }

    /// Returns the value of the mode: seconds for time mode, word count for words mode.
    pub fn value(&self) -> usize {
        match self {
            Mode::Time(t) => t.get(),
            Mode::Words(w) => w.get(),
        }
    }

    /// Creates a new time-based Mode with the specified duration in seconds.
    ///
    /// This is a convenience method for creating a time-limited typing test.
    /// If secs is 0, it uses the default duration.
    ///
    /// # Arguments
    /// * `secs` - The duration of the test in seconds
    ///
    /// # Returns
    /// The new Mode
    ///
    /// # Examples
    /// ```
    /// use termitype::config::Mode;
    /// let mode = Mode::with_time(60); // 1-minute test
    /// ```
    pub fn with_time(secs: usize) -> Self {
        Mode::Time(
            NonZeroUsize::new(secs)
                .unwrap_or(NonZeroUsize::new(DEFAULT_TIME_MODE_DURATION_IN_SECS).unwrap()),
        )
    }

    /// Creates a new word-count based Mode with the specified number of words.
    ///
    /// This is a convenience method for creating a word-count limited typing test.
    /// If count is 0, it uses the default word count.
    ///
    /// # Arguments
    /// * `count` - The number of words to type
    ///
    /// # Returns
    /// The new Mode
    ///
    /// # Examples
    /// ```
    /// use termitype::config::Mode;
    /// let mode = Mode::with_words(50); // 50-word test
    /// ```
    pub fn with_words(count: usize) -> Self {
        Mode::Words(
            NonZeroUsize::new(count).unwrap_or(NonZeroUsize::new(DEFAULT_WORD_MODE_COUNT).unwrap()),
        )
    }

    /// Creats a new time-based Mode with the default time of `30 seconds`
    ///
    /// ```
    /// use termitype::constants::DEFAULT_TIME_MODE_DURATION_IN_SECS;
    /// assert_eq!(DEFAULT_TIME_MODE_DURATION_IN_SECS, 30);
    ///
    /// use termitype::config::Mode;
    /// let mode = Mode::with_default_time();
    /// ```
    pub fn with_default_time() -> Self {
        Mode::Time(NonZeroUsize::new(DEFAULT_TIME_MODE_DURATION_IN_SECS).unwrap())
    }

    /// Creats a new words-based Mode with the default word count of `50 words`
    ///
    /// ```
    /// use termitype::constants::DEFAULT_WORD_MODE_COUNT;
    /// assert_eq!(DEFAULT_WORD_MODE_COUNT, 50);
    ///
    /// use termitype::config::Mode;
    /// let mode = Mode::with_default_words();
    /// ```
    pub fn with_default_words() -> Self {
        Mode::Words(NonZeroUsize::new(DEFAULT_WORD_MODE_COUNT).unwrap())
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Time(t) => write!(f, "Time: {} seconds", t.get()),
            Mode::Words(w) => write!(f, "Words: {}", w.get()),
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::with_time(DEFAULT_TIME_MODE_DURATION_IN_SECS)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigState {
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub numbers: bool,
    #[serde(default)]
    pub symbols: bool,
    #[serde(default)]
    pub punctuation: bool,
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub cursor_variant: CursorVariant,
    #[serde(default)]
    pub picker_variant: PickerVariant,
    #[serde(default)]
    pub results_variant: ResultsVariant,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub lines: u8,
    #[serde(default)]
    pub hide_live_wpm: bool,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            debug: false,
            mode: Mode::default(),
            numbers: false,
            symbols: false,
            punctuation: false,
            lines: DEFAULT_LINE_COUNT,
            language: Some(DEFAULT_LANGUAGE.to_string()),
            theme: Some(DEFAULT_THEME.to_string()),
            cursor_variant: CursorVariant::default(),
            picker_variant: PickerVariant::default(),
            results_variant: ResultsVariant::default(),
            hide_live_wpm: false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub cli: Cli,
    state: ConfigState,
    persistence: Persistence,
}

// impl Default for Config {}

impl Config {
    pub fn new() -> Result<Self> {
        let args = Cli::parse();
        let persistence = Persistence::new()?;
        let mut config = Self {
            cli: args.clone(),
            state: Self::load_state(&persistence)?,
            persistence,
        };
        if config.state.theme.is_none() {
            config.state.theme = Some(DEFAULT_THEME.to_string());
        }
        config.apply_cli_args(args);
        config.persist()?;
        Ok(config)
    }

    fn load_state(p: &Persistence) -> Result<ConfigState> {
        if let Some(j) = p.get("config") {
            Ok(serde_json::from_str(j)?)
        } else {
            Ok(ConfigState::default())
        }
    }

    pub fn persist(&mut self) -> Result<()> {
        let json = serde_json::to_string(&self.state)?;
        let _ = self.persistence.set("config", &json);
        self.persistence.flush()?;
        Ok(())
    }

    pub(crate) fn apply_cli_args(&mut self, cli: Cli) {
        if let Some(time) = cli.time {
            self.state.mode = Mode::with_time(time as usize);
            // TODO: maybe is not a good idea to internally unwrap the option. It could be confusing for the user
            // if let Ok(mode) = Mode::with_time(time as usize) {
            //     self.state.mode = mode;
            // }
        }

        // NOTE: this is wrong. Currently `with_words` assumes we pass the number of words but
        // really what we want to pass is the words that the test itself is going to use.
        if let Some(count) = cli.words_count {
            self.state.mode = Mode::with_words(count)
        }

        if let Some(words_str) = &cli.words {
            let word_count = words_str.split_whitespace().count();
            self.state.mode = Mode::with_words(word_count);
        }

        if let Some(theme_str) = &cli.theme {
            if theme_str.parse::<Theme>().is_ok() {
                self.state.theme = Some(theme_str.clone())
            }
        }

        if let Some(cursor_str) = &cli.cursor {
            if let Ok(variant) = cursor_str.parse::<CursorVariant>() {
                self.state.cursor_variant = variant;
            }
        }

        if let Some(picker_str) = &cli.picker {
            if let Ok(variant) = picker_str.parse::<PickerVariant>() {
                self.state.picker_variant = variant;
            }
        }

        if let Some(results_str) = &cli.results {
            if let Ok(variant) = results_str.parse::<ResultsVariant>() {
                self.state.results_variant = variant;
            }
        }

        if cli.use_symbols {
            self.state.symbols = true;
        }

        if cli.use_numbers {
            self.state.numbers = true;
        }

        if cli.use_punctuation {
            self.state.punctuation = true;
        }

        if cli.hide_live_wpm {
            self.state.hide_live_wpm = true;
        }

        self.state.lines = cli.visible_lines;

        #[cfg(debug_assertions)]
        if cli.debug {
            self.state.debug = true;
        }
    }

    #[cfg(debug_assertions)]
    pub fn is_debug(&self) -> bool {
        self.state.debug
    }

    pub fn current_mode(&self) -> Mode {
        self.state.mode
    }

    pub fn current_language(&self) -> String {
        if self.cli.words.is_some() {
            "Custom".to_string()
        } else {
            self.state
                .language
                .clone()
                .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string())
        }
    }

    pub fn current_theme(&self) -> Option<String> {
        self.state.theme.clone()
    }

    pub fn current_cursor_variant(&self) -> CursorVariant {
        self.state.cursor_variant
    }

    pub fn current_picker_variant(&self) -> PickerVariant {
        self.state.picker_variant
    }

    pub fn current_results_variant(&self) -> ResultsVariant {
        self.state.results_variant
    }

    pub fn current_line_count(&self) -> u8 {
        self.state.lines
    }

    pub fn change_theme(&mut self, theme: Theme) {
        self.state.theme = Some(theme.id.to_string())
    }

    pub fn change_language(&mut self, lang: String) {
        self.state.language = Some(lang);
    }

    pub fn change_mode(&mut self, mode: Mode) -> Result<()> {
        self.state.mode = mode;
        Ok(())
    }

    pub fn change_cursor_variant(&mut self, variant: CursorVariant) {
        self.state.cursor_variant = variant;
    }

    pub fn change_picker_variant(&mut self, variant: PickerVariant) {
        self.state.picker_variant = variant;
    }

    pub fn change_results_variant(&mut self, variant: ResultsVariant) {
        self.state.results_variant = variant;
    }

    pub fn change_visible_lines_count(&mut self, count: u8) {
        self.state.lines = count;
    }

    pub fn should_hide_live_wpm(&self) -> bool {
        self.state.hide_live_wpm
    }

    pub fn is_enabled(&self, setting: Setting) -> bool {
        match setting {
            Setting::Symbols => self.state.symbols,
            Setting::Numbers => self.state.numbers,
            Setting::Punctuation => self.state.punctuation,
            Setting::LiveWPM => !self.state.hide_live_wpm,
        }
    }

    pub fn toggle(&mut self, setting: Setting) -> Result<(), AppError> {
        match setting {
            Setting::Symbols => self.state.symbols = !self.state.symbols,
            Setting::Numbers => self.state.numbers = !self.state.numbers,
            Setting::Punctuation => self.state.punctuation = !self.state.punctuation,
            Setting::LiveWPM => self.state.hide_live_wpm = !self.state.hide_live_wpm,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let config = Config::default();
        assert!(config.current_mode().is_time_mode());
        assert!(!config.current_mode().is_words_mode());
        assert_eq!(
            config.current_mode().value(),
            DEFAULT_TIME_MODE_DURATION_IN_SECS
        );
        assert_eq!(config.current_line_count(), DEFAULT_LINE_COUNT);
        assert_eq!(config.current_language(), DEFAULT_LANGUAGE.to_string());
        assert!(!config.is_enabled(Setting::Symbols));
        assert!(!config.is_enabled(Setting::Numbers));
        assert!(!config.is_enabled(Setting::Punctuation));
    }

    #[test]
    fn test_mode_constructor() {
        let time_mode = Mode::with_time(65);
        assert_eq!(time_mode.duration(), Some(Duration::from_secs(65)));
        assert_eq!(time_mode.value(), 65);
        assert_eq!(time_mode.count(), None);

        let word_mode = Mode::with_words(15);
        assert_eq!(word_mode.count(), Some(15));
        assert_eq!(word_mode.value(), 15);
        assert_eq!(word_mode.duration(), None);
    }

    #[test]
    fn test_toggles() {
        let mut config = Config::default();

        assert!(!config.is_enabled(Setting::Symbols));
        config.toggle(Setting::Symbols).unwrap();
        assert!(config.is_enabled(Setting::Symbols));

        assert!(!config.is_enabled(Setting::Punctuation));
        config.toggle(Setting::Punctuation).unwrap();
        assert!(config.is_enabled(Setting::Punctuation));

        assert!(!config.state.hide_live_wpm);
        assert!(!config.should_hide_live_wpm());
        config.toggle(Setting::LiveWPM).unwrap();
        assert!(config.should_hide_live_wpm())
    }

    #[test]
    fn test_change_mode() {
        let mut config = Config::default();

        config.change_mode(Mode::with_time(150)).unwrap();
        assert_eq!(
            config.current_mode().duration(),
            Some(Duration::from_secs(150))
        );
        assert!(config.current_mode().is_time_mode());
        assert_eq!(config.current_mode().value(), 150);

        config.change_mode(Mode::with_words(79)).unwrap();
        assert_eq!(config.current_mode().count(), Some(79));
        assert!(config.current_mode().is_words_mode());
        assert_eq!(config.current_mode().value(), 79);
    }

    #[test]
    fn test_change_variants() {
        let mut config = Config::default();
        assert_eq!(config.current_cursor_variant(), CursorVariant::default());
        assert_eq!(config.current_picker_variant(), PickerVariant::default());
        assert_eq!(config.current_results_variant(), ResultsVariant::default());

        config.change_cursor_variant(CursorVariant::Underline);
        config.change_picker_variant(PickerVariant::Telescope);
        config.change_results_variant(ResultsVariant::Neofetch);

        assert_eq!(config.current_cursor_variant(), CursorVariant::Underline);
        assert_eq!(config.current_picker_variant(), PickerVariant::Telescope);
        assert_eq!(config.current_results_variant(), ResultsVariant::Neofetch);
    }

    #[test]
    fn test_current_language_with_custom_words() {
        let custom_word = "pog".to_string();
        let mut cli = Cli::default();
        let mut config = Config::default();
        cli.words = Some(custom_word.clone());
        config.cli = cli;

        assert_eq!(config.cli.words, Some(custom_word));
        assert_eq!(config.current_language(), "Custom");
    }

    #[test]
    fn test_custom_words_should_set_word_mode_with_count() {
        let custom_word = "about too soon".to_string();
        let cli = Cli {
            words: Some(custom_word),
            ..Default::default()
        };
        let mut config = Config {
            cli: Cli::default(),
            ..Default::default()
        };

        config.apply_cli_args(cli);

        assert_eq!(
            config.current_mode(),
            Mode::Words(NonZeroUsize::new(3).unwrap())
        );
    }
}
