use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::{style::Style, widgets::Block, Frame};

use crate::termi::Termi;

use crate::tracker::Status;

use super::{components::*, layout::*};
use crate::constants::{
    MENU_HEIGHT, MIN_HEIGHT, MIN_WIDTH, WINDOW_HEIGHT_PERCENT, WINDOW_WIDTH_PERCENT,
};

/// Main workhorse. This basically draws the whole ui
pub fn draw_ui(f: &mut Frame, termi: &mut Termi) {
    termi.clickable_regions.clear(); // NOTE: we must always clear clickable regions before rendering

    let theme = termi.get_current_theme().clone();
    let buffer_area = f.area();

    let container = Block::default().style(Style::default().bg(theme.bg()));
    f.render_widget(container, buffer_area);

    // is our window too small question mark
    if buffer_area.width < MIN_WIDTH || buffer_area.height < MIN_HEIGHT {
        // must have at least 1x1 of space
        if buffer_area.width == 0 || buffer_area.height == 0 {
            return;
        }

        let mut warning_lines = Vec::new();

        if buffer_area.height >= 1 {
            warning_lines.push(Line::from(vec![Span::styled(
                if buffer_area.width < 14 {
                    "!"
                } else {
                    "Window too small!"
                },
                Style::default().fg(theme.error()),
            )]));
        }

        if buffer_area.height >= 2 {
            warning_lines.push(Line::from(vec![Span::styled(
                format!("{}x{}", MIN_WIDTH, MIN_HEIGHT),
                Style::default().fg(theme.muted()),
            )]));
        }

        if buffer_area.height >= 3 {
            warning_lines.push(Line::from(vec![Span::styled(
                format!("{}x{}", buffer_area.width, buffer_area.height),
                Style::default().fg(theme.muted()),
            )]));
        }

        let available_height = buffer_area.height;
        let content_height = warning_lines.len() as u16;
        let v_pad = (available_height.saturating_sub(content_height)) / 2;

        let warning_area = Rect {
            x: 0,
            y: v_pad,
            width: buffer_area.width,
            height: content_height.min(available_height),
        };

        let warning_block = Paragraph::new(warning_lines)
            .style(Style::default())
            .alignment(Alignment::Center);
        f.render_widget(warning_block, warning_area);
        return;
    }

    if !termi.menu.is_open() && termi.preview_theme.is_some() {
        termi.preview_theme = None;
    }

    let window_area = centered_rect(WINDOW_WIDTH_PERCENT, WINDOW_HEIGHT_PERCENT, buffer_area)
        .intersection(buffer_area); // window area must fit within the buffer
    let layout = create_main_layout(window_area);

    match termi.tracker.status {
        Status::Typing => {
            let typing_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
                .split(layout[2].intersection(buffer_area));

            progress_info(f, termi, typing_chunks[0]);
            typing_area(f, termi, typing_chunks[2]);
        }
        Status::Completed => {
            let vertical_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(10), // Top margin
                    Constraint::Percentage(80), // Results area
                    Constraint::Percentage(10), // Bottom margin
                ])
                .split(buffer_area);

            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(10), // Left margin
                    Constraint::Percentage(80), // Results area
                    Constraint::Percentage(10), // Right margin
                ])
                .split(vertical_layout[1]);

            let content_width = 80u16;
            let content_height = 20u16;

            let centered_rect = Rect {
                x: horizontal_layout[1].x
                    + (horizontal_layout[1].width.saturating_sub(content_width)) / 2,
                y: horizontal_layout[1].y
                    + (horizontal_layout[1].height.saturating_sub(content_height)) / 2,
                width: content_width.min(horizontal_layout[1].width),
                height: content_height.min(horizontal_layout[1].height),
            };

            let centered_rect = centered_rect.intersection(buffer_area);
            results_screen(f, termi, centered_rect);
        }
        _ => {
            title(f, termi, layout[0].intersection(buffer_area));
            top_bar(f, termi, layout[1].intersection(buffer_area));

            let typing_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
                .split(layout[2].intersection(buffer_area));

            progress_info(f, termi, typing_chunks[0]);
            typing_area(f, termi, typing_chunks[2]);
            command_bar(f, termi, layout[3].intersection(buffer_area));
            footer(f, termi, layout[4].intersection(buffer_area));
        }
    }

    draw_menu(f, termi, buffer_area);

    #[cfg(debug_assertions)]
    if let Some(debug) = &termi.debug {
        if debug.visible {
            let debug_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(buffer_area)[1];

            let debug_area = debug_area.intersection(buffer_area);
            f.render_widget(Clear, debug_area);
            debug.draw(f, termi, debug_area);
        }
    }
}

