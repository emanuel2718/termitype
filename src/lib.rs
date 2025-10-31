use crate::{config::Config, db::reset_database, persistence::reset_persistence};
use clap::Parser;
use crossterm::{
    cursor::SetCursorStyle,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::io;

pub mod actions;
pub mod app;
pub mod ascii;
pub mod assets;
pub mod builders;
pub mod cli;
pub mod common;
pub mod config;
pub mod constants;
pub mod db;
pub mod error;
pub mod input;
pub mod leaderboard;
pub mod logger;
pub mod menu;
pub mod modal;
pub mod notifications;
pub mod persistence;
pub mod theme;
pub mod tracker;
pub mod tui;
pub mod variants;

pub mod prelude {
    #[cfg(debug_assertions)]
    pub use crate::log_debug;
    pub use crate::{log_error, log_info, log_warn};
}

pub fn start() -> anyhow::Result<()> {
    logger::init()?;
    let args = cli::Cli::parse();
    args.validate().map_err(|e| anyhow::anyhow!(e))?;

    if args.reset {
        reset_persistence()?;
        reset_database()?;
        println!("[ OK ] Successfully reset termitype back to default state.");
        return Ok(());
    }

    let config = Config::new(args)?;

    let crossterm_cursor = config.current_cursor_variant().to_crossterm();

    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        crossterm_cursor
    )?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let out = app::run(&mut terminal, &config);

    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        SetCursorStyle::DefaultUserShape
    )?;

    terminal.show_cursor()?;

    out
}
