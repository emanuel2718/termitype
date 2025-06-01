use std::str::FromStr;

use clap::{ArgGroup, Parser};
use crossterm::cursor::SetCursorStyle;

use crate::{
    ascii::DEFAULT_ASCII_ART_NAME,
    builder::Builder,
    constants::{
        DEFAULT_CURSOR_STYLE, DEFAULT_LANGUAGE, DEFAULT_LINE_COUNT, DEFAULT_PICKER_STYLE,
        DEFAULT_RESULTS_STYLE, DEFAULT_THEME, DEFAULT_TIME_MODE_DURATION, DEFAULT_WORD_MODE_COUNT,
        WPS_TARGET,
    },
    persistence::Persistence,
    theme::{ColorSupport, Theme, ThemeLoader},
};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "termitype",
    about = "Terminal-based typing game.",
    after_help = "EXAMPLES:\n  \
                  termitype -t 60                        Run a 60-second typing test\n  \
                  termitype --word-count 100             Test will contain exactly 100 words\n  \
                  termitype -T \"catppuccin-mocha\"        Use cattpuccin-mocha theme\n  \
                  termitype -l spanish                   Use Spanish test words\n  \
                  termitype -spn                         Enable symbols, punctuation, and numbers\n  \
                  termitype --list-themes                Show all available themes\n  \
                  termitype --results-style neofetch     Use neofetch inspired results\n  \
                  termitype --picker-style telescope     Use floating menu style\n\n\
                  Note that all of the options can also be changed at runtime as well via the menu\n\
                  Visit https://github.com/emanuel2718/termitype for more information.",
    version
)]
#[command(group(
    ArgGroup::new("mode")
        .args(&["time", "word_count"])
        .required(false)
        .multiple(false)
))]
pub struct Config {
    /// Language dictionary to use
    #[arg(
        short = 'l',
        long,
        value_name = "LANGUAGE",
        help = "Language dictionary to use"
    )]
    pub language: Option<String>,

    /// Test duration in seconds
    #[arg(
        short = 't',
        long = "time",
        group = "mode",
        value_name = "SECONDS",
        help = "Test duration in seconds"
    )]
    pub time: Option<u64>,

    /// Custom words for the test
    #[arg(
        short = 'w',
        long = "words",
        group = "mode",
        value_name = "\"WORD1 WORD2 ...\"",
        help = "Custom words for the test"
    )]
    pub words: Option<String>,

    /// Number of words to type
    #[arg(
        long = "word-count",
        group = "mode",
        value_name = "COUNT",
        help = "Number of words to type"
    )]
    pub word_count: Option<usize>,

    /// Color theme to use
    #[arg(
        short = 'T',
        long = "theme",
        value_name = "THEME_NAME",
        help = "Color theme to use"
    )]
    pub theme: Option<String>,

    /// ASCII art for results screen
    #[arg(
        long = "ascii",
        value_name = "ART_NAME",
        help = "ASCII art for results screen"
    )]
    pub ascii: Option<String>,

    /// Menu Picker style
    #[arg(
        long = "picker-style",
        value_name = "STYLE",
        value_parser = ["quake", "telescope", "ivy", "minimal"],
        help = "Menu style"
    )]
    pub picker_style: Option<String>,

    /// Results display style
    #[arg(
        long = "results-style",
        value_name = "STYLE",
        value_parser = ["graph", "neofetch"],
        help = "Results display style"
    )]
    pub results_style: Option<String>,

    /// Cursor style
    #[arg(
        long = "cursor-style",
        value_name = "STYLE",
        value_parser = ["beam", "block", "underline", "blinking-beam", "blinking-block", "blinking-underline"],
        help = "Cursor style"
    )]
    pub cursor_style: Option<String>,

    /// List all available themes
    #[arg(long = "list-themes", help = "List all available themes")]
    pub list_themes: bool,

    /// List all available languages
    #[arg(long = "list-languages", help = "List all available languages")]
    pub list_languages: bool,

    /// List all available ASCII arts
    #[arg(long = "list-ascii", help = "List all available ASCII arts")]
    pub list_ascii: bool,

    /// Include symbols in test words
    #[arg(
        short = 's',
        long = "use-symbols",
        help = "Include symbols in test words"
    )]
    pub use_symbols: bool,

    /// Include punctuation in test words
    #[arg(
        short = 'p',
        long = "use-punctuation",
        help = "Include punctuation in test words"
    )]
    pub use_punctuation: bool,

    /// Include numbers in test words
    #[arg(
        short = 'n',
        long = "use-numbers",
        help = "Include numbers in test words"
    )]
    pub use_numbers: bool,

    /// Color support level
    #[arg(
        long = "color-mode",
        value_name = "MODE",
        value_parser = ["basic", "extended", "truecolor"],
        help = "Color support"
    )]
    pub color_mode: Option<String>,

    /// Number of visible text lines
    #[arg(
        long = "lines",
        default_value_t = DEFAULT_LINE_COUNT,
        value_name = "COUNT",
        help = "Number of visible text lines"
    )]
    pub visible_lines: u8,

    /// Enable debug mode
    #[cfg(debug_assertions)]
    #[arg(short = 'd', long = "debug", help = "Enable debug mode")]
    pub debug: bool,

    /// Display FPS counter
    #[arg(long = "show-fps", help = "Display FPS counter")]
    pub show_fps: bool,

    /// Hide live WPM counter
    #[arg(long = "hide-live-wpm", help = "Hide live WPM counter")]
    pub hide_live_wpm: bool,

    /// Hide menu cursor highlight
    #[arg(long = "hide-cursorline", help = "Hide menu cursor highlight")]
    pub hide_cursorline: bool,

    /// Use simplified results colors
    #[arg(long = "monochromatic-results", help = "Use simplified results colors")]
    pub monocrhomatic_results: bool,

    /// Stores the persistence of the game. Set automatically.
    #[arg(skip)]
    persistent: Option<Persistence>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModeType {
    Time = 0,
    Words = 1,
}

