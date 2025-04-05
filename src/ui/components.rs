use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Modifier, Style, Stylize},
    symbols::line,
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
    Frame,
};

use crate::{
    config::{Mode, ModeType},
    constants::{APPNAME, APP_LOGO, FULL_LOGO_MIN_WIDTH},
    termi::Termi,
    theme::Theme,
    tracker::Status,
    version::VERSION,
};

#[derive(Debug)]
pub struct ClickableRegion {
    pub area: Rect,
    pub action: ClickAction,
}

#[derive(Debug, Clone)]
pub enum ClickAction {
    TogglePunctuation,
    ToggleSymbols,
    ToggleNumbers,
    SwitchMode(ModeType),
    SetModeValue(usize),
    ToggleThemePicker,
    OpenLanguagePicker,
    ToggleAbout,
}

#[derive(Debug)]
struct UIElement {
    content: String,
    width: u16,
    is_active: bool,
    action: Option<ClickAction>,
}

impl UIElement {
    fn new(content: impl Into<String>, is_active: bool, action: Option<ClickAction>) -> Self {
        let content = content.into();
        let width = content.chars().count() as u16;
        Self {
            content,
            width,
            is_active,
            action,
        }
    }

    fn to_span(&self, theme: &Theme) -> Span<'_> {
        Span::styled(
            self.content.as_str(),
            Style::default()
                .fg(if self.is_active {
                    theme.highlight()
                } else {
                    theme.muted()
                })
                .add_modifier(if self.is_active {
                    Modifier::BOLD
                } else {
                    Modifier::DIM
                }),
        )
    }
}

pub fn title(f: &mut Frame, termi: &Termi, area: Rect) {
    let theme = termi.get_current_theme();
    let title = Paragraph::new(APPNAME)
        .style(Style::default().fg(theme.highlight()))
        .add_modifier(Modifier::BOLD)
        .alignment(Alignment::Left);
    f.render_widget(title, area);
}

pub fn progress_info(f: &mut Frame, termi: &mut Termi, area: Rect) {
    if termi.tracker.status == crate::tracker::Status::Idle {
        let language = termi.config.language.as_str().to_string();

        let element = UIElement::new(language, false, Some(ClickAction::OpenLanguagePicker));

        let start_x = area.x + (area.width.saturating_sub(element.width)) / 2;

        termi.clickable_regions.push(ClickableRegion {
            area: Rect {
                x: start_x,
                y: area.y,
                width: element.width,
                height: 1,
            },
            action: ClickAction::OpenLanguagePicker,
        });

        let theme = termi.get_current_theme();

        let paragraph =
            Paragraph::new(Line::from(vec![element.to_span(theme)])).alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let theme = termi.get_current_theme();

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
        Span::styled(progress_text, Style::default().fg(theme.info())),
        Span::styled(
            wpm_text,
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        ),
    ];

    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

#[derive(Debug)]
pub struct WordPosition {
    pub start_index: usize,
    pub line: usize,
    pub col: usize,
}

fn calculate_word_positions(text: &str, available_width: usize) -> Vec<WordPosition> {
    let word_count = text.split_whitespace().count();
    let mut positions = Vec::with_capacity(word_count);
    let mut current_line = 0;
    let mut current_col = 0;
    let mut current_index = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        // do we need to wrap and is the current word longer than the available width?
        if current_col > 0
            && (current_col + word_len >= available_width || current_col + 1 >= available_width)
        {
            current_line += 1;
            current_col = 0;
        }

        // move to next line if we have to (longer words)
        if word_len >= available_width {
            if current_col > 0 {
                current_line += 1;
            }
            current_col = 0;
        }

        positions.push(WordPosition {
            start_index: current_index,
            line: current_line,
            col: current_col,
        });

        current_col += word_len + 1; // word + space
        current_index += word_len + 1;

        // force wrap after very long words
        if current_col >= available_width {
            current_line += 1;
            current_col = 0;
        }
    }

    positions
}

