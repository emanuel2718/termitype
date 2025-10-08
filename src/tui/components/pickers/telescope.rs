use crate::{
    actions::Action,
    app::App,
    menu::{Menu, MenuAction, MenuVisualizer},
    theme::Theme,
    tui::{
        components::pickers::search_bar,
        helpers::menu_items_padding,
        layout::{picker_overlay_area, picker_should_show_visualizer, picker_should_use_full_area},
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

fn calculate_total_menu_area(overlay_area: Rect, screen_area: Rect, has_visualizer: bool) -> Rect {
    if !has_visualizer {
        // single panel
        return overlay_area;
    }

    let bar_height = 3u16;
    let extended_height =
        (overlay_area.height + bar_height).min(screen_area.height.saturating_sub(overlay_area.y));

    Rect {
        x: overlay_area.x,
        y: overlay_area.y,
        width: overlay_area.width,
        height: extended_height,
    }
}

// TODO: if we want to do multiple pickers this needs to be refaactored, will be too messy
pub fn render_telescope_picker(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let menu = &app.menu;

    if let Some(current_menu) = menu.current_menu() {
        let items = menu.current_items();
        let has_no_items = items.is_empty();

        let title = current_menu.title.clone();
        let overlay_area = picker_overlay_area(area);
        let is_informational_menu = current_menu.is_informational;

        // don't show visualizer on small height. Doesnt look good
        let has_visualizer = current_menu.has_visualizer() && picker_should_show_visualizer(area);
        let total_menu_area = calculate_total_menu_area(overlay_area, area, has_visualizer);

        frame.render_widget(Clear, total_menu_area);

        // has visualizer: two panels
        // no visualizer: single panel
        let is_low_width = overlay_area.width <= 75;

        let (items_area, visualizer_area, bottom_bar_anchor): (Rect, Rect, Rect) = if has_visualizer
        {
            if is_low_width {
                let bar_height: u16 = 3;
                let half_top_len = overlay_area.height / 2;

                let rows = if menu.is_searching() {
                    let top_len = half_top_len.saturating_sub(bar_height);
                    Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(top_len),
                            Constraint::Length(bar_height),
                            Constraint::Min(0),
                        ])
                        .split(total_menu_area)
                } else {
                    Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(half_top_len), Constraint::Min(0)])
                        .split(total_menu_area)
                };

                let top_panel = rows[0];
                let bottom_panel_base = rows[rows.len() - 1];

                // items (top)
                let top_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .title(format!(" {} ", title))
                    .title_alignment(Alignment::Center)
                    .title_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .style(Style::default().bg(theme.bg()));
                let top_inner = top_block.inner(top_panel);
                frame.render_widget(top_block, top_panel);

                // visualizer (bot)
                let bottom_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .style(Style::default().bg(theme.bg()));
                let bottom_inner = bottom_block.inner(bottom_panel_base);
                frame.render_widget(bottom_block, bottom_panel_base);

                (top_inner, bottom_inner, top_panel)
            } else {
                let columns = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(45), // left
                        Constraint::Length(0),
                        Constraint::Percentage(55), // right
                    ])
                    .split(overlay_area);

                let left_panel = columns[0];
                let right_panel_base = columns[2];

                // items (left)
                let left_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .title(format!(" {} ", title))
                    .title_alignment(Alignment::Center)
                    .title_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .style(Style::default().bg(theme.bg()));
                let left_inner = left_block.inner(left_panel);
                frame.render_widget(left_block, left_panel);

                let bar_height: u16 = 3; // TODO: refactor magic number for menu_bar height
                let right_panel = Rect {
                    x: right_panel_base.x,
                    y: overlay_area.y,
                    width: right_panel_base.width,
                    height: (overlay_area.height + bar_height)
                        .min(area.height.saturating_sub(overlay_area.y)),
                };

                // right (visualizer)
                let right_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .style(Style::default().bg(theme.bg()));
                let right_inner = right_block.inner(right_panel);
                frame.render_widget(right_block, right_panel);

                (left_inner, right_inner, left_panel)
            }
        } else {
            // single panel (no visualizer)
            let menu_block = Block::default()
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(theme.border())
                        .bg(theme.bg())
                        .remove_modifier(Modifier::all()),
                )
                .title(format!(" {} ", title))
                .title_alignment(Alignment::Center)
                .title_style(
                    Style::default()
                        .fg(theme.border())
                        .bg(theme.bg())
                        .remove_modifier(Modifier::all()),
                )
                .style(Style::default().bg(theme.bg()));
            let inner_area = menu_block.inner(overlay_area);
            frame.render_widget(menu_block, overlay_area);
            (inner_area, Rect::default(), overlay_area)
        };

        let items_area_height = items_area.height as usize;
        let mut scroll_offset = current_menu.scroll_offset;

        let max_key_len = if is_informational_menu {
            items
                .iter()
                .filter_map(|item| {
                    if let MenuAction::Info(key, _) = &item.action {
                        Some(key.len())
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0)
        } else {
            0
        };

        let mut lines: Vec<Line> = Vec::new();

        if has_no_items {
            let no_items_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
            lines.push(Line::from(Span::styled("No items found", no_items_style)));
        } else {
            let current_index = if menu.has_search_query() {
                if let Some(curr) = current_menu.current_item() {
                    items.iter().position(|&item| item == curr).unwrap_or(0)
                } else {
                    0
                }
            } else {
                current_menu.current_index
            };

            // ensure the current index is visible
            if current_index < scroll_offset {
                scroll_offset = current_index;
            } else if current_index >= scroll_offset + items_area_height {
                scroll_offset = current_index.saturating_sub(items_area_height.saturating_sub(1));
            }

            let visible_items = items
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(items_area_height)
                .collect::<Vec<_>>();

            for (idx, item) in visible_items {
                let is_selected = idx == current_index;
                let is_toggle = matches!(&item.action, MenuAction::Action(Action::Toggle(_)));
                let is_info = matches!(item.action, MenuAction::Info(_, _));
                let mut style = Style::default().fg(theme.fg());

                if is_selected && !is_info {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                if item.is_disabled {
                    style = style.add_modifier(Modifier::DIM);
                }

                if is_toggle {
                    if let MenuAction::Action(Action::Toggle(setting)) = &item.action {
                        let enabled = app.config.is_enabled(setting.clone());
                        if !enabled {
                            style = style.add_modifier(Modifier::DIM);
                        } else {
                            style = style.fg(theme.success())
                        }
                    }
                }

                if item.is_disabled {
                    style = style.add_modifier(Modifier::DIM);
                }

                if is_toggle {
                    if let MenuAction::Action(Action::Toggle(setting)) = &item.action {
                        let enabled = app.config.is_enabled(setting.clone());
                        if !enabled {
                            style = style.add_modifier(Modifier::DIM);
                        } else {
                            style = style.fg(theme.success())
                        }
                    }
                }

                let indicator = if is_toggle {
                    if let MenuAction::Action(Action::Toggle(setting)) = &item.action {
                        let enabled = app.config.is_enabled(setting.clone());
                        if enabled {
                            "[x]"
                        } else {
                            "[ ]"
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    match is_selected {
                        true => ">",
                        false => " ",
                    }
                };

                let spans = if is_info {
                    if let MenuAction::Info(key, value) = &item.action {
                        let padded_key = format!(" {:<width$}", key, width = max_key_len + 1);
                        vec![
                            Span::styled(indicator, Style::default().fg(theme.info())),
                            Span::styled(padded_key, Style::default().fg(theme.fg())),
                            Span::styled(" ", Style::default()),
                            Span::styled(value, Style::default().fg(theme.info())),
                        ]
                    } else {
                        unreachable!()
                    }
                } else {
                    let label = format!("{} {}", indicator, item.label());
                    vec![Span::styled(label, style)]
                };

                lines.push(Line::from(spans));
            }
        }

        let items_paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .block(Block::default().padding(menu_items_padding()));

        // render the items of the menu
        frame.render_widget(items_paragraph, items_area);

        // render visualization
        if let Some(menu) = menu.current_menu() {
            if menu.has_visualizer() && menu.visualizer.is_some() {
                let visualizer = menu.visualizer.as_ref().unwrap();
                super::visualizer::render_menu_visualizer(
                    frame,
                    theme,
                    visualizer,
                    visualizer_area,
                    app,
                );
            }
        }

        // Bottom bar should span the width of the items area panel when a visualizer exists.
        // Otherwise, it spans the classic overlay width.
        // In low-width mode, hide the bar unless searching; when searching, place it between
        // the stacked panels (anchored to the top/items panel).
        // In full area mode, always show the bar since space is reserved for it, except for vertical splits.
        if !is_low_width
            || menu.is_searching()
            || picker_should_use_full_area(area) && !has_visualizer
        {
            search_bar::render_menu_bottom_bar(
                frame,
                bottom_bar_anchor,
                area,
                theme,
                menu,
                has_visualizer,
            );
        }
    }
}
