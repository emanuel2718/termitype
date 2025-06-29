use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Text,
};

use crate::{actions::TermiClickAction, theme::Theme};

#[derive(Debug)]
pub struct TermiElement<'a> {
    pub content: Text<'a>,
    pub action: Option<TermiClickAction>,
    pub is_active: bool,
}

impl<'a> TermiElement<'a> {
    pub fn new(
        content: impl Into<Text<'a>>,
        is_active: bool,
        action: Option<TermiClickAction>,
    ) -> Self {
        Self {
            content: content.into(),
            is_active,
            action,
        }
    }

    pub fn to_styled(mut self, theme: &Theme) -> Self {
        let style_to_apply = if self.is_active {
            Style::default()
                .fg(theme.highlight())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM)
        };

        for line in self.content.lines.iter_mut() {
            for span in line.spans.iter_mut() {
                span.style = span.style.patch(style_to_apply);
            }
        }

        self
    }

    pub fn text(content: impl Into<Text<'a>>) -> Self {
        Self::new(content, false, None)
    }

    pub fn active(content: impl Into<Text<'a>>, action: Option<TermiClickAction>) -> Self {
        Self::new(content, true, action)
    }

    pub fn inactive(content: impl Into<Text<'a>>, action: Option<TermiClickAction>) -> Self {
        Self::new(content, false, action)
    }

    pub fn spacer(width: usize) -> Self {
        Self::text(" ".repeat(width))
    }
}

#[derive(Debug, Default)]
pub struct TermiClickableRegions {
    pub regions: Vec<(Rect, TermiClickAction)>, // TODO: change this to a struct { rect, action }
}

impl TermiClickableRegions {
    pub fn add(&mut self, area: Rect, action: TermiClickAction) {
        if area.width > 0 && area.height > 0 {
            self.regions.push((area, action));
        }
    }
}
