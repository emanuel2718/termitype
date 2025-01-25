use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

use crate::{
    constants::{APPNAME, WINDOW_HEIGHT_PERCENT, WINDOW_WIDTH_PERCENT},
    termi::Termi,
    tracker::Status,
    version::VERSION,
};

/// Draws the UI inside the given frame.
///
/// # Parameters
///
/// - `f`: A mutable reference to the `Frame` where widgets will be rendered.
/// - `termi`: A reference to the `Termi` struct containing application state and configuration.
/// # Example
pub fn draw_ui(f: &mut Frame, termi: &Termi) {
    let size = f.area();
    let area = centered_rect(
        WINDOW_WIDTH_PERCENT as u16,
        WINDOW_HEIGHT_PERCENT as u16,
        size,
    );
    let block = create_main_block(termi);
    let inner_area = block.inner(area);

    f.render_widget(&block, area);

    match termi.tracker.status {
        Status::Completed => {
            let results = results_widget(termi, inner_area);
            f.render_widget(&results, inner_area);
        }
        _ => {
            let layout_areas = create_main_layout(inner_area);
            render_widgets(f, termi, &layout_areas);
        }
    }
}

/// Creates the main widget layout for the entire UI
fn create_main_block(termi: &Termi) -> Block<'static> {
    Block::bordered()
        .border_style(Style::new().fg(termi.theme.border))
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(r"{}-{}", APPNAME, VERSION),
            Style::default()
                .fg(termi.theme.highlight)
                .add_modifier(Modifier::BOLD),
        ))
        .padding(Padding::horizontal(2))
}

fn create_main_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),    // Top spacer
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Test words
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(area)
        .to_vec()
}

fn render_widgets(f: &mut Frame, termi: &Termi, layout_areas: &[Rect]) {
    let header = header_widget(termi);
    let test_words = words_widget(termi);
    let footer = footer_widget(termi);

    f.render_widget(&header, layout_areas[1]);
    f.render_widget(&test_words, layout_areas[2]);
    f.render_widget(&footer, layout_areas[3]);
}

fn header_widget(termi: &Termi) -> Paragraph {
    Paragraph::new(Text::raw(format!(
        "Mode: {} | Time: {:.0?} | WPM: {:.0} | Status: {:?}",
        termi.config.current_mode().value(),
        termi.tracker.time_remaining.unwrap().as_secs(),
        termi.tracker.wpm,
        termi.tracker.status
    )))
    .style(Style::default().fg(termi.theme.highlight))
    .alignment(Alignment::Center)
}

fn words_widget(termi: &Termi) -> Paragraph {
    let words = &termi.words;
    let cursor_pos = termi.tracker.cursor_position;
    let target_chars: Vec<char> = words.chars().collect();

    let spans: Vec<Span> = target_chars
        .iter()
        .enumerate()
        .map(|(i, &char)| {
            let style = match termi.tracker.user_input.get(i).copied().flatten() {
                Some(input_char) if input_char == char => Style::default().fg(termi.theme.success),
                Some(_) => Style::default().fg(termi.theme.error),
                None => Style::default()
                    .fg(termi.theme.inactive)
                    .add_modifier(Modifier::DIM),
            };

            let style = if i == cursor_pos {
                style.add_modifier(Modifier::UNDERLINED)
            } else {
                style
            };

            Span::styled(char.to_string(), style)
        })
        .collect();

    Paragraph::new(Text::from(Line::from(spans)))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
}

fn footer_widget(termi: &Termi) -> Paragraph {
    Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled(
                "tab + enter",
                Style::default()
                    .fg(termi.theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - restart test").fg(termi.theme.inactive),
        ]),
        Line::from(vec![
            Span::styled(
                "ctrl + c",
                Style::default()
                    .fg(termi.theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" or ").fg(termi.theme.inactive),
            Span::styled(
                "ctrl + z",
                Style::default()
                    .fg(termi.theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - exit").fg(termi.theme.inactive),
        ]),
    ]))
    .alignment(Alignment::Center)
}

fn results_widget(termi: &Termi, area: Rect) -> Paragraph<'static> {
    let completion_time = termi.tracker.completion_time.unwrap_or(0.0);

    let content_lines = vec![
        Line::from(vec![Span::styled(
            "Test Completed!",
            Style::default()
                .fg(termi.theme.success)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("WPM: "),
            Span::styled(
                format!("{:.0}", termi.tracker.wpm),
                Style::default().fg(termi.theme.highlight),
            ),
        ]),
        Line::from(vec![
            Span::raw("Accuracy: "),
            Span::styled(
                format!("{}%", termi.tracker.accuracy),
                Style::default().fg(termi.theme.highlight),
            ),
        ]),
        Line::from(vec![
            Span::raw("Time: "),
            Span::styled(
                format!("{:.1}s", completion_time),
                Style::default().fg(termi.theme.highlight),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press tab + enter to restart",
            Style::default()
                .fg(termi.theme.inactive)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let total_height = area.height as usize;
    let content_height = content_lines.len();
    let padding_height = (total_height - content_height) / 2;

    let padding = vec![Line::from(""); padding_height];

    let mut lines = padding.clone();
    lines.extend(content_lines);
    lines.extend(padding);

    Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}

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
