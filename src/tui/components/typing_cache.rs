use crate::{theme::Theme, tracker::Tracker};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

#[derive(Debug, Default, Clone)]
pub struct TypingRenderCache {
    revision: u64,
    width: u16,
    line_count: u8,
    theme_revision: u64,
    lines: Vec<Line<'static>>,
    cursor_line: usize,
    cursor_x: u16,
}

impl TypingRenderCache {
    pub fn invalidate(&mut self) {
        self.revision = u64::MAX;
    }

    pub fn ensure(
        &mut self,
        tracker: &Tracker,
        theme: &Theme,
        width: u16,
        line_count: u8,
        revision: u64,
    ) {
        let unchanged = self.revision == revision
            && self.width == width
            && self.line_count == line_count
            && self.theme_revision == theme.revision();
        if unchanged {
            return;
        }

        let (lines, cursor_line, cursor_x) = build_target_text_lines(tracker, theme, width);

        self.lines = lines;
        self.cursor_line = cursor_line;
        self.cursor_x = cursor_x;
        self.revision = revision;
        self.width = width;
        self.line_count = line_count;
        self.theme_revision = theme.revision();
    }

    pub fn lines(&self) -> &[Line<'static>] {
        &self.lines
    }

    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }

    pub fn cursor_x(&self) -> u16 {
        self.cursor_x
    }
}

fn build_target_text_lines(
    state: &Tracker,
    theme: &Theme,
    max_width: u16,
) -> (Vec<Line<'static>>, usize, u16) {
    let mut spans = Vec::with_capacity(state.tokens.len());
    let mut word_idx = 0;

    for (i, token) in state.tokens.iter().enumerate() {
        if token.target == ' ' {
            word_idx += 1;
        }

        let is_past_wrong_word = word_idx < state.current_word_idx && state.is_word_wrong(word_idx);
        let fg_color = if token.is_skipped {
            theme.fg()
        } else if i < state.current_pos {
            if token.is_wrong {
                theme.error()
            } else {
                theme.success()
            }
        } else {
            theme.fg()
        };

        let mut style = Style::default().fg(fg_color);
        if token.is_skipped || i >= state.current_pos {
            style = style.add_modifier(Modifier::DIM);
        }

        if token.is_extra_token() {
            style = style.add_modifier(Modifier::DIM);
        }

        if is_past_wrong_word && token.target != ' ' {
            style = style
                .add_modifier(Modifier::UNDERLINED)
                .underline_color(theme.error());
        }

        spans.push(Span::styled(token.target.to_string(), style));
    }

    let mut lines = Vec::new();
    let mut current_line: Vec<Span<'static>> = Vec::new();
    let mut current_width: u16 = 0;

    for span in spans {
        let span_width = span.content.len() as u16;
        if current_width + span_width > max_width {
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

    let (cursor_line, cursor_x) = resolve_cursor(lines.as_slice(), state.current_pos);
    (lines, cursor_line, cursor_x)
}

fn resolve_cursor(lines: &[Line<'static>], current_pos: usize) -> (usize, u16) {
    let mut cumulative = 0;
    let mut cursor_line = 0;
    let mut cursor_x = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_len = line.spans.iter().map(|s| s.content.len()).sum::<usize>();
        if current_pos < cumulative + line_len {
            cursor_line = i;
            cursor_x = (current_pos - cumulative) as u16;
            break;
        }
        cumulative += line_len;
    }

    (cursor_line, cursor_x)
}
