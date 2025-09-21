use crate::{
    actions::{self, Action},
    builders::menu_builder,
    common::strings::fuzzy_match,
    config::Config,
    error::AppError,
};

#[derive(Clone, Debug, PartialEq)]
pub enum MenuContext {
    Root,
    Options,
    Themes,
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
    pub action: MenuAction,
    pub shortcut: Option<char>,
    pub description: Option<String>,
    pub close_on_select: bool,
}

impl MenuItem {
    pub fn new<S: Into<String>>(label: S, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            is_disabled: false,
            has_preview: false,
            action,
            shortcut: None,
            description: None,
            close_on_select: false,
        }
    }
    // TODO: when doing the builder for this, ensure you only allow either action() or submenu()

    pub fn action<S: Into<String>>(label: S, action: actions::Action) -> Self {
        Self::new(label, MenuAction::Action(action))
    }

    pub fn submenu<S: Into<String>>(label: S, context: MenuContext) -> Self {
        Self::new(label, MenuAction::SubMenu(context))
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }

    pub fn shortcut(mut self, shortcut: char) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn preivew(mut self) -> Self {
        self.has_preview = true;
        self
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn label(&self) -> String {
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
}

#[derive(Clone, Debug)]
pub struct MenuContent {
    items: Vec<MenuItem>,
    pub title: String,
    pub ctx: MenuContext,
    pub current_index: usize,
    pub scroll_offset: usize,
    pub visualizer: Option<MenuVisualizer>,
}

impl Default for MenuContent {
    fn default() -> Self {
        Self::new("Root", MenuContext::Root, Vec::new(), None)
    }
}

impl MenuContent {
    pub fn new<S: Into<String>>(
        title: S,
        ctx: MenuContext,
        items: Vec<MenuItem>,
        visualizer: Option<MenuVisualizer>,
    ) -> Self {
        Self {
            title: title.into(),
            ctx,
            items,
            current_index: 0,
            scroll_offset: 0,
            visualizer,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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
                .filter(|item| !item.is_disabled && fuzzy_match(&item.label.to_lowercase(), &query))
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
            MenuMotion::Up => (current_index + len - 1) % len,
            MenuMotion::Down => (current_index + 1) % len,
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
                    menu.current_index = original_idx;
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
        let menu = menu_builder::build_menu_from_context(ctx, config);
        self.stack.push(menu);
        let ui_height = self.ui_height;
        if let Some(current_menu) = self.current_menu_mut() {
            MenuContent::update_scroll_offset(current_menu, current_menu.current_index, ui_height);
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), AppError> {
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
    }

    pub fn init_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
    }

    pub fn exit_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
    }

    pub fn is_searching(&self) -> bool {
        self.search_mode
    }

    pub fn has_search_query(&self) -> bool {
        !self.search_query().is_empty()
    }

    pub fn update_search(&mut self, query: String) {
        self.search_query = query.clone();
        // when the search change try to keep current selection selected
        if let Some(menu) = self.current_menu_mut() {
            let items = menu.items(&query);
            if items.is_empty() {
                menu.current_index = 0;
            } else {
                // does the current item still in the new results?
                if let Some(current_item) = menu.current_item() {
                    if items.iter().any(|&item| {
                        item.label() == current_item.label() && item.action == current_item.action
                    }) {
                        // it is, keep it selected
                    } else {
                        // the current item is not in the new resutls, select the first rresult
                        if let Some(first_item) = items.first() {
                            for (original_idx, original_item) in menu.items.iter().enumerate() {
                                if first_item.label() == original_item.label()
                                    && first_item.action == original_item.action
                                {
                                    menu.current_index = original_idx;
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
                    if item.close_on_select {
                        self.close()?;
                    }
                    return Ok(Some(act));
                }
                MenuAction::SubMenu(ctx) => self.open(ctx, config)?,
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
    fn test_menu_item_description() {
        let item = MenuItem::new("Test", MenuAction::Action(actions::Action::Quit))
            .description("A test item");
        assert_eq!(item.description, Some("A test item".to_string()));
    }

    #[test]
    fn test_menu_item_label() {
        let item = MenuItem::new("Test", MenuAction::Action(actions::Action::Quit));
        assert_eq!(item.label(), "Test");

        let item_with_shortcut = item.shortcut('T');
        assert_eq!(item_with_shortcut.label(), "Test [T]");
    }

    #[test]
    fn test_menu_content_new() {
        let items = vec![MenuItem::new(
            "Item1",
            MenuAction::Action(actions::Action::Quit),
        )];
        let content = MenuContent::new("Title", MenuContext::Root, items, None);
        assert_eq!(content.title, "Title");
        assert_eq!(content.ctx, MenuContext::Root);
        assert_eq!(content.len(), 1);
        assert_eq!(content.current_index, 0);
        assert_eq!(content.scroll_offset, 0);
    }

    #[test]
    fn test_menu_content_len_and_is_empty() {
        let empty_content = MenuContent::new("Empty", MenuContext::Root, vec![], None);
        assert_eq!(empty_content.len(), 0);
        assert!(empty_content.is_empty());

        let items = vec![MenuItem::new(
            "Item1",
            MenuAction::Action(actions::Action::Quit),
        )];
        let content = MenuContent::new("Title", MenuContext::Root, items, None);
        assert_eq!(content.len(), 1);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_menu_content_current_item() {
        let items = vec![
            MenuItem::new("Item1", MenuAction::Action(actions::Action::Quit)),
            MenuItem::new("Item2", MenuAction::Action(actions::Action::Quit)),
        ];
        let mut content = MenuContent::new("Title", MenuContext::Root, items, None);
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
        let mut content = MenuContent::new("Title", MenuContext::Root, items, None);

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
        assert_eq!(content.current_index, 4); // we at the end

        // wrap back up
        content.nav(MenuMotion::Down, 10, None);
        assert_eq!(content.current_index, 0);
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
        let content = MenuContent::new("Title", MenuContext::Root, items, None);

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
        let content = MenuContent::new("Title", MenuContext::Root, items, None);

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
    fn test_menu_content_nav_with_query() {
        let items = vec![
            MenuItem::new("termitype", MenuAction::Action(Action::NoOp)),
            MenuItem::new("test", MenuAction::Action(Action::NoOp)),
            MenuItem::new("hi", MenuAction::Action(Action::NoOp)),
        ];
        let mut content = MenuContent::new("Title", MenuContext::Root, items, None);

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
            assert_eq!(current_menu.current_index, 1);
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

        app.handle_menu_open(MenuContext::Themes).unwrap();
        assert!(app.menu.is_open());
        assert!(!app.menu.is_empty());
        app.handle_menu_init_search().unwrap();
        app.handle_menu_update_search(theme_name.to_string())
            .unwrap();
        assert!(!app.menu.is_empty());
        assert!(theme::is_using_preview_theme());

        let current_item = app.menu.current_item().unwrap();
        assert!(current_item.label() == theme_name);

        let select_action = app.menu.select(&config).unwrap();
        assert_eq!(
            select_action,
            Some(Action::ChangeTheme(theme_name.to_string()))
        );
    }
}
