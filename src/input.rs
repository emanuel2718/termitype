use crate::{menu::MenuItem, termi::Termi, tracker::Status};
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
    MenuUp,
    MenuDown,
    MenuSelect,
    MenuToggle,
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
    pub fn handle_input(&mut self, key: KeyEvent, is_menu_visible: bool) -> Action {
        self.update_history(key.code);

        if self.is_quit_sequence(&key) {
            return Action::Quit;
        }
        if self.is_restart_sequence() && matches!(key.code, KeyCode::Enter) {
            return Action::Start;
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) if is_menu_visible => Action::MenuUp,
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) if is_menu_visible => Action::MenuDown,
            (KeyCode::Char(' '), _) if is_menu_visible => Action::MenuToggle,
            (KeyCode::Char(c), _) => Action::TypeCharacter(c),
            (KeyCode::Enter, _) if is_menu_visible => Action::MenuSelect,
            (KeyCode::Backspace, _) => Action::Backspace,
            (KeyCode::Esc, _) => Action::Pause,
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
    fn handle_type_char(&mut self, c: char) -> Action;
    fn handle_backspace(&mut self) -> Action;
    fn handle_start(&mut self) -> Action;
    fn handle_pause(&mut self) -> Action;
    fn handle_menu_up(&mut self) -> Action;
    fn handle_menu_down(&mut self) -> Action;
    fn handle_menu_select(&mut self) -> Action;
    fn handle_menu_toggle(&mut self);
}

impl InputProcessor for Termi {
    fn handle_type_char(&mut self, c: char) -> Action {
        match self.tracker.status {
            Status::Paused => self.tracker.resume(),
            Status::Idle => self.tracker.start_typing(),
            _ => {}
        }
        self.tracker.type_char(c);
        Action::None
    }

    fn handle_backspace(&mut self) -> Action {
        if self.tracker.status == Status::Paused {
            self.tracker.resume();
        }
        self.tracker.backspace();
        Action::None
    }

    fn handle_start(&mut self) -> Action {
        self.start();
        Action::None
    }

    fn handle_pause(&mut self) -> Action {
        if self.menu.is_visible() {
            self.menu.toggle();
            self.tracker.resume();
        } else {
            self.tracker.pause();
            self.menu.toggle();
        }
        Action::None
    }
    fn handle_menu_up(&mut self) -> Action {
        self.menu.select_prev();
        Action::None
    }

    fn handle_menu_down(&mut self) -> Action {
        self.menu.select_next();
        Action::None
    }

    fn handle_menu_select(&mut self) -> Action {
        if let Some(item) = self.menu.selected_item() {
            match item {
                MenuItem::Restart => {
                    self.menu.toggle();
                    self.start();
                }
                MenuItem::TogglePunctuation => self.config.toggle_punctuation(),
                MenuItem::ToggleNumbers => self.config.toggle_numbers(),
                MenuItem::ToggleSymbols => self.config.toggle_symbols(),
                MenuItem::SwitchMode => {
                    // TODO: implement mode switching
                }
                MenuItem::ChangeTheme => {
                    // TODO: implement theme changing
                }
                MenuItem::Quit => return Action::Quit,
            }
        }
        Action::None
    }

    fn handle_menu_toggle(&mut self) {
        if let Some(item) = self.menu.selected_item() {
            if self.menu.is_toggleable(item) {
                match item {
                    MenuItem::TogglePunctuation => self.config.toggle_punctuation(),
                    MenuItem::ToggleNumbers => self.config.toggle_numbers(),
                    MenuItem::ToggleSymbols => self.config.toggle_symbols(),
                    _ => {}
                }
                self.start();
            }
        }
    }
}

pub fn process_action(action: Action, state: &mut impl InputProcessor) -> Action {
    match action {
        Action::TypeCharacter(c) => state.handle_type_char(c),
        Action::Backspace => state.handle_backspace(),
        Action::Start => state.handle_start(),
        Action::Pause => state.handle_pause(),
        Action::MenuUp => state.handle_menu_up(),
        Action::MenuDown => state.handle_menu_down(),
        Action::MenuSelect => state.handle_menu_select(),
        Action::MenuToggle => {
            state.handle_menu_toggle();
            Action::None
        }
        Action::Quit => Action::Quit,
        Action::None => Action::None,
    }
}
