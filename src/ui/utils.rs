use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::{
    config,
    constants::{MENU_HEIGHT, MIN_THEME_PREVIEW_WIDTH},
    termi::Termi,
};

// TODO: maybe having this file is not entirely needed question mark
#[derive(Debug, Clone, Copy)]
pub struct WordPosition {
    pub start_index: usize,
    pub line: usize,
    pub col: usize,
}

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

/// Calculate the actual menu area that will be rendered, considering picker style and menu type
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

/// Core menu area calculation logic
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
