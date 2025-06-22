use crossterm::execute;
use ratatui::layout::Position;

use crate::{
    config::ModeType,
    constants::DEFAULT_LINE_COUNT,
    modal::{build_modal, handle_modal_confirm, ModalContext},
    termi::Termi,
    theme::Theme,
    tracker::Status,
    ui::components::elements::TermiClickableRegions,
};

/// Top-Level actions
#[derive(Debug, Clone, PartialEq)]
pub enum TermiAction {
    // === Global ===
    Quit,
    NoOp, // input that results in no state change

    // === State ===
    Start,
    Redo,
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
    ChangePickerStyle(String),
    ChangeMode(ModeType, Option<usize>),
    ChangeTime(u64),
    ChangeVisibleLines(u64),
    ChangeWordCount(usize),
    ChangeAsciiArt(String),
    ChangeResultsStyle(String),

    // === Toggles ===
    TogglePunctuation,
    ToggleNumbers,
    ToggleSymbols,
    ToggleFPS,
    ToggleLiveWPM,
    ToggleMonochromaticResults,
    ToggleCursorline,

    // === Previews ===
    ApplyPreview(PreviewType),
    ClearPreview,

    // === Debug Helper Actions ===
    #[cfg(debug_assertions)]
    DebugToggleResults,
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum MenuContext {
    Root,
    Theme,
    Language,
    Cursor,
    PickerStyle,
    Mode,
    Time,
    Words,
    LineCount,
    Help,
    About,
    AsciiArt,
    Options,
    Results,
}

// ============== PREVIEW ==============

#[derive(Debug, Clone, PartialEq)]
pub enum PreviewType {
    Theme(String),
    Cursor(String),
    AsciiArt(String),
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
        TermiAction::NoOp => {}
        TermiAction::Quit => termi.quit(),
        TermiAction::Start => termi.start(),
        TermiAction::Redo => termi.redo(),
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
            let was_typing = termi.tracker.status == Status::Typing;
            termi.tracker.type_char(char);
            if was_typing && termi.tracker.status == Status::Completed {
                termi.save_results();
            }
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
        // === Menu Actions ===
        TermiAction::MenuNavigate(_)
        | TermiAction::MenuSearch(_)
        | TermiAction::MenuOpen(_)
        | TermiAction::MenuClose
        | TermiAction::MenuSelect => {
            if let Some(menu_action) = termi.menu.handle_action(action, &termi.config) {
                process_action(menu_action, termi);
            }
        }

        // === Modal Actions ===
        TermiAction::ModalClose => termi.modal = None,
        TermiAction::ModalOpen(ctx) => {
            termi.modal = Some(build_modal(ctx));
        }
        TermiAction::ModalInput(char) => {
            if let Some(modal) = termi.modal.as_mut() {
                modal.handle_char(char);
            }
        }
        TermiAction::ModalConfirm => {
            handle_modal_confirm(termi);
            termi.menu.close();
        }
        TermiAction::ModalBackspace => {
            if let Some(modal) = termi.modal.as_mut() {
                modal.handle_backspace();
            }
        }
        TermiAction::ChangeLanguage(lang) => {
            termi.config.change_language(&lang);
            termi.start();
            termi.menu.close();
        }
        TermiAction::ChangeCursor(style) => {
            termi.config.change_cursor_style(&style);
            execute!(
                std::io::stdout(),
                termi.config.resolve_current_cursor_style()
            )
            .ok();
            termi.menu.close();
        }
        TermiAction::ChangePickerStyle(style) => {
            termi.config.change_picker_style(&style);
        }
        TermiAction::ChangeMode(mode_type, val) => {
            termi.config.change_mode(mode_type, val);
            termi.start();
            termi.menu.close();
        }
        TermiAction::ChangeTime(time) => {
            termi
                .config
                .change_mode(ModeType::Time, Some(time as usize));
            termi.start();
            termi.menu.close();
        }
        TermiAction::ChangeWordCount(word_count) => {
            termi.config.change_mode(ModeType::Words, Some(word_count));
            termi.start();
            termi.menu.close();
        }
        TermiAction::ChangeVisibleLines(line_count) => termi
            .config
            .change_visible_lines(line_count.try_into().unwrap_or(DEFAULT_LINE_COUNT)),
        TermiAction::ChangeAsciiArt(art) => {
            termi.config.change_ascii_art(&art);
            termi.menu.close();
        }
        TermiAction::ChangeResultsStyle(style) => {
            termi.config.change_results_style(&style);
            termi.menu.close();
        }
        TermiAction::TogglePunctuation => {
            termi.config.toggle_punctuation();
            termi.menu.sync_toggle_items(&termi.config);
            termi.start();
        }
        TermiAction::ToggleNumbers => {
            termi.config.toggle_numbers();
            termi.menu.sync_toggle_items(&termi.config);
            termi.start()
        }
        TermiAction::ToggleSymbols => {
            termi.config.toggle_symbols();
            termi.menu.sync_toggle_items(&termi.config);
            termi.start()
        }
        TermiAction::ToggleFPS => {
            termi.config.toggle_fps();
            termi.menu.sync_toggle_items(&termi.config);
        }
        TermiAction::ToggleLiveWPM => {
            termi.config.toggle_live_wpm();
            termi.menu.sync_toggle_items(&termi.config);
        }
        TermiAction::ToggleMonochromaticResults => {
            termi.config.toggle_monochromatic_results();
            termi.menu.sync_toggle_items(&termi.config);
        }
        TermiAction::ToggleCursorline => {
            termi.config.toggle_cursorline();
            termi.menu.sync_toggle_items(&termi.config);
        }
        TermiAction::ChangeTheme(name) => {
            termi.config.change_theme(&name);
            termi.theme = Theme::from_name(&name);
            termi.preview_theme = None;
            execute!(
                std::io::stdout(),
                termi.config.resolve_current_cursor_style()
            )
            .ok();
            termi.menu.close();
        }
        TermiAction::ApplyPreview(preview_type) => match preview_type {
            PreviewType::Theme(name) => termi.preview_theme = Some(Theme::from_name(&name)),
            PreviewType::Cursor(name) => {
                let style = termi.config.resolve_cursor_style_from_name(&name);
                termi.preview_cursor = Some(style);
                execute!(std::io::stdout(), style).ok();
            }
            PreviewType::AsciiArt(name) => {
                termi.preview_ascii_art = Some(name);
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
            if termi.preview_ascii_art.is_some() {
                termi.preview_ascii_art = None;
            }
        }
        #[cfg(debug_assertions)]
        TermiAction::DebugToggleResults => match termi.tracker.status {
            Status::Completed => termi.start(),
            _ => termi.tracker.complete(),
        },
    }
}
