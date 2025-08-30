use ratatui::{
    layout::Alignment,
    text::{Line, Span, Text},
};

use crate::{termi::Termi, theme::Theme, ui::helpers::TermiStyle};

use super::elements::TermiElement;

pub struct CommandBarComponent;

impl CommandBarComponent {
    pub fn create(termi: &Termi) -> Vec<TermiElement<'_>> {
        let theme = termi.current_theme();

        let command_groups = [
            vec![vec![
                ("tab", true),
                (" + ", false),
                ("enter", true),
                (" - restart", false),
            ]],
            vec![
                vec![("esc", true), (" - menu", false)],
                vec![
                    ("ctrl", true),
                    (" + ", false),
                    ("c", true),
                    (" - quit", false),
                ],
            ],
        ];

        let mut lines = Vec::new();
        for line_groups in command_groups {
            let mut spans = Vec::new();
            for (i, group) in line_groups.iter().enumerate() {
                let group_spans: Vec<Span<'static>> = group
                    .iter()
                    .map(|&(text, is_key)| Self::styled_span(text.to_string(), is_key, theme))
                    .collect();
                spans.extend(group_spans);

                if i < line_groups.len() - 1 {
                    spans.push(Self::styled_span("  ".to_string(), false, theme));
                }
            }
            lines.push(Line::from(spans).alignment(Alignment::Center));
        }

        let text = Text::from(lines).alignment(Alignment::Center);
        vec![TermiElement::new(text, false, None)]
    }

    fn styled_span(content: String, is_key: bool, theme: &Theme) -> Span<'static> {
        let style = if is_key {
            TermiStyle::highlight(theme)
        } else {
            TermiStyle::muted(theme)
        };
        Span::styled(content, style)
    }
}
