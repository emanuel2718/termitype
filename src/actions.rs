use crate::{app::App, error::AppError, theme};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    NoOp,
    Quit,
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
        Action::Start => app.start(),
        Action::RandomizeTheme => theme::use_random_theme(),
        _ => Ok(()),
    }
}
