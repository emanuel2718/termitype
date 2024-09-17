use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, Parser)]
#[command(name = "TermiType", about = "Terminal-based typing game")]
pub struct Config {
    /// Mode of the typing test (Time or Words)
    #[arg(long, default_value = "time", value_enum)]
    pub mode: Mode,

    /// Duration seconds (only used in Time mode)
    #[arg(long, default_value = "30", requires_if("mdoe", "time"))]
    pub time: u64,

    // Number of words (only used in Words mode)
    #[arg(long, default_value = "50", requires_if("mode", "words"))]
    pub words: usize,
}

// Available modes
#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    Time,
    Words,
}
