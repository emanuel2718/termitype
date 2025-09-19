use crate::constants::DEFAULT_LINE_COUNT;
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

    /// Cursor style variant
    #[arg(
        long = "cursor",
        value_name = "STYLE",
        help = "Cursor style: beam, block, underline, blinking-beam, blinking-block, blinking-underline"
    )]
    pub cursor: Option<String>,

    /// Picker style variant
    #[arg(
        long = "picker",
        value_name = "STYLE",
        help = "Picker style: quake, telescope, ivy, minimal"
    )]
    pub picker: Option<String>,

    /// Results display variant
    #[arg(
        long = "results",
        value_name = "STYLE",
        help = "Results style: graph, neofetch, minimal"
    )]
    pub results: Option<String>,

    /// Number of visible text lines
    #[arg(
        long = "lines",
        default_value_t = DEFAULT_LINE_COUNT,
        value_name = "COUNT",
        help = "Number of visible text lines"
    )]
    pub visible_lines: u8,

    /// Enables debug mode
    #[cfg(debug_assertions)]
    #[arg(short = 'd', long = "debug", help = "Enables debug mode")]
    pub debug: bool,

    /// Start the app showing the results screen (debug only)
    #[cfg(debug_assertions)]
    #[arg(long = "show-results", help = "Start with results screen shown")]
    pub show_results: bool,
}

impl Cli {
    pub fn clear_custom_words_flag(&mut self) {
        if self.words.is_some() {
            self.words = None
        }
    }
}
