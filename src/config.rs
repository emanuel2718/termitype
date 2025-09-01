use crate::{
    cli::Cli,
    constants::{DEFAULT_LANGUAGE, DEFAULT_TIME_MODE_DURATION_IN_SECS, DEFAULT_WORD_MODE_COUNT},
    persistence::Persistence,
};
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fmt, num::NonZeroUsize, time::Duration};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimeModeValue(NonZeroUsize);
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct WordsModeValue(NonZeroUsize);

impl TimeModeValue {
    pub fn new(value: usize) -> Self {
        Self(
            NonZeroUsize::new(value)
                .unwrap_or(NonZeroUsize::new(DEFAULT_TIME_MODE_DURATION_IN_SECS).unwrap()),
        )
    }

    fn get(&self) -> usize {
        self.0.get()
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.get() as u64)
    }
}

impl WordsModeValue {
    pub fn new(value: usize) -> Self {
        Self(
            NonZeroUsize::new(value).unwrap_or(NonZeroUsize::new(DEFAULT_WORD_MODE_COUNT).unwrap()),
        )
    }

    pub fn count(&self) -> usize {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ModeValue {
    Time(TimeModeValue),
    Words(WordsModeValue),
}

impl ModeValue {
    pub fn duration(&self) -> Option<Duration> {
        match self {
            ModeValue::Time(time) => Some(time.duration()),
            ModeValue::Words(_) => None,
        }
    }

    pub fn count(&self) -> Option<usize> {
        match self {
            ModeValue::Time(_) => None,
            ModeValue::Words(words) => Some(words.count()),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ModeKind {
    Time,
    Words,
}

/// Represents a typing test mode, either time-based or word-count based.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Mode {
    kind: ModeKind,
    value: ModeValue,
}

impl Mode {
    /// Returns the duration of the test in seconds if is a time-limited test.
    pub fn duration(&self) -> Option<Duration> {
        self.value.duration()
    }

    /// Returns the number of words in the test word pool if is a word based test.
    pub fn count(&self) -> Option<usize> {
        self.value.count()
    }

    /// Returns true if this is a time-based mode.
    pub fn is_time_mode(&self) -> bool {
        matches!(self.kind, ModeKind::Time)
    }

    /// Returns true if this is a word-count based mode.
    pub fn is_words_mode(&self) -> bool {
        matches!(self.kind, ModeKind::Words)
    }

    /// Returns the value of the mode: seconds for time mode, word count for words mode.
    pub fn value(&self) -> usize {
        match &self.value {
            ModeValue::Time(t) => t.0.get(),
            ModeValue::Words(w) => w.0.get(),
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
        Self::change_mode(ModeKind::Time, secs)
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
        Self::change_mode(ModeKind::Words, count)
    }

    /// Internal helper function to create a Mode based on the given kind and value.
    ///
    /// This function handles the common logic for both time and words modes,
    /// constructing the appropriate ModeValue. If val is 0, it uses the default value.
    ///
    /// # Arguments
    /// * `kind` - The type of mode to create
    /// * `val` - The value (seconds for Time, count for Words)
    ///
    /// # Returns
    /// The new Mode
    fn change_mode(kind: ModeKind, val: usize) -> Self {
        match kind {
            ModeKind::Time => Self {
                kind: ModeKind::Time,
                value: ModeValue::Time(TimeModeValue::new(val)),
            },
            ModeKind::Words => Self {
                kind: ModeKind::Words,
                value: ModeValue::Words(WordsModeValue::new(val)),
            },
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            ModeValue::Time(t) => write!(f, "Time: {} seconds", t.0.get()),
            ModeValue::Words(w) => write!(f, "Words: {}", w.0.get()),
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
    pub mode: Mode,
    pub language: Option<String>,
    pub numbers: bool,
    pub symbols: bool,
    pub punctuation: bool,
    #[serde(default)]
    pub debug: bool,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            language: Some(DEFAULT_LANGUAGE.to_string()),
            numbers: false,
            symbols: false,
            punctuation: false,
            debug: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    state: ConfigState,
    persistence: Persistence,
}

// impl Default for Config {}

impl Config {
    pub fn new() -> Result<Self> {
        let args = Cli::parse();
        let persistence = Persistence::new()?;
        let mut config = Self {
            state: Self::load_state(&persistence)?,
            persistence,
        };
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

    fn persist(&mut self) -> Result<()> {
        let json = serde_json::to_string(&self.state)?;
        let _ = self.persistence.set("config", &json);
        self.persistence.flush()?;
        Ok(())
    }

    fn apply_cli_args(&mut self, cli: Cli) {
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

        if cli.use_symbols {
            self.state.symbols = true;
        }

        if cli.use_numbers {
            self.state.numbers = true;
        }

        if cli.use_punctuation {
            self.state.punctuation = true;
        }

        #[cfg(debug_assertions)]
        if cli.debug {
            self.state.debug = true;
        }
    }

    pub fn current_mode(&self) -> Mode {
        self.state.mode
    }

    pub fn current_language(&self) -> String {
        self.state
            .language
            .clone()
            .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string())
    }

    pub fn change_mode(&mut self, mode: Mode) -> Result<()> {
        self.state.mode = mode;
        Ok(())
    }

    pub fn using_symbols(&self) -> bool {
        self.state.symbols
    }

    pub fn using_numbers(&self) -> bool {
        self.state.numbers
    }

    pub fn using_punctuation(&self) -> bool {
        self.state.punctuation
    }

    #[cfg(debug_assertions)]
    pub fn is_debug(&self) -> bool {
        self.state.debug
    }

    pub fn toggle_symbols(&mut self) {
        self.state.symbols = !self.state.symbols;
    }

    pub fn toggle_numbers(&mut self) {
        self.state.numbers = !self.state.numbers;
    }

    pub fn toggle_punctuation(&mut self) {
        self.state.punctuation = !self.state.punctuation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let config = Config::default();
        assert_eq!(config.current_mode().kind, ModeKind::Time);
        assert_eq!(
            config.current_mode().value(),
            DEFAULT_TIME_MODE_DURATION_IN_SECS
        );
        assert_eq!(config.current_language(), DEFAULT_LANGUAGE.to_string());
        assert!(!config.using_numbers());
        assert!(!config.using_symbols());
        assert!(!config.using_punctuation());
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
        assert!(!config.using_symbols());
        config.toggle_symbols();
        assert!(config.using_symbols());
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
}
