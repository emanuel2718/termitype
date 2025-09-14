use crate::{app::App, tui::layout::AppLayout};
use ratatui::{layout::Position, text::Line, widgets::Padding, Frame};

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
    // Add offset for header lines (stats or language + empty line)
    let header_offset = 2;
    frame.set_cursor_position(Position {
        x: layout.center_area.x + cursor_x,
        y: layout.center_area.y + pad_size as u16 + visible_cursor_y as u16 + header_offset as u16,
    });
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

pub fn max_line_width(lines: &[ratatui::text::Line]) -> u16 {
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

pub fn calculate_horizontal_padding(content_width: u16, total_width: u16) -> (u16, u16) {
    let left_pad = total_width.saturating_sub(content_width) / 2;
    let right_pad = total_width.saturating_sub(content_width + left_pad);
    (left_pad, right_pad)
}

pub fn center_lines_vertically(lines: Vec<ratatui::text::Line<'static>>, height: u16) -> Vec<ratatui::text::Line<'static>> {
    let padding_top = super::layout::calculate_padding(&lines, height);
    let padding_bottom = (height as usize).saturating_sub(lines.len()).saturating_sub(padding_top);
    let mut result = Vec::with_capacity(height as usize);
    result.extend((0..padding_top).map(|_| ratatui::text::Line::from("")));
    result.extend(lines);
    result.extend((0..padding_bottom).map(|_| ratatui::text::Line::from("")));
    result
}
