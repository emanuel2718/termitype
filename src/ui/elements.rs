use crate::{
    actions::TermiClickAction,
    config::{Mode, ModeType},
    constants::{
        APPNAME, DEFAULT_LANGUAGE, DEFAULT_TIME_DURATION_LIST, DEFAULT_WORD_COUNT_LIST,
        MIN_TERM_HEIGHT, MIN_TERM_WIDTH,
    },
    menu::{MenuItem, MenuItemResult},
    modal::ModalContext,
    termi::Termi,
    theme::Theme,
    tracker::Status,
    ui::utils::WordPosition,
    version::VERSION,
};
use ratatui::{
    layout::Alignment,
    style::Color,
    style::{Modifier, Style, Stylize},
    symbols::line::DOUBLE_VERTICAL_LEFT,
    text::{Line, Span, Text},
    widgets::ListItem,
};
use std::collections::VecDeque;

// PERF: pre-allocate span pools
thread_local! {
    static SPAN_POOL: std::cell::RefCell<VecDeque<String>> =
        std::cell::RefCell::new({
            let mut pool = VecDeque::with_capacity(1000);
            for _ in 0..1000 {
                pool.push_back(String::with_capacity(4)); // UTF-8 chars
            }
            pool
        });
}

fn get_pooled_string() -> String {
    SPAN_POOL.with(|pool| {
        pool.borrow_mut()
            .pop_front()
            .unwrap_or_else(|| String::with_capacity(4))
    })
}

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
    let theme = termi.current_theme();
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
    let theme = termi.current_theme();
    let config = &termi.config;
    let current_value = config.current_mode().value();
    let is_time_mode = matches!(config.current_mode(), Mode::Time { .. });
    let presets = if is_time_mode {
        DEFAULT_TIME_DURATION_LIST
    } else {
        DEFAULT_WORD_COUNT_LIST
    };

    let is_custom_active = !presets.contains(&{ current_value });

    // NOTE: this is okay because the <custom> on the action bar will only select between
    //       custom time or custom words by design
    let custom_ctx = if is_time_mode {
        ModalContext::CustomTime
    } else {
        ModalContext::CustomWordCount
    };

    let supports_unicode = theme.supports_unicode();
    let punct_symbol = if supports_unicode { "!" } else { "P" };
    let num_symbol = if supports_unicode { "#" } else { "N" };
    let symbol_symbol = if supports_unicode { "@" } else { "S" };
    let divider = if supports_unicode { "│" } else { "|" };
    let custom_symbol = if supports_unicode { "⚙" } else { "<c>" };

    let elements = vec![
        TermiElement::new(
            format!("{} punctuation ", punct_symbol),
            config.use_punctuation,
            Some(TermiClickAction::TogglePunctuation),
        ),
        TermiElement::new(
            format!("{} numbers ", num_symbol),
            config.use_numbers,
            Some(TermiClickAction::ToggleNumbers),
        ),
        TermiElement::new(
            format!("{} symbols ", symbol_symbol),
            config.use_symbols,
            Some(TermiClickAction::ToggleSymbols),
        ),
        TermiElement::new(format!("{} ", divider), false, None),
        TermiElement::new(
            "T time ",
            is_time_mode,
            Some(TermiClickAction::SwitchMode(ModeType::Time)),
        ),
        TermiElement::new(
            "A words ",
            !is_time_mode,
            Some(TermiClickAction::SwitchMode(ModeType::Words)),
        ),
        TermiElement::new(format!("{} ", divider), false, None),
        TermiElement::new(
            format!("{} ", presets[0]),
            current_value == presets[0],
            Some(TermiClickAction::SetModeValue(presets[0])),
        ),
        TermiElement::new(
            format!("{} ", presets[1]),
            current_value == presets[1],
            Some(TermiClickAction::SetModeValue(presets[1])),
        ),
        TermiElement::new(
            format!("{} ", presets[2]),
            current_value == presets[2],
            Some(TermiClickAction::SetModeValue(presets[2])),
        ),
        TermiElement::new(
            format!("{} ", presets[3]),
            current_value == presets[3],
            Some(TermiClickAction::SetModeValue(presets[3])),
        ),
        TermiElement::new(
            format!("{} ", custom_symbol),
            is_custom_active,
            Some(TermiClickAction::ToggleModal(custom_ctx)),
        ),
    ];

    elements
        .into_iter()
        .map(|element| element.to_styled(theme))
        .collect()
}

