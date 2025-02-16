use crate::{
    config::ModeType,
    menu::{MenuAction, MenuState},
    termi::Termi,
    theme::Theme,
    tracker::Status,
};
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    execute,
};
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
    StartSearch,
    UpdateSearch(char),
    FinishSearch,
    CancelSearch,
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
            if menu.is_searching() {
                return match key.code {
                    KeyCode::Esc => Action::CancelSearch,
                    KeyCode::Enter => Action::MenuSelect,
                    KeyCode::Char(c) => Action::UpdateSearch(c),
                    KeyCode::Backspace => Action::UpdateSearch('\x08'), // Special backspace character
                    KeyCode::Up => Action::MenuUp,
                    KeyCode::Down => Action::MenuDown,
                    _ => Action::None,
                };
            }
            return match key.code {
                KeyCode::Char('/') => Action::StartSearch,
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
    fn handle_start_search(&mut self) -> Action;
    fn handle_update_search(&mut self, c: char) -> Action;
    fn handle_finish_search(&mut self) -> Action;
    fn handle_cancel_search(&mut self) -> Action;
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
        if self.menu.is_searching() {
            self.menu.cancel_search();
            return Action::None;
        }
        self.menu.back();
        if self.preview_theme.is_some() {
            self.preview_theme = None;
        }
        self.preview_theme = None; // Q: do we should be handling this here?
        if self.preview_cursor.is_some() {
            self.preview_cursor = None;
            execute!(
                std::io::stdout(),
                self.config.resolve_current_cursor_style()
            )
            .ok();
        }
        if !self.menu.is_open() {
            self.tracker.resume();
        }
        Action::None
    }

    fn handle_menu_up(&mut self) -> Action {
        self.menu.prev_item();
        if let Some(item) = self.menu.current_menu().unwrap().selected_item() {
            if let MenuAction::ChangeTheme(_) = &item.action {
                self.menu.preview_selected();
                self.update_preview_theme();
            } else if let MenuAction::ChangeCursorStyle(_) = &item.action {
                self.menu.preview_selected();
                self.update_preview_cursor();
            }
        }
        Action::None
    }

    fn handle_menu_down(&mut self) -> Action {
        self.menu.next_item();
        if let Some(item) = self.menu.current_menu().unwrap().selected_item() {
            if let MenuAction::ChangeTheme(_) = &item.action {
                self.menu.preview_selected();
                self.update_preview_theme();
            } else if let MenuAction::ChangeCursorStyle(_) = &item.action {
                self.menu.preview_selected();
                self.update_preview_cursor();
            }
        }
        Action::None
    }

    fn handle_menu_select(&mut self) -> Action {
        // Exit search mode and clear query if we're in it
        if self.menu.is_searching() {
            self.menu.cancel_search();
        }

        if let Some(action) = self.menu.enter(&self.config) {
            match action {
                MenuAction::Restart => {
                    self.start();
                    return Action::None;
                }
                MenuAction::ToggleFeature(feature) => {
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
                MenuAction::ChangeMode(mode) => {
                    self.config.change_mode(mode, None);
                }
                MenuAction::ChangeTime(time) => {
                    self.config.change_mode(ModeType::Time, Some(time as usize));
                }
                MenuAction::ChangeWordCount(count) => {
                    self.config.change_mode(ModeType::Words, Some(count));
                }
                MenuAction::ChangeTheme(theme_name) => {
                    self.config.change_theme(&theme_name);
                    self.theme = Theme::from_name(&theme_name);
                }
                MenuAction::ChangeCursorStyle(style) => {
                    self.config.change_cursor_style(&style);
                    execute!(
                        std::io::stdout(),
                        self.config.resolve_current_cursor_style()
                    )
                    .ok();
                }
                MenuAction::ChangeLanguage(lang) => {
                    self.config.language = Some(lang);
                    self.start();
                }
                MenuAction::Quit => return Action::Quit,
                _ => {}
            }
        }
        Action::None
    }

    fn handle_start_search(&mut self) -> Action {
        self.menu.start_search();
        Action::None
    }

    fn handle_update_search(&mut self, c: char) -> Action {
        // backspace
        if c == '\x08' {
            let mut query = self.menu.search_query().to_string();
            query.pop();
            self.menu.update_search(&query);
        } else {
            let query = format!("{}{}", self.menu.search_query(), c);
            self.menu.update_search(&query);
        }
        Action::None
    }

    fn handle_finish_search(&mut self) -> Action {
        self.menu.finish_search();
        Action::None
    }

    fn handle_cancel_search(&mut self) -> Action {
        self.menu.cancel_search();
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
        Action::StartSearch => state.handle_start_search(),
        Action::UpdateSearch(c) => state.handle_update_search(c),
        Action::FinishSearch => state.handle_finish_search(),
        Action::CancelSearch => state.handle_cancel_search(),
    }
}