/// Represents the operationlal mode of the game>
pub enum Mode {
    Time { duration: u64 },
    Words { count: usize },
}

impl Mode {
    /// Returns the value of the current mode.
    pub fn value(&self) -> usize {
        match self {
            Mode::Time { duration } => *duration as usize,
            Mode::Words { count } => *count,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            time: Some(30),
            words: None,
            word_count: None,
            use_symbols: false,
            use_numbers: false,
            use_punctuation: false,
            theme: None,
            ascii: Some(DEFAULT_ASCII_ART_NAME.to_string()),
            language: Some(DEFAULT_LANGUAGE.to_string()),
            cursor_style: Some(DEFAULT_CURSOR_STYLE.to_string()),
            picker_style: Some(DEFAULT_PICKER_STYLE.to_string()),
            results_style: Some(DEFAULT_RESULTS_STYLE.to_string()),
            visible_lines: DEFAULT_LINE_COUNT,
            color_mode: None,
            list_ascii: false,
            list_themes: false,
            list_languages: false,
            show_fps: false,
            hide_live_wpm: false,
            hide_cursorline: false,
            monocrhomatic_results: false,
            #[cfg(debug_assertions)]
            debug: false,
            persistent: None,
        }
    }
}

impl Config {
    /// Returns a new instance of the Config struct with default values.
    pub fn new() -> Self {
        let mut config = Self::default();
        Self::override_with_persistence(&mut config);
        config
    }

