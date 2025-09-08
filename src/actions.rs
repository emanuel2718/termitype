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

    RandomizeTheme,
}

pub fn handle_action(app: &mut App, action: Action) -> Result<(), AppError> {
    match action {
        Action::NoOp => Ok(()),
        Action::Quit => app.quit(),
        Action::Restart => app.restart(),
        Action::Input(c) => app.handle_input(c),
        Action::Backspace => app.handle_backspace(),
        Action::RandomizeTheme => theme::use_random_theme(),
        _ => Ok(()),
    }
}
