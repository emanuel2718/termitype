use std::{collections::HashMap, fs, path::PathBuf, str::FromStr, sync::OnceLock};

use ratatui::style::Color;

use crate::{config::Config, constants::DEFAULT_THEME};

#[derive(Debug)]
pub struct ThemeLoader {
    themes: HashMap<String, Theme>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub identifier: String,
    colors: [Color; 14],
    pub color_support: ColorSupport,
}

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum ColorIndex {
    Background = 0,
    Foreground,
    Cursor,
    CursorText,
    SelectionBg,
    SelectionFg,
    Border,
    Error,
    Success,
    Warning,
    Info,
    Accent,
    Highlight,
    Muted,
}

/// Represents the terminal's color support capabilities
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ColorSupport {
    /// Basic ANSI colors (8 colors)
    Basic = 4,
    /// Extended color palette (256 colors)
    Extended = 8,
    /// Full RGB/True Color support (16.7 million colors)
    TrueColor = 24,
}

impl ColorSupport {
    pub fn supports_themes(self) -> bool {
        self >= ColorSupport::Extended
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

impl Theme {
    pub fn new(config: &Config) -> Self {
        let color_support = config
            .color_mode
            .as_deref()
            .and_then(|s| ColorSupport::from_str(s).ok())
            .unwrap_or_else(Self::detect_color_support);

        // No theme support. Get a better terminal m8
        if !color_support.supports_themes() {
            return Self::fallback_theme_with_support(color_support);
        }
        let mut loader = ThemeLoader::init();
        let theme_name = config.theme.as_str();
        let mut theme = loader.get_theme(theme_name).unwrap_or_else(|_| {
            loader
                .get_theme(DEFAULT_THEME)
                .expect("Default theme must exist")
        });
        theme.color_support = color_support;
        theme
    }
    /// Detects the terminal's color support level
    fn detect_color_support() -> ColorSupport {
        if let Ok(colors) = std::env::var("COLORTERM") {
            match colors.as_str() {
                "truecolor" | "24bit" => ColorSupport::TrueColor,
                "256color" => ColorSupport::Extended,
                _ => ColorSupport::Basic,
            }
        } else {
            ColorSupport::Basic
        }
    }

    fn fallback_theme() -> Self {
        // TODO: must notice the user somehow that we are defaulting to "default" theme
        // if Self::detect_color_support() > ColorSupport::Extended {
        //     return Self::default();
        // }
        Self {
            identifier: "Default".to_string(),
            colors: [
                Color::Black,     // Background = 0,
                Color::White,     // Foreground,
                Color::LightCyan, // Cursor,
                Color::Black,     // CursorText,
                Color::Cyan,      // SelectionBg,
                Color::Black,     // SelectionFg,
                Color::LightBlue, // Border,
                Color::Red,       // Error,
                Color::Green,     // Success,
                Color::Yellow,    // Warning,
                Color::LightCyan, // Info,
                Color::Magenta,   // Accent,
                Color::Cyan,      // Highlight,
                Color::Gray,      // Muted,
            ],
            color_support: ColorSupport::Basic,
        }
    }

    fn fallback_theme_with_support(color_support: ColorSupport) -> Self {
        let mut theme = Self::fallback_theme();
        theme.color_support = color_support;
        theme
    }

    #[allow(clippy::field_reassign_with_default)]
    pub fn from_name(name: &str) -> Self {
        let mut config = Config::default();
        config.theme = name.to_string();
        Self::new(&config)
    }

    // ************** COLORS_FN **************
    pub fn background(&self) -> Color {
        self.colors[ColorIndex::Background as usize]
    }

    pub fn foreground(&self) -> Color {
        self.colors[ColorIndex::Foreground as usize]
    }

    pub fn cursor(&self) -> Color {
        self.colors[ColorIndex::Cursor as usize]
    }

    pub fn cursor_text(&self) -> Color {
        self.colors[ColorIndex::CursorText as usize]
    }

    pub fn selection_bg(&self) -> Color {
        self.colors[ColorIndex::SelectionBg as usize]
    }

    pub fn selection_fg(&self) -> Color {
        self.colors[ColorIndex::SelectionFg as usize]
    }

    pub fn border(&self) -> Color {
        self.colors[ColorIndex::Border as usize]
    }

    pub fn error(&self) -> Color {
        self.colors[ColorIndex::Error as usize]
    }

    pub fn success(&self) -> Color {
        self.colors[ColorIndex::Success as usize]
    }

    pub fn warning(&self) -> Color {
        self.colors[ColorIndex::Warning as usize]
    }
    pub fn info(&self) -> Color {
        self.colors[ColorIndex::Info as usize]
    }

    pub fn accent(&self) -> Color {
        self.colors[ColorIndex::Accent as usize]
    }

    pub fn highlight(&self) -> Color {
        self.colors[ColorIndex::Highlight as usize]
    }

    pub fn muted(&self) -> Color {
        self.colors[ColorIndex::Muted as usize]
    }

    // ***************************************

    pub fn new_from_name(name: &str) -> Self {
        let mut loader = ThemeLoader::init();
        loader
            .get_theme(name)
            .unwrap_or_else(|_| Self::fallback_theme())
    }
}

impl std::str::FromStr for ColorSupport {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic" => Ok(Self::Basic),
            "256" | "extended" => Ok(Self::Extended),
            "true" | "truecolor" => Ok(Self::TrueColor),
            _ => Err(format!("Invalid color support value: {}", s)),
        }
    }
}

