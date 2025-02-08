use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    Restart,
    Toggle(String),
    ChangeMode,
    ChangeTheme(String),
    ChangeCursorStyle(String),
    ChangeLanguage(String),
    Quit,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub content: MenuContent,
    pub is_active: bool,
    pub is_toggleable: bool,
}

#[derive(Debug, Clone)]
pub enum MenuContent {
    Action(MenuAction),
    SubMenu(Vec<MenuItem>),
}

impl MenuItem {
    pub fn new(label: impl Into<String>, content: MenuContent) -> Self {
        Self {
            label: label.into(),
            content,
            is_toggleable: false,
            is_active: false,
        }
    }

    pub fn toggleable(mut self, is_active: bool) -> Self {
        self.is_toggleable = true;
        self.is_active = is_active;
        self
    }
}

#[derive(Default, Debug, Clone)]
pub struct MenuState {
    menu_stack: Vec<(Vec<MenuItem>, usize)>, // (items, selected_idx)
    preview_theme: Option<String>,
    preview_cursor: Option<String>,
}

impl MenuState {
    pub fn new(config: &Config) -> Self {
        let mut state = Self::default();
        state.build_main_menu(config);
        state
    }

    pub fn build_main_menu(&mut self, config: &Config) {
        let menu = vec![
            MenuItem::new("Restart", MenuContent::Action(MenuAction::Restart)),
            MenuItem::new(
                "Toggle Punctuation",
                MenuContent::Action(MenuAction::Toggle("punctuation".into())),
            )
            .toggleable(config.use_punctuation),
            MenuItem::new(
                "Toggle Numbers",
                MenuContent::Action(MenuAction::Toggle("numbers".into())),
            )
            .toggleable(config.use_numbers),
            MenuItem::new(
                "Toggle Symbols",
                MenuContent::Action(MenuAction::Toggle("symbols".into())),
            )
            .toggleable(config.use_symbols),
            MenuItem::new("Change Mode", MenuContent::Action(MenuAction::ChangeMode)),
            MenuItem::new(
                "Language Picker",
                MenuContent::SubMenu(Self::build_language_picker()),
            ),
            MenuItem::new(
                "Theme Picker",
                MenuContent::SubMenu(Self::build_theme_picker()),
            ),
            MenuItem::new(
                "Change Cursor",
                MenuContent::SubMenu(Self::build_cursor_style_menu()),
            ),
            MenuItem::new("Exit", MenuContent::Action(MenuAction::Quit)),
        ];
        self.menu_stack.push((menu, 0));
    }

    // TODO: we could make this be a generic builder to avoid duplication.
    fn build_language_picker() -> Vec<MenuItem> {
        let mut languages = crate::builder::Builder::available_languages().to_vec();
        languages.sort_by_key(|a| a.to_lowercase());

        languages
            .into_iter()
            .map(|lang| {
                MenuItem::new(
                    lang.clone(),
                    MenuContent::Action(MenuAction::ChangeLanguage(lang.clone())),
                )
            })
            .collect()
    }

    fn build_theme_picker() -> Vec<MenuItem> {
        let mut themes = crate::theme::available_themes().to_vec();
        themes.sort_by_key(|a| a.to_lowercase());

        themes
            .into_iter()
            .map(|theme| {
                MenuItem::new(
                    theme.clone(),
                    MenuContent::Action(MenuAction::ChangeTheme(theme.clone())),
                )
            })
            .collect()
    }

    fn build_cursor_style_menu() -> Vec<MenuItem> {
        vec![
            MenuItem::new(
                "Beam",
                MenuContent::Action(MenuAction::ChangeCursorStyle("beam".into())),
            ),
            MenuItem::new(
                "Block",
                MenuContent::Action(MenuAction::ChangeCursorStyle("block".into())),
            ),
            MenuItem::new(
                "Underline",
                MenuContent::Action(MenuAction::ChangeCursorStyle("underline".into())),
            ),
            MenuItem::new(
                "Blinking Beam",
                MenuContent::Action(MenuAction::ChangeCursorStyle("blinking-beam".into())),
            ),
            MenuItem::new(
                "Blinking Block",
                MenuContent::Action(MenuAction::ChangeCursorStyle("blinking-block".into())),
            ),
            MenuItem::new(
                "Blinking Underline",
                MenuContent::Action(MenuAction::ChangeCursorStyle("blinking-underline".into())),
            ),
        ]
    }

