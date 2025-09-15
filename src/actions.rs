use crate::{
    app::App,
    config::Setting,
    error::AppError,
    menu::{MenuContext, MenuMotion},
    theme,
};
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

    MenuNav(MenuMotion),
    MenuOpen(MenuContext),
    MenuShortcut(char),
    MenuClose,
    MenuGoBack,
    MenuSelect,
    MenuInitSearch,
    MenuExitSearch,
    MenuUpdateSearch(String),

    Toggle(Setting),

    ChangeLineCount(u8),
    ChangeTheme(String),
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
        Action::MenuNav(motion) => todo!(),
        Action::MenuOpen(ctx) => app.handle_menu_open(ctx),
        Action::MenuShortcut(shortcut) => todo!(),
        Action::MenuClose => app.handle_menu_close(),
        Action::MenuGoBack => app.handle_menu_backtrack(),
        Action::MenuSelect => todo!(),
        Action::MenuInitSearch => todo!(),
        Action::MenuExitSearch => todo!(),
        Action::MenuUpdateSearch(query) => todo!(),
        Action::Toggle(setting) => app.config.toggle(setting),
        Action::ChangeLineCount(_) => app.handle_change_line_count(),
        Action::ChangeTheme(name) => theme::set_as_current_theme(&name),
        Action::RandomizeTheme => theme::use_random_theme(),
        _ => Ok(()),
    }
}
