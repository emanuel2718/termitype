use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, List, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::{
    config::Mode,
    constants::{ASCII_ART, MENU_HEIGHT, MODAL_HEIGHT, MODAL_WIDTH},
    modal::InputModal,
    termi::Termi,
    tracker::Status,
    version::VERSION,
};

use super::{
    actions::TermiClickAction,
    elements::{
        build_menu_items, create_action_bar, create_command_bar, create_footer, create_header,
        create_menu_footer_text, create_minimal_size_warning, create_mode_bar,
        create_show_menu_button, create_styled_typing_text, TermiElement,
    },
    layout::create_layout,
    utils::{calculate_word_positions, center_div, WordPosition},
};

#[derive(Debug, Default)]
pub struct TermiClickableRegions {
    pub regions: Vec<(Rect, TermiClickAction)>,
}

impl TermiClickableRegions {
    pub fn add(&mut self, area: Rect, action: TermiClickAction) {
        if area.width > 0 && area.height > 0 {
            self.regions.push((area, action));
        }
    }
}

/// Main entry point for the rendering
pub fn draw_ui(frame: &mut Frame, termi: &mut Termi, fps: Option<f64>) -> TermiClickableRegions {
    let mut regions = TermiClickableRegions::default();
    let theme = termi.get_current_theme();
    let area = frame.area();

    let dummy_layout = create_layout(Block::new().inner(area), termi);
    let container =
        Block::new()
            .style(Style::default().bg(theme.bg()))
            .padding(if dummy_layout.is_minimal() {
                Padding::ZERO
            } else if dummy_layout.w_small() {
                Padding::uniform(2)
            } else {
                Padding::symmetric(8, 2)
            });

    let inner_area = container.inner(area);
    let layout = if dummy_layout.is_minimal() {
        dummy_layout
    } else {
        create_layout(inner_area, termi)
    };

    frame.render_widget(container, area);

    if layout.is_minimal() {
        let warning = create_minimal_size_warning(termi, inner_area.width, inner_area.height);
        render(frame, &mut regions, warning, area);
        return regions;
    }

    let layout = create_layout(inner_area, termi);

    let mut fps_widget: Option<(Paragraph, Rect)> = None;
    if let Some(fps_value) = fps {
        let fps_text = format!("FPS: {:.0}", fps_value);
        let widget = Paragraph::new(fps_text)
            .style(Style::default().fg(theme.muted()))
            .add_modifier(Modifier::DIM)
            .alignment(Alignment::Right);

        let widget_width = 10;
        let fps_area = Rect::new(
            area.right().saturating_sub(widget_width + 1),
            area.top() + 1,
            widget_width,
            1,
        );
        fps_widget = Some((widget, fps_area));
    }

    let header = create_header(termi);

    match termi.tracker.status {
        Status::Typing => {
            let mode_bar = create_mode_bar(termi);
            render(frame, &mut regions, header, layout.section.header);
            render(frame, &mut regions, mode_bar, layout.section.mode_bar);
            render_typing_area(frame, termi, layout.section.typing_area);
        }
        Status::Completed => {
            render_results_screen(frame, termi, area, layout.is_small());
        }
        _ => {
            let mode_bar = create_mode_bar(termi);
            let command_bar = create_command_bar(termi);
            let footer = create_footer(termi);
            render(frame, &mut regions, mode_bar, layout.section.mode_bar);
            render(frame, &mut regions, header, layout.section.header);
            render_typing_area(frame, termi, layout.section.typing_area);
            // small width big enough height
            if layout.w_small() && !layout.h_small() {
                let menu_button = create_show_menu_button(termi);
                render(frame, &mut regions, menu_button, layout.section.action_bar);
                // TODO: there has to be a better way
                if layout.show_footer() {
                    render(frame, &mut regions, command_bar, layout.section.command_bar);
                    render(frame, &mut regions, footer, layout.section.footer);
                }
            } else if !layout.is_small() {
                let action_bar = create_action_bar(termi);
                render(frame, &mut regions, action_bar, layout.section.action_bar);
                // TODO: there has to be a better way
                if layout.show_footer() {
                    render(frame, &mut regions, command_bar, layout.section.command_bar);
                    render(frame, &mut regions, footer, layout.section.footer);
                }
            }
        }
    }

    if termi.menu.is_open() {
        render_menu(frame, termi, area);
    }

    if let Some(modal) = &termi.modal {
        if let Some(region) = render_modal(frame, termi, area, modal.clone()) {
            regions.add(region.0, region.1);
        }
    }

    if let Some((widget, fps_area)) = fps_widget {
        if fps_area.right() <= frame.area().right() && fps_area.bottom() <= frame.area().bottom() {
            frame.render_widget(widget, fps_area);
        }
    }

    regions
}

