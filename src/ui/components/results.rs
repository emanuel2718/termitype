use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Padding, Paragraph, Wrap},
    Frame,
};

use crate::{
    ascii,
    config::{Mode, ResultsStyle},
    termi::Termi,
    ui::helpers::TermiUtils,
    version::VERSION,
};

pub struct ResultsComponent;

impl ResultsComponent {
    pub fn render(frame: &mut Frame, termi: &mut Termi, area: Rect, is_small: bool) {
        let results_style = termi.current_results_style();

        match results_style {
            ResultsStyle::Graph => Self::render_graph_results(frame, termi, area, is_small),
            ResultsStyle::Minimal => Self::render_minimal_results(frame, termi, area, is_small),
            ResultsStyle::Neofetch => Self::render_neofetch_results(frame, termi, area, is_small),
        }
    }

    /// Render neofetch-styled results screen
    fn render_neofetch_results(frame: &mut Frame, termi: &mut Termi, area: Rect, is_small: bool) {
        let tracker = &termi.tracker;
        let theme = termi.current_theme();
        let config = &termi.config;
        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        let hostname = "termitype";

        let header = format!("{}@{}", username, hostname);
        let separator = "─".repeat(header.chars().count());

        let is_monochromatic = termi.config.monocrhomatic_results;

        let color_success = if is_monochromatic {
            theme.highlight()
        } else {
            theme.success()
        };
        let color_fg = if is_monochromatic {
            theme.muted()
        } else {
            theme.fg()
        };
        let color_warning = if is_monochromatic {
            theme.highlight()
        } else {
            theme.warning()
        };

        let current_art_name = termi
            .preview_ascii_art
            .as_deref()
            .or(config.ascii.as_deref())
            .unwrap_or(ascii::DEFAULT_ASCII_ART_NAME);

        let current_ascii_art = ascii::get_ascii_art_by_name(current_art_name).unwrap_or("");

        let mut stats_lines = vec![
            Line::from(vec![
                Span::styled(username, Style::default().fg(color_success)),
                Span::styled("@", Style::default().fg(color_fg)),
                Span::styled(hostname, Style::default().fg(color_success)),
            ]),
            Line::from(Span::styled(separator, Style::default().fg(color_fg))),
        ];

        let add_stat = |label: &str, value: String| -> Line {
            Line::from(vec![
                Span::styled(format!("{}: ", label), Style::default().fg(color_warning)),
                Span::styled(value, Style::default().fg(color_fg)),
            ])
        };

        let errors = tracker
            .total_keystrokes
            .saturating_sub(tracker.correct_keystrokes);
        let (min_wpm, max_wpm) = tracker
            .wpm_samples
            .iter()
            .fold((u32::MAX, 0), |(min, max), &val| {
                (min.min(val), max.max(val))
            });
        let wpm_range_str = if min_wpm == u32::MAX {
            None
        } else {
            Some(format!("{}-{}", min_wpm, max_wpm))
        };

        let mode_str = match config.current_mode() {
            Mode::Time { duration } => format!("Time ({}s)", duration),
            Mode::Words { count } => format!("Words ({})", count),
        };

        let high_score_mark = if termi.tracker.is_high_score {
            "(High Score)"
        } else {
            ""
        };

        stats_lines.push(add_stat("OS", "termitype".to_string()));
        stats_lines.push(add_stat("Version", VERSION.to_string()));
        stats_lines.push(add_stat("Mode", mode_str));
        stats_lines.push(add_stat(
            "Lang",
            config.language.clone().unwrap_or_default(),
        ));
        stats_lines.push(add_stat(
            "WPM",
            format!("{:.0} {}", tracker.wpm, high_score_mark),
        ));
        stats_lines.push(add_stat("Raw WPM", format!("{:.0}", tracker.raw_wpm)));

        if let Some(time) = tracker.completion_time {
            stats_lines.push(add_stat("Duration", format!("{:.1}s", time)));
        } else if let Mode::Time { duration } = config.current_mode() {
            stats_lines.push(add_stat("Duration", format!("{}s", duration)));
        }

        stats_lines.push(add_stat("Accuracy", format!("{}%", tracker.accuracy)));
        stats_lines.push(add_stat(
            "Consistency",
            format!("{:.0}%", tracker.calculate_consistency()),
        ));
        stats_lines.push(add_stat(
            "Keystrokes",
            format!("{} ({})", tracker.total_keystrokes, tracker.accuracy),
        ));
        stats_lines.push(add_stat(
            "Correct",
            format!("{}", tracker.correct_keystrokes),
        ));
        stats_lines.push(add_stat("Errors", format!("{}", errors)));
        stats_lines.push(add_stat(
            "Backspaces",
            format!("{}", tracker.backspace_count),
        ));
        if let Some(wpm_range) = wpm_range_str {
            stats_lines.push(add_stat("WPM Range", wpm_range));
        }
        stats_lines.push(Line::from(""));

        if !is_small {
            let mut color_blocks = vec![];
            for color in [
                theme.cursor_text(),
                theme.error(),
                theme.success(),
                theme.warning(),
                theme.info(),
                theme.primary(),
                theme.highlight(),
                theme.fg(),
            ] {
                color_blocks.push(Span::styled("██", Style::default().fg(color)));
            }
            stats_lines.push(Line::from(color_blocks));
        }

        Self::_render_neofetch_layout(
            frame,
            area,
            is_small,
            current_ascii_art,
            color_warning,
            stats_lines,
            theme,
        );
    }

