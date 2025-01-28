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
    pub background: Color,
    pub background_secondary: Color,
    pub foreground: Color,
    pub foreground_secondary: Color,

    pub cursor: Color,
    pub cursor_text: Color,
    pub selection: Color,
    pub border: Color,

    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub info: Color,

    pub accent: Color,
    pub highlight: Color,
    pub inactive: Color,

    pub color_support: ColorSupport,
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

impl Theme {
    pub fn new(config: &Config) -> Self {
        let color_support = Self::detect_color_support();

        // No theme support. Get a better terminal m8
        if !color_support.supports_themes() {
            return Self::fallback_theme();
        }
        let mut loader = ThemeLoader::init();
        let theme_name = config.theme.as_deref().unwrap_or(DEFAULT_THEME);
        loader.get_theme(theme_name).unwrap_or_else(|_| {
            loader
                .get_theme(DEFAULT_THEME)
                .expect("Default theme must exist")
        })
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

    // TODO: figure out why this looks horrible on ghostty on my work machine
    fn fallback_theme() -> Self {
        Self {
            identifier: "Default".to_string(),
            background: Color::Black,
            background_secondary: Color::Black,
            foreground: Color::White,
            foreground_secondary: Color::White,
            cursor: Color::White,
            cursor_text: Color::Black,
            selection: Color::White,
            border: Color::White,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            info: Color::Blue,
            accent: Color::Magenta,
            highlight: Color::Cyan,
            inactive: Color::DarkGray,
            color_support: ColorSupport::Basic,
        }
    }
}

impl ThemeLoader {
    fn init() -> Self {
        let mut loader = Self {
            themes: HashMap::new(),
        };
        if let Err(_) = loader.load_theme(DEFAULT_THEME) {
            loader.themes.insert(DEFAULT_THEME.to_string(), Theme::fallback_theme());
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
        if !self.themes.contains_key(theme_name) {
            if let Err(_) = self.load_theme(theme_name) {
                return Ok(Theme::fallback_theme());
            }
        }
        Ok(self.themes.get(theme_name).unwrap().clone())
    }

    fn parse_color(
        colors: &HashMap<String, String>,
        key: &str,
    ) -> Result<Color, Box<dyn std::error::Error>> {
        let value = colors.get(key).ok_or(format!("Missing {}", key))?;
        Color::from_str(&format!("#{}", value))
            .map_err(|e| format!("Invalid color for {}: {}", key, e).into())
    }

    fn parse_theme_file(content: &str, name: &str) -> Result<Theme, Box<dyn std::error::Error>> {
        let mut colors: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            if line.starts_with("palette =") {
                // palette entries with (format: "palette = N=#XXXXXX")
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 3 {
                    let index = parts[1].trim();
                    let color = parts[2].trim().trim_start_matches('#');
                    colors.insert(format!("palette{}", index), color.to_string());
                }
            // regular entries with (format: "key = value")
            } else if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_start_matches('#');
                colors.insert(key.to_string(), value.to_string());
            }
        }

        Ok(Theme {
            identifier: name.to_string(),
            background: Self::parse_color(&colors, "background")?,
            background_secondary: Self::parse_color(&colors, "selection-background")?,
            foreground: Self::parse_color(&colors, "foreground")?,
            foreground_secondary: Self::parse_color(&colors, "palette8")?,

            cursor: Self::parse_color(&colors, "cursor-color")?,
            cursor_text: Self::parse_color(&colors, "cursor-text")?,
            selection: Self::parse_color(&colors, "selection-background")?,
            border: Self::parse_color(&colors, "palette8")?,

            error: Self::parse_color(&colors, "palette1")?,
            success: Self::parse_color(&colors, "palette2")?,
            warning: Self::parse_color(&colors, "palette3")?,
            info: Self::parse_color(&colors, "palette4")?,

            accent: Self::parse_color(&colors, "palette5")?,
            highlight: Self::parse_color(&colors, "palette6")?,
            inactive: Self::parse_color(&colors, "palette8")?,
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
    themes.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    println!("\n{} Available Themes ({}):", "•", themes.len());
    println!("{}", "─".repeat(40));

    for theme in themes {
        let is_default = theme == DEFAULT_THEME;
        let theme_name = if is_default {
            format!("{} (default)", theme)
        } else {
            theme
        };
        println!("  {} {}", "•", theme_name);
    }

    println!("\nUsage:");
    println!("  {} Set theme:    termitype --theme <name>", "•");
    println!("  {} List themes:  termitype --list-themes", "•");
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
            palette1 = #ff0000
            palette2 = #00ff00
            palette3 = #ffff00
            palette4 = #0000ff
            palette5 = #ff00ff
            palette6 = #00ffff
            palette8 = #888888
        "#;

        let theme = ThemeLoader::parse_theme_file(content, "test").unwrap();

        assert_eq!(theme.background, Color::Rgb(0, 0, 0));
        assert_eq!(theme.foreground, Color::Rgb(255, 255, 255));
        assert_eq!(theme.cursor, Color::Rgb(204, 204, 204));
        assert_eq!(theme.error, Color::Rgb(255, 0, 0));
        assert_eq!(theme.success, Color::Rgb(0, 255, 0));
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
        assert_eq!(theme.background, Color::Black);
        assert_eq!(theme.foreground, Color::White);
        assert_eq!(theme.identifier, "Default".to_string());
    }
}
