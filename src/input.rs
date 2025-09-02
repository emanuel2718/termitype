use crate::actions::Action;
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
        if self.is_quit_sequence(&event) {
            return Action::Quit;
        }

        if self.is_restart_sequence(&event.code) {
            self.last_keycode = Some(event.code);
            return Action::Start;
        }
        Action::NoOp
    }

    fn is_quit_sequence(&self, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('c' | 'z'))
            && event.modifiers.contains(KeyModifiers::CONTROL)
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
