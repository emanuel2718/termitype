use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug)]
pub struct AppLayout {
    pub top_area: Rect,
    pub center_area: Rect,
    pub command_area: Rect,
    pub footer_area: Rect,
}

#[derive(Debug)]
pub struct ResultsLayout {
    pub results_area: Rect,
    pub footer_area: Rect,
}

const MAX_WIDTH: u16 = 80;

pub fn create_main_layout(area: Rect) -> AppLayout {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15), // top area
            Constraint::Percentage(75), // typing area
            Constraint::Percentage(7),  // command area
            Constraint::Percentage(2),  // footer
        ])
        .split(area);

    let top_area = vertical_layout[0];
    let typing_area = vertical_layout[1];
    let command_area = vertical_layout[2];
    let footer_area = vertical_layout[3];

    let center_width = MAX_WIDTH
        .min((typing_area.width as f32 * 0.8) as u16)
        .max(1);
    let left_width = (typing_area.width - center_width) / 2;
    let right_width = typing_area.width - center_width - left_width;

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left_width),
            Constraint::Length(center_width),
            Constraint::Length(right_width),
        ])
        .split(typing_area);

    let center_area = horizontal_layout[1];
    AppLayout {
        top_area,
        center_area,
        command_area,
        footer_area,
    }
}

pub fn create_results_layout(area: Rect) -> ResultsLayout {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10), // top margin
            Constraint::Percentage(83), // results
            Constraint::Percentage(7),  // footer
        ])
        .split(area);

    let results_area = vertical_layout[1];
    let footer_area = vertical_layout[2];

    ResultsLayout {
        results_area,
        footer_area,
    }
}
