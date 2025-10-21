use crate::{menu::Menu, theme::Theme};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

pub fn render_menu_bottom_bar(
    frame: &mut Frame,
    overlay_area: Rect,
    area: Rect,
    theme: &Theme,
    menu: &Menu,
    has_visualizer: bool,
) {
    let bar_height = 3u16;
    let bar_area = Rect {
        x: overlay_area.x,
        y: overlay_area.y + overlay_area.height,
        width: overlay_area.width,
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

        // left 70%
        // right 30%
        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(inner);

        let left_area = sections[0];
        let right_area = sections[1];

        let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);

        // left
        if menu.is_searching() {
            let highlight_style = Style::default().fg(theme.highlight());
            let left = Paragraph::new(vec![Line::from(vec![
                Span::styled(">", highlight_style),
                Span::styled(format!(" {}", menu.search_query()), dim_style),
            ])]);
            frame.render_widget(left, left_area);

            // Hide cursor for cursor menu when visualizer is not shown
            let is_cursor_menu_without_visualizer = if let Some(current_menu) = menu.current_menu()
            {
                matches!(
                    current_menu.visualizer,
                    Some(crate::menu::MenuVisualizer::CursorVisualizer)
                ) && !has_visualizer
            } else {
                false
            };

            if !is_cursor_menu_without_visualizer {
                let base_offset: u16 = 2; // "> "
                let qlen = menu.search_query().chars().count() as u16;
                let mut x = left_area.x + base_offset + qlen;
                if x >= left_area.x + left_area.width.saturating_sub(1) {
                    x = left_area.x + left_area.width.saturating_sub(2);
                }
                let y = left_area.y;
                frame.set_cursor_position(Position { x, y });
            }
        } else {
            let left = Paragraph::new("> ")
                .style(dim_style)
                .alignment(Alignment::Left);
            frame.render_widget(left, left_area);
        }

        // right <m>/<N>
        let (m, n) = if let Some(current_menu) = menu.current_menu() {
            let items = menu.current_items();
            let n = items.len();
            let current_index = if menu.has_search_query() {
                if let Some(curr) = current_menu.current_item() {
                    items.iter().position(|&item| item == curr).unwrap_or(0)
                } else {
                    0
                }
            } else {
                current_menu.current_index()
            };
            (current_index.saturating_add(1), n)
        } else {
            (0, 0)
        };

        let right_text = Paragraph::new(format!("{}/{}", if n == 0 { 0 } else { m }, n))
            .style(dim_style)
            .alignment(Alignment::Right);
        frame.render_widget(right_text, right_area);
    }
}
