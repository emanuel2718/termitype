use crate::{termi::Termi, tracker::Status};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    TypeCharacter(char),
    Backspace,
    Start,
    Pause,
    Quit,
    None,
}

#[derive(Default)]
pub struct InputHandler {
    input_history: VecDeque<KeyCode>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            input_history: VecDeque::with_capacity(2),
        }
    }

    /// Converts a keyboard event into an Action
    pub fn handle_input(&mut self, key: KeyEvent) -> Action {
        self.update_history(key.code);

        if self.is_quit_sequence(&key) {
            return Action::Quit;
        }
        if self.is_restart_sequence() && matches!(key.code, KeyCode::Enter) {
            return Action::Start;
        }

        match key.code {
            KeyCode::Char(c) => Action::TypeCharacter(c),
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Esc => Action::Pause,
            _ => Action::None,
        }
    }

    fn update_history(&mut self, key_code: KeyCode) {
        if self.input_history.len() >= 2 {
            self.input_history.pop_front();
        }
        self.input_history.push_back(key_code);
    }

    fn is_quit_sequence(&self, key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('c' | 'z'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    fn is_restart_sequence(&self) -> bool {
        matches!(
            self.input_history.iter().collect::<Vec<_>>()[..],
            [KeyCode::Tab, KeyCode::Enter]
        )
    }
}

pub trait InputProcessor {
    fn handle_type_char(&mut self, c: char);
    fn handle_backspace(&mut self);
    fn handle_start(&mut self);
    fn handle_pause(&mut self);
}

impl InputProcessor for Termi {
    fn handle_type_char(&mut self, c: char) {
        match self.tracker.status {
            Status::Paused => self.tracker.resume(),
            Status::Idle => self.tracker.start_typing(),
            _ => {}
        }
        self.tracker.type_char(c);
    }

    fn handle_backspace(&mut self) {
        if self.tracker.status == Status::Paused {
            self.tracker.resume();
        }
        self.tracker.backspace();
    }

    fn handle_start(&mut self) {
        self.start();
    }

    fn handle_pause(&mut self) {
        self.tracker.pause();
    }
}

pub fn process_action(action: Action, state: &mut impl InputProcessor) {
    match action {
        Action::TypeCharacter(c) => state.handle_type_char(c),
        Action::Backspace => state.handle_backspace(),
        Action::Start => state.handle_start(),
        Action::Pause => state.handle_pause(),
        Action::Quit | Action::None => {}
    }
}
