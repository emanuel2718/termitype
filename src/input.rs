use crate::{
    actions::Action,
    builders::keymap_builder::{global_keymap, typing_keymap},
    log_debug,
};
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputContext {
    Idle,
    Typing,
    Menu { searching: bool },
}

#[derive(Default)]
pub struct Input {
    last_keycode: Option<KeyCode>,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle(&mut self, event: KeyEvent, ctx: InputContext) -> Action {
        if let Some(action) = global_keymap().get_action_from(&event) {
            self.last_keycode = Some(event.code);
            log_debug!("The action from input.handle: {action:?}");
            return action;
        }

        if self.is_restart_sequence(&event.code) {
            self.last_keycode = Some(event.code);
            return Action::Start;
        }

        if self.is_typing_input(event, &ctx) {
            self.last_keycode = Some(event.code);
            if let Some(c) = event.code.as_char() {
                log_debug!("The action from input.handle: {:?}", Action::Input(c));
                return Action::Input(c);
            }
        }

        let keymap = match ctx {
            InputContext::Typing => typing_keymap(),
            _ => global_keymap(),
        };

        self.last_keycode = Some(event.code);
        let action = keymap.get_action_from(&event).unwrap_or(Action::NoOp);
        log_debug!("The action from input.handle: {action:?}");
        action
    }

    fn is_restart_sequence(&self, current_keycode: &KeyCode) -> bool {
        match self.last_keycode {
            Some(last_keycode) => {
                matches!(last_keycode, KeyCode::Tab) && matches!(current_keycode, KeyCode::Enter)
            }
            _ => false,
        }
    }

    fn is_typing_input(&self, event: KeyEvent, ctx: &InputContext) -> bool {
        matches!(event.code, KeyCode::Char(_)) && *ctx == InputContext::Typing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn create_event(mods: KeyModifiers, code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn test_quit_sequence() {
        let mut input = Input::new();
        let event = create_event(KeyModifiers::CONTROL, KeyCode::Char('c'));
        let action = input.handle(event, InputContext::Idle);

        assert_eq!(action, Action::Quit);
    }

    #[test]
    fn test_is_typing_input() {
        let mut input = Input::new();

        let tab_event = create_event(KeyModifiers::NONE, KeyCode::Tab);
        let typing_event = create_event(KeyModifiers::NONE, KeyCode::Char('c'));
        assert!(input.is_typing_input(typing_event, &InputContext::Typing));
        assert!(!input.is_typing_input(typing_event, &InputContext::Idle));
        assert!(!input.is_typing_input(tab_event, &InputContext::Typing));

        // should be a quit sequence
        let non_typing_event = create_event(KeyModifiers::CONTROL, KeyCode::Char('c'));
        assert_eq!(
            input.handle(non_typing_event, InputContext::Typing),
            Action::Quit
        );
    }

    #[test]
    fn test_restart_sequence() {
        // Tab+Enter for redo redo
        let mut input = Input::new();
        let event = create_event(KeyModifiers::NONE, KeyCode::Tab);
        let action = input.handle(event, InputContext::Idle);

        assert_eq!(action, Action::NoOp);

        let second_event = create_event(KeyModifiers::NONE, KeyCode::Enter);
        let second_action = input.handle(second_event, InputContext::Idle);
        assert_eq!(second_action, Action::Start);
    }
}
