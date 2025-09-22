use crate::{
    app::App,
    config::{Mode, Setting},
    constants::APP_NAME,
    theme::Theme,
    tracker::Tracker,
    tui::{
        layout::AppLayout,
        utils::{
            calculate_padding, calculate_visible_lines, footer_padding, mode_line_padding,
            set_cursor_position, title_padding,
        },
    },
};
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn create_title<'a>(app: &App, theme: &Theme) -> Paragraph<'a> {
    let is_typing = app.tracker.is_typing();
    Paragraph::new(APP_NAME)
        .style(Style::default().fg(theme.highlight()))
        .add_modifier(if is_typing {
            Modifier::DIM
        } else {
            Modifier::empty()
        })
        .alignment(Alignment::Left)
        .block(Block::default().padding(title_padding()))
}

pub fn create_mode_line<'a>(app: &App, theme: &Theme) -> Paragraph<'a> {
    let mut spans = Vec::new();
    let fg_dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let highlight_style = Style::default().fg(theme.highlight());

    // punctuation
    let punctuation_style = if app.config.is_enabled(Setting::Punctuation) {
        highlight_style
    } else {
        fg_dim_style
    };
    spans.push(Span::styled("! punctuation ", punctuation_style));

    // numbers
    let numbers_style = if app.config.is_enabled(Setting::Numbers) {
        highlight_style
    } else {
        fg_dim_style
    };
    spans.push(Span::styled("# numbers ", numbers_style));

    // symbols
    let symbols_style = if app.config.is_enabled(Setting::Symbols) {
        highlight_style
    } else {
        fg_dim_style
    };
    spans.push(Span::styled("@ symbols ", symbols_style));

    // separator
    spans.push(Span::styled("| ", fg_dim_style));

    // time
    let time_mode_style = if app.config.current_mode().is_time_mode() {
        highlight_style
    } else {
        fg_dim_style
    };
    spans.push(Span::styled("T time ", time_mode_style));

    // words
    let word_mode_style = if app.config.current_mode().is_words_mode() {
        highlight_style
    } else {
        fg_dim_style
    };
    spans.push(Span::styled("A words ", word_mode_style));

    // separator
    spans.push(Span::styled("| ", fg_dim_style));

    let durations = [15, 30, 60, 120];
    for &dur in &durations {
        let dur_style = if app.config.current_mode().is_time_mode()
            && app.config.current_mode().value() == dur
        {
            highlight_style
        } else {
            fg_dim_style
        };
        spans.push(Span::styled(format!("{} ", dur), dur_style));
    }

    let mode_line = Line::from(spans);
    Paragraph::new(mode_line)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default().padding(mode_line_padding()))
}

pub fn create_typing_area<'a>(
    frame: &mut Frame,
    app: &mut App,
    theme: &Theme,
    layout: &AppLayout,
) -> Paragraph<'a> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    let top_line = if app.tracker.is_typing() {
        create_tracker_line(app, theme)
    } else {
        create_language_line(app, theme)
    };
    let target_text_lines = create_target_text_line(&app.tracker, theme, layout.center_area.width);

    let visible_lines = calculate_visible_lines(&target_text_lines, app);
    lines.push(top_line);
    lines.push(Line::from(""));
    lines.extend(visible_lines);

    let padding = calculate_padding(&lines, layout.center_area.height).saturating_sub(1);
    let mut padded_lines = vec![Line::from(""); padding];
    padded_lines.extend(lines);

    set_cursor_position(frame, app, &target_text_lines, layout, padding);

    Paragraph::new(padded_lines).style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
}

fn create_language_line(app: &mut App, theme: &Theme) -> Line<'static> {
    let language_span = Span::styled(
        app.config.current_language(),
        Style::default().fg(theme.fg()),
    );
    Line::from(vec![language_span]).alignment(Alignment::Center)
}

fn create_tracker_line(app: &mut App, theme: &Theme) -> Line<'static> {
    let mode_progress = match app.tracker.mode {
        Mode::Time(_) => {
            let total_secs = app.tracker.mode.value();
            let elapsed_secs = app.tracker.elapsed_time().as_secs();
            let secs_left = (total_secs as i64 - elapsed_secs as i64).max(0);
            format!("{}", secs_left)
        }
        Mode::Words(_) => {
            let summary = app.tracker.summary();
            format!("{}/{}", summary.completed_words, summary.total_words)
        }
    };
    let mut spans = vec![Span::styled(
        mode_progress,
        Style::default().fg(theme.highlight()),
    )];
    if !app.config.should_hide_live_wpm() {
        let wpm = Span::styled(
            format!("  {:.0} wpm", app.tracker.wpm()),
            Style::default().fg(theme.fg()),
        );
        spans.push(wpm);
    }
    Line::from(spans)
}