    fn override_with_persistence(&mut self) {
        if let Ok(persistence) = Persistence::new() {
            // Theme
            if self.theme.is_none() {
                if let Some(theme) = persistence.get("theme") {
                    if self.theme.is_none() && ThemeLoader::has_theme(theme) {
                        self.theme = Some(theme.to_string());
                    }
                } else {
                    self.theme = Some(DEFAULT_THEME.to_string())
                }
            }

            // Ascii Art
            if self.ascii.is_none() {
                if let Some(art_name) = persistence.get("ascii") {
                    self.ascii = Some(art_name.to_string());
                } else {
                    self.ascii = Some(DEFAULT_ASCII_ART_NAME.to_string());
                }
            }

            // Cursor
            if self.cursor_style.is_none() {
                if let Some(cursor) = persistence.get("cursor") {
                    self.change_cursor_style(cursor);
                } else {
                    self.cursor_style = Some(DEFAULT_CURSOR_STYLE.to_string())
                }
            }

            // Picker Style
            if self.picker_style.is_none() {
                if let Some(picker) = persistence.get("picker_style") {
                    if picker.parse::<PickerStyle>().is_ok() {
                        self.picker_style = Some(picker.to_string());
                    }
                } else {
                    self.picker_style = Some(DEFAULT_PICKER_STYLE.to_string())
                }
            }

            // Current Mode
            if self.time.is_none() && self.words.is_none() {
                // Mode and its value
                let mode_type = persistence
                    .get("mode")
                    .and_then(|mode| self.resolve_mode_from_str(mode));
                let mode_value = persistence
                    .get("mode_value")
                    .and_then(|val| val.parse::<usize>().ok());

                if let Some(mode_type) = mode_type {
                    self.change_mode(mode_type, mode_value);
                }
            }

            // Language
            if self.language.is_none() {
                if let Some(lang) = persistence.get("language") {
                    if Builder::has_language(lang) {
                        self.language = Some(lang.to_string());
                    }
                } else {
                    self.language = Some(DEFAULT_LANGUAGE.to_string())
                }
            }

            // Symbols
            if !self.use_symbols {
                if let Some(use_symbols) = persistence.get("use_symbols") {
                    let val = match use_symbols {
                        "false" => false,
                        "true" => true,
                        _ => false,
                    };
                    self.set_symbols(val);
                }
            }

            // Numbers
            if !self.use_numbers {
                if let Some(use_numbers) = persistence.get("use_numbers") {
                    let val = match use_numbers {
                        "false" => false,
                        "true" => true,
                        _ => false,
                    };
                    self.set_numbers(val);
                }
            }

            // Punctuation
            if !self.use_punctuation {
                if let Some(use_punctuation) = persistence.get("use_punctuation") {
                    let val = match use_punctuation {
                        "false" => false,
                        "true" => true,
                        _ => false,
                    };
                    self.set_punctuation(val);
                }
            }

            // Live WPM
            if !self.hide_live_wpm {
                if let Some(hide_live_wpm) = persistence.get("hide_live_wpm") {
                    let val = matches!(hide_live_wpm, "true");
                    self.set_live_wpm(val)
                }
            }

            // Cursorline
            if !self.hide_cursorline {
                if let Some(hide_cursorline) = persistence.get("hide_cursorline") {
                    let val = matches!(hide_cursorline, "true");
                    self.set_cursorline(val)
                }
            }

            // Results Style
            if self.results_style.is_none() {
                if let Some(results_style) = persistence.get("results_style") {
                    if results_style.parse::<ResultsStyle>().is_ok() {
                        self.results_style = Some(results_style.to_string());
                    }
                } else {
                    self.results_style = Some(DEFAULT_RESULTS_STYLE.to_string())
                }
            }

            self.persistent = Some(persistence);
        }
    }

    pub fn try_parse() -> Result<Self, clap::Error> {
        let mut config = <Self as Parser>::try_parse()?;
        Self::override_with_persistence(&mut config);
        Ok(config)
    }

    /// Resolves the mode based onf the provided arguments
    /// Defaults to time mode with (30) seconds if no options are provided.
    /// If *both* `time` and `word` mode are passed, it will default to time mode.
    pub fn current_mode(&self) -> Mode {
        if let Some(words) = self.words.clone() {
            Mode::Words {
                count: words.split_ascii_whitespace().count(),
            }
        } else {
            match (self.time, self.word_count) {
                (Some(time), None) => Mode::Time { duration: time },
                (None, Some(count)) => Mode::Words { count },
                (None, None) => Mode::Time { duration: 30 },
                _ => unreachable!("Both Time mode and Words mode cannot be used at the same time."),
            }
        }
    }

    pub fn current_mode_type(&self) -> ModeType {
        match self.current_mode() {
            Mode::Time { .. } => ModeType::Time,
            Mode::Words { .. } => ModeType::Words,
        }
    }

