use crate::{app::App, config::Setting, theme::Theme};
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

const MODE_LINE_MIN_WIDTH: u16 = 80;

pub fn create_mode_line<'a>(
    app: &App,
    theme: &Theme,
    container_height: u16,
    container_width: u16,
) -> Paragraph<'a> {
    let mut spans = Vec::new();
    let fg_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let highlight_style = Style::default()
        .fg(theme.highlight())
        .add_modifier(Modifier::BOLD);

    let current_mode = app.config.current_mode();

    // show compact menu if not enough width
    if container_width < MODE_LINE_MIN_WIDTH {
        let compact_spans = vec![
            Span::styled("☰ ", fg_style),
            Span::styled("Menu ", fg_style),
            Span::styled("<esc>", fg_style),
        ];
        let compact_line = Line::from(compact_spans);
        return Paragraph::new(compact_line)
            .style(Style::default())
            .alignment(Alignment::Center)
            .block(Block::default().padding(mode_line_padding(container_height)));
    }

    // punctuation
    let punctuation_style = if app.config.is_enabled(Setting::Punctuation) {
        highlight_style
    } else {
        fg_style
    };
    spans.push(Span::styled("! punctuation ", punctuation_style));

    // numbers
    let numbers_style = if app.config.is_enabled(Setting::Numbers) {
        highlight_style
    } else {
        fg_style
    };
    spans.push(Span::styled("# numbers ", numbers_style));

    // symbols
    let symbols_style = if app.config.is_enabled(Setting::Symbols) {
        highlight_style
    } else {
        fg_style
    };
    spans.push(Span::styled("@ symbols ", symbols_style));

    // separator
    spans.push(Span::styled("| ", fg_style));

    // time
    let time_mode_style = if app.config.current_mode().is_time_mode() {
        highlight_style
    } else {
        fg_style
    };
    spans.push(Span::styled("T time ", time_mode_style));

    // words
    let word_mode_style = if app.config.current_mode().is_words_mode() {
        highlight_style
    } else {
        fg_style
    };
    spans.push(Span::styled("A words ", word_mode_style));

    // separator
    spans.push(Span::styled("| ", fg_style));

    // TODO: add custom values
    let is_time_mode = current_mode.is_time_mode();
    let values = if is_time_mode {
        [15, 30, 60, 120]
    } else {
        [10, 25, 50, 100]
    };
    for &val in &values {
        let val_style = if current_mode.value() == val {
            highlight_style
        } else {
            fg_style
        };
        spans.push(Span::styled(format!("{} ", val), val_style));
    }

    let mode_line = Line::from(spans);
    Paragraph::new(mode_line)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default().padding(mode_line_padding(container_height)))
}

fn mode_line_padding(height: u16) -> Padding {
    let top_padding = if height >= 8 { 3 } else { 0 };
    Padding {
        left: 0,
        right: 0,
        top: top_padding,
        bottom: 0,
    }
}
