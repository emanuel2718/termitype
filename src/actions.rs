use crate::{app::App, config::Setting, error::AppError, menu::MenuContext, theme};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    NoOp,
    Quit,
    Restart,
    Start,
    Redo,

    Input(char),
    Backspace,

    MenuOpen(MenuContext),
    MenuClose,
    MenuGoBack,

    Toggle(Setting),
    ChangeTheme(String),

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
        Action::MenuOpen(ctx) => app.handle_menu_open(ctx),
        Action::MenuClose => app.handle_menu_close(),
        Action::MenuGoBack => app.handle_menu_backtrack(),
        Action::Toggle(setting) => app.config.toggle(setting),
        Action::ChangeLineCount(_) => app.handle_change_line_count(),
        Action::RandomizeTheme => theme::use_random_theme(),
        _ => Ok(()),
    }
}
