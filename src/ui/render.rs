use ratatui::{
    style::Style,
    widgets::{Block, Padding},
    Frame,
};

use crate::{termi::Termi, tracker::Status};

use super::{
    elements::{
        create_action_bar, create_command_bar, create_footer, create_header,
        create_minimal_size_warning, create_mode_bar, create_show_menu_button, create_typing_area,
    },
    layout::create_layout,
};

/// Main entry point for the rendering
pub fn draw_ui(frame: &mut Frame, termi: &mut Termi) {
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
        return frame.render_widget(warning, area);
    }

    let header_widget = create_header(termi);
    let action_bar_widget = create_action_bar(termi);
    let mode_bar_widget = create_mode_bar(termi);
    let typing_area_widget = create_typing_area(termi);
    let menu_action_widget = create_show_menu_button(termi);
    let command_bar_widget = create_command_bar(termi);
    let footer_widget = create_footer(termi);

    match termi.tracker.status {
        Status::Typing => {
            frame.render_widget(header_widget, layout.section.header);
            frame.render_widget(mode_bar_widget, layout.section.mode_bar);
            frame.render_widget(typing_area_widget, layout.section.typing_area);
        }
        Status::Idle | Status::Paused => {
            frame.render_widget(header_widget, layout.section.header);
            frame.render_widget(typing_area_widget, layout.section.typing_area);
            if !layout.is_small() {
                frame.render_widget(menu_action_widget, layout.section.action_bar);
                frame.render_widget(mode_bar_widget, layout.section.mode_bar);
                frame.render_widget(action_bar_widget, layout.section.action_bar);
                frame.render_widget(command_bar_widget, layout.section.command_bar);
                frame.render_widget(footer_widget, layout.section.footer);
            }
        }
        Status::Completed => {}
    }
}