    /// Changes the mode of the game.
    pub fn change_mode(&mut self, mode: ModeType, value: Option<usize>) {
        match mode {
            ModeType::Time => {
                self.word_count = None;
                self.time = Some(value.unwrap_or(DEFAULT_TIME_MODE_DURATION) as u64);
                if let Some(persistence) = &mut self.persistent {
                    let _ = persistence.set("mode", "Time");
                    let _ = persistence.set("mode_value", &value.unwrap_or(30).to_string());
                }
            }
            ModeType::Words => {
                self.time = None;
                self.word_count = Some(value.unwrap_or(DEFAULT_WORD_MODE_COUNT));
                if let Some(persistence) = &mut self.persistent {
                    let _ = persistence.set("mode", "Words");
                    let _ = persistence.set(
                        "mode_value",
                        &value.unwrap_or(DEFAULT_WORD_MODE_COUNT).to_string(),
                    );
                }
            }
        }
    }

    /// Chages the current theme of the game.
    pub fn change_theme(&mut self, theme_name: &str) {
        self.theme = Some(theme_name.to_string());
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("theme", theme_name);
        }
    }

    /// Changes the language if available.
    pub fn change_language(&mut self, lang: &str) -> bool {
        if Builder::has_language(lang) {
            self.language = Some(lang.to_string());
            if let Some(persistence) = &mut self.persistent {
                let _ = persistence.set("language", lang);
            }
            true
        } else {
            false
        }
    }

    /// Chages the number of visible lines in the test.
    pub fn change_visible_lines(&mut self, lines: u8) {
        self.visible_lines = lines;
    }

    /// Chages the current style of the cursor.
    pub fn change_cursor_style(&mut self, style: &str) {
        self.cursor_style = Some(style.to_string());
        // TODO: there must be a better way to do this
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("cursor", style);
        }
    }

    /// Resets the words flag after a test has been run with it.
    pub fn reset_words_flag(&mut self) {
        self.words = None;
    }

    /// Changes the value of the current mode.
    pub fn change_mode_value(&mut self, value: usize) {
        match self.current_mode() {
            Mode::Time { .. } => self.time = Some(value as u64),
            Mode::Words { .. } => self.word_count = Some(value),
        }
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("mode_value", &value.to_string());
        }
    }

    fn resolve_mode_from_str(&self, mode: &str) -> Option<ModeType> {
        match mode {
            "Time" => Some(ModeType::Time),
            "Words" => Some(ModeType::Words),
            _ => None,
        }
    }

    /// Resolves the test word count based on current configuration.
    pub fn resolve_word_count(&self) -> usize {
        if let Some(word_count) = self.word_count {
            word_count
        } else if let Some(duration) = self.time {
            let estimated_wc = (duration as f64 * WPS_TARGET).ceil() as usize;
            std::cmp::max(estimated_wc, DEFAULT_WORD_MODE_COUNT)
        } else {
            let estimated_wc = (DEFAULT_WORD_MODE_COUNT as f64 * WPS_TARGET).ceil() as usize;
            std::cmp::max(estimated_wc, DEFAULT_WORD_MODE_COUNT)
        }
    }

    /// Resolves the test duration based on current configuration.
    pub fn resolve_duration(&self) -> u64 {
        match (self.time, self.word_count) {
            (Some(duration), None) => duration,
            _ => DEFAULT_TIME_MODE_DURATION as u64,
        }
    }

    /// Resolves the cursor style based on current configuration.
    pub fn resolve_current_cursor_style(&self) -> SetCursorStyle {
        if let Some(style) = &self.cursor_style {
            match style.as_str() {
                "beam" => SetCursorStyle::SteadyBar,
                "block" => SetCursorStyle::DefaultUserShape,
                "underline" => SetCursorStyle::SteadyUnderScore,
                "blinking-beam" => SetCursorStyle::BlinkingBar,
                "blinking-block" => SetCursorStyle::BlinkingBlock,
                "blinking-underline" => SetCursorStyle::BlinkingUnderScore,
                _ => SetCursorStyle::BlinkingBar, // default to beam style
            }
        } else {
            SetCursorStyle::BlinkingBar
        }
    }

    pub fn resolve_cursor_style_from_name(&self, name: &str) -> SetCursorStyle {
        match name {
            "beam" => SetCursorStyle::SteadyBar,
            "block" => SetCursorStyle::DefaultUserShape,
            "underline" => SetCursorStyle::SteadyUnderScore,
            "blinking-beam" => SetCursorStyle::BlinkingBar,
            "blinking-block" => SetCursorStyle::BlinkingBlock,
            "blinking-underline" => SetCursorStyle::BlinkingUnderScore,
            _ => SetCursorStyle::SteadyBar,
        }
    }

    pub fn resolve_cursor_name_from_style(&self, style: &Option<SetCursorStyle>) -> &str {
        if let Some(style) = style {
            match style {
                SetCursorStyle::SteadyBar => "beam",
                SetCursorStyle::DefaultUserShape => "block",
                SetCursorStyle::SteadyUnderScore => "underline",
                SetCursorStyle::BlinkingBar => "blinking-beam",
                SetCursorStyle::BlinkingBlock => "blinking-block",
                SetCursorStyle::BlinkingUnderScore => "blinking-underline",
                _ => "beam",
            }
        } else {
            "Not found."
        }
    }
    /// FIXME(ema): this setter are getting out of hand. This must be refactored ASAP
    fn set_numbers(&mut self, val: bool) {
        self.use_numbers = val;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("use_numbers", val.to_string().as_str());
        }
    }

    fn set_symbols(&mut self, val: bool) {
        self.use_symbols = val;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("use_symbols", val.to_string().as_str());
        }
    }

    fn set_punctuation(&mut self, val: bool) {
        self.use_punctuation = val;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("use_punctuation", val.to_string().as_str());
        }
    }

    fn set_live_wpm(&mut self, val: bool) {
        self.hide_live_wpm = val;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("hide_live_wpm", val.to_string().as_str());
        }
    }

    fn set_cursorline(&mut self, val: bool) {
        self.hide_cursorline = val;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("hide_cursorline", val.to_string().as_str());
        }
    }

    /// Toggles the presence of numbers in the test word pool.
    pub fn toggle_numbers(&mut self) {
        let val = !self.use_numbers;
        self.use_numbers = val;
        self.set_numbers(val);
    }

    /// Toggles the presence of punctuation in the test word pool.
    pub fn toggle_punctuation(&mut self) {
        let val = !self.use_punctuation;
        self.use_punctuation = val;
        self.set_punctuation(val);
    }

    /// Toggles the presence of symbols in the test word pool.
    pub fn toggle_symbols(&mut self) {
        let val = !self.use_symbols;
        self.use_symbols = val;
        self.set_symbols(val);
    }

    /// Toggles the FPS display.
    pub fn toggle_fps(&mut self) {
        self.show_fps = !self.show_fps;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("show_fps", self.show_fps.to_string().as_str());
        }
    }

    /// Toggles the live WPM display.
    pub fn toggle_live_wpm(&mut self) {
        let val = !self.hide_live_wpm;
        self.hide_live_wpm = val;
        self.set_live_wpm(val);
    }

    /// Toggles the monochromatic results display.
    pub fn toggle_monochromatic_results(&mut self) {
        self.monocrhomatic_results = !self.monocrhomatic_results;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set(
                "monocrhomatic_results",
                self.monocrhomatic_results.to_string().as_str(),
            );
        }
    }

    /// Toggles the cursorline display in menus.
    pub fn toggle_cursorline(&mut self) {
        self.hide_cursorline = !self.hide_cursorline;
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("hide_cursorline", self.hide_cursorline.to_string().as_str());
        }
    }

    /// Chesk if the current terminal has proper color support. Mainly used for themes
    pub fn term_has_color_support(&self) -> bool {
        let color_support = self
            .color_mode
            .as_deref()
            .and_then(|s| ColorSupport::from_str(s).ok())
            .unwrap_or_else(Theme::detect_color_support);
        color_support.supports_themes()
    }

    /// Changes the ascii art shown on the results screen.
    pub fn change_ascii_art(&mut self, art_name: &str) {
        self.ascii = Some(art_name.to_string());
        if let Some(persistence) = &mut self.persistent {
            let _ = persistence.set("ascii", art_name.to_string().as_str());
        }
    }

    /// Changes the picker style for menus.
    pub fn change_picker_style(&mut self, style: &str) {
        if style.parse::<PickerStyle>().is_ok() {
            self.picker_style = Some(style.to_string());
            if let Some(persistence) = &mut self.persistent {
                let _ = persistence.set("picker_style", style);
            }
        }
    }

    /// Resolves the current picker style.
    pub fn resolve_picker_style(&self) -> PickerStyle {
        self.picker_style
            .as_deref()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }

    /// Changes the results style for the results screen.
    pub fn change_results_style(&mut self, style: &str) {
        if style.parse::<ResultsStyle>().is_ok() {
            self.results_style = Some(style.to_string());
            if let Some(persistence) = &mut self.persistent {
                let _ = persistence.set("results_style", style);
            }
        }
    }

    /// Resolves the current results style.
    pub fn resolve_results_style(&self) -> ResultsStyle {
        self.results_style
            .as_deref()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PickerStyle {
    Quake,     // Opens from the top a la quake terminal style, hence the name
    Telescope, // Floating menu just like Telescopic johnson does
    Ivy,       // Opens from the bottom
    Minimal,   // Telescope style picker without preview folds/splits
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickerStyleParseError {
    pub invalid_input: String,
}

impl std::fmt::Display for PickerStyleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid picker style: '{}'. Valid options are: quake, telescope, ivy, minimal",
            self.invalid_input
        )
    }
}

