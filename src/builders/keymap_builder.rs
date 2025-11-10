use crate::actions::Action;
use crate::leaderboard::{LeaderboardMotion, SortColumn};
use crate::log_debug;
use crate::menu::{MenuContext, MenuMotion};
use crate::variants::ResultsVariant;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::sync::OnceLock;

pub static GLOBAL_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static IDLE_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static TYPING_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static RESULTS_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static MENU_BASE_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static MENU_SEARCH_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static MODAL_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static LEADERBOARD_KEYMAP: OnceLock<KeyMap> = OnceLock::new();

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

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

pub fn modal_keymap() -> &'static KeyMap {
    MODAL_KEYMAP.get_or_init(build_modal_keymap)
}

pub fn leaderboard_keymap() -> &'static KeyMap {
    LEADERBOARD_KEYMAP.get_or_init(build_leaderboard_keymap)
}

/// Global keybinds are those keybinds that no matter the current context they will have the same`
/// resulting action
fn build_global_keymap() -> KeyMap {
    log_debug!("building the global keymap");
    KeyMap::new()
        .bind_with_mod(CTRL, KeyCode::Char(' '), Action::MenuToggle)
        .bind_with_mod(CTRL, KeyCode::Char('o'), Action::CommandPaletteToggle)
        .bind_with_mod(CTRL, KeyCode::Char('l'), Action::LeaderboardToggle)
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

#[rustfmt::skip]
fn build_results_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Char('q'), Action::Quit)
        .bind(KeyCode::Esc, Action::MenuOpen(MenuContext::Root))
        .bind( KeyCode::Char('m'), Action::SetResultVariant(ResultsVariant::Minimal))
        .bind( KeyCode::Char('g'), Action::SetResultVariant(ResultsVariant::Graph))
        .bind( KeyCode::Char('n'), Action::SetResultVariant(ResultsVariant::Neofetch))
        .bind( KeyCode::Up, Action::CycleNextArt)
        .bind( KeyCode::Down, Action::CyclePreviousArt)
        .bind_with_mod(SHIFT, KeyCode::Char('N'), Action::Restart)
        .bind_with_mod(SHIFT, KeyCode::Char('R'), Action::Redo)
}

fn build_menu_base_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Esc, Action::MenuGoBack)
        .bind(KeyCode::Char('q'), Action::MenuClose)
        .bind(KeyCode::Enter, Action::MenuSelect)
        .bind(KeyCode::Char(' '), Action::MenuSelect)
        .bind(KeyCode::Up, Action::MenuNav(MenuMotion::Up))
        .bind(KeyCode::Down, Action::MenuNav(MenuMotion::Down))
        .bind(KeyCode::Char('k'), Action::MenuNav(MenuMotion::Up))
        .bind(KeyCode::Char('j'), Action::MenuNav(MenuMotion::Down))
        .bind(KeyCode::Char('/'), Action::MenuInitSearch)
        .bind_with_mod(CTRL, KeyCode::Char('y'), Action::MenuSelect)
        .bind_with_mod(CTRL, KeyCode::Char('p'), Action::MenuNav(MenuMotion::Up))
        .bind_with_mod(CTRL, KeyCode::Char('n'), Action::MenuNav(MenuMotion::Down))
}

fn build_menu_search_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Esc, Action::MenuExitSearch)
        .bind(KeyCode::Enter, Action::MenuSelect)
        .bind(KeyCode::Backspace, Action::MenuBackspaceSearch)
        .bind(KeyCode::Up, Action::MenuNav(MenuMotion::Up))
        .bind(KeyCode::Down, Action::MenuNav(MenuMotion::Down))
        .bind_with_mod(CTRL, KeyCode::Char('y'), Action::MenuSelect)
        .bind_with_mod(CTRL, KeyCode::Char('n'), Action::MenuNav(MenuMotion::Down))
        .bind_with_mod(CTRL, KeyCode::Char('p'), Action::MenuNav(MenuMotion::Up))
}

fn build_modal_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Esc, Action::ModalClose)
        .bind(KeyCode::Enter, Action::ModalConfirm)
        .bind(KeyCode::Backspace, Action::ModalBackspace)
}

#[rustfmt::skip]
fn build_leaderboard_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Esc, Action::LeaderboardClose)
        .bind(KeyCode::Char('q'), Action::LeaderboardClose)
        .bind(KeyCode::Char('j'), Action::LeaderboardNav(LeaderboardMotion::Down))
        .bind(KeyCode::Char('k'), Action::LeaderboardNav(LeaderboardMotion::Up))
        .bind(KeyCode::Down, Action::LeaderboardNav(LeaderboardMotion::Down))
        .bind(KeyCode::Up, Action::LeaderboardNav(LeaderboardMotion::Up))
        .bind(KeyCode::Char('g'), Action::LeaderboardNav(LeaderboardMotion::Home))
        .bind(KeyCode::Char('m'), Action::LeaderboardSort(SortColumn::Mode))
        .bind(KeyCode::Char('l'), Action::LeaderboardSort(SortColumn::Language))
        .bind(KeyCode::Char('w'), Action::LeaderboardSort(SortColumn::Wpm))
        .bind(KeyCode::Char('r'), Action::LeaderboardSort(SortColumn::RawWpm))
        .bind(KeyCode::Char('a'), Action::LeaderboardSort(SortColumn::Accuracy))
        .bind(KeyCode::Char('c'), Action::LeaderboardSort(SortColumn::Consistency))
        .bind(KeyCode::Char('e'), Action::LeaderboardSort(SortColumn::ErrorCount))
        .bind(KeyCode::Char('d'), Action::LeaderboardSort(SortColumn::CreatedAt))
        .bind_with_mod(SHIFT, KeyCode::Char('G'), Action::LeaderboardNav(LeaderboardMotion::End))
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
