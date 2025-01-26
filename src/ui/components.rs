use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    symbols::line,
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
    Frame,
};

use super::constants::{FOOTER_HEIGHT, HEADER_HEIGHT, VERTICAL_MARGIN};
use crate::{config::Mode, constants::APPNAME, termi::Termi, theme::Theme, version::VERSION};

pub fn render_title(f: &mut Frame, termi: &Termi, area: Rect) {
    let title = Paragraph::new(APPNAME)
        .style(Style::default().fg(termi.theme.highlight))
        .add_modifier(Modifier::BOLD)
        .alignment(Alignment::Center);
    f.render_widget(title, area);
}

pub fn render_progress_info(f: &mut Frame, termi: &Termi, area: Rect) {
    let progress_text = match termi.config.current_mode() {
        Mode::Time { duration } => {
            if let Some(remaining) = termi.tracker.time_remaining {
                format!("{:.0}", remaining.as_secs())
            } else {
                format!("{}", duration)
            }
        }
        Mode::Words { count } => {
            let completed_words = termi
                .tracker
                .user_input
                .iter()
                .filter(|&&c| c == Some(' '))
                .count();
            format!("{}/{}", completed_words, count)
        }
    };

    let wpm_text = format!(" {:>3.0} wpm", termi.tracker.wpm);

    let spans = vec![
        Span::styled(progress_text, Style::default().fg(termi.theme.highlight)),
        Span::styled(wpm_text, Style::default().fg(termi.theme.inactive)),
    ];

    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

pub fn render_typing_area(f: &mut Frame, termi: &Termi, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    let words = create_styled_words(termi);

    let typing_area = Paragraph::new(words)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    f.render_widget(typing_area, layout[1]);
}

fn styled_span(theme: &Theme, text: &str, is_active: bool) -> Span<'static> {
    Span::styled(
        text.to_string(),
        Style::default()
            .fg(if is_active {
                theme.highlight
            } else {
                theme.inactive
            })
            .add_modifier(if is_active {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
    )
}

pub fn render_top_bar(f: &mut Frame, termi: &Termi, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    let separator = || {
        Span::styled(
            format!(" {} ", line::VERTICAL),
            Style::default().fg(termi.theme.inactive),
        )
    };

    // first group
    spans.extend(vec![
        styled_span(&termi.theme, "@ ", termi.config.use_punctuation),
        styled_span(&termi.theme, "punctuation", termi.config.use_punctuation),
        Span::raw(" "),
        styled_span(&termi.theme, "# ", termi.config.use_numbers),
        styled_span(&termi.theme, "numbers", termi.config.use_numbers),
    ]);

    spans.push(separator());

    // second gropu
    let is_time_mode = matches!(termi.config.current_mode(), Mode::Time { .. });
    let is_words_mode = matches!(termi.config.current_mode(), Mode::Words { .. });
    spans.extend(vec![
        styled_span(&termi.theme, "⏱ ", is_time_mode),
        styled_span(&termi.theme, "time", is_time_mode),
        Span::raw(" "),
        styled_span(&termi.theme, "A ", is_words_mode),
        styled_span(&termi.theme, "words", is_words_mode),
    ]);

    spans.push(separator());

    // third group
    let modes = match is_time_mode {
        true => vec![15, 30, 60, 120],
        false => vec![10, 25, 50, 100],
    };
    let current_value = termi.config.current_mode().value();
    for mode in modes {
        spans.push(styled_span(
            &termi.theme,
            &format!("{} ", mode),
            mode == current_value,
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

pub fn render_command_bar(f: &mut Frame, termi: &Termi, area: Rect) {
    let spans = vec![
        Span::styled(
            "tab",
            Style::default()
                .fg(termi.theme.inactive)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" + ", Style::default().fg(termi.theme.inactive)),
        Span::styled(
            "enter",
            Style::default()
                .fg(termi.theme.inactive)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " - restart test   ",
            Style::default().fg(termi.theme.inactive),
        ),
        Span::styled(
            "esc",
            Style::default()
                .fg(termi.theme.inactive)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" - command line", Style::default().fg(termi.theme.inactive)),
    ];

    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

pub fn render_footer(f: &mut Frame, termi: &Termi, area: Rect) {
    let footer_items = vec!["github/emanuel2718"];

    let spans: Vec<Span> = footer_items
        .iter()
        .map(|&item| {
            vec![
                Span::styled(item, Style::default().fg(termi.theme.inactive)),
                Span::raw(" "),
            ]
        })
        .flatten()
        .collect();

    let version = format!("{}", VERSION);
    let mut all_spans = spans;
    all_spans.extend(vec![
        Span::raw("    "),
        // theme
        Span::styled(
            termi.theme.identifier.clone(), // NOTE: this is not good
            Style::default().fg(termi.theme.inactive),
        ),
        Span::raw("    "),
        // version
        Span::styled(
            line::DOUBLE_VERTICAL_RIGHT,
            Style::default().fg(termi.theme.inactive),
        ),
        Span::raw(" "),
        Span::styled(version, Style::default().fg(termi.theme.inactive)),
    ]);

    let paragraph = Paragraph::new(Line::from(all_spans)).alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

pub fn render_results_screen(f: &mut Frame, termi: &Termi, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(VERTICAL_MARGIN),
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(1),
            Constraint::Length(FOOTER_HEIGHT),
            Constraint::Length(VERTICAL_MARGIN),
        ])
        .split(area);

    render_title(f, termi, layout[1]);

    let results = create_results_widget(termi);
    f.render_widget(results, layout[2]);

    render_footer(f, termi, layout[3]);
}

fn create_results_widget(termi: &Termi) -> Paragraph<'static> {
    let completion_time = termi.tracker.completion_time.unwrap_or(0.0);
    let content_lines = vec![
        Line::from(vec![
            Span::raw("wpm: ").fg(termi.theme.inactive),
            Span::styled(
                format!("{:.0}", termi.tracker.wpm),
                Style::default().fg(termi.theme.highlight),
            ),
        ]),
        Line::from(vec![
            Span::raw("acc: ").fg(termi.theme.inactive),
            Span::styled(
                format!("{}%", termi.tracker.accuracy),
                Style::default().fg(termi.theme.highlight),
            ),
        ]),
        Line::from(vec![
            Span::raw("time: ").fg(termi.theme.inactive),
            Span::styled(
                format!("{:.1}s", completion_time),
                Style::default().fg(termi.theme.highlight),
            ),
        ]),
    ];

    Paragraph::new(content_lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}

fn create_styled_words(termi: &Termi) -> Text<'static> {
    let words = &termi.words;
    let cursor_pos = termi.tracker.cursor_position;
    let target_chars: Vec<char> = words.chars().collect();

    let spans: Vec<Span> = target_chars
        .iter()
        .enumerate()
        .map(|(i, &char)| style_character(termi, i, char, cursor_pos))
        .collect();

    Text::from(Line::from(spans))
}

fn style_character(termi: &Termi, index: usize, char: char, cursor_pos: usize) -> Span<'static> {
    let base_style = match termi.tracker.user_input.get(index).copied().flatten() {
        Some(input_char) if input_char == char => Style::default().fg(termi.theme.success),
        Some(_) => Style::default().fg(termi.theme.error),
        None => Style::default()
            .fg(termi.theme.inactive)
            .add_modifier(Modifier::DIM),
    };

    let style = if index == cursor_pos {
        base_style.add_modifier(Modifier::UNDERLINED)
    } else {
        base_style
    };

    Span::styled(char.to_string(), style)
}
