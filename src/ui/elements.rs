use crate::{
    config::{Mode, ModeType},
    constants::{APPNAME, DEFAULT_LANGUAGE, MIN_TERM_HEIGHT, MIN_TERM_WIDTH},
    termi::Termi,
    theme::Theme,
    tracker::Status,
    version::VERSION,
};
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style, Stylize},
    symbols::line::DOUBLE_VERTICAL_LEFT,
    text::{Line, Span, Text},
    widgets::ListItem,
};

use super::actions::TermiClickAction;

#[derive(Debug)]
pub struct TermiElement<'a> {
    pub content: Text<'a>,
    pub action: Option<TermiClickAction>,
    pub is_active: bool,
}

impl<'a> TermiElement<'a> {
    pub fn new(
        content: impl Into<Text<'a>>,
        is_active: bool,
        action: Option<TermiClickAction>,
    ) -> Self {
        Self {
            content: content.into(),
            is_active,
            action,
        }
    }

    pub fn to_styled(mut self, theme: &Theme) -> Self {
        let style_to_apply = if self.is_active {
            Style::default()
                .fg(theme.highlight())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM)
        };

        for line in self.content.lines.iter_mut() {
            for span in line.spans.iter_mut() {
                span.style = span.style.patch(style_to_apply);
            }
        }
        self
    }
}

pub fn create_header(termi: &Termi) -> Vec<TermiElement> {
    let theme = termi.get_current_theme();
    let text = Text::from(APPNAME)
        .alignment(Alignment::Left)
        .style(Style::default().fg(theme.highlight()))
        .patch_style(if termi.tracker.status == Status::Typing {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            Style::default()
        });
    vec![TermiElement::new(text, false, None)]
}

pub fn create_action_bar(termi: &Termi) -> Vec<TermiElement> {
    let theme = termi.get_current_theme().clone();
    let is_time_mode = matches!(termi.config.current_mode(), Mode::Time { .. });

    let elements = vec![
        TermiElement::new(
            "@ punctuation ",
            termi.config.use_punctuation,
            Some(TermiClickAction::TogglePunctuation),
        ),
        TermiElement::new(
            "# numbers ",
            termi.config.use_numbers,
            Some(TermiClickAction::ToggleNumbers),
        ),
        TermiElement::new(
            "@ symbols ",
            termi.config.use_symbols,
            Some(TermiClickAction::ToggleSymbols),
        ),
        TermiElement::new("│ ", false, None),
        TermiElement::new(
            "⏱ time ",
            is_time_mode,
            Some(TermiClickAction::SwitchMode(ModeType::Time)),
        ),
        TermiElement::new(
            "A words ",
            !is_time_mode,
            Some(TermiClickAction::SwitchMode(ModeType::Words)),
        ),
        TermiElement::new("│ ", false, None),
        TermiElement::new(
            format!("{} ", if is_time_mode { 15 } else { 10 }),
            termi.config.current_mode().value() == if is_time_mode { 15 } else { 10 },
            Some(TermiClickAction::SetModeValue(if is_time_mode {
                15
            } else {
                10
            })),
        ),
        TermiElement::new(
            format!("{} ", if is_time_mode { 30 } else { 25 }),
            termi.config.current_mode().value() == if is_time_mode { 30 } else { 25 },
            Some(TermiClickAction::SetModeValue(if is_time_mode {
                30
            } else {
                25
            })),
        ),
        TermiElement::new(
            format!("{} ", if is_time_mode { 60 } else { 50 }),
            termi.config.current_mode().value() == if is_time_mode { 60 } else { 50 },
            Some(TermiClickAction::SetModeValue(if is_time_mode {
                60
            } else {
                50
            })),
        ),
        TermiElement::new(
            format!("{} ", if is_time_mode { 120 } else { 100 }),
            termi.config.current_mode().value() == if is_time_mode { 120 } else { 100 },
            Some(TermiClickAction::SetModeValue(if is_time_mode {
                120
            } else {
                100
            })),
        ),
    ];

    elements
        .into_iter()
        .map(|element| element.to_styled(&theme))
        .collect()
}