    pub fn is_open(&self) -> bool {
        !self.menu_stack.is_empty()
    }

    pub fn menu_depth(&self) -> usize {
        self.menu_stack.len()
    }

    pub fn current_menu(&self) -> Option<&(Vec<MenuItem>, usize)> {
        self.menu_stack.last()
    }

    pub fn current_menu_mut(&mut self) -> Option<&mut (Vec<MenuItem>, usize)> {
        self.menu_stack.last_mut()
    }

    pub fn select_from_menu(&mut self, index: usize) {
        if let Some((items, idx)) = self.current_menu_mut() {
            if index < items.len() {
                *idx = index;
            }
        }
    }

    pub fn selected_menu_item(&self) -> Option<&MenuItem> {
        self.current_menu().and_then(|(items, idx)| items.get(*idx))
    }

    pub fn prev_menu_item(&mut self) {
        if let Some((_, idx)) = self.current_menu_mut() {
            if *idx > 0 {
                *idx -= 1;
            }
        }
    }

    pub fn next_menu_item(&mut self) {
        if let Some((items, idx)) = self.current_menu_mut() {
            if *idx < items.len() - 1 {
                *idx += 1;
            }
        }
    }

    pub fn toggle(&mut self, config: &Config) {
        if self.is_open() {
            self.menu_stack.clear();
            self.preview_theme = None;
            self.preview_cursor = None;
        } else {
            self.build_main_menu(config);
        }
    }

    pub fn get_preview_theme(&self) -> Option<&String> {
        self.preview_theme.as_ref()
    }

    pub fn get_preview_cursor(&self) -> Option<&String> {
        self.preview_cursor.as_ref()
    }

    pub fn preview_selected_theme(&mut self) {
        if let Some(item) = self.selected_menu_item() {
            if let MenuContent::Action(MenuAction::ChangeTheme(theme)) = &item.content {
                self.preview_theme = Some(theme.clone());
            }
        }
    }

    pub fn preview_selected_cursor(&mut self) {
        if let Some(item) = self.selected_menu_item() {
            if let MenuContent::Action(MenuAction::ChangeCursorStyle(cursor)) = &item.content {
                self.preview_cursor = Some(cursor.clone());
            }
        }
    }

    pub fn menu_back(&mut self) {
        if self.menu_depth() > 1 {
            if let Some((items, _)) = self.current_menu() {
                if items.iter().any(|item| {
                    matches!(
                        item.content,
                        MenuContent::Action(MenuAction::ChangeTheme(_))
                    )
                }) {
                    // NOTE: we shouldn't clear something that doesn't need clearing...wasteful
                    self.preview_theme = None;
                    self.preview_cursor = None;
                }
            }
            self.menu_stack.pop();
        } else {
            self.menu_stack.clear();
            self.preview_theme = None;
            self.preview_cursor = None;
        }
    }

