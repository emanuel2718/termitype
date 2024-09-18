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

    // 60% width and 50% height of the current trminal window size
    let area = centered_rect(60, 50, size);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            &*termi.title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Left);

    f.render_widget(&block, area);

    let inner_area = block.inner(area);

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1), // Top spacer
                Constraint::Length(3), // Header
                Constraint::Min(1), // Target Text
                Constraint::Min(1), // Bottom spacer
            ]
            .as_ref(),
        )
        .split(inner_area);

    let header_line = match termi.mode {
        Mode::Time { .. } => {
            let minutes = termi.time_remaining.as_secs() / 60;
            let seconds = termi.time_remaining.as_secs() % 60;
            let time_str = format!("Time Remaining: {:02}:{:02}", minutes, seconds);
            let wpm_str = format!("WPM: {}", termi.wpm.round());

            Line::from(vec![
                Span::styled(time_str, Style::default().fg(Color::Yellow)),
                Span::raw("   "), // Spacer
                Span::styled(wpm_str, Style::default().fg(Color::Cyan)),
            ])
        }
        Mode::Words { .. } => {
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
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    f.render_widget(header_paragraph, vertical_layout[1]);

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

