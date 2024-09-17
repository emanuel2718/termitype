use crate::termi::Termi;
use crossterm::event::{KeyCode, KeyEvent};

pub struct InputHandler {
    tab_pressed: bool,
}



impl InputHandler {
    pub fn new() -> Self {
        Self { tab_pressed: false }
    }

    pub fn handle_input(&mut self, key: KeyEvent, termi: &mut Termi) -> bool {
        if !termi.is_started {
            termi.is_started = true
        }
        if termi.is_finished {
            if key.code == KeyCode::Enter {
                termi.restart();
            } else if key.code == KeyCode::Esc {
                return true;
            }
            return false;
        }

        match key.code {
            KeyCode::Char(c) => {
                if termi.cursor_pos < termi.target_text.chars().count() {
                    termi.user_input[termi.cursor_pos] = Some(c);
                    if c == termi.target_text.chars().nth(termi.cursor_pos).unwrap() {
                        termi.correct_chars += 1;
                    }
                    termi.cursor_pos += 1;
                    termi.check_completion();
                }
            }
            KeyCode::Tab => {
                self.tab_pressed = true;
            }
            KeyCode::Enter => {
                if self.tab_pressed {
                    termi.restart();
                    self.tab_pressed = false;
                }
            }
            KeyCode::Backspace => {
                if termi.cursor_pos > 0 {
                    termi.cursor_pos -= 1;
                    if let Some(Some(ch)) = termi.user_input.get(termi.cursor_pos) {
                        if *ch == termi.target_text.chars().nth(termi.cursor_pos).unwrap() {
                            termi.correct_chars = termi.correct_chars.saturating_sub(1);
                        }
                    }
                    termi.user_input[termi.cursor_pos] = None;
                }
            }
            KeyCode::Esc => return true,
            _ => {}
        }
        false
    }

}
