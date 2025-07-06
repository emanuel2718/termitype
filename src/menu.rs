use crate::{
    actions::{MenuContext, MenuNavAction, MenuSearchAction, PreviewType, TermiAction},
    ascii::DEFAULT_ASCII_ART_NAME,
    config::Config,
    constants::{DEFAULT_TIME_DURATION_LIST, DEFAULT_TIME_MODE_DURATION},
    helpers::fuzzy_match,
    log_debug,
    menu_builder::build_menu,
    styles::TermiStyle,
};

/// Represents the resulting action of selecting a menu item
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemResult {
    TriggerAction(TermiAction),
    OpenSubMenu(MenuContext),
    ToggleState,
    NoOp,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub key: Option<String>,
    pub label: String,
    pub result: MenuItemResult,
    // TODO: this is weird, want to find a better name represantion than `is_active`.
    pub is_active: Option<bool>, // true/false if toggleable, None otherwise
    pub is_disabled: bool,
    pub preview_type: Option<PreviewType>,
}

impl MenuItem {
    pub fn action(id: &str, label: &str, action: TermiAction) -> Self {
        Self {
            id: id.to_string(),
            key: None,
            label: label.to_string(),
            result: MenuItemResult::TriggerAction(action),
            is_active: None,
            is_disabled: false,
            preview_type: None,
        }
    }

    // NOTE: eventually info items could have an action (i.e opening the url for the repo etc.)
    /// MenuItem with Key Value pair information
    pub fn info(id: &str, key: &str, desc: &str) -> Self {
        Self {
            id: id.to_string(),
            key: Some(key.to_string()),
            label: desc.to_string(),
            result: MenuItemResult::NoOp,
            is_active: None,
            is_disabled: false,
            preview_type: None,
        }
    }

    pub fn toggle(id: &str, label: &str, is_active: bool) -> Self {
        Self {
            id: id.to_string(),
            key: None,
            label: label.to_string(),
            result: MenuItemResult::ToggleState,
            is_active: Some(is_active),
            is_disabled: false,
            preview_type: None,
        }
    }

    pub fn sub_menu(id: &str, label: &str, ctx: MenuContext) -> Self {
        Self {
            id: id.to_string(),
            key: None,
            label: label.to_string(),
            result: MenuItemResult::OpenSubMenu(ctx),
            is_active: None,
            is_disabled: false,
            preview_type: None,
        }
    }

    pub fn with_preview(mut self, preview: PreviewType) -> Self {
        self.preview_type = Some(preview);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub ctx: MenuContext,
    pub title: String,
    _items: Vec<MenuItem>,
    current_index: usize,
    filtered_indices: Option<Vec<usize>>,
}

impl Menu {
    pub fn new(ctx: MenuContext, title: String, items: Vec<MenuItem>) -> Self {
        Menu {
            ctx,
            title,
            _items: items,
            current_index: 0,
            filtered_indices: None,
        }
    }

    pub fn current_selection_index(&self) -> usize {
        self.current_index
    }

    pub fn navigate(&mut self, nav: MenuNavAction, ui_height: usize) {
        let items_count = self.size();
        if items_count == 0 {
            return;
        }

        match nav {
            MenuNavAction::Up => self.current_index = self.current_index.saturating_sub(1),
            MenuNavAction::Down => {
                self.current_index = (self.current_index + 1).min(items_count - 1)
            }
            MenuNavAction::PageUp => {
                let scroll_amount = (ui_height / 2).max(1);
                self.current_index = self.current_index.saturating_sub(scroll_amount).max(0)
            }
            MenuNavAction::PageDown => {
                let scroll_amount = (ui_height / 2).max(1);
                self.current_index = (self.current_index + scroll_amount).min(items_count - 1)
            }
            MenuNavAction::Home => {
                self.current_index = 0;
            }
            MenuNavAction::End => self.current_index = items_count - 1,
            MenuNavAction::Back => {} // handled by MenuState
        }
    }

    /// Amount of items in the current menu. Takes into consideration if there is a current filer
    pub fn size(&self) -> usize {
        if let Some(indices) = &self.filtered_indices {
            indices.len()
        } else {
            self._items.len()
        }
    }

