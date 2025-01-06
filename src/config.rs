use clap::{ArgGroup, Parser};

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

    /// Introduces symbols to the test words.
    #[arg(short = 's', long = "use-symbols", value_name = "SYMBOLS")]
    pub use_symbols: bool,

    /// Introduces numbers to the test words
    #[arg(short = 'n', long = "use-numbers", value_name = "NUMBERS")]
    pub use_numbers: bool,

    /// Sets the theme if a valid theme is given, ignored otherwise
    #[arg(long = "theme", value_name = "THEME")]
    pub use_theme: Option<String>,
}

/// Represents the operationlal mode of the game>
pub enum Mode {
    Time { duration: u64 },
    Words { count: usize },
}

impl Config {
    /// Resolves the mode based onf the provided arguments
    /// Defaults to time mode with (30) seconds if no options are provided.
    /// If *both* `time` and `word` mode are passed, it will default to time mode.
    pub fn resolve_mode(&self) -> Mode {
        match (self.time, self.words) {
            (Some(time), None) => Mode::Time { duration: time },
            (None, Some(count)) => Mode::Words { count },
            (None, None) => Mode::Time { duration: 30 },
            _ => unreachable!("Both Time mode and Words mode cannot be used at the same time."),
        }
    }

    pub fn resolve_word_count(&self) -> usize {
        match (self.time, self.words) {
            (None, Some(count)) => count,
            _ => 100,
        }
    }

    pub fn resolve_duration(&self) -> u64 {
        match (self.time, self.words) {
            (Some(time), None) => time,
            _ => 30,
        }
    }

    /// Toggles the presence of numbers in the test word pool.
    pub fn toggle_use_numbers(&mut self) {
        self.use_numbers = !self.use_numbers;
    }

    /// Toggles the presence of symbols in the test word pool.
    pub fn toggle_use_symbols(&mut self) {
        self.use_symbols = !self.use_symbols;
    }
}