fn render(f: &mut Frame, cr: &mut TermiClickableRegions, elements: Vec<TermiElement>, area: Rect) {
    if elements.is_empty() {
        return;
    }

    if elements.len() == 1 {
        let element = &elements[0];
        let alignment = element.content.alignment.unwrap_or(Alignment::Left);
        let text_height = element.content.height() as u16;

        let centered_area = Rect {
            x: area.x,
            y: area.y + (area.height.saturating_sub(text_height)) / 2,
            width: area.width,
            height: area.height,
        };

        let paragraph = Paragraph::new(element.content.clone()).alignment(alignment);
        f.render_widget(paragraph, centered_area);

        if let Some(action) = element.action {
            cr.add(centered_area, action);
        }
    } else {
        let mut spans = Vec::new();
        let mut clickable_regions_to_add = Vec::new();
        let mut total_width: u16 = 0;

        let mut element_data = Vec::new();
        for element in &elements {
            let line_width = element.content.lines.first().map_or(0, |line| line.width()) as u16;
            total_width += line_width;
            element_data.push((line_width, element.action));
        }

        let start_x = area.x + (area.width.saturating_sub(total_width)) / 2;

        let mut current_x_offset: u16 = 0;
        for (i, element) in elements.iter().enumerate() {
            let (element_width, action) = element_data[i];

            if let Some(line) = element.content.lines.first() {
                spans.extend(line.spans.clone());
            }

            if let Some(action) = action {
                let region_rect = Rect {
                    x: start_x + current_x_offset,
                    y: area.y,
                    width: element_width,
                    height: area.height.min(1),
                };
                if element_width > 0 {
                    clickable_regions_to_add.push((region_rect, action));
                }
            }
            current_x_offset += element_width;
        }

        f.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            area,
        );

        for (rect, action) in clickable_regions_to_add {
            cr.add(rect, action);
        }
    }
}

fn render_typing_area(frame: &mut Frame, termi: &Termi, area: Rect) {
    let available_width = area.width as usize;
    let theme = termi.get_current_theme();

    let styled_text = create_styled_typing_text(termi, theme, Some(available_width));
    let word_positions = calculate_word_positions(&termi.words, available_width);

    let cursor_idx = termi.tracker.cursor_position;

    let current_word_pos = word_positions
        .iter()
        .filter(|pos| cursor_idx >= pos.start_index)
        .next_back()
        .unwrap_or_else(|| {
            word_positions.first().unwrap_or(&WordPosition {
                start_index: 0,
                line: 0,
                col: 0,
            })
        });

    let current_line = current_word_pos.line;
    let line_count = termi.config.visible_lines as usize;

    let scroll_offset = if line_count <= 1 {
        current_line
    } else {
        let half_visible = line_count / 2;
        if current_line < half_visible {
            0
        } else if current_line >= half_visible {
            current_line.saturating_sub(half_visible)
        } else {
            current_line
        }
    };

    let paragraph = Paragraph::new(styled_text)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);

    let menu_height = MENU_HEIGHT.min(frame.area().height);
    let estimated_menu_area = Rect {
        x: frame.area().x,
        y: frame.area().y,
        width: frame.area().width,
        height: menu_height,
    };

    // show cursor if:
    //     - typing/idle
    //     - menu is closed OR menu do not overlap typing area
    //          * this is to be able to preview the cursor if we can see the typing area
    let show_cursor = (termi.tracker.status == Status::Idle
        || termi.tracker.status == Status::Typing)
        && (!termi.menu.is_open() || !estimated_menu_area.intersects(area));

    if show_cursor {
        let offset = cursor_idx.saturating_sub(current_word_pos.start_index);
        let cursor_x = area.x + (current_word_pos.col + offset) as u16;
        let cursor_y = area.y + (current_line.saturating_sub(scroll_offset)) as u16;

        if cursor_x >= area.left()
            && cursor_x < area.right()
            && cursor_y >= area.top()
            && cursor_y < area.bottom()
        {
            frame.set_cursor_position(Position {
                x: cursor_x,
                y: cursor_y,
            });
        }
    }
}