impl ThemeLoader {
    fn init() -> Self {
        let mut loader = Self {
            themes: HashMap::new(),
        };
        if loader.load_theme(DEFAULT_THEME).is_err() {
            loader
                .themes
                .insert(DEFAULT_THEME.to_string(), Theme::fallback_theme());
        }
        loader
    }

    /// Checks if the given theme is available
    pub fn has_theme(theme: &str) -> bool {
        available_themes().contains(&theme.to_string())
    }

    /// Loads a theme from file
    fn load_theme(&mut self, theme_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Self::has_theme(theme_name) {
            return Err(format!("Theme '{theme_name}' is not available.").into());
        }

        if self.themes.contains_key(theme_name) {
            return Ok(());
        }

        let path = PathBuf::from(format!("assets/themes/{theme_name}"));
        let content = fs::read_to_string(path)?;
        let theme = Self::parse_theme_file(&content, theme_name)?;

        self.themes.insert(theme_name.to_string(), theme);
        Ok(())
    }

    /// Get a theme by name, loading it if necessary
    pub fn get_theme(&mut self, theme_name: &str) -> Result<Theme, Box<dyn std::error::Error>> {
        if !self.themes.contains_key(theme_name) && self.load_theme(theme_name).is_err() {
            return Ok(Theme::fallback_theme());
        }
        Ok(self.themes.get(theme_name).unwrap().clone())
    }