fn typing_text<'a>(termi: &'a Termi, word_positions: &[WordPosition]) -> Text<'a> {
    let theme = termi.get_current_theme();
    let mut lines: Vec<Line> =
        Vec::with_capacity(word_positions.last().map(|p| p.line + 1).unwrap_or(1));
    let mut current_line = 0;
    let mut current_line_spans = Vec::with_capacity(50);

    let words: Vec<&str> = termi.words.split_whitespace().collect();

    for (word_idx, pos) in word_positions.iter().enumerate() {
        // make new line if we have to
        if pos.line > current_line {
            lines.push(Line::from(std::mem::take(&mut current_line_spans)));
            // current_line_spans is now empty but capacity is preserved
            current_line = pos.line;
        }

        let word = words[word_idx];
        let word_start = pos.start_index;
        let word_len = word.chars().count();

        let is_current_word = termi.tracker.cursor_position >= word_start
            && termi.tracker.cursor_position <= word_start + word_len;

        let is_wrong_word = !is_current_word && termi.tracker.is_word_wrong(word_start);

        #[cfg(debug_assertions)]
        if is_wrong_word {
            use crate::debug::LOG;
            LOG(format!(
                "Word at {} is wrong and is not current word (cursor at {})",
                word_start, termi.tracker.cursor_position
            ));
        }

        let mut chars = Vec::with_capacity(word_len);

        for (i, c) in word.chars().enumerate() {
            let char_idx = word_start + i;
            let style = match termi.tracker.user_input.get(char_idx).copied().flatten() {
                Some(input) if input == c => {
                    let mut style = Style::default().fg(theme.success());
                    if is_wrong_word {
                        style = style
                            .add_modifier(Modifier::UNDERLINED)
                            .underline_color(theme.error());
                    }
                    style
                }
                Some(_) => {
                    let mut style = Style::default().fg(theme.error());
                    if !is_current_word {
                        style = style
                            .add_modifier(Modifier::UNDERLINED)
                            .underline_color(theme.error());
                    }
                    style
                }
                None => Style::default()
                    .fg(theme.foreground())
                    .add_modifier(Modifier::DIM),
            };

            chars.push(Span::styled(c.to_string(), style));
        }

        current_line_spans.extend(chars);

        if word_idx < word_positions.len() - 1 {
            current_line_spans.push(Span::styled(
                " ",
                Style::default()
                    .fg(theme.muted())
                    .add_modifier(Modifier::DIM),
            ));
        }
    }

    if !current_line_spans.is_empty() {
        lines.push(Line::from(current_line_spans));
    }

    Text::from(lines)
}

