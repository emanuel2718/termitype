use crate::{app::App, constants::APP_NAME, theme::Theme};
use ratatui::{
    layout::Alignment,
    prelude::Stylize,
    style::{Modifier, Style},
    widgets::{Block, Padding, Paragraph},
};

pub fn create_title<'a>(app: &App, theme: &Theme) -> Paragraph<'a> {
    let is_typing = app.tracker.is_typing();
    Paragraph::new(APP_NAME)
        .style(Style::default().fg(theme.highlight()))
        .add_modifier(if is_typing {
            Modifier::DIM
        } else {
            Modifier::empty()
        })
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding {
            left: 4,
            right: 0,
            top: 0,
            bottom: 0,
        }))
}
