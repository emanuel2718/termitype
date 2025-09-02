use crate::actions::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::sync::OnceLock;

static GLOBAL_KEYMAP: OnceLock<KeyMap> = OnceLock::new();

#[derive(Debug, Clone, Default)]
pub struct KeyMap {
    bindings: HashMap<(KeyModifiers, KeyCode), Action>,
}

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

    pub fn get(&self, event: &KeyEvent) -> Option<Action> {
        self.bindings.get(&(event.modifiers, event.code)).cloned()
    }
}

fn build_global_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyCode::F(1), Action::NoOp)
        .bind(KeyCode::F(2), Action::NoOp)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_event(mods: KeyModifiers, code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn test_keymap_bind() {
        let keymap = KeyMap::new()
            .bind(KeyCode::Char('q'), Action::Quit)
            .bind_with_mod(KeyModifiers::CONTROL, KeyCode::Char('s'), Action::Start);

        let first_event = create_event(KeyModifiers::NONE, KeyCode::Char('q'));
        assert_eq!(keymap.get(&first_event), Some(Action::Quit));
        let second_event = create_event(KeyModifiers::CONTROL, KeyCode::Char('s'));
        assert_eq!(keymap.get(&second_event), Some(Action::Start));

        let fake_event = create_event(KeyModifiers::SHIFT, KeyCode::Char('n'));
        assert_eq!(keymap.get(&fake_event), None);
    }
}
