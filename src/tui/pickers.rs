use crate::{app::App, theme::Theme, variants::PickerVariant};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_menu_picker(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let variant = app.config.current_picker_variant();
    match variant {
        PickerVariant::Telescope => render_telescope_picker(frame, app, theme, area),
        _ => render_telescope_picker(frame, app, theme, area),
    }
}

fn render_telescope_picker(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let menu = &app.menu;

    if let Some(current_menu) = menu.current_menu() {
        let items = menu.current_items();
        if items.is_empty() {
            return;
        }

        let max_width = 60.min(area.width.saturating_sub(4));
        let max_height = 20.min(area.height.saturating_sub(4));
        let item_count = items.len() as u16;
        let content_height = item_count.min(max_height.saturating_sub(2)); // borders

        let overlay_width = max_width;
        let overlay_height = content_height + 2; // +2 for borders

        let overlay_area = Rect {
            x: (area.width - overlay_width) / 2,
            y: (area.height - overlay_height) / 2,
            width: overlay_width,
            height: overlay_height,
        };

        let title = if menu.is_searching() {
            format!("{} > {}", current_menu.title, menu.search_query())
        } else {
            current_menu.title.clone()
        };

        let bg = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD))
            .title(format!(" {} ", title))
            .title_style(Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(theme.bg()));

        let inner_area = bg.inner(overlay_area);

        frame.render_widget(Clear, inner_area);
        frame.render_widget(bg, overlay_area);

        let mut lines: Vec<Line> = Vec::new();
        let current_index = current_menu.current_index;

        for (idx, item) in items.iter().enumerate() {
            let is_selected = idx == current_index;
            let mut style = Style::default().fg(theme.fg());

            if is_selected {
                style = style.add_modifier(Modifier::REVERSED);
            }

            if item.is_disabled {
                style = style.add_modifier(Modifier::DIM);
            }

            let prefix = if is_selected { "> " } else { "  " };
            let label = format!("{}{}", prefix, item.label());

            lines.push(Line::from(vec![Span::styled(label, style)]));
        }

        let para = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(para, inner_area);
    }
}
