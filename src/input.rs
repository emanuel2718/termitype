use crate::{
    menu::{MenuAction, MenuContent, MenuState},
    termi::Termi,
    tracker::Status,
};
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
    MenuBack,
    MenuSelect,
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
    pub fn handle_input(&mut self, key: KeyEvent, menu: &MenuState) -> Action {
        self.update_history(key.code);

        if self.is_quit_sequence(&key) {
            return Action::Quit;
        }
        if self.is_restart_sequence() && matches!(key.code, KeyCode::Enter) {
            return Action::Start;
        }

        if menu.is_open() {
            return match key.code {
                KeyCode::Char('k') | KeyCode::Up => Action::MenuUp,
                KeyCode::Char('j') | KeyCode::Down => Action::MenuDown,
                KeyCode::Enter | KeyCode::Char(' ') => Action::MenuSelect,
                KeyCode::Esc => Action::MenuBack,
                _ => Action::None,
            };
        }
        match (key.code, key.modifiers) {
            (KeyCode::Char(c), _) => Action::TypeCharacter(c),
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
    fn handle_menu_back(&mut self) -> Action;
    fn handle_menu_select(&mut self) -> Action;
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
        if self.menu.is_open() {
            self.menu.toggle(&self.config);
            self.tracker.resume();
        } else {
            self.tracker.pause();
            self.menu.toggle(&self.config);
        }
        Action::None
    }

    fn handle_menu_back(&mut self) -> Action {
        self.menu.menu_back();
        if !self.menu.is_open() {
            self.tracker.resume();
        }
        Action::None
    }

    fn handle_menu_up(&mut self) -> Action {
        self.menu.prev_menu_item();
        if let Some(item) = self.menu.selected_menu_item() {
            if let MenuContent::Action(MenuAction::ChangeTheme(_)) = &item.content {
                self.menu.preview_selected_theme();
                self.update_preview_theme();
            }
        }
        Action::None
    }

    fn handle_menu_down(&mut self) -> Action {
        self.menu.next_menu_item();
        if let Some(item) = self.menu.selected_menu_item() {
            if let MenuContent::Action(MenuAction::ChangeTheme(_)) = &item.content {
                self.menu.preview_selected_theme();
                self.update_preview_theme();
            }
        }
        Action::None
    }

    fn handle_menu_select(&mut self) -> Action {
        if let Some(action) = self.menu.menu_enter() {
            match action {
                MenuAction::Restart => {
                    self.start();
                    return Action::None;
                }
                MenuAction::Toggle(feature) => {
                    match feature.as_str() {
                        "punctuation" => {
                            self.config.toggle_punctuation();
                        }
                        "numbers" => {
                            self.config.toggle_numbers();
                        }
                        "symbols" => {
                            self.config.toggle_symbols();
                        }
                        _ => {}
                    }
                    self.menu.update_toggles(&self.config);
                }
                MenuAction::ChangeMode => {}
                MenuAction::ChangeTheme(theme_name) => {
                    self.config.change_theme(&theme_name);
                }
                MenuAction::Quit => return Action::Quit,
            }
        }
        Action::None
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
        Action::MenuBack => state.handle_menu_back(),
        Action::Quit => Action::Quit,
        Action::None => Action::None,
    }
}