pub fn create_mode_bar(termi: &Termi) -> Vec<TermiElement> {
    let status = termi.tracker.status.clone();
    let theme = termi.current_theme();
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
            let mut spans = vec![Span::styled(info, Style::default().fg(theme.highlight()))];

            // the live wpm is an option toggleable by the user
            if !termi.config.hide_live_wpm {
                spans.push(Span::styled(
                    wpm,
                    Style::default()
                        .fg(theme.muted())
                        .add_modifier(Modifier::DIM),
                ));
            }
            let line = Line::from(spans);
            let text = Text::from(line);
            TermiElement::new(text, false, None)
        }
        _ => TermiElement::new(Text::raw(""), false, None),
    };
    vec![element]
}

pub fn create_typing_area<'a>(
    termi: &'a Termi,
    scroll_offset: usize,
    visible_line_count: usize,
    word_positions: &[WordPosition],
) -> Text<'a> {
    let theme = termi.current_theme();

    if word_positions.is_empty() {
        return Text::raw("");
    }

    let words: Vec<&str> = termi.words.split_whitespace().collect();
    let mut lines: Vec<Line> = Vec::with_capacity(visible_line_count);

    let first_line_to_render = scroll_offset;
    let last_line_to_render = scroll_offset + visible_line_count;

    let mut current_line_spans = Vec::with_capacity(200);
    let mut current_line_idx_in_full_text = 0;

    if let Some(first_pos) = word_positions.first() {
        current_line_idx_in_full_text = first_pos.line;
    }

    let cursor_pos = termi.tracker.cursor_position;
    let supports_themes = theme.color_support.supports_themes();

    let success_style = Style::default().fg(theme.success());
    let error_style = Style::default().fg(theme.error());
    let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let underline_error_style = if supports_themes {
        error_style
            .add_modifier(Modifier::UNDERLINED)
            .underline_color(theme.error())
    } else {
        error_style
    };
    let underline_success_style = if supports_themes {
        success_style
            .add_modifier(Modifier::UNDERLINED)
            .underline_color(theme.error())
    } else {
        success_style
    };

    for (i, pos) in word_positions.iter().enumerate() {
        if pos.line > current_line_idx_in_full_text {
            if current_line_idx_in_full_text >= first_line_to_render
                && current_line_idx_in_full_text < last_line_to_render
            {
                if !current_line_spans.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                    current_line_spans.reserve(200);
                }
            } else {
                current_line_spans.clear();
            }
            current_line_idx_in_full_text = pos.line;

            if lines.len() >= visible_line_count {
                break;
            }
        }

        if pos.line >= first_line_to_render && pos.line < last_line_to_render {
            let word = words.get(i).unwrap_or(&"");
            let word_start = pos.start_index;
            let word_len = word.chars().count();

            let is_word_wrong = termi.tracker.is_word_wrong(word_start);
            let is_past_word = cursor_pos > word_start + word_len;
            let should_underline_word = is_word_wrong && is_past_word && supports_themes;

            for (char_i, c) in word.chars().enumerate() {
                let char_idx = word_start + char_i;

                let is_correct =
                    termi.tracker.user_input.get(char_idx).copied().flatten() == Some(c);
                let has_input = termi.tracker.user_input.get(char_idx).is_some();

                let style = if !has_input {
                    dim_style
                } else if is_correct {
                    if should_underline_word {
                        underline_success_style
                    } else {
                        success_style
                    }
                } else if should_underline_word {
                    underline_error_style
                } else {
                    error_style
                };

                let mut char_string = get_pooled_string();
                char_string.push(c);
                current_line_spans.push(Span::styled(char_string, style));
            }

            if i < words.len() - 1
                && word_positions
                    .get(i + 1)
                    .is_some_and(|next_pos| next_pos.line == pos.line)
            {
                current_line_spans.push(Span::raw(" "));
            }
        }
    }

    if !current_line_spans.is_empty()
        && current_line_idx_in_full_text >= first_line_to_render
        && current_line_idx_in_full_text < last_line_to_render
        && lines.len() < visible_line_count
    {
        lines.push(Line::from(current_line_spans));
    }

    Text::from(lines)
}

