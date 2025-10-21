use crate::theme::Theme;
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

pub fn create_size_warning_element<'a>(
    theme: &Theme,
    height: u16,
    width: u16,
) -> (Paragraph<'a>, u16) {
    let first_line = Line::from(vec![Span::styled(
        "! too smol.",
        Style::default()
            .fg(theme.warning())
            .add_modifier(Modifier::BOLD),
    )]);

    let height_color = if height < 8 {
        theme.error()
    } else {
        theme.success()
    };
    let width_color = if width < 35 {
        theme.error()
    } else {
        theme.success()
    };

    let second_line = Line::from(vec![
        Span::styled("(", Style::default().fg(theme.fg())),
        Span::styled(height.to_string(), Style::default().fg(height_color)),
        Span::styled(", ", Style::default().fg(theme.fg())),
        Span::styled(width.to_string(), Style::default().fg(width_color)),
        Span::styled(")", Style::default().fg(theme.fg()).bg(theme.bg())),
    ]);

    let text = vec![first_line, second_line];
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().bg(theme.bg()))
        .block(Block::default());

    let max_width = 18;

    (paragraph, max_width)
}