    pub fn current_item(&self) -> Option<&MenuItem> {
        // if filtering, `selected_index` is an index into `filtered_indexes`
        // which then itself points to the actual item in `self.items`. cool
        if let Some(indices) = &self.filtered_indices {
            indices
                .get(self.current_index)
                .and_then(|&og_idx| self._items.get(og_idx))
        } else {
            self._items.get(self.current_index)
        }
    }

    /// Toggles the state of the current item if is a toggleable item
    pub fn toggle_item(&mut self, id: &str) {
        if let Some(item) = self._items.iter_mut().find(|item| item.id == id) {
            if let Some(is_active) = item.is_active {
                item.is_active = Some(!is_active)
            }
        }
    }

    pub fn items(&self) -> Vec<MenuItem> {
        if let Some(indices) = &self.filtered_indices {
            indices
                .iter()
                .filter_map(|&i| self._items.get(i).cloned())
                .collect()
        } else {
            self._items.clone()
        }
    }

    pub fn items_with_indices(&self) -> Vec<(usize, &MenuItem)> {
        self._items.iter().enumerate().collect()
    }

    pub fn filtered_items(&self, query: &str) -> Vec<(usize, &MenuItem)> {
        if query.is_empty() {
            return self._items.iter().enumerate().collect();
        }

        let query = query.to_lowercase();
        self._items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                !item.is_disabled && fuzzy_match(&item.label.to_lowercase(), &query)
            })
            .collect()
    }

    pub fn filter_items(&mut self, query: &str) -> Vec<MenuItem> {
        self.update_filtered_indices(query);
        self.items()
    }

    fn update_filtered_indices(&mut self, query: &str) {
        if query.is_empty() {
            self.filtered_indices = None;
            // self.current_index = 0;
            return;
        }

        let query = query.to_lowercase();
        let indices: Vec<usize> = self
            ._items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                !item.is_disabled && fuzzy_match(&item.label.to_lowercase(), &query)
            })
            .map(|(i, _)| i)
            .collect();

        self.filtered_indices = Some(indices);
        self.current_index = 0
    }
}

#[derive(Default, Debug, Clone)]
pub struct MenuState {
    stack: Vec<Menu>,
    search_query: String,
    is_searching: bool,
    pub ui_height: usize,
}

