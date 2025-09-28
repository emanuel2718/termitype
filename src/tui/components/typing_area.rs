use crate::{
    app::App,
    theme::Theme,
    tracker::Tracker,
    tui::{
        helpers::{calculate_padding, calculate_visible_lines, set_cursor_position},
        layout::AppLayout,
    },
};
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_typing_area(frame: &mut Frame, app: &mut App, theme: &Theme, layout: &AppLayout) {
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

    let paragraph = Paragraph::new(padded_lines)
        .style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM));

    frame.render_widget(paragraph, layout.center_area);

    set_cursor_position(frame, app, &target_text_lines, layout, padding);
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
        crate::config::Mode::Time(_) => {
            let total_secs = app.tracker.mode.value();
            let elapsed_secs = app.tracker.elapsed_time().as_secs();
            let secs_left = (total_secs as i64 - elapsed_secs as i64).max(0);
            format!("{}", secs_left)
        }
        crate::config::Mode::Words(_) => {
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
