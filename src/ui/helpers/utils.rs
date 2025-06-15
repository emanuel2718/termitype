use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::TermiStyle;

pub struct TermiUtils;

impl TermiUtils {
    pub fn hash_text(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_symbols(supports_unicode: bool) -> TermiSymbols {
        if supports_unicode {
            TermiSymbols::unicode()
        } else {
            TermiSymbols::ascii()
        }
    }

    pub fn has_minimum_size(area: ratatui::layout::Rect, min_width: u16, min_height: u16) -> bool {
        area.width >= min_width && area.height >= min_height
    }

    pub fn truncate_text(text: &str, max_width: usize) -> String {
        if text.len() <= max_width {
            text.to_string()
        } else if max_width <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &text[..max_width - 3])
        }
    }

    pub fn create_menu_footer_text(termi: &crate::termi::Termi) -> ratatui::text::Line {
        use ratatui::{
            style::{Modifier, Style},
            text::{Line, Span},
        };

        let theme = termi.current_theme();
        let menu_state = &termi.menu;
        let symbols = Self::get_symbols(theme.supports_unicode());

        if menu_state.is_searching() {
            Line::from(vec![
                Span::styled("Filter: ", Style::default().fg(theme.accent())),
                Span::styled(menu_state.search_query(), Style::default().fg(theme.fg())),
                Span::styled(
                    symbols.cursor,
                    Style::default()
                        .fg(theme.cursor())
                        .add_modifier(Modifier::RAPID_BLINK),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(
                    format!("[{}/k]", symbols.up_arrow),
                    TermiStyle::highlight(theme),
                ),
                Span::styled(" up ", TermiStyle::muted(theme)),
                Span::styled(
                    format!("[{}/j]", symbols.down_arrow),
                    TermiStyle::highlight(theme),
                ),
                Span::styled(" down ", TermiStyle::muted(theme)),
                Span::styled("[/]", TermiStyle::highlight(theme)),
                Span::styled(" search ", TermiStyle::muted(theme)),
                Span::styled("[ent]", TermiStyle::highlight(theme)),
                Span::styled(" sel ", TermiStyle::muted(theme)),
                Span::styled("[space]", TermiStyle::highlight(theme)),
                Span::styled(" toggle ", TermiStyle::muted(theme)),
                Span::styled("[esc]", TermiStyle::highlight(theme)),
                Span::styled(" close", TermiStyle::muted(theme)),
            ])
        }
    }

    pub fn create_results_footer_text(theme: &crate::theme::Theme) -> ratatui::text::Line {
        use ratatui::{
            layout::Alignment,
            style::Style,
            text::{Line, Span},
        };

        Line::from(vec![
            Span::styled("[N]", TermiStyle::warning(theme)),
            Span::styled("ew", TermiStyle::muted(theme)),
            Span::styled(" ", Style::default()),
            Span::styled("[R]", TermiStyle::warning(theme)),
            Span::styled("edo", TermiStyle::muted(theme)),
            Span::styled(" ", Style::default()),
            Span::styled("[Q]", TermiStyle::warning(theme)),
            Span::styled("uit", TermiStyle::muted(theme)),
            Span::styled(" ", Style::default()),
            Span::styled("[ESC]", TermiStyle::warning(theme)),
            Span::styled(" menu", TermiStyle::muted(theme)),
        ])
        .alignment(Alignment::Center)
    }
}

pub struct TermiSymbols {
    pub punctuation: &'static str,
    pub numbers: &'static str,
    pub symbols: &'static str,
    pub divider: &'static str,
    pub custom: &'static str,
    pub info: &'static str,
    pub cursor: &'static str,
    pub up_arrow: &'static str,
    pub down_arrow: &'static str,
    pub arrow: &'static str,
    pub submenu_arrow: &'static str,
}

impl TermiSymbols {
    pub fn unicode() -> Self {
        Self {
            punctuation: "!",
            numbers: "#",
            symbols: "@",
            divider: "│",
            custom: "⚙",
            info: "ⓘ",
            cursor: "█",
            up_arrow: "↑",
            down_arrow: "↓",
            arrow: "❯ ",
            submenu_arrow: " →",
        }
    }

