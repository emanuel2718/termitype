use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{style::Style, widgets::Block, Frame};

use crate::termi::Termi;

use crate::tracker::Status;

use crate::constants::{WINDOW_HEIGHT_PERCENT, WINDOW_WIDTH_PERCENT};
use super::{components::*, layout::*};

/// Main workhorse. This basically draws the whole ui
pub fn draw_ui(f: &mut Frame, termi: &mut Termi) {
    let container = Block::default()
        .style(Style::default().bg(termi.theme.background));
    f.render_widget(container, f.area());

    let window_area = centered_rect(
        WINDOW_WIDTH_PERCENT,
        WINDOW_HEIGHT_PERCENT,
        f.area(),
    );

    let layout = create_main_layout(window_area);

    match termi.tracker.status {
        Status::Typing => {
            let typing_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
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
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
                .split(layout[2]);
            typing_area(f, termi, typing_chunks[1]);
            command_bar(f, termi, layout[3]);
            footer(f, termi, layout[4]);
        }
    }
}
