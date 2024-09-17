use clap::{ArgGroup, Parser};

/// Terminal-based typing game
#[derive(Parser, Debug)]
#[command(name = "TermiType", about = "Terminal-based typing test")]
#[command(group(
    ArgGroup::new("mode")
        .args(&["time", "words"])
        .required(false)
        .multiple(false)
))]
pub struct Config {
    /// Duration in seconds (only used in Time mode)
    #[arg(short = 't', long = "time", group = "mode")]
    pub time: Option<u64>,

    /// Number of words (only used in Words mode)
    #[arg(short = 'w', long = "words", group = "mode")]
    pub words: Option<usize>,
}

/// Represents the operational mode of the game
pub enum Mode {
    Time { duration: u64 },
    Words { word_count: usize },
}

impl Config {
    /// Determines the mode based on provided options.
    /// Defaults to Time mode with 309 seconds if no options are provided.
    pub fn determine_mode(&self) -> Mode {
        match (self.time, self.words) {
            (Some(time), None) => Mode::Time { duration: time },
            (None, Some(count)) => Mode::Words { word_count: count },
            (None, None) => Mode::Time { duration: 30 }, // Default mode
            _ => unreachable!("Clap's ArgGroup ensures only one option is set."),
        }
    }
}
