use ratatui::{
    layout::Alignment,
    text::{Line, Span, Text},
};

use crate::{
    actions::TermiClickAction, config::Mode, constants::DEFAULT_LANGUAGE, termi::Termi,
    tracker::Status, ui::helpers::TermiStyle,
};

use super::elements::TermiElement;

pub struct ModeBarComponent;

impl ModeBarComponent {
    pub fn create(termi: &Termi) -> Vec<TermiElement> {
        let status = termi.tracker.status.clone();
        let theme = termi.current_theme();

        let element = match status {
            Status::Idle | Status::Paused => {
                let current_language = termi.config.language.as_deref().unwrap_or(DEFAULT_LANGUAGE);
                let text = Text::from(current_language)
                    .style(TermiStyle::muted(theme))
                    .alignment(Alignment::Center);
                TermiElement::new(text, false, Some(TermiClickAction::ToggleLanguagePicker))
            }
            Status::Typing => {
                let info = match termi.config.current_mode() {
                    Mode::Time { duration } => {
                        if let Some(rem) = termi.tracker.time_remaining {
                            format!("{:.0}", rem.as_secs())
                        } else {
                            format!("{duration}")
                        }
                    }
                    Mode::Words { count } => {
                        let completed = termi
                            .tracker
                            .user_input
                            .iter()
                            .filter(|&&c| c == Some(' '))
                            .count();
                        format!("{completed}/{count}")
                    }
                };
                let wpm = format!(" {:>3.0} wpm", termi.tracker.wpm);
                let mut spans = vec![Span::styled(info, TermiStyle::highlight(theme))];

                // NOTE: the live wpm is an option toggleable by the user
                if !termi.config.hide_live_wpm {
                    spans.push(Span::styled(wpm, TermiStyle::muted(theme)));
                }
                let line = Line::from(spans);
                let text = Text::from(line);
                TermiElement::new(text, false, None)
            }
            _ => TermiElement::new(Text::raw(""), false, None),
        };

        vec![element]
    }
}
