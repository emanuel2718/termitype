use crate::{
    app::App,
    menu::Menu,
    theme::Theme,
    tui::{helpers::menu_items_padding, layout::picker_overlay_area},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
};

pub fn render_command_palette(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let menu = &app.menu;

    if let Some(current_menu) = menu.current_menu() {
        let items = menu.current_items();
        let has_no_items = items.is_empty();
        let title = current_menu.title.clone();

        let overlay_area = picker_overlay_area(area);

        frame.render_widget(Clear, overlay_area);

        let menu_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
            .title(format!(" {} ", title))
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
            .style(Style::default().bg(theme.bg()))
            .padding(Padding {
                left: 1,
                right: 1,
                top: 1,
                bottom: 0,
            });
        let inner_area = menu_block.inner(overlay_area);
        frame.render_widget(menu_block, overlay_area);

        let search_bar_height = 1u16;
        let spacer_height = 1u16;
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(search_bar_height),
                Constraint::Length(spacer_height),
                Constraint::Min(0),
            ])
            .split(inner_area);

        let search_bar_area = rows[0];
        let items_area = rows[2];

        render_search_bar(frame, search_bar_area, theme, menu);

        let items_area_height = items_area.height as usize;
        let mut scroll_offset = current_menu.scroll_offset;

        let mut lines: Vec<Line> = Vec::new();

        if has_no_items {
            let no_items_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
            lines.push(Line::from(Span::styled("No items found", no_items_style)));
        } else {
            let current_index = menu.current_index().unwrap_or(0);

            if current_index < scroll_offset {
                scroll_offset = current_index;
            } else if current_index >= scroll_offset + items_area_height {
                scroll_offset = current_index.saturating_sub(items_area_height.saturating_sub(1));
            }

            for (idx, item) in items
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(items_area_height)
            {
                let is_selected = idx == current_index;
                let base_style = Style::default().fg(theme.fg());

                let label = item.get_description();
                let spans = if let Some(tag) = &item.tag {
                    let tag_style = if is_selected {
                        Style::default()
                            .fg(theme.bg())
                            .bg(theme.fg())
                            .add_modifier(Modifier::DIM)
                    } else {
                        Style::default().fg(theme.fg()).add_modifier(Modifier::DIM)
                    };

                    let label_style = if is_selected {
                        if item.is_disabled {
                            base_style
                                .fg(theme.bg())
                                .bg(theme.fg())
                                .add_modifier(Modifier::DIM)
                        } else {
                            base_style.fg(theme.bg()).bg(theme.fg())
                        }
                    } else if item.is_disabled {
                        base_style.add_modifier(Modifier::DIM)
                    } else {
                        base_style
                    };

                    vec![
                        Span::styled(format!("{tag}: "), tag_style),
                        Span::styled(label, label_style),
                    ]
                } else {
                    let label_style = if is_selected {
                        if item.is_disabled {
                            base_style
                                .fg(theme.bg())
                                .bg(theme.fg())
                                .add_modifier(Modifier::DIM)
                        } else {
                            base_style.fg(theme.bg()).bg(theme.fg())
                        }
                    } else if item.is_disabled {
                        base_style.add_modifier(Modifier::DIM)
                    } else {
                        base_style
                    };
                    vec![Span::styled(label, label_style)]
                };

                lines.push(Line::from(spans));
            }
        }

        let items_paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .block(Block::default().padding(menu_items_padding()));

        frame.render_widget(items_paragraph, items_area);
    }
}

fn render_search_bar(frame: &mut Frame, area: Rect, theme: &Theme, menu: &Menu) {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);

    let left_area = sections[0];
    let right_area = sections[1];

    let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);

    let highlight_style = Style::default().fg(theme.fg());
    let mut spans = vec![Span::styled(">", highlight_style)];
    if !menu.has_search_query() {
        spans.push(Span::styled(" Search...", dim_style));
    }
    if !menu.search_query().is_empty() {
        spans.push(Span::styled(
            format!(" {}", menu.search_query()),
            highlight_style,
        ));
    }

    let left = Paragraph::new(vec![Line::from(spans)]);
    frame.render_widget(left, left_area);

    let base_offset: u16 = 2; // "> "
    let qlen = menu.search_query().chars().count() as u16;
    let mut x = left_area.x + base_offset + qlen;
    if x >= left_area.x + left_area.width.saturating_sub(1) {
        x = left_area.x + left_area.width.saturating_sub(2);
    }
    let y = left_area.y;
    frame.set_cursor_position(Position { x, y });
    let (m, n) = if menu.current_menu().is_some() {
        let items = menu.current_items();
        let n = items.len();
        let current_index = menu.current_index().unwrap_or(0);
        (current_index.saturating_add(1), n)
    } else {
        (0, 0)
    };
    let right = Paragraph::new(format!("{}/{}", if n == 0 { 0 } else { m }, n))
        .style(dim_style)
        .alignment(Alignment::Right);
    frame.render_widget(right, right_area);
}