pub fn create_command_bar(termi: &Termi) -> Vec<TermiElement> {
    let theme = termi.current_theme();

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
        vec![vec![
            ("tab", true),
            (" + ", false),
            ("enter", true),
            (" - restart", false),
        ]],
        vec![
            vec![("esc", true), (" - menu", false)],
            vec![
                ("ctrl", true),
                (" + ", false),
                ("c", true),
                (" - quit", false),
            ],
        ],
    ];

    let mut lines = Vec::new();
    for line_groups in command_groups {
        let mut spans = Vec::new();
        for (i, group) in line_groups.iter().enumerate() {
            let group_spans: Vec<Span<'static>> = group
                .iter()
                .map(|&(text, is_key)| styled_span(text.to_string(), is_key, theme))
                .collect();
            spans.extend(group_spans);

            if i < line_groups.len() - 1 {
                spans.push(styled_span("  ".to_string(), false, theme));
            }
        }
        lines.push(Line::from(spans).alignment(Alignment::Center));
    }

    let text = Text::from(lines).alignment(Alignment::Center);
    vec![TermiElement::new(text, false, None)]
}

pub fn create_footer<'a>(termi: &Termi) -> Vec<TermiElement<'a>> {
    let theme = termi.current_theme();

    // Check if terminal supports Unicode
    let supports_unicode = theme.supports_unicode();
    let info_symbol = if supports_unicode { "ⓘ" } else { "i" };
    let divider = if supports_unicode {
        DOUBLE_VERTICAL_LEFT
    } else {
        "|"
    };

    let theme_click_action = if termi.theme.color_support.supports_themes() {
        Some(TermiClickAction::ToggleThemePicker)
    } else {
        None
    };

    let elements = vec![
        TermiElement::new(
            format!("{} about", info_symbol),
            termi.menu.is_about_menu(),
            Some(TermiClickAction::ToggleAbout),
        ),
        TermiElement::new(" ", false, None),
        TermiElement::new(divider, false, None),
        TermiElement::new(" ", false, None),
        TermiElement::new(
            termi.theme.id.clone(),
            termi.preview_theme.is_some(),
            theme_click_action,
        ),
        TermiElement::new(" ", false, None),
        TermiElement::new(divider, false, None),
        TermiElement::new(" ", false, None),
        TermiElement::new(format!("v{VERSION}"), false, None),
    ];

    elements
        .into_iter()
        .map(|element| element.to_styled(theme))
        .collect()
}

pub fn create_minimal_size_warning(termi: &Termi, width: u16, height: u16) -> Vec<TermiElement> {
    let theme = termi.current_theme();
    let warning_lines = vec![
        Line::from(Span::styled(
            "! size too small",
            Style::default().fg(theme.error()),
        )),
        Line::from(""),
        Line::from("Current:"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Width = ",
                Style::default()
                    .fg(theme.muted())
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                format!("{}", width),
                Style::default().fg(if width < MIN_TERM_WIDTH {
                    theme.error()
                } else {
                    theme.success()
                }),
            ),
            Span::styled(" Height = ", Style::default().fg(theme.muted()))
                .add_modifier(Modifier::DIM),
            Span::styled(
                format!("{}", height),
                Style::default().fg(if height < MIN_TERM_HEIGHT {
                    theme.error()
                } else {
                    theme.success()
                }),
            ),
        ]),
        Line::from(""),
        Line::from("Needed:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Width = ", Style::default().fg(theme.muted()))
                .add_modifier(Modifier::DIM),
            Span::styled(
                format!("{}", MIN_TERM_WIDTH),
                Style::default()
                    .fg(theme.muted())
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(" Height = ", Style::default().fg(theme.muted()))
                .add_modifier(Modifier::DIM),
            Span::styled(
                format!("{}", MIN_TERM_HEIGHT),
                Style::default()
                    .fg(theme.muted())
                    .add_modifier(Modifier::DIM),
            ),
        ]),
    ];
    let text = Text::from(warning_lines).alignment(Alignment::Center);
    vec![TermiElement::new(text, false, None)]
}

