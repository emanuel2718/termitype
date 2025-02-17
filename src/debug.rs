use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs},
    Frame,
};
use std::collections::VecDeque;

use crate::{termi::Termi, theme::Theme};

const MAX_LOG_ENTRIES: usize = 100;

#[derive(Debug, Clone)]
pub struct Debug {
    pub visible: bool,
    pub current_tab: usize,
    pub logs: VecDeque<String>,
    state_scroll: usize,
    logs_scroll: usize,
    logs_auto_scroll: bool,
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            visible: false,
            current_tab: 0,
            logs: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            state_scroll: 0,
            logs_scroll: 0,
            logs_auto_scroll: true,
        }
    }
}

impl Debug {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn log(&mut self, message: impl Into<String>) {
        let message = message.into();
        if self.logs.len() >= MAX_LOG_ENTRIES {
            self.logs.pop_front();
        }
        self.logs.push_back(message);

        if self.logs_auto_scroll {
            self.logs_scroll = self.logs.len().saturating_sub(1);
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 2;
    }

    pub fn prev_tab(&mut self) {
        self.current_tab = if self.current_tab == 0 { 1 } else { 0 };
    }

    pub fn scroll_up(&mut self) {
        match self.current_tab {
            0 => self.state_scroll = self.state_scroll.saturating_sub(1),
            1 => {
                self.logs_scroll = self.logs_scroll.saturating_sub(1);
                self.logs_auto_scroll = false;
            }
            _ => unreachable!(),
        }
    }

    pub fn scroll_down(&mut self, max_lines: usize) {
        match self.current_tab {
            0 => self.state_scroll = (self.state_scroll + 1).min(max_lines.saturating_sub(1)),
            1 => {
                let new_scroll = (self.logs_scroll + 1).min(self.logs.len().saturating_sub(1));
                self.logs_scroll = new_scroll;
                self.logs_auto_scroll = new_scroll >= self.logs.len().saturating_sub(1);
            }
            _ => unreachable!(),
        }
    }

    pub fn toggle_auto_scroll(&mut self) {
        if self.current_tab == 1 {
            self.logs_auto_scroll = !self.logs_auto_scroll;
            if self.logs_auto_scroll {
                self.logs_scroll = self.logs.len().saturating_sub(1);
            }
        }
    }

    pub fn draw(&self, f: &mut Frame, termi: &Termi, area: Rect) {
        let theme = termi.get_current_theme();

        let debug_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(3),    // Content
            ])
            .split(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.background()).fg(theme.border()))
            .title(" Debug Panel ");
        f.render_widget(block, area);

        self.draw_tabs(f, theme, debug_area[0]);

        match self.current_tab {
            0 => self.draw_state_tab(f, termi, debug_area[1]),
            1 => self.draw_logs_tab(f, theme, debug_area[1]),
            _ => unreachable!(),
        }
    }

    fn draw_tabs(&self, f: &mut Frame, theme: &Theme, area: Rect) {
        let titles = vec!["State", "Logs"];
        let tabs = Tabs::new(titles)
            .style(Style::default().fg(theme.muted()))
            .highlight_style(
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            )
            .select(self.current_tab);
        f.render_widget(tabs, area);
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

        let content_height = area.height.saturating_sub(2) as usize;
        let scroll_offset = self
            .state_scroll
            .min(state_text.len().saturating_sub(content_height));

        let state_widget = Paragraph::new(state_text.clone())
            .style(Style::default().fg(theme.foreground()))
            .block(Block::default().padding(ratatui::widgets::Padding::new(1, 1, 0, 0)))
            .scroll((scroll_offset as u16, 0));

        f.render_widget(state_widget, area);

        // render scrollbar if content exceeds view height
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

            f.render_stateful_widget(
                scrollbar,
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 1,
                }),
                &mut scrollbar_state,
            );
        }
    }

    fn draw_logs_tab(&self, f: &mut Frame, theme: &Theme, area: Rect) {
        let logs: Vec<Line> = self
            .logs
            .iter()
            .map(|log| {
                Line::from(vec![Span::styled(
                    log,
                    Style::default().fg(theme.foreground()),
                )])
            })
            .collect();

        let content_height = area.height.saturating_sub(2) as usize;
        let scroll_offset = self
            .logs_scroll
            .min(logs.len().saturating_sub(content_height));

        let logs_widget = Paragraph::new(logs.clone())
            .style(Style::default().fg(theme.foreground()))
            .block(Block::default().padding(ratatui::widgets::Padding::new(1, 1, 0, 0)))
            .scroll((scroll_offset as u16, 0));

        f.render_widget(logs_widget, area);

        if logs.len() > content_height {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .style(Style::default().fg(theme.border()));

            let mut scrollbar_state = ScrollbarState::default()
                .content_length(logs.len())
                .position(scroll_offset);

            f.render_stateful_widget(
                scrollbar,
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 1,
                }),
                &mut scrollbar_state,
            );
        }
    }
}
