use std::{collections::HashMap, fs, path::PathBuf, str::FromStr, sync::OnceLock};

use ratatui::style::Color;

use crate::{config::Config, constants::DEFAULT_THEME};

#[derive(Debug)]
pub struct ThemeLoader {
    themes: HashMap<String, Theme>,
}

#[derive(Debug, Clone)]
pub struct Theme {
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
}

impl ThemeLoader {
    fn init() -> Self {
        let mut loader = Self {
            themes: HashMap::new(),
        };
        loader
            .load_theme(DEFAULT_THEME)
            .expect("Default theme must exist");
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
        let theme = Self::parse_theme_file(&content)?;

        self.themes.insert(theme_name.to_string(), theme);
        Ok(())
    }

    /// Get a theme by name, loading it if necessary
    pub fn get_theme(&mut self, theme_name: &str) -> Result<Theme, Box<dyn std::error::Error>> {
        if !self.themes.contains_key(theme_name) {
            self.load_theme(theme_name)?;
        }
        Ok(self.themes.get(theme_name).unwrap().clone())
    }

    fn parse_theme_file(content: &str) -> Result<Theme, Box<dyn std::error::Error>> {
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
            background: Color::from_str(&format!(
                "#{}",
                colors.get("background").ok_or("Missing background")?
            ))?,
            background_secondary: Color::from_str(&format!(
                "#{}",
                colors
                    .get("selection-background")
                    .ok_or("Missing selection-background")?
            ))?,
            foreground: Color::from_str(&format!(
                "#{}",
                colors.get("foreground").ok_or("Missing foreground")?
            ))?,
            foreground_secondary: Color::from_str(&format!(
                "#{}",
                colors.get("palette8").ok_or("Missing palette 8")?
            ))?,

            cursor: Color::from_str(&format!(
                "#{}",
                colors.get("cursor-color").ok_or("Missing cursor-color")?
            ))?,
            cursor_text: Color::from_str(&format!(
                "#{}",
                colors.get("cursor-text").ok_or("Missing cursor-text")?
            ))?,
            selection: Color::from_str(&format!(
                "#{}",
                colors
                    .get("selection-background")
                    .ok_or("Missing selection-background")?
            ))?,
            border: Color::from_str(&format!(
                "#{}",
                colors.get("palette8").ok_or("Missing palette 8")?
            ))?,

            error: Color::from_str(&format!(
                "#{}",
                colors.get("palette1").ok_or("Missing palette 1")?
            ))?,
            success: Color::from_str(&format!(
                "#{}",
                colors.get("palette2").ok_or("Missing palette 2")?
            ))?,
            warning: Color::from_str(&format!(
                "#{}",
                colors.get("palette3").ok_or("Missing palette 3")?
            ))?,
            info: Color::from_str(&format!(
                "#{}",
                colors.get("palette4").ok_or("Missing palette 4")?
            ))?,

            accent: Color::from_str(&format!(
                "#{}",
                colors.get("palette5").ok_or("Missing palette 5")?
            ))?,
            highlight: Color::from_str(&format!(
                "#{}",
                colors.get("palette6").ok_or("Missing palette 6")?
            ))?,
            inactive: Color::from_str(&format!(
                "#{}",
                colors.get("palette8").ok_or("Missing palette 8")?
            ))?,
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

impl Theme {
    pub fn new(config: &Config) -> Self {
        let mut loader = ThemeLoader::init();
        let theme_name = config.theme.as_deref().unwrap_or(DEFAULT_THEME);
        loader.get_theme(theme_name).unwrap_or_else(|_| {
            loader
                .get_theme(DEFAULT_THEME)
                .expect("Default theme must exist")
        })
    }
}

pub fn print_theme_list() {
    let mut themes: Vec<String> = available_themes().to_vec();
    themes.sort();

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

        let theme = ThemeLoader::parse_theme_file(content).unwrap();
        
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

        let mut loader = ThemeLoader { themes: HashMap::new() };
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

        let result = ThemeLoader::parse_theme_file(content);
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

        let result = ThemeLoader::parse_theme_file(content);
        assert!(result.is_err());
    }
}
