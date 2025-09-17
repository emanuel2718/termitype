use crate::{app::App, tui::layout::AppLayout};
use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    text::Line,
    widgets::Padding,
    Frame,
};

pub fn calculate_visible_lines(
    target_lines: &[Line<'static>],
    app: &crate::app::App,
) -> Vec<Line<'static>> {
    let cursor_line = {
        let mut cumulative = 0;
        let mut line = 0;
        for (i, l) in target_lines.iter().enumerate() {
            let line_len = l.spans.iter().map(|s| s.content.len()).sum::<usize>();
            if app.tracker.current_pos < cumulative + line_len {
                line = i;
                break;
            }
            cumulative += line_len;
        }
        line
    };
    let line_count = app.config.current_line_count() as usize;
    let scroll_offset = (line_count - 1).saturating_sub(1);
    let visible_start = cursor_line.saturating_sub(scroll_offset);
    let visible_end = (visible_start + line_count).min(target_lines.len());
    target_lines[visible_start..visible_end].to_vec()
}

pub fn set_cursor_position(
    frame: &mut Frame,
    app: &mut App,
    lines: &Vec<Line>,
    layout: &AppLayout,
    pad_size: usize,
) {
    let mut cumulative = 0;
    let mut cursor_x = 0;
    let mut cursor_y = 0;
    for (i, line) in lines.iter().enumerate() {
        let line_len = line.spans.iter().map(|s| s.content.len()).sum::<usize>();
        if app.tracker.current_pos < cumulative + line_len {
            cursor_y = i;
            cursor_x = (app.tracker.current_pos - cumulative) as u16;
            break;
        }
        cumulative += line_len;
    }
    let line_count = app.config.current_line_count() as usize;
    let scroll_offset = (line_count - 1).saturating_sub(1);
    let visible_start = cursor_y.saturating_sub(scroll_offset);
    let visible_cursor_y = cursor_y - visible_start;
    let header_offset = 2;

    // dont render the cursor if the menu is open or we should be showing the results
    if !app.menu.is_open() && !app.tracker.is_complete() && !app.tracker.should_complete() {
        frame.set_cursor_position(Position {
            x: layout.center_area.x + cursor_x,
            y: layout.center_area.y
                + pad_size as u16
                + visible_cursor_y as u16
                + header_offset as u16,
        });
    }
}

pub fn wrap_text(text: &str, max_width: u16) -> Vec<Line<'static>> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in words {
        let potential = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };
        if potential.len() as u16 > max_width {
            if !current_line.is_empty() {
                lines.push(Line::from(current_line));
                current_line = word.to_string();
            } else {
                // the word is longer than `max_width`, add as is (may wrap)
                lines.push(Line::from(word.to_string()));
            }
        } else {
            current_line = potential;
        }
    }
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }
    lines
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn calculate_padding(lines: &[Line], height: u16) -> usize {
    let num_lines = lines.len();
    let height = height as usize;
    if num_lines < height {
        (height - num_lines) / 2
    } else {
        0
    }
}
pub fn title_padding() -> Padding {
    Padding {
        left: 8,
        right: 0,
        top: 2,
        bottom: 0,
    }
}

pub fn mode_line_padding() -> Padding {
    Padding {
        left: 0,
        right: 0,
        top: 6,
        bottom: 0,
    }
}

pub fn footer_padding() -> Padding {
    Padding {
        left: 0,
        right: 1,
        top: 0,
        bottom: 0,
    }
}

pub fn menu_items_padding() -> Padding {
    Padding {
        left: 0,
        right: 1,
        top: 0,
        bottom: 0,
    }
}

pub fn max_line_width(lines: &[Line]) -> u16 {
    lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|s| s.content.chars().count())
                .sum::<usize>() as u16
        })
        .max()
        .unwrap_or(0)
}

pub fn horizontally_center(area: Rect, target_width: u16) -> Rect {
    let width = target_width.min(area.width);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    Rect {
        x,
        y: area.y,
        width,
        height: area.height,
    }
}

pub fn calculate_horizontal_padding(content_width: u16, total_width: u16) -> (u16, u16) {
    let left_pad = total_width.saturating_sub(content_width) / 2;
    let right_pad = total_width.saturating_sub(content_width + left_pad);
    (left_pad, right_pad)
}

pub fn center_lines_vertically(lines: Vec<Line<'static>>, height: u16) -> Vec<Line<'static>> {
    let padding_top = calculate_padding(&lines, height);
    let padding_bottom = (height as usize)
        .saturating_sub(lines.len())
        .saturating_sub(padding_top);
    let mut result = Vec::with_capacity(height as usize);
    result.extend((0..padding_top).map(|_| Line::from("")));
    result.extend(lines);
    result.extend((0..padding_bottom).map(|_| Line::from("")));
    result
}
