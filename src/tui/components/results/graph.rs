use crate::{app::App, theme::Theme, tracker::Summary};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

/// ResultsVariant::graph
pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let summary = app.tracker.summary();

    let r = Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).split(area);

    render_wpm_chart(frame, &summary, theme, r[0]);
    render_stats_section(frame, app, &summary, theme, r[1]);
}

fn render_wpm_chart(frame: &mut Frame, summary: &Summary, theme: &Theme, area: Rect) {
    let snapshots = &summary.snapshots;

    let data: Vec<(f64, f64)> = if !snapshots.is_empty() {
        snapshots
            .iter()
            .enumerate()
            .map(|(i, &wpm)| (i as f64, wpm))
            .collect()
    } else {
        vec![]
    };

    let max_time = if data.len() > 1 {
        (data.len() - 1) as f64
    } else {
        1.0
    };
    let max_wpm = snapshots.max().max(10.0);
    let min_wpm = 0.0;

    let y_upper_bound = (max_wpm * 1.2).max(20.0);
    let axis_value_style = Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD);
    let axis_line_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);

    // X-axis label
    let x_labels = if max_time > 0.0 {
        vec![
            Span::styled("0s", axis_value_style),
            Span::styled(format!("{}s", (max_time / 2.0).round()), axis_value_style),
            Span::styled(format!("{}s", max_time.round()), axis_value_style),
        ]
    } else {
        vec![
            Span::styled("0s", axis_value_style),
            Span::raw(""),
            Span::styled("0s", axis_value_style),
        ]
    };

    // Y-axis labels
    let y_labels = vec![
        Span::styled(format!("{:.0}", min_wpm), axis_value_style),
        Span::styled(format!("{:.0}", y_upper_bound / 2.0), axis_value_style),
        Span::styled(format!("{:.0}", y_upper_bound), axis_value_style),
    ];

    let dataset = Dataset::default()
        .name("WPM")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.accent()))
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::bordered()
                .border_style(axis_line_style)
                .style(Style::default().bg(theme.bg()))
                .title(
                    Line::from(vec![Span::styled(
                        "WPM Over Time",
                        Style::default()
                            .fg(theme.accent())
                            .add_modifier(Modifier::BOLD),
                    )])
                    .centered(),
                ),
        )
        .style(Style::default().bg(theme.bg()))
        .x_axis(
            Axis::default()
                .title("Time")
                .style(axis_line_style)
                .labels(x_labels)
                .bounds([0.0, max_time.max(1.0)]),
        )
        .y_axis(
            Axis::default()
                .title("WPM")
                .style(axis_line_style)
                .labels(y_labels)
                .bounds([min_wpm, y_upper_bound]),
        );

    frame.render_widget(chart, area);
}

fn render_stats_section(
    frame: &mut Frame,
    app: &App,
    summary: &crate::tracker::Summary,
    theme: &Theme,
    area: Rect,
) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_performance_section(frame, summary, theme, columns[0]);
    render_details_section(frame, app, summary, theme, columns[1]);
}

fn render_performance_section(
    frame: &mut Frame,
    summary: &crate::tracker::Summary,
    theme: &Theme,
    area: Rect,
) {
    let label_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let value_style = Style::default().fg(theme.fg());
    let accent_style = Style::default()
        .fg(theme.accent())
        .add_modifier(Modifier::BOLD);
    let warning_style = Style::default().fg(theme.warning());

    let lines = vec![
        Line::from(vec![
            Span::styled("WPM: ", label_style),
            Span::styled(format!("{:.0}", summary.wpm), accent_style),
        ]),
        Line::from(vec![
            Span::styled("Raw WPM: ", label_style),
            Span::styled(format!("{:.0}", summary.raw_wpm()), value_style),
        ]),
        Line::from(vec![
            Span::styled("Accuracy: ", label_style),
            Span::styled(format!("{:.0}%", summary.accuracy * 100.0), warning_style),
        ]),
        Line::from(vec![
            Span::styled("Consistency: ", label_style),
            Span::styled(format!("{:.0}%", summary.consistency), value_style),
        ]),
    ];

    let block = Block::bordered()
        .title("Performance")
        .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
        .style(Style::default().bg(theme.bg()));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(theme.bg()));

    frame.render_widget(paragraph, area);
}

fn render_details_section(
    frame: &mut Frame,
    app: &App,
    summary: &crate::tracker::Summary,
    theme: &Theme,
    area: Rect,
) {
    let label_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let value_style = Style::default().fg(theme.fg());
    let error_style = Style::default().fg(theme.error());
    let success_style = Style::default().fg(theme.success());

    let mode_info = if app.config.current_mode().is_time_mode() {
        format!("Time ({}s)", app.config.current_mode().value())
    } else {
        format!("Words ({})", app.config.current_mode().value())
    };

    let elapsed_secs = summary.elapsed_time.as_secs_f64();
    let total_keystrokes = summary.correct_chars + summary.total_errors;

    let wpm_range = {
        let min_wpm = if summary.snapshots.is_empty() {
            summary.wpm
        } else {
            summary.snapshots.min().min(summary.wpm)
        };
        let max_wpm = if summary.snapshots.is_empty() {
            summary.wpm
        } else {
            summary.snapshots.max().max(summary.wpm)
        };
        format!("{:.0}-{:.0}", min_wpm, max_wpm)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Mode: ", label_style),
            Span::styled(mode_info, value_style),
        ]),
        Line::from(vec![
            Span::styled("Language: ", label_style),
            Span::styled(app.config.current_language(), value_style),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", label_style),
            Span::styled(format!("{:.0}s", elapsed_secs), value_style),
        ]),
        Line::from(vec![
            Span::styled("Keystrokes: ", label_style),
            Span::styled(format!("{}", total_keystrokes), value_style),
        ]),
        Line::from(vec![
            Span::styled("Correct: ", label_style),
            Span::styled(format!("{}", summary.correct_chars), success_style),
        ]),
        Line::from(vec![
            Span::styled("Errors: ", label_style),
            Span::styled(format!("{}", summary.total_errors), error_style),
        ]),
        Line::from(vec![
            Span::styled("Backspaces: ", label_style),
            Span::styled("0", value_style),
        ]),
        Line::from(vec![
            Span::styled("WPM Range: ", label_style),
            Span::styled(wpm_range, value_style),
        ]),
    ];

    let block = Block::bordered()
        .title("Details")
        .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
        .style(Style::default().bg(theme.bg()));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(theme.bg()));

    frame.render_widget(paragraph, area);
}
