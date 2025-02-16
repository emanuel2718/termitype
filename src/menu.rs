use crate::{
    config::{Config, ModeType},
    constants::{DEFAULT_CURSOR_STYLE, DEFAULT_LANGUAGE, DEFAULT_THEME},
};

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
    pub has_submenu: bool,
    pub is_active: bool,
    pub is_toggleable: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            action,
            is_active: false,
            has_submenu: false,
            is_toggleable: false,
        }
    }

    pub fn toggleable(mut self, is_active: bool) -> Self {
        self.is_toggleable = true;
        self.is_active = is_active;
        self
    }

    pub fn submenufy(mut self) -> Self {
        self.has_submenu = true;
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

    pub fn current_item(&self) -> MenuItem {
        self.items[self.selected_index].clone()
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

    pub fn filtered_items(&self, query: &str) -> Vec<(usize, &MenuItem)> {
        if query.is_empty() {
            return self.items.iter().enumerate().collect();
        }

        let query = query.to_lowercase();
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                let label = item.label.to_lowercase();
                // most simple fuzzy search on the market
                Self::fuzzy_match(&label, &query)
            })
            .collect()
    }

    pub fn next_filtered_item(&mut self, query: &str) {
        let filtered = self.filtered_items(query);
        if filtered.is_empty() {
            return;
        }

        let current_pos = filtered.iter().position(|(i, _)| *i == self.selected_index);
        if let Some(pos) = current_pos {
            if pos + 1 < filtered.len() {
                self.selected_index = filtered[pos + 1].0;
            }
        } else {
            self.selected_index = filtered[0].0;
        }
    }

    pub fn prev_filtered_item(&mut self, query: &str) {
        let filtered = self.filtered_items(query);
        if filtered.is_empty() {
            return;
        }

        let current_pos = filtered.iter().position(|(i, _)| *i == self.selected_index);
        if let Some(pos) = current_pos {
            if pos > 0 {
                self.selected_index = filtered[pos - 1].0;
            }
        } else {
            self.selected_index = filtered[0].0;
        }
    }

    // TODO: improve this
    // TODO: move this to utils file
    fn fuzzy_match(text: &str, pattern: &str) -> bool {
        let text = text.chars().collect::<Vec<_>>();
        let pattern = pattern.chars().collect::<Vec<_>>();

        let mut text_idx = 0;
        let mut pattern_idx = 0;

        while text_idx < text.len() && pattern_idx < pattern.len() {
            if text[text_idx] == pattern[pattern_idx] {
                pattern_idx += 1;
            }
            text_idx += 1;
        }

        pattern_idx == pattern.len()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MenuState {
    menu_stack: Vec<Menu>,
    preview_theme: Option<String>,
    preview_cursor: Option<String>,
    search_query: String,
    is_searching: bool,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            menu_stack: Vec::new(),
            preview_theme: None,
            preview_cursor: None,
            search_query: String::new(),
            is_searching: false,
        }
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

    fn get_label_index(items: &[MenuItem], label: &str) -> Option<usize> {
        items
            .iter()
            .position(|item| item.label.to_lowercase() == label.to_lowercase())
    }

    pub fn execute(&mut self, action: MenuAction, config: &Config) -> Option<MenuAction> {
        match action {
            MenuAction::OpenMainMenu => {
                let menu = Menu::new(Self::build_main_menu(config));
                self.menu_stack.push(menu);
                None
            }
            MenuAction::OpenThemePicker => {
                let mut menu = Menu::new(Self::build_theme_picker());
                if let Some(index) = Self::get_label_index(
                    menu.items(),
                    config.theme.as_ref().unwrap_or(&DEFAULT_THEME.to_string()),
                ) {
                    menu.select(index);
                }
                self.menu_stack.push(menu);
                None
            }
            MenuAction::OpenLanguagePicker => {
                let mut menu = Menu::new(Self::build_language_picker());
                if let Some(index) = Self::get_label_index(
                    menu.items(),
                    config
                        .language
                        .as_ref()
                        .unwrap_or(&DEFAULT_LANGUAGE.to_string()),
                ) {
                    menu.select(index);
                }
                self.menu_stack.push(menu);
                None
            }
            MenuAction::OpenCursorPicker => {
                let mut menu = Menu::new(Self::build_cursor_picker());
                if let Some(index) = Self::get_label_index(
                    menu.items(),
                    config
                        .cursor_style
                        .as_ref()
                        .unwrap_or(&DEFAULT_CURSOR_STYLE.to_string()),
                ) {
                    menu.select(index);
                }
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
        self.is_searching = false;
        self.search_query.clear();
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
            MenuItem::new("Mode...", MenuAction::OpenModeSelector).submenufy(),
            MenuItem::new("Time...", MenuAction::OpenTimeSelector).submenufy(),
            MenuItem::new("Words...", MenuAction::OpenWordSelector).submenufy(),
            MenuItem::new("Language...", MenuAction::OpenLanguagePicker).submenufy(),
            MenuItem::new("Theme...", MenuAction::OpenThemePicker).submenufy(),
            MenuItem::new("Cursor...", MenuAction::OpenCursorPicker).submenufy(),
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
        Self::build_generic_menu(themes, MenuAction::ChangeTheme, {
            |a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase())
        })
    }

    fn build_language_picker() -> Vec<MenuItem> {
        let languages = crate::builder::Builder::available_languages().to_vec();
        Self::build_generic_menu(languages, MenuAction::ChangeLanguage, {
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
        Self::build_generic_menu(times, MenuAction::ChangeTime, {
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
        Self::build_generic_menu(word_counts, MenuAction::ChangeWordCount, {
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

    pub fn enter(&mut self, config: &Config) -> Option<MenuAction> {
        if let Some(menu) = self.current_menu() {
            let selected_item = menu.selected_item()?;
            let action = selected_item.action.clone();
            self.execute(action, config)
        } else {
            None
        }
    }

    pub fn next_item(&mut self) {
        let is_searching = self.is_searching();
        let query = if is_searching {
            self.search_query.clone()
        } else {
            String::new()
        };

        if let Some(menu) = self.current_menu_mut() {
            if is_searching {
                menu.next_filtered_item(&query);
            } else {
                menu.next_item();
            }
            self.preview_selected();
        }
    }

    pub fn prev_item(&mut self) {
        let is_searching = self.is_searching();
        let query = if is_searching {
            self.search_query.clone()
        } else {
            String::new()
        };

        if let Some(menu) = self.current_menu_mut() {
            if is_searching {
                menu.prev_filtered_item(&query);
            } else {
                menu.prev_item();
            }
            self.preview_selected();
        }
    }

    pub fn update_toggles(&mut self, config: &Config) {
        if let Some(menu) = self.current_menu_mut() {
            menu.update_toggles(config);
        }
    }

    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn start_search(&mut self) {
        self.is_searching = true;
    }

    pub fn cancel_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
    }

    pub fn finish_search(&mut self) {
        self.is_searching = false;
    }

    pub fn update_search(&mut self, query: &str) {
        self.search_query = query.to_string();
        if let Some(menu) = self.current_menu() {
            if let Some(index) = self.find_best_match(menu, &self.search_query) {
                self.select(index);
            }
        }
    }

    pub fn find_best_match(&self, menu: &Menu, query: &str) -> Option<usize> {
        if query.is_empty() {
            return None;
        }

        let filtered = menu.filtered_items(query);
        if filtered.is_empty() {
            None
        } else {
            Some(filtered[0].0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_menu() -> MenuState {
        MenuState::new()
    }

    #[test]
    fn test_menu_navigation() {
        let mut menu = create_test_menu();
        menu.toggle(&Config::default());
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
        menu.toggle(&Config::default());

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
        menu.toggle(&Config::default());
        config.use_punctuation = true;

        menu.update_toggles(&config);

        if let Some(menu_ref) = menu.current_menu() {
            let toggle_item = menu_ref.items().iter()
                .find(|item| matches!(item.action, MenuAction::ToggleFeature(ref f) if f == "punctuation"))
                .unwrap();
            assert!(toggle_item.is_active);
        }
    }

    #[test]
    fn test_search_functionality() {
        let mut menu = create_test_menu();
        menu.toggle(&Config::default());
        assert!(!menu.is_searching());

        // start search
        menu.start_search();
        assert!(menu.is_searching());
        assert_eq!(menu.search_query(), "");

        // update serach query
        menu.update_search("theme");
        assert_eq!(menu.search_query(), "theme");
        if let Some(menu) = menu.current_menu() {
            let selected_item = menu.selected_item().unwrap();
            assert_eq!(selected_item.label, "Theme...");
        }

        // fuzzy
        menu.update_search("thm");
        if let Some(menu) = menu.current_menu() {
            let selected_item = menu.selected_item().unwrap();
            assert_eq!(selected_item.label, "Theme...");
        }

        // cancel
        menu.cancel_search();
        assert!(!menu.is_searching());
        assert_eq!(menu.search_query(), "");

        menu.update_search("cur");
        menu.finish_search();
        assert!(!menu.is_searching());
        assert_eq!(menu.search_query(), "cur");

        menu.start_search();
        assert!(menu.is_searching());
        assert_eq!(menu.search_query(), "cur");
    }
}
