use crate::config::{Config, ModeType};

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    OpenMainMenu,
    OpenThemePicker,
    OpenLanguagePicker,
    OpenCursorPicker,
    OpenModeSelector,
    OpenTimeSelector,
    OpenWordSelector,
    Back,
    Close,

    ToggleFeature(String),
    ChangeMode(ModeType),
    ChangeTime(u64),
    ChangeWordCount(usize),
    ChangeTheme(String),
    ChangeCursorStyle(String),
    ChangeLanguage(String),
    Restart,
    Quit,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
    pub is_active: bool,
    pub is_toggleable: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            action,
            is_active: false,
            is_toggleable: false,
        }
    }

    pub fn toggleable(mut self, is_active: bool) -> Self {
        self.is_toggleable = true;
        self.is_active = is_active;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Menu {
    items: Vec<MenuItem>,
    selected_index: usize,
}

impl Menu {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            selected_index: 0,
        }
    }

    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_index)
    }

    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = index;
        }
    }

    pub fn next_item(&mut self) {
        if self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn prev_item(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn update_toggles(&mut self, config: &Config) {
        for item in &mut self.items {
            if item.is_toggleable {
                if let MenuAction::ToggleFeature(feature) = &item.action {
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

#[derive(Default, Debug, Clone)]
pub struct MenuState {
    menu_stack: Vec<Menu>,
    preview_theme: Option<String>,
    preview_cursor: Option<String>,
}

impl MenuState {
    pub fn new(config: &Config) -> Self {
        let mut state = Self::default();
        state.execute(MenuAction::OpenMainMenu, config);
        state
    }

    pub fn is_open(&self) -> bool {
        !self.menu_stack.is_empty()
    }

    pub fn menu_depth(&self) -> usize {
        self.menu_stack.len()
    }

    pub fn current_menu(&self) -> Option<&Menu> {
        self.menu_stack.last()
    }

    pub fn current_menu_mut(&mut self) -> Option<&mut Menu> {
        self.menu_stack.last_mut()
    }

    pub fn toggle(&mut self, config: &Config) {
        if self.is_open() {
            self.menu_stack.clear();
            self.clear_previews();
        } else {
            self.execute(MenuAction::OpenMainMenu, config);
        }
    }

    pub fn back(&mut self) {
        if self.menu_depth() > 1 {
            self.menu_stack.pop();
        } else {
            self.menu_stack.clear();
        }
        self.clear_previews();
    }

    pub fn execute(&mut self, action: MenuAction, config: &Config) -> Option<MenuAction> {
        match action {
            MenuAction::OpenMainMenu => {
                let menu = Menu::new(Self::build_main_menu(config));
                self.menu_stack.push(menu);
                None
            }
            MenuAction::OpenThemePicker => {
                let menu = Menu::new(Self::build_theme_picker());
                self.menu_stack.push(menu);
                None
            }
            MenuAction::OpenLanguagePicker => {
                let menu = Menu::new(Self::build_language_picker());
                self.menu_stack.push(menu);
                None
            }
            MenuAction::OpenCursorPicker => {
                let menu = Menu::new(Self::build_cursor_picker());
                self.menu_stack.push(menu);
                None
            }
            MenuAction::OpenModeSelector => {
                let menu = Menu::new(Self::build_mode_menu());
                self.menu_stack.push(menu);
                None
            }
            MenuAction::OpenTimeSelector => {
                let menu = Menu::new(Self::build_time_menu());
                self.menu_stack.push(menu);
                None
            }
            MenuAction::OpenWordSelector => {
                let menu = Menu::new(Self::build_words_menu());
                self.menu_stack.push(menu);
                None
            }
            MenuAction::Back => {
                self.back();
                None
            }
            MenuAction::Close => {
                self.menu_stack.clear();
                self.clear_previews();
                None
            }
            // return the other actions to be handled by the caller
            action => {
                // clear menu stack for non-toggle actions
                if !matches!(action, MenuAction::ToggleFeature(_)) {
                    self.menu_stack.clear();
                    self.clear_previews();
                }
                Some(action)
            }
        }
    }

    fn clear_previews(&mut self) {
        self.preview_theme = None;
        self.preview_cursor = None;
    }

    pub fn preview_selected(&mut self) {
        let action = self
            .current_menu()
            .and_then(|menu| menu.selected_item())
            .map(|item| item.action.clone());

        if let Some(action) = action {
            match action {
                MenuAction::ChangeTheme(theme) => self.preview_theme = Some(theme),
                MenuAction::ChangeCursorStyle(cursor) => self.preview_cursor = Some(cursor),
                _ => {}
            }
        }
    }

    pub fn get_preview_theme(&self) -> Option<&String> {
        self.preview_theme.as_ref()
    }

    pub fn get_preview_cursor(&self) -> Option<&String> {
        self.preview_cursor.as_ref()
    }

    fn build_main_menu(config: &Config) -> Vec<MenuItem> {
        vec![
            MenuItem::new("Restart", MenuAction::Restart),
            MenuItem::new(
                "Toggle Punctuation",
                MenuAction::ToggleFeature("punctuation".into()),
            )
            .toggleable(config.use_punctuation),
            MenuItem::new(
                "Toggle Numbers",
                MenuAction::ToggleFeature("numbers".into()),
            )
            .toggleable(config.use_numbers),
            MenuItem::new(
                "Toggle Symbols",
                MenuAction::ToggleFeature("symbols".into()),
            )
            .toggleable(config.use_symbols),
            MenuItem::new("Mode...", MenuAction::OpenModeSelector),
            MenuItem::new("Time...", MenuAction::OpenTimeSelector),
            MenuItem::new("Words...", MenuAction::OpenWordSelector),
            MenuItem::new("Language...", MenuAction::OpenLanguagePicker),
            MenuItem::new("Theme...", MenuAction::OpenThemePicker),
            MenuItem::new("Cursor...", MenuAction::OpenCursorPicker),
            MenuItem::new("Exit", MenuAction::Quit),
        ]
    }

    fn build_generic_menu<T: ToString + Clone>(
        items: Vec<T>,
        action_builder: impl Fn(T) -> MenuAction,
        sorter: impl Fn(&MenuItem, &MenuItem) -> std::cmp::Ordering,
    ) -> Vec<MenuItem> {
        let mut menu_items: Vec<MenuItem> = items
            .into_iter()
            .map(|item| {
                let label = item.to_string();
                MenuItem::new(label, action_builder(item))
            })
            .collect();

        menu_items.sort_by(sorter);
        menu_items
    }

    fn build_theme_picker() -> Vec<MenuItem> {
        let themes = crate::theme::available_themes().to_vec();
        Self::build_generic_menu(themes, |theme| MenuAction::ChangeTheme(theme), {
            |a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase())
        })
    }

    fn build_language_picker() -> Vec<MenuItem> {
        let languages = crate::builder::Builder::available_languages().to_vec();
        Self::build_generic_menu(languages, |lang| MenuAction::ChangeLanguage(lang), {
            |a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase())
        })
    }

    fn build_cursor_picker() -> Vec<MenuItem> {
        vec![
            MenuItem::new("Beam", MenuAction::ChangeCursorStyle("beam".into())),
            MenuItem::new("Block", MenuAction::ChangeCursorStyle("block".into())),
            MenuItem::new(
                "Underline",
                MenuAction::ChangeCursorStyle("underline".into()),
            ),
            MenuItem::new(
                "Blinking Beam",
                MenuAction::ChangeCursorStyle("blinking-beam".into()),
            ),
            MenuItem::new(
                "Blinking Block",
                MenuAction::ChangeCursorStyle("blinking-block".into()),
            ),
            MenuItem::new(
                "Blinking Underline",
                MenuAction::ChangeCursorStyle("blinking-underline".into()),
            ),
        ]
    }

    fn build_mode_menu() -> Vec<MenuItem> {
        vec![
            MenuItem::new("Time", MenuAction::ChangeMode(ModeType::Time)),
            MenuItem::new("Words", MenuAction::ChangeMode(ModeType::Words)),
        ]
    }

    fn build_time_menu() -> Vec<MenuItem> {
        let times = vec![15, 30, 60, 120];
        Self::build_generic_menu(times, |time| MenuAction::ChangeTime(time), {
            |a, b| {
                a.label
                    .parse::<u32>()
                    .unwrap_or(0)
                    .cmp(&b.label.parse::<u32>().unwrap_or(0))
            }
        })
    }

    fn build_words_menu() -> Vec<MenuItem> {
        let word_counts = vec![10, 25, 50, 100];
        Self::build_generic_menu(word_counts, |count| MenuAction::ChangeWordCount(count), {
            |a, b| {
                a.label
                    .parse::<u32>()
                    .unwrap_or(0)
                    .cmp(&b.label.parse::<u32>().unwrap_or(0))
            }
        })
    }

    pub fn select(&mut self, index: usize) {
        if let Some(menu) = self.current_menu_mut() {
            menu.select(index);
            self.preview_selected();
        }
    }

    pub fn enter(&mut self) -> Option<MenuAction> {
        if let Some(menu) = self.current_menu() {
            let selected_item = menu.selected_item()?;
            let action = selected_item.action.clone();
            self.execute(action, &Config::default())
        } else {
            None
        }
    }

    pub fn next_item(&mut self) {
        if let Some(menu) = self.current_menu_mut() {
            menu.next_item();
            self.preview_selected();
        }
    }

    pub fn prev_item(&mut self) {
        if let Some(menu) = self.current_menu_mut() {
            menu.prev_item();
            self.preview_selected();
        }
    }

    pub fn update_toggles(&mut self, config: &Config) {
        if let Some(menu) = self.current_menu_mut() {
            menu.update_toggles(config);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_menu() -> MenuState {
        MenuState::new(&Config::default())
    }

    #[test]
    fn test_menu_navigation() {
        let mut menu = create_test_menu();
        assert!(menu.is_open());

        menu.next_item();
        menu.next_item();
        assert_eq!(menu.current_menu().unwrap().selected_index(), 2);

        menu.prev_item();
        assert_eq!(menu.current_menu().unwrap().selected_index(), 1);

        menu.select(5);
        assert_eq!(menu.current_menu().unwrap().selected_index(), 5);
    }

    #[test]
    fn test_theme_picker() {
        let mut menu = create_test_menu();

        let theme_index = if let Some(menu_ref) = menu.current_menu() {
            menu_ref
                .items()
                .iter()
                .position(|item| matches!(item.action, MenuAction::OpenThemePicker))
                .unwrap()
        } else {
            panic!("Menu should be open");
        };
        menu.select(theme_index);

        let theme_action = menu
            .current_menu()
            .and_then(|menu_ref| menu_ref.selected_item())
            .map(|item| item.action.clone());

        if let Some(action) = theme_action {
            menu.execute(action, &Config::default());
        }

        assert_eq!(menu.menu_depth(), 2);

        let has_theme = menu
            .current_menu()
            .and_then(|menu_ref| menu_ref.selected_item())
            .map(|item| matches!(item.action, MenuAction::ChangeTheme(_)))
            .unwrap_or(false);

        if has_theme {
            menu.preview_selected();
            assert!(menu.get_preview_theme().is_some());
        }
    }

    #[test]
    fn test_toggle_features() {
        let mut menu = create_test_menu();
        let mut config = Config::default();
        config.use_punctuation = true;

        menu.update_toggles(&config);

        if let Some(menu_ref) = menu.current_menu() {
            let toggle_item = menu_ref.items().iter()
                .find(|item| matches!(item.action, MenuAction::ToggleFeature(ref f) if f == "punctuation"))
                .unwrap();
            assert!(toggle_item.is_active);
        }
    }
}
