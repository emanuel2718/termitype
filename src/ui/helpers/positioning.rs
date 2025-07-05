use crate::{
    constants::{MENU_HEIGHT, MIN_THEME_PREVIEW_WIDTH},
    styles,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Text,
};

pub struct LayoutHelper;

impl LayoutHelper {
    /// Do you know how to center a div question mark
    pub fn center_div(width: u16, height: u16, parent: Rect) -> Rect {
        let parent_w = parent.width;
        let parent_h = parent.height;

        let width = width.min(parent_w);
        let height = height.min(parent_h);

        let x = parent.x + (parent_w.saturating_sub(width)) / 2;
        let y = parent.y + (parent_h.saturating_sub(height)) / 2;

        Rect::new(x, y, width, height)
    }

    /// Center a rect within another
    pub fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        Rect {
            x,
            y,
            width: width.min(area.width),
            height: height.min(area.height),
        }
    }

    /// Center text content within an area
    pub fn center_text_rect(area: Rect, text: &Text) -> Rect {
        let text_height = text.height() as u16;

        Rect {
            x: area.x,
            y: area.y + (area.height.saturating_sub(text_height)) / 2,
            width: area.width,
            height: area.height,
        }
    }

    /// Center rect within a max width
    pub fn center_with_max_width(area: Rect, max_width: u16) -> Rect {
        let width = area.width.min(max_width);
        let x_offset = (area.width.saturating_sub(width)) / 2;

        Rect {
            x: area.x + x_offset,
            y: area.y,
            width,
            height: area.height,
        }
    }

    pub fn calculate_menu_area_from_parts(
        picker_style: styles::PickerStyle,
        is_theme_picker: bool,
        is_help_menu: bool,
        is_about_menu: bool,
        is_ascii_art_picker: bool,
        area: Rect,
    ) -> Rect {
        let small_width = area.width <= MIN_THEME_PREVIEW_WIDTH;
        let menu_height =
            if (is_theme_picker || is_help_menu || is_about_menu || is_ascii_art_picker)
                && small_width
            {
                area.height
            } else {
                MENU_HEIGHT.min(area.height)
            };

        match picker_style {
            // top
            styles::PickerStyle::Quake => Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: menu_height,
            },
            // floating
            styles::PickerStyle::Telescope => {
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
            styles::PickerStyle::Minimal => {
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
            styles::PickerStyle::Ivy => {
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

    /// Calculate clickable area for text with specific alignment
    pub fn clickable_text_area(
        area: Rect,
        text_width: u16,
        text_height: u16,
        alignment: Alignment,
    ) -> Rect {
        match alignment {
            Alignment::Center => {
                let start_x = area.x + (area.width.saturating_sub(text_width)) / 2;
                Rect {
                    x: start_x,
                    y: area.y,
                    width: text_width.min(area.width) + 1,
                    height: text_height.min(area.height),
                }
            }
            Alignment::Right => {
                let start_x = area.x + area.width.saturating_sub(text_width);
                Rect {
                    x: start_x,
                    y: area.y,
                    width: text_width.min(area.width) + 1,
                    height: text_height.min(area.height),
                }
            }
            Alignment::Left => Rect {
                x: area.x,
                y: area.y,
                width: text_width.min(area.width) + 1,
                height: text_height.min(area.height),
            },
        }
    }

    /// Calculate scroll offset for keeping cursor in view
    pub fn calculate_scroll_offset(cursor_line: usize, visible_lines: usize) -> usize {
        if visible_lines <= 1 {
            cursor_line
        } else {
            let half_visible = visible_lines >> 1;
            cursor_line.saturating_sub(half_visible)
        }
    }

    /// Apply horizontal centering to an area with percentage width
    pub fn apply_horizontal_centering(area: Rect, width_percentage: u16) -> Rect {
        let padding = (100 - width_percentage) / 2;
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(padding),
                Constraint::Percentage(width_percentage),
                Constraint::Percentage(padding),
            ])
            .split(area)[1]
    }
}
