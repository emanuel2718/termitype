use crate::{config::ModeType, log_debug, modal::ModalContext, termi::Termi, tracker::Status};

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
    ChangePreview(PreviewType),
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
        TermiAction::ModalOpen(modal_context) => todo!(),
        TermiAction::ModalInput(_) => todo!(),
        TermiAction::ModalClose => todo!(),
        TermiAction::ModalConfirm => todo!(),
        TermiAction::ModalBackspace => todo!(),
        TermiAction::ChangeTheme(_) => todo!(),
        TermiAction::ChangePreview(preview_type) => todo!(),
        TermiAction::ChangeLanguage(_) => todo!(),
        TermiAction::ChangeCursor(_) => todo!(),
        TermiAction::ChangeMode(mode_type, _) => todo!(),
        TermiAction::ChangeTime(_) => todo!(),
        TermiAction::ChangeVisibleLines(_) => todo!(),
        TermiAction::ChangeWordCount(_) => todo!(),
        TermiAction::TogglePunctuation => todo!(),
        TermiAction::ToggleNumbers => todo!(),
        TermiAction::ToggleSymbols => todo!(),
        TermiAction::ApplyPreview(preview_type) => todo!(),
        TermiAction::ClearPreview => todo!(),
    }
}