pub fn draw_menu(f: &mut Frame, termi: &mut Termi, area: Rect) {
    let theme = termi.get_current_theme().clone();

    let menu = &mut termi.menu;
    if !menu.is_open() {
        return;
    }

    let menu_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: MENU_HEIGHT.min(area.height.saturating_sub(2)),
    };
    let menu_area = menu_area.intersection(area);

    f.render_widget(Clear, menu_area);

    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Min(1),    // content area
            Constraint::Length(3), // footer (3 units: 1 for content + 2 for borders)
        ])
        .split(menu_area);

    if let Some(current_menu) = menu.current_menu() {
        let menu_items = current_menu.items();
        let filtered_items: Vec<_> = if menu.is_searching() {
            current_menu.filtered_items(menu.search_query())
        } else {
            menu_items.iter().enumerate().collect()
        };

        let total_items = filtered_items.len();
        let max_visible = menu_layout[0].height.saturating_sub(4) as usize;

        let content_block = Block::default()
            .title(" Menu ")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()));

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
            let menu_widget = List::new(no_matches)
                .style(Style::default().bg(theme.bg()))
                .block(content_block);
            f.render_widget(menu_widget, menu_layout[0].intersection(area));
        } else {
            let scroll_offset = if total_items <= max_visible {
                0
            } else {
                let halfway = max_visible / 2;

                if menu.is_searching() {
                    let selected_index = current_menu.selected_index();
                    let filtered_position = filtered_items
                        .iter()
                        .position(|(idx, _)| *idx == selected_index)
                        .unwrap_or(0);

                    if filtered_position < halfway {
                        0
                    } else if filtered_position >= total_items.saturating_sub(halfway) {
                        total_items.saturating_sub(max_visible)
                    } else {
                        filtered_position.saturating_sub(halfway)
                    }
                } else if current_menu.selected_index() < halfway {
                    0
                } else if current_menu.selected_index() >= total_items.saturating_sub(halfway) {
                    total_items.saturating_sub(max_visible)
                } else {
                    current_menu.selected_index().saturating_sub(halfway)
                }
            };

            let items: Vec<ListItem> = std::iter::once(ListItem::new(""))
                .chain(
                    filtered_items
                        .iter()
                        .skip(scroll_offset)
                        .take(max_visible)
                        .map(|&(i, item)| {
                            let is_selected = i == current_menu.selected_index();
                            let style = Style::default()
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
                                Span::styled(&item.label, style),
                                if item.has_submenu {
                                    Span::styled(" →", Style::default().fg(theme.accent()))
                                } else {
                                    Span::raw("")
                                },
                            ]))
                        }),
                )
                .collect();

            let menu_widget = List::new(items)
                .style(Style::default().bg(theme.bg()))
                .block(content_block);

            let content_area = menu_layout[0].intersection(area);
            f.render_widget(menu_widget, content_area);

            // scrollbar
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
                    horizontal: 1,
                });
                let scrollbar_area = scrollbar_area.intersection(area);

                let mut scrollbar_state = ScrollbarState::default()
                    .content_length(total_items)
                    .position(scroll_offset);

                f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
            }
        }
    }

    let footer_text = if menu.is_searching() {
        Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(theme.accent())),
            Span::styled(menu.search_query(), Style::default().fg(theme.fg())),
            Span::styled(
                "█",
                Style::default()
                    .fg(theme.cursor())
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("[↑/k]", Style::default().fg(theme.highlight())),
            Span::styled(" up", Style::default().fg(theme.muted())),
            Span::styled(" [↓/j]", Style::default().fg(theme.highlight())),
            Span::styled(" down", Style::default().fg(theme.muted())),
            Span::styled(" [/]", Style::default().fg(theme.highlight())),
            Span::styled(" search", Style::default().fg(theme.muted())),
            Span::styled(" [enter]", Style::default().fg(theme.highlight())),
            Span::styled(" select", Style::default().fg(theme.muted())),
            Span::styled(" [space]", Style::default().fg(theme.highlight())),
            Span::styled(" toggle", Style::default().fg(theme.muted())),
            Span::styled(" [esc]", Style::default().fg(theme.highlight())),
            Span::styled(" close", Style::default().fg(theme.muted())),
        ])
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().bg(theme.bg()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border())),
        )
        .alignment(Alignment::Left);
    f.render_widget(footer, menu_layout[1].intersection(area));
}