pub fn typing_area(f: &mut Frame, termi: &mut Termi, area: Rect) {
    // NOTE: i'm sure this is not the best way to go about this, but here we are.
    // enforce min and max height to be `AMOUNT_OF_VISIBLE_LINES`.
    let min_height = termi.config.visible_lines as u16;
    let max_height = termi.config.visible_lines as u16;
    let area = Rect {
        height: area.height.clamp(min_height, max_height),
        ..area
    };

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    let content_area = layout[1];
    let available_width = content_area.width as usize;

    let word_positions = calculate_word_positions(&termi.words, available_width);

    let cursor_pos = termi.tracker.cursor_position;
    let current_word_pos = word_positions
        .iter()
        .rev()
        .find(|pos| cursor_pos >= pos.start_index)
        .unwrap_or(&word_positions[0]);

    let visible_lines = content_area.height as usize;
    let current_line = current_word_pos.line;

    let scroll_offset = current_line.saturating_sub(visible_lines.saturating_sub(2));

    let text = typing_text(termi, &word_positions);

    let visible_lines_count = content_area.height as usize;
    let start = scroll_offset;
    let end = (start + visible_lines_count).min(text.lines.len());

    let visible_lines_capacity = end.saturating_sub(start);
    let mut visible_lines = Vec::with_capacity(visible_lines_capacity);

    for i in start..end {
        if let Some(line) = text.lines.get(i) {
            visible_lines.push(line.clone());
        }
    }

    let visible_text = Text::from(visible_lines);

    let typing_area = Paragraph::new(visible_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(typing_area, content_area);

    // only show cursor while IDLE or TYPING
    if (termi.tracker.status == Status::Idle || termi.tracker.status == Status::Typing)
        && !termi.has_floating_box_open()
    {
        // adjust for accounting scroll offset
        let offset = termi.tracker.cursor_position - current_word_pos.start_index;
        let x = content_area.x + (current_word_pos.col + offset) as u16;
        let y = content_area.y + (current_word_pos.line.saturating_sub(scroll_offset)) as u16;

        f.set_cursor_position(Position::new(x, y));
    }
}

// TODO: this could be simplified I think
pub fn top_bar(f: &mut Frame, termi: &mut Termi, area: Rect) {
    let is_time_mode = matches!(termi.config.current_mode(), Mode::Time { .. });

    let elements = vec![
        UIElement::new(
            "@ punctuation ",
            termi.config.use_punctuation,
            Some(ClickAction::TogglePunctuation),
        ),
        UIElement::new(
            "# numbers ",
            termi.config.use_numbers,
            Some(ClickAction::ToggleNumbers),
        ),
        UIElement::new(
            "@ symbols ",
            termi.config.use_symbols,
            Some(ClickAction::ToggleSymbols),
        ),
        UIElement::new("│ ", false, None), // separator
        UIElement::new(
            "⏱ time ",
            is_time_mode,
            Some(ClickAction::SwitchMode(ModeType::Time)),
        ),
        UIElement::new(
            "A words ",
            !is_time_mode,
            Some(ClickAction::SwitchMode(ModeType::Words)),
        ),
        UIElement::new("│ ", false, None), // separator
        UIElement::new(
            format!("{} ", if is_time_mode { "15" } else { "10" }),
            termi.config.current_mode().value() == if is_time_mode { 15 } else { 10 },
            Some(ClickAction::SetModeValue(if is_time_mode {
                15
            } else {
                10
            })),
        ),
        UIElement::new(
            format!("{} ", if is_time_mode { "30" } else { "25" }),
            termi.config.current_mode().value() == if is_time_mode { 30 } else { 25 },
            Some(ClickAction::SetModeValue(if is_time_mode {
                30
            } else {
                25
            })),
        ),
        UIElement::new(
            format!("{} ", if is_time_mode { "60" } else { "50" }),
            termi.config.current_mode().value() == if is_time_mode { 60 } else { 50 },
            Some(ClickAction::SetModeValue(if is_time_mode {
                60
            } else {
                50
            })),
        ),
        UIElement::new(
            format!("{} ", if is_time_mode { "120" } else { "100" }),
            termi.config.current_mode().value() == if is_time_mode { 120 } else { 100 },
            Some(ClickAction::SetModeValue(if is_time_mode {
                120
            } else {
                100
            })),
        ),
    ];

    let total_width: u16 = elements.iter().map(|e| e.width).sum();
    let start_x = area.x + (area.width.saturating_sub(total_width)) / 2;

    let mut current_x = start_x;
    let mut spans = Vec::new();

    for element in &elements {
        spans.push(element.to_span(termi.get_current_theme()));

        if let Some(action) = &element.action {
            termi.clickable_regions.push(ClickableRegion {
                area: Rect {
                    x: current_x,
                    y: area.y,
                    width: element.width,
                    height: 1,
                },
                action: action.clone(),
            });
        }

        current_x += element.width;
    }

    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

pub fn command_bar(f: &mut Frame, termi: &Termi, area: Rect) {
    let theme = termi.get_current_theme();
    fn styled_span(content: String, is_key: bool, theme: &Theme) -> Span<'static> {
        if is_key {
            return Span::styled(
                content,
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            );
        }
        Span::styled(
            content,
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        )
    }

    let command_groups = [
        vec![vec![("", false)]],
        vec![
            vec![
                ("tab", true),
                (" + ", false),
                ("enter", true),
                (" - restart test", false),
            ],
            vec![("esc", true), (" - menu", false)],
        ],
        vec![vec![("", false)]],
        vec![vec![
            ("ctrl", true),
            (" + ", false),
            ("c", true),
            (" or ", false),
            ("ctrl", true),
            (" + ", false),
            ("z", true),
            (" - to quit", false),
        ]],
    ];

    let lines: Vec<Line<'static>> = command_groups
        .iter()
        .map(|line_groups| {
            let total_width: usize = line_groups
                .iter()
                .enumerate()
                .map(|(i, group)| {
                    let group_width: usize = group.iter().map(|(text, _)| text.len()).sum();
                    group_width + if i < line_groups.len() - 1 { 3 } else { 0 }
                })
                .sum();

            let left_padding = (area.width as usize).saturating_sub(total_width) / 2;

            let mut spans = Vec::new();
            spans.push(styled_span(" ".repeat(left_padding), false, theme));

            for (i, group) in line_groups.iter().enumerate() {
                let group_spans: Vec<Span<'static>> = group
                    .iter()
                    .map(|&(text, is_key)| styled_span(text.to_string(), is_key, theme))
                    .collect();
                spans.extend(group_spans);

                if i < line_groups.len() - 1 {
                    spans.push(styled_span("   ".to_string(), false, theme));
                }
            }

            Line::from(spans)
        })
        .collect();

    f.render_widget(Paragraph::new(lines), area);
}

pub fn footer(f: &mut Frame, termi: &mut Termi, area: Rect) {
    let elements = vec![
        UIElement::new(" ", false, None),
        UIElement::new("ⓘ about", termi.about_open, Some(ClickAction::ToggleAbout)),
        UIElement::new(" ", false, None),
        UIElement::new(line::DOUBLE_VERTICAL_LEFT, false, None),
        UIElement::new(" ", false, None),
        UIElement::new(
            termi.theme.identifier.clone(),
            termi.menu.get_preview_theme().is_some(),
            Some(ClickAction::ToggleThemePicker),
        ),
        UIElement::new(" ", false, None),
        UIElement::new(line::DOUBLE_VERTICAL_RIGHT, false, None),
        UIElement::new(" ", false, None),
        UIElement::new(VERSION.to_string(), false, None),
    ];

    let total_width: u16 = elements.iter().map(|e| e.width).sum();
    let start_x = area.x + (area.width.saturating_sub(total_width)) / 2;
    let mut current_x = start_x;

    let spans: Vec<Span> = elements
        .iter()
        .map(|element| {
            let span = element.to_span(termi.get_current_theme());

            if let Some(action) = &element.action {
                termi.clickable_regions.push(ClickableRegion {
                    area: Rect {
                        x: current_x,
                        y: area.y,
                        width: element.width,
                        height: 1,
                    },
                    action: action.clone(),
                });
            }

            current_x += element.width;
            span
        })
        .collect();

    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

pub fn results_screen(f: &mut Frame, termi: &mut Termi, area: Rect) {
    f.render_widget(create_results_widget(termi, area), area);
}

fn create_results_widget(termi: &Termi, area: Rect) -> Paragraph<'static> {
    let theme = termi.get_current_theme();

    let mode_display = match termi.config.current_mode() {
        Mode::Time { duration } => format!("{} seconds", duration),
        Mode::Words { count } => format!("{} words", count),
    };

    let mode_type = match termi.config.current_mode() {
        Mode::Time { .. } => "Time",
        Mode::Words { .. } => "Words",
    };

    let total_chars = termi.tracker.total_keystrokes;
    let correct_chars = termi.tracker.correct_keystrokes;
    let wrong_chars = total_chars.saturating_sub(correct_chars);
    let wpm = termi.tracker.wpm.round() as u32;
    let raw_wpm = termi.tracker.raw_wpm.round() as u32;
    let accuracy = termi.tracker.accuracy;
    let language = termi.config.language.as_str();

    let elapsed_seconds = termi.tracker.completion_time.unwrap_or(0.0).round() as u32;
    let minutes = elapsed_seconds / 60;
    let seconds = elapsed_seconds % 60;

    let mut content_lines: Vec<Line<'static>> = Vec::new();

    let stats_offset = if area.width >= FULL_LOGO_MIN_WIDTH {
        35
    } else {
        15
    };
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let hostname = "termitype";
    let header = format!("{}@{}", username, hostname);
    let separator = "─".repeat(header.chars().count());

    content_lines.push(Line::from(vec![
        Span::raw(" ".repeat(stats_offset)),
        Span::styled(header, Style::default().fg(theme.highlight())),
    ]));
    content_lines.push(Line::from(vec![
        Span::raw(" ".repeat(stats_offset)),
        Span::styled(separator, Style::default().fg(theme.highlight())),
    ]));

    let stats = vec![
        ("OS", format!("termitype {}", VERSION)),
        ("Lang", language.to_string()),
        ("Mode", format!("{} ({})", mode_type, mode_display)),
        ("WPM", format!("{} wpm", wpm)),
        ("Raw", format!("{} wpm", raw_wpm)),
        ("Accuracy", format!("{}%", accuracy)),
        ("Time", format!("{}m {}s", minutes, seconds)),
        ("Keystrokes", format!("{} ({}%)", total_chars, accuracy)),
        ("Correct", format!("{} chars", correct_chars)),
        ("Errors", format!("{} chars", wrong_chars)),
        (
            "Consistency",
            format!("{:.1}%", (raw_wpm as f64 / wpm as f64 * 100.0).min(100.0)),
        ),
    ];

    let logo = if area.width >= FULL_LOGO_MIN_WIDTH {
        APP_LOGO
    } else {
        &[]
    };

    for (i, (label, value)) in stats.iter().enumerate() {
        let mut line = Vec::new();

        if i < logo.len() {
            line.push(Span::styled(
                format!("{:width$}", logo[i], width = stats_offset),
                Style::default().fg(theme.highlight()),
            ));
        } else {
            line.push(Span::raw(" ".repeat(stats_offset)));
        }

        line.push(Span::styled(
            format!("{}: ", label),
            Style::default().fg(theme.highlight()),
        ));
        line.push(Span::styled(
            value.clone(),
            Style::default().fg(theme.muted()),
        ));

        content_lines.push(Line::from(line));
    }

    content_lines.push(Line::default());

    let mut color_blocks = Vec::new();
    color_blocks.push(Span::raw(" ".repeat(stats_offset)));

    for color in [
        theme.accent(),
        theme.error(),
        theme.success(),
        theme.warning(),
        theme.info(),
        theme.highlight(),
        theme.muted(),
    ] {
        color_blocks.push(Span::styled("██", Style::default().fg(color)));
    }

    content_lines.push(Line::from(color_blocks));

    content_lines.push(Line::default());
    content_lines.push(Line::default());

    // TODO: reuse `command_bar`
    let bottom_hints = vec![
        vec![
            ("tab", true),
            (" + ", false),
            ("enter", true),
            (" restart test", false),
        ],
        vec![
            ("ctrl", true),
            (" + ", false),
            ("c", true),
            (" quit", false),
        ],
        vec![("esc", true), (" menu", false)],
    ];

    for hint_group in &bottom_hints {
        let mut spans = Vec::new();

        let total_length: usize = hint_group.iter().map(|(text, _)| text.len()).sum();
        let center_position = (area.width.saturating_sub(total_length as u16)) / 2;

        spans.push(Span::raw(" ".repeat(center_position as usize)));

        for &(text, is_key) in hint_group {
            let span = if is_key {
                Span::styled(
                    text.to_string(),
                    Style::default()
                        .fg(theme.highlight())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    text.to_string(),
                    Style::default()
                        .fg(theme.muted())
                        .add_modifier(Modifier::DIM),
                )
            };
            spans.push(span);
        }

        content_lines.push(Line::from(spans));
    }

    Paragraph::new(content_lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_word_position_basic() {
        let text = "hello world";
        let available_width = 20;
        let positions = calculate_word_positions(text, available_width);

        assert_eq!(positions.len(), 2, "Should have positions for two words");
        assert_eq!(positions[0].start_index, 0, "First word starts at 0");
        assert_eq!(positions[0].line, 0, "First word on line 0");
        assert_eq!(positions[0].col, 0, "First word at column 0");

        assert_eq!(
            positions[1].start_index, 6,
            "Second word starts after 'hello '"
        );
        assert_eq!(positions[1].line, 0, "Second word on line 0");
        assert_eq!(positions[1].col, 6, "Second word after first word + space");
    }

    #[test]
    fn test_word_position_wrapping() {
        let text = "hello world wrap";
        let available_width = 8; // force wrap after "hello"
        let positions = calculate_word_positions(text, available_width);

        assert_eq!(positions[0].line, 0, "First word on line 0");
        assert_eq!(positions[1].line, 1, "Second word should wrap to line 1");
        assert_eq!(positions[1].col, 0, "Wrapped word starts at column 0");
        assert_eq!(positions[2].line, 2, "Third word on line 2");
    }

    #[test]
    fn test_cursor_positions() {
        let text = "hello world next";
        let available_width = 20;
        let positions = calculate_word_positions(text, available_width);

        let test_positions = vec![
            (0, 0, "Start of text"),
            (5, 0, "End of first word"),
            (6, 1, "Start of second word"),
            (11, 1, "End of second word"),
            (12, 2, "Start of third word"),
        ];

        for (cursor_pos, expected_word_idx, description) in test_positions {
            let current_pos = positions
                .iter()
                .rev()
                .find(|pos| cursor_pos >= pos.start_index)
                .unwrap();

            assert_eq!(
                positions
                    .iter()
                    .position(|p| p.start_index == current_pos.start_index)
                    .unwrap(),
                expected_word_idx,
                "{}",
                description
            );
        }
    }

    #[test]
    fn test_empty_text() {
        let text = "";
        let available_width = 10;
        let positions = calculate_word_positions(text, available_width);
        assert!(positions.is_empty(), "Empty text should have no positions");
    }
}
