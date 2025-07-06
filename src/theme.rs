use std::{
    cell::RefCell,
    collections::HashMap,
    str::FromStr,
    sync::{OnceLock, RwLock},
    thread_local,
};

use ratatui::style::Color;

use crate::{assets, config::Config, constants::DEFAULT_THEME, notify_warning};

static THEME_LOADER: OnceLock<RwLock<ThemeLoader>> = OnceLock::new();

const NUM_COLORS: usize = 14;

#[derive(Debug)]
pub struct ThemeLoader {
    themes: HashMap<String, Theme>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: String,
    colors: [Color; NUM_COLORS],
    pub color_support: ColorSupport,
}

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum ThemeColor {
    Background = 0, // Main Backround
    Foreground,     // Default Text
    Muted,          // Dimmed Text
    Accent,         // Carets and Arrows
    Info,           // Information
    Primary,        // Primary
    Highlight,      // Selected items
    Success,        // Correct Words
    Error,          // Wrong Words
    Warning,        // Warning messages
    Cursor,         // Cursor Color
    CursorText,     // Text under Cursor
    SelectionBg,    // Text Selection Background
    SelectionFg,    // Text Selection Foreground
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
    /// Checks if the terminal likely supports theming
    pub fn supports_themes(self) -> bool {
        self >= ColorSupport::Extended
    }

    /// Checks if the terminal likely supports Unicode characters
    pub fn supports_unicode(self) -> bool {
        // TODO: improve this detection. This heuristic will probably be wrong in some cases
        self >= ColorSupport::Extended
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

thread_local! {
    static THEME_CACHE: RefCell<HashMap<String, Theme>> = RefCell::new(HashMap::new());
    static CACHE_INITIALIZED: RefCell<bool> = const { RefCell::new(false) };
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
        let loader = ThemeLoader::init();
        let theme_name = config.theme.as_deref().unwrap_or(DEFAULT_THEME);

        let mut theme = match loader.write() {
            Ok(mut loader_guard) => {
                match loader_guard.get_theme(theme_name) {
                    Ok(theme) => theme,
                    Err(_) => {
                        // if fail, then default to `DEFAULT_THEME`.
                        match loader_guard.get_theme(DEFAULT_THEME) {
                            Ok(theme) => {
                                notify_warning!(format!("Could not find theme '{theme_name}'"));
                                theme
                            }
                            Err(_) => {
                                // if all else fails then use the fallback theme
                                Self::fallback_theme_with_support(color_support)
                            }
                        }
                    }
                }
            }
            Err(_) => Self::fallback_theme_with_support(color_support),
        };

        theme.color_support = color_support;
        theme
    }

    /// Detects the terminal's color support level
    pub fn detect_color_support() -> ColorSupport {
        // $COLORTERM
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            match colorterm.to_lowercase().as_str() {
                "truecolor" | "24bit" => return ColorSupport::TrueColor,
                // NOTE: don't assume Basic support yet.
                _ => {}
            }
        }

        // known problematic terminal overrides. this might be dumb but good enough for now
        if cfg!(target_os = "macos") {
            if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
                #[allow(clippy::single_match)]
                match term_program.as_str() {
                    // MacOS Terminal.app blatanly lies about 256color support and do not really supports it
                    "Apple_Terminal" => return ColorSupport::Basic,
                    _ => {}
                }
            }
        }

        // TODO: spin up termitype on a windows VM and check

        // $TERM
        if let Ok(term) = std::env::var("TERM") {
            let target = term.to_lowercase();

            if target.contains("truecolor") {
                return ColorSupport::TrueColor;
            }

            // known terminals that support truecolor
            if matches!(target.as_str(), "alacritty" | "kitty" | "wezterm") {
                return ColorSupport::TrueColor;
            }

            if target.contains("256color") {
                return ColorSupport::Extended;
            }

            if target.starts_with("screen") || target.starts_with("tmux") {
                return ColorSupport::Extended;
            }
        }

        // use fallback theme, at this point I don't know if theres something else we can do
        ColorSupport::Basic
    }

    fn fallback_theme() -> Self {
        // TODO: detect system color (dark/light) and show a respective fallback theme.
        //      could use: https://crates.io/crates/terminal-light
        notify_warning!("Using fallback theme");
        Self {
            id: "Fallback".to_string(),
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
            color_support: ColorSupport::Basic,
        }
    }

    fn fallback_theme_with_support(color_support: ColorSupport) -> Self {
        let mut theme = Self::fallback_theme();
        theme.color_support = color_support;
        theme
    }

    fn ensure_all_themes_loaded() {
        let initialized = CACHE_INITIALIZED.with(|init| *init.borrow());
        if initialized {
            return;
        }

        let loader = ThemeLoader::init();
        let themes = available_themes();

        // loader
        for theme_name in themes {
            let needs_loading = {
                let read_guard = loader.read().unwrap();
                !read_guard.themes.contains_key(theme_name)
            };

            if needs_loading {
                loader.write().unwrap().load_theme(theme_name).ok();
            }
        }

        // thread-local cache
        {
            let read_guard = loader.read().unwrap();
            THEME_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                for (name, theme) in &read_guard.themes {
                    cache.insert(name.clone(), theme.clone());
                }
            });
        }

        // Mark it as good
        CACHE_INITIALIZED.with(|init| {
            *init.borrow_mut() = true;
        });
    }

    pub fn from_name(name: &str) -> Self {
        Self::ensure_all_themes_loaded();

        // cache hit from thread-local
        let cached_theme = THEME_CACHE.with(|cache| {
            let cache = cache.borrow();
            cache.get(name).cloned()
        });

        if let Some(theme) = cached_theme {
            return theme;
        }

        // fallback to loader (this should not happen often)
        let loader = ThemeLoader::init();
        let theme = loader
            .read()
            .unwrap()
            .themes
            .get(name)
            .cloned()
            .unwrap_or_else(|| {
                let fallback = Self::fallback_theme();

                THEME_CACHE.with(|cache| {
                    let mut cache = cache.borrow_mut();
                    cache.insert(name.to_string(), fallback.clone());
                });

                fallback
            });

        theme
    }

    // ************** COLORS_FN **************
    pub fn bg(&self) -> Color {
        self.colors[ThemeColor::Background as usize]
    }

    pub fn fg(&self) -> Color {
        self.colors[ThemeColor::Foreground as usize]
    }

    pub fn muted(&self) -> Color {
        self.colors[ThemeColor::Muted as usize]
    }

    pub fn accent(&self) -> Color {
        self.colors[ThemeColor::Accent as usize]
    }

    pub fn info(&self) -> Color {
        self.colors[ThemeColor::Info as usize]
    }

    pub fn primary(&self) -> Color {
        self.colors[ThemeColor::Primary as usize]
    }

    pub fn highlight(&self) -> Color {
        self.colors[ThemeColor::Highlight as usize]
    }

    pub fn success(&self) -> Color {
        self.colors[ThemeColor::Success as usize]
    }

    pub fn error(&self) -> Color {
        self.colors[ThemeColor::Error as usize]
    }

    pub fn warning(&self) -> Color {
        self.colors[ThemeColor::Warning as usize]
    }

    pub fn border(&self) -> Color {
        self.colors[ThemeColor::Muted as usize]
    }

    pub fn cursor(&self) -> Color {
        self.colors[ThemeColor::Cursor as usize]
    }

    pub fn cursor_text(&self) -> Color {
        self.colors[ThemeColor::CursorText as usize]
    }

    pub fn selection_bg(&self) -> Color {
        self.colors[ThemeColor::SelectionBg as usize]
    }

    pub fn selection_fg(&self) -> Color {
        self.colors[ThemeColor::SelectionFg as usize]
    }

    /// Checks if the terminal likely supports Unicode characters
    pub fn supports_unicode(&self) -> bool {
        self.color_support.supports_unicode()
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

impl ThemeLoader {
    fn init() -> &'static RwLock<ThemeLoader> {
        THEME_LOADER.get_or_init(|| {
            let loader = Self {
                themes: HashMap::new(),
            };

            // NOTE: is this tradeoff worth it? The increase in startup time is neglible
            //       but we should consider the memory overheaad of having all the themes
            //       loaded when in reality not everytime the user will use the theme previewer.
            // for theme_name in assets::list_themes() {
            //     loader.load_theme(&theme_name).ok();
            // }

            RwLock::new(loader)
        })
    }

    /// Checks if the given theme is available
    pub fn has_theme(theme: &str) -> bool {
        assets::get_theme(theme).is_some()
    }

    /// Loads a theme from assets
    fn load_theme(&mut self, theme_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Self::has_theme(theme_name) {
            return Err(format!("[ERROR] Theme '{theme_name}' is not available.").into());
        }

        if self.themes.contains_key(theme_name) {
            return Ok(());
        }

        let content = assets::get_theme(theme_name)
            .ok_or_else(|| format!("Theme '{theme_name}' not found"))?;
        let theme = Self::parse_theme_file(&content, theme_name)?;

        self.themes.insert(theme_name.to_string(), theme);
        Ok(())
    }

    /// Get a theme by name, loading it if necessary
    pub fn get_theme(&mut self, theme_name: &str) -> Result<Theme, Box<dyn std::error::Error>> {
        if let Some(theme) = self.themes.get(theme_name) {
            return Ok(theme.clone());
        }

        self.load_theme(theme_name)?;

        Ok(self
            .themes
            .get(theme_name)
            .cloned()
            .unwrap_or_else(Theme::fallback_theme))
    }

    fn parse_theme_file(content: &str, name: &str) -> Result<Theme, Box<dyn std::error::Error>> {
        let mut color_map: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            if line.starts_with("palette =") {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 3 {
                    let index = parts[1].trim();
                    let color = parts[2].trim().trim_start_matches('#');
                    color_map.insert(format!("palette{index}"), color.to_string());
                }
            } else if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_start_matches('#');
                color_map.insert(key.to_string(), value.to_string());
            }
        }

        let parse_color = |key: &str| -> Result<Color, Box<dyn std::error::Error>> {
            let value = color_map.get(key).ok_or(format!("Missing {key}"))?;
            Color::from_str(&format!("#{value}"))
                .map_err(|e| format!("Invalid color for {key}: {e}").into())
        };

        let mut colors = [Color::Black; NUM_COLORS];
        colors[ThemeColor::Background as usize] = parse_color("background")?;
        colors[ThemeColor::Foreground as usize] = parse_color("foreground")?;
        // colors[ThemeColor::Muted as usize] = parse_color("palette0")?;
        colors[ThemeColor::Muted as usize] = parse_color("foreground")?;
        // colors[ThemeColor::Muted as usize] = parse_color("cursor-color")?;
        // colors[ThemeColor::Muted as usize] = parse_color("palette12")?;
        // colors[ThemeColor::Muted as usize] = parse_color("selection-foreground")?;
        // colors[ThemeColor::Muted as usize] = parse_color("palette15")?;
        colors[ThemeColor::Warning as usize] = parse_color("palette3")?;
        colors[ThemeColor::Accent as usize] = parse_color("palette10")?;
        colors[ThemeColor::Info as usize] = parse_color("palette4")?;
        colors[ThemeColor::Primary as usize] = parse_color("palette5")?;
        colors[ThemeColor::Highlight as usize] = parse_color("palette6")?;
        colors[ThemeColor::Success as usize] = parse_color("palette2")?;
        colors[ThemeColor::Error as usize] = parse_color("palette1")?;
        colors[ThemeColor::Cursor as usize] = parse_color("cursor-color")?;
        colors[ThemeColor::CursorText as usize] = parse_color("palette0")?;
        colors[ThemeColor::SelectionBg as usize] = parse_color("selection-background")?;
        colors[ThemeColor::SelectionFg as usize] = parse_color("selection-foreground")?;

        Ok(Theme {
            id: name.to_string(),
            colors,
            color_support: ColorSupport::Extended,
        })
    }
}

