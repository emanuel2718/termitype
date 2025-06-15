use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    actions::TermiClickAction,
    constants::{MODAL_HEIGHT, MODAL_WIDTH},
    modal::{InputModal, ModalContext},
    termi::Termi,
    ui::helpers::{LayoutHelper, TermiStyle},
};

pub struct ModalComponent;

impl ModalComponent {
    pub fn render(
        frame: &mut Frame,
        termi: &Termi,
        area: Rect,
        modal: InputModal,
    ) -> Option<(Rect, TermiClickAction)> {
        let theme = &termi.theme;
        let modal_area = LayoutHelper::center_div(MODAL_WIDTH, MODAL_HEIGHT, area);
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(TermiStyle::border(theme))
            .style(Style::default().bg(theme.bg()));

        let inner_area = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // title
                Constraint::Length(1), // desc
                Constraint::Length(1), // space
                Constraint::Length(1), // input
                Constraint::Length(1), // error/space
                Constraint::Length(1), // space
                Constraint::Length(1), // [ OK ]
            ])
            .split(inner_area);

        // title
        Self::render_modal_title(frame, &modal, theme, layout[0]);

        // description
        Self::render_modal_description(frame, &modal, theme, layout[1]);

        // input field
        Self::render_modal_input_field(frame, &modal, theme, layout[3]);

        // error message if present
        Self::render_modal_error_message(frame, &modal, theme, layout[4]);

        // [ OK ] button
        Self::render_modal_ok_button(frame, theme, layout[6])
    }

    fn render_modal_title(
        frame: &mut Frame,
        modal: &InputModal,
        theme: &crate::theme::Theme,
        area: Rect,
    ) {
        let title = Paragraph::new(modal.title.clone())
            .style(TermiStyle::highlight(theme))
            .alignment(Alignment::Center);

        frame.render_widget(title, area);
    }

    fn render_modal_description(
        frame: &mut Frame,
        modal: &InputModal,
        theme: &crate::theme::Theme,
        area: Rect,
    ) {
        let desc = Paragraph::new(modal.description.clone())
            .style(TermiStyle::muted(theme))
            .alignment(Alignment::Center);
        frame.render_widget(desc, area);
    }

    fn render_modal_input_field(
        frame: &mut Frame,
        modal: &InputModal,
        theme: &crate::theme::Theme,
        area: Rect,
    ) {
        let input_style = Style::default().fg(theme.fg());
        let cursor_style = Style::default().fg(theme.cursor_text()).bg(theme.cursor());
        let suffix_style = Style::default()
            .fg(theme.muted())
            .add_modifier(Modifier::DIM);

        let suffix = match modal.ctx {
            ModalContext::CustomTime => " second(s)",
            ModalContext::CustomWordCount => " word(s)",
        };

        let input_text = &modal.buffer.input;
        let cursor_pos = modal.buffer.cursor_pos;

        let display_text_width = (input_text.len() + 1 + suffix.len()) as u16;
        let padding = (area.width.saturating_sub(display_text_width)) / 2;
        let padding_span = Span::raw(" ".repeat(padding as usize));

        let input_spans = vec![
            padding_span,
            Span::styled(&input_text[..cursor_pos], input_style),
            Span::styled(" ", cursor_style),
            Span::styled(&input_text[cursor_pos..], input_style),
            Span::styled(suffix, suffix_style),
        ];

        let input_field = Paragraph::new(Line::from(input_spans));
        frame.render_widget(input_field, area);
    }

    fn render_modal_error_message(
        frame: &mut Frame,
        modal: &InputModal,
        theme: &crate::theme::Theme,
        area: Rect,
    ) {
        if let Some(error) = &modal.buffer.error_msg {
            let error_text = Paragraph::new(error.as_str())
                .style(TermiStyle::error(theme))
                .alignment(Alignment::Center);
            frame.render_widget(error_text, area);
        }
    }

    fn render_modal_ok_button(
        frame: &mut Frame,
        theme: &crate::theme::Theme,
        area: Rect,
    ) -> Option<(Rect, TermiClickAction)> {
        let ok_button = Paragraph::new("[ OK ]")
            .style(TermiStyle::highlight(theme))
            .alignment(Alignment::Center);
        frame.render_widget(ok_button, area);

        Some((area, TermiClickAction::ModalConfirm))
    }
}
