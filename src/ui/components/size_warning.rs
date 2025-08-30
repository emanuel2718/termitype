use ratatui::{
    layout::Alignment,
    text::{Line, Span, Text},
};

use crate::{
    constants::{MIN_TERM_HEIGHT, MIN_TERM_WIDTH},
    termi::Termi,
};

use super::elements::TermiElement;
use crate::ui::helpers::TermiStyle;

pub struct SizeWarningComponent;

impl SizeWarningComponent {
    pub fn create(termi: &Termi, width: u16, height: u16) -> Vec<TermiElement<'_>> {
        let theme = termi.current_theme();
        let warning_lines = vec![
            Line::from(Span::styled("! size too small", TermiStyle::error(theme))),
            Line::from(""),
            Line::from("Current:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Width = ", TermiStyle::muted(theme)),
                Span::styled(
                    format!("{width}"),
                    if width < MIN_TERM_WIDTH {
                        TermiStyle::error(theme)
                    } else {
                        TermiStyle::success(theme)
                    },
                ),
                Span::styled(" Height = ", TermiStyle::muted(theme)),
                Span::styled(
                    format!("{height}"),
                    if height < MIN_TERM_HEIGHT {
                        TermiStyle::error(theme)
                    } else {
                        TermiStyle::success(theme)
                    },
                ),
            ]),
            Line::from(""),
            Line::from("Needed:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Width = ", TermiStyle::muted(theme)),
                Span::styled(format!("{MIN_TERM_WIDTH}"), TermiStyle::muted(theme)),
                Span::styled(" Height = ", TermiStyle::muted(theme)),
                Span::styled(format!("{MIN_TERM_HEIGHT}"), TermiStyle::muted(theme)),
            ]),
        ];
        let text = Text::from(warning_lines).alignment(Alignment::Center);
        vec![TermiElement::new(text, false, None)]
    }
}
