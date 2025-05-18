use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    actions::{MenuNavAction, MenuSearchAction, TermiAction},
    log_debug,
    termi::Termi,
    tracker::Status,
};

/// Current input context
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Typing,
    Results,
    Modal,
    Menu { is_searching: bool },
}

#[derive(Default)]
pub struct InputHandler {
    last_keycode: Option<KeyCode>,
    pending_accent: Option<char>,
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {
            last_keycode: None,
            pending_accent: None,
        }
    }

    pub fn resolve_input_mode(&self, termi: &Termi) -> InputMode {
        // TODO: think about improving this resolver
        if termi.modal.is_some() {
            InputMode::Modal
        } else if termi.menu.is_open() {
            InputMode::Menu {
                is_searching: termi.menu.is_searching(),
            }
        } else if termi.tracker.status == Status::Completed {
            InputMode::Results
        } else {
            InputMode::Typing
        }
    }

    pub fn handle_input(
        &mut self,
        event: KeyEvent,
        last_event: Option<KeyEvent>,
        mode: InputMode,
    ) -> TermiAction {
        if self.is_quit_sequence(&event) {
            return TermiAction::Quit;
        }

        if let Some(last_ev) = last_event {
            self.last_keycode = Some(last_ev.code);
            if self.is_restart_sequence(&event.code, &last_ev.code) {
                return TermiAction::Start;
            }
        }

        #[cfg(debug_assertions)]
        if let Some(debug_action) = self.handle_debug_input(event) {
            return debug_action;
        }

        let action = match mode {
            InputMode::Typing => self.handle_typing_input(event),
            InputMode::Results => self.handle_results_input(event),
            InputMode::Modal => self.handle_modal_input(event),
            InputMode::Menu { is_searching } => self.handle_menu_input(event, is_searching),
        };

        action
    }

    #[cfg(debug_assertions)]
    fn handle_debug_input(&mut self, event: KeyEvent) -> Option<TermiAction> {
        match (event.code, event.modifiers) {
            (KeyCode::F(10), _) => Some(TermiAction::DebugToggleResults),
            _ => None,
        }
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

    fn handle_results_input(&mut self, event: KeyEvent) -> TermiAction {
        match (event.code, event.modifiers) {
            (KeyCode::Char('r'), KeyModifiers::NONE) => TermiAction::Redo,
            (KeyCode::Char('n'), KeyModifiers::NONE) => TermiAction::Start,
            (KeyCode::Char('q'), KeyModifiers::NONE) => TermiAction::Quit,
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.pending_accent = None;
                TermiAction::TogglePause
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
                    if let Some(code) = self.last_keycode {
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

    fn is_restart_sequence(&self, current_key: &KeyCode, last_key: &KeyCode) -> bool {
        matches!(last_key, KeyCode::Tab) && matches!(current_key, KeyCode::Enter)
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

#[cfg(test)]
mod tests {

    use super::*;

    fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_default_state() {
        let handler = InputHandler::new();
        assert!(handler.last_keycode.is_none());
        assert!(handler.pending_accent.is_none());
    }

    #[test]
    fn test_quit_sequence() {
        let handler = InputHandler::new();

        // <ctrl-c>
        let event = create_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(handler.is_quit_sequence(&event));

        // <ctrl+z>
        let event = create_key_event(KeyCode::Char('z'), KeyModifiers::CONTROL);
        assert!(handler.is_quit_sequence(&event));

        // non-quit seq
        let event = create_key_event(KeyCode::Char('c'), KeyModifiers::NONE);
        assert!(!handler.is_quit_sequence(&event));
    }

    #[test]
    fn test_restart_sequence() {
        let handler = InputHandler::new();

        assert!(handler.is_restart_sequence(&KeyCode::Enter, &KeyCode::Tab));
        assert!(!handler.is_restart_sequence(&KeyCode::Enter, &KeyCode::Enter));
        assert!(!handler.is_restart_sequence(&KeyCode::Tab, &KeyCode::Tab));
    }

    #[test]
    fn test_typing_input() {
        let mut handler = InputHandler::new();

        let event = create_key_event(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(handler.handle_typing_input(event), TermiAction::Input('a'));

        let event = create_key_event(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(handler.handle_typing_input(event), TermiAction::Backspace);

        let event = create_key_event(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(handler.handle_typing_input(event), TermiAction::TogglePause);
    }

    #[test]
    fn test_results_input() {
        let mut handler = InputHandler::new();

        // <r>
        let event = create_key_event(KeyCode::Char('r'), KeyModifiers::NONE);
        assert_eq!(handler.handle_results_input(event), TermiAction::Redo);

        // <n>
        let event = create_key_event(KeyCode::Char('n'), KeyModifiers::NONE);
        assert_eq!(handler.handle_results_input(event), TermiAction::Start);

        // <q>
        let event = create_key_event(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(handler.handle_results_input(event), TermiAction::Quit);

        // <esc>
        let event = create_key_event(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(
            handler.handle_results_input(event),
            TermiAction::TogglePause
        );
    }

    #[test]
    fn test_menu_search_input() {
        let handler = InputHandler::new();

        let event = create_key_event(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(
            handler.handle_menu_input(event, true),
            TermiAction::MenuSearch(MenuSearchAction::Input('a'))
        );

        let event = create_key_event(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(
            handler.handle_menu_input(event, true),
            TermiAction::MenuNavigate(MenuNavAction::Up)
        );

        // vim nav
        let event = create_key_event(KeyCode::Char('j'), KeyModifiers::CONTROL);
        assert_eq!(
            handler.handle_menu_input(event, true),
            TermiAction::MenuNavigate(MenuNavAction::Down)
        );
    }

    #[test]
    fn test_menu_navigation() {
        let handler = InputHandler::new();

        let event = create_key_event(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(
            handler.handle_menu_input(event, false),
            TermiAction::MenuNavigate(MenuNavAction::Up)
        );

        let event = create_key_event(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(
            handler.handle_menu_input(event, false),
            TermiAction::MenuNavigate(MenuNavAction::Down)
        );

        let event = create_key_event(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            handler.handle_menu_input(event, false),
            TermiAction::MenuSelect
        );

        let event = create_key_event(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(
            handler.handle_menu_input(event, false),
            TermiAction::MenuNavigate(MenuNavAction::Back)
        );
    }

    #[test]
    fn test_modal_input() {
        let handler = InputHandler::new();

        // <a>
        let event = create_key_event(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(
            handler.handle_modal_input(event),
            TermiAction::ModalInput('a')
        );

        // <esc>
        let event = create_key_event(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(handler.handle_modal_input(event), TermiAction::ModalClose);

        // <enter>
        let event = create_key_event(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(handler.handle_modal_input(event), TermiAction::ModalConfirm);

        // <backspace>
        let event = create_key_event(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(
            handler.handle_modal_input(event),
            TermiAction::ModalBackspace
        );
    }
}