impl std::error::Error for PickerStyleParseError {}

impl FromStr for PickerStyle {
    type Err = PickerStyleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quake" => Ok(Self::Quake),
            "telescope" => Ok(Self::Telescope),
            "ivy" => Ok(Self::Ivy),
            "minimal" => Ok(Self::Minimal),
            _ => Err(PickerStyleParseError {
                invalid_input: s.to_string(),
            }),
        }
    }
}

impl PickerStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Quake => "quake",
            Self::Telescope => "telescope",
            Self::Ivy => "ivy",
            Self::Minimal => "minimal",
        }
    }

    pub fn all() -> &'static [&'static str] {
        &["quake", "ivy", "telescope", "minimal"]
    }

    pub fn label_from_str(label: &str) -> &'static str {
        match label {
            "quake" => "Quake",
            "telescope" => "Telescope",
            "ivy" => "Ivy",
            "minimal" => "Minimal",
            _ => "Wrong picker",
        }
    }
}

impl Default for PickerStyle {
    fn default() -> Self {
        Self::Quake
    }
}

// TODO: is annoying having to recreate this over and over again. Will probablly be best
// to have some of `style.rs` that takes care of the boiiler plate. It's mostly the same for all styles
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ResultsStyle {
    Graph,
    Neofetch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResultsStyleParseError {
    pub invalid_input: String,
}

impl std::fmt::Display for ResultsStyleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid results style: '{}'. Valid options are: neofetch, graph",
            self.invalid_input
        )
    }
}

