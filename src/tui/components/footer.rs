use crate::{theme::Theme, tui::helpers::footer_padding};
use ratatui::{
    layout::Alignment,
    prelude::Stylize,
    style::{Modifier, Style},
    widgets::{Block, Paragraph},
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn create_footer_element<'a>(theme: &Theme) -> Paragraph<'a> {
    let footer_text = format!("{} v{}", theme.id, APP_VERSION);
    Paragraph::new(footer_text)
        .style(Style::default().fg(theme.fg()))
        .add_modifier(Modifier::DIM)
        .alignment(Alignment::Right)
        .block(Block::default().padding(footer_padding()))
}
