use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{
    constants::{APPNAME, WINDOW_HEIGHT_PERCENT, WINDOW_WIDTH_PERCENT},
    termi::Termi,
};

pub fn draw_ui(f: &mut Frame, termi: &Termi) {
    let size = f.area();

    // 60% width and 50% height of the current trminal window size
    let area = centered_rect(
        WINDOW_WIDTH_PERCENT as u16,
        WINDOW_HEIGHT_PERCENT as u16,
        size,
    );

    let block = Block::bordered()
        .border_style(Style::new().fg(termi.theme.border))
        .borders(Borders::ALL)
        .title(Span::styled(
            APPNAME,
            Style::default()
                .fg(termi.theme.highlight)
                .add_modifier(Modifier::BOLD),
        ));

    let inner_area = block.inner(area);

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),    // Top spacer
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Target Text
                Constraint::Min(1),    // Bottom spacer
            ]
            .as_ref(),
        )
        .split(inner_area);

    f.render_widget(&block, area);

    f.render_widget(render_test_words(termi), vertical_layout[2]);
}

fn render_test_words(termi: &Termi) -> Paragraph {
    let mut text = Text::default().fg(termi.theme.inactive);
    let target_chars: Vec<char> = termi.words.chars().collect();

    let mut spans = Vec::new();

    // loop through all the characters in the test words and paint the appropiately
    for (i, &char) in target_chars.iter().enumerate() {
        let style = if let Some(input_char) = termi.tracker.user_input.get(i).and_then(|&x| x) {
            if input_char == char {
                Style::default().fg(termi.theme.success)
            } else {
                Style::default().fg(termi.theme.error)
            }
        } else {
            Style::default()
                .fg(termi.theme.inactive)
                .add_modifier(Modifier::DIM)
        };
        let styled_char = if i == termi.tracker.cursor_position {
            Span::styled(char.to_string(), style.add_modifier(Modifier::UNDERLINED))
        } else {
            Span::styled(char.to_string(), style)
        };
        spans.push(styled_char)
    }

    text.lines.push(Line::from(spans));

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(termi.theme.error))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    paragraph
}

/// Creates a centered rectangle with the given width and height percentages.
///
/// # Arguments
///
/// * `px` - The width percentage (0-100) of the rectangle
/// * `py` - The height percentage (0-100) of the rectangle
/// * `r` - The outer rectangle to center within
///
/// # Returns
///
/// A `Rect` representing the centered area
fn centered_rect(px: u16, py: u16, r: Rect) -> Rect {
    let horizontal_margin = (r.width.saturating_sub(r.width * px / 100)) / 2;
    let vertical_margin = (r.height.saturating_sub(r.height * py / 100)) / 2;

    Rect {
        x: r.x + horizontal_margin,
        y: r.y + vertical_margin,
        width: r.width * px / 100,
        height: r.height * py / 100,
    }
}
