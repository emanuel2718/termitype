use crate::theme;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

pub fn create_target_text_line(
    state: &crate::tracker::Tracker,
    theme: &theme::Theme,
    max_width: u16,
) -> Vec<Line<'static>> {
    let mut spans = Vec::new();

    for (i, token) in state.tokens.iter().enumerate() {
        let style = if i < state.current_pos {
            // character already typed
            if token.is_wrong {
                Style::default()
                    .fg(theme.error())
                    .remove_modifier(Modifier::DIM)
            } else {
                Style::default()
                    .fg(theme.success())
                    .remove_modifier(Modifier::DIM)
            }
        } else {
            // upcoming
            Style::default().fg(theme.fg()).add_modifier(Modifier::DIM)
        };

        spans.push(Span::styled(token.target.to_string(), style));
    }

    let mut lines = Vec::new();
    let mut current_line: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0;
    for span in spans {
        let span_width = span.content.len() as u16;
        if current_width + span_width > max_width {
            // breakpoints
            let mut break_index = current_line.len();
            for (i, s) in current_line.iter().enumerate().rev() {
                if s.content == " " {
                    break_index = i + 1;
                    break;
                }
            }
            if break_index < current_line.len() {
                let next_line = current_line.split_off(break_index);
                lines.push(Line::from(current_line));
                current_line = next_line;
                current_width = current_line.iter().map(|s| s.content.len() as u16).sum();
            } else {
                lines.push(Line::from(current_line));
                current_line = Vec::new();
                current_width = 0;
            }
            current_line.push(span);
            current_width += span_width;
        } else {
            current_line.push(span);
            current_width += span_width;
        }
    }
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }
    lines
}

pub fn wrap_text(text: &str, max_width: u16) -> Vec<Line<'static>> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in words {
        let potential = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };
        if potential.len() as u16 > max_width {
            if !current_line.is_empty() {
                lines.push(Line::from(current_line));
                current_line = word.to_string();
            } else {
                // the word is longer than `max_width`, add as is (may wrap)
                lines.push(Line::from(word.to_string()));
            }
        } else {
            current_line = potential;
        }
    }
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }
    lines
}
