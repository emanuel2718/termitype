use crate::{
    modal::{Modal, ModalContext, ModalKind},
    theme::Theme,
    tui::helpers,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub struct ModalDialog;

impl ModalDialog {
    pub fn render(f: &mut Frame, modal: &Modal, theme: &Theme, area: Rect) {
        let (w, h) = match modal.kind {
            ModalKind::Input => (60, 12),
            ModalKind::Confirmation => (60, 10),
        };

        let modal_area = helpers::centered_fixed_rect(w, h, area);
        f.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.bg()));

        let inner_area = block.inner(modal_area);
        f.render_widget(block, modal_area);
        Self::render_modal_dialog(f, modal, theme, inner_area);
    }

    fn render_modal_dialog(f: &mut Frame, modal: &Modal, theme: &Theme, area: Rect) {
        match modal.kind {
            ModalKind::Input => Self::render_input_kind_modal(f, modal, theme, area),
            ModalKind::Confirmation => Self::render_confirmation_kind_modal(f, modal, theme, area),
        }
    }

    fn render_input_kind_modal(f: &mut Frame, modal: &Modal, theme: &Theme, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(vec![
                Constraint::Length(1), // title
                Constraint::Length(1), // space
                Constraint::Length(1), // desc
                Constraint::Length(1), // gap
                Constraint::Length(1), // input
                Constraint::Length(1), // gap/error
                Constraint::Length(1), // gap
                Constraint::Length(1), // ok button
            ])
            .split(area);

        Self::render_modal_title(f, &modal.title, theme, layout[0]);
        Self::render_modal_description(f, &modal.description, theme, layout[2]);
        Self::render_modal_input_field(f, modal, theme, layout[4]);
        Self::render_modal_error_message(f, modal, theme, layout[5]);
        Self::render_button(f, &modal.kind, theme, layout[7]);
    }

    fn render_confirmation_kind_modal(f: &mut Frame, modal: &Modal, theme: &Theme, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(vec![
                Constraint::Length(1), // title
                Constraint::Length(1), // gap
                Constraint::Length(1), // desc
                Constraint::Length(1), // gap
                Constraint::Length(1), // gap
                Constraint::Length(1), // yes button
            ])
            .split(area);

        Self::render_modal_title(f, &modal.title, theme, layout[0]);
        Self::render_modal_description(f, &modal.description, theme, layout[2]);
        Self::render_button(f, &modal.kind, theme, layout[5]);
    }

    fn render_modal_title(f: &mut Frame, title: &str, theme: &Theme, area: Rect) {
        let title = Paragraph::new(title)
            .style(Style::default().fg(theme.highlight()))
            .alignment(Alignment::Center);
        f.render_widget(title, area);
    }

    fn render_modal_description(frame: &mut Frame, description: &str, theme: &Theme, area: Rect) {
        let desc = Paragraph::new(description)
            .style(Style::default().fg(theme.fg()))
            .alignment(Alignment::Center);
        frame.render_widget(desc, area);
    }

    fn render_modal_input_field(f: &mut Frame, modal: &Modal, theme: &Theme, area: Rect) {
        let input_style = Style::default().fg(theme.fg());
        let cursor_style = Style::default().fg(theme.cursor_text()).bg(theme.cursor());
        let suffix_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);

        let (width, suffix) = match modal.ctx {
            ModalContext::CustomTime => (3, " second(s)"), // 300 is the max custom time
            ModalContext::CustomWordCount => (4, " word(s)"), // 5000 is the max custom word count
            ModalContext::CustomLineCount => (2, " line(s)"), // 10 is the max custom line
            _ => unreachable!(),
        };

        if let Some(buffer) = modal.buffer.clone() {
            let input_text = buffer.input;

            let input_str = &input_text[..input_text.len().min(width)];
            let cursor_pos = buffer.cursor_pos.min(width);

            let total_width = width + suffix.len();
            let left_padding = (area.width as usize).saturating_sub(total_width) / 2;

            let input_spans = vec![
                Span::raw(" ".repeat(left_padding)),
                Span::styled(&input_str[..cursor_pos], input_style),
                Span::styled(" ", cursor_style),
                Span::styled(&input_str[cursor_pos..], input_style),
                Span::raw(" ".repeat(width.saturating_sub(input_str.len()))),
                Span::styled(suffix, suffix_style),
            ];

            let input_field = Paragraph::new(Line::from(input_spans));
            f.render_widget(input_field, area);
        }
    }

    fn render_modal_error_message(frame: &mut Frame, modal: &Modal, theme: &Theme, area: Rect) {
        if let Some(buffer) = &modal.buffer {
            if let Some(error_msg) = &buffer.error {
                let error_text = Paragraph::new(error_msg.as_str())
                    .style(Style::default().fg(theme.error()))
                    .alignment(Alignment::Center);
                frame.render_widget(error_text, area);
            }
        }
    }

    fn render_button(frame: &mut Frame, kind: &ModalKind, theme: &Theme, area: Rect) {
        let text = match kind {
            ModalKind::Input => "<OK>",
            ModalKind::Confirmation => "<Yes>",
        };
        let style = Style::default()
            .fg(theme.bg())
            .bg(theme.highlight())
            .add_modifier(Modifier::BOLD);
        let padding = (area.width.saturating_sub(text.len() as u16)) / 2;
        let line = Line::from(vec![
            Span::raw(" ".repeat(padding as usize)),
            Span::styled(text, style),
        ]);
        frame.render_widget(Paragraph::new(line), area);
    }
}
