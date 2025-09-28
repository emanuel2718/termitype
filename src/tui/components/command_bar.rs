use crate::theme::Theme;
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

pub fn create_command_area<'a>(theme: &Theme) -> Paragraph<'a> {
    let higlight_style = Style::default().fg(theme.highlight());
    let fg_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let commands_lines = vec![
        Line::from(vec![
            Span::styled("tab", higlight_style),
            Span::styled(" + ", fg_style),
            Span::styled("enter", higlight_style),
            Span::styled(" - restart", fg_style),
        ]),
        Line::from(vec![
            Span::styled("esc", higlight_style),
            Span::styled(" - menu  ", fg_style),
            Span::styled("ctrl", higlight_style),
            Span::styled(" + ", fg_style),
            Span::styled("c", higlight_style),
            Span::styled(" - quit", fg_style),
        ]),
    ];
    Paragraph::new(commands_lines)
        .style(Style::default().fg(theme.fg()))
        .alignment(Alignment::Center)
        .block(Block::default())
}
