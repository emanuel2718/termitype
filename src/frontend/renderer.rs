use crate::{app::App, theme};
use anyhow::Result;
use ratatui::{style::Style, widgets::Paragraph, Frame};

pub fn draw_ui(frame: &mut Frame, app: &App) -> Result<()> {
    let area = frame.area();
    let theme = theme::current_theme();
    let words_text = app.lexicon.words.join(" ");
    let text = format!("{}\nctrl-c to quit", words_text);
    frame.render_widget(
        Paragraph::new(text).style(Style::default().bg(theme.bg()).fg(theme.fg())),
        area,
    );
    Ok(())
}
