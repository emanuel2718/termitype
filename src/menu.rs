use crate::{
    actions::{MenuContext, MenuNavAction, MenuSearchAction, PreviewType, TermiAction},
    config::{Config, ModeType},
    log_debug,
    modal::ModalContext,
    utils::fuzzy_match,
    version::VERSION,
};

/// Represents the resulting action of selecting a menu item
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemResult {
    TriggerAction(TermiAction),
    OpenSubMenu(MenuContext),
    ToggleState,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
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
            label: label.to_string(),
            result: MenuItemResult::TriggerAction(action),
            is_active: None,
            is_disabled: false,
            preview_type: None,
        }
    }

    pub fn toggle(id: &str, label: &str, is_active: bool) -> Self {
        Self {
            id: id.to_string(),
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

    pub fn navigate(&mut self, nav: MenuNavAction) {
        let items_count = self.size();
        log_debug!(
            "Calling navigate with nav action: {:?} and count: {}",
            nav,
            items_count
        );
        if items_count == 0 {
            return;
        }
        let curr = self.current_index;
        log_debug!("current_index = {}", curr);
        match nav {
            MenuNavAction::Up => self.current_index = curr.saturating_sub(1),
            MenuNavAction::Down => self.current_index = (curr + 1).min(items_count - 1),
            MenuNavAction::PageUp => {
                let scroll_amount = (self.size() / 2).max(1);
                self.current_index = curr.saturating_sub(scroll_amount).max(0)
            }
            MenuNavAction::PageDown => {
                let scroll_amount = (self.size() / 2).max(1);
                self.current_index = (curr + scroll_amount).min(items_count - 1)
            }
            MenuNavAction::Home => {
                self.current_index = 0;
                log_debug!("Calling home")
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

    pub fn selected_index(&self) -> usize {
        self.current_index
    }

    pub fn filter_items(&mut self, query: &str) -> Vec<MenuItem> {
        self.update_filtered_indices(query);
        self.items()
    }

    fn update_filtered_indices(&mut self, query: &str) {
        if query.is_empty() {
            self.filtered_indices = None;
            self.current_index = 0;
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
        let size = self.size();
        if size > 0 {
            self.current_index = self.current_index.min(size - 1)
        } else {
            self.current_index = 0
        }
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

    // NOTE: might need a more generic way to detect if the current menu is an `X` menu.
    // could get annoying when we get a lot of menus
    pub fn is_theme_menu(&self) -> bool {
        let curr_menu = self.current_menu();
        if let Some(menu) = curr_menu {
            match menu.ctx {
                MenuContext::Theme => true,
                _ => false,
            };
        }
        false
    }

    pub fn is_language_menu(&self) -> bool {
        let curr_menu = self.current_menu();
        if let Some(menu) = curr_menu {
            match menu.ctx {
                MenuContext::Language => true,
                _ => false,
            };
        }
        false
    }

    pub fn is_about_menu(&self) -> bool {
        let curr_menu = self.current_menu();
        if let Some(menu) = curr_menu {
            match menu.ctx {
                MenuContext::About => true,
                _ => false,
            };
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
                log_debug!("------> menu open: {:?}", ctx);
                // Before opening, if it's a direct request (not sub-menu), clear stack.
                // This logic might need refinement based on desired sub-menu behavior.
                if self.stack.is_empty() || self.stack.last().map_or(true, |m| m.ctx != ctx) {
                    // This check is to prevent re-pushing same menu if already on top from sub-menu logic
                }
                self.open(ctx, config);
                self.preview_selection()
            }
            // Close
            TermiAction::MenuClose => {
                self.stack.clear();
                self.clear_search();
                return Some(TermiAction::ClearPreview);
            }
            // Select
            TermiAction::MenuSelect => return self.execute_menu_action(config),
            // Search
            TermiAction::MenuSearch(search_action) => {
                self.handle_search_action(search_action);
                return self.preview_selection();
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
                    menu.navigate(nav_action);
                }
                return self.preview_selection();
            }
            _ => Some(TermiAction::NoOp),
        };
        None
    }

    fn handle_search_action(&mut self, action: MenuSearchAction) {
        match action {
            MenuSearchAction::Start => self.is_searching = true,
            MenuSearchAction::Close => self.is_searching = false,
            MenuSearchAction::Confirm => self.is_searching = false,
            MenuSearchAction::Clear => self.search_query.clear(),
            MenuSearchAction::Backspace => {
                self.search_query.pop();
            }
            MenuSearchAction::Input(c) if self.is_searching => self.search_query.push(c),
            _ => {}
        }

        if let Some(menu) = self.stack.last_mut() {
            if self.is_searching {
                menu.filter_items(&self.search_query);
            } else {
                menu.filter_items("");
            }
        }
    }

    // =============== EXECUTOR ===============
    fn execute_menu_action(&mut self, config: &Config) -> Option<TermiAction> {
        let item_result = self
            .stack
            .last()
            .and_then(|menu| menu.current_item())
            .filter(|item| !item.is_disabled)
            .map(|item| item.result.clone());

        if let Some(result) = item_result {
            match result {
                MenuItemResult::TriggerAction(action) => {
                    self.stack.clear();
                    self.clear_search();
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
                            "root/punctuation" => TermiAction::TogglePunctuation,
                            "root/numbers" => TermiAction::ToggleNumbers,
                            "root/symbols" => TermiAction::ToggleSymbols,
                            _ => TermiAction::NoOp,
                        };
                        return Some(act);
                    }
                }
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
        let menu = self.build_menu(ctx, config);
        self.stack.push(menu);
        self.clear_search();
    }

    fn close(&mut self) {
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
            if let Some(item) = menu._items.iter_mut().find(|i| i.id == "root/punctuation") {
                item.is_active = Some(config.use_punctuation);
            }
            if let Some(item) = menu._items.iter_mut().find(|i| i.id == "root/symbols") {
                item.is_active = Some(config.use_symbols);
            }
            if let Some(item) = menu._items.iter_mut().find(|i| i.id == "root/numbers") {
                item.is_active = Some(config.use_numbers);
            }
        }
    }

    fn build_menu(&self, ctx: MenuContext, config: &Config) -> Menu {
        match ctx {
            MenuContext::Root => self.build_root_menu(config),
            MenuContext::Theme => self.build_theme_menu(),
            MenuContext::Language => self.build_language_menu(),
            MenuContext::Cursor => self.build_cursor_menu(),
            MenuContext::Mode => self.build_mode_menu(),
            MenuContext::Time => self.build_time_menu(),
            MenuContext::Words => self.build_words_menu(),
            MenuContext::LineCount => self.build_lines_count_menu(),
            MenuContext::About => self.build_about_menu(),
        }
    }

    fn build_root_menu(&self, config: &Config) -> Menu {
        let items = vec![
            MenuItem::toggle("root/punctuation", "Punctuation", config.use_punctuation),
            MenuItem::toggle("root/numbers", "Numbers", config.use_numbers),
            MenuItem::toggle("root/symbols", "Symbols", config.use_symbols),
            MenuItem::sub_menu("root/mode", "Mode...", MenuContext::Mode),
            MenuItem::sub_menu("root/time", "Time...", MenuContext::Time),
            MenuItem::sub_menu("root/words", "Words...", MenuContext::Words),
            MenuItem::sub_menu("root/language", "Language...", MenuContext::Language),
            MenuItem::sub_menu("root/theme", "Theme...", MenuContext::Theme)
                .disabled(!config.term_has_color_support()),
            MenuItem::sub_menu("root/cursor", "Cursor...", MenuContext::Cursor),
            MenuItem::sub_menu("root/lines", "Visible Lines...", MenuContext::LineCount),
            MenuItem::sub_menu("root/about", "About...", MenuContext::About),
            MenuItem::action("root/quit", "Exit", TermiAction::Quit),
        ];
        Menu::new(MenuContext::Root, "Main Menu".to_string(), items)
    }

    // Chnage theme menu
    fn build_theme_menu(&self) -> Menu {
        let themes = crate::theme::available_themes();
        let items: Vec<MenuItem> = themes
            .iter()
            .map(|name| {
                MenuItem::action(
                    &format!("themes/{}", name),
                    name,
                    TermiAction::ChangeTheme(name.to_string()),
                )
                .with_preview(PreviewType::Theme(name.to_string()))
            })
            .collect();
        Menu::new(MenuContext::Theme, "Select Theme".to_string(), items)
    }

    // Chnage test language menu
    fn build_language_menu(&self) -> Menu {
        let languages = crate::builder::Builder::available_languages();
        let items = languages
            .iter()
            .map(|lang| {
                MenuItem::action(
                    &format!("lang/{}", lang),
                    lang,
                    TermiAction::ChangeLanguage(lang.to_string()),
                )
            })
            .collect();
        Menu::new(MenuContext::Language, "Select Language".to_string(), items)
    }

    // Chnage cursor style menu
    fn build_cursor_menu(&self) -> Menu {
        let styles = [
            "beam",
            "block",
            "underline",
            "blinking-beam",
            "blinking-block",
            "blinking-underline",
        ];
        let items = styles
            .iter()
            .map(|&style| {
                MenuItem::action(
                    &format!("cursor/{}", style),
                    style,
                    TermiAction::ChangeCursor(style.to_string()),
                )
                .with_preview(PreviewType::Cursor(style.to_string()))
            })
            .collect();
        Menu::new(
            MenuContext::Cursor,
            "Select Cursor Style".to_string(),
            items,
        )
    }

    // Change test mode menu
    fn build_mode_menu(&self) -> Menu {
        let items = vec![
            MenuItem::action(
                "mode/time",
                "Time",
                TermiAction::ChangeMode(ModeType::Time, None),
            ),
            MenuItem::action(
                "mode/words",
                "Words",
                TermiAction::ChangeMode(ModeType::Words, None),
            ),
        ];
        Menu::new(MenuContext::Mode, "Select Mode".to_string(), items)
    }

    // Change test duration menu
    fn build_time_menu(&self) -> Menu {
        let times = [15, 30, 60, 120];
        let mut items: Vec<MenuItem> = times
            .iter()
            .map(|&t| {
                MenuItem::action(
                    &format!("time/{}", t),
                    &t.to_string(),
                    TermiAction::ChangeTime(t as u64),
                )
            })
            .collect();
        items.push(MenuItem::action(
            "time/custom",
            "Custom...",
            TermiAction::ModalOpen(ModalContext::CustomTime),
        ));
        Menu::new(MenuContext::Time, "Select Time".to_string(), items)
    }

    // Change word count menu
    fn build_words_menu(&self) -> Menu {
        let counts = [10, 25, 50, 100];
        let mut items: Vec<MenuItem> = counts
            .iter()
            .map(|&c| {
                MenuItem::action(
                    &format!("words/{}", c),
                    &c.to_string(),
                    TermiAction::ChangeWordCount(c),
                )
            })
            .collect();
        items.push(MenuItem::action(
            "words/custom",
            "Custom...",
            TermiAction::ModalOpen(ModalContext::CustomWordCount),
        ));
        Menu::new(MenuContext::Words, "Select Word Count".to_string(), items)
    }
    // Visible Line count menu
    fn build_lines_count_menu(&self) -> Menu {
        let lines = [1, 2, 3, 4, 5];
        let items: Vec<MenuItem> = lines
            .iter()
            .map(|&line_count| {
                MenuItem::action(
                    &format!("lines/{}", line_count),
                    &line_count.to_string(),
                    TermiAction::ChangeVisibleLines(line_count),
                )
            })
            .collect();
        Menu::new(
            MenuContext::LineCount,
            "Select Visible Lines".to_string(),
            items,
        )
    }

    // About menu
    fn build_about_menu(&self) -> Menu {
        let items = vec![
            MenuItem::action("about/name", "Name: termitype", TermiAction::NoOp),
            MenuItem::action(
                "about/version",
                &format!("Version: {}", VERSION),
                TermiAction::NoOp,
            ),
            MenuItem::action(
                "about/desc",
                "Description: A typing game for the terminal.",
                TermiAction::NoOp,
            ),
            MenuItem::action(
                "about/source",
                "Source: http://github.com/emanuel2718/termitype",
                TermiAction::NoOp,
            ),
        ];
        Menu::new(MenuContext::About, "About Termitype".to_string(), items)
    }
}
