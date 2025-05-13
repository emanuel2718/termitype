use crossterm::execute;
use ratatui::layout::Position;

use crate::{
    config::ModeType,
    constants::DEFAULT_LINE_COUNT,
    log_debug,
    modal::{build_modal, handle_modal_confirm, ModalContext},
    termi::Termi,
    theme::Theme,
    tracker::Status,
    ui::render::TermiClickableRegions,
};

/// Top-Level actions
#[derive(Debug, Clone, PartialEq)]
pub enum TermiAction {
    // === Global ===
    Quit,
    NoOp, // input that results in no state change

    // === State ===
    Start,
    TogglePause,

    // === Typing ===
    Input(char),
    Backspace,

    // TODO: think if this is right
    // === Menu ===
    MenuOpen(MenuContext),
    MenuNavigate(MenuNavAction),
    MenuSearch(MenuSearchAction),
    MenuClose,
    MenuSelect,

    // === Modal ===
    ModalOpen(ModalContext),
    ModalInput(char),
    ModalClose,
    ModalConfirm,
    ModalBackspace,

    // === Configuration/State Changes ===
    ChangeTheme(String),
    // ChangePreview(PreviewType),
    ChangeLanguage(String),
    ChangeCursor(String),
    ChangeMode(ModeType, Option<usize>),
    ChangeTime(u64),
    ChangeVisibleLines(u64),
    ChangeWordCount(usize),

    // === Toggles ===
    TogglePunctuation,
    ToggleNumbers,
    ToggleSymbols,

    // === Previews ===
    ApplyPreview(PreviewType),
    ClearPreview,
}

// ============== MENU ==============

/// User navigation through the menus
#[derive(Debug, Clone, PartialEq)]
pub enum MenuNavAction {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Back,
}

/// Actions that can be taken during search
#[derive(Debug, Clone, PartialEq)]
pub enum MenuSearchAction {
    Input(char),
    Backspace,
    Clear,
    Start,
    Confirm,
    Close,
}

/// Specifies available menus
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuContext {
    Root,
    Theme,
    Language,
    Cursor,
    Mode,
    Time,
    Words,
    LineCount,
    About,
}

// ============== PREVIEW ==============

#[derive(Debug, Clone, PartialEq)]
pub enum PreviewType {
    Theme(String),
    Cursor(String),
}

// ============== CLICK ACTIONS ==============

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TermiClickAction {
    SwitchMode(ModeType),
    SetModeValue(usize),
    ToggleMenu,
    TogglePunctuation,
    ToggleSymbols,
    ToggleNumbers,
    ToggleThemePicker,
    ToggleLanguagePicker,
    ToggleAbout,
    ToggleModal(ModalContext),
    ModalConfirm,
}
pub fn handle_click_action(
    termi: &mut Termi,
    reg: &TermiClickableRegions,
    x: u16,
    y: u16,
) -> Option<TermiAction> {
    let mut clicked_action: Option<TermiClickAction> = None;
    for (rect, action) in reg.regions.iter().rev() {
        if rect.contains(Position { x, y }) {
            clicked_action = Some(*action);
            break;
        }
    }

    if let Some(action) = clicked_action {
        match action {
            TermiClickAction::SwitchMode(mode) => Some(TermiAction::ChangeMode(mode, None)),
            TermiClickAction::SetModeValue(value) => match termi.config.current_mode_type() {
                ModeType::Time => Some(TermiAction::ChangeTime(value as u64)),
                ModeType::Words => Some(TermiAction::ChangeWordCount(value)),
            },
            TermiClickAction::ToggleMenu => {
                if termi.menu.is_open() {
                    Some(TermiAction::MenuClose)
                } else {
                    Some(TermiAction::MenuOpen(MenuContext::Root))
                }
            }
            TermiClickAction::TogglePunctuation => Some(TermiAction::TogglePunctuation),
            TermiClickAction::ToggleSymbols => Some(TermiAction::ToggleSymbols),
            TermiClickAction::ToggleNumbers => Some(TermiAction::ToggleNumbers),
            TermiClickAction::ToggleThemePicker => {
                if termi.theme.color_support.supports_themes() && termi.menu.is_theme_menu() {
                    Some(TermiAction::MenuClose)
                } else {
                    Some(TermiAction::MenuOpen(MenuContext::Theme))
                }
            }
            TermiClickAction::ToggleLanguagePicker => {
                if termi.menu.is_language_menu() {
                    Some(TermiAction::MenuClose)
                } else {
                    Some(TermiAction::MenuOpen(MenuContext::Language))
                }
            }
            TermiClickAction::ToggleAbout => Some(TermiAction::MenuOpen(MenuContext::About)),
            TermiClickAction::ToggleModal(modal_context) => {
                if termi.modal.is_some() {
                    Some(TermiAction::ModalClose)
                } else {
                    Some(TermiAction::ModalOpen(modal_context))
                }
            }
            TermiClickAction::ModalConfirm => Some(TermiAction::ModalConfirm),
        }
    } else {
        None
    }
}

