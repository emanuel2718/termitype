use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    constants::{APPNAME, WINDOW_HEIGHT_PERCENT, WINDOW_WIDTH_PERCENT},
    termi::Termi,
};

pub fn draw_ui(f: &mut Frame, termi: &Termi) {
    let size = f.area();

    // 60% width and 50% height of the current trminal window size
    let area = centered_rect(
        WINDOW_WIDTH_PERCENT as u16,
        WINDOW_HEIGHT_PERCENT as u16,
        size,
    );

    let block = Block::bordered()
        .border_style(Style::new().fg(termi.theme.border))
        .borders(Borders::ALL)
        .title(Span::styled(
            APPNAME,
            Style::default()
                .fg(termi.theme.highlight)
                .add_modifier(Modifier::BOLD),
        ));

    f.render_widget(&block, area);
}

/// Creates a centered rectangle with the given width and height percentages.
///
/// # Arguments
///
/// * `px` - The width percentage (0-100) of the rectangle
/// * `py` - The height percentage (0-100) of the rectangle
/// * `r` - The outer rectangle to center within
///
/// # Returns
///
/// A `Rect` representing the centered area
fn centered_rect(px: u16, py: u16, r: Rect) -> Rect {
    let horizontal_margin = (r.width.saturating_sub(r.width * px / 100)) / 2;
    let vertical_margin = (r.height.saturating_sub(r.height * py / 100)) / 2;

    Rect {
        x: r.x + horizontal_margin,
        y: r.y + vertical_margin,
        width: r.width * px / 100,
        height: r.height * py / 100,
    }
}