pub fn create_show_menu_button(termi: &Termi) -> Vec<TermiElement> {
    let theme = termi.current_theme();
    let menu_text = "≡ Show Menu";
    // bound the text in non clickable padding to avoid having a wider click area
    let padding = " ".repeat((menu_text.len() / 2).max(1));

    vec![
        TermiElement::new(padding.clone(), false, None),
        TermiElement::new(
            menu_text,
            termi.menu.is_open(),
            Some(TermiClickAction::ToggleMenu),
        )
        .to_styled(theme),
        TermiElement::new(padding, false, None),
    ]
}

pub fn create_menu_footer_text(termi: &Termi) -> Line {
    let theme = termi.current_theme();
    let menu_state = &termi.menu;

    // Check if terminal supports Unicode
    let supports_unicode = theme.supports_unicode();
    let cursor_symbol = if supports_unicode { "█" } else { "_" };
    let up_arrow = if supports_unicode { "↑" } else { "^" };
    let down_arrow = if supports_unicode { "↓" } else { "v" };

    if menu_state.is_searching() {
        Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(theme.accent())),
            Span::styled(menu_state.search_query(), Style::default().fg(theme.fg())),
            Span::styled(
                cursor_symbol,
                Style::default()
                    .fg(theme.cursor())
                    .add_modifier(Modifier::RAPID_BLINK),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!("[{}/k]", up_arrow),
                Style::default().fg(theme.highlight()),
            ),
            Span::styled(" up ", Style::default().fg(theme.muted())).add_modifier(Modifier::DIM),
            Span::styled(
                format!("[{}/j]", down_arrow),
                Style::default().fg(theme.highlight()),
            ),
            Span::styled(" down ", Style::default().fg(theme.muted())).add_modifier(Modifier::DIM),
            Span::styled("[/]", Style::default().fg(theme.highlight())),
            Span::styled(" search ", Style::default().fg(theme.muted()))
                .add_modifier(Modifier::DIM),
            Span::styled("[ent]", Style::default().fg(theme.highlight())),
            Span::styled(" sel ", Style::default().fg(theme.muted())).add_modifier(Modifier::DIM),
            Span::styled("[space]", Style::default().fg(theme.highlight())),
            Span::styled(" toggle ", Style::default().fg(theme.muted()))
                .add_modifier(Modifier::DIM),
            Span::styled("[esc]", Style::default().fg(theme.highlight())),
            Span::styled(" close", Style::default().fg(theme.muted())).add_modifier(Modifier::DIM),
        ])
    }
}

pub fn create_results_footer_text(theme: &Theme) -> Line {
    Line::from(vec![
        Span::styled("[N]", Style::default().fg(theme.warning())),
        Span::styled(
            "ew",
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        ),
        Span::styled(" ", Style::default()),
        Span::styled("[R]", Style::default().fg(theme.warning())),
        Span::styled(
            "edo",
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        ),
        Span::styled(" ", Style::default()),
        Span::styled("[Q]", Style::default().fg(theme.warning())),
        Span::styled(
            "uit",
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        ),
        Span::styled(" ", Style::default()),
        Span::styled("[ESC]", Style::default().fg(theme.warning())),
        Span::styled(
            " menu",
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        ),
    ])
    .alignment(Alignment::Center)
}

fn get_content_bg(
    is_selected: bool,
    is_disabled: bool,
    hide_cursorline: bool,
    theme: &Theme,
) -> Color {
    if is_selected && !is_disabled && !hide_cursorline {
        theme.selection_bg()
    } else {
        theme.bg()
    }
}

fn get_label_style(
    item: &MenuItem,
    is_selected: bool,
    should_render_cursorline: bool,
    theme: &Theme,
) -> Style {
    if is_selected && !should_render_cursorline {
        Style::default()
            .fg(theme.success())
            .add_modifier(Modifier::BOLD)
    } else if is_selected && should_render_cursorline {
        Style::default()
            .fg(theme.selection_fg())
            .bg(theme.selection_bg())
    } else if item.is_disabled {
        Style::default()
            .fg(theme.muted())
            .add_modifier(Modifier::DIM)
    } else {
        match &item.result {
            MenuItemResult::OpenSubMenu(_) => Style::default().fg(theme.fg()),
            MenuItemResult::ToggleState => {
                if item.is_active == Some(true) {
                    Style::default().fg(theme.success())
                } else {
                    Style::default()
                        .fg(theme.muted())
                        .add_modifier(Modifier::DIM)
                }
            }
            _ => Style::default().fg(theme.fg()),
        }
    }
}

