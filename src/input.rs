use crate::termi::Termi;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_input(key: KeyEvent, termi: &mut Termi) -> bool {
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
        KeyCode::Tab => {
            termi.restart();
        }
        KeyCode::Esc => return true,
        _ => {}
    }
    false
}
