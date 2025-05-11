use crate::{config::ModeType, modal::ModalContext};

/// Top-Level actions
#[derive(Debug, Clone, PartialEq)]
pub enum TermiAction {
    // === Global ===
    Quit,
    Redraw,
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
    ChangeWordCount(ModeType, Option<usize>),

    // === Toggles ===
    TogglePunctuation,
    ToggleNumbers,
    ToggleSymbols,
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