    fn parse_theme_file(content: &str, name: &str) -> Result<Theme, Box<dyn std::error::Error>> {
        let mut color_map: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            if line.starts_with("palette =") {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 3 {
                    let index = parts[1].trim();
                    let color = parts[2].trim().trim_start_matches('#');
                    color_map.insert(format!("palette{}", index), color.to_string());
                }
            } else if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_start_matches('#');
                color_map.insert(key.to_string(), value.to_string());
            }
        }

        let parse_color = |key: &str| -> Result<Color, Box<dyn std::error::Error>> {
            let value = color_map.get(key).ok_or(format!("Missing {}", key))?;
            Color::from_str(&format!("#{}", value))
                .map_err(|e| format!("Invalid color for {}: {}", key, e).into())
        };

        let mut colors = [Color::Black; 14];
        colors[ColorIndex::Background as usize] = parse_color("background")?;
        colors[ColorIndex::Foreground as usize] = parse_color("foreground")?;
        colors[ColorIndex::Cursor as usize] = parse_color("cursor-color")?;
        colors[ColorIndex::CursorText as usize] = parse_color("cursor-text")?;
        colors[ColorIndex::SelectionBg as usize] = parse_color("selection-background")?;
        colors[ColorIndex::SelectionFg as usize] = parse_color("selection-foreground")?;
        colors[ColorIndex::Border as usize] = parse_color("palette8")?;
        colors[ColorIndex::Error as usize] = parse_color("palette1")?;
        colors[ColorIndex::Success as usize] = parse_color("palette2")?;
        colors[ColorIndex::Warning as usize] = parse_color("palette3")?;
        colors[ColorIndex::Info as usize] = parse_color("palette4")?;
        colors[ColorIndex::Accent as usize] = parse_color("palette5")?;
        colors[ColorIndex::Highlight as usize] = parse_color("palette6")?;
        colors[ColorIndex::Muted as usize] = parse_color("palette8")?;

        Ok(Theme {
            identifier: name.to_string(),
            colors,
            color_support: ColorSupport::Extended,
        })
    }
}
/// Returns the list of available themes.
pub fn available_themes() -> &'static [String] {
    static THEMES: OnceLock<Vec<String>> = OnceLock::new();
    THEMES.get_or_init(|| {
        let paths = fs::read_dir("assets/themes")
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter(|entry| entry.path().is_file())
                    .filter_map(|entry| {
                        entry
                            .path()
                            .file_name()
                            .and_then(|n| n.to_str())
                            .filter(|name| *name != ".gitkeep")
                            .map(String::from)
                    })
                    .collect()
            })
            .unwrap_or_else(|_| vec![DEFAULT_THEME.to_string()]);
        paths
    })
}

