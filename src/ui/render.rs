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
use crate::constants::{MENU_HEIGHT, WINDOW_HEIGHT_PERCENT, WINDOW_WIDTH_PERCENT};

/// Main workhorse. This basically draws the whole ui
pub fn draw_ui(f: &mut Frame, termi: &mut Termi) {
    termi.clickable_regions.clear(); // NOTE: we must always clear clickable regions before rendering

    if !termi.menu.is_open() && termi.preview_theme.is_some() {
        termi.preview_theme = None;
    }

    let theme = termi.get_current_theme().clone();

    let container = Block::default().style(Style::default().bg(theme.background()));
    f.render_widget(container, f.area());

    let window_area = centered_rect(WINDOW_WIDTH_PERCENT, WINDOW_HEIGHT_PERCENT, f.area());
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
                .split(layout[2]);

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
                .split(f.area());

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

            results_screen(f, termi, centered_rect);
        }
        _ => {
            title(f, termi, layout[0]);
            top_bar(f, termi, layout[1]);

            let typing_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
                .split(layout[2]);

            progress_info(f, termi, typing_chunks[0]);
            typing_area(f, termi, typing_chunks[2]);
            command_bar(f, termi, layout[3]);
            footer(f, termi, layout[4]);
        }
    }

    draw_menu(f, termi, f.area());

    #[cfg(debug_assertions)]
    if let Some(debug) = &termi.debug {
        if debug.visible {
            let debug_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(f.area())[1];

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
                .style(Style::default().bg(theme.background()))
                .block(content_block);
            f.render_widget(menu_widget, menu_layout[0]);
        } else {
            let scroll_offset = if total_items <= max_visible {
                0
            } else {
                let halfway = max_visible / 2;
                if current_menu.selected_index() < halfway {
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
                                    theme.foreground()
                                })
                                .bg(if is_selected {
                                    theme.selection_bg()
                                } else {
                                    theme.background()
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
                .style(Style::default().bg(theme.background()))
                .block(content_block);

            f.render_widget(menu_widget, menu_layout[0]);

            // scrollbar
            if total_items > max_visible {
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("│"))
                    .thumb_symbol("┃")
                    .style(Style::default().fg(theme.accent()));

                f.render_stateful_widget(
                    scrollbar,
                    menu_layout[0].inner(Margin {
                        vertical: 1,
                        horizontal: 1,
                    }),
                    &mut ScrollbarState::default()
                        .content_length(total_items)
                        .position(scroll_offset),
                );
            }
        }
    }

    let footer_text = if menu.is_searching() {
        Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(theme.accent())),
            Span::styled(menu.search_query(), Style::default().fg(theme.foreground())),
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
        .style(Style::default().bg(theme.background()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border())),
        )
        .alignment(Alignment::Left);
    f.render_widget(footer, menu_layout[1]);
}
