use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug)]
pub struct AppLayout {
    pub top_area: Rect,
    pub center_area: Rect,
    pub command_area: Rect,
    pub footer_area: Rect,
    pub title_area: Option<Rect>,
    pub mode_bar_area: Option<Rect>,
    pub show_title: bool,
    pub show_mode_bar: bool,
    pub show_footer: bool,
    pub show_command_bar: bool,
}

#[derive(Debug)]
pub struct ResultsLayout {
    pub results_area: Rect,
    pub footer_area: Rect,
}

#[derive(Default)]
pub struct LayoutBuilder {
    top_percent: u16,
    center_percent: u16,
    command_percent: u16,
    footer_percent: u16,
}

impl LayoutBuilder {
    pub fn new() -> Self {
        Self {
            top_percent: 15,
            center_percent: 75,
            command_percent: 7,
            footer_percent: 2,
        }
    }

    pub fn is_too_smol(area: Rect) -> bool {
        area.height < 8 || area.width < 35
    }

    pub fn top_percent(mut self, percent: u16) -> Self {
        self.top_percent = percent;
        self
    }

    pub fn center_percent(mut self, percent: u16) -> Self {
        self.center_percent = percent;
        self
    }

    pub fn command_percent(mut self, percent: u16) -> Self {
        self.command_percent = percent;
        self
    }

    pub fn footer_percent(mut self, percent: u16) -> Self {
        self.footer_percent = percent;
        self
    }

    pub fn build(self, area: Rect) -> AppLayout {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(self.top_percent),     // top area
                Constraint::Percentage(self.center_percent),  // typing area
                Constraint::Percentage(self.command_percent), // command area
                Constraint::Length(1),                        // gap
                Constraint::Percentage(self.footer_percent),  // footer
            ])
            .split(area);

        let top_area = vertical_layout[0];
        let typing_area = vertical_layout[1];
        let command_area = vertical_layout[2];
        let footer_area = vertical_layout[4];

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

        let show_title = area.height >= 14;
        let show_mode_bar = area.height >= 15;

        let padding_top = 1;
        let available_height = top_area.height.saturating_sub(padding_top);
        let title_height = if show_title { 1 } else { 0 };
        let mode_bar_height = if show_mode_bar {
            available_height.saturating_sub(title_height + 1)
        } else {
            0
        };

        // log_debug!("height: {}, width: {}", area.height, area.width);

        let title_area = if show_title && available_height >= title_height {
            Some(Rect {
                x: top_area.x,
                y: top_area.y + padding_top,
                width: top_area.width,
                height: title_height,
            })
        } else {
            None
        };

        let mode_bar_area = if show_mode_bar {
            Some(Rect {
                x: top_area.x,
                y: top_area.y + padding_top + title_height + 1,
                width: top_area.width,
                height: mode_bar_height,
            })
        } else {
            None
        };

        AppLayout {
            top_area,
            center_area,
            command_area,
            footer_area,
            title_area,
            mode_bar_area,
            show_title,
            show_mode_bar,
            show_footer: area.height >= 5,
            show_command_bar: area.height >= 28,
        }
    }
}

const MAX_WIDTH: u16 = 80;

pub fn create_main_layout(area: Rect) -> AppLayout {
    LayoutBuilder::new().build(area)
}

pub fn create_results_layout(area: Rect) -> ResultsLayout {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let results_area = vertical_layout[0];
    let footer_area = vertical_layout[1];

    ResultsLayout {
        results_area,
        footer_area,
    }
}

pub fn picker_should_use_full_area(area: Rect) -> bool {
    area.height < 25 || area.width < 60
}

pub fn picker_should_show_visualizer(area: Rect) -> bool {
    area.height >= 17
}

pub fn picker_overlay_area(area: Rect) -> Rect {
    if picker_should_use_full_area(area) {
        let bar_height = 3;
        let height = area.height.saturating_sub(bar_height);
        return Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height,
        };
    }

    let max_width = 90.min(area.width.saturating_sub(6));
    let max_height = 30
        .min(area.height.saturating_sub(6))
        .max(12)
        .min(area.height);

    Rect {
        x: (area.width - max_width) / 2,
        y: (area.height - max_height) / 2 - 2,
        width: max_width,
        height: max_height,
    }
}
