use crate::{app::App, config::Setting, error::AppError, theme};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    NoOp,
    Quit,
    Restart,
    Start,
    Resume,
    Redo,
    Pause,

    Toggle(Setting),
    ChangeTheme(String),

    Input(char),
    Backspace,

    ChangeLineCount(u8),
    RandomizeTheme,
}

pub fn handle_action(app: &mut App, action: Action) -> Result<(), AppError> {
    match action {
        Action::NoOp => Ok(()),
        Action::Quit => app.quit(),
        Action::Restart => app.restart(),
        Action::Redo => app.redo(),
        Action::Input(c) => app.handle_input(c),
        Action::Backspace => app.handle_backspace(),
        Action::Toggle(setting) => app.config.toggle(setting),
        Action::ChangeLineCount(_) => app.handle_change_line_count(),
        Action::RandomizeTheme => theme::use_random_theme(),
        _ => Ok(()),
    }
}
