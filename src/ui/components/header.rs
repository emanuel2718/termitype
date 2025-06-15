use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::Text,
};

use crate::{constants::APPNAME, termi::Termi, tracker::Status, ui::helpers::TermiStyle};

use super::elements::TermiElement;

pub struct HeaderComponent;

impl HeaderComponent {
    pub fn create(termi: &Termi) -> Vec<TermiElement> {
        let theme = termi.current_theme();

        let text = Text::from(APPNAME)
            .alignment(Alignment::Left)
            .style(TermiStyle::highlight(theme))
            .patch_style(if termi.tracker.status == Status::Typing {
                Style::default().add_modifier(Modifier::DIM)
            } else {
                Style::default()
            });

        vec![TermiElement::text(text)]
    }
}
