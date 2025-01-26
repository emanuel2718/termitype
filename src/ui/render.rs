use ratatui::{style::Style, widgets::Block, Frame};

use crate::termi::Termi;

use crate::constants::{WINDOW_HEIGHT_PERCENT, WINDOW_WIDTH_PERCENT};
use crate::tracker::Status;

use super::{components::*, layout::*};

pub fn draw_ui(f: &mut Frame, termi: &mut Termi) {
    f.render_widget(
        Block::default().style(Style::default().bg(termi.theme.background)),
        f.area(),
    );

    let size = f.area();
    let area = centered_rect(
        WINDOW_WIDTH_PERCENT as u16,
        WINDOW_HEIGHT_PERCENT as u16,
        size,
    );

    let container = Block::default();
    let inner_area = container.inner(area);
    f.render_widget(&container, area);

    let layout = create_main_layout(inner_area);

    match termi.tracker.status {
        Status::Typing => {
            progress_info(f, termi, layout[4]);
            typing_area(f, termi, layout[4]);
        }
        Status::Completed => results_screen(f, termi, inner_area),
        _ => {
            title(f, termi, layout[1]);
            top_bar(f, termi, layout[3]);
            typing_area(f, termi, layout[4]);
            command_bar(f, termi, layout[5]);
            footer(f, termi, layout[6]);
        }
    }
}
