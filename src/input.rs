use crate::{
    config::ModeType,
    constants::{AMOUNT_OF_VISIBLE_LINES, BACKSPACE},
    menu::{MenuAction, MenuState},
    termi::Termi,
    theme::Theme,
    tracker::Status,
};
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    execute,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    None,
    Start,
    Pause,
    Quit,
    TypeCharacter(char),
    Backspace,
    Menu(MenuInputAction),
    Toggle(ToggleAction),
    #[cfg(debug_assertions)]
    Debug(DebugAction),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuInputAction {
    Up,
    Down,
    Back,
    Select,
    StartSearch,
    UpdateSearch(char),
    FinishSearch,
    CancelSearch,
    Close,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToggleAction {
    #[cfg(debug_assertions)]
    ToggleDebugPanel,
    #[cfg(not(debug_assertions))]
    _Placeholder, // Need at least one variant
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
pub enum DebugAction {
    TogglePanel,
    NextTab,
    PrevTab,
    ScrollUp,
    ScrollDown,
    ToggleAutoScroll,
}

#[derive(Default)]
pub struct InputHandler {
    last_key: Option<KeyCode>,
    pending_accent: Option<char>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            last_key: None,
            pending_accent: None,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent, menu: &MenuState, _is_debug: bool) -> Action {
        let last_key_cache = self.last_key;
        self.last_key = Some(key.code);

        if self.is_quit_sequence(&key) {
            return Action::Quit;
        }

        if self.is_restart_sequence(&key.code, &last_key_cache) {
            return Action::Start;
        }

        #[cfg(debug_assertions)]
        if _is_debug {
            if let Some(action) = self.handle_debug_keys(&key) {
                return action;
            }
        }

        if menu.is_open() {
            return self.handle_menu_input(key, menu);
        }
        self.handle_type_input(key)
    }

    fn handle_type_input(&mut self, key: KeyEvent) -> Action {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c' | 'z'), KeyModifiers::CONTROL) => Action::Quit,
            #[cfg(debug_assertions)]
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                Action::Toggle(ToggleAction::ToggleDebugPanel)
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                self.pending_accent = None;
                Action::Backspace
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.pending_accent = None;
                Action::Pause
            }
            (KeyCode::Char(c), KeyModifiers::ALT) => {
                self.pending_accent = Some(c);
                Action::None
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                if self.pending_accent.take().is_some() {
                    if let Some(composed) = self.compose_accent(c) {
                        Action::TypeCharacter(composed)
                    } else {
                        Action::TypeCharacter(c)
                    }
                } else {
                    Action::TypeCharacter(c)
                }
            }
            _ => Action::None,
        }
    }

    fn handle_menu_input(&self, key: KeyEvent, menu: &MenuState) -> Action {
        if menu.is_searching() {
            return match (key.code, key.modifiers) {
                (KeyCode::Esc, _) => Action::Menu(MenuInputAction::CancelSearch),
                (KeyCode::Enter, _) => Action::Menu(MenuInputAction::Select),
                (KeyCode::Char('j' | 'n'), KeyModifiers::CONTROL) => {
                    Action::Menu(MenuInputAction::Down)
                }
                (KeyCode::Char('k' | 'p'), KeyModifiers::CONTROL) => {
                    Action::Menu(MenuInputAction::Up)
                }
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    Action::Menu(MenuInputAction::UpdateSearch(c))
                }
                (KeyCode::Backspace, _) => Action::Menu(MenuInputAction::UpdateSearch(BACKSPACE)),
                (KeyCode::Up, _) => Action::Menu(MenuInputAction::Up),
                (KeyCode::Down, _) => Action::Menu(MenuInputAction::Down),
                _ => Action::None,
            };
        }

        match key.code {
            KeyCode::Char('/') => Action::Menu(MenuInputAction::StartSearch),
            KeyCode::Char('k') | KeyCode::Up => Action::Menu(MenuInputAction::Up),
            KeyCode::Char('j') | KeyCode::Down => Action::Menu(MenuInputAction::Down),
            KeyCode::Enter => Action::Menu(MenuInputAction::Select),
            KeyCode::Char('l') => {
                if let Some(menu) = menu.current_menu() {
                    if menu.current_item().has_submenu {
                        return Action::Menu(MenuInputAction::Select);
                    }
                }
                Action::None
            }
            KeyCode::Char(' ') => {
                if let Some(menu) = menu.current_menu() {
                    if menu.current_item().is_toggleable {
                        return Action::Menu(MenuInputAction::Select);
                    }
                }
                Action::None
            }
            KeyCode::Char('h') | KeyCode::Esc => {
                if menu.should_close_completely() {
                    Action::Menu(MenuInputAction::Close)
                } else if menu.menu_depth() > 1 {
                    Action::Menu(MenuInputAction::Back)
                } else {
                    Action::Menu(MenuInputAction::Close)
                }
            }
            _ => Action::None,
        }
    }

    #[cfg(debug_assertions)]
    fn handle_debug_keys(&self, key: &KeyEvent) -> Option<Action> {
        use crate::constants::DEBUG_KEY;

        match (key.code, key.modifiers) {
            (KeyCode::Char(DEBUG_KEY), KeyModifiers::CONTROL) => {
                Some(Action::Debug(DebugAction::TogglePanel))
            }
            (KeyCode::Left, KeyModifiers::NONE) => Some(Action::Debug(DebugAction::PrevTab)),
            (KeyCode::Right, KeyModifiers::NONE) => Some(Action::Debug(DebugAction::NextTab)),
            (KeyCode::Up, KeyModifiers::NONE) => Some(Action::Debug(DebugAction::ScrollUp)),
            (KeyCode::Down, KeyModifiers::NONE) => Some(Action::Debug(DebugAction::ScrollDown)),
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                Some(Action::Debug(DebugAction::ToggleAutoScroll))
            }
            _ => None,
        }
    }

    fn is_quit_sequence(&self, key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('c' | 'z'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    fn is_restart_sequence(&self, current_key: &KeyCode, last_key: &Option<KeyCode>) -> bool {
        matches!(last_key, Some(KeyCode::Tab)) && matches!(current_key, KeyCode::Enter)
    }

    // TODO: this is dumb, i think (?). There has to be a better way to handle this
    fn compose_accent(&self, c: char) -> Option<char> {
        match c {
            'e' => Some('é'),
            'a' => Some('á'),
            'i' => Some('í'),
            'o' => Some('ó'),
            'u' => Some('ú'),
            'n' => Some('ñ'),
            _ => None,
        }
    }
}

pub fn process_action(action: Action, state: &mut Termi) -> Action {
    match action {
        Action::None => Action::None,
        Action::TypeCharacter(char) => {
            if state.menu.is_open() {
                return Action::None;
            }

            match state.tracker.status {
                Status::Paused => state.tracker.resume(),
                Status::Idle => state.tracker.start_typing(),
                _ => {}
            }
            state.tracker.type_char(char);
            Action::None
        }
        Action::Backspace => {
            if state.menu.is_open() {
                return Action::None;
            }

            if state.tracker.status == Status::Paused {
                state.tracker.resume();
            }
            state.tracker.backspace();
            Action::None
        }
        Action::Menu(menu_action) => execute_menu_action(menu_action, state),
        #[cfg(debug_assertions)]
        Action::Debug(debug_action) => execute_debug_action(debug_action, state),
        Action::Start => {
            state.start();
            Action::None
        }
        Action::Pause => {
            if state.menu.is_open() {
                state.menu.toggle(&state.config);
                state.tracker.resume();
            } else {
                state.tracker.pause();
                state.menu.toggle(&state.config);
            }
            Action::None
        }
        Action::Quit => Action::Quit,
        Action::Toggle(toggle_action) => {
            #[cfg(debug_assertions)]
            match toggle_action {
                ToggleAction::ToggleDebugPanel => {
                    if let Some(debug) = &mut state.debug {
                        debug.visible = !debug.visible;
                    }
                }
            }
            #[cfg(not(debug_assertions))]
            match toggle_action {
                ToggleAction::_Placeholder => {}
            }
            Action::None
        }
    }
}

fn execute_menu_action(action: MenuInputAction, state: &mut Termi) -> Action {
    match action {
        MenuInputAction::Up => {
            state.menu.prev_item();
            if let Some(item) = state.menu.current_menu().unwrap().selected_item() {
                if let crate::menu::MenuAction::ChangeTheme(_) = &item.action {
                    state.menu.preview_selected();
                    state.update_preview_theme();
                } else if let crate::menu::MenuAction::ChangeCursorStyle(_) = &item.action {
                    state.menu.preview_selected();
                    state.update_preview_cursor();
                }
            }
            Action::None
        }
        MenuInputAction::Down => {
            state.menu.next_item();
            if let Some(item) = state.menu.current_menu().unwrap().selected_item() {
                if let crate::menu::MenuAction::ChangeTheme(_) = &item.action {
                    state.menu.preview_selected();
                    state.update_preview_theme();
                } else if let crate::menu::MenuAction::ChangeCursorStyle(_) = &item.action {
                    state.menu.preview_selected();
                    state.update_preview_cursor();
                }
            }
            Action::None
        }
        MenuInputAction::Back => {
            if state.menu.is_searching() {
                state.menu.cancel_search();
                return Action::None;
            }
            state.menu.back();
            if state.preview_theme.is_some() {
                state.preview_theme = None;
            }
            if state.preview_cursor.is_some() {
                state.preview_cursor = None;
                execute!(
                    std::io::stdout(),
                    state.config.resolve_current_cursor_style()
                )
                .ok();
            }
            if !state.menu.is_open() {
                state.tracker.resume();
            }
            Action::None
        }
        MenuInputAction::Select => {
            if state.menu.is_searching() {
                state.menu.cancel_search();
            }

            if let Some(action) = state.menu.enter(&state.config) {
                match action {
                    MenuAction::Restart => {
                        state.start();
                        return Action::None;
                    }
                    MenuAction::ToggleFeature(feature) => {
                        match feature.as_str() {
                            "punctuation" => {
                                state.config.toggle_punctuation();
                            }
                            "numbers" => {
                                state.config.toggle_numbers();
                            }
                            "symbols" => {
                                state.config.toggle_symbols();
                            }
                            _ => {}
                        }
                        state.menu.update_toggles(&state.config);
                        state.start();
                    }
                    MenuAction::ChangeMode(mode) => {
                        state.config.change_mode(mode, None);
                    }
                    MenuAction::ChangeTime(time) => {
                        state
                            .config
                            .change_mode(ModeType::Time, Some(time as usize));
                    }
                    MenuAction::ChangeWordCount(count) => {
                        state.config.change_mode(ModeType::Words, Some(count));
                    }
                    MenuAction::ChangeVisibleLineCount(count) => {
                        state.config.change_visible_lines(
                            count.try_into().unwrap_or(AMOUNT_OF_VISIBLE_LINES),
                        );
                    }
                    MenuAction::ChangeTheme(theme_name) => {
                        state.config.change_theme(&theme_name);
                        state.theme = Theme::from_name(&theme_name);
                    }
                    MenuAction::ChangeCursorStyle(style) => {
                        state.config.change_cursor_style(&style);
                        execute!(
                            std::io::stdout(),
                            state.config.resolve_current_cursor_style()
                        )
                        .ok();
                    }
                    MenuAction::ChangeLanguage(lang) => {
                        state.config.language = lang;
                        state.start();
                    }
                    MenuAction::Quit => return Action::Quit,
                    _ => {}
                }
            }
            Action::None
        }
        MenuInputAction::StartSearch => {
            state.menu.start_search();
            Action::None
        }
        MenuInputAction::UpdateSearch(c) => {
            if c == BACKSPACE {
                let mut query = state.menu.search_query().to_string();
                query.pop();
                state.menu.update_search(&query);
            } else {
                let query = format!("{}{}", state.menu.search_query(), c);
                state.menu.update_search(&query);
            }
            if let Some(menu) = state.menu.current_menu() {
                if let Some(item) = menu.selected_item() {
                    // TODO: eventually update other stuff like cursor style and language
                    match &item.action {
                        MenuAction::ChangeTheme(_) => {
                            state.menu.preview_selected();
                            state.update_preview_theme();
                        }
                        MenuAction::ChangeCursorStyle(_) => {
                            state.menu.preview_selected();
                            state.update_preview_cursor();
                        }
                        _ => {}
                    }
                }
            }
            Action::None
        }
        MenuInputAction::FinishSearch => {
            state.menu.finish_search();
            Action::None
        }
        MenuInputAction::CancelSearch => {
            state.menu.cancel_search();
            Action::None
        }
        MenuInputAction::Close => {
            state.menu.close();
            Action::None
        }
    }
}
#[cfg(debug_assertions)]
fn execute_debug_action(action: DebugAction, state: &mut Termi) -> Action {
    match action {
        DebugAction::TogglePanel => {
            if let Some(debug) = state.debug.as_mut() {
                debug.toggle();
            }
            Action::None
        }
        DebugAction::NextTab => {
            if let Some(debug) = state.debug.as_mut() {
                debug.next_tab();
            }
            Action::None
        }
        DebugAction::PrevTab => {
            if let Some(debug) = state.debug.as_mut() {
                debug.prev_tab();
            }
            Action::None
        }
        DebugAction::ScrollUp => {
            if let Some(debug) = state.debug.as_mut() {
                debug.scroll_up();
            }
            Action::None
        }
        DebugAction::ScrollDown => {
            if let Some(debug) = state.debug.as_mut() {
                let max_lines = match debug.current_tab {
                    0 => 3,
                    1 => debug.logs.len(),
                    _ => unreachable!(),
                };
                debug.scroll_down(max_lines);
            }
            Action::None
        }
        DebugAction::ToggleAutoScroll => {
            if let Some(debug) = state.debug.as_mut() {
                debug.toggle_auto_scroll();
            }
            Action::None
        }
    }
}