impl std::error::Error for ResultsStyleParseError {}

impl FromStr for ResultsStyle {
    type Err = ResultsStyleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "graph" => Ok(Self::Graph),
            "neofetch" => Ok(Self::Neofetch),
            _ => Err(ResultsStyleParseError {
                invalid_input: s.to_string(),
            }),
        }
    }
}

impl ResultsStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Graph => "graph",
            Self::Neofetch => "neofetch",
        }
    }

    pub fn all() -> &'static [&'static str] {
        &["graph", "neofetch"]
    }

    pub fn label_from_str(label: &str) -> &'static str {
        match label {
            "graph" => "Graph",
            "neofetch" => "Neofetch",
            _ => "Unknown style",
        }
    }
}

impl Default for ResultsStyle {
    fn default() -> Self {
        Self::Graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_default_state() {
        let config = Config::default();
        assert!(config.time.is_some());
        assert!(config.word_count.is_none());

        assert_eq!(config.language, Some(DEFAULT_LANGUAGE.to_string()));
        assert_eq!(config.theme, None);
        assert_eq!(config.visible_lines, DEFAULT_LINE_COUNT);

        assert!(!config.use_symbols);
        assert!(!config.use_punctuation);
        #[cfg(debug_assertions)]
        assert!(!config.debug);
        assert!(!config.show_fps);
    }

    fn assert_mode(config: &Config, expected_mode: ModeType, expected_value: usize) {
        match config.current_mode() {
            Mode::Time { duration } if matches!(expected_mode, ModeType::Time) => {
                assert_eq!(duration as usize, expected_value)
            }
            Mode::Words { count } if matches!(expected_mode, ModeType::Words) => {
                assert_eq!(count, expected_value)
            }
            _ => panic!("Unexpected mode"),
        }
    }

    #[test]
    fn test_config_change_mode() {
        let mut config = create_config();
        config.change_mode(ModeType::Time, Some(30));
        assert!(config.word_count.is_none());
        assert_mode(&config, ModeType::Time, 30);
    }

    #[test]
    fn test_config_change_theme() {
        let mut config = create_config();
        let theme_name = "Monokai Classic";
        config.change_theme(theme_name);
        assert_eq!(config.theme, Some("Monokai Classic".to_string()));
    }

    #[test]
    fn test_config_change_visible_lines() {
        let mut config = create_config();
        assert_eq!(config.visible_lines, DEFAULT_LINE_COUNT);
        config.change_visible_lines(10);
        assert_eq!(config.visible_lines, 10);
    }

    #[test]
    fn test_config_toggles() {
        let mut config = create_config();
        assert!(!config.use_numbers);
        assert!(!config.use_punctuation);
        assert!(!config.use_symbols);

        config.toggle_numbers();
        assert!(config.use_numbers);

        config.toggle_punctuation();
        assert!(config.use_punctuation);

        config.toggle_symbols();
        assert!(config.use_symbols);
    }

    #[test]
    fn test_config_live_wpm() {
        let config = create_config();
        assert!(!config.hide_live_wpm) // we default this to false (show live WPM)
    }

    #[test]
    fn test_config_resolvers() {
        let mut config = create_config();
        config.change_mode(ModeType::Time, Some(45));
        assert_eq!(config.resolve_duration(), 45);
        config.change_mode(ModeType::Words, Some(75));
        assert_eq!(config.resolve_word_count(), 75);
    }

    #[test]
    fn test_config_resolve_cursor_style() {
        let mut config = create_config();

        // the default
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingBar
        );

        config.cursor_style = Some("beam".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::SteadyBar
        );

        config.cursor_style = Some("block".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::DefaultUserShape
        );

        config.cursor_style = Some("underline".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::SteadyUnderScore
        );

        config.cursor_style = Some("blinking-beam".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingBar
        );

        config.cursor_style = Some("blinking-block".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingBlock
        );

        config.cursor_style = Some("blinking-underline".to_string());
        matches!(
            config.resolve_current_cursor_style(),
            SetCursorStyle::BlinkingUnderScore
        );
    }