pub fn print_theme_list() {
    let mut themes: Vec<String> = available_themes().to_vec();
    themes.sort_by_key(|a| a.to_lowercase());

    println!("\n• Available Themes ({}):", themes.len());

    println!("{}", "─".repeat(40));

    for theme in themes {
        let is_default = theme == DEFAULT_THEME;
        let theme_name = if is_default {
            format!("{} (default)", theme)
        } else {
            theme
        };
        println!("  • {}", theme_name);
    }

    println!("\nUsage:");
    println!("  • Set theme:    termitype --theme <name>");
    println!("  • List themes:  termitype --list-themes");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_theme(dir: &TempDir, name: &str, content: &str) {
        let theme_dir = dir.path().join("assets").join("themes");
        fs::create_dir_all(&theme_dir).unwrap();
        fs::write(theme_dir.join(name), content).unwrap();
    }

    #[test]
    fn test_theme_parsing() {
        let content = r#"
            background = #000000
            foreground = #ffffff
            cursor-color = #cccccc
            cursor-text = #000000
            selection-background = #333333
            selection-foreground =  #ffffff
            palette1 = #ff0000
            palette2 = #00ff00
            palette3 = #ffff00
            palette4 = #0000ff
            palette5 = #ff00ff
            palette6 = #00ffff
            palette7 = #888888
            palette8 = #008888
        "#;

        let theme = ThemeLoader::parse_theme_file(content, "test").unwrap();

        assert_eq!(theme.background(), Color::Rgb(0, 0, 0));
        assert_eq!(theme.foreground(), Color::Rgb(255, 255, 255));
        assert_eq!(theme.error(), Color::Rgb(255, 0, 0));
        assert_eq!(theme.success(), Color::Rgb(0, 255, 0));
    }

    #[test]
    fn test_theme_loading() {
        let temp_dir = TempDir::new().unwrap();
        let test_theme_content = r#"
            background = #000000
            foreground = #ffffff
            cursor-color = #cccccc
            cursor-text = #000000
            selection-background = #333333
            palette1 = #ff0000
            palette2 = #00ff00
            palette3 = #ffff00
            palette4 = #0000ff
            palette5 = #ff00ff
            palette6 = #00ffff
            palette8 = #888888
        "#;

        create_test_theme(&temp_dir, "test_theme", test_theme_content);

        let mut loader = ThemeLoader {
            themes: HashMap::new(),
        };
        let result = loader.load_theme("test_theme");
        assert!(result.is_err()); // should fail because wer not using the real assets dir
    }

    #[test]
    fn test_invalid_theme_color() {
        let content = r#"
            background = #GGGGGG  # Invalid hex color
            foreground = #ffffff
            cursor-color = #cccccc
            cursor-text = #000000
            selection-background = #333333
            palette1 = #ff0000
            palette2 = #00ff00
            palette3 = #ffff00
            palette4 = #0000ff
            palette5 = #ff00ff
            palette6 = #00ffff
            palette8 = #888888
        "#;

        let result = ThemeLoader::parse_theme_file(content, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_color() {
        let content = r#"
            # Missing background color
            foreground = #ffffff
            cursor-color = #cccccc
            cursor-text = #000000
            selection-background = #333333
            palette1 = #ff0000
            palette2 = #00ff00
            palette3 = #ffff00
            palette4 = #0000ff
            palette5 = #ff00ff
            palette6 = #00ffff
            palette8 = #888888
        "#;

        let result = ThemeLoader::parse_theme_file(content, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_color_support_comparison() {
        assert!(ColorSupport::TrueColor > ColorSupport::Extended);
        assert!(ColorSupport::Extended > ColorSupport::Basic);
        assert!(ColorSupport::TrueColor > ColorSupport::Basic);
    }

    #[test]
    fn test_supports_themes() {
        assert_eq!(ColorSupport::Basic.supports_themes(), false);
        assert_eq!(ColorSupport::Extended.supports_themes(), true);
        assert_eq!(ColorSupport::TrueColor.supports_themes(), true);
    }

    #[test]
    fn test_detect_color_support() {
        env::set_var("COLORTERM", "truecolor");
        assert_eq!(Theme::detect_color_support(), ColorSupport::TrueColor);

        env::set_var("COLORTERM", "24bit");
        assert_eq!(Theme::detect_color_support(), ColorSupport::TrueColor);

        env::set_var("COLORTERM", "256color");
        assert_eq!(Theme::detect_color_support(), ColorSupport::Extended);

        // fallback
        env::set_var("COLORTERM", "other");
        assert_eq!(Theme::detect_color_support(), ColorSupport::Basic);

        // no $COLORTERM found
        env::remove_var("COLORTERM");
        assert_eq!(Theme::detect_color_support(), ColorSupport::Basic);
    }

    #[test]
    fn test_fallback_theme() {
        let theme = Theme::fallback_theme();
        assert_eq!(theme.color_support, ColorSupport::Basic);
        assert_eq!(theme.background(), Color::Black);
        assert_eq!(theme.foreground(), Color::White);
        assert_eq!(theme.identifier, "Default".to_string());
    }

    #[test]
    fn test_color_mode_from_config() {
        let mut config = Config::default();

        config.color_mode = Some("basic".to_string());
        assert_eq!(Theme::new(&config).color_support, ColorSupport::Basic);

        config.color_mode = Some("extended".to_string());
        assert_eq!(Theme::new(&config).color_support, ColorSupport::Extended);

        config.color_mode = Some("truecolor".to_string());
        assert_eq!(Theme::new(&config).color_support, ColorSupport::TrueColor);

        // Invalid mode given - should fallback to auto-detection
        config.color_mode = Some("invalid".to_string());
        let detected = Theme::detect_color_support();
        assert_eq!(Theme::new(&config).color_support, detected);

        // should auto-detect if no mode was given
        config.color_mode = None;
        assert_eq!(Theme::new(&config).color_support, detected);

        // Config given have higher priority than the current terminal "capabiilities"
        env::set_var("COLORTERM", "truecolor");
        config.color_mode = Some("basic".to_string());
        assert_eq!(Theme::new(&config).color_support, ColorSupport::Basic);
    }
}
