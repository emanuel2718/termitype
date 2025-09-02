use crate::config::Config;
use crossterm::{
    cursor::SetCursorStyle,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::io;

pub mod app;
pub mod cli;
pub mod config;
pub mod constants;
pub mod error;
pub mod logger;
pub mod persistence;
pub mod utils;
pub mod variants;

pub mod prelude {
    #[cfg(debug_assertions)]
    pub use crate::log_debug;
    pub use crate::{log_error, log_info, log_warn};
}

pub fn start() -> anyhow::Result<()> {
    let config = Config::new()?;
    logger::init()?;

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
