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
    constants::{
        AMOUNT_OF_VISIBLE_LINES, APPNAME, COMMAND_BAR_HEIGHT, FOOTER_HEIGHT, MIN_TYPING_HEIGHT,
    },
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
    ToggleNumbers,
    SwitchMode(ModeType),
    SetModeValue(usize),
    OpenThemePicker,
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

pub fn progress_info(f: &mut Frame, termi: &Termi, area: Rect) {
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
        let total_len = word_len + 1; // word + space

        // do we need to wrap?
        if current_col + word_len > available_width {
            current_line += 1;
            current_col = 0;
        }

        positions.push(WordPosition {
            start_index: current_index,
            line: current_line,
            col: current_col,
        });

        current_col += total_len;
        current_index += total_len;
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

        // style
        let word_start = pos.start_index;
        let chars: Vec<Span> = word
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let char_idx = word_start + i;
                let style = match termi.tracker.user_input.get(char_idx).copied().flatten() {
                    Some(input) if input == c => Style::default().fg(theme.success()),
                    Some(_) => Style::default()
                        .fg(theme.error())
                        .add_modifier(Modifier::DIM),
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
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    let available_width = layout[1].width as usize;
    let word_positions = calculate_word_positions(&termi.words, available_width);

    let current_word_pos = word_positions
        .iter()
        .rev()
        .find(|pos| termi.tracker.cursor_position >= pos.start_index)
        .unwrap_or(&word_positions[0]);

    // let visible_lines = layout[1].height as usize;
    let visible_lines = AMOUNT_OF_VISIBLE_LINES as usize;
    let current_line = current_word_pos.line;

    let scroll_offset = if current_line > visible_lines / 2 {
        current_line - visible_lines / 2
    } else {
        0
    };

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
    let y = layout[1].y + (current_word_pos.line - scroll_offset) as u16;

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
                format!(" {} ", content),
                Style::default()
                    .fg(theme.selection_fg())
                    .bg(theme.selection_bg())
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

pub fn footer(f: &mut Frame, termi: &Termi, area: Rect) -> Vec<ClickableRegion> {
    let mut regions = Vec::new();

    let elements = vec![
        UIElement::new(" ", false, None),
        UIElement::new("github.com/emanuel2718/termitype", false, None),
        UIElement::new(" ", false, None),
        UIElement::new(line::DOUBLE_VERTICAL_LEFT, false, None),
        UIElement::new(" ", false, None),
        UIElement::new(
            termi.theme.identifier.clone(),
            false,
            Some(ClickAction::OpenThemePicker),
        ),
        UIElement::new("    ", false, None),
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
                regions.push(ClickableRegion {
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

    regions
}

pub fn results_screen(f: &mut Frame, termi: &Termi, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),                     // space
            Constraint::Min(MIN_TYPING_HEIGHT),     // results
            Constraint::Min(1),                     // space
            Constraint::Length(COMMAND_BAR_HEIGHT), // command bar
            Constraint::Length(FOOTER_HEIGHT),      // footer
        ])
        .split(area);

    f.render_widget(create_results_widget(termi), layout[1]);
    command_bar(f, termi, layout[3]);
    footer(f, termi, layout[4]);
}

fn create_results_widget(termi: &Termi) -> Paragraph<'static> {
    let theme = termi.get_current_theme();
    let results = vec![
        ("wpm", format!("{:.0}", termi.tracker.wpm)),
        ("acc", format!("{}%", termi.tracker.accuracy)),
        (
            "time",
            format!("{:.1}s", termi.tracker.completion_time.unwrap_or(0.0)),
        ),
    ];

    let content_lines: Vec<Line> = results
        .into_iter()
        .map(|(label, value)| {
            Line::from(vec![
                Span::raw(format!("{}: ", label))
                    .fg(theme.muted())
                    .add_modifier(Modifier::DIM),
                Span::styled(value, Style::default().fg(theme.foreground())),
            ])
        })
        .collect();

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
        println!("{:?}", positions);

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
