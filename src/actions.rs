use crate::{
    app::App,
    config::Setting,
    error::AppError,
    menu::{MenuContext, MenuMotion},
    modal::ModalContext,
    theme,
    variants::{CursorVariant, PickerVariant, ResultsVariant},
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
    MenuToggle,
    MenuGoBack,
    MenuSelect,
    MenuInitSearch,
    MenuExitSearch,
    MenuBackspaceSearch,
    MenuUpdateSearch(String),

    ModalOpen(ModalContext),
    ModalInput(char),
    ModalBackspace,
    ModalConfirm,
    // ModalConfirm(Box<Action>),
    ModalClose,

    Toggle(Setting),

    SetLineCount(u8),
    SetTheme(String),
    SetCursorVariant(CursorVariant),
    SetPickerVariant(PickerVariant),
    SetResultVariant(ResultsVariant),

    SetTime(u16),
    SetWords(u16),
    SetAsciiArt(String),
    SetLanguage(String),

    RandomizeTheme,
    CyclePreviousArt,
    CycleNextArt,
}

pub fn handle_action(app: &mut App, action: Action) -> Result<(), AppError> {
    match action {
        Action::NoOp => Ok(()),
        Action::Quit => app.quit(),
        Action::Restart => app.restart(),
        Action::Redo => app.redo(),
        Action::Input(c) => app.handle_input(c),
        Action::Backspace => app.handle_backspace(),
        Action::MenuNav(motion) => app.handle_menu_navigate(motion),
        Action::MenuOpen(ctx) => app.handle_menu_open(ctx),
        Action::MenuShortcut(shortcut) => app.handle_menu_shortcut(shortcut),
        Action::MenuClose => app.handle_menu_close(),
        Action::MenuToggle => app.handle_menu_toggle(),
        Action::MenuGoBack => app.handle_menu_backtrack(),
        Action::MenuSelect => app.handle_menu_select(),
        Action::MenuInitSearch => app.handle_menu_init_search(),
        Action::MenuExitSearch => app.handle_menu_exit_search(),
        Action::MenuBackspaceSearch => app.handle_menu_backspace_search(),
        Action::MenuUpdateSearch(query) => app.handle_menu_update_search(query),
        Action::ModalOpen(ctx) => app.handle_modal_open(ctx),
        Action::ModalInput(c) => app.handle_modal_input(c),
        Action::ModalBackspace => app.handle_modal_backspace(),
        Action::ModalConfirm => app.handle_modal_confirm(),
        Action::ModalClose => app.handle_modal_close(),
        Action::Toggle(setting) => app.handle_toggle_setting(setting),
        Action::SetLineCount(count) => app.handle_set_line_count(count),
        Action::SetTheme(name) => theme::set_as_current_theme(&name),
        Action::SetCursorVariant(variant) => app.handle_set_cursor(variant),
        Action::SetPickerVariant(variant) => app.handle_set_picker(variant),
        Action::SetResultVariant(variant) => app.handle_set_result(variant),
        Action::SetTime(secs) => app.handle_set_time(secs as usize),
        Action::SetWords(count) => app.handle_set_words(count as usize),
        Action::SetLanguage(lang) => app.handle_set_language(lang),
        Action::SetAsciiArt(art) => app.handle_set_ascii_art(art),
        Action::RandomizeTheme => theme::use_random_theme(),
        Action::CycleNextArt => app.handle_cycle_prev_art(),
        Action::CyclePreviousArt => app.handle_cycle_next_art(),
        _ => Ok(()),
    }
}
