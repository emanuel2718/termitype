use crate::{app::App, theme};
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

pub fn handle_action(app: &mut App, action: Action) -> Result<()> {
    match action {
        Action::NoOp => Ok(()),
        Action::Quit => {
            app.quit();
            Ok(())
        }
        Action::Start => {
            app.start();
            Ok(())
        }
        Action::RandomizeTheme => theme::randomize_theme(),
        _ => Ok(()),
    }
}
