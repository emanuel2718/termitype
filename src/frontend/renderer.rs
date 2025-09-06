use crate::{app::App, log_debug, theme};
use anyhow::Result;
use ratatui::{style::Style, widgets::Paragraph, Frame};

pub fn draw_ui(frame: &mut Frame, app: &App) -> Result<()> {
    let area = frame.area();
    let theme = theme::current_theme();
    frame.render_widget(
        Paragraph::new("ctrl-c to quit").style(Style::default().bg(theme.bg()).fg(theme.fg())),
        area,
    );
    Ok(())
}
