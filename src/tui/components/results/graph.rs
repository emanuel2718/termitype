use crate::theme::Theme;
use crate::tui::helpers::{calculate_horizontal_padding, center_lines_vertically, max_line_width};
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

// TODO: implement this
pub fn create_graph_results<'a>(theme: &Theme, height: u16, width: u16) -> Paragraph<'a> {
    let title_style = Style::default()
        .fg(theme.accent())
        .add_modifier(Modifier::BOLD);
    let subtitle_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);

    let lines = vec![
        Line::from(Span::styled("Graph View", title_style)),
        Line::from(""),
        Line::from(Span::styled("Coming Soon", subtitle_style)),
    ];

    let vertically_padded = center_lines_vertically(lines, height);
    let content_max_width = max_line_width(&vertically_padded);
    let (left_pad, right_pad) = calculate_horizontal_padding(content_max_width, width);

    Paragraph::new(vertically_padded)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default().padding(Padding {
            left: left_pad,
            right: right_pad,
            top: 0,
            bottom: 0,
        }))
}
