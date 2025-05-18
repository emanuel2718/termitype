use crate::{
    constants::{
        ACTION_BAR_HEIGHT, BOTTOM_AREA_HEIGHT, BOTTOM_PADDING, COMMAND_BAR_HEIGHT, FOOTER_HEIGHT,
        HEADER_HEIGHT, MIN_FOOTER_WIDTH, MIN_TERM_HEIGHT, MIN_TERM_WIDTH, MODE_BAR_HEIGHT,
        SMALL_RESULTS_HEIGHT, SMALL_RESULTS_WIDTH, SMALL_TERM_HEIGHT, SMALL_TERM_WIDTH,
        TOP_AREA_HEIGHT, TYPING_AREA_WIDTH,
    },
    termi::Termi,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone)]
pub struct TermiSection {
    pub header: Rect,
    pub action_bar: Rect,
    pub mode_bar: Rect,
    pub typing_area: Rect,
    pub command_bar: Rect,
    pub footer: Rect,
}

#[derive(Debug, Clone)]
pub struct TermiLayout {
    pub area: Rect,
    pub section: TermiSection,
}

impl TermiLayout {
    pub fn is_minimal(&self) -> bool {
        _is_minimal(self.area)
    }

    pub fn is_small(&self) -> bool {
        self.area.width < SMALL_TERM_WIDTH || self.area.height < SMALL_TERM_HEIGHT
    }

    pub fn w_small(&self) -> bool {
        self.area.width < SMALL_TERM_WIDTH
    }

    pub fn h_small(&self) -> bool {
        self.area.height < SMALL_TERM_HEIGHT
    }

    pub fn show_footer(&self) -> bool {
        self.area.width >= MIN_FOOTER_WIDTH
    }

    pub fn show_small_results(&self) -> bool {
        self.area.width < SMALL_RESULTS_WIDTH || self.area.height < SMALL_RESULTS_HEIGHT
    }
}

fn _is_minimal(area: Rect) -> bool {
    area.height < MIN_TERM_HEIGHT || area.width < MIN_TERM_WIDTH
}

fn _build_minimal_section(x: u16, y: u16) -> TermiSection {
    let rect = Rect::new(x, y, 0, 0);
    TermiSection {
        header: rect,
        action_bar: rect,
        mode_bar: rect,
        typing_area: rect,
        command_bar: rect,
        footer: rect,
    }
}

pub fn create_layout(area: Rect, termi: &Termi) -> TermiLayout {
    if _is_minimal(area) {
        return TermiLayout {
            area,
            section: _build_minimal_section(area.x, area.y),
        };
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TOP_AREA_HEIGHT),    // Top
            Constraint::Min(0),                     // Middle
            Constraint::Length(BOTTOM_AREA_HEIGHT), // Bottom
        ])
        .split(area);

    let top_area = chunks[0];
    let mid_area = chunks[1];
    let bot_area = chunks[2];

    // ==== TOP ====
    let top_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Length(ACTION_BAR_HEIGHT),
        ])
        .split(top_area);

    let header_section = top_chunk[0];
    let action_bar_section = create_centered_rect_with_max_width(top_chunk[1], TYPING_AREA_WIDTH);

    // ==== MIDDLE ====
    let mid_outer_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0), // top padding
            Constraint::Length(MODE_BAR_HEIGHT + termi.config.visible_lines as u16), // typing area
            Constraint::Min(0), // bottom padding
        ])
        .split(mid_area)[1];

    let mid_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(MODE_BAR_HEIGHT),
            Constraint::Length(termi.config.visible_lines as u16),
        ])
        .split(mid_outer_chunk);

    let mode_bar_section = create_centered_rect_with_max_width(mid_chunk[0], TYPING_AREA_WIDTH);
    let typing_area_section = create_centered_rect_with_max_width(mid_chunk[1], TYPING_AREA_WIDTH);

    // ==== BOTTOM ====
    let bot_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(COMMAND_BAR_HEIGHT),
            Constraint::Length(BOTTOM_PADDING),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(bot_area);

    let command_bar_section = create_centered_rect_with_max_width(bot_chunks[0], TYPING_AREA_WIDTH);
    let footer_section = create_centered_rect_with_max_width(bot_chunks[2], TYPING_AREA_WIDTH);

    let section = TermiSection {
        header: header_section,
        action_bar: action_bar_section,
        mode_bar: mode_bar_section,
        typing_area: typing_area_section,
        command_bar: command_bar_section,
        footer: footer_section,
    };

    TermiLayout { area, section }
}

fn create_centered_rect_with_max_width(area: Rect, max_content_width: u16) -> Rect {
    let padding = (100 - max_content_width) / 2;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(padding),
            Constraint::Percentage(max_content_width),
            Constraint::Percentage(padding),
        ])
        .split(area)[1] // centered chunk
}