    pub fn ascii() -> Self {
        Self {
            punctuation: "P",
            numbers: "N",
            symbols: "S",
            divider: "|",
            custom: "<c>",
            info: "i",
            cursor: "_",
            up_arrow: "^",
            down_arrow: "v",
            arrow: "> ",
            submenu_arrow: " >",
        }
    }
}

/// Cache for UI calcs
pub struct UiCache {
    pub last_word_positions: Option<Vec<WordPosition>>,
    pub last_text_hash: u64,
    pub last_width: usize,
}

impl UiCache {
    pub fn new() -> Self {
        Self {
            last_word_positions: None,
            last_text_hash: 0,
            last_width: 0,
        }
    }

    pub fn invalidate(&mut self) {
        self.last_word_positions = None;
        self.last_text_hash = 0;
        self.last_width = 0;
    }
}

impl Default for UiCache {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    pub static UI_CACHE: std::cell::RefCell<UiCache> = std::cell::RefCell::new(UiCache::new());
}

// TODO: maybe having this file is not entirely needed question mark
#[derive(Debug, Clone, Copy)]
pub struct WordPosition {
    pub start_index: usize,
    pub line: usize,
    pub col: usize,
}

use crate::{
    config,
    constants::{MENU_HEIGHT, MIN_THEME_PREVIEW_WIDTH},
    termi::Termi,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::collections::HashMap;

// PERF: cache for word positions
thread_local! {
    static WORD_POSITION_CACHE: std::cell::RefCell<HashMap<u64, Vec<WordPosition>>> =
        std::cell::RefCell::new(HashMap::new());
}

fn calculate_cache_key(text: &str, available_width: usize) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    available_width.hash(&mut hasher);
    hasher.finish()
}

pub fn center_div(width: u16, height: u16, parent: Rect) -> Rect {
    let parent_w = parent.width;
    let parent_h = parent.height;

    let width = width.min(parent_w);
    let height = height.min(parent_h);

    let x = parent.x + (parent_w.saturating_sub(width)) / 2;
    let y = parent.y + (parent_h.saturating_sub(height)) / 2;

    Rect::new(x, y, width, height)
}

pub fn apply_horizontal_centering(area: Rect, width_percent: u16) -> Rect {
    let padding = (100 - width_percent) / 2;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(padding),
            Constraint::Percentage(width_percent),
            Constraint::Percentage(padding),
        ])
        .split(area)[1]
}

pub fn calculate_word_positions(text: &str, available_width: usize) -> Vec<WordPosition> {
    let cache_key = calculate_cache_key(text, available_width);

    WORD_POSITION_CACHE.with(|cache| {
        let mut cache_ref = cache.borrow_mut();

        if let Some(cached_positions) = cache_ref.get(&cache_key) {
            return cached_positions.clone();
        }

        let positions = _calculate_positions(text, available_width);

        if cache_ref.len() > 20 {
            cache_ref.clear();
        }
        cache_ref.insert(cache_key, positions.clone());

        positions
    })
}

