use crate::{
    constants::{
        MIN_TERM_HEIGHT, MIN_TERM_WIDTH, MIN_WIDTH_TO_SHOW_FOOTER, SMALL_SCREEN_HEIGHT,
        SMALL_SCREEN_WIDTH, TYPING_AREA_WIDTH_PERCENT,
    },
    termi::Termi,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

// TODO: move to constats
const HEADER_HEIGHT: u16 = 4;
const ACTION_BAR_HEIGHT: u16 = 1;
const TOP_AREA_HEIGHT: u16 = HEADER_HEIGHT + ACTION_BAR_HEIGHT;

const MODE_BAR_HEIGHT: u16 = 2;

const COMMAND_BAR_HEIGHT: u16 = 3;
const FOOTER_HEIGHT: u16 = 1;
const BOTTOM_PADDING: u16 = 1;
const BOTTOM_AREA_HEIGHT: u16 = COMMAND_BAR_HEIGHT + BOTTOM_PADDING + FOOTER_HEIGHT;

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

    // TODO: divide better h_small and w_small or something
    pub fn is_small(&self) -> bool {
        self.area.width < SMALL_SCREEN_WIDTH || self.area.height < SMALL_SCREEN_HEIGHT
    }

    pub fn w_small(&self) -> bool {
        self.area.width < SMALL_SCREEN_WIDTH
    }

    pub fn h_small(&self) -> bool {
        self.area.height < SMALL_SCREEN_HEIGHT
    }

    pub fn show_footer(&self) -> bool {
        self.area.width >= MIN_WIDTH_TO_SHOW_FOOTER
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
            Constraint::Length(BOTTOM_AREA_HEIGHT), // Bottom (height increased)
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
    let action_bar_section = apply_horizontal_centering(top_chunk[1], TYPING_AREA_WIDTH_PERCENT);

    // ==== MIDDLE ====
    let mid_outer_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0), // top padding
            Constraint::Length(MODE_BAR_HEIGHT + termi.config.visible_lines as u16), // Typing area
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

    let mode_bar_section = apply_horizontal_centering(mid_chunk[0], TYPING_AREA_WIDTH_PERCENT);
    let typing_area_section = apply_horizontal_centering(mid_chunk[1], TYPING_AREA_WIDTH_PERCENT);

    // ==== BOTTOM ====
    let bot_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(COMMAND_BAR_HEIGHT),
            Constraint::Length(BOTTOM_PADDING),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(bot_area);

    let command_bar_section = apply_horizontal_centering(bot_chunks[0], TYPING_AREA_WIDTH_PERCENT);
    let footer_section = apply_horizontal_centering(bot_chunks[2], TYPING_AREA_WIDTH_PERCENT);

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

fn apply_horizontal_centering(area: Rect, width_percent: u16) -> Rect {
    let padding = (100 - width_percent) / 2;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(padding),
            Constraint::Percentage(width_percent),
            Constraint::Percentage(padding),
        ])
        .split(area)[1] // centered chunk
}