fn create_target_text_line(state: &Tracker, theme: &Theme, max_width: u16) -> Vec<Line<'static>> {
    let mut spans = Vec::new();
    let dim_mod = Modifier::DIM;
    let default_style = Style::default();

    let upcoming_token_style = default_style.fg(theme.fg()).add_modifier(dim_mod);
    let correct_token_style = default_style.fg(theme.success()).remove_modifier(dim_mod);
    let wrong_token_style = default_style.fg(theme.error()).remove_modifier(dim_mod);

    for (i, token) in state.tokens.iter().enumerate() {
        let token_style = if i < state.current_pos {
            // tokens already typed
            if token.is_wrong {
                wrong_token_style
            } else {
                correct_token_style
            }
        } else {
            // upcoming
            upcoming_token_style
        };

        spans.push(Span::styled(token.target.to_string(), token_style));
    }

    let mut lines = Vec::new();
    let mut current_line: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0;
    for span in spans {
        let span_width = span.content.len() as u16;
        if current_width + span_width > max_width {
            // breakpoints
            let mut break_index = current_line.len();
            for (i, s) in current_line.iter().enumerate().rev() {
                if s.content == " " {
                    break_index = i + 1;
                    break;
                }
            }
            if break_index < current_line.len() {
                let next_line = current_line.split_off(break_index);
                lines.push(Line::from(current_line));
                current_line = next_line;
                current_width = current_line.iter().map(|s| s.content.len() as u16).sum();
            } else {
                lines.push(Line::from(current_line));
                current_line = Vec::new();
                current_width = 0;
            }
            current_line.push(span);
            current_width += span_width;
        } else {
            current_line.push(span);
            current_width += span_width;
        }
    }
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }
    lines
}

pub fn create_command_area<'a>(theme: &Theme) -> Paragraph<'a> {
    let commands_lines = vec![
        Line::from(vec![
            Span::styled("tab", Style::default().fg(theme.highlight())),
            Span::styled(
                " + ",
                Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
            ),
            Span::styled("enter", Style::default().fg(theme.highlight())),
            Span::styled(
                " - restart",
                Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled("esc", Style::default().fg(theme.highlight())),
            Span::styled(
                " - menu  ",
                Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
            ),
            Span::styled("ctrl", Style::default().fg(theme.highlight())),
            Span::styled(
                " + ",
                Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
            ),
            Span::styled("c", Style::default().fg(theme.highlight())),
            Span::styled(
                " - quit",
                Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
            ),
        ]),
    ];
    Paragraph::new(commands_lines)
        .style(Style::default().fg(theme.fg()))
        .alignment(Alignment::Center)
        .block(Block::default())
}

pub fn create_footer_element<'a>(theme: &Theme) -> Paragraph<'a> {
    let footer_text = format!("{} v{}", theme.id, APP_VERSION);
    Paragraph::new(footer_text)
        .style(Style::default().fg(theme.fg()))
        .add_modifier(Modifier::DIM)
        .alignment(Alignment::Right)
        .block(Block::default().padding(footer_padding()))
}

pub fn create_menu_search_bar<'a>(theme: &Theme, searching: bool, query: &str) -> Paragraph<'a> {
    if !searching {
        let dim = Modifier::DIM;
        let spans = vec![
            Span::styled(" [↑/k]", Style::default().fg(theme.highlight())),
            Span::styled(" up  ", Style::default().fg(theme.fg()).add_modifier(dim)),
            Span::styled("[↓/j]", Style::default().fg(theme.highlight())),
            Span::styled(" down  ", Style::default().fg(theme.fg()).add_modifier(dim)),
            Span::styled("[/]", Style::default().fg(theme.highlight())),
            Span::styled(
                " search  ",
                Style::default().fg(theme.fg()).add_modifier(dim),
            ),
            Span::styled("[enter]", Style::default().fg(theme.highlight())),
            Span::styled(
                " select  ",
                Style::default().fg(theme.fg()).add_modifier(dim),
            ),
            Span::styled("[esc]", Style::default().fg(theme.highlight())),
            Span::styled(" close", Style::default().fg(theme.fg()).add_modifier(dim)),
        ];
        return Paragraph::new(Line::from(spans))
            .style(Style::default().fg(theme.fg()))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD)),
            );
    }

    let line = Line::from(vec![
        Span::styled("> ", Style::default().fg(theme.primary())),
        Span::styled(query.to_string(), Style::default().fg(theme.fg())),
    ]);
    Paragraph::new(line)
        .style(Style::default().fg(theme.fg()))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD))
                .padding(ratatui::widgets::Padding {
                    left: 1,
                    right: 1,
                    top: 0,
                    bottom: 0,
                }),
        )
}