fn _calculate_positions(text: &str, available_width: usize) -> Vec<WordPosition> {
    if text.is_empty() || available_width == 0 {
        return vec![];
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut positions = Vec::with_capacity(words.len());
    let text_chars: Vec<char> = text.chars().collect();
    let mut char_index = 0;
    let mut line = 0;
    let mut col = 0;

    for (word_idx, word) in words.iter().enumerate() {
        // start position of the word in char indexes
        let word_chars: Vec<char> = word.chars().collect();
        while char_index < text_chars.len() {
            if char_index + word_chars.len() <= text_chars.len()
                && text_chars[char_index..char_index + word_chars.len()] == word_chars
            {
                break;
            }
            char_index += 1;
        }

        let word_len = word.chars().count();

        // does the word fit in the current line question mark
        if col > 0 && col + word_len > available_width {
            line += 1;
            col = 0;
        }

        positions.push(WordPosition {
            start_index: char_index,
            line,
            col,
        });

        char_index += word_len;
        col += word_len;

        if word_idx < words.len() - 1 && col < available_width {
            col += 1;
            char_index += 1; // skip the character space
        }
    }

    positions
}

pub fn calculate_menu_area(termi: &Termi, area: Rect) -> Rect {
    let picker_style = termi.config.resolve_picker_style();
    let menu_state = &termi.menu;

    let is_theme_picker = menu_state.is_theme_menu();
    let is_help_menu = menu_state.is_help_menu();
    let is_about_menu = menu_state.is_about_menu();
    let is_ascii_art_picker = menu_state.is_ascii_art_menu();

    calculate_menu_area_from_parts(
        picker_style,
        is_theme_picker,
        is_help_menu,
        is_about_menu,
        is_ascii_art_picker,
        area,
    )
}

pub fn calculate_menu_area_from_parts(
    picker_style: config::PickerStyle,
    is_theme_picker: bool,
    is_help_menu: bool,
    is_about_menu: bool,
    is_ascii_art_picker: bool,
    area: Rect,
) -> Rect {
    let small_width = area.width <= MIN_THEME_PREVIEW_WIDTH;
    let menu_height = if (is_theme_picker || is_help_menu || is_about_menu || is_ascii_art_picker)
        && small_width
    {
        area.height
    } else {
        MENU_HEIGHT.min(area.height)
    };

    match picker_style {
        // top
        config::PickerStyle::Quake => Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: menu_height,
        },
        // floating
        config::PickerStyle::Telescope => {
            let menu_width = (area.width as f32 * 0.90).min(95.0) as u16;
            let menu_height = (area.height as f32 * 0.6).min(menu_height as f32) as u16;
            let x = area.x + (area.width.saturating_sub(menu_width)) / 2;
            let y = area.y + (area.height.saturating_sub(menu_height)) / 2;
            Rect {
                x,
                y,
                width: menu_width,
                height: menu_height,
            }
        }
        // floating, no preview folds
        config::PickerStyle::Minimal => {
            let menu_width = (area.width as f32 * 0.90).min(95.0) as u16;
            let menu_height = (area.height as f32 * 0.6).min(menu_height as f32) as u16;
            let x = area.x + (area.width.saturating_sub(menu_width)) / 2;
            let y = area.y + (area.height.saturating_sub(menu_height)) / 2;
            Rect {
                x,
                y,
                width: menu_width,
                height: menu_height,
            }
        }
        // bottom
        config::PickerStyle::Ivy => {
            let y = area.y + area.height.saturating_sub(menu_height);
            Rect {
                x: area.x,
                y,
                width: area.width,
                height: menu_height,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_position_basic() {
        let text = "hello world ";
        let available_width = 20;
        let positions = calculate_word_positions(text, available_width);

        assert_eq!(positions.len(), 2, "Should have positions for two words ");
        assert_eq!(positions[0].start_index, 0, "First word starts at 0");
        assert_eq!(positions[0].line, 0, "First word on line 0");
        assert_eq!(positions[0].col, 0, "First word at column 0");

        assert_eq!(
            positions[1].start_index, 6,
            "Second word starts after \"hello \""
        );
        assert_eq!(positions[1].line, 0, "Second word on line 0");
        assert_eq!(positions[1].col, 6, "Second word after first word + space");
    }

    #[test]
    fn test_word_position_wrapping() {
        let text = "hello world wrap";
        let available_width = 8; // force wrap after "hello"
        let positions = calculate_word_positions(text, available_width);

        assert_eq!(positions[0].line, 0, "First word on line 0");
        assert_eq!(positions[1].line, 1, "Second word should wrap to line 1");
        assert_eq!(positions[1].col, 0, "Wrapped word starts at column 0");
        assert_eq!(positions[2].line, 2, "Third word on line 2");
    }

    #[test]
    fn test_cursor_positions() {
        let text = "hello world next";
        let available_width = 20;
        let positions = calculate_word_positions(text, available_width);

        let test_positions = vec![
            (0, 0, "Start of text"),
            (5, 0, "End of first word"),
            (6, 1, "Start of second word"),
            (11, 1, "End of second word"),
            (12, 2, "Start of third word"),
        ];

        for (cursor_pos, expected_word_idx, description) in test_positions {
            let current_pos = positions
                .iter()
                .rev()
                .find(|pos| cursor_pos >= pos.start_index)
                .unwrap();

            assert_eq!(
                positions
                    .iter()
                    .position(|p| p.start_index == current_pos.start_index)
                    .unwrap(),
                expected_word_idx,
                "{}",
                description
            );
        }
    }

    #[test]
    fn test_empty_text() {
        let text = "";
        let available_width = 10;
        let positions = calculate_word_positions(text, available_width);
        assert!(positions.is_empty(), "Empty text should have no positions");
    }
}