impl MenuState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    /// Checks if the given `MenuContext` is the current active context
    pub fn is_current_ctx(&self, ctx: MenuContext) -> bool {
        if let Some(menu) = self.current_menu() {
            return menu.ctx == ctx;
        }
        false
    }

    pub fn current_menu(&self) -> Option<&Menu> {
        self.stack.last()
    }

    pub fn current_menu_mut(&mut self) -> Option<&mut Menu> {
        self.stack.last_mut()
    }

    // =============== HANDLERS ===============
    pub fn handle_action(&mut self, action: TermiAction, config: &Config) -> Option<TermiAction> {
        match action {
            // Open
            TermiAction::MenuOpen(ctx) => {
                if self.is_current_ctx(MenuContext::Theme)
                    || self.is_current_ctx(MenuContext::Help)
                    || self.is_current_ctx(MenuContext::About)
                    || self.is_current_ctx(MenuContext::AsciiArt)
                    || self.is_current_ctx(MenuContext::Options)
                {
                    self.close();
                } else {
                    self.open(ctx, config);
                }
                self.preview_selection()
            }
            // Close
            TermiAction::MenuClose => {
                self.stack.clear();
                self.clear_search();
                return Some(TermiAction::ClearPreview);
            }
            // Select
            TermiAction::MenuSelect => {
                return self.execute_menu_action(config);
            }
            // Search
            TermiAction::MenuSearch(search_action) => {
                let action = self.handle_search_action(search_action.clone(), config);
                if action.is_some() {
                    return action;
                }
                self.preview_selection()
            }
            // Navigate
            TermiAction::MenuNavigate(nav_action) => {
                // TODO: this could be simplified
                if nav_action == MenuNavAction::Back {
                    if self.is_searching {
                        self.clear_search();
                        if let Some(menu) = self.stack.last_mut() {
                            menu.filter_items("");
                        }
                    } else {
                        self.stack.pop();
                        if self.stack.is_empty() {
                            return Some(TermiAction::ClearPreview);
                        }
                    }
                } else if let Some(menu) = self.stack.last_mut() {
                    menu.navigate(nav_action, self.ui_height);
                }
                return self.preview_selection();
            }
            _ => Some(TermiAction::NoOp),
        };
        None
    }

    fn handle_search_action(
        &mut self,
        action: MenuSearchAction,
        config: &Config,
    ) -> Option<TermiAction> {
        match action {
            MenuSearchAction::Start => self.is_searching = true,
            MenuSearchAction::Close => self.clear_search(),
            MenuSearchAction::Confirm => {
                self.is_searching = false;
                return self.execute_menu_action(config);
            }
            MenuSearchAction::Clear => self.search_query.clear(),
            MenuSearchAction::Backspace => {
                self.search_query.pop();
            }
            MenuSearchAction::Input(c) if self.is_searching => self.search_query.push(c),
            _ => {}
        }

        if let Some(menu) = self.stack.last_mut() {
            let query = &self.search_query;
            menu.filter_items(query);
        }

        self.preview_selection()
    }

    // =============== EXECUTOR ===============
    fn execute_menu_action(&mut self, config: &Config) -> Option<TermiAction> {
        let item_action = self
            .stack
            .last()
            .and_then(|menu| menu.current_item())
            .filter(|item| !item.is_disabled)
            .map(|item| item.result.clone());

        log_debug!("item action: {:?}", item_action);
        if let Some(action) = item_action {
            match action {
                MenuItemResult::TriggerAction(action) => {
                    // self.stack.clear();
                    // self.clear_search();
                    return Some(action);
                }
                MenuItemResult::OpenSubMenu(ctx) => {
                    self.open(ctx, config);
                    return self.preview_selection();
                }
                MenuItemResult::ToggleState => {
                    // TODO: extract this to a fn
                    if let Some(id) = self
                        .stack
                        .last()
                        .and_then(|m| m.current_item().map(|i| i.id.clone()))
                    {
                        // TODO: maybe this ids need to be MenuItemId enum
                        let act = match id.as_str() {
                            "options/symbols" => TermiAction::ToggleSymbols,
                            "options/punctuation" => TermiAction::TogglePunctuation,
                            "options/numbers" => TermiAction::ToggleNumbers,
                            "options/fps" => TermiAction::ToggleFPS,
                            "options/show_live_wpm" => TermiAction::ToggleLiveWPM,
                            "options/monochromatic" => TermiAction::ToggleMonochromaticResults,
                            "options/show_cursorline" => TermiAction::ToggleCursorline,
                            _ => TermiAction::NoOp,
                        };
                        return Some(act);
                    }
                }
                MenuItemResult::NoOp => return None,
            }
        }
        None
    }

    fn preview_selection(&self) -> Option<TermiAction> {
        self.stack
            .last()
            .and_then(|menu| menu.current_item())
            .and_then(|item| item.preview_type.clone())
            .map(TermiAction::ApplyPreview)
            .or(Some(TermiAction::ClearPreview))
    }

    fn open(&mut self, ctx: MenuContext, config: &Config) {
        self.stack.push(build_menu(ctx, config));
        self.clear_search();
        if let Some(menu) = self.stack.last_mut() {
            let index = MenuState::resolve_index(menu, config);
            if index < menu.items().len() {
                menu.current_index = index;
            }
        }
    }

    pub fn close(&mut self) {
        self.stack.clear();
        self.clear_search();
    }

    pub fn toggle(&mut self, config: &Config) {
        if self.is_open() {
            self.close();
        } else {
            self.open(MenuContext::Root, config);
        }
    }

    fn clear_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn sync_toggle_items(&mut self, config: &Config) {
        if let Some(menu) = self.stack.last_mut() {
            // TODO: there has to be a better way of doing this
            // FIXME: having to iterate throught the items to find every item and match to a toggle is not smart.
            // Options menu - basically will contain all the toggles of the game
            if let Some(item) = menu
                ._items
                .iter_mut()
                .find(|i| i.id == "options/punctuation")
            {
                item.is_active = Some(config.use_punctuation);
            }
            if let Some(item) = menu._items.iter_mut().find(|i| i.id == "options/symbols") {
                item.is_active = Some(config.use_symbols);
            }
            if let Some(item) = menu._items.iter_mut().find(|i| i.id == "options/numbers") {
                item.is_active = Some(config.use_numbers);
            }
            if let Some(item) = menu._items.iter_mut().find(|i| i.id == "options/fps") {
                item.is_active = Some(config.show_fps);
            }
            if let Some(item) = menu
                ._items
                .iter_mut()
                .find(|i| i.id == "options/show_live_wpm")
            {
                item.is_active = Some(!config.hide_live_wpm);
            }
            if let Some(item) = menu
                ._items
                .iter_mut()
                .find(|i| i.id == "options/show_cursorline")
            {
                item.is_active = Some(!config.hide_cursorline);
            }
            if let Some(item) = menu
                ._items
                .iter_mut()
                .find(|i| i.id == "options/monochromatic")
            {
                item.is_active = Some(config.monocrhomatic_results);
            }
        }
    }

    /// Used for auto selecting the currently selected item when opening a submenu
    fn resolve_index(menu: &Menu, config: &Config) -> usize {
        let target_id: Option<String> = match menu.ctx {
            MenuContext::Theme => {
                let theme_name = config
                    .theme
                    .as_deref()
                    .unwrap_or(crate::constants::DEFAULT_THEME);
                Some(format!("themes/{theme_name}"))
            }
            MenuContext::Language => {
                let lang_name = config
                    .language
                    .as_deref()
                    .unwrap_or(crate::constants::DEFAULT_LANGUAGE);
                Some(format!("lang/{lang_name}"))
            }
            MenuContext::AsciiArt => {
                let art_name = config.ascii.as_deref().unwrap_or(DEFAULT_ASCII_ART_NAME);
                Some(format!("ascii/{art_name}"))
            }
            MenuContext::Cursor => {
                let cursor_style = config
                    .cursor_style
                    .as_deref()
                    .unwrap_or(crate::constants::DEFAULT_CURSOR_STYLE);
                Some(format!("cursor/{cursor_style}"))
            }
            MenuContext::PickerStyle => {
                let picker_style = config.resolve_picker_style();
                let picker_style_str = picker_style.as_str();
                Some(format!("picker/{picker_style_str}"))
            }
            MenuContext::Mode => Some(format!(
                "mode/{}",
                match config.current_mode_type() {
                    crate::config::ModeType::Time => "time",
                    crate::config::ModeType::Words => "words",
                }
            )),
            MenuContext::Time => {
                let presets = DEFAULT_TIME_DURATION_LIST;
                let current_val = config.time.unwrap_or(DEFAULT_TIME_MODE_DURATION as u64);
                if presets.contains(&(current_val as usize)) {
                    Some(format!("time/{current_val}"))
                } else {
                    Some("time/custom".to_string())
                }
            }
            MenuContext::Words => {
                let presets: Vec<usize> = vec![10, 25, 50, 100];
                let current_val = config
                    .word_count
                    .unwrap_or(crate::constants::DEFAULT_WORD_MODE_COUNT);
                if presets.contains(&current_val) {
                    Some(format!("words/{current_val}"))
                } else {
                    Some("words/custom".to_string())
                }
            }
            MenuContext::LineCount => {
                let visible_lines = config.visible_lines;
                Some(format!("lines/{visible_lines}"))
            }
            MenuContext::Results => {
                let results_style = config.resolve_results_style();
                let results_style_str = results_style.as_str();
                Some(format!("results/{results_style_str}"))
            }
            MenuContext::Root
            | MenuContext::About
            | MenuContext::Help
            | MenuContext::Options
            | MenuContext::Leaderboard => None,
        };

        if let Some(id) = target_id {
            menu._items
                .iter()
                .position(|item| item.id == id)
                .unwrap_or(0)
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_menu() -> MenuState {
        MenuState::new()
    }

    #[test]
    fn test_menu_navigation() {
        let mut menu = create_test_menu();
        let config = Config::default();
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Root), &config);
        assert!(menu.is_open());

        menu.handle_action(TermiAction::MenuNavigate(MenuNavAction::Down), &config);
        menu.handle_action(TermiAction::MenuNavigate(MenuNavAction::Down), &config);
        assert_eq!(menu.current_menu().unwrap().current_selection_index(), 2);

        menu.handle_action(TermiAction::MenuNavigate(MenuNavAction::Up), &config);
        assert_eq!(menu.current_menu().unwrap().current_selection_index(), 1);

        menu.handle_action(TermiAction::MenuNavigate(MenuNavAction::Down), &config); // idx 2
        menu.handle_action(TermiAction::MenuNavigate(MenuNavAction::Down), &config); // idx 3
        menu.handle_action(TermiAction::MenuNavigate(MenuNavAction::Down), &config); // idx 4
        menu.handle_action(TermiAction::MenuNavigate(MenuNavAction::Down), &config); // idx 5
        assert_eq!(menu.current_menu().unwrap().current_selection_index(), 5);
    }

    #[test]
    fn test_theme_picker() {
        // must set this manually as the theme sub-menu is disbabled if the
        // current environment doesn't have proper color support and without it
        // this test will fail in CI for example.
        env::set_var("COLORTERM", "truecolor");
        let mut menu = create_test_menu();
        let config = Config::default();
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Root), &config);
        assert!(menu.is_open());
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Theme), &config);
        assert!(menu.stack.len() == 2);
        assert!(menu.is_current_ctx(MenuContext::Theme));

        let action_result = menu.handle_action(TermiAction::MenuSelect, &config);
        assert!(matches!(action_result, Some(TermiAction::ChangeTheme(_))));
    }

    #[test]
    fn test_toggle_features() {
        let mut menu = create_test_menu();
        let mut config = Config::default();
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Root), &config);
        assert!(menu.is_open());
        config.use_punctuation = true;
        menu.sync_toggle_items(&config);
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Options), &config);

        if let Some(menu_ref) = menu.current_menu() {
            let current_items = menu_ref.items();
            let item = current_items
                .iter()
                .find(|i| i.label == "Use Punctuation")
                .unwrap();
            assert_eq!(item.is_active, Some(true));
        } else {
            panic!("We should have a menu opened Options by this point")
        }
    }

    #[test]
    fn test_search() {
        let mut menu = create_test_menu();
        let config = Config::default();
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Root), &config);

        assert!(!menu.is_searching());

        menu.handle_action(TermiAction::MenuOpen(MenuContext::Theme), &config);
        assert!(menu.is_current_ctx(MenuContext::Theme));

        assert!(!menu.is_searching());
        assert_ne!(menu.current_menu().unwrap().items().len(), 2);

        menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Start), &config);
        assert!(menu.is_searching());

        for c in "termitype".chars() {
            menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Input(c)), &config);
        }

        // NOTE: this could be a flaky test if we ever add more termitype themes.
        assert_eq!(menu.current_menu().unwrap().items().len(), 2);
    }
    #[test]
    fn test_staring_search_should_not_change_start_index() {
        let mut menu = create_test_menu();
        let config = Config::default();
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Root), &config);
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Theme), &config);
        menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Start), &config);
        assert!(menu.is_searching());

        for c in "termitype".chars() {
            menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Input(c)), &config);
        }
        menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Confirm), &config);

        assert!(!menu.is_searching());
        // now if we start searching for a theem again, it should not jump to the first item
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Root), &config);
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Theme), &config);
        menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Start), &config);
        assert_ne!(menu.current_menu().unwrap().current_selection_index(), 0);
    }

    #[test]
    fn test_closing_search_should_clear_query() {
        let mut menu = create_test_menu();
        let config = Config::default();
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Root), &config);
        menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Start), &config);
        assert!(menu.search_query.is_empty());
        let str: &str = "not_found";
        for char in str.chars() {
            menu.handle_action(
                TermiAction::MenuSearch(MenuSearchAction::Input(char)),
                &config,
            );
        }

        assert!(menu.is_searching);
        assert!(!menu.search_query.is_empty());
        assert_eq!(menu.search_query, str);

        menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Close), &config);

        assert!(!menu.is_searching());
        assert!(menu.search_query.is_empty());
    }

    #[test]
    fn test_closing_search_should_clear_filtered_items() {
        let mut menu = create_test_menu();
        let config = Config::default();
        menu.handle_action(TermiAction::MenuOpen(MenuContext::Root), &config);
        menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Start), &config);
        assert!(menu.search_query.is_empty());
        let str: &str = "not_found";
        for char in str.chars() {
            menu.handle_action(
                TermiAction::MenuSearch(MenuSearchAction::Input(char)),
                &config,
            );
        }

        menu.handle_action(TermiAction::MenuSearch(MenuSearchAction::Close), &config);
        assert!(!menu.current_menu().unwrap().items().is_empty());
    }
}
