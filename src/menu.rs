use crate::{actions, common::strings::fuzzy_match};

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
    pub disabled: bool,
    pub action: MenuAction,
    pub shortcut: Option<char>,
    pub description: Option<String>,
}

impl MenuItem {
    pub fn new<S: Into<String>>(label: S, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            disabled: false,
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
        self.disabled = disabled;
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

#[derive(Clone, Debug)]
pub struct MenuContent {
    pub title: String,
    pub ctx: MenuContext,
    items: Vec<MenuItem>,
    pub current_index: usize,
}

impl Default for MenuContent {
    fn default() -> Self {
        Self::new("Root", MenuContext::Root, Vec::new())
    }
}

impl MenuContent {
    pub fn new<S: Into<String>>(title: S, ctx: MenuContext, items: Vec<MenuItem>) -> Self {
        Self {
            title: title.into(),
            ctx,
            items,
            current_index: 0,
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
                .filter(|item| !item.disabled && fuzzy_match(&item.label.to_lowercase(), &query))
                .collect()
        }
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

    pub fn find_by_shortcut(&self, shortcut: char) -> Option<(usize, &MenuItem)> {
        self.items
            .iter()
            .enumerate()
            .find(|(_, item)| item.shortcut == Some(shortcut) && !item.disabled)
    }
}

#[derive(Clone, Debug)]
pub struct Menu {
    pub stack: Vec<MenuContent>,
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
