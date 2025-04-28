use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::Style,
    text::Line,
    widgets::{
        Block, Clear, List, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::{constants::MENU_HEIGHT, termi::Termi, tracker::Status};

use super::{
    actions::TermiClickAction,
    elements::{
        create_action_bar, create_command_bar, create_footer, create_header,
        create_menu_footer_text, create_minimal_size_warning, create_mode_bar, create_typing_area,
        prepare_menu_list_items, TermiElement,
    },
    layout::create_layout,
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
pub fn draw_ui(frame: &mut Frame, termi: &mut Termi) -> TermiClickableRegions {
    let mut regions = TermiClickableRegions::default();
    let theme = termi.get_current_theme();
    let area = frame.area();

    let potential_container = Block::new().padding(Padding::symmetric(8, 2));
    let potential_inner_area = potential_container.inner(area);
    let is_potentially_minimal = potential_inner_area.width < crate::constants::MIN_TERM_WIDTH
        || potential_inner_area.height < crate::constants::MIN_TERM_HEIGHT;

    let container =
        Block::new()
            .style(Style::default().bg(theme.bg()))
            .padding(if is_potentially_minimal {
                Padding::ZERO
            } else {
                Padding::symmetric(8, 2)
            });

    let inner_area = container.inner(area);
    frame.render_widget(container, area);

    if inner_area.width < crate::constants::MIN_TERM_WIDTH
        || inner_area.height < crate::constants::MIN_TERM_HEIGHT
    {
        let warning_elements =
            create_minimal_size_warning(termi, inner_area.width, inner_area.height);
        render(frame, &mut regions, warning_elements, inner_area);
        return regions;
    }

    let layout = create_layout(inner_area, termi);

    let header = create_header(termi);
    let typing_area = create_typing_area(termi);

    match termi.tracker.status {
        Status::Typing => {
            let mode_bar = create_mode_bar(termi);
            render(frame, &mut regions, header, layout.section.header);
            render(frame, &mut regions, mode_bar, layout.section.mode_bar);
            render(frame, &mut regions, typing_area, layout.section.typing_area);
        }
        Status::Idle | Status::Paused => {
            let mode_bar = create_mode_bar(termi);
            let action_bar = create_action_bar(termi);
            let command_bar = create_command_bar(termi);
            let footer = create_footer(termi);
            render(frame, &mut regions, header, layout.section.header);
            render(frame, &mut regions, typing_area, layout.section.typing_area);
            if !layout.is_small() {
                render(frame, &mut regions, mode_bar, layout.section.mode_bar);
                render(frame, &mut regions, action_bar, layout.section.action_bar);
                render(frame, &mut regions, command_bar, layout.section.command_bar);
                render(frame, &mut regions, footer, layout.section.footer);
            }
            if termi.menu.is_open() {
                render_menu(frame, termi, area);
            }
        }
        Status::Completed => {
            let command_bar = create_command_bar(termi);
            let footer = create_footer(termi);
            render(frame, &mut regions, header, layout.section.header);
            render(frame, &mut regions, typing_area, layout.section.typing_area);
            if !layout.is_small() {
                render(frame, &mut regions, command_bar, layout.section.command_bar);
                render(frame, &mut regions, footer, layout.section.footer);
            }
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
        let paragraph = Paragraph::new(element.content.clone()).alignment(alignment);
        f.render_widget(paragraph, area);

        if let Some(action) = element.action {
            cr.add(area, action);
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

        let (list_items, total_items) = prepare_menu_list_items(termi, scroll_offset, max_visible);

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
