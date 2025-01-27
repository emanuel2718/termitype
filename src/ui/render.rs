use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Borders, Clear, List, ListItem};
use ratatui::{style::Style, widgets::Block, Frame};

use crate::menu::Menu;
use crate::termi::Termi;

use crate::tracker::Status;

use super::{components::*, layout::*};
use crate::constants::{WINDOW_HEIGHT_PERCENT, WINDOW_WIDTH_PERCENT};

/// Main workhorse. This basically draws the whole ui
pub fn draw_ui(f: &mut Frame, termi: &mut Termi) {
    let container = Block::default().style(Style::default().bg(termi.theme.background));
    f.render_widget(container, f.area());

    let window_area = centered_rect(WINDOW_WIDTH_PERCENT, WINDOW_HEIGHT_PERCENT, f.area());

    let layout = create_main_layout(window_area);

    match termi.tracker.status {
        Status::Typing => {
            let typing_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(layout[2]);
            progress_info(f, termi, typing_chunks[0]);
            typing_area(f, termi, typing_chunks[1]);
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
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(layout[2]);
            typing_area(f, termi, typing_chunks[1]);
            command_bar(f, termi, layout[3]);
            footer(f, termi, layout[4]);
        }
    }
    if termi.menu.is_visible() {
        draw_menu(f, termi, f.area());
    }
}

pub fn draw_menu(f: &mut Frame, termi: &Termi, area: Rect) {
    let menu = &termi.menu;
    if !menu.is_visible() {
        return;
    }

    let menu_area = {
        let width = 30u16;
        let height = (menu.items.len() + 2) as u16;

        Rect {
            x: area.x + (area.width.saturating_sub(width)) / 2,
            y: area.y + (area.height.saturating_sub(height)) / 2,
            width: width.min(area.width),
            height: height.min(area.height),
        }
    };

    let background = Block::default()
        .style(Style::default().bg(termi.theme.background))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(termi.theme.border));

    let items: Vec<ListItem> = menu
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let content = Menu::get_display_text(item);
            let is_selected = i == menu.selected;

            let base_style = if menu.is_toggleable(item) {
                if menu.is_toggle_active(item, termi) {
                    Style::default().fg(termi.theme.foreground)
                } else {
                    Style::default().fg(termi.theme.inactive)
                }
            } else {
                Style::default().fg(termi.theme.foreground)
            };

            let style = if is_selected {
                base_style
                    .bg(termi.theme.selection)
                    .add_modifier(Modifier::BOLD)
            } else {
                base_style.bg(termi.theme.background)
            };

            ListItem::new(Line::from(vec![Span::styled(
                format!(" {} ", content),
                style,
            )]))
        })
        .collect();

    let menu_widget = List::new(items)
        .block(background)
        .style(Style::default().bg(termi.theme.background));

    f.render_widget(Clear, menu_area);
    f.render_widget(menu_widget, menu_area);
}