    fn _render_neofetch_layout(
        frame: &mut Frame,
        area: Rect,
        is_small: bool,
        ascii_art: &str,
        art_color: Color,
        stats_lines: Vec<Line>,
        theme: &crate::theme::Theme,
    ) {
        let art_text = Text::from(ascii_art).style(Style::default().fg(art_color));
        let art_height = art_text.height() as u16;
        let art_width = art_text.width() as u16;

        let stats_text = Text::from(stats_lines);
        let stats_height = stats_text.height() as u16;
        let stats_width = stats_text
            .lines
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or(0) as u16;

        let footer_height = if is_small { 0 } else { 2 };
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if is_small {
                vec![Constraint::Percentage(100)]
            } else {
                vec![
                    Constraint::Min(0),                // content
                    Constraint::Length(footer_height), // footer
                ]
            })
            .split(area);

        let content_area = main_layout[0];
        let footer_area = if is_small { None } else { Some(main_layout[1]) };

        if is_small {
            let centered_rect = Rect {
                x: content_area.x + content_area.width.saturating_sub(stats_width) / 2,
                y: content_area.y + content_area.height.saturating_sub(stats_height) / 2,
                width: stats_width.min(content_area.width),
                height: stats_height.min(content_area.height),
            };
            frame.render_widget(Paragraph::new(stats_text), centered_rect);
        } else {
            let total_needed_width = art_width + stats_width + 5;
            let horizontal_padding = content_area.width.saturating_sub(total_needed_width) / 2;
            let vertical_padding = content_area
                .height
                .saturating_sub(stats_height.max(art_height))
                / 2;

            let layout_area = Rect {
                x: content_area.x + horizontal_padding,
                y: content_area.y + vertical_padding,
                width: total_needed_width.min(content_area.width),
                height: stats_height.max(art_height).min(content_area.height),
            };

            if layout_area.right() > content_area.right() || layout_area.width == 0 {
                let centered_rect = Rect {
                    x: content_area.x + content_area.width.saturating_sub(stats_width) / 2,
                    y: content_area.y + content_area.height.saturating_sub(stats_height) / 2,
                    width: stats_width.min(content_area.width),
                    height: stats_height.min(content_area.height),
                };
                frame.render_widget(Paragraph::new(stats_text), centered_rect);
            } else {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(art_width),
                        Constraint::Length(5), // padding
                        Constraint::Length(stats_width),
                    ])
                    .split(layout_area);