    #[test]
    fn test_picker_style_functionality() {
        let mut config = create_config();

        assert_eq!(config.resolve_picker_style(), PickerStyle::Quake);

        config.change_picker_style("telescope");
        assert_eq!(config.resolve_picker_style(), PickerStyle::Telescope);

        config.change_picker_style("ivy");
        assert_eq!(config.resolve_picker_style(), PickerStyle::Ivy);

        config.change_picker_style("quake");
        assert_eq!(config.resolve_picker_style(), PickerStyle::Quake);

        config.change_picker_style("invalid");
        assert_eq!(config.resolve_picker_style(), PickerStyle::Quake);
    }

    #[test]
    fn test_picker_style_from_str() {
        assert_eq!("quake".parse::<PickerStyle>(), Ok(PickerStyle::Quake));
        assert_eq!(
            "telescope".parse::<PickerStyle>(),
            Ok(PickerStyle::Telescope)
        );
        assert_eq!("ivy".parse::<PickerStyle>(), Ok(PickerStyle::Ivy));
        assert_eq!("minimal".parse::<PickerStyle>(), Ok(PickerStyle::Minimal));
        assert_eq!("QUAKE".parse::<PickerStyle>(), Ok(PickerStyle::Quake));
        assert!("invalid".parse::<PickerStyle>().is_err());
    }
}