/// Returns the list of available themes.
pub fn available_themes() -> &'static [String] {
    static THEMES: OnceLock<Vec<String>> = OnceLock::new();
    THEMES.get_or_init(|| {
        let mut themes = assets::list_themes();
        themes.sort_by_key(|a| a.to_lowercase());
        themes
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
            format!("{theme} (default)")
        } else {
            theme
        };
        println!("  • {theme_name}");
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
            palette0 = #ff0000
            palette1 = #ff0000
            palette2 = #00ff00
            palette3 = #ffff00
            palette4 = #0000ff
            palette5 = #ff00ff
            palette6 = #00ffff
            palette7 = #888888
            palette8 = #008888
            palette10 = #008888
            palette14 = #00ffff
        "#;

        let theme = ThemeLoader::parse_theme_file(content, "test").unwrap();

        assert_eq!(theme.bg(), Color::Rgb(0, 0, 0));
        assert_eq!(theme.fg(), Color::Rgb(255, 255, 255));
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
            selection-foreground = #ffffff
            palette0 = #ff0000
            palette1 = #ff0000
            palette2 = #00ff00
            palette3 = #ffff00
            palette4 = #0000ff
            palette5 = #ff00ff
            palette6 = #00ffff
            palette8 = #888888
            palette10 = #008888
        "#;

        create_test_theme(&temp_dir, "test_theme", test_theme_content);

        let mut loader = ThemeLoader {
            themes: HashMap::new(),
        };
        let result = loader.load_theme("test_theme");
        assert!(result.is_err()); // should fail because wer not using the real assets dir
    }

    #[test]
    fn test_loading_incorrect_theme_with_truecolor() {
        env::set_var("COLORTERM", "truecolor");
        let mut config = Config::new();
        config.theme = Some("random-theme-that-does-not-exists".to_string());
        let theme = Theme::new(&config);
        // if the given theme does not exists, it will default to the `DEFAULT_THEME`,
        // if that is not found it will default to `Fallbakck` as last measure.
        assert!(theme.id == DEFAULT_THEME || theme.id == "Fallback");
    }

    #[test]
    fn test_loading_incorrect_theme_without_truecolor() {
        env::remove_var("TERM");
        env::remove_var("COLORTERM");
        let mut config = Config::new();
        config.theme = Some("random-theme-that-does-not-exists".to_string());
        let theme = Theme::new(&config);
        assert_eq!(theme.id, "Fallback".to_string());
    }

    #[test]
    fn test_invalid_theme_color() {
        let content = r#"
            background = #GGGGGG  # Invalid hex color
            foreground = #ffffff
            cursor-color = #cccccc
            cursor-text = #000000
            selection-background = #333333
            palette0 = #ff0000
            palette1 = #ff0000
            palette2 = #00ff00
            palette3 = #ffff00
            palette4 = #0000ff
            palette5 = #ff00ff
            palette6 = #00ffff
            palette8 = #888888
            palette10 = #008888
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
            palette0 = #ff0000
            palette1 = #ff0000
            palette2 = #00ff00
            palette3 = #ffff00
            palette4 = #0000ff
            palette5 = #ff00ff
            palette6 = #00ffff
            palette8 = #888888
            palette10 = #008888
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
        assert!(!ColorSupport::Basic.supports_themes());
        assert!(ColorSupport::Extended.supports_themes());
        assert!(ColorSupport::TrueColor.supports_themes());
    }

    #[test]
    fn test_detect_color_support() {
        env::remove_var("TERM");
        env::remove_var("TERM_PROGRAM");

        env::set_var("COLORTERM", "truecolor");
        assert_eq!(Theme::detect_color_support(), ColorSupport::TrueColor);

        env::set_var("COLORTERM", "24bit");
        assert_eq!(Theme::detect_color_support(), ColorSupport::TrueColor);

        env::set_var("COLORTERM", "256color");
        assert_eq!(Theme::detect_color_support(), ColorSupport::Basic);

        env::set_var("COLORTERM", "other");
        assert_eq!(Theme::detect_color_support(), ColorSupport::Basic);

        env::remove_var("COLORTERM");
        assert_eq!(Theme::detect_color_support(), ColorSupport::Basic);

        env::remove_var("COLORTERM");
        env::set_var("TERM", "xterm-256color");
        assert_eq!(Theme::detect_color_support(), ColorSupport::Extended);

        env::set_var("TERM", "screen-256color");
        assert_eq!(Theme::detect_color_support(), ColorSupport::Extended);

        env::set_var("TERM", "tmux-256color");
        assert_eq!(Theme::detect_color_support(), ColorSupport::Extended);

        env::set_var("TERM", "alacritty");
        assert_eq!(Theme::detect_color_support(), ColorSupport::TrueColor);
    }

    #[test]
    fn test_fallback_theme() {
        let theme = Theme::fallback_theme();
        assert_eq!(theme.color_support, ColorSupport::Basic);
        assert_eq!(theme.bg(), Color::Black);
        assert_eq!(theme.fg(), Color::White);
        assert_eq!(theme.id, "Fallback".to_string());
    }

    #[test]
    fn test_color_mode_from_config() {
        let mut config = Config::default();
        config.color_mode = Some("basic".to_string());
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
