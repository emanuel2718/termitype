use crate::actions::Action;
use crate::config::{Config, Setting};
use crate::menu::{MenuContent, MenuContext, MenuItem, MenuVisualizer};
use crate::modal::ModalContext;
use crate::theme;

pub struct MenuBuilder {
    title: String,
    ctx: MenuContext,
    items: Vec<MenuItem>,
    visualizer: Option<MenuVisualizer>,
    is_informational: bool,
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
            is_informational: false,
        }
    }

    pub fn build(self) -> MenuContent {
        MenuContent::new(
            self.title,
            self.ctx,
            self.items,
            self.visualizer,
            self.is_informational,
        )
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

    pub fn informational(mut self) -> Self {
        self.is_informational = true;
        self
    }

    pub fn info<S: Into<String>>(mut self, key: S, value: S) -> Self {
        self.items.push(MenuItem::info(key, value));
        self
    }
}

pub fn build_menu_from_context(ctx: MenuContext, config: &Config) -> MenuContent {
    match ctx {
        MenuContext::Root => build_root_menu(),
        MenuContext::Options => build_options_menu(),
        MenuContext::Themes => build_themes_menu(config),
        MenuContext::Time => build_time_menu(),
        MenuContext::Words => build_words_menu(),
        MenuContext::Language => build_language_menu(config),
        MenuContext::Cursor => build_cursor_menu(config),
        MenuContext::VisibleLines => build_visible_lines_menu(),
        MenuContext::About => build_about_menu(),
    }
}

#[rustfmt::skip]
fn build_root_menu() -> MenuContent {
    MenuBuilder::new("Main Menu", MenuContext::Root)
        .submenu("Time", MenuContext::Time) .shortcut('t') .description("Set test duration")
        .submenu("Words", MenuContext::Words) .shortcut('w') .description("Set word count")
        .submenu("Language", MenuContext::Language) .shortcut('l') .description("Select language")
        .submenu("Options", MenuContext::Options) .shortcut('o') .description("Configure typing preferences")
        .submenu("Theme", MenuContext::Themes) .shortcut('T') .description("Available Themes")
        .submenu("Lines", MenuContext::VisibleLines) .shortcut('L') .description("Visible Lines")
        .submenu("Cursor", MenuContext::Cursor) .shortcut('c') .description("Available Cursors")
        .submenu("About", MenuContext::About) .shortcut('a') .description("About termitype")
        .action("Exit", Action::ModalOpen(ModalContext::ExitConfirmation)) .shortcut('Q')
        .build()
}

#[rustfmt::skip]
fn build_options_menu() -> MenuContent {
    MenuBuilder::new("Options", MenuContext::Options)
        .action("Use symbols", Action::Toggle(Setting::Symbols)) .shortcut('s') .description("Include symbols in the generated test (@, #, etc)")
        .action("Use numbers", Action::Toggle(Setting::Numbers)) .shortcut('n') .description("Include numbers in the generated test")
        .action("Use punctuation", Action::Toggle(Setting::Punctuation)) .shortcut('p') .description("Include punctuation in the generated test (!, ?, etc)")
        .action("Show live WPM", Action::Toggle(Setting::LiveWPM)) .shortcut('w') .description("Show the live word per minutes during the test")
        .build()
}

fn build_themes_menu(config: &Config) -> MenuContent {
    let themes = theme::available_themes();
    let mut builder = MenuBuilder::new("Select Theme", MenuContext::Themes);
    for name in &themes {
        builder = builder
            .action(name, Action::SetTheme(name.clone()))
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

fn build_time_menu() -> MenuContent {
    // NOTE(ema):  for some reason I prefer the manual version of this (i.e `build_words_menu`)
    const DEFAULT_TIME_DURATION_LIST: [u16; 4] = [15, 30, 60, 120];
    let mut builder = MenuBuilder::new("Select Time", MenuContext::Time);
    for &duration in &DEFAULT_TIME_DURATION_LIST {
        builder = builder
            .action(duration.to_string(), Action::SetTime(duration))
            .close_on_select();
    }

    builder = builder
        .action("Custom", Action::ModalOpen(ModalContext::CustomTime))
        .shortcut('c');

    builder.build()
}

#[rustfmt::skip]
fn build_words_menu() -> MenuContent {
    MenuBuilder::new("Select Words", MenuContext::Words)
        .action("10", Action::SetWords(10)) .close_on_select()
        .action("25", Action::SetWords(25)) .close_on_select()
        .action("50", Action::SetWords(50)) .close_on_select()
        .action("100", Action::SetWords(100)) .close_on_select()
        .action("Custom", Action::ModalOpen(ModalContext::CustomWordCount)) .shortcut('c')
        .build()
}

fn build_language_menu(config: &Config) -> MenuContent {
    use crate::builders::lexicon_builder::LexiconBuilder;
    let languages = LexiconBuilder::available_languages();
    let mut builder = MenuBuilder::new("Select Language", MenuContext::Language);
    for lang in languages {
        builder = builder
            .action(lang.clone(), Action::SetLanguage(lang.clone()))
            .close_on_select();
    }
    let mut menu = builder.build();

    if let Some(idx) = languages
        .iter()
        .position(|lang| lang.clone() == config.current_language())
    {
        menu.current_index = idx
    }

    menu
}

fn build_cursor_menu(config: &Config) -> MenuContent {
    use crate::variants::CursorVariant;
    let variants = CursorVariant::all();
    let mut builder = MenuBuilder::new("Select Cursor", MenuContext::Cursor)
        .add_visualizer(MenuVisualizer::CursorVisualizer);

    for &variant in variants {
        builder = builder
            .action(variant.label(), Action::SetCursorVariant(variant))
            .preivew()
            .close_on_select();
    }

    let mut menu = builder.build();

    let current_variant = config.current_cursor_variant();
    if let Some(idx) = variants.iter().position(|&v| v == current_variant) {
        menu.current_index = idx
    }

    menu
}

#[rustfmt::skip]
fn build_visible_lines_menu() -> MenuContent {
    MenuBuilder::new("Select Line Count", MenuContext::VisibleLines)
        .action("1", Action::SetLineCount(1)) .shortcut('1') .close_on_select()
        .action("2", Action::SetLineCount(2)) .shortcut('2') .close_on_select()
        .action("3", Action::SetLineCount(3)) .shortcut('3') .close_on_select()
        .action("4", Action::SetLineCount(4)) .shortcut('4') .close_on_select()
        .action("5", Action::SetLineCount(5)) .shortcut('5') .close_on_select()
        .action("Custom", Action::ModalOpen(ModalContext::CustomLineCount)) .shortcut('c')
        .build()
}

fn build_about_menu() -> MenuContent {
    MenuBuilder::new("About", MenuContext::About)
        .informational()
        .info("Name", "termitype")
        .info("Version", env!("CARGO_PKG_VERSION"))
        .info("License", env!("CARGO_PKG_LICENSE"))
        .info("Description", "TUI typing game")
        .info("Source", "https://github.com/emanuel2718/termitype")
        .build()
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
