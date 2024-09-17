use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{config::Mode, termi::Termi};

pub fn draw_ui(f: &mut Frame, termi: &Termi) {
    let size = f.area();

    let area = centered_rect(60, 50, size);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Termitype")
        .title_alignment(Alignment::Left);

    f.render_widget(&block, area);

    let inner_area = block.inner(area);

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),       // Header: Time/WPM or Words/WPM
                Constraint::Length(1),       // Spacer
                Constraint::Percentage(100), // Target text
            ]
            .as_ref(),
        )
        .split(inner_area);

    let header_line = match termi.mode {
        Mode::Time => {
            let minutes = termi.time_remaining.as_secs() / 60;
            let seconds = termi.time_remaining.as_secs() % 60;
            let time_str = format!("Time Remaining: {:02}:{:02}", minutes, seconds);
            let wpm_str = format!("WPM: {}", termi.wpm.round());

            Line::from(vec![
                Span::styled(
                    time_str,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "), // Spacer
                Span::styled(
                    wpm_str,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        }
        Mode::Words => {
            let wpm_str = format!("WPM: {}", termi.wpm.round());

            Line::from(vec![Span::styled(
                wpm_str,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])
        }
    };

    let header_paragraph = Paragraph::new(header_line)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(header_paragraph, vertical_layout[0]);

    let text = render_text(termi);

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, vertical_layout[2]);

    if termi.is_finished {
        let completion_message = Paragraph::new("TODO: show stats here! Press Enter to restart.")
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
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(0),
                Constraint::Percentage(100),
                Constraint::Percentage(0),
            ]
            .as_ref(),
        )
        .split(vertical_layout[1]);

    horizontal_layout[1]
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