    pub fn menu_enter(&mut self) -> Option<MenuAction> {
        let action = if let Some((items, selected)) = self.current_menu() {
            if let Some(item) = items.get(*selected) {
                match &item.content {
                    MenuContent::Action(action) => {
                        let should_clear = !item.is_toggleable;
                        let action = action.clone();
                        if should_clear {
                            self.menu_stack.clear();
                            if matches!(action, MenuAction::ChangeTheme(_)) {
                                self.preview_theme = None;
                            }
                            if matches!(action, MenuAction::ChangeCursorStyle(_)) {
                                self.preview_cursor = None;
                            }
                        }
                        Some(action)
                    }
                    MenuContent::SubMenu(submenu) => {
                        self.menu_stack.push((submenu.clone(), 0));
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };
        action
    }

    pub fn update_toggles(&mut self, config: &Config) {
        if let Some((items, _)) = self.menu_stack.first_mut() {
            for item in items.iter_mut() {
                if item.is_toggleable {
                    if let MenuContent::Action(MenuAction::Toggle(feature)) = &item.content {
                        item.is_active = match feature.as_str() {
                            "punctuation" => config.use_punctuation,
                            "numbers" => config.use_numbers,
                            "symbols" => config.use_symbols,
                            _ => item.is_active,
                        };
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_menu() -> MenuState {
        let mut menu = MenuState::default();
        let config = Config::default();
        menu.build_main_menu(&config);
        menu
    }

    fn go_back_and_check_theme_preview_is_cleared(menu: &mut MenuState) {
        menu.menu_back();
        assert!(menu.get_preview_theme().is_none());
    }

    #[test]
    fn test_theme_preview() {
        let mut menu = create_test_menu();

        menu.select_from_menu(6); // select theme picker
        assert!(menu.menu_enter().is_none()); // should return None as we're entering a submenu
        assert_eq!(menu.menu_depth(), 2); // should be in submenu
        assert!(menu.get_preview_theme().is_none()); // no theme preview yet

        // preview first theme we find
        if let Some((items, _)) = menu.current_menu() {
            let first_theme = items[0].label.clone();
            menu.preview_selected_theme();
            assert_eq!(menu.get_preview_theme(), Some(&first_theme));
        }

        // preview theme should be clear when we go back
        go_back_and_check_theme_preview_is_cleared(&mut menu);
    }

    #[test]
    fn test_language_picker() {
        let mut menu = create_test_menu();

        menu.select_from_menu(5);
        assert!(menu.menu_enter().is_none());
        assert_eq!(menu.menu_depth(), 2);

        // select first language
        if let Some((items, _)) = menu.current_menu() {
            let first_lang = items[0].label.clone();
            if let Some(MenuAction::ChangeLanguage(selected_lang)) = menu.menu_enter() {
                assert_eq!(selected_lang, first_lang);
            } else {
                panic!("Expected ChangeLanguage action");
            }
            assert!(!menu.is_open());
        }
    }

    #[test]
    fn test_theme_selection() {
        let mut menu = create_test_menu();

        menu.select_from_menu(6);
        menu.menu_enter();

        // select first theme
        if let Some((items, _)) = menu.current_menu() {
            let first_theme = items[0].label.clone();
            menu.preview_selected_theme();

            assert_eq!(menu.get_preview_theme(), Some(&first_theme));

            if let Some(MenuAction::ChangeTheme(selected_theme)) = menu.menu_enter() {
                assert_eq!(selected_theme, first_theme);
            } else {
                panic!("Expected ChangeTheme action");
            }

            assert!(!menu.is_open());
            assert!(menu.get_preview_theme().is_none());
        }
    }
    #[test]
    fn test_clear_cusor_preview() {
        let mut menu = create_test_menu();
        menu.select_from_menu(7);
        menu.menu_enter();

        if let Some((items, _)) = menu.current_menu() {
            let mut second_cursor = items[2].label.clone();
            menu.next_menu_item();
            menu.next_menu_item();

            menu.preview_selected_cursor();

            // Why is capitalizing the first letter of a string so convoluted in Rust?
            // https://stackoverflow.com/a/69996191
            let uncapitalized_cursor =
                format!("{}{second_cursor}", second_cursor.remove(0).to_lowercase());
            assert_eq!(menu.get_preview_cursor(), Some(&uncapitalized_cursor));

            if let Some(MenuAction::ChangeCursorStyle(selected_cursor)) = menu.menu_enter() {
                assert_eq!(selected_cursor, uncapitalized_cursor);
            } else {
                panic!("Expected ChangeCursorStyle action");
            }

            assert!(!menu.is_open());
            assert!(menu.get_preview_cursor().is_none());
        }
    }
}
