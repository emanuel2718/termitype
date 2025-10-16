use crate::{
    app::App, ascii, common::strings::truncate_to_width, constants::APP_NAME, theme::Theme,
};
use ratatui::{
    layout::Alignment,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};
use unicode_width::UnicodeWidthStr;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATS_MIN_WIDTH: usize = 40; // min width needed for stats
const SPACING: usize = 4; // space between art and stats
const MIN_PADDING: usize = 4; // min horizontal_padding

use ratatui::{layout::Rect, Frame};

pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let height = area.height;
    let width = area.width;
    let summary = app.tracker.summary();

    let ascii_name = ascii::get_ascii(app.config.current_ascii_art().as_str()).unwrap_or_default();
    let ascii_lines: Vec<&str> = ascii_name.lines().collect();

    // ascii art width, we need this because if we get to a breakpoint where the ascii art width is longer than
    // the currently avaialble space for the art, then we should hide the art completely.
    let raw_ascii_width = ascii_lines
        .iter()
        .map(|l| UnicodeWidthStr::width(*l))
        .max()
        .unwrap_or(0);

    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());

    let ascii_style = Style::default().fg(theme.warning());
    let header_style = Style::default().fg(theme.success());
    let label_style = Style::default().fg(theme.warning());
    let value_style = Style::default().fg(theme.fg());
    let dim_style = Style::default().fg(theme.muted());

    let mode_str = if app.config.current_mode().is_time_mode() {
        format!("Time ({}s)", app.config.current_mode().value())
    } else {
        format!("Words ({})", app.config.current_mode().value())
    };

    let total_keystrokes = summary.correct_chars + summary.total_errors;

    let language = app.config.current_language();
    let wpm_str = format!("{:.0}", summary.wpm);
    let raw_wpm_str = format!("{:.0}", summary.net_wpm());
    let duration_str = format!("{:.1}s", summary.elapsed_time.as_secs_f64());
    let accuracy_str = format!("{:.0}%", summary.accuracy * 100.0);
    let consistency_str = format!("{:.0}%", summary.consistency);
    let keystrokes_str = format!("{} ({})", summary.correct_chars, total_keystrokes);
    let correct_str = format!("{}", summary.correct_chars);
    let errors_str = format!("{}", summary.total_errors);
    let backspaces_str = format!("{}", summary.total_errors);
    let wpm_range_str = format!(
        "{:.0}–{:.0}",
        summary.snapshots.min(),
        summary.snapshots.max(),
    );

    let stats = vec![
        (
            format!("{}@{}", username, APP_NAME),
            header_style,
            value_style,
        ),
        ("".to_string(), dim_style, value_style), // separator line
        ("OS".to_string(), label_style, value_style),
        ("Version".to_string(), label_style, value_style),
        ("Mode".to_string(), label_style, value_style),
        ("Lang".to_string(), label_style, value_style),
        ("WPM".to_string(), label_style, value_style),
        ("Raw WPM".to_string(), label_style, value_style),
        ("Duration".to_string(), label_style, value_style),
        ("Accuracy".to_string(), label_style, value_style),
        ("Consistency".to_string(), label_style, value_style),
        ("Keystrokes".to_string(), label_style, value_style),
        ("Correct".to_string(), label_style, value_style),
        ("Errors".to_string(), label_style, value_style),
        ("Backspaces".to_string(), label_style, value_style),
        ("WPM Range".to_string(), label_style, value_style),
    ];

    let values = vec![
        "",
        "",
        APP_NAME,
        VERSION,
        &mode_str,
        &language,
        &wpm_str,
        &raw_wpm_str,
        &duration_str,
        &accuracy_str,
        &consistency_str,
        &keystrokes_str,
        &correct_str,
        &errors_str,
        &backspaces_str,
        &wpm_range_str,
    ];

    let stats_width = stats
        .iter()
        .enumerate()
        .map(|(i, (label, _, _))| {
            // HACK: find a better way to handle this
            if i == 0 {
                // header
                UnicodeWidthStr::width(label.as_str())
            } else if i == 1 {
                // separator
                username.len() + 1 + "termitype".len()
            } else if i < values.len() {
                // stats
                let label_width = UnicodeWidthStr::width(label.as_str());
                let value_width = UnicodeWidthStr::width(values[i]);
                label_width + 2 + value_width // +2 for ": "
            } else {
                0
            }
        })
        .max()
        .unwrap_or(STATS_MIN_WIDTH);

    // determine if we should show or hide the ascii art, depends on available width
    let available_width = width.saturating_sub(MIN_PADDING as u16) as usize;
    let required_width_with_ascii = raw_ascii_width + SPACING + stats_width;
    let (ascii_width, show_ascii) = if required_width_with_ascii <= available_width {
        (raw_ascii_width, true)
    } else {
        (0, false)
    };

    let mut lines: Vec<Line> = Vec::new();

    // vertical offset for the vertical stats alignement
    let stats_offset = if show_ascii {
        let ascii_height = ascii_lines.len().saturating_add(1);
        (ascii_height.saturating_sub(stats.len())) / 2
    } else {
        0
    };

    let max_lines = if show_ascii {
        let ascii_height = ascii_lines.len().saturating_add(1);
        let stats_and_colors_height = stats_offset + stats.len() + 2; // +2 for empty line and color blocks
        ascii_height.max(stats_and_colors_height)
    } else {
        stats.len() + 2 // +2 for empty line and color blocks
    };

    let palette_colors = vec![
        theme.muted(),
        theme.error(),
        theme.success(),
        theme.warning(),
        theme.info(),
        theme.primary(),
        theme.highlight(),
        theme.fg(),
    ];

    for i in 0..max_lines {
        let mut spans = Vec::new();

        if show_ascii {
            let ascii_line_idx = i.saturating_sub(1);
            if i > 0 && ascii_line_idx < ascii_lines.len() {
                let line = ascii_lines[ascii_line_idx];
                let line_width = UnicodeWidthStr::width(line);
                let display_line = if line_width > ascii_width {
                    truncate_to_width(line, ascii_width)
                } else {
                    line.to_string()
                };

                let display_width = UnicodeWidthStr::width(display_line.as_str());
                let padding = ascii_width.saturating_sub(display_width);

                spans.push(Span::styled(
                    format!("{}{}", display_line, " ".repeat(padding)),
                    ascii_style,
                ));
            } else {
                spans.push(Span::raw(" ".repeat(ascii_width)));
            }

            spans.push(Span::raw(" ".repeat(SPACING)));
        }

        if i >= stats_offset && i < (stats_offset + stats.len()) {
            let stat_idx = i - stats_offset;
            if stat_idx == 0 {
                // header
                spans.push(Span::styled(username.clone(), header_style));
                spans.push(Span::styled("@", value_style));
                spans.push(Span::styled("termitype", header_style));
            } else if stat_idx == 1 {
                // separator
                let sep = "─".repeat(username.len() + 1 + "termitype".len());
                spans.push(Span::styled(sep, dim_style));
            } else if stat_idx < values.len() {
                // stats
                spans.push(Span::styled(
                    format!("{}: ", stats[stat_idx].0),
                    stats[stat_idx].1,
                ));
                spans.push(Span::styled(
                    values[stat_idx].to_string(),
                    stats[stat_idx].2,
                ));
            }
        } else if i == stats_offset + stats.len() {
            // empty line after stats
        } else if i == stats_offset + stats.len() + 1 {
            // color blocks
            for color in &palette_colors {
                spans.push(Span::styled("██", Style::default().fg(*color)));
            }
        }

        lines.push(Line::from(spans));
    }

    let content_height = lines.len() as u16;
    let vertical_padding = if height > content_height {
        (height - content_height) / 2
    } else {
        0
    };

    let content_width = if show_ascii {
        ascii_width + SPACING + stats_width
    } else {
        stats_width
    };
    let horizontal_padding = if width as usize > content_width {
        ((width as usize - content_width) / 2) as u16
    } else {
        (MIN_PADDING / 2) as u16
    };

    let widget = Paragraph::new(lines)
        .style(Style::default().fg(theme.fg()))
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding {
            left: horizontal_padding,
            right: horizontal_padding,
            top: vertical_padding,
            bottom: 0,
        }));

    frame.render_widget(widget, area);
}
