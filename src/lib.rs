use std::io;

use config::Config;
use constants::LOG_FILE;
use crossterm::{
    cursor::SetCursorStyle,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use helpers::get_config_dir;
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::helpers::should_print_to_console;

pub mod actions;
pub mod ascii;
pub mod assets;
pub mod builder;
pub mod config;
pub mod constants;
pub mod db;
pub mod error;
pub mod helpers;
pub mod input;
pub mod leaderboard;
pub mod log;
pub mod macros;
pub mod menu;
pub mod menu_builder;
pub mod modal;
pub mod persistence;
pub mod termi;
pub mod theme;
pub mod tracker;
pub mod ui;
pub mod version;

pub fn run() -> anyhow::Result<()> {
    let config = Config::try_parse()?;

    // init logger
    if let Ok(log_dir) = get_config_dir() {
        let log_file = log_dir.join(LOG_FILE);
        #[cfg(debug_assertions)]
        if let Err(e) = log::init(log_file, config.debug) {
            eprintln!("Failed to init termitype logger: {e}");
        }
        #[cfg(not(debug_assertions))]
        if let Err(e) = log::init(log_file, false) {
            eprintln!("Failed to init termitype logger: {e}");
        }
    }

    log_debug!("Debug logging enabled");
    log_info!("Starting termitype...");

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
