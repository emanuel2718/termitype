use std::rc::Rc;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use super::constants::*;

pub fn centered_rect(px: u16, py: u16, r: Rect) -> Rect {
    let horiz_margin = r.width.saturating_sub(r.width * px / 100) / 2;
    let vert_margin = r.height.saturating_sub(r.height * py / 100) / 2;

    Rect {
        x: r.x + horiz_margin,
        y: r.y + vert_margin,
        width: r.width * px / 100,
        height: r.height * py / 100,
    }
}

pub fn create_main_layout(area: Rect) -> Rc<[Rect]> {
    let constraints = [
        Constraint::Length(VERTICAL_MARGIN),
        Constraint::Length(HEADER_HEIGHT),
        Constraint::Length(TOP_BAR_HEIGHT),
        Constraint::Length(MODE_BAR_HEIGHT),
        Constraint::Length(TYPING_AREA_MARGIN),
        Constraint::Length(PROGRESS_HEIGHT),
        Constraint::Min(1),
        Constraint::Length(COMMAND_BAR_HEIGHT),
        Constraint::Length(FOOTER_HEIGHT),
        Constraint::Length(VERTICAL_MARGIN),
    ];

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
}
