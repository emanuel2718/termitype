use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::{style::Style, widgets::Block, Frame};

use crate::termi::Termi;

use crate::tracker::Status;

use super::{components::*, layout::*};
use crate::constants::{WINDOW_HEIGHT_PERCENT, WINDOW_WIDTH_PERCENT};

/// Main workhorse. This basically draws the whole ui
pub fn draw_ui(f: &mut Frame, termi: &mut Termi) {
    let theme = termi.get_current_theme();

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
            results_screen(f, termi, layout[2]);
        }
        _ => {
            title(f, termi, layout[0]);
            top_bar(f, termi, layout[1]);
            // NOTE: hack to keep the typing area from shifting when we enter `Typing` mode
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
    if termi.menu.is_open() {
        draw_menu(f, termi, f.area());
    }
}

pub fn draw_menu(f: &mut Frame, termi: &Termi, area: Rect) {
    let menu = &termi.menu;
    let theme = termi.get_current_theme();
    if !menu.is_open() {
        return;
    }

    let menu_area = {
        let width = 30u16;
        let max_visible_items = 10u16;
        let height = if let Some(current_menu) = menu.current_menu() {
            // 2 for the border/title and 2 for the footer
            // Add 1 for search bar when searching
            (current_menu.items().len().min(max_visible_items as usize)
                + 4
                + if menu.is_searching() { 1 } else { 0 }) as u16
        } else {
            4
        };
        Rect {
            x: area.x + (area.width.saturating_sub(width)) / 2,
            y: area.y + (area.height.saturating_sub(height)) / 2,
            width: width.min(area.width),
            height: height.min(area.height),
        }
    };

    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if menu.is_searching() {
            vec![
                Constraint::Length(1), // Title
                Constraint::Length(1), // Search bar
                Constraint::Min(1),    // Menu items
                Constraint::Length(2), // Footer
            ]
        } else {
            vec![
                Constraint::Length(1), // Title
                Constraint::Min(1),    // Menu items
                Constraint::Length(2), // Footer
            ]
        })
        .split(menu_area);

    let title = if menu.menu_depth() > 1 {
        "<SubMenu>"
    } else {
        "Menu"
    };

    let background = Block::default()
        .style(Style::default().bg(theme.background()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border()))
        .title(title)
        .title_alignment(Alignment::Center);

    let mut scrollbar_state = ScrollbarState::default();

    let items: Vec<ListItem> = if let Some(current_menu) = menu.current_menu() {
        let menu_items = current_menu.items();

        let filtered_items: Vec<_> = if menu.is_searching() {
            current_menu.filtered_items(menu.search_query())
        } else {
            menu_items.iter().enumerate().collect()
        };

        let total_items = filtered_items.len();
        let max_visible =
            (menu_area.height as usize).saturating_sub(4 + if menu.is_searching() { 1 } else { 0 }); // minus border + footer + search bar if searching

        // just to make sure we keep the selected item visible
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

        // update scrolbar position
        scrollbar_state = scrollbar_state
            .content_length(total_items)
            .position(scroll_offset);

        filtered_items
            .iter()
            .skip(scroll_offset)
            .take(max_visible)
            .map(|&(i, item)| {
                let is_selected = i == current_menu.selected_index();
                let style = Style::default()
                    .fg(if item.is_toggleable {
                        if item.is_active {
                            theme.success()
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
                    Span::styled(
                        if is_selected { ">" } else { " " },
                        Style::default().fg(theme.highlight()),
                    ),
                    Span::raw(" "),
                    Span::styled(&item.label, style),
                ]))
            })
            .collect()
    } else {
        vec![]
    };

    let footer_text = vec![
        Line::from(vec![
            Span::styled("↑/k", Style::default().fg(theme.highlight())),
            Span::styled(" up   ", Style::default().fg(theme.muted())),
            Span::styled("↓/j", Style::default().fg(theme.highlight())),
            Span::styled(" down", Style::default().fg(theme.muted())),
        ]),
        Line::from(vec![
            Span::styled("enter", Style::default().fg(theme.highlight())),
            Span::styled(" select   ", Style::default().fg(theme.muted())),
            Span::styled("esc", Style::default().fg(theme.highlight())),
            Span::styled(" close", Style::default().fg(theme.muted())),
        ]),
    ];

    let footer = Paragraph::new(footer_text).alignment(Alignment::Center);
    let menu_widget = List::new(items).style(Style::default().bg(theme.background()));

    f.render_widget(Clear, menu_area);
    f.render_widget(background, menu_area);

    // search bar
    if menu.is_searching() {
        let search_text = Line::from(vec![
            Span::styled("/", Style::default().fg(theme.highlight())),
            Span::styled(menu.search_query(), Style::default().fg(theme.foreground())),
        ]);
        let search_bar = Paragraph::new(search_text).style(Style::default().bg(theme.background()));
        f.render_widget(search_bar, menu_layout[1]);
        f.render_widget(menu_widget, menu_layout[2]);
        f.render_widget(footer, menu_layout[3]);
    } else {
        f.render_widget(menu_widget, menu_layout[1]);
        f.render_widget(footer, menu_layout[2]);
    }

    // Only show scrollbar if we have more items than can fit in the visible area
    if let Some(current_menu) = menu.current_menu() {
        let total_items = current_menu.items().len();
        let max_visible =
            (menu_area.height as usize).saturating_sub(4 + if menu.is_searching() { 1 } else { 0 });

        if total_items > max_visible {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .style(Style::default().fg(theme.border()));

            let scrollbar_area = if menu.is_searching() {
                menu_layout[2]
            } else {
                menu_layout[1]
            };

            f.render_stateful_widget(
                scrollbar,
                scrollbar_area.inner(Margin {
                    vertical: 0,
                    horizontal: 1,
                }),
                &mut scrollbar_state,
            );
        }
    }
}
