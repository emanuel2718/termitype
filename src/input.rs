use crate::{
    actions::Action,
    builders::keymaps::{global_keymap, typing_keymap},
    log_debug,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
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
        if self.is_restart_sequence(&event.code) {
            self.last_keycode = Some(event.code);
            return Action::Start;
        }

        if let Some(action) = global_keymap().get_action_from(&event) {
            self.last_keycode = Some(event.code);
            log_debug!("The action from input.handle: {action:?}");
            return action;
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
