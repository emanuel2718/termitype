use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    symbols::line,
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
    Frame,
};

use crate::{
    config::{Mode, ModeType},
    constants::{APPNAME, COMMAND_BAR_HEIGHT, FOOTER_HEIGHT, MIN_TYPING_HEIGHT},
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
                    theme.highlight
                } else {
                    theme.inactive
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
    let title = Paragraph::new(APPNAME)
        .style(Style::default().fg(termi.theme.highlight))
        .add_modifier(Modifier::BOLD)
        .alignment(Alignment::Center);
    f.render_widget(title, area);
}

pub fn progress_info(f: &mut Frame, termi: &Termi, area: Rect) {
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

pub fn typing_area(f: &mut Frame, termi: &Termi, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    let typing_area = Paragraph::new(text(termi))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    f.render_widget(typing_area, layout[1]);
}

// TODO: do we like this? hmmm
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
    fn styled_span(content: &str, is_key: bool, theme: &Theme) -> Span<'static> {
        let mut style = Style::default().fg(theme.inactive);
        if is_key {
            style = style.fg(theme.background).bg(theme.foreground);
            return Span::styled(format!(" {} ", content), style.add_modifier(Modifier::BOLD));
        }

        Span::styled(content.to_string(), style)
    }

    let command_groups = vec![
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
                        .map(|&(text, is_key)| styled_span(text, is_key, &termi.theme))
                        .collect();

                    if i < line_groups.len() - 1 {
                        group_spans.push(styled_span("   ", false, &termi.theme));
                    }

                    group_spans
                })
                .collect();

            Line::from(spans)
        })
        .collect();

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

pub fn footer(f: &mut Frame, termi: &Termi, area: Rect) {
    let spans = vec![
        Span::styled(
            "github/emanuel2718",
            Style::default().fg(termi.theme.inactive),
        ),
        Span::raw("    "),
        Span::styled(
            termi.theme.identifier.clone(),
            Style::default().fg(termi.theme.inactive),
        ),
        Span::raw("    "),
        Span::styled(
            line::DOUBLE_VERTICAL_RIGHT,
            Style::default().fg(termi.theme.inactive),
        ),
        Span::raw(" "),
        Span::styled(
            VERSION.to_string(),
            Style::default().fg(termi.theme.inactive),
        ),
    ];

    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
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
                Span::raw(format!("{}: ", label)).fg(termi.theme.inactive),
                Span::styled(value, Style::default().fg(termi.theme.highlight)),
            ])
        })
        .collect();

    Paragraph::new(content_lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}

fn text(termi: &Termi) -> Text<'static> {
    let target_chars: Vec<char> = termi.words.chars().collect();

    let spans: Vec<Span> = target_chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let mut style = match termi.tracker.user_input.get(i).copied().flatten() {
                Some(input) if input == c => Style::default().fg(termi.theme.success),
                Some(_) => Style::default().fg(termi.theme.error),
                None => Style::default()
                    .fg(termi.theme.inactive)
                    .add_modifier(Modifier::DIM),
            };

            if i == termi.tracker.cursor_position {
                style = style.fg(termi.theme.cursor_text).bg(termi.theme.cursor)
            }

            Span::styled(c.to_string(), style)
        })
        .collect();

    Text::from(Line::from(spans))
}
