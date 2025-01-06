use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::termi::Termi;

pub struct InputHandler {
    last_keycode: Option<KeyCode>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self { last_keycode: None }
    }

    pub fn handle_input(&mut self, key: KeyEvent, termi: &mut Termi) -> bool {
        if self.is_sigkill(key) {
            return true;
        }
        match key.code {
            KeyCode::Char(c) => {
                if termi.tracker.cursor_position < termi.words.chars().count() {
                    termi.tracker.user_input.push(Some(c));
                    termi.tracker.cursor_position += 1;
                }
                // TODO: update tracker
            }
            KeyCode::Enter => {
                if self.last_keycode.is_some() && self.last_keycode == Some(KeyCode::Tab) {
                    termi.start();
                }
            }
            KeyCode::Backspace => {
                // TODO: handle basckspace
                if termi.tracker.cursor_position > 0 {
                    termi.tracker.user_input.pop();
                    termi.tracker.cursor_position -= 1;
                }
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
