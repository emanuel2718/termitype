use std::str::FromStr;

use ratatui::style::Color;

use crate::config::Config;

#[derive(Debug)]
pub struct Theme {
    pub text: Color,
    pub background: Color,
    pub hint: Color,
    pub border: Color,
    pub highlight: Color,
    pub error: Color,
    pub success: Color,
    pub inactive: Color,
}

impl Theme {
    pub fn new(config: &Config) -> Self {
        Self::get_theme_from_config(config)
    }

    fn get_theme_from_config(_: &Config) -> Self {
        /* TODO
            1. check if `--theme` flag was given with a valid theme and check if it exists in `$XDG_CONFIG_HOME/termitype/themes/<theme>`
            2. check if [theme] is set in `$XDG_CONFIG_HOME/termitype/config`. Still thinking about doing toml or simple `.ini` like
            3. if none of the above is true select the default theme.
        */
        Self::default_theme()
    }

    /// Default theme is the tokyonight colorscheme for now
    fn default_theme() -> Self {
        Self {
            text: Color::from_str("#c0caf5").unwrap(),
            background: Color::from_str("#1a1b26").unwrap(),
            hint: Color::from_str("#565f89").unwrap(),
            border: Color::from_str("#414868").unwrap(),
            highlight: Color::from_str("#7aa2f7").unwrap(),
            error: Color::from_str("#f7768e").unwrap(),
            success: Color::from_str("#9ece6a").unwrap(),
            inactive: Color::from_str("#545c7e").unwrap(),
        }
    }

    pub fn change_theme(&mut self, new_theme: String) {
        println!("Chaning theme to {new_theme}")
    }
}
