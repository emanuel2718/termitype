use crate::tui::utils::{create_target_text_line, wrap_text};
use crate::{app::App, theme, tracker::Tracker};
use anyhow::Result;
use ratatui::style::Modifier;
use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

pub fn draw_ui(frame: &mut Frame, app: &mut App) -> Result<()> {
    let area = frame.area();
    let theme = theme::current_theme();

    let (_content_area, centered_area) = create_layout(area);

    let mut lines = Vec::new();

    if app.tracker.is_typing() || app.tracker.is_complete() {
        render_typing_state(&mut lines, app, &theme, centered_area.width);
    } else {
        render_initial_state(&mut lines, app, centered_area.width);
    }

    let padding_top = calculate_padding(&lines, centered_area.height);
    for _ in 0..padding_top {
        lines.insert(0, Line::from(""));
    }

    let paragraph = Paragraph::new(lines).style(
        Style::default()
            .bg(theme.bg())
            .fg(theme.fg())
            .add_modifier(Modifier::DIM),
    );

    let bg_block = Block::default().style(Style::default().bg(theme.bg()));
    frame.render_widget(bg_block, area);

    frame.render_widget(paragraph, centered_area);

    if !app.tracker.is_complete() {
        set_cursor_position(frame, app, &theme, centered_area, padding_top);
    }

    Ok(())
}

fn create_layout(area: Rect) -> (Rect, Rect) {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);

    let content_area = vertical_layout[1];

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(content_area);

    let centered_area = horizontal_layout[1];
    (content_area, centered_area)
}

fn render_typing_state(lines: &mut Vec<Line>, app: &mut App, theme: &theme::Theme, max_width: u16) {
    let target_lines = create_target_text_line(&app.tracker, theme, max_width);
    let visible_lines = calculate_visible_lines(&target_lines, app);
    lines.extend(visible_lines);

    if app.tracker.is_complete() {
        render_completion(lines, &mut app.tracker, max_width);
    }
}

fn calculate_visible_lines(target_lines: &[Line<'static>], app: &App) -> Vec<Line<'static>> {
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
    let visible_start = cursor_line.saturating_sub(line_count - 1);
    let visible_end = (visible_start + line_count).min(target_lines.len());
    target_lines[visible_start..visible_end].to_vec()
}

fn render_completion(lines: &mut Vec<Line>, tracker: &mut Tracker, max_width: u16) {
    let summary = tracker.summary();
    let completion_text = format!(
        "Test Complete! Final WPM: {:.1}, Accuracy: {:.1}%",
        summary.wpm,
        summary.accuracy * 100.0
    );
    let completion_lines = wrap_text(&completion_text, max_width);
    lines.push(Line::from(""));
    lines.extend(completion_lines);
}

fn render_initial_state(lines: &mut Vec<Line>, app: &App, max_width: u16) {
    let wrapped_lines = wrap_text(&app.lexicon.words, max_width);
    let visible_lines = wrapped_lines
        .into_iter()
        .take(app.config.current_line_count() as usize)
        .collect::<Vec<_>>();
    lines.extend(visible_lines);
}

fn calculate_padding(lines: &[Line], height: u16) -> usize {
    let num_lines = lines.len();
    let height = height as usize;
    if num_lines < height {
        (height - num_lines) / 2
    } else {
        0
    }
}

fn set_cursor_position(
    frame: &mut Frame,
    app: &App,
    theme: &theme::Theme,
    centered_area: Rect,
    padding_top: usize,
) {
    let target_lines = create_target_text_line(&app.tracker, theme, centered_area.width);
    let mut cumulative = 0;
    let mut cursor_x = 0;
    let mut cursor_y = 0;
    for (i, line) in target_lines.iter().enumerate() {
        let line_len = line.spans.iter().map(|s| s.content.len()).sum::<usize>();
        if app.tracker.current_pos < cumulative + line_len {
            cursor_y = i;
            cursor_x = (app.tracker.current_pos - cumulative) as u16;
            break;
        }
        cumulative += line_len;
    }
    let line_count = app.config.current_line_count() as usize;
    let visible_start = cursor_y.saturating_sub(line_count - 1);
    let visible_cursor_y = cursor_y - visible_start;
    frame.set_cursor_position(Position {
        x: centered_area.x + cursor_x,
        y: centered_area.y + padding_top as u16 + visible_cursor_y as u16,
    });
}
