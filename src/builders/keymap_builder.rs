use crate::actions::Action;
use crate::log_debug;
use crate::menu::{MenuContext, MenuMotion};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::sync::OnceLock;

pub static GLOBAL_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static IDLE_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static TYPING_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static RESULTS_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static MENU_BASE_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static MENU_SEARCH_KEYMAP: OnceLock<KeyMap> = OnceLock::new();

const CTRL: KeyModifiers = KeyModifiers::CONTROL;

#[derive(Debug, Clone, Default)]
pub struct KeyMap {
    bindings: HashMap<(KeyModifiers, KeyCode), Action>,
}

// NOTE: eventually, maybe, we would like to allow the user to change the keybinds?
// NOTE: this way of setting the keybinds could allow for super easy showing of keybinds on `help`. We used to this manually...
//       for this we would need to add some sort of `description` like stting a keyamp in nvim with `desc`. I'll do this later, too lazy for this now

impl KeyMap {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind(mut self, key: KeyCode, action: Action) -> Self {
        self.bindings.insert((KeyModifiers::NONE, key), action);
        self
    }

    pub fn bind_with_mod(mut self, mods: KeyModifiers, key: KeyCode, action: Action) -> Self {
        self.bindings.insert((mods, key), action);
        self
    }

    pub fn get_action_from(&self, event: &KeyEvent) -> Option<Action> {
        self.bindings.get(&(event.modifiers, event.code)).cloned()
    }
}

pub fn global_keymap() -> &'static KeyMap {
    GLOBAL_KEYMAP.get_or_init(build_global_keymap)
}

pub fn idle_keymap() -> &'static KeyMap {
    IDLE_KEYMAP.get_or_init(build_idle_keymap)
}

pub fn typing_keymap() -> &'static KeyMap {
    TYPING_KEYMAP.get_or_init(build_typing_keymap)
}

pub fn results_keymap() -> &'static KeyMap {
    RESULTS_KEYMAP.get_or_init(build_results_keymap)
}

pub fn menu_base_keymap() -> &'static KeyMap {
    MENU_BASE_KEYMAP.get_or_init(build_menu_base_keymap)
}

pub fn menu_search_keymap() -> &'static KeyMap {
    MENU_SEARCH_KEYMAP.get_or_init(build_menu_search_keymap)
}

/// Global keybinds are those keybinds that no matter the current context they will have the same`
/// resulting action
fn build_global_keymap() -> KeyMap {
    log_debug!("building the global keymap");
    KeyMap::new()
        .bind(KeyCode::F(1), Action::NoOp)
        .bind(KeyCode::F(2), Action::NoOp)
        .bind(KeyCode::F(3), Action::NoOp)
        .bind_with_mod(CTRL, KeyCode::Char(' '), Action::MenuToggle)
        .bind_with_mod(CTRL, KeyCode::Char('l'), Action::ChangeLineCount(1))
        .bind_with_mod(CTRL, KeyCode::Char('t'), Action::RandomizeTheme)
        .bind_with_mod(CTRL, KeyCode::Char('c'), Action::Quit)
        .bind_with_mod(CTRL, KeyCode::Char('z'), Action::Quit)
}

fn build_idle_keymap() -> KeyMap {
    KeyMap::new().bind(KeyCode::Esc, Action::MenuOpen(MenuContext::Root))
}

fn build_typing_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Esc, Action::MenuOpen(MenuContext::Root))
        .bind(KeyCode::Backspace, Action::Backspace)
}

fn build_results_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Char('q'), Action::Quit)
        .bind(KeyCode::Char('r'), Action::Redo)
        .bind(KeyCode::Char('n'), Action::Restart)
        .bind(KeyCode::Esc, Action::MenuOpen(MenuContext::Root))
}

fn build_menu_base_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Esc, Action::MenuGoBack)
        .bind(KeyCode::Enter, Action::MenuSelect)
        .bind(KeyCode::Char(' '), Action::MenuSelect)
        .bind(KeyCode::Up, Action::MenuNav(MenuMotion::Up))
        .bind(KeyCode::Down, Action::MenuNav(MenuMotion::Down))
        .bind(KeyCode::Char('k'), Action::MenuNav(MenuMotion::Up))
        .bind_with_mod(CTRL, KeyCode::Char('p'), Action::MenuNav(MenuMotion::Up))
        .bind(KeyCode::Char('j'), Action::MenuNav(MenuMotion::Down))
        .bind_with_mod(CTRL, KeyCode::Char('n'), Action::MenuNav(MenuMotion::Down))
        .bind(KeyCode::Char('l'), Action::MenuSelect)
        .bind(KeyCode::Char('h'), Action::MenuGoBack)
        .bind(KeyCode::Char('/'), Action::MenuInitSearch)
}

fn build_menu_search_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Esc, Action::MenuExitSearch)
        .bind(KeyCode::Enter, Action::MenuSelect)
        .bind_with_mod(CTRL, KeyCode::Char('n'), Action::MenuNav(MenuMotion::Down))
        .bind_with_mod(CTRL, KeyCode::Char('p'), Action::MenuNav(MenuMotion::Up))
        .bind(KeyCode::Backspace, Action::MenuBackspaceSearch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_event(mods: KeyModifiers, code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn test_random_keymap_bind() {
        let keymap = KeyMap::new()
            .bind(KeyCode::Char('q'), Action::Quit)
            .bind_with_mod(KeyModifiers::CONTROL, KeyCode::Char('s'), Action::Start);

        let first_event = create_event(KeyModifiers::NONE, KeyCode::Char('q'));
        assert_eq!(keymap.get_action_from(&first_event), Some(Action::Quit));
        let second_event = create_event(CTRL, KeyCode::Char('s'));
        assert_eq!(keymap.get_action_from(&second_event), Some(Action::Start));

        let fake_event = create_event(KeyModifiers::SHIFT, KeyCode::Char('n'));
        assert_eq!(keymap.get_action_from(&fake_event), None);
    }
}