fn render_modal(
    frame: &mut Frame,
    termi: &Termi,
    area: Rect,
    modal: InputModal,
) -> Option<(Rect, TermiClickAction)> {
    let theme = termi.theme.clone();
    let modal_area = center_div(MODAL_WIDTH, MODAL_HEIGHT, area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.fg()))
        .style(Style::default().bg(theme.bg()));

    let inner_area = block.inner(modal_area);

    frame.render_widget(block, modal_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // desc
            Constraint::Length(1), // space
            Constraint::Length(1), // input
            Constraint::Length(1), // error/space
            Constraint::Length(1), // space
            Constraint::Length(1), // [ OK ]
        ])
        .split(inner_area);

    // ------ TITLE ------
    let title_area = layout[0];
    let title = Paragraph::new(modal.title.clone())
        .style(
            Style::default()
                .fg(theme.highlight())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    frame.render_widget(title, title_area);

    // ------ DESCRIPTION ------
    let desc_area = layout[1];
    let desc = Paragraph::new(modal.description.clone())
        .style(Style::default().fg(theme.muted()))
        .alignment(Alignment::Center);
    frame.render_widget(desc, desc_area);

    // ------ INPUT FIELD ------
    let input_area = layout[3];
    let input_style = Style::default().fg(theme.fg());
    let cursor_style = Style::default().fg(theme.cursor_text()).bg(theme.cursor());
    let input_spans = vec![
        Span::styled(&modal.buffer.input[..modal.buffer.cursor_pos], input_style),
        Span::styled(" ", cursor_style),
        Span::styled(&modal.buffer.input[modal.buffer.cursor_pos..], input_style),
    ];

    let input_field = Paragraph::new(Line::from(input_spans));
    frame.render_widget(input_field, input_area);

    // ------ ERROR ------
    let error_area = layout[4];
    if let Some(error) = &modal.buffer.error_msg {
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(theme.error()))
            .alignment(Alignment::Center);
        frame.render_widget(error_text, error_area);
    }

    // ------ [ OK ] ------
    let ok_area = layout[6];
    let ok_button = Paragraph::new("[ OK ]")
        .style(Style::default().fg(theme.highlight()))
        .alignment(Alignment::Center);
    frame.render_widget(ok_button, ok_area);

    frame.set_cursor_position(Position {
        x: input_area.x + modal.buffer.cursor_pos as u16,
        y: input_area.y,
    });
    Some((ok_area, TermiClickAction::ModalConfirm))
}

fn render_menu(frame: &mut Frame, termi: &Termi, area: Rect) {
    let theme = termi.get_current_theme();
    let menu_state = &termi.menu;

    let menu_height = MENU_HEIGHT.min(area.height);

    let menu_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: menu_height,
    };
    let menu_area = menu_area.intersection(area);

    frame.render_widget(Clear, menu_area);

    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(menu_area);
    let content_area = menu_layout[0];
    let footer_area = menu_layout[1];

    if let Some(current_menu) = menu_state.current_menu() {
        let max_visible = content_area.height.saturating_sub(2) as usize;

        let filtered_items_for_scroll_calc: Vec<_> = if menu_state.is_searching() {
            current_menu.filtered_items(menu_state.search_query())
        } else {
            current_menu.items().iter().enumerate().collect()
        };
        let total_items_for_scroll_calc = filtered_items_for_scroll_calc.len();

        let scroll_offset = if total_items_for_scroll_calc <= max_visible || max_visible == 0 {
            0
        } else {
            let halfway = max_visible / 2;
            let selected_index = current_menu.selected_index();

            if menu_state.is_searching() {
                let filtered_position = filtered_items_for_scroll_calc
                    .iter()
                    .position(|(idx, _)| *idx == selected_index)
                    .unwrap_or(0);

                if filtered_position < halfway {
                    0
                } else if filtered_position >= total_items_for_scroll_calc.saturating_sub(halfway) {
                    total_items_for_scroll_calc.saturating_sub(max_visible)
                } else {
                    filtered_position.saturating_sub(halfway)
                }
            } else if selected_index < halfway {
                0
            } else if selected_index >= total_items_for_scroll_calc.saturating_sub(halfway) {
                total_items_for_scroll_calc.saturating_sub(max_visible)
            } else {
                selected_index.saturating_sub(halfway)
            }
        };

        let (list_items, total_items) = build_menu_items(termi, scroll_offset, max_visible);

        let content_block = Block::default()
            .title(" Menu ")
            .title_alignment(Alignment::Left)
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.bg()));

        let menu_widget = List::new(list_items)
            .block(content_block)
            .style(Style::default().bg(theme.bg()));

        frame.render_widget(menu_widget, content_area);

        if total_items > max_visible {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .style(Style::default().fg(theme.accent()));

            let scrollbar_area = content_area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            });

            let mut scrollbar_state = ScrollbarState::new(total_items).position(scroll_offset);

            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }

    let footer_text = create_menu_footer_text(termi);
    let footer_block = Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(Style::default().fg(theme.border()))
        .style(Style::default().bg(theme.bg()));

    let footer_widget = Paragraph::new(footer_text)
        .block(footer_block)
        .style(Style::default().bg(theme.bg()))
        .alignment(Alignment::Left);

    frame.render_widget(footer_widget, footer_area);
}

