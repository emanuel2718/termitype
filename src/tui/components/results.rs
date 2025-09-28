use crate::{
    app::App,
    theme::Theme,
    tui::helpers::{calculate_horizontal_padding, center_lines_vertically, max_line_width},
};
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

///  ResultsVariant::Minimal
pub fn create_minimal_results_display<'a>(
    app: &mut App,
    theme: &Theme,
    height: u16,
    width: u16,
) -> Paragraph<'a> {
    let summary = app.tracker.summary();

    let label_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let value_style = Style::default().fg(theme.fg());
    let accent_style = Style::default().fg(theme.accent());
    let error_style = Style::default().fg(theme.error());

    let mode_info = if app.config.current_mode().is_time_mode() {
        format!("Time({}s)", app.config.current_mode().value())
    } else {
        format!("Words({})", app.config.current_mode().value())
    };

    let stats: Vec<Line> = [
        (
            "WPM",
            Span::styled(format!("{:.0}", summary.wpm), accent_style),
        ),
        (
            "Errors",
            Span::styled(format!("{}", summary.total_errors), error_style),
        ),
        (
            "Accuracy",
            Span::styled(format!("{:.1}%", summary.accuracy * 100.0), value_style),
        ),
        (
            "Language",
            Span::styled(app.config.current_language(), value_style),
        ),
        ("Mode", Span::styled(mode_info, value_style)),
    ]
    .into_iter()
    .map(|(label, value)| Line::from(vec![Span::styled(format!("{label}: "), label_style), value]))
    .chain(std::iter::once(Line::from("")))
    .collect();

    let vertically_padded = center_lines_vertically(stats, height);

    let content_max_width = max_line_width(&vertically_padded);
    let (left_pad, right_pad) = calculate_horizontal_padding(content_max_width, width);

    Paragraph::new(vertically_padded)
        .style(Style::default().fg(theme.fg()))
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding {
            left: left_pad,
            right: right_pad,
            top: 0,
            bottom: 0,
        }))
}

pub fn create_results_footer_element(theme: &Theme) -> Paragraph<'_> {
    let dim = Modifier::DIM;
    let items = vec![
        ("[N]", "ew "),
        ("[R]", "edo "),
        ("[Q]", "uit "),
        ("[ESC]", " menu"),
    ];

    let spans: Vec<Span> = items
        .into_iter()
        .flat_map(|(key, text)| {
            vec![
                Span::styled(key, Style::default().fg(theme.highlight())),
                Span::styled(text, Style::default().fg(theme.fg()).add_modifier(dim)),
            ]
        })
        .collect();

    let line = Line::from(spans);
    Paragraph::new(line)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default())
}