pub fn create_mode_bar(termi: &Termi) -> Vec<TermiElement> {
    let status = termi.tracker.status.clone();
    let theme = termi.get_current_theme().clone();
    let element = match status {
        Status::Idle | Status::Paused => {
            let current_language = termi.config.language.as_deref().unwrap_or(DEFAULT_LANGUAGE);
            let text = Text::from(current_language)
                .style(Style::new().fg(theme.muted()))
                .add_modifier(Modifier::DIM)
                .alignment(Alignment::Center);
            TermiElement::new(text, false, Some(TermiClickAction::ToggleLanguagePicker))
        }
        Status::Typing => {
            let info = match termi.config.current_mode() {
                Mode::Time { duration } => {
                    if let Some(rem) = termi.tracker.time_remaining {
                        format!("{:.0}", rem.as_secs())
                    } else {
                        format!("{}", duration)
                    }
                }
                Mode::Words { count } => {
                    let completed = termi
                        .tracker
                        .user_input
                        .iter()
                        .filter(|&&c| c == Some(' '))
                        .count();
                    format!("{}/{}", completed, count)
                }
            };
            let wpm = format!(" {:>3.0} wpm", termi.tracker.wpm);
            let spans = vec![
                Span::styled(info, Style::default().fg(theme.highlight())),
                Span::styled(
                    wpm,
                    Style::default()
                        .fg(theme.muted())
                        .add_modifier(Modifier::DIM),
                ),
            ];
            let line = Line::from(spans);
            let text = Text::from(line);
            TermiElement::new(text, false, None)
        }
        _ => TermiElement::new(Text::raw(""), false, None),
    };
    vec![element]
}

fn create_styled_typing_text<'a>(termi: &'a Termi, theme: &Theme) -> Text<'a> {
    let mut spans = Vec::with_capacity(termi.words.len());
    let mut current_char_index = 0;

    for word in termi.words.split_inclusive(' ') {
        for (i, c) in word.chars().enumerate() {
            let char_idx = current_char_index + i;
            let style = match termi.tracker.user_input.get(char_idx).copied().flatten() {
                Some(input) if input == c => Style::default().fg(theme.success()),
                Some(_) => Style::default().fg(theme.error()),
                None => {
                    if c == ' ' {
                        Style::default()
                            .fg(theme.muted())
                            .add_modifier(Modifier::DIM)
                    } else {
                        Style::default().fg(theme.fg()).add_modifier(Modifier::DIM)
                    }
                }
            };
            spans.push(Span::styled(c.to_string(), style));
        }
        current_char_index += word.chars().count();
    }

    Text::from(Line::from(spans))
}

pub fn create_typing_area(termi: &Termi) -> Vec<TermiElement> {
    let theme = termi.get_current_theme().clone();
    let text = create_styled_typing_text(termi, &theme);
    vec![TermiElement::new(text, false, None)]
}

pub fn create_command_bar(termi: &Termi) -> Vec<TermiElement> {
    let theme = termi.get_current_theme();

    fn styled_span(content: String, is_key: bool, theme: &Theme) -> Span<'static> {
        let style = if is_key {
            Style::default()
                .fg(theme.highlight())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM)
        };
        Span::styled(content, style)
    }

    let command_groups = [
        vec![
            vec![
                ("tab", true),
                (" + ", false),
                ("enter", true),
                (" - restart test", false),
            ],
            vec![("esc", true), (" - menu", false)],
        ],
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

    let mut lines = Vec::new();
    for (row_idx, line_groups) in command_groups.iter().enumerate() {
        let mut spans = Vec::new();
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
        lines.push(Line::from(spans).alignment(Alignment::Center));

        if row_idx == 0 {
            lines.push(Line::raw("").alignment(Alignment::Center));
        }
    }

    let text = Text::from(lines).alignment(Alignment::Center);
    vec![TermiElement::new(text, false, None)]
}

pub fn create_footer<'a>(termi: &Termi) -> Vec<TermiElement<'a>> {
    let theme = termi.get_current_theme().clone();
    let elements = vec![
        TermiElement::new(
            "ⓘ about",
            termi.menu.is_about_menu(),
            Some(TermiClickAction::ToggleAbout),
        ),
        TermiElement::new(" ", false, None),
        TermiElement::new(DOUBLE_VERTICAL_LEFT, false, None),
        TermiElement::new(" ", false, None),
        TermiElement::new(
            termi.theme.id.clone(),
            termi.preview_theme.is_some(),
            Some(TermiClickAction::ToggleThemePicker),
        ),
        TermiElement::new(" ", false, None),
        TermiElement::new(DOUBLE_VERTICAL_LEFT, false, None),
        TermiElement::new(" ", false, None),
        TermiElement::new(VERSION, false, None),
    ];

    elements
        .into_iter()
        .map(|element| element.to_styled(&theme))
        .collect()
}