pub fn render_results_screen(frame: &mut Frame, termi: &mut Termi, area: Rect, is_small: bool) {
    let tracker = &termi.tracker;
    let theme = termi.get_current_theme();
    let config = &termi.config;
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let hostname = "termitype";

    let header = format!("{}@{}", username, hostname);
    let separator = "─".repeat(header.chars().count());

    // TODO: improve the coloring of this to match fastfetch. They have the @ in a different color.
    let mut stats_lines = vec![
        Line::from(vec![
            Span::styled(username, Style::default().fg(theme.highlight())),
            Span::styled("@", Style::default().fg(theme.highlight())),
            Span::styled(hostname, Style::default().fg(theme.highlight())),
        ]),
        Line::from(Span::styled(
            separator,
            Style::default().fg(theme.highlight()),
        )),
    ];

    let add_stat = |label: &str, value: String| -> Line {
        Line::from(vec![
            Span::styled(
                format!("{}: ", label),
                Style::default().fg(theme.highlight()),
            ),
            Span::styled(
                value,
                Style::default()
                    .fg(theme.muted())
                    .add_modifier(Modifier::DIM),
            ),
        ])
    };

    let errors = tracker
        .total_keystrokes
        .saturating_sub(tracker.correct_keystrokes);
    let (min_wpm, max_wpm) = tracker
        .wpm_samples
        .iter()
        .fold((u32::MAX, 0), |(min, max), &val| {
            (min.min(val), max.max(val))
        });
    let wpm_range_str = if min_wpm == u32::MAX {
        "N/A".to_string()
    } else {
        format!("{}-{}", min_wpm, max_wpm)
    };

    let mode_str = match config.current_mode() {
        Mode::Time { duration } => format!("Time ({}s)", duration),
        Mode::Words { count } => format!("Words ({})", count),
    };

    stats_lines.push(add_stat("OS", format!("termitype {}", VERSION)));
    stats_lines.push(add_stat("Mode", mode_str));
    stats_lines.push(add_stat(
        "Lang",
        config.language.clone().unwrap_or_default(),
    ));
    stats_lines.push(add_stat("WPM", format!("{:.0}", tracker.wpm)));
    if let Some(time) = tracker.completion_time {
        stats_lines.push(add_stat("Time", format!("{:.1}s", time)));
    } else if let Mode::Time { duration } = config.current_mode() {
        stats_lines.push(add_stat("Time", format!("{}s", duration)));
    }
    stats_lines.push(add_stat("Accuracy", format!("{}%", tracker.accuracy)));
    stats_lines.push(add_stat(
        "Consistency",
        format!("{:.0}%", tracker.calculate_consistency()),
    ));
    stats_lines.push(add_stat("Raw WPM", format!("{:.0}", tracker.raw_wpm)));
    stats_lines.push(add_stat(
        "Keystrokes",
        format!("{} ({})", tracker.total_keystrokes, tracker.accuracy),
    ));
    stats_lines.push(add_stat(
        "Correct",
        format!("{}", tracker.correct_keystrokes),
    ));
    stats_lines.push(add_stat("Errors", format!("{}", errors)));
    stats_lines.push(add_stat(
        "Backspaces",
        format!("{}", tracker.backspace_count),
    ));
    stats_lines.push(add_stat("WPM Range", wpm_range_str));

    stats_lines.push(Line::from(""));

    let mut color_blocks = vec![];
    for color in [
        theme.fg(),
        theme.highlight(),
        theme.accent(),
        theme.muted(),
        theme.success(),
        theme.warning(),
        theme.error(),
    ] {
        color_blocks.push(Span::styled("██", Style::default().fg(color)));
    }
    stats_lines.push(Line::from(color_blocks));

    let art_text = Text::from(ASCII_ART).style(Style::default().fg(theme.highlight()));
    let art_height = art_text.height() as u16;
    let art_width = art_text.width() as u16;

    let stats_text = Text::from(stats_lines);
    let stats_height = stats_text.height() as u16;
    let stats_width = stats_text
        .lines
        .iter()
        .map(|line| line.width())
        .max()
        .unwrap_or(0) as u16;

    if is_small {
        let centered_rect = Rect {
            x: area.x + area.width.saturating_sub(stats_width) / 2,
            y: area.y + area.height.saturating_sub(stats_height) / 2,
            width: stats_width.min(area.width),
            height: stats_height.min(area.height),
        };
        frame.render_widget(Paragraph::new(stats_text), centered_rect);
    } else {
        let total_needed_width = art_width + stats_width + 5;
        let horizontal_padding = area.width.saturating_sub(total_needed_width) / 2;
        let vertical_padding = area.height.saturating_sub(stats_height.max(art_height)) / 2;

        let layout_area = Rect {
            x: area.x + horizontal_padding,
            y: area.y + vertical_padding,
            width: total_needed_width.min(area.width),
            height: stats_height.max(art_height).min(area.height),
        };

        if layout_area.right() > area.right() || layout_area.width == 0 {
            let centered_rect = Rect {
                x: area.x + area.width.saturating_sub(stats_width) / 2,
                y: area.y + area.height.saturating_sub(stats_height) / 2,
                width: stats_width.min(area.width),
                height: stats_height.min(area.height),
            };
            frame.render_widget(Paragraph::new(stats_text), centered_rect);
        } else {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(art_width),
                        Constraint::Length(5), // padding
                        Constraint::Length(stats_width),
                    ]
                    .as_ref(),
                )
                .split(layout_area);

            let art_area = chunks[0];
            let stats_area = chunks[2];

            // if the art is shorter than stats then center it
            let art_y_padding = art_area.height.saturating_sub(art_height) / 2;
            let centered_art_area = Rect {
                y: art_area.y + art_y_padding,
                height: art_height.min(art_area.height),
                ..art_area
            };

            // do the same as above for the stats
            let stats_y_padding = stats_area.height.saturating_sub(stats_height) / 2;
            let centered_stats_area = Rect {
                y: stats_area.y + stats_y_padding,
                height: stats_height.min(stats_area.height),
                ..stats_area
            };

            frame.render_widget(Paragraph::new(art_text), centered_art_area);
            frame.render_widget(Paragraph::new(stats_text), centered_stats_area);
        }
    }

    let restart_line = Line::from(vec![
        Span::styled(
            "tab",
            Style::default()
                .fg(theme.highlight())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " + ",
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        ),
        Span::styled(
            "enter",
            Style::default()
                .fg(theme.highlight())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " - restart test",
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        ),
    ])
    .alignment(Alignment::Center);

    let restart_height: u16 = 4;
    if area.height > restart_height {
        let restart_area = Rect {
            x: area.x,
            y: area.bottom().saturating_sub(restart_height),
            width: area.width,
            height: restart_height,
        };
        frame.render_widget(Paragraph::new(restart_line), restart_area);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_word_position_basic() {
        let text = "hello world ";
        let available_width = 20;
        let positions = calculate_word_positions(text, available_width);

        assert_eq!(positions.len(), 2, "Should have positions for two words ");
        assert_eq!(positions[0].start_index, 0, "First word starts at 0");
        assert_eq!(positions[0].line, 0, "First word on line 0");
        assert_eq!(positions[0].col, 0, "First word at column 0");

        assert_eq!(
            positions[1].start_index, 6,
            "Second word starts after \"hello \""
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
