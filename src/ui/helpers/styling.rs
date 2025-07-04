use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::theme::Theme;

pub struct TermiStyle;

impl TermiStyle {
    pub fn highlight(theme: &Theme) -> Style {
        Style::default()
            .fg(theme.highlight())
            .add_modifier(Modifier::BOLD)
    }

    pub fn muted(theme: &Theme) -> Style {
        Style::default()
            .fg(theme.muted())
            .add_modifier(Modifier::DIM)
    }

    pub fn success(theme: &Theme) -> Style {
        Style::default().fg(theme.success())
    }

    pub fn error(theme: &Theme) -> Style {
        Style::default().fg(theme.error())
    }

    pub fn success_underlined(theme: &Theme) -> Style {
        if theme.color_support.supports_themes() {
            Self::success(theme)
                .add_modifier(Modifier::UNDERLINED)
                .underline_color(theme.error())
        } else {
            Self::success(theme)
        }
    }

    pub fn error_underlined(theme: &Theme) -> Style {
        if theme.color_support.supports_themes() {
            Self::error(theme)
                .add_modifier(Modifier::UNDERLINED)
                .underline_color(theme.error())
        } else {
            Self::error(theme)
        }
    }

    pub fn skipped(theme: &Theme) -> Style {
        if theme.color_support.supports_themes() {
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::UNDERLINED | Modifier::DIM)
                .underline_color(theme.error())
        } else {
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM)
        }
    }

    pub fn dim(theme: &Theme) -> Style {
        Style::default().fg(theme.fg()).add_modifier(Modifier::DIM)
    }

    pub fn active_or_muted(is_active: bool, theme: &Theme) -> Style {
        if is_active {
            Self::highlight(theme)
        } else {
            Self::muted(theme)
        }
    }

    pub fn border(theme: &Theme) -> Style {
        Style::default()
            .fg(theme.border())
            .add_modifier(Modifier::DIM)
    }

    pub fn warning(theme: &Theme) -> Style {
        Style::default().fg(theme.warning())
    }
}

pub fn create_key_value_line(key: &str, value: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{key}: "),
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), Style::default().fg(theme.fg())),
    ])
}

pub fn create_command_hint(key: &str, description: &str, theme: &Theme) -> Vec<Span<'static>> {
    vec![
        Span::styled(key.to_string(), TermiStyle::highlight(theme)),
        Span::styled(format!(" {description}"), TermiStyle::muted(theme)),
    ]
}

pub fn create_separator(theme: &Theme, supports_unicode: bool) -> Span<'static> {
    let divider = if supports_unicode { "│" } else { "|" };
    Span::styled(divider.to_string(), TermiStyle::muted(theme))
}
