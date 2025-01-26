use std::io;

use anyhow::Result;
use clap::Parser;
use config::Config;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};

pub mod builder;
pub mod config;
pub mod constants;
pub mod input;
pub mod termi;
pub mod theme;
pub mod tracker;
#[path = "ui/ui.rs"]
pub mod ui;
pub mod version;

pub fn run() -> Result<()> {
    let config = Config::try_parse()?;

    // NOTE: there should be a better way to do this
    if should_print_to_console(&config) {
        return Ok(());
    }

    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let result = termi::run(&mut terminal, &config);

    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

fn should_print_to_console(config: &Config) -> bool {
    if config.list_themes {
        theme::print_theme_list();
        return true;
    }
    false
}