fn create_item_spans(
    item: &MenuItem,
    is_selected: bool,
    max_key_width: usize,
    hide_description: bool,
    hide_cursorline: bool,
    theme: &Theme,
) -> Vec<Span<'static>> {
    let supports_unicode = theme.supports_unicode();
    let arrow_symbol = if supports_unicode { "❯ " } else { "> " };
    let submenu_symbol = if supports_unicode { " →" } else { " >" };

    let should_render_cursorline = is_selected && !item.is_disabled && !hide_cursorline;
    let content_bg = get_content_bg(is_selected, item.is_disabled, hide_cursorline, theme);

    let mut spans = vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            if is_selected { arrow_symbol } else { "  " },
            Style::default()
                .fg(if is_selected {
                    theme.success()
                } else {
                    theme.fg()
                })
                .bg(content_bg),
        ),
    ];

    if let Some(key_text) = &item.key {
        // Info items (kv pairs)
        let formatted_key = if hide_description {
            key_text.to_string()
        } else {
            format!("{:<width$}", key_text, width = max_key_width + 2)
        };
        spans.push(Span::styled(
            formatted_key,
            Style::default()
                .fg(theme.accent())
                .bg(content_bg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            item.label.clone(),
            Style::default().fg(theme.fg()).bg(content_bg),
        ));
    } else {
        let label_style = get_label_style(item, is_selected, should_render_cursorline, theme);

        // toggleable checkbox
        if let Some(is_active) = item.is_active {
            let checkbox_text = if is_active { "[✓] " } else { "[ ] " };
            let checkbox_style = if is_active {
                Style::default().fg(theme.success()).bg(content_bg)
            } else {
                Style::default()
                    .fg(theme.border())
                    .bg(content_bg)
                    .add_modifier(Modifier::DIM)
            };
            spans.push(Span::styled(checkbox_text, checkbox_style));
        }

        spans.push(Span::styled(item.label.clone(), label_style.bg(content_bg)));
    }

    // sub-menu arrow
    if matches!(item.result, MenuItemResult::OpenSubMenu(_)) {
        spans.push(Span::styled(
            submenu_symbol,
            Style::default().fg(theme.primary()).bg(content_bg),
        ));
    }

    spans
}

pub fn build_menu_items<'a>(
    termi: &'a Termi,
    scroll_offset: usize,
    max_visible: usize,
    hide_description: bool,
) -> (Vec<ListItem<'a>>, usize) {
    let theme = termi.current_theme().clone();

    let Some(menu) = termi.menu.current_menu() else {
        return (vec![ListItem::new("  No menu content")], 0);
    };

    let items = menu.items();
    let total_items = items.len();

    if total_items == 0 {
        let no_matches = vec![
            ListItem::new(""),
            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    "grep: pattern not found",
                    Style::default()
                        .fg(theme.muted())
                        .add_modifier(Modifier::DIM),
                ),
            ])),
        ];
        return (no_matches, 0);
    }

    let current_item_id = menu
        .current_item()
        .map(|i| i.id.clone())
        .unwrap_or_default();

    let visible_items: Vec<_> = items
        .iter()
        .skip(scroll_offset)
        .take(max_visible)
        .cloned()
        .collect();

    let max_key_width = visible_items
        .iter()
        .filter_map(|item| item.key.as_ref())
        .map(|key_text| key_text.chars().count())
        .max()
        .unwrap_or(0);

    let list_items: Vec<ListItem<'a>> = std::iter::once(ListItem::new(""))
        .chain(visible_items.iter().map(|item| {
            let is_selected = item.id == current_item_id;
            let spans = create_item_spans(
                item,
                is_selected,
                max_key_width,
                hide_description,
                termi.config.hide_cursorline,
                &theme,
            );
            ListItem::new(Line::from(spans))
        }))
        .collect();

    (list_items, total_items)
}
