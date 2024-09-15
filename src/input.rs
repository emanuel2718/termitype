use crossterm::event::{KeyCode, KeyEvent};
use crate::termi::Termi;

pub fn handle_input(key: KeyEvent, termi: &mut Termi) -> bool {
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
                termi.cursor_pos += 1;
                termi.check_completion();
            }
        },
        KeyCode::Backspace => {
            if termi.cursor_pos > 0 {
                termi.cursor_pos -= 1;
                termi.user_input[termi.cursor_pos] = None;
            }
        },
        KeyCode::Esc => {
            return true
        },
        _ => {}
    }
    false
}
