use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::rc::Rc;

use super::constants::*;

pub fn centered_rect(px: u16, py: u16, r: Rect) -> Rect {
    let width = r.width.saturating_mul(px) / 100;
    let height = r.height.saturating_mul(py) / 100;

    let min_width = 40;
    let min_height = 10;

    let width = width.max(min_width);
    let height = height.max(min_height);

    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;

    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

pub fn create_main_layout(area: Rect) -> Rc<[Rect]> {
    const MIN_TYPING_HEIGHT: u16 = 3;
    const MIN_HEADER_HEIGHT: u16 = 1;
    const MIN_FOOTER_HEIGHT: u16 = 1;

    const ABSOLUTE_MIN_HEIGHT: u16 = 5; // minimum height to show anything

    if area.height < ABSOLUTE_MIN_HEIGHT {
        return Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1)])
            .split(area);
    }

    let critical_height = MIN_TYPING_HEIGHT + MIN_HEADER_HEIGHT + MIN_FOOTER_HEIGHT;

    if area.height <= critical_height {
        return Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // header
                Constraint::Min(2),    // typing area
                Constraint::Length(1), // footer
            ])
            .split(area);
    }

    // Title > TypingArea > Footer > Mode Bar > Top Bar > CommandBar
    let constraints = [
        Constraint::Ratio(1, 3),                // [0] Top flexible space
        Constraint::Max(HEADER_HEIGHT),         // [1] Header (1st priority)
        Constraint::Length(MODE_BAR_HEIGHT),    // [2] Mode bar (4th priority)
        Constraint::Length(TOP_BAR_HEIGHT),     // [3] Top bar (5th priority)
        Constraint::Min(MIN_TYPING_HEIGHT),     // [4] Typing area (2nd priority)
        Constraint::Length(COMMAND_BAR_HEIGHT), // [5] Command bar (6th priority)
        Constraint::Max(FOOTER_HEIGHT),         // [6] Footer (3rd priority)
        Constraint::Ratio(1, 3),                // [7] Bottom flexible space
    ];

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
}
