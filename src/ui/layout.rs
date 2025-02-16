use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::rc::Rc;

use crate::constants::{
    COMMAND_BAR_HEIGHT, FOOTER_HEIGHT, MIN_HEIGHT, MIN_TYPING_HEIGHT, MIN_WIDTH, MODE_BAR_OFFSET,
    TOP_BAR_HEIGHT, TYPING_AREA_WIDTH_PERCENT,
};

pub fn centered_rect(px: u16, py: u16, r: Rect) -> Rect {
    let width = r.width.saturating_mul(px) / 100;
    let height = r.height.saturating_mul(py) / 100;

    let width = width.max(MIN_WIDTH);
    let height = height.max(MIN_HEIGHT);

    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;

    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

/// Main layout
pub fn create_main_layout(area: Rect) -> Rc<[Rect]> {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TOP_BAR_HEIGHT), // [0] Top section (title + top bar + offset)
            Constraint::Min(0),                 // [1] space
            Constraint::Min(MIN_TYPING_HEIGHT), // [2] Typing area
            Constraint::Min(6),                 // [3] space
            Constraint::Length(COMMAND_BAR_HEIGHT), // [4] Command bar
            Constraint::Length(2),              // [5] space
            Constraint::Length(FOOTER_HEIGHT),  // [6] Footer
        ])
        .split(area);

    // top section
    let top_section = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TOP_BAR_HEIGHT),  // title
            Constraint::Length(MODE_BAR_OFFSET), // space
            Constraint::Length(MODE_BAR_OFFSET), // top bar
        ])
        .split(vertical_layout[0]);

    // title
    let title_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Termitype
            Constraint::Min(1),
        ])
        .split(top_section[0])[0];

    // TODO: left align the progress info
    Rc::from([
        title_area,         // [0] Title area (left-aligned)
        top_section[1],     // [1] Top bar area (centered)
        vertical_layout[2], // [2] Typing area (centered)
        vertical_layout[4], // [3] Command bar
        vertical_layout[6], // [4] Footer
    ])
}

/// Typing area layout
pub fn create_typing_area_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - TYPING_AREA_WIDTH_PERCENT) / 2),
            Constraint::Percentage(TYPING_AREA_WIDTH_PERCENT),
            Constraint::Percentage((100 - TYPING_AREA_WIDTH_PERCENT) / 2),
        ])
        .split(area)
}
