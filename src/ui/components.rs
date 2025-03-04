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
    constants::APPNAME,
    termi::Termi,
    theme::Theme,
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

    fn to_span(&self, theme: &Theme) -> Span<'static> {
        Span::styled(
            self.content.clone(),
            Style::default()
                .fg(if self.is_active {
                    theme.highlight()
                } else {
                    theme.muted()
                })
                .add_modifier(if self.is_active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        )
    }
}

pub fn title(f: &mut Frame, termi: &Termi, area: Rect) {
    let theme = termi.get_current_theme();
    let title = Paragraph::new(APPNAME)
        .style(Style::default().fg(theme.highlight()))
        .add_modifier(Modifier::BOLD)
        .alignment(Alignment::Center);
    f.render_widget(title, area);
}

pub fn progress_info(f: &mut Frame, termi: &mut Termi, area: Rect) {
    if termi.tracker.status == crate::tracker::Status::Idle {
        let language = termi.config.language.as_str();
        let language_text = format!("語 {}", language);

        let element = UIElement::new(language_text, false, Some(ClickAction::OpenLanguagePicker));

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
struct WordPosition {
    start_index: usize,
    line: usize,
    col: usize,
}

fn calculate_word_positions(text: &str, available_width: usize) -> Vec<WordPosition> {
    let mut positions = Vec::new();
    let mut current_line = 0;
    let mut current_col = 0;
    let mut current_index = 0;

    for word in text.split_whitespace() {
        let word_len = word.len();

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

    for (word_idx, pos) in word_positions.iter().enumerate() {
        // make new line if we have to
        if pos.line > current_line {
            lines.push(Line::from(current_line_spans.clone()));
            current_line_spans.clear();
            current_line = pos.line;
        }

        let word = termi.words.split_whitespace().nth(word_idx).unwrap();
        let word_start = pos.start_index;
        let is_current_word = termi.tracker.cursor_position >= word_start
            && termi.tracker.cursor_position <= word_start + word.len();
        let is_wrong_word = !is_current_word && termi.tracker.is_word_wrong(word_start);

        // style
        let chars: Vec<Span> = word
            .chars()
            .enumerate()
            .map(|(i, c)| {
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
                Span::styled(c.to_string(), style)
            })
            .collect();

        current_line_spans.extend(chars);

        // add space after word (except for last word)
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

pub fn typing_area(f: &mut Frame, termi: &Termi, area: Rect) {
    // NOTE: i'm sure this is not the best way to go about this, but here we are.
    // enfore min and max height to be `AMOUNT_OF_VISIBLE_LINES`.
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

    let available_width = layout[1].width as usize;
    let word_positions = calculate_word_positions(&termi.words, available_width);

    let current_word_pos = word_positions
        .iter()
        .rev()
        .find(|pos| termi.tracker.cursor_position >= pos.start_index)
        .unwrap_or(&word_positions[0]);

    let visible_lines = layout[1].height as usize;
    let current_line = current_word_pos.line;

    let scroll_offset = current_line.saturating_sub(visible_lines.saturating_sub(2));

    let text = typing_text(termi, &word_positions);
    let visible_text = Text::from(
        text.lines
            .iter()
            .skip(scroll_offset)
            .take(visible_lines)
            .cloned()
            .collect::<Vec<_>>(),
    );

    let typing_area = Paragraph::new(visible_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(typing_area, layout[1]);

    // adjust for accounting scroll offset
    let offset = termi.tracker.cursor_position - current_word_pos.start_index;
    let x = layout[1].x + (current_word_pos.col + offset) as u16;
    let y = layout[1].y + (current_word_pos.line.saturating_sub(scroll_offset)) as u16;

    f.set_cursor_position(Position::new(x, y));
}

// TODO: this could be simplified I think
pub fn top_bar(f: &mut Frame, termi: &mut Termi, area: Rect) {
    termi.clickable_regions.clear();

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
        spans.push(element.to_span(&termi.theme));

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

// TODO: this could be simplified I think
pub fn command_bar(f: &mut Frame, termi: &Termi, area: Rect) {
    let theme = termi.get_current_theme();
    fn styled_span(content: &str, is_key: bool, theme: &Theme) -> Span<'static> {
        if is_key {
            return Span::styled(
                content.to_string(),
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            );
        }
        Span::styled(
            content.to_string(),
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        )
    }

    let command_groups = [
        vec![vec![("", false)]], // spacing
        vec![
            vec![
                ("tab", true),
                (" + ", false),
                ("enter", true),
                (" - restart test", false),
            ],
            vec![("esc", true), (" - menu", false)],
        ],
        vec![vec![("", false)]], // spacing
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

    let lines: Vec<Line> = command_groups
        .iter()
        .map(|line_groups| {
            let spans: Vec<Span> = line_groups
                .iter()
                .enumerate()
                .flat_map(|(i, group)| {
                    let mut group_spans: Vec<Span> = group
                        .iter()
                        .map(|&(text, is_key)| styled_span(text, is_key, theme))
                        .collect();

                    if i < line_groups.len() - 1 {
                        group_spans.push(styled_span("   ", false, theme));
                    }

                    group_spans
                })
                .collect();

            Line::from(spans)
        })
        .collect();

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
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
            let span = element.to_span(&termi.theme);

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

    let test_type = match termi.config.current_mode() {
        Mode::Time { duration } => format!("{}s {}", duration, termi.config.language),
        Mode::Words { count } => format!("{} words {}", count, termi.config.language),
    };

    let total_chars = termi.tracker.total_keystrokes;
    let correct_chars = termi.tracker.correct_keystrokes;
    let wrong_chars = total_chars.saturating_sub(correct_chars);

    let wpm = format!("{}", termi.tracker.wpm.round() as u32);
    let raw_wpm = format!("{}", termi.tracker.raw_wpm.round() as u32);
    let accuracy = format!("{}%", termi.tracker.accuracy);

    let mut content_lines = Vec::new();

    // we are running out of space
    if area.height <= 3 {
        content_lines.push(Line::from(vec![
            Span::styled(wpm, Style::default().fg(theme.highlight())),
            Span::styled(" wpm", Style::default().fg(theme.muted())),
        ]));
    }
    // compact mode (4-6 lines)
    else if area.height <= 6 {
        content_lines.push(Line::from(vec![
            Span::styled(wpm, Style::default().fg(theme.highlight())),
            Span::styled(" wpm", Style::default().fg(theme.muted())),
        ]));
        content_lines.push(Line::from(vec![Span::styled(
            test_type,
            Style::default().fg(theme.foreground()),
        )]));
        content_lines.push(Line::from(vec![
            Span::styled(accuracy, Style::default().fg(theme.info())),
            Span::styled(" accuracy", Style::default().fg(theme.muted())),
        ]));
    }
    // full results
    else {
        content_lines.push(Line::default());
        content_lines.push(Line::from(vec![
            Span::styled(
                wpm,
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" wpm", Style::default().fg(theme.muted())),
        ]));
        content_lines.push(Line::default());

        content_lines.push(Line::from(vec![Span::styled(
            test_type,
            Style::default().fg(theme.foreground()),
        )]));
        content_lines.push(Line::default());

        content_lines.push(Line::from(vec![
            Span::styled(raw_wpm, Style::default().fg(theme.foreground())),
            Span::styled(" raw", Style::default().fg(theme.muted())),
            Span::raw("  "),
            Span::styled(accuracy, Style::default().fg(theme.info())),
            Span::styled(" accuracy", Style::default().fg(theme.muted())),
        ]));

        content_lines.push(Line::from(vec![
            Span::styled(
                format!("{}", correct_chars),
                Style::default().fg(theme.success()),
            ),
            Span::styled(" correct", Style::default().fg(theme.muted())),
            Span::raw("  "),
            Span::styled(
                format!("{}", wrong_chars),
                Style::default().fg(theme.error()),
            ),
            Span::styled(" errors", Style::default().fg(theme.muted())),
        ]));

        if area.height > 8 {
            content_lines.push(Line::default());
            content_lines.push(Line::from(vec![
                Span::styled("tab", Style::default().fg(theme.highlight())),
                Span::styled(" + ", Style::default().fg(theme.muted())),
                Span::styled("enter", Style::default().fg(theme.highlight())),
                Span::styled(" restart", Style::default().fg(theme.muted())),
                Span::raw("  "),
                Span::styled("ctrl", Style::default().fg(theme.highlight())),
                Span::styled(" + ", Style::default().fg(theme.muted())),
                Span::styled("q", Style::default().fg(theme.highlight())),
                Span::styled(" menu", Style::default().fg(theme.muted())),
            ]));
        }
    }

    Paragraph::new(content_lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
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
