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
            termi.input.push(c);
            // println!("received char: {}", c);
            // TODO: update metrics
        },
        KeyCode::Backspace => {
            termi.input.pop();
            // println!("Received backspace");
            // TODO: update metrics
        },
        KeyCode::Esc => {
            return true
        },
        _ => {}
    }
    false
}
