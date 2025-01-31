use crate::termi::Termi;

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem {
    Restart,
    TogglePunctuation,
    ToggleNumbers,
    ToggleSymbols,
    SwitchMode,
    ChangeTheme,
    Quit,
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub items: Vec<MenuItem>,
    pub selected: usize,
    pub visible: bool,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            items: vec![
                MenuItem::Restart,
                MenuItem::TogglePunctuation,
                MenuItem::ToggleNumbers,
                MenuItem::ToggleSymbols,
                MenuItem::SwitchMode,
                MenuItem::ChangeTheme,
                MenuItem::Quit,
            ],
            selected: 0,
            visible: false,
        }
    }
}

impl Menu {
    pub fn toggle(&mut self) {
        self.visible = !self.visible
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % self.items.len()
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.checked_sub(1).unwrap_or(self.items.len() - 1)
    }

    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected)
    }

    pub fn get_display_text(item: &MenuItem) -> String {
        match item {
            MenuItem::Restart => "restart".into(),
            MenuItem::TogglePunctuation => "toggle punctuation".into(),
            MenuItem::ToggleNumbers => "toggle numbers".into(),
            MenuItem::ToggleSymbols => "toggle symbols".into(),
            MenuItem::SwitchMode => "switch mode".into(),
            MenuItem::ChangeTheme => "change theme".into(),
            MenuItem::Quit => "exit".into(),
        }
    }

    pub fn is_toggleable(&self, item: &MenuItem) -> bool {
        matches!(
            item,
            MenuItem::TogglePunctuation | MenuItem::ToggleNumbers | MenuItem::ToggleSymbols
        )
    }

    pub fn is_toggle_active(&self, item: &MenuItem, termi: &Termi) -> bool {
        match item {
            MenuItem::TogglePunctuation => termi.config.use_punctuation,
            MenuItem::ToggleNumbers => termi.config.use_numbers,
            MenuItem::ToggleSymbols => termi.config.use_symbols,
            _ => false,
        }
    }
}
