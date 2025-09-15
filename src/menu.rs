use crate::{
    actions, builders::menu_builder, common::strings::fuzzy_match, config::Config, error::AppError,
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

#[derive(Clone, Debug)]
pub struct MenuItem {
    label: String,
    pub is_disabled: bool,
    pub action: MenuAction,
    pub shortcut: Option<char>,
    pub description: Option<String>,
}

impl MenuItem {
    pub fn new<S: Into<String>>(label: S, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            is_disabled: false,
            action,
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }

    pub fn shortcut(mut self, shortcut: char) -> Self {
        self.shortcut = Some(shortcut);
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

    pub fn nav(&mut self, motion: MenuMotion, ui_height: usize) {
        let len = self.items.len();
        if len == 0 {
            return;
        }

        let scroll = ui_height.saturating_sub(1) / 2;
        match motion {
            MenuMotion::Up => self.current_index = self.current_index.saturating_sub(1),
            MenuMotion::Down => self.current_index = (self.current_index + 1).min(len - 1),
            MenuMotion::PageUp => self.current_index = self.current_index.saturating_sub(scroll),
            MenuMotion::PageDown => self.current_index = (self.current_index + scroll).min(len - 1),
            MenuMotion::Home => self.current_index = 0,
            MenuMotion::End => self.current_index = len - 1,
            MenuMotion::Back => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct Menu {
    stack: Vec<MenuContent>,
    search_query: String,
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
            ui_height: 10,
        }
    }

    pub fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn open(&mut self, ctx: MenuContext, config: &Config) -> Result<(), AppError> {
        let menu = menu_builder::build_menu_from_context(ctx, config);
        self.stack.push(menu);
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), AppError> {
        self.stack.clear();
        Ok(())
    }

    pub fn back(&mut self) -> Result<(), AppError> {
        // TODO: handle clear previews
        self.stack.clear();
        Ok(())
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

    pub fn is_searching(&self) -> bool {
        !self.search_query.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;

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

        content.nav(MenuMotion::Down, 10);
        assert_eq!(content.current_index, 1);

        content.nav(MenuMotion::Up, 10);
        assert_eq!(content.current_index, 0);

        content.nav(MenuMotion::End, 10);
        assert_eq!(content.current_index, 4);

        content.nav(MenuMotion::Home, 10);
        assert_eq!(content.current_index, 0);

        content.nav(MenuMotion::PageDown, 3); // ui_height=3, scroll=1
        assert_eq!(content.current_index, 1);

        content.nav(MenuMotion::PageUp, 3);
        assert_eq!(content.current_index, 0);
    }

    #[test]
    fn test_menu_content_find_by_shortcut() {
        let items = vec![
            MenuItem::new("Item1", MenuAction::Action(actions::Action::Quit)).shortcut('A'),
            MenuItem::new("Item2", MenuAction::Action(actions::Action::Quit))
                .shortcut('B')
                .disabled(true),
            MenuItem::new("Item3", MenuAction::Action(actions::Action::Quit)).shortcut('C'),
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
        menu.search_query = "test".to_string();
        assert!(menu.is_searching());
    }
}
