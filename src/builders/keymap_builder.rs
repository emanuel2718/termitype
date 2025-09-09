use crate::actions::Action;
use crate::log_debug;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::sync::OnceLock;

pub static GLOBAL_KEYMAP: OnceLock<KeyMap> = OnceLock::new();
pub static TYPING_KEYMAP: OnceLock<KeyMap> = OnceLock::new();

const MOD_CTRL: KeyModifiers = KeyModifiers::CONTROL;

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

pub fn typing_keymap() -> &'static KeyMap {
    TYPING_KEYMAP.get_or_init(build_typing_keymap)
}

/// Global keybinds are those keybinds that no matter the current context they will have the same`
/// resulting action
fn build_global_keymap() -> KeyMap {
    log_debug!("building the global keymap");
    KeyMap::new()
        .bind(KeyCode::F(1), Action::NoOp)
        .bind(KeyCode::F(2), Action::NoOp)
        .bind(KeyCode::F(3), Action::NoOp)
        .bind_with_mod(MOD_CTRL, KeyCode::Char('l'), Action::ChangeLineCount(1))
        .bind_with_mod(MOD_CTRL, KeyCode::Char('t'), Action::RandomizeTheme)
        .bind_with_mod(MOD_CTRL, KeyCode::Char('c'), Action::Quit)
        .bind_with_mod(MOD_CTRL, KeyCode::Char('z'), Action::Quit)
}

fn build_typing_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::Esc, Action::Pause)
        .bind(KeyCode::Backspace, Action::Backspace)
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
        let second_event = create_event(MOD_CTRL, KeyCode::Char('s'));
        assert_eq!(keymap.get_action_from(&second_event), Some(Action::Start));

        let fake_event = create_event(KeyModifiers::SHIFT, KeyCode::Char('n'));
        assert_eq!(keymap.get_action_from(&fake_event), None);
    }
}
