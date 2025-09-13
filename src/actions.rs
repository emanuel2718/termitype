use crate::{app::App, error::AppError, theme};
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
        Action::ChangeLineCount(_) => app.handle_change_line_count(),
        Action::RandomizeTheme => theme::use_random_theme(),
        _ => Ok(()),
    }
}
