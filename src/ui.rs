use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::termi::Termi;

pub fn draw_ui(f: &mut Frame, termi: &Termi) {
    let size = f.area();

    let area = centered_rect(60, 50, size);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(&*termi.title)
        .title_alignment(Alignment::Left);

    f.render_widget(&block, area);

    let text = render_text(termi);

    let inner_area = block.inner(area);

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(40), // top padding
                Constraint::Percentage(20), // text
                Constraint::Percentage(40), // bottom padding
            ]
            .as_ref(),
        )
        .split(inner_area);

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false }); // enable text wrapping

    f.render_widget(paragraph, vertical_layout[1]);

    if termi.is_finished {
        let completion_message =
            Paragraph::new("You have completed the test! Press Enter to restart.")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
        f.render_widget(completion_message, size);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(vertical_layout[1])[1]
}

fn render_text(termi: &Termi) -> Text {
    let mut text = Text::default();
    let target_chars: Vec<char> = termi.target_text.chars().collect();
    let mut spans = Vec::new();

    for (i, &target_char) in target_chars.iter().enumerate() {
        let style = if let Some(input_char) = termi.user_input.get(i).and_then(|&x| x) {
            if input_char == target_char {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            }
        } else {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::DIM)
        };

        let styled_char = if i == termi.cursor_pos {
            Span::styled(
                target_char.to_string(),
                style.add_modifier(Modifier::UNDERLINED),
            )
        } else {
            Span::styled(target_char.to_string(), style)
        };

        spans.push(styled_char);
    }

    text.lines.push(Line::from(spans));

    text
}
