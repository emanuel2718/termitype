use std::io;

use anyhow::Result;
use clap::Parser;
use config::Config;
use constants::APPNAME;
use crossterm::{
    cursor::SetCursorStyle,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use utils::get_config_dir;
use version::VERSION;

pub mod assets;
pub mod builder;
pub mod config;
pub mod constants;
pub mod debug;
pub mod error;
pub mod input;
pub mod log;
pub mod menu;
pub mod termi;
pub mod theme;
pub mod tracker;
#[path = "ui/ui.rs"]
pub mod ui;
pub mod utils;
pub mod version;

pub fn run() -> Result<()> {
    let config = Config::try_parse()?;

    // init logger
    if let Ok(log_dir) = get_config_dir() {
        let log_file = log_dir.join("debug.log"); // could be calle termitype.log
        if let Err(e) = log::init(log_file) {
            eprintln!("Failed to init termitype logger: {}", e);
        }
    }

    // NOTE: there should be a better way to do this
    if should_print_to_console(&config) {
        return Ok(());
    }

    let cursor_style = config.resolve_current_cursor_style();

    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor_style
    )?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let result = termi::run(&mut terminal, &config);

    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        SetCursorStyle::SteadyBar
    )?;
    terminal.show_cursor()?;
    result
}

fn should_print_to_console(config: &Config) -> bool {
    if config.version {
        println!("{} {}", APPNAME, VERSION);
        return true;
    }
    if config.list_themes {
        theme::print_theme_list();
        return true;
    }

    if config.list_languages {
        builder::print_language_list();
        return true;
    }
    false
}
