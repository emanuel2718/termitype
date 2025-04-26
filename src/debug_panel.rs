#[cfg(debug_assertions)]
use once_cell::sync::Lazy;
#[cfg(debug_assertions)]
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
#[cfg(debug_assertions)]
use std::collections::VecDeque;
#[cfg(debug_assertions)]
use std::sync::Mutex;

#[cfg(debug_assertions)]
use crate::{termi::Termi, theme::Theme};

#[cfg(debug_assertions)]
const MAX_LOG_ENTRIES: usize = 100;

#[cfg(debug_assertions)]
static GLOBAL_DEBUG: Lazy<Mutex<VecDeque<String>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES)));

/// GLOBAL DEBUG BECAUSE THIS IS DEBUG LAND
#[cfg(debug_assertions)]
#[allow(non_snake_case)]
pub fn LOG(message: impl Into<String>) {
    if let Ok(mut logs) = GLOBAL_DEBUG.lock() {
        let message = message.into();
        if logs.len() >= MAX_LOG_ENTRIES {
            logs.pop_front();
        }
        logs.push_back(message);
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Default)]
pub struct DebugPanel {
    pub visible: bool,
    state_scroll: usize,
}

#[cfg(debug_assertions)]
impl DebugPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn draw(&self, f: &mut Frame, termi: &Termi, area: Rect) {
        let theme = termi.get_current_theme();

        let debug_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3), // Content
            ])
            .split(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.bg()).fg(theme.border()))
            .title(" Debug Panel ");
        f.render_widget(block, area);

        self.draw_state_tab(f, termi, debug_area[0]);
    }

    fn create_state_line(
        &self,
        label: &str,
        value: impl std::fmt::Display,
        theme: &Theme,
    ) -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(theme.muted())),
            Span::styled(format!("{}", value), Style::default().fg(theme.highlight())),
        ])
    }

    fn draw_state_tab(&self, f: &mut Frame, termi: &Termi, area: Rect) {
        let theme = termi.get_current_theme();
        let state_text = vec![
            self.create_state_line("WPM", format!("{:.2}", termi.tracker.wpm), theme),
            self.create_state_line("Raw WPM", format!("{:.2}", termi.tracker.raw_wpm), theme),
            self.create_state_line("Accuracy", format!("{:.2}%", termi.tracker.accuracy), theme),
            self.create_state_line(
                "Time Paused",
                format!("{:?}", termi.tracker.time_paused),
                theme,
            ),
            self.create_state_line(
                "Cursor Position",
                format!("{}", termi.tracker.cursor_position),
                theme,
            ),
            self.create_state_line("Word Count", format!("{}", termi.tracker.word_count), theme),
            self.create_state_line(
                "User Input",
                format!("{}", termi.tracker.user_input.len()),
                theme,
            ),
            self.create_state_line(
                "Total Keystrokes",
                format!("{}", termi.tracker.total_keystrokes),
                theme,
            ),
            self.create_state_line(
                "Correct Keystrokes",
                format!("{}", termi.tracker.correct_keystrokes),
                theme,
            ),
            self.create_state_line(
                "Wrong Words Indexes",
                format!("{:?}", termi.tracker.wrong_words_start_indexes),
                theme,
            ),
            self.create_state_line("Status", format!("{:?}", termi.tracker.status), theme),
            self.create_state_line("Word Count", termi.tracker.word_count, theme),
        ];

        let content_area = area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        });
        let content_height = content_area.height as usize;
        let scroll_offset = self
            .state_scroll
            .min(state_text.len().saturating_sub(content_height));

        let state_widget = Paragraph::new(state_text.clone())
            .style(Style::default().fg(theme.fg()))
            .block(Block::default().padding(ratatui::widgets::Padding::new(1, 1, 1, 1)))
            .scroll((scroll_offset as u16, 0));

        f.render_widget(state_widget, content_area);

        if state_text.len() > content_height {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .style(Style::default().fg(theme.border()));

            let mut scrollbar_state = ScrollbarState::default()
                .content_length(state_text.len())
                .position(scroll_offset);

            f.render_stateful_widget(scrollbar, content_area, &mut scrollbar_state);
        }
    }
}
