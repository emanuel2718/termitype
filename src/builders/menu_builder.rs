use crate::actions::Action;
use crate::config::{Config, Setting};
use crate::menu::{MenuContent, MenuContext, MenuItem, MenuVisualizer};
use crate::theme;

pub struct MenuBuilder {
    title: String,
    ctx: MenuContext,
    items: Vec<MenuItem>,
    visualizer: Option<MenuVisualizer>,
}

impl Default for MenuBuilder {
    fn default() -> Self {
        Self::new("Menu", MenuContext::Root)
    }
}

impl MenuBuilder {
    pub fn new<S: Into<String>>(title: S, ctx: MenuContext) -> Self {
        Self {
            ctx,
            title: title.into(),
            items: Vec::new(),
            visualizer: None,
        }
    }

    pub fn build(self) -> MenuContent {
        MenuContent::new(self.title, self.ctx, self.items, self.visualizer)
    }

    pub fn action<S: Into<String>>(mut self, label: S, action: Action) -> Self {
        self.items.push(MenuItem::action(label, action));
        self
    }

    pub fn submenu<S: Into<String>>(mut self, label: S, ctx: MenuContext) -> Self {
        self.items.push(MenuItem::submenu(label, ctx));
        self
    }

    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = MenuItem>,
    {
        self.items.extend(items);
        self
    }

    pub fn shortcut(mut self, shortcut: char) -> Self {
        if let Some(item) = self.items.last_mut() {
            item.shortcut = Some(shortcut);
        }
        self
    }

    pub fn preivew(mut self) -> Self {
        if let Some(item) = self.items.last_mut() {
            item.has_preview = true;
        }
        self
    }

    pub fn close_on_select(mut self) -> Self {
        if let Some(item) = self.items.last_mut() {
            item.close_on_select = true;
        }
        self
    }

    pub fn description<S: Into<String>>(mut self, desc: S) -> Self {
        if let Some(item) = self.items.last_mut() {
            item.description = Some(desc.into());
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        if let Some(item) = self.items.last_mut() {
            item.is_disabled = disabled
        }
        self
    }

    pub fn add_visualizer(mut self, visualizer: MenuVisualizer) -> Self {
        self.visualizer = Some(visualizer);
        self
    }
}

pub fn build_menu_from_context(ctx: MenuContext, config: &Config) -> MenuContent {
    match ctx {
        MenuContext::Root => build_root_menu(),
        MenuContext::Options => build_options_menu(),
        MenuContext::Themes => build_themes_menu(config),
    }
}

fn build_root_menu() -> MenuContent {
    MenuBuilder::new("Main Menu", MenuContext::Root)
        .submenu("Options", MenuContext::Options)
        .shortcut('o')
        .description("Configure typing preferences")
        .submenu("Theme", MenuContext::Themes)
        .shortcut('t')
        .description("Available Themes")
        .build()
}

fn build_options_menu() -> MenuContent {
    MenuBuilder::new("Options", MenuContext::Options)
        .action("Use symbols", Action::Toggle(Setting::Symbols))
        .shortcut('s')
        .description("Include symbols in the generated test (@, #, etc)")
        .action("Use numbers", Action::Toggle(Setting::Numbers))
        .shortcut('n')
        .description("Include numbers in the generated test")
        .action("Use punctuation", Action::Toggle(Setting::Punctuation))
        .shortcut('p')
        .description("Include punctuation in the generated test (!, ?, etc)")
        .action("Show live WPM", Action::Toggle(Setting::LiveWPM))
        .shortcut('w')
        .description("Show the live word per minutes during the test")
        .build()
}

fn build_themes_menu(config: &Config) -> MenuContent {
    let themes = theme::available_themes();
    let mut builder = MenuBuilder::new("Select Theme", MenuContext::Themes);
    for name in &themes {
        builder = builder
            .action(name, Action::ChangeTheme(name.clone()))
            .preivew()
            .add_visualizer(MenuVisualizer::ThemeVisualizer)
            .close_on_select();
    }

    let mut menu = builder.build();
    if let Some(current_theme_name) = config.current_theme() {
        if let Some(idx) = themes.iter().position(|name| name == &current_theme_name) {
            menu.current_index = idx;
        }
    }
    menu
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::{MenuAction, MenuVisualizer};

    #[test]
    fn test_build_from_context() {
        let ctx = MenuContext::Root;
        let config = Config::default();
        let content_from_ctx = build_menu_from_context(ctx, &config);
        let root_content = build_root_menu();
        assert_eq!(content_from_ctx.title, root_content.title);
        assert_eq!(content_from_ctx.ctx, root_content.ctx);
        assert_eq!(content_from_ctx.current_index, root_content.current_index);
    }

    #[test]
    fn test_new_builder() {
        let title = "termitype";
        let builder = MenuBuilder::new(title, MenuContext::Root);
        assert_eq!(builder.title, title);
        assert_eq!(builder.ctx, MenuContext::Root);
        assert!(builder.items.is_empty());
    }

    #[test]
    fn test_default_builder() {
        let builder = MenuBuilder::default();
        assert_eq!(builder.title, "Menu");
        assert_eq!(builder.ctx, MenuContext::Root);
        assert!(builder.items.is_empty());
    }

    #[test]
    fn test_builder_item_action() {
        let builder = MenuBuilder::default().action("Quit", Action::Quit);
        let item = &builder.items[0];
        assert_eq!(item.label(), "Quit");
        assert_eq!(item.action, MenuAction::Action(Action::Quit));
    }

    #[test]
    fn test_builder_item_shortcut() {
        let builder = MenuBuilder::default()
            .action("Quit", Action::Quit)
            .shortcut('q');
        let item = &builder.items[0];
        assert_eq!(item.shortcut, Some('q'))
    }

    #[test]
    fn test_builder_item_disabled() {
        let builder = MenuBuilder::default()
            .action("Quit", Action::Quit)
            .disabled(true);
        let item = &builder.items[0];
        assert!(item.is_disabled);
    }

    #[test]
    fn test_builder_menu_visualizer() {
        let builder = MenuBuilder::default().add_visualizer(MenuVisualizer::ThemeVisualizer);
        let menu = builder.build();
        assert!(menu.has_visualizer());
        assert_eq!(menu.visualizer, Some(MenuVisualizer::ThemeVisualizer))
    }
}
