use crate::{
    app::App,
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

        let bar_height = 3u16;
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(bar_height), Constraint::Min(0)])
            .split(overlay_area);

        let search_bar_area = rows[0];
        let menu_area = rows[1];

        frame.render_widget(Clear, overlay_area);

        render_search_bar(frame, search_bar_area, area, theme, menu);

        let menu_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
            .title(format!(" {} ", title))
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
            .style(Style::default().bg(theme.bg()));
        let items_area = menu_block.inner(menu_area);
        frame.render_widget(menu_block, menu_area);

        let items_area_height = items_area.height as usize;
        let mut scroll_offset = current_menu.scroll_offset;

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
                current_menu.current_index()
            };

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
                let mut style = Style::default().fg(theme.fg());

                if is_selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                if item.is_disabled {
                    style = style.add_modifier(Modifier::DIM);
                }

                let label = item.get_description();
                let spans = if let Some(tag) = &item.tag {
                    vec![
                        Span::styled(
                            tag,
                            Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
                        ),
                        Span::styled(" ", Style::default()),
                        Span::styled(label, style),
                    ]
                } else {
                    vec![Span::styled(label, style)]
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

fn render_search_bar(
    frame: &mut Frame,
    search_bar_area: Rect,
    area: Rect,
    theme: &Theme,
    menu: &crate::menu::Menu,
) {
    let bar_height = 3u16;
    let bar_area = Rect {
        x: search_bar_area.x,
        y: search_bar_area.y,
        width: search_bar_area.width,
        height: bar_height,
    };

    if bar_area.y + bar_area.height <= area.y + area.height {
        frame.render_widget(Clear, bar_area);
        let border_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
            .style(Style::default().bg(theme.bg()))
            .padding(Padding {
                left: 1,
                right: 1,
                top: 0,
                bottom: 0,
            });
        let inner = border_block.inner(bar_area);
        frame.render_widget(border_block, bar_area);

        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(inner);

        let left_area = sections[0];
        let right_area = sections[1];

        let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);

        if menu.is_searching() {
            let highlight_style = Style::default().fg(theme.fg());
            let left = Paragraph::new(vec![Line::from(vec![
                Span::styled(">", highlight_style),
                Span::styled(format!(" {}", menu.search_query()), dim_style),
            ])]);
            frame.render_widget(left, left_area);

            let base_offset: u16 = 2; // "> "
            let qlen = menu.search_query().chars().count() as u16;
            let mut x = left_area.x + base_offset + qlen;
            if x >= left_area.x + left_area.width.saturating_sub(1) {
                x = left_area.x + left_area.width.saturating_sub(2);
            }
            let y = left_area.y;
            frame.set_cursor_position(Position { x, y });
        } else {
            let left = Paragraph::new("> ")
                .style(dim_style)
                .alignment(Alignment::Left);
            frame.render_widget(left, left_area);
        }

        let (m, n) = if let Some(current_menu) = menu.current_menu() {
            let filtered_count = menu.current_items().len();
            let total_count = current_menu.len();
            (filtered_count, total_count)
        } else {
            (0, 0)
        };
        let right = Paragraph::new(format!("{}/{}", m, n))
            .style(dim_style)
            .alignment(Alignment::Right);
        frame.render_widget(right, right_area);
    }
}