pub fn create_minimal_size_warning(termi: &Termi, width: u16, height: u16) -> Vec<TermiElement> {
    let theme = termi.get_current_theme().clone();
    let warning_lines = vec![
        Line::from(Span::styled(
            "! too small",
            Style::default().fg(theme.error()),
        )),
        Line::from(Span::styled(
            format!("Current: ({}x{})", width, height),
            Style::default().fg(theme.muted()),
        )),
        Line::from(Span::styled(
            format!("Minimum: ({}x{})", MIN_TERM_WIDTH, MIN_TERM_HEIGHT),
            Style::default().fg(theme.muted()),
        )),
    ];
    let text = Text::from(warning_lines).alignment(Alignment::Center);
    vec![TermiElement::new(text, false, None)]
}

pub fn create_show_menu_button(_termi: &Termi) -> Vec<TermiElement> {
    let text = Text::raw("TODO: <icon> Show Menu").alignment(Alignment::Center);
    vec![TermiElement::new(text, false, None)]
}

pub fn create_menu_footer_text(termi: &Termi) -> Line {
    let theme = termi.get_current_theme();
    let menu_state = &termi.menu;

    if menu_state.is_searching() {
        Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(theme.accent())),
            Span::styled(menu_state.search_query(), Style::default().fg(theme.fg())),
            Span::styled(
                "█",
                Style::default()
                    .fg(theme.cursor())
                    .add_modifier(Modifier::RAPID_BLINK),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("[↑/k]", Style::default().fg(theme.highlight())),
            Span::styled(" up ", Style::default().fg(theme.muted())),
            Span::styled("[↓/j]", Style::default().fg(theme.highlight())),
            Span::styled(" down ", Style::default().fg(theme.muted())),
            Span::styled("[/]", Style::default().fg(theme.highlight())),
            Span::styled(" search ", Style::default().fg(theme.muted())),
            Span::styled("[ent]", Style::default().fg(theme.highlight())),
            Span::styled(" sel ", Style::default().fg(theme.muted())),
            Span::styled("[space]", Style::default().fg(theme.highlight())),
            Span::styled(" toggle ", Style::default().fg(theme.muted())),
            Span::styled("[esc]", Style::default().fg(theme.highlight())),
            Span::styled(" close", Style::default().fg(theme.muted())),
        ])
    }
}

pub fn prepare_menu_list_items(
    termi: &Termi,
    scroll_offset: usize,
    max_visible: usize,
) -> (Vec<ListItem>, usize) {
    let theme = termi.get_current_theme();

    if let Some(menu) = &termi.menu.current_menu() {
        let filtered_items: Vec<_> = if termi.menu.is_searching() {
            menu.filtered_items(termi.menu.search_query())
        } else {
            menu.items().iter().enumerate().collect()
        };

        let total_items = filtered_items.len();

        if total_items == 0 {
            let no_matches = vec![
                ListItem::new(""),
                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(
                        "grep: pattern not found",
                        Style::default().fg(theme.muted()),
                    ),
                ])),
            ];
            (no_matches, 0)
        } else {
            let items: Vec<ListItem> = std::iter::once(ListItem::new(""))
                .chain(
                    filtered_items
                        .iter()
                        .skip(scroll_offset)
                        .take(max_visible)
                        .map(|&(i, item)| {
                            let is_selected = i == menu.selected_index();
                            let item_style = Style::default()
                                .fg(if item.is_toggleable {
                                    if item.is_active {
                                        theme.highlight()
                                    } else {
                                        theme.muted()
                                    }
                                } else if is_selected {
                                    theme.selection_fg()
                                } else {
                                    theme.fg()
                                })
                                .bg(if is_selected {
                                    theme.selection_bg()
                                } else {
                                    theme.bg()
                                });

                            ListItem::new(Line::from(vec![
                                Span::styled("  ", Style::default()),
                                Span::styled(
                                    if is_selected { "❯ " } else { "  " },
                                    Style::default().fg(theme.accent()),
                                ),
                                Span::styled(&item.label, item_style),
                                if item.has_submenu {
                                    Span::styled(" →", Style::default().fg(theme.accent()))
                                } else {
                                    Span::raw("")
                                },
                            ]))
                        }),
                )
                .collect();
            (items, total_items)
        }
    } else {
        (vec![ListItem::new("  No menu content")], 0)
    }
}
