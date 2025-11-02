use crate::{
    actions::{self, Action},
    builders::menu_builder,
    common::strings::fuzzy_match,
    config::Config,
    error::AppError,
    theme,
};

#[derive(Clone, Debug, PartialEq)]
pub enum MenuContext {
    Root,
    Options,
    Themes,
    Time,
    Words,
    Language,
    Cursor,
    Ascii,
    VisibleLines,
    Leaderboard,
    About,
    CommandPalette,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuMotion {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Back,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuAction {
    Action(actions::Action),
    SubMenu(MenuContext),
    Info(String, String),
}

impl MenuAction {
    /// Checks if this menu action opens a submenu or not
    pub fn is_submenu(&self) -> bool {
        matches!(self, MenuAction::SubMenu(_))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    label: String,
    pub is_disabled: bool,
    pub has_preview: bool,
    pub close_on_select: bool,
    pub action: MenuAction,
    pub tag: Option<String>,
    pub shortcut: Option<char>,
    pub description: Option<String>,
}

impl MenuItem {
    pub fn new<S: Into<String>>(label: S, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            is_disabled: false,
            has_preview: false,
            close_on_select: false,
            action,
            tag: None,
            shortcut: None,
            description: None,
        }
    }
    // TODO: when doing the builder for this, ensure you only allow either action() or submenu()

    pub fn action<S: Into<String>>(label: S, action: actions::Action) -> Self {
        Self::new(label, MenuAction::Action(action))
    }

    pub fn submenu<S: Into<String>>(label: S, context: MenuContext) -> Self {
        Self::new(label, MenuAction::SubMenu(context))
    }

    pub fn info<S: Into<String>>(key: S, value: S) -> Self {
        let key = key.into();
        let value = value.into();
        Self::new(format!("{} {}", key, value), MenuAction::Info(key, value))
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }

    pub fn get_tag(&self) -> String {
        self.tag.clone().unwrap_or("".to_string())
    }

    pub fn get_shortcut(&self) -> char {
        self.shortcut.unwrap_or_default()
    }

    pub fn shortcut(mut self, shortcut: char) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn preivew(mut self) -> Self {
        self.has_preview = true;
        self
    }

    pub fn get_description(&self) -> String {
        self.description.as_ref().unwrap_or(&self.label).to_string()
    }

    pub fn get_label(&self) -> String {
        if let Some(shortcut) = self.shortcut {
            format!("{} [{}]", self.label, shortcut)
        } else {
            self.label.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuVisualizer {
    ThemeVisualizer,
    CursorVisualizer,
    AsciiVisualizer,
}

#[derive(Clone, Debug)]
pub struct MenuContent {
    items: Vec<MenuItem>,
    current_index: usize,
    pub title: String,
    pub ctx: MenuContext,
    pub scroll_offset: usize,
    pub visualizer: Option<MenuVisualizer>,
    pub is_informational: bool,
    pub is_cmd_palette: bool,
}

impl Default for MenuContent {
    fn default() -> Self {
        Self::new("Root", MenuContext::Root, Vec::new(), None, false)
    }
}

impl MenuContent {
    pub fn new<S: Into<String>>(
        title: S,
        ctx: MenuContext,
        items: Vec<MenuItem>,
        visualizer: Option<MenuVisualizer>,
        is_informational: bool,
    ) -> Self {
        Self {
            title: title.into(),
            ctx: ctx.clone(),
            items,
            current_index: 0,
            scroll_offset: 0,
            visualizer,
            is_informational,
            is_cmd_palette: matches!(ctx, MenuContext::CommandPalette),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn set_current_index(&mut self, idx: usize) {
        self.current_index = idx;
    }

    pub fn current_item(&self) -> Option<&MenuItem> {
        self.items.get(self.current_index)
    }

    pub fn items(&self, query: &str) -> Vec<&MenuItem> {
        if query.is_empty() {
            self.items.iter().collect()
        } else {
            let query = query.to_lowercase();
            self.items
                .iter()
                .filter(|item| {
                    if item.is_disabled {
                        return false;
                    }
                    let label_target = if self.is_cmd_palette {
                        item.get_description()
                    } else {
                        item.get_label()
                    };
                    let label_matches = fuzzy_match(&label_target.to_lowercase(), &query);
                    let tag_matches = fuzzy_match(&item.get_tag().to_lowercase(), &query);
                    // in the cmd palette, mathc the full display text (`tag: description`)
                    let full_display_matches = if self.is_cmd_palette {
                        let full_display = if let Some(tag) = &item.tag {
                            format!("{}: {}", tag, label_target)
                        } else {
                            label_target.clone()
                        };
                        fuzzy_match(&full_display.to_lowercase(), &query)
                    } else {
                        false
                    };
                    label_matches || tag_matches || full_display_matches
                })
                .collect()
        }
    }

    pub fn find_by_shortcut(&self, shortcut: char) -> Option<(usize, &MenuItem)> {
        self.items
            .iter()
            .enumerate()
            .find(|(_, item)| item.shortcut == Some(shortcut) && !item.is_disabled)
    }

    pub fn has_visualizer(&self) -> bool {
        self.visualizer.is_some()
    }

    pub fn nav(&mut self, motion: MenuMotion, ui_height: usize, query: Option<&str>) {
        let query = query.unwrap_or("");
        if query.is_empty() {
            if self.items.is_empty() {
                return;
            }

            let len = self.items.len();
            self.current_index =
                Self::calculate_new_index(motion, self.current_index, len, ui_height);

            Self::update_scroll_offset(self, self.current_index, ui_height);
        } else {
            // nav while searching
            let filtered_items: Vec<MenuItem> = self.items(query).into_iter().cloned().collect();
            if filtered_items.is_empty() {
                return;
            }

            let current_filtered_index = Self::get_current_filtered_index(self, &filtered_items);
            let new_filtered_index = Self::calculate_new_index(
                motion,
                current_filtered_index,
                filtered_items.len(),
                ui_height,
            );

            Self::update_menu_indices(self, &filtered_items, new_filtered_index);
            Self::update_scroll_offset(self, current_filtered_index, ui_height);
        }
    }

    fn get_current_filtered_index(menu: &MenuContent, filtered_items: &[MenuItem]) -> usize {
        filtered_items
            .iter()
            .position(|item| *item == menu.items[menu.current_index])
            .unwrap_or(0)
    }

    fn calculate_new_index(
        motion: MenuMotion,
        current_index: usize,
        len: usize,
        viewport_height: usize,
    ) -> usize {
        match motion {
            MenuMotion::Up => current_index.saturating_sub(1),
            MenuMotion::Down => (current_index + 1).min(len.saturating_sub(1)),
            MenuMotion::PageUp => {
                let scroll = viewport_height.saturating_sub(1) / 2;
                current_index.saturating_sub(scroll)
            }
            MenuMotion::PageDown => {
                let scroll = viewport_height.saturating_sub(1) / 2;
                (current_index + scroll).min(len.saturating_sub(1))
            }
            MenuMotion::Home => 0,
            MenuMotion::End => len.saturating_sub(1),
            MenuMotion::Back => current_index,
        }
    }

    fn update_menu_indices(
        menu: &mut MenuContent,
        filtered_items: &[MenuItem],
        new_filtered_index: usize,
    ) {
        if let Some(item) = filtered_items.get(new_filtered_index) {
            for (original_idx, original_item) in menu.items.iter().enumerate() {
                if *item == *original_item {
                    menu.set_current_index(original_idx);
                    break;
                }
            }
        }
    }

    pub fn update_scroll_offset(
        menu: &mut MenuContent,
        current_filtered_index: usize,
        viewport_height: usize,
    ) {
        if current_filtered_index < menu.scroll_offset {
            menu.scroll_offset = current_filtered_index;
        } else if current_filtered_index >= menu.scroll_offset + viewport_height {
            menu.scroll_offset = current_filtered_index - viewport_height + 1;
        }
    }

    pub fn reset_selection(&mut self, ui_height: usize) {
        self.current_index = 0;
        self.scroll_offset = 0;
        Self::update_scroll_offset(self, self.current_index, ui_height);
    }
}

#[derive(Clone, Debug)]
pub struct Menu {
    stack: Vec<MenuContent>,
    search_query: String,
    search_mode: bool,
    pub ui_height: usize,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            search_query: String::new(),
            search_mode: false,
            ui_height: 10,
        }
    }

    pub fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn open(&mut self, ctx: MenuContext, config: &Config) -> Result<(), AppError> {
        let is_cmd_palette = ctx == MenuContext::CommandPalette;
        let menu = menu_builder::build_menu_from_context(ctx, config);
        self.stack.push(menu);
        let ui_height = self.ui_height;
        if let Some(current_menu) = self.current_menu_mut() {
            if is_cmd_palette {
                current_menu.set_current_index(0);
            }
            MenuContent::update_scroll_offset(current_menu, current_menu.current_index, ui_height);
        }
        if is_cmd_palette {
            self.init_search();
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), AppError> {
        theme::cancel_theme_preview();
        self.stack.clear();
        Ok(())
    }

    pub fn back(&mut self) -> Result<(), AppError> {
        // TODO: handle clear previews
        if !self.stack.is_empty() {
            self.stack.pop();
        } else {
            self.stack.clear();
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.current_items().len() == 0
    }

    pub fn current_menu(&self) -> Option<&MenuContent> {
        self.stack.last()
    }

    pub fn current_menu_mut(&mut self) -> Option<&mut MenuContent> {
        self.stack.last_mut()
    }

    pub fn current_item(&self) -> Option<&MenuItem> {
        self.current_menu()?.current_item()
    }

    pub fn current_items(&self) -> Vec<&MenuItem> {
        self.current_menu()
            .map(|menu| menu.items(&self.search_query))
            .unwrap_or_default()
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_mode = false;
        let ui_height = self.ui_height;
        // in the command palette we want to reset to index 0, is cleaner that way
        if let Some(menu) = self.current_menu_mut() {
            if menu.is_cmd_palette {
                menu.reset_selection(ui_height);
            }
        }
    }

    pub fn init_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
    }

    pub fn exit_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        // in the command palette we want to reset to index 0, is cleaner that way
        if let Some(menu) = self.current_menu_mut() {
            if menu.is_cmd_palette {
                let _ = self.close();
                // menu.set_current_index(0);
                // menu.scroll_offset = 0;
            }
        }
    }

    pub fn is_searching(&self) -> bool {
        self.search_mode
    }

    pub fn has_search_query(&self) -> bool {
        !self.search_query().is_empty()
    }

    // NOTE: this is getting kinda hary just saying
    pub fn update_search(&mut self, query: String) {
        self.search_query = query.clone();
        let ui_height = self.ui_height;
        // when the search change try to keep current selection
        if let Some(menu) = self.current_menu_mut() {
            if menu.is_cmd_palette && query.is_empty() {
                menu.reset_selection(ui_height);
            } else {
                let items = menu.items(&query);
                if items.is_empty() {
                    menu.set_current_index(0);
                } else {
                    // does the current item still in the new results?
                    if let Some(current_item) = menu.current_item() {
                        if items.iter().any(|&item| {
                            item.label == current_item.label && item.action == current_item.action
                        }) {
                            // it is, keep it selected
                        } else {
                            // the current item is not in the new resutls, select the first rresult
                            if let Some(first_item) = items.first() {
                                for (original_idx, original_item) in menu.items.iter().enumerate() {
                                    if first_item.label == original_item.label
                                        && first_item.action == original_item.action
                                    {
                                        menu.set_current_index(original_idx);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                menu.scroll_offset = 0;
            }
        }
    }

    pub fn backspace_search(&mut self) {
        if !self.search_query.is_empty() {
            self.search_query.pop();
            self.update_search(self.search_query.clone());
        }
    }

    pub fn navigate(&mut self, motion: MenuMotion) {
        let viewport_height = self.ui_height;
        let query = if self.has_search_query() {
            Some(self.search_query.clone())
        } else {
            None
        };

        if let Some(menu) = self.current_menu_mut() {
            menu.nav(motion, viewport_height, query.as_deref());
        }
    }

    pub fn select(&mut self, config: &Config) -> Result<Option<Action>, AppError> {
        if let Some(item) = self.current_item().cloned() {
            if self.is_searching() {
                self.clear_search();
            }
            let action = item.action.clone();
            match action {
                MenuAction::Action(act) => {
                    let is_cmd_palette = self.stack.last().is_some_and(|m| m.is_cmd_palette);
                    if item.close_on_select || is_cmd_palette {
                        self.close()?;
                    }
                    return Ok(Some(act));
                }
                MenuAction::SubMenu(ctx) => self.open(ctx, config)?,
                MenuAction::Info(_, _) => {} // No action for info items
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        actions::Action,
        app::App,
        config::Setting,
        theme::{self},
    };

    #[test]
    fn test_open_menu() {
        let config = Config::default();
        let mut menu = Menu::new();
        assert!(!menu.is_open());
        menu.open(MenuContext::Root, &config).unwrap();
        assert!(menu.is_open());
    }

    #[test]
    fn test_close_menu() {
        let config = Config::default();
        let mut menu = Menu::new();
        menu.open(MenuContext::Root, &config).unwrap();
        assert!(menu.is_open());
        menu.close().unwrap();
        assert!(!menu.is_open());
    }

    #[test]
    fn test_menu_action_is_submenu() {
        let action = MenuAction::Action(actions::Action::Quit);
        assert!(!action.is_submenu());

        let submenu = MenuAction::SubMenu(MenuContext::Root);
        assert!(submenu.is_submenu());
    }

    #[test]
    fn test_menu_item_new() {
        let item = MenuItem::new("Test", MenuAction::Action(actions::Action::Quit));
        assert_eq!(item.label, "Test");
        assert!(!item.is_disabled);
        assert_eq!(item.shortcut, None);
        assert_eq!(item.description, None);
    }

    #[test]
    fn test_menu_item_submenu() {
        let item = MenuItem::submenu("Options", MenuContext::Options);
        assert_eq!(item.label, "Options");
        assert!(matches!(
            item.action,
            MenuAction::SubMenu(MenuContext::Options)
        ));
    }

    #[test]
    fn test_menu_item_label() {
        let item = MenuItem::new("Test", MenuAction::Action(actions::Action::Quit));
        assert_eq!(item.get_label(), "Test");

        let item_with_shortcut = item.shortcut('T');
        assert_eq!(item_with_shortcut.get_label(), "Test [T]");
    }

    #[test]
    fn test_menu_content_new() {
        let items = vec![MenuItem::new(
            "Item1",
            MenuAction::Action(actions::Action::Quit),
        )];
        let content = MenuContent::new("Title", MenuContext::Root, items, None, false);
        assert_eq!(content.title, "Title");
        assert_eq!(content.ctx, MenuContext::Root);
        assert_eq!(content.len(), 1);
        assert_eq!(content.current_index, 0);
        assert_eq!(content.scroll_offset, 0);
    }

    #[test]
    fn test_menu_content_len_and_is_empty() {
        let empty_content = MenuContent::new("Empty", MenuContext::Root, vec![], None, false);
        assert_eq!(empty_content.len(), 0);
        assert!(empty_content.is_empty());

        let items = vec![MenuItem::new(
            "Item1",
            MenuAction::Action(actions::Action::Quit),
        )];
        let content = MenuContent::new("Title", MenuContext::Root, items, None, false);
        assert_eq!(content.len(), 1);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_menu_content_current_item() {
        let items = vec![
            MenuItem::new("Item1", MenuAction::Action(actions::Action::Quit)),
            MenuItem::new("Item2", MenuAction::Action(actions::Action::Quit)),
        ];
        let mut content = MenuContent::new("Title", MenuContext::Root, items, None, false);
        assert_eq!(content.current_item().unwrap().label, "Item1");

        content.current_index = 1;
        assert_eq!(content.current_item().unwrap().label, "Item2");

        content.current_index = 2;
        assert!(content.current_item().is_none());
    }

    #[test]
    fn test_menu_content_nav() {
        let items = vec![
            MenuItem::new("1", MenuAction::Action(Action::NoOp)),
            MenuItem::new("2", MenuAction::Action(Action::NoOp)),
            MenuItem::new("3", MenuAction::Action(Action::NoOp)),
            MenuItem::new("4", MenuAction::Action(Action::NoOp)),
            MenuItem::new("5", MenuAction::Action(Action::NoOp)),
        ];
        let mut content = MenuContent::new("Title", MenuContext::Root, items, None, false);

        content.nav(MenuMotion::Down, 10, None);
        assert_eq!(content.current_index, 1);

        content.nav(MenuMotion::Up, 10, None);
        assert_eq!(content.current_index, 0);

        content.nav(MenuMotion::End, 10, None);
        assert_eq!(content.current_index, 4);

        content.nav(MenuMotion::Home, 10, None);
        assert_eq!(content.current_index, 0);

        content.nav(MenuMotion::PageDown, 3, None); // ui_height=3, scroll=1
        assert_eq!(content.current_index, 1);

        content.nav(MenuMotion::PageUp, 3, None);
        assert_eq!(content.current_index, 0);

        content.nav(MenuMotion::Up, 10, None);
        assert_eq!(content.current_index, 0); // no wrapping
    }

    #[test]
    fn test_menu_content_find_by_shortcut() {
        let items = vec![
            MenuItem::new("1", MenuAction::Action(actions::Action::Quit)).shortcut('A'),
            MenuItem::new("2", MenuAction::Action(actions::Action::Quit))
                .shortcut('B')
                .disabled(true),
            MenuItem::new("3", MenuAction::Action(actions::Action::Quit)).shortcut('C'),
        ];
        let content = MenuContent::new("Title", MenuContext::Root, items, None, false);

        let found = content.find_by_shortcut('A');
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, 0);

        let not_found = content.find_by_shortcut('B'); // disabled
        assert!(not_found.is_none());

        let not_found2 = content.find_by_shortcut('D');
        assert!(not_found2.is_none());
    }

    #[test]
    fn test_menu_back() {
        let config = Config::default();
        let mut menu = Menu::new();
        menu.open(MenuContext::Root, &config).unwrap();
        assert!(menu.is_open());
        menu.back().unwrap();
        assert!(!menu.is_open());
    }

    #[test]
    fn test_menu_current_menu() {
        let config = Config::default();
        let mut menu = Menu::new();
        assert!(menu.current_menu().is_none());
        menu.open(MenuContext::Root, &config).unwrap();
        assert!(menu.current_menu().is_some());
    }

    #[test]
    fn test_menu_current_item() {
        let config = Config::default();
        let mut menu = Menu::new();
        assert!(menu.current_item().is_none());
        menu.open(MenuContext::Root, &config).unwrap();
        assert!(menu.current_item().is_some()); // root must always have items
    }

    #[test]
    fn test_menu_current_items() {
        let config = Config::default();
        let mut menu = Menu::new();
        assert!(menu.current_items().is_empty());
        menu.open(MenuContext::Root, &config).unwrap();
        assert!(!menu.current_items().is_empty());
    }

    #[test]
    fn test_menu_search_query() {
        let mut menu = Menu::new();
        assert_eq!(menu.search_query(), "");
        menu.search_query = "test".to_string();
        assert_eq!(menu.search_query(), "test");
    }

    #[test]
    fn test_menu_is_searching() {
        let mut menu = Menu::new();
        assert!(!menu.is_searching());
        menu.init_search();
        menu.update_search("test".to_string());
        assert!(menu.is_searching());
        assert_eq!(menu.search_query(), "test".to_string());
    }

    #[test]
    fn test_menu_search_confirm_should_clear_search_query() {
        let mut menu = Menu::new();
        let config = Config::default();
        menu.open(MenuContext::Root, &config).unwrap();
        menu.init_search();
        menu.update_search("theme".to_string());
        assert_eq!(menu.search_query(), "theme");

        menu.select(&config).unwrap();
        assert_eq!(menu.search_query(), "");
    }

    #[test]
    fn test_menu_go_back() {
        let config = Config::default();
        let mut menu = Menu::new();
        assert!(menu.stack.is_empty());
        menu.open(MenuContext::Root, &config).unwrap();
        assert!(menu.stack.len() == 1);
        menu.select(&config).unwrap();
        assert!(menu.stack.len() == 2);
        menu.back().unwrap();
        assert!(menu.stack.len() == 1);
        menu.back().unwrap();
        assert!(menu.stack.is_empty());
    }

    #[test]
    fn test_menu_content_items_with_query() {
        let items = vec![
            MenuItem::new("termitype", MenuAction::Action(Action::NoOp)),
            MenuItem::new("test", MenuAction::Action(Action::NoOp)),
            MenuItem::new("hi", MenuAction::Action(Action::NoOp)).disabled(true),
        ];
        let content = MenuContent::new("Title", MenuContext::Root, items, None, false);

        let all_items = content.items("");
        assert_eq!(all_items.len(), 3);

        let filtered = content.items("t");
        assert_eq!(filtered.len(), 2); // termitype and test
        assert_eq!(filtered[0].label, "termitype");
        assert_eq!(filtered[1].label, "test");

        let no_match = content.items("xyz");
        assert!(no_match.is_empty());
    }

    #[test]
    fn test_menu_content_cmd_palette_full_display_matching() {
        let mut theme_item = MenuItem::new("Monkeytype", MenuAction::Action(Action::NoOp));
        theme_item.tag = Some("theme".to_string());
        theme_item.description = Some("Monkeytype".to_string());

        let mut option_item = MenuItem::new("Symbols", MenuAction::Action(Action::NoOp));
        option_item.tag = Some("option".to_string());
        option_item.description = Some("Enable Symbols".to_string());
        option_item.description = Some("Disable Symbols".to_string());

        let items = vec![theme_item, option_item];
        let content = MenuContent::new("CP", MenuContext::CommandPalette, items, None, true);

        // match tag
        let filtered_theme = content.items("th");
        assert_eq!(filtered_theme.len(), 1);

        // match description
        let filtered_monkey = content.items("Mnk");
        assert_eq!(filtered_monkey.len(), 1);

        // match full display `theme: Monkeytype`
        let filtered_combo = content.items("thMonk");
        assert_eq!(filtered_combo.len(), 1);
    }

    #[test]
    fn test_menu_content_nav_with_query() {
        let items = vec![
            MenuItem::new("termitype", MenuAction::Action(Action::NoOp)),
            MenuItem::new("test", MenuAction::Action(Action::NoOp)),
            MenuItem::new("hi", MenuAction::Action(Action::NoOp)),
        ];
        let mut content = MenuContent::new("Title", MenuContext::Root, items, None, false);

        content.nav(MenuMotion::Down, 10, Some("t"));
        // must only have `termitype` and `test` as items, in that order
        assert_eq!(content.current_index, 1); // move to `test`

        content.nav(MenuMotion::Down, 10, Some("r"));
        assert_eq!(content.current_index, 0); // termitype
    }

    #[test]
    fn test_menu_navigate() {
        let config = Config::default();
        let mut menu = Menu::new();
        menu.open(MenuContext::Root, &config).unwrap();
        menu.navigate(MenuMotion::Down);
        if let Some(current_menu) = menu.current_menu() {
            assert_eq!(current_menu.current_index(), 1);
        }
    }

    #[test]
    fn test_menu_item_preview() {
        let item = MenuItem::new("Test", MenuAction::Action(Action::NoOp)).preivew();
        assert!(item.has_preview);
    }

    #[test]
    fn test_menu_item_action_with_search() {
        let config = Config::default();
        let mut app = App::new(&config);
        let theme_name = "Termitype Dark";
        theme::set_as_current_theme("Fallback").unwrap();
        assert!(!theme::is_using_preview_theme());

        assert_eq!(
            theme::current_theme().id.to_string(),
            "Fallback".to_string()
        );

        app.handler
            .handle_menu_open(&mut app, MenuContext::Themes)
            .unwrap();
        assert!(app.menu.is_open());
        assert!(!app.menu.is_empty());
        app.handler.handle_menu_init_search(&mut app).unwrap();
        app.handler
            .handle_menu_update_search(&mut app, theme_name.to_string())
            .unwrap();
        assert!(!app.menu.is_empty());
        assert!(theme::is_using_preview_theme());

        let current_item = app.menu.current_item().unwrap();
        assert!(current_item.label == theme_name);

        let select_action = app.menu.select(&config).unwrap();
        assert_eq!(
            select_action,
            Some(Action::SetTheme(theme_name.to_string()))
        );
    }

    #[test]
    fn test_menu_search_no_matches_keeps_menu_open() {
        let config = Config::default();
        let mut menu = Menu::new();
        menu.open(MenuContext::Root, &config).unwrap();
        assert!(menu.is_open());
        assert!(!menu.current_items().is_empty());

        menu.init_search();
        menu.update_search("nonexistent".to_string());
        assert!(menu.is_open());
        assert!(menu.current_items().is_empty());
        assert!(menu.is_searching());
        assert_eq!(menu.search_query(), "nonexistent");
    }

    #[test]
    fn test_menu_exit_search_clears_query_and_mode() {
        let config = Config::default();
        let mut menu = Menu::new();
        menu.open(MenuContext::Root, &config).unwrap();

        menu.init_search();
        menu.update_search("test".to_string());
        assert!(menu.is_searching());
        assert_eq!(menu.search_query(), "test");

        menu.exit_search();
        assert!(!menu.is_searching());
        assert_eq!(menu.search_query(), "");
    }

    #[test]
    fn test_menu_backspace_while_searching() {
        let config = Config::default();
        let mut menu = Menu::new();
        menu.open(MenuContext::Root, &config).unwrap();
        menu.init_search();

        menu.update_search("t2".to_string());
        assert!(menu.is_searching());
        assert_eq!(menu.search_query(), "t2");

        menu.backspace_search();
        assert_eq!(menu.search_query(), "t");

        menu.backspace_search();
        menu.backspace_search();
        assert!(menu.is_searching()); // backspace should *not* take us out of search mode
        assert!(menu.search_query().is_empty());
    }

    #[test]
    fn test_keep_menu_open_when_selecting_item_without_close_on_select() {
        let config = Config::default();
        let mut menu = Menu::new();
        menu.open(MenuContext::Options, &config).unwrap();
        assert!(menu.is_open());
        assert!(!menu.current_items().is_empty());
        let current_item = menu.current_item().unwrap();

        assert!(!current_item.close_on_select);

        menu.select(&config).unwrap();

        assert!(menu.is_open());
    }

    #[test]
    fn test_close_after_selection_with_close_on_select() {
        let config = Config::default();
        let mut menu = Menu::new();
        menu.open(MenuContext::Themes, &config).unwrap();
        assert!(menu.is_open());
        assert!(!menu.current_items().is_empty());
        let current_item = menu.current_item().unwrap();

        assert!(current_item.close_on_select);

        menu.select(&config).unwrap();

        assert!(!menu.is_open());
    }

    #[test]
    fn test_close_after_cmd_selection_in_command_palette() {
        let config = Config::default();
        let mut menu = Menu::new();
        menu.open(MenuContext::CommandPalette, &config).unwrap();
        menu.update_search("Enable symbols".to_string());
        assert!(menu.is_open());
        assert_eq!(menu.current_items().len(), 1);
        let current_item = menu.current_item().unwrap();
        let target_action = MenuAction::Action(Action::Enable(Setting::Symbols));

        assert_eq!(current_item.action, target_action);
        assert!(!current_item.close_on_select);

        // on command palette we ALWAYS want to close the palette after selection no matter what
        menu.select(&config).unwrap();

        assert!(!menu.is_open());
    }
}