                let art_area = chunks[0];
                let stats_area = chunks[2];

                let art_y_padding = art_area.height.saturating_sub(art_height) / 2;
                let centered_art_area = Rect {
                    y: art_area.y + art_y_padding,
                    height: art_height.min(art_area.height),
                    ..art_area
                };

                let stats_y_padding = stats_area.height.saturating_sub(stats_height) / 2;
                let centered_stats_area = Rect {
                    y: stats_area.y + stats_y_padding,
                    height: stats_height.min(stats_area.height),
                    ..stats_area
                };

                frame.render_widget(Paragraph::new(art_text), centered_art_area);
                frame.render_widget(Paragraph::new(stats_text), centered_stats_area);
            }
        }

        if let Some(footer_area) = footer_area {
            let footer_line = TermiUtils::create_results_footer_text(theme);
            frame.render_widget(Paragraph::new(footer_line), footer_area);
        }
    }

    /// Render Graph results
    fn render_graph_results(frame: &mut Frame, termi: &mut Termi, area: Rect, is_small: bool) {
        let theme = termi.current_theme();
        let is_monochromatic = termi.config.monocrhomatic_results;

        let color_success = if is_monochromatic {
            theme.highlight()
        } else {
            theme.success()
        };
        let color_fg = if is_monochromatic {
            theme.muted()
        } else {
            theme.fg()
        };
        let color_muted = if is_monochromatic {
            theme.fg()
        } else {
            theme.muted()
        };

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if is_small {
                [Constraint::Percentage(100)].as_ref()
            } else {
                [
                    Constraint::Percentage(60), // graph
                    Constraint::Percentage(38), // stats
                    Constraint::Length(2),      // footer
                ]
                .as_ref()
            })
            .split(area);

        let graph_area = main_layout[0];
        let stats_area = if is_small { area } else { main_layout[1] };
        let footer_area = if is_small { None } else { Some(main_layout[2]) };

        if !is_small && !termi.tracker.wpm_samples.is_empty() {
            Self::_render_wpm_graph(frame, termi, graph_area);
        }

        Self::_render_graph_stats(
            frame,
            termi,
            stats_area,
            is_small,
            color_success,
            color_fg,
            color_muted,
        );

        if let Some(footer_area) = footer_area {
            let footer_line = TermiUtils::create_results_footer_text(theme);
            frame.render_widget(Paragraph::new(footer_line), footer_area);
        }
    }

    fn _render_wpm_graph(frame: &mut Frame, termi: &Termi, area: Rect) {
        let theme = termi.current_theme();

        if termi.tracker.wpm_samples.is_empty() {
            return;
        }

        let wpm_data: Vec<(f64, f64)> = termi
            .tracker
            .wpm_samples
            .iter()
            .enumerate()
            .map(|(i, &wpm)| ((i + 1) as f64, wpm as f64))
            .collect();

        let max_wpm = termi.tracker.wpm_samples.iter().max().copied().unwrap_or(0) as f64;
        let min_wpm = termi.tracker.wpm_samples.iter().min().copied().unwrap_or(0) as f64;
        let max_time = if let Mode::Time { duration } = termi.config.current_mode() {
            (duration as f64).floor()
        } else {
            termi.tracker.wpm_samples.len() as f64
        };

        // padding
        let y_max = (max_wpm * 1.1).max(max_wpm + 10.0);
        let y_min = (min_wpm * 0.9).min(min_wpm - 10.0).max(0.0);

        let dataset = Dataset::default()
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.success()))
            .data(&wpm_data);

        let chart = Chart::new(vec![dataset])
            .style(Style::default().bg(theme.bg()))
            .block(
                Block::default()
                    .title(" WMP Over Time ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(theme.border())
                            .add_modifier(Modifier::DIM),
                    )
                    .style(Style::default().bg(theme.bg())),
            )
            .x_axis(
                Axis::default()
                    .title("Time (seconds)")
                    .style(Style::default().fg(theme.muted()))
                    .bounds([1.0, max_time])
                    .labels(vec![
                        Span::styled("1", Style::default().fg(theme.muted())),
                        Span::styled(
                            format!("{:.0}", max_time / 2.0),
                            Style::default().fg(theme.muted()),
                        ),
                        Span::styled(
                            format!("{:.0}", max_time),
                            Style::default().fg(theme.muted()),
                        ),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title("WMP")
                    .style(Style::default().fg(theme.muted()))
                    .bounds([y_min, y_max])
                    .labels(vec![
                        Span::styled(format!("{:.0}", y_min), Style::default().fg(theme.muted())),
                        Span::styled(
                            format!("{:.0}", (y_min + y_max) / 2.0),
                            Style::default().fg(theme.muted()),
                        ),
                        Span::styled(format!("{:.0}", y_max), Style::default().fg(theme.muted())),
                    ]),
            );

        frame.render_widget(chart, area);
    }

    fn _render_graph_stats(
        frame: &mut Frame,
        termi: &Termi,
        area: Rect,
        is_small: bool,
        color_success: Color,
        color_fg: Color,
        color_muted: Color,
    ) {
        let tracker = &termi.tracker;
        let config = &termi.config;
        let theme = termi.current_theme();

        let layout = if is_small {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area)
        };

        let perf_stats_area = layout[0];
        let details_stats_area = layout[1];

        // Performance Stats
        let perf_lines = vec![
            Line::from(vec![
                Span::styled("WPM: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{:.0}", tracker.wpm),
                    Style::default()
                        .fg(color_success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Accuracy: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{}%", tracker.accuracy),
                    Style::default()
                        .fg(color_success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Raw WPM: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{:.0}", tracker.raw_wpm),
                    Style::default().fg(color_fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("Consistency: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{:.0}%", tracker.calculate_consistency()),
                    Style::default().fg(color_fg),
                ),
            ]),
        ];

        let perf_block = Block::default()
            .title(" Performance ")
            .title_alignment(Alignment::Left)
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(theme.border())
                    .add_modifier(Modifier::DIM),
            )
            .style(Style::default().bg(theme.bg()));

        let perf_paragraph = Paragraph::new(perf_lines)
            .block(perf_block)
            .wrap(Wrap { trim: false });

        frame.render_widget(perf_paragraph, perf_stats_area);

        // Details Stats
        let errors = tracker
            .total_keystrokes
            .saturating_sub(tracker.correct_keystrokes);
        let (min_wpm, max_wpm) = tracker
            .wpm_samples
            .iter()
            .fold((u32::MAX, 0), |(min, max), &val| {
                (min.min(val), max.max(val))
            });

        let mode_str = match config.current_mode() {
            Mode::Time { duration } => format!("Time ({}s)", duration),
            Mode::Words { count } => format!("Words ({})", count),
        };

        let duration = if let Mode::Time { duration } = config.current_mode() {
            format!("{}s", duration)
        } else {
            let time = tracker.completion_time.unwrap_or(0.0);
            format!("{:.1}s", time)
        };

        let mut details_lines = vec![
            Line::from(vec![
                Span::styled("Mode: ", Style::default().fg(color_muted)),
                Span::styled(mode_str, Style::default().fg(color_fg)),
            ]),
            Line::from(vec![
                Span::styled("Language: ", Style::default().fg(color_muted)),
                Span::styled(
                    config.language.clone().unwrap_or_default(),
                    Style::default().fg(color_fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(color_muted)),
                Span::styled(duration, Style::default().fg(color_fg)),
            ]),
            Line::from(vec![
                Span::styled("Keystrokes: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{}", tracker.total_keystrokes),
                    Style::default().fg(color_fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("Correct: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{}", tracker.correct_keystrokes),
                    Style::default().fg(color_success),
                ),
            ]),
            Line::from(vec![
                Span::styled("Errors: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{}", errors),
                    Style::default().fg(if errors > 0 { theme.error() } else { color_fg }),
                ),
            ]),
            Line::from(vec![
                Span::styled("Backspaces: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{}", tracker.backspace_count),
                    Style::default().fg(color_fg),
                ),
            ]),
        ];

        if min_wpm != u32::MAX {
            details_lines.push(Line::from(vec![
                Span::styled("WPM Range: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{}-{}", min_wpm, max_wpm),
                    Style::default().fg(color_fg),
                ),
            ]));
        }

        let details_block = Block::default()
            .title(" Details ")
            .title_alignment(Alignment::Left)
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(theme.border())
                    .add_modifier(Modifier::DIM),
            )
            .style(Style::default().bg(theme.bg()));

        let details_paragraph = Paragraph::new(details_lines)
            .block(details_block)
            .wrap(Wrap { trim: false });

        frame.render_widget(details_paragraph, details_stats_area);
    }

    /// Render minimal results.
    fn render_minimal_results(frame: &mut Frame, termi: &mut Termi, area: Rect, is_small: bool) {
        let tracker = &termi.tracker;
        let config = &termi.config;
        let theme = termi.current_theme();

        let is_monochromatic = termi.config.monocrhomatic_results;

        let color_success = if is_monochromatic {
            theme.highlight()
        } else {
            theme.success()
        };
        let color_fg = if is_monochromatic {
            theme.muted()
        } else {
            theme.fg()
        };
        let color_muted = if is_monochromatic {
            theme.fg()
        } else {
            theme.muted()
        };

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if is_small {
                [Constraint::Percentage(100)].as_ref()
            } else {
                [
                    Constraint::Min(0),    // content
                    Constraint::Length(2), // footer
                ]
                .as_ref()
            })
            .split(area);

        let content_area = main_layout[0];
        let footer_area = if is_small { None } else { Some(main_layout[1]) };

        let mode_str = match config.current_mode() {
            Mode::Time { duration } => format!("Time ({}s)", duration),
            Mode::Words { count } => format!("Words ({})", count),
        };

        let duration = if let Mode::Time { duration } = config.current_mode() {
            format!("{}s", duration)
        } else {
            let time = tracker.completion_time.unwrap_or(0.0);
            format!("{:.1}s", time)
        };

        let errors = tracker
            .total_keystrokes
            .saturating_sub(tracker.correct_keystrokes);

        let stats_lines = vec![
            Line::from(vec![
                Span::styled("WPM: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{:.0}", tracker.wpm),
                    Style::default()
                        .fg(color_success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Accuracy: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{}%", tracker.accuracy),
                    Style::default()
                        .fg(color_success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Raw WPM: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{:.0}", tracker.raw_wpm),
                    Style::default().fg(color_fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("Language: ", Style::default().fg(color_muted)),
                Span::styled(
                    config.language.clone().unwrap_or_default(),
                    Style::default().fg(color_fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("Mode: ", Style::default().fg(color_muted)),
                Span::styled(mode_str, Style::default().fg(color_fg)),
            ]),
            Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(color_muted)),
                Span::styled(duration, Style::default().fg(color_fg)),
            ]),
            Line::from(vec![
                Span::styled("Errors: ", Style::default().fg(color_muted)),
                Span::styled(
                    format!("{}", errors),
                    Style::default().fg(if errors > 0 { theme.error() } else { color_fg }),
                ),
            ]),
        ];

        let stats_text = Text::from(stats_lines);
        let stats_height = stats_text.height() as u16;
        let stats_width = stats_text
            .lines
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or(0) as u16;

        let centered_rect = Rect {
            x: content_area.x + content_area.width.saturating_sub(stats_width) / 2,
            y: content_area.y + content_area.height.saturating_sub(stats_height) / 2,
            width: stats_width.min(content_area.width),
            height: stats_height.min(content_area.height),
        };

        frame.render_widget(Paragraph::new(stats_text), centered_rect);

        if let Some(footer_area) = footer_area {
            let footer_line = TermiUtils::create_results_footer_text(theme);
            frame.render_widget(Paragraph::new(footer_line), footer_area);
        }
    }
}
