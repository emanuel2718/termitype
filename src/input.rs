use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    actions::{MenuNavAction, MenuSearchAction, TermiAction},
    log_debug,
    termi::Termi,
};

/// Current input context
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Typing,
    Modal,
    Menu { is_searching: bool },
}

#[derive(Default)]
pub struct InputHandler {
    last_key_code: Option<KeyCode>,
    pending_accent: Option<char>,
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {
            last_key_code: None,
            pending_accent: None,
        }
    }

    pub fn resolve_input_mode(&self, termi: &Termi) -> InputMode {
        if termi.modal.is_some() {
            InputMode::Modal
        } else if termi.menu.is_open() {
            InputMode::Menu {
                is_searching: termi.menu.is_searching(),
            }
        } else {
            InputMode::Typing
        }
    }

    pub fn handle_input(&mut self, event: KeyEvent, mode: InputMode) -> TermiAction {
        let last_key_cache = self.last_key_code;
        let curr_key_code = event.code;

        if self.is_quit_sequence(&event) {
            return TermiAction::Quit;
        }

        if self.is_restart_sequence(&event.code, &last_key_cache) {
            return TermiAction::Start;
        }

        let action = match mode {
            InputMode::Typing => self.handle_typing_input(event),
            InputMode::Modal => self.handle_modal_input(event),
            InputMode::Menu { is_searching } => self.handle_menu_input(event, is_searching),
        };

        self.last_key_code = Some(curr_key_code);
        action
    }

    fn handle_typing_input(&mut self, event: KeyEvent) -> TermiAction {
        match (event.code, event.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.pending_accent = None;
                TermiAction::TogglePause
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                self.pending_accent = None;
                TermiAction::Backspace
            }
            (KeyCode::Char(c), KeyModifiers::ALT) => {
                self.pending_accent = Some(c);
                TermiAction::NoOp
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                if self.pending_accent.take().is_some() {
                    // TODO: improve composition. This will only work with spanish
                    let composed = self.compose_accent(c).unwrap_or(c);
                    TermiAction::Input(composed)
                } else {
                    TermiAction::Input(c)
                }
            }
            _ => TermiAction::NoOp,
        }
    }

    fn handle_menu_input(&self, event: KeyEvent, is_searching: bool) -> TermiAction {
        if is_searching {
            match (event.code, event.modifiers) {
                (KeyCode::Esc, _) => TermiAction::MenuSearch(MenuSearchAction::Close),
                (KeyCode::Enter, _) => TermiAction::MenuSearch(MenuSearchAction::Confirm),
                (KeyCode::Backspace, _) => TermiAction::MenuSearch(MenuSearchAction::Backspace),
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    TermiAction::MenuSearch(MenuSearchAction::Input(c))
                }
                (KeyCode::Up, _) => TermiAction::MenuNavigate(MenuNavAction::Up),
                (KeyCode::Down, _) => TermiAction::MenuNavigate(MenuNavAction::Down),
                // TODO: eventually we want something like vi/emacs keymaps distintction.
                // starting with hardcoded vi kyemaps for now
                (KeyCode::Char('j' | 'n'), KeyModifiers::CONTROL) => {
                    TermiAction::MenuNavigate(MenuNavAction::Down)
                }
                (KeyCode::Char('k' | 'p'), KeyModifiers::CONTROL) => {
                    TermiAction::MenuNavigate(MenuNavAction::Up)
                }
                _ => TermiAction::NoOp,
            }
        } else {
            match (event.code, event.modifiers) {
                // actions
                (KeyCode::Char('/'), _) => TermiAction::MenuSearch(MenuSearchAction::Start),
                (KeyCode::Enter, _) => TermiAction::MenuSelect,
                (KeyCode::Char(' '), KeyModifiers::NONE) => TermiAction::MenuSelect,
                (KeyCode::Char('l') | KeyCode::Char(' '), _) => TermiAction::MenuSelect,
                (KeyCode::Esc, _) => TermiAction::MenuNavigate(MenuNavAction::Back),
                (KeyCode::Char('h'), _) => TermiAction::MenuNavigate(MenuNavAction::Back),

                // nav
                (KeyCode::Up | KeyCode::Char('k'), _) => {
                    TermiAction::MenuNavigate(MenuNavAction::Up)
                }
                (KeyCode::Down | KeyCode::Char('j'), _) => {
                    TermiAction::MenuNavigate(MenuNavAction::Down)
                }
                // ctrl-n + ctrl+p movement
                (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                    TermiAction::MenuNavigate(MenuNavAction::Up)
                }
                (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                    TermiAction::MenuNavigate(MenuNavAction::Down)
                }
                (KeyCode::Char('y'), KeyModifiers::CONTROL) => TermiAction::MenuSelect,
                (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                    TermiAction::MenuNavigate(MenuNavAction::PageUp)
                }
                (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    TermiAction::MenuNavigate(MenuNavAction::PageDown)
                }
                (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                    TermiAction::MenuNavigate(MenuNavAction::End)
                }
                (KeyCode::Char('g'), KeyModifiers::NONE) => {
                    // this is fugly
                    let mut phone_home = false;
                    if let Some(code) = self.last_key_code {
                        log_debug!("Code: {}.... result = {}", code, code == KeyCode::Char('g'));
                        if code == KeyCode::Char('g') {
                            phone_home = true
                        }
                    }
                    if !phone_home {
                        return TermiAction::NoOp;
                    }
                    TermiAction::MenuNavigate(MenuNavAction::Home)
                }

                _ => TermiAction::NoOp,
            }
        }
    }

    fn handle_modal_input(&self, event: KeyEvent) -> TermiAction {
        match event.code {
            KeyCode::Esc => TermiAction::ModalClose,
            KeyCode::Enter => TermiAction::ModalConfirm,
            KeyCode::Backspace => TermiAction::ModalBackspace,
            KeyCode::Char(c) => TermiAction::ModalInput(c),
            _ => TermiAction::NoOp,
        }
    }

    fn is_quit_sequence(&self, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('c' | 'z'))
            && event.modifiers.contains(KeyModifiers::CONTROL)
    }

    fn is_restart_sequence(&self, current_key: &KeyCode, last_key: &Option<KeyCode>) -> bool {
        matches!(last_key, Some(KeyCode::Tab)) && matches!(current_key, KeyCode::Enter)
    }

    // TODO: this is dumb, i think (?). There has to be a better way to handle this
    fn compose_accent(&self, c: char) -> Option<char> {
        match c {
            'e' => Some('é'),
            'a' => Some('á'),
            'i' => Some('í'),
            'o' => Some('ó'),
            'u' => Some('ú'),
            'n' => Some('ñ'),
            _ => None,
        }
    }
}
