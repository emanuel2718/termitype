use crate::{
    actions::Action,
    builders::keymap_builder::{
        global_keymap, idle_keymap, menu_base_keymap, menu_search_keymap, modal_keymap,
        results_keymap, typing_keymap,
    },
    log_debug,
};
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputContext {
    Idle,
    Typing,
    Completed,
    Modal,
    Menu { searching: bool },
}

#[derive(Default)]
pub struct Input {
    last_keycode: Option<KeyCode>,
}

#[derive(Debug, PartialEq)]
pub struct InputResult {
    pub action: Action,
    pub skip_debounce: bool,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle(&mut self, event: KeyEvent, ctx: InputContext) -> InputResult {
        if let Some(action) = global_keymap().get_action_from(&event) {
            self.last_keycode = Some(event.code);
            log_debug!("The action from input.handle: {action:?}");
            return Self::wrap_input_result(action, false);
        }

        if self.is_restart_sequence(&event.code) {
            self.last_keycode = Some(event.code);
            return Self::wrap_input_result(Action::Restart, true);
        }

        if self.is_typing_input(event, &ctx) {
            self.last_keycode = Some(event.code);
            if let Some(c) = event.code.as_char() {
                log_debug!("The action from input.handle: {:?}", Action::Input(c));
                return Self::wrap_input_result(Action::Input(c), false);
            }
        }

        let keymap = match ctx {
            InputContext::Idle => idle_keymap(),
            InputContext::Typing => typing_keymap(),
            InputContext::Completed => results_keymap(),
            InputContext::Modal => modal_keymap(),
            InputContext::Menu { searching: false } => menu_base_keymap(),
            InputContext::Menu { searching: true } => menu_search_keymap(),
        };

        self.last_keycode = Some(event.code);
        if let Some(action) = keymap.get_action_from(&event) {
            log_debug!("The action from input.handle: {action:?}");
            return Self::wrap_input_result(action, false);
        }

        // try handling menu shortcuts key inputs
        if self.is_menu_shortcut_input(event, &ctx) {
            if let Some(c) = event.code.as_char() {
                return Self::wrap_input_result(Action::MenuShortcut(c), false);
            }
        }

        // handle menu search query input
        if self.is_menu_search_input(event, &ctx) {
            if let Some(c) = event.code.as_char() {
                let action = Action::MenuUpdateSearch(c.to_string());
                return Self::wrap_input_result(action, false);
            }
        }

        // handle modal inputs
        if self.is_modal_input(event, &ctx) {
            if let Some(c) = event.code.as_char() {
                return Self::wrap_input_result(Action::ModalInput(c), false);
            }
        }

        Self::wrap_input_result(Action::NoOp, false)
    }

    fn is_restart_sequence(&self, current_keycode: &KeyCode) -> bool {
        match self.last_keycode {
            Some(last_keycode) => {
                matches!(last_keycode, KeyCode::Tab) && matches!(current_keycode, KeyCode::Enter)
            }
            _ => false,
        }
    }

    fn is_typing_input(&self, event: KeyEvent, ctx: &InputContext) -> bool {
        matches!(event.code, KeyCode::Char(_))
            && matches!(ctx, InputContext::Idle | InputContext::Typing)
            && !matches!(ctx, InputContext::Menu { .. })
    }

    fn is_modal_input(&self, event: KeyEvent, ctx: &InputContext) -> bool {
        matches!(event.code, KeyCode::Char(_)) && matches!(ctx, InputContext::Modal)
    }

    fn is_menu_shortcut_input(&self, event: KeyEvent, ctx: &InputContext) -> bool {
        matches!(event.code, KeyCode::Char(_))
            && matches!(ctx, InputContext::Menu { searching: false })
    }

    fn is_menu_search_input(&self, event: KeyEvent, ctx: &InputContext) -> bool {
        matches!(event.code, KeyCode::Char(_))
            && matches!(ctx, InputContext::Menu { searching: true })
    }

    /// Creates an InputResult with the given action and debounce skip flag.
    ///
    /// # Args
    /// * action - The action to return.
    /// * skip_debounce - Whether to skip debouncing or not in the main loop.
    fn wrap_input_result(action: Action, skip_debounce: bool) -> InputResult {
        InputResult {
            action,
            skip_debounce,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::MenuContext;
    use crossterm::event::KeyModifiers;

    fn create_event(mods: KeyModifiers, code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn test_quit_sequence() {
        let mut input = Input::new();
        let event = create_event(KeyModifiers::CONTROL, KeyCode::Char('c'));
        let result = input.handle(event, InputContext::Idle);

        assert_eq!(result.action, Action::Quit);
    }

    #[test]
    fn test_is_typing_input() {
        let mut input = Input::new();

        let typing_event = create_event(KeyModifiers::NONE, KeyCode::Char('c'));
        assert!(input.is_typing_input(typing_event, &InputContext::Idle));
        assert!(input.is_typing_input(typing_event, &InputContext::Typing));

        // should be a quit sequence
        let non_typing_event = create_event(KeyModifiers::CONTROL, KeyCode::Char('c'));
        assert_eq!(
            input.handle(non_typing_event, InputContext::Typing).action,
            Action::Quit
        );
    }

    #[test]
    fn test_restart_sequence() {
        // Tab+Enter for redo redo
        let mut input = Input::new();
        let event = create_event(KeyModifiers::NONE, KeyCode::Tab);
        let result = input.handle(event, InputContext::Idle);

        assert_eq!(result.action, Action::NoOp);

        let second_event = create_event(KeyModifiers::NONE, KeyCode::Enter);
        let second_result = input.handle(second_event, InputContext::Idle);
        assert_eq!(second_result.action, Action::Restart);
    }

    #[test]
    fn test_typing_input_action() {
        let mut input = Input::new();
        let event = create_event(KeyModifiers::NONE, KeyCode::Char('a'));
        let result = input.handle(event, InputContext::Typing);
        assert_eq!(result.action, Action::Input('a'));
    }

    #[test]
    fn test_menu_shortcut() {
        let mut input = Input::new();
        let event = create_event(KeyModifiers::NONE, KeyCode::Char('s'));
        let result = input.handle(event, InputContext::Menu { searching: false });
        assert_eq!(result.action, Action::MenuShortcut('s'));
    }

    #[test]
    fn test_menu_search_update() {
        let mut input = Input::new();
        let event = create_event(KeyModifiers::NONE, KeyCode::Char('q'));
        let result = input.handle(event, InputContext::Menu { searching: true });
        assert_eq!(result.action, Action::MenuUpdateSearch("q".to_string()));
    }

    #[test]
    fn test_idle_keymap_action() {
        let mut input = Input::new();
        let event = create_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = input.handle(event, InputContext::Idle);
        assert_eq!(result.action, Action::MenuOpen(MenuContext::Root));
    }

    #[test]
    fn test_noop_for_unhandled_key() {
        let mut input = Input::new();
        let event = create_event(KeyModifiers::NONE, KeyCode::F(12)); // Not bound
        let result = input.handle(event, InputContext::Idle);
        assert_eq!(result.action, Action::NoOp);
    }
}
