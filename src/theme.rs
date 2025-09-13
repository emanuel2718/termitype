use crate::{config::Config, constants::DEFAULT_THEME, error::AppError};
use anyhow::Result;
use rand::{rng, Rng};
use ratatui::style::Color;
use std::{
    collections::HashMap,
    convert::Infallible,
    str::FromStr,
    sync::{Arc, RwLock},
};

const NUM_COLORS: usize = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeColor {
    Background = 0,
    Foreground,
    Muted,
    Accent,
    Info,
    Primary,
    Highlight,
    Success,
    Error,
    Warning,
    Cursor,
    CursorText,
    SelectionBg,
    SelectionFg,
}

impl ThemeColor {
    pub fn all() -> &'static [Self] {
        &[
            ThemeColor::Background,
            ThemeColor::Foreground,
            ThemeColor::Muted,
            ThemeColor::Accent,
            ThemeColor::Info,
            ThemeColor::Primary,
            ThemeColor::Highlight,
            ThemeColor::Success,
            ThemeColor::Error,
            ThemeColor::Warning,
            ThemeColor::Cursor,
            ThemeColor::CursorText,
            ThemeColor::SelectionBg,
            ThemeColor::SelectionFg,
        ]
    }

    fn map_to_palette_key(self) -> &'static str {
        match self {
            ThemeColor::Background => "background",
            ThemeColor::Foreground => "foreground",
            ThemeColor::Muted => "palette8",
            ThemeColor::Accent => "palette10",
            ThemeColor::Info => "palette4",
            ThemeColor::Primary => "palette5",
            ThemeColor::Highlight => "palette6",
            ThemeColor::Success => "palette2",
            ThemeColor::Error => "palette1",
            ThemeColor::Warning => "palette3",
            ThemeColor::Cursor => "cursor-color",
            ThemeColor::CursorText => "palette0",
            ThemeColor::SelectionBg => "selection-background",
            ThemeColor::SelectionFg => "selection-foreground",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum ColorSupport {
    #[default]
    /// Basic ANSI colors (8 colors)
    Basic = 4,
    /// Extended color palette (256 colors)
    Extended = 8,
    /// Full RGB/True Color support (16.7 million colors)
    TrueColor = 24,
}

impl ColorSupport {
    pub fn support_themes(self) -> bool {
        self >= ColorSupport::Extended
    }

    pub fn support_unicode(self) -> bool {
        self >= ColorSupport::Extended
    }

    pub fn detect_color_support() -> Self {
        // TODO: improve this
        if let Ok(ct) = std::env::var("COLORTERM") {
            if Self::is_truecolor_term(&ct) {
                return ColorSupport::TrueColor;
            }
        }
        ColorSupport::Basic
    }

    fn is_truecolor_term(colorterm: &str) -> bool {
        matches!(colorterm.to_lowercase().as_str(), "truecolor | 24bit")
    }
}

impl std::str::FromStr for ColorSupport {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic" => Ok(Self::Basic),
            "256" | "extended" => Ok(Self::Extended),
            "true" | "truecolor" => Ok(Self::TrueColor),
            _ => Err(format!("Invalid color support value: {s}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: Arc<str>,
    colors: [Color; NUM_COLORS],
}

impl Default for Theme {
    fn default() -> Self {
        Self::fallback()
    }
}

impl FromStr for Theme {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(theme_manager()
            .get_theme(s)
            .unwrap_or_else(|_| Theme::fallback()))
    }
}

impl Theme {
    pub fn get(&self, color: ThemeColor) -> Color {
        self.colors[color as usize]
    }

    pub fn from_colorscheme(name: impl Into<Arc<str>>, colorscheme: &str) -> Result<Self> {
        let color_map = Self::parse_colors(colorscheme)?;
        let colors = Self::build_colors(&color_map)?;
        Ok(Theme {
            id: name.into(),
            colors,
        })
        // TODO: implement this
    }

    fn parse_colors(scheme: &str) -> Result<HashMap<String, String>> {
        let mut color_map = HashMap::new();
        for line in scheme.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            /*
            Possible cases by shape:
                background = #000000
                palette = 0=#000000
            */

            if line.starts_with("palette =") {
                // case: palette = 0=#000000
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 3 {
                    let idx = parts[1].trim();
                    let color = parts[2].trim();
                    let color_value = if color.starts_with('#') {
                        color.to_string()
                    } else {
                        format!("#{}", color) // TODO: further validate is a valid hex
                    };

                    color_map.insert(format!("palette{}", idx), color_value);
                }
            } else if let Some((key, val)) = line.split_once('=') {
                // case: background = #000000
                let key = key.trim();
                let value = val.trim();
                let color_value = if value.starts_with('#') {
                    value.to_string()
                } else {
                    format!("#{}", value) // TODO: further validate is a valid hex
                };
                color_map.insert(key.to_string(), color_value);
            }
        }
        Ok(color_map)
    }

    fn build_colors(color_map: &HashMap<String, String>) -> Result<[Color; NUM_COLORS]> {
        let mut colors = [Color::Black; NUM_COLORS];
        for &theme_color in ThemeColor::all() {
            let key = theme_color.map_to_palette_key();
            let color_hex = color_map
                .get(key)
                .ok_or_else(|| anyhow::anyhow!("Missing required color: {}", key))?;
            let color = Color::from_str(color_hex.as_str())
                .map_err(|e| anyhow::anyhow!("Invalid color for {}: {}", key, e))?;
            colors[theme_color as usize] = color;
        }
        Ok(colors)
    }

    pub fn fallback() -> Self {
        Self {
            id: Arc::from("Fallback"),
            colors: [
                Color::Black,        // Background
                Color::White,        // Foreground
                Color::Gray,         // Muted
                Color::LightMagenta, // Accent
                Color::Cyan,         // Info
                Color::LightCyan,    // Primary
                Color::LightGreen,   // Highlight
                Color::Green,        // Success
                Color::Red,          // Error
                Color::LightYellow,  // Warning
                Color::White,        // Cursor
                Color::Black,        // CursorText
                Color::DarkGray,     // SelectionBg
                Color::White,        // SelectionFg
            ],
        }
    }

    pub fn bg(&self) -> Color {
        self.get(ThemeColor::Background)
    }
    pub fn fg(&self) -> Color {
        self.get(ThemeColor::Foreground)
    }
    pub fn muted(&self) -> Color {
        self.get(ThemeColor::Muted)
    }
    pub fn accent(&self) -> Color {
        self.get(ThemeColor::Accent)
    }
    pub fn info(&self) -> Color {
        self.get(ThemeColor::Info)
    }
    pub fn primary(&self) -> Color {
        self.get(ThemeColor::Primary)
    }
    pub fn highlight(&self) -> Color {
        self.get(ThemeColor::Highlight)
    }
    pub fn success(&self) -> Color {
        self.get(ThemeColor::Success)
    }
    pub fn error(&self) -> Color {
        self.get(ThemeColor::Error)
    }
    pub fn warning(&self) -> Color {
        self.get(ThemeColor::Warning)
    }
    pub fn cursor(&self) -> Color {
        self.get(ThemeColor::Cursor)
    }
    pub fn cursor_text(&self) -> Color {
        self.get(ThemeColor::CursorText)
    }
    pub fn selection_bg(&self) -> Color {
        self.get(ThemeColor::SelectionBg)
    }
    pub fn selection_fg(&self) -> Color {
        self.get(ThemeColor::SelectionFg)
    }

    pub fn border(&self) -> Color {
        self.muted()
    }
}

#[derive(Default)]
pub struct ThemeManager {
    themes: Arc<RwLock<HashMap<String, Theme>>>,
    current_theme: Arc<RwLock<Option<Theme>>>,
    preview_theme: Arc<RwLock<Option<Theme>>>,
    color_support: ColorSupport,
}

impl ThemeManager {
    pub fn new() -> Self {
        let color_support = ColorSupport::detect_color_support();
        Self {
            themes: Arc::new(RwLock::new(HashMap::new())),
            current_theme: Arc::new(RwLock::new(None)),
            preview_theme: Arc::new(RwLock::new(None)),
            color_support,
        }
    }

    pub fn init_from_config(&self, config: &Config) -> Result<()> {
        let theme = config.current_theme();
        let theme_name = theme.as_deref().unwrap_or(DEFAULT_THEME);
        self.set_as_current_theme(theme_name)?;
        Ok(())
    }

    pub fn load_theme(&self, name: &str) -> Result<()> {
        if !self.themes.read().unwrap().contains_key(name) {
            if let Some(scheme) = crate::assets::get_theme(name) {
                let theme = Theme::from_colorscheme(name, &scheme)?;
                self.themes.write().unwrap().insert(name.to_string(), theme);
            } else {
                return Err(anyhow::anyhow!("Theme '{name}' not found"));
            }
        }
        Ok(())
    }

    pub fn get_theme(&self, name: &str) -> Result<Theme> {
        // get with read lock first
        {
            let themes = self.themes.read().unwrap();
            if let Some(theme) = themes.get(name) {
                return Ok(theme.clone());
            }
        } // drop

        // try to load then
        if let Some(scheme) = crate::assets::get_theme(name) {
            let theme = Theme::from_colorscheme(name, &scheme)?;
            let mut themes = self.themes.write().unwrap();
            if let Some(existing) = themes.get(name) {
                return Ok(existing.clone());
            }
            themes.insert(name.to_string(), theme.clone());
            Ok(theme)
        } else {
            Ok(Theme::fallback())
        }
    }

    /// Gets the currently active theme. If there's a an active `preview_theme` it takes
    /// precedence over the `current_theme`.
    pub fn get_active_theme(&self) -> Option<Theme> {
        if let Some(preview) = self.preview_theme.read().unwrap().as_ref() {
            Some(preview.clone())
        } else {
            self.current_theme.read().unwrap().clone()
        }
    }

    pub fn set_as_current_theme(&self, name: &str) -> Result<()> {
        let theme = self.get_theme(name)?;
        *self.current_theme.write().unwrap() = Some(theme);
        Ok(())
    }

    pub fn set_as_preview_theme(&self, name: &str) -> Result<()> {
        let theme = self.get_theme(name)?;
        *self.preview_theme.write().unwrap() = Some(theme);
        Ok(())
    }

    pub fn confirm_preview_as_current_theme(&self) -> Result<()> {
        if let Some(preview) = self.preview_theme.write().unwrap().take() {
            *self.current_theme.write().unwrap() = Some(preview);
        }
        Ok(())
    }

    pub fn cancel_theme_preview(&self) {
        *self.preview_theme.write().unwrap() = None;
    }

    pub fn is_theme_loaded(&self, name: &str) -> bool {
        self.themes.read().unwrap().contains_key(name)
    }

    pub fn loaded_theme_count(&self) -> usize {
        self.themes.read().unwrap().len()
    }

    pub fn clear_cache(&self) {
        self.themes.write().unwrap().clear();
    }

    pub fn available_themes(&self) -> Vec<String> {
        crate::assets::list_themes()
    }

    pub fn color_support(&self) -> ColorSupport {
        self.color_support
    }

    pub fn use_random_theme(&self) -> Result<(), AppError> {
        let available = self.available_themes();
        if available.is_empty() {
            return Err(AppError::ThemesNotFound);
        }
        let mut rng = rng();
        let idx = rng.random_range(0..available.len());
        let name = &available[idx];
        self.set_as_current_theme(name)?;
        Ok(())
    }

    pub fn randomize_theme(&self) -> Result<()> {
        let mut rng = rng();
        let colors: [Color; NUM_COLORS] = ThemeColor::all()
            .iter()
            .map(|_| {
                let r = rng.random::<u8>();
                let g = rng.random::<u8>();
                let b = rng.random::<u8>();
                Color::Rgb(r, g, b)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let theme = Theme {
            id: Arc::from("Random"),
            colors,
        };
        *self.current_theme.write().unwrap() = Some(theme);
        Ok(())
    }
}

static THEME_MANAGER: once_cell::sync::Lazy<ThemeManager> =
    once_cell::sync::Lazy::new(ThemeManager::new);

pub fn theme_manager() -> &'static ThemeManager {
    &THEME_MANAGER
}

pub fn init_from_config(config: &Config) -> Result<()> {
    theme_manager().init_from_config(config)
}

pub fn current_theme() -> Theme {
    theme_manager()
        .get_active_theme()
        .unwrap_or_else(Theme::fallback)
}

pub fn set_as_preview_theme(name: &str) -> Result<()> {
    theme_manager().set_as_preview_theme(name)
}

pub fn set_as_current_theme(name: &str) -> Result<()> {
    theme_manager().set_as_current_theme(name)
}

pub fn confirm_preview_as_current_theme() -> Result<()> {
    theme_manager().confirm_preview_as_current_theme()
}

pub fn cancel_theme_preview() {
    theme_manager().cancel_theme_preview()
}

pub fn available_themes() -> Vec<String> {
    theme_manager().available_themes()
}

pub fn use_random_theme() -> Result<(), AppError> {
    theme_manager().use_random_theme()
}

pub fn randomize_theme() -> Result<()> {
    theme_manager().randomize_theme()
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLORSCHEME: &str = r#"
background = #000000
foreground = #ffffff
palette0 = #000000
palette1 = #ff0000
palette2 = #00ff00
palette3 = #ffff00
palette4 = #0000ff
palette5 = #ff00ff
palette6 = #00ffff
palette7 = #ffffff
palette8 = #808080
palette9 = #ff8080
palette10 = #80ff80
palette11 = #ffff80
palette12 = #8080ff
palette13 = #ff80ff
cursor-color = #ffffff
selection-background = #808080
selection-foreground = #ffffff
"#;

    #[test]
    fn test_all_theme_colors() {
        let all_colors = ThemeColor::all();
        assert_eq!(all_colors.len(), NUM_COLORS);
        assert_eq!(all_colors[0], ThemeColor::Background);
    }

    #[test]
    fn test_get_theme() {
        let theme = Theme::fallback();
        assert_eq!(theme.get(ThemeColor::Background), Color::Black);
    }

    #[test]
    fn test_parse_colorscheme() {
        let theme = Theme::from_colorscheme("test1", COLORSCHEME).unwrap();
        assert_eq!(theme.id.as_ref(), "test1");
        assert_eq!(
            theme.get(ThemeColor::Background),
            Color::from_str("#000000").unwrap()
        );
        assert_eq!(
            theme.get(ThemeColor::SelectionBg),
            Color::from_str("#808080").unwrap()
        );
    }

    #[test]
    fn test_parse_incomplete_colorscheme() {
        let content = "background = #000000\nforeground = #ffffff";
        assert!(Theme::from_colorscheme(content, "test").is_err());
    }
}