pub fn process_action(action: TermiAction, termi: &mut Termi) {
    match action {
        TermiAction::Quit => {} // already handled as an inmediate action above
        TermiAction::NoOp => {}
        TermiAction::Start => termi.start(),
        TermiAction::TogglePause => {
            if termi.tracker.status == Status::Paused {
                termi.tracker.resume();
            } else {
                termi.tracker.pause();
            }
            termi.menu.toggle(&termi.config);
        }
        TermiAction::Input(char) => {
            if termi.menu.is_open() {
                return;
            }

            // if the first input char is <space> then do nothing
            // rationale: the first character of any given test will NEVER be <space>
            let first_test_char =
                termi.tracker.cursor_position == 0 && termi.tracker.user_input.is_empty();
            if char == ' ' && (termi.tracker.status == Status::Idle || first_test_char) {
                return;
            }

            match termi.tracker.status {
                Status::Paused => termi.tracker.resume(),
                Status::Idle => termi.tracker.start_typing(),
                _ => {}
            }
            termi.tracker.type_char(char);
        }
        TermiAction::Backspace => {
            if termi.menu.is_open() {
                return;
            }

            if termi.tracker.status == Status::Paused {
                termi.tracker.resume();
            }
            termi.tracker.backspace();
        }
        TermiAction::MenuNavigate(nav_action) => {
            let action = TermiAction::MenuNavigate(nav_action);
            termi.menu.handle_action(action, &termi.config);
        }
        TermiAction::MenuSearch(search_action) => {
            let action = TermiAction::MenuSearch(search_action);
            termi.menu.handle_action(action, &termi.config);
        }
        TermiAction::MenuOpen(ctx) => {
            let action = TermiAction::MenuOpen(ctx);
            termi.menu.handle_action(action, &termi.config);
        }
        TermiAction::MenuClose => {
            let action = TermiAction::MenuClose;
            termi.menu.handle_action(action, &termi.config);
        }
        TermiAction::MenuSelect => {
            let action = TermiAction::MenuSelect;
            termi.menu.handle_action(action, &termi.config);
        }
        TermiAction::ModalClose => termi.modal = None,
        TermiAction::ModalOpen(ctx) => {
            termi.modal = Some(build_modal(ctx));
            log_debug!("Opening modal with: {ctx:?}")
        }
        TermiAction::ModalInput(char) => {
            if let Some(modal) = termi.modal.as_mut() {
                modal.handle_char(char);
            }
        }
        TermiAction::ModalConfirm => {
            handle_modal_confirm(termi);
        }
        TermiAction::ModalBackspace => {
            if let Some(modal) = termi.modal.as_mut() {
                modal.handle_backspace();
            }
        }
        TermiAction::ChangeLanguage(lang) => {
            termi.config.change_language(&lang);
            termi.start();
        }
        TermiAction::ChangeCursor(style) => {
            termi.config.change_cursor_style(&style);
            execute!(
                std::io::stdout(),
                termi.config.resolve_current_cursor_style()
            )
            .ok();
        }
        TermiAction::ChangeMode(mode_type, val) => {
            termi.config.change_mode(mode_type, val);
            termi.start();
        }
        TermiAction::ChangeTime(time) => {
            termi
                .config
                .change_mode(ModeType::Time, Some(time as usize));
            termi.start();
        }
        TermiAction::ChangeWordCount(word_count) => {
            termi.config.change_mode(ModeType::Words, Some(word_count));
        }
        TermiAction::ChangeVisibleLines(line_count) => termi
            .config
            .change_visible_lines(line_count.try_into().unwrap_or(DEFAULT_LINE_COUNT)),
        TermiAction::TogglePunctuation => termi.config.toggle_punctuation(),
        TermiAction::ToggleNumbers => termi.config.toggle_numbers(),
        TermiAction::ToggleSymbols => termi.config.toggle_symbols(),
        TermiAction::ChangeTheme(name) => {
            termi.config.change_theme(&name);
            termi.theme = Theme::from_name(&name);
            termi.preview_theme = None;
            execute!(
                std::io::stdout(),
                termi.config.resolve_current_cursor_style()
            )
            .ok();
        }
        TermiAction::ApplyPreview(preview_type) => match preview_type {
            PreviewType::Theme(name) => termi.preview_theme = Some(Theme::from_name(&name)),
            PreviewType::Cursor(name) => {
                let style = termi.config.resolve_cursor_style_from_name(&name);
                termi.preview_cursor = Some(style);
                execute!(std::io::stdout(), style).ok();
            }
        },
        TermiAction::ClearPreview => {
            if termi.preview_theme.is_some() {
                termi.preview_theme = None;
            }
            if termi.preview_cursor.take().is_some() {
                execute!(
                    std::io::stdout(),
                    termi.config.resolve_current_cursor_style()
                )
                .ok();
            }
        }
    }
}
