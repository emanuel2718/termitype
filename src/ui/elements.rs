use crate::{
    config::Mode,
    constants::{APPNAME, DEFAULT_LANGUAGE, MIN_TERM_HEIGHT, MIN_TERM_WIDTH},
    termi::Termi,
    tracker::Status,
};
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use super::actions::TermiClickAction;

pub struct TermiElement<'a> {
    pub widget: Paragraph<'a>,
    pub is_active: bool,
    pub action: Option<TermiClickAction>,
}

impl<'a> From<Paragraph<'a>> for TermiElement<'a> {
    fn from(widget: Paragraph<'a>) -> Self {
        Self {
            widget,
            is_active: false,
            action: None,
        }
    }
}

pub fn create_header(termi: &Termi) -> TermiElement {
    let theme = termi.get_current_theme();
    Paragraph::new(APPNAME)
        .style(Style::default().fg(theme.highlight()))
        .add_modifier(if termi.tracker.status == Status::Typing {
            Modifier::DIM
        } else {
            Modifier::empty()
        })
        .alignment(Alignment::Left)
        .into()
}

pub fn create_action_bar(_termi: &Termi) -> TermiElement {
    Paragraph::new("TODO: add action bar logic")
        .alignment(Alignment::Center)
        .into()
}

pub fn create_mode_bar(termi: &Termi) -> TermiElement {
    let status = termi.tracker.status.clone();
    let theme = termi.get_current_theme().clone();
    match status {
        Status::Idle | Status::Paused => {
            let current_language = termi.config.language.as_deref().unwrap_or(DEFAULT_LANGUAGE);
            let widget = Paragraph::new(current_language)
                .centered()
                .style(Style::new().fg(theme.muted()));
            TermiElement {
                widget,
                is_active: false,
                action: Some(TermiClickAction::ToggleLanguagePicker),
            }
        }
        Status::Typing => {
            let info = match termi.config.current_mode() {
                Mode::Time { duration } => {
                    if let Some(rem) = termi.tracker.time_remaining {
                        format!("{:.0}", rem.as_secs())
                    } else {
                        format!("{}", duration)
                    }
                }
                Mode::Words { count } => {
                    // TODO: could have a nice helper for this to not have to do this here. or better yet track this better
                    let completed = termi
                        .tracker
                        .user_input
                        .iter()
                        .filter(|&&c| c == Some(' '))
                        .count();
                    format!("{}/{}", completed, count)
                }
            };
            let wpm = format!(" {:>3.0} wpm", termi.tracker.wpm);
            let spans = vec![
                Span::styled(info, Style::default().fg(theme.highlight())),
                Span::styled(
                    wpm,
                    Style::default()
                        .fg(theme.muted())
                        .add_modifier(Modifier::DIM),
                ),
            ];
            Paragraph::new(Line::from(spans)).into()
        }
        _ => Paragraph::new("").into(),
    }
}

pub fn create_typing_area(termi: &Termi) -> TermiElement {
    let words_text = termi.words.clone();
    Paragraph::new(words_text).into()
}

pub fn create_command_bar(_termi: &Termi) -> TermiElement {
    Paragraph::new("TODO: add command bar logic")
        .alignment(Alignment::Center)
        .into()
}

pub fn create_footer(_termi: &Termi) -> TermiElement {
    Paragraph::new("TODO: add footer logic")
        .alignment(Alignment::Center)
        .into()
}

pub fn create_minimal_size_warning(termi: &Termi, width: u16, height: u16) -> TermiElement {
    let theme = termi.get_current_theme().clone();
    let warning = vec![
        Line::from(Span::styled(
            "! too small",
            Style::default().fg(theme.error()),
        )),
        Line::from(Span::styled(
            format!("Current: ({}x{})", width, height),
            Style::default().fg(theme.muted()),
        )),
        Line::from(Span::styled(
            format!("Minimum: ({}x{})", MIN_TERM_WIDTH, MIN_TERM_HEIGHT),
            Style::default().fg(theme.muted()),
        )),
    ];
    Paragraph::new(warning)
        .alignment(Alignment::Center)
        .block(Block::new().padding(Padding::new(0, 0, (height / 2).saturating_sub(1), 0)))
        .into()
}

pub fn create_show_menu_button(_termi: &Termi) -> TermiElement {
    Paragraph::new("TODO: <icon> Show Menu")
        .alignment(Alignment::Center)
        .into()
}
