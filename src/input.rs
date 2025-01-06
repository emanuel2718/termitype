use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::termi::Termi;

pub struct InputHandler {
    last_keycode: Option<KeyCode>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self { last_keycode: None }
    }

    pub fn handle_input(&mut self, key: KeyEvent, _: &mut Termi) -> bool {
        if self.is_sigkill(key) {
            return true;
        }
        match key.code {
            KeyCode::Char(_) => {
                // TODO: update tracker
            }
            KeyCode::Enter => {
                if self.last_keycode.is_some() && self.last_keycode == Some(KeyCode::Tab) {
                    // TODO: Restart the game
                }
            }
            KeyCode::Backspace => {
                // TODO: handle basckspace
            }
            KeyCode::Esc => {
                // TODO: handle putting the game in pause
            }
            _ => {}
        }
        self.last_keycode = Some(key.code);

        false
    }

    /// Quit the game when `Ctrl-c` or `Ctrl-z` is pressed
    fn is_sigkill(&self, key: KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('c' | 'z'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
    }
}
