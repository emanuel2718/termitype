use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Padding},
    Frame,
};

use crate::{termi::Termi, tracker::Status};

use super::{
    actions::TermiClickAction,
    elements::{
        create_action_bar, create_command_bar, create_footer, create_header,
        create_minimal_size_warning, create_mode_bar, create_show_menu_button, create_typing_area,
        TermiElement,
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

    // Calculate layout first to check is_minimal
    let dummy_layout = create_layout(Block::new().inner(area), termi);
    let container =
        Block::new()
            .style(Style::default().bg(theme.bg()))
            .padding(if dummy_layout.is_minimal() {
                Padding::ZERO
            } else {
                Padding::symmetric(8, 2)
            });

    let inner_area = container.inner(area);
    let layout = if dummy_layout.is_minimal() {
        dummy_layout
    } else {
        create_layout(inner_area, termi)
    };

    // conatiner that renders the background and outer padding
    frame.render_widget(container, area);

    if layout.is_minimal() {
        let warning = create_minimal_size_warning(termi, area.width, area.height);
        frame.render_widget(warning.widget, area);
        return regions;
    }

    let header = create_header(termi);
    let action_bar = create_action_bar(termi);
    let mode_bar = create_mode_bar(termi);
    let typing_area = create_typing_area(termi);
    let menu_action = create_show_menu_button(termi);
    let command_bar = create_command_bar(termi);
    let footer = create_footer(termi);

    match termi.tracker.status {
        Status::Typing => {
            render(frame, &mut regions, header, layout.section.header);
            render(frame, &mut regions, mode_bar, layout.section.mode_bar);
            render(frame, &mut regions, typing_area, layout.section.typing_area);
        }
        Status::Idle | Status::Paused => {
            render(frame, &mut regions, header, layout.section.header);
            render(frame, &mut regions, typing_area, layout.section.typing_area);

            if !layout.is_small() {
                render(frame, &mut regions, menu_action, layout.section.action_bar);
                render(frame, &mut regions, mode_bar, layout.section.mode_bar);
                render(frame, &mut regions, action_bar, layout.section.action_bar);
                render(frame, &mut regions, command_bar, layout.section.command_bar);
                render(frame, &mut regions, footer, layout.section.footer);
            }
        }
        Status::Completed => {}
    }

    regions
}

fn render(f: &mut Frame, cr: &mut TermiClickableRegions, element: TermiElement, rect: Rect) {
    f.render_widget(element.widget, rect);
    if let Some(action) = element.action {
        cr.add(rect, action);
    }
}
