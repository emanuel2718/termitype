use crate::{
    app::App,
    theme::Theme,
    tui::{
        helpers::{calculate_padding, resolve_visible_window, set_cursor_position},
        layout::AppLayout,
    },
};
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn render_typing_area(frame: &mut Frame, app: &mut App, theme: &Theme, layout: &AppLayout) {
    let mut lines: Vec<Line<'static>> = Vec::new();

    let top_line = if app.tracker.is_typing() {
        create_tracker_line(app, theme)
    } else {
        create_language_line(app, theme)
    };
    let line_count = app.config.current_line_count();
    app.typing_cache.ensure(
        &app.tracker,
        theme,
        layout.center_area.width,
        line_count,
        app.typing_revision,
    );
    let viewport = resolve_visible_window(
        app.typing_cache.cursor_line(),
        app.typing_cache.lines().len(),
        line_count as usize,
    );

    let visible_lines = app.typing_cache.lines()[viewport.start..viewport.end].to_vec();
    lines.push(top_line);
    lines.push(Line::from(""));
    lines.extend(visible_lines);

    let padding = calculate_padding(&lines, layout.center_area.height).saturating_sub(1);
    let mut padded_lines = vec![Line::from(""); padding];
    padded_lines.extend(lines);

    let paragraph = Paragraph::new(padded_lines).style(Style::default().fg(theme.fg()));

    frame.render_widget(paragraph, layout.center_area);

    set_cursor_position(
        frame,
        app,
        layout,
        padding,
        app.typing_cache.cursor_x(),
        viewport.visible_cursor_y,
    );
}

fn create_language_line(app: &mut App, theme: &Theme) -> Line<'static> {
    let language_span = Span::styled(
        app.config.current_language(),
        Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
    );
    Line::from(vec![language_span]).alignment(Alignment::Center)
}

fn create_tracker_line(app: &mut App, theme: &Theme) -> Line<'static> {
    let mode_progress = match app.tracker.mode {
        crate::config::Mode::Time(_) => {
            let total_secs = app.tracker.mode.value();
            let elapsed_secs = app.tracker.elapsed_time().as_secs();
            let secs_left = (total_secs as i64 - elapsed_secs as i64).max(0);
            format!("{secs_left}")
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
            format!("  {:.0}", app.tracker.wpm()),
            // format!("  {:.0} wpm", app.tracker.wpm()),
            Style::default().fg(theme.highlight()),
        );
        spans.push(wpm);
    }
    Line::from(spans)
}
