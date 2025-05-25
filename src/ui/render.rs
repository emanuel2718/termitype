use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Clear, Dataset, GraphType, List, Padding,
        Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::{
    actions::TermiClickAction,
    ascii,
    config::Mode,
    constants::{MIN_THEME_PREVIEW_WIDTH, MODAL_HEIGHT, MODAL_WIDTH, TYPING_AREA_WIDTH},
    modal::InputModal,
    termi::Termi,
    tracker::Status,
    version::VERSION,
};

use super::{
    elements::{
        build_menu_items, create_action_bar, create_command_bar, create_footer, create_header,
        create_menu_footer_text, create_minimal_size_warning, create_mode_bar,
        create_results_footer_text, create_show_menu_button, create_typing_area, TermiElement,
    },
    layout::create_layout,
    utils::{
        apply_horizontal_centering, calculate_menu_area, calculate_menu_area_from_parts,
        calculate_word_positions, center_div,
    },
};
use crate::modal::ModalContext;

#[derive(Debug, Default)]
pub struct TermiClickableRegions {
    pub regions: Vec<(Rect, TermiClickAction)>,
}

impl TermiClickableRegions {
    pub fn add(&mut self, area: Rect, action: TermiClickAction) {
        if area.width > 0 && area.height > 0 {
            self.regions.push((area, action));
        }
    }
}

/// Main entry point for the rendering
pub fn draw_ui(frame: &mut Frame, termi: &mut Termi, fps: Option<f64>) -> TermiClickableRegions {
    let mut regions = TermiClickableRegions::default();

    let theme = termi.current_theme();
    let area = frame.area();

    let dummy_layout = create_layout(Block::new().inner(area), termi);
    let container =
        Block::new()
            .style(Style::default().bg(theme.bg()))
            .padding(if dummy_layout.is_minimal() {
                Padding::ZERO
            } else if dummy_layout.w_small() {
                Padding::uniform(1)
            } else {
                Padding::symmetric(2, 1)
            });

    let inner_area = container.inner(area);
    let layout = if dummy_layout.is_minimal() {
        dummy_layout
    } else {
        create_layout(inner_area, termi)
    };

    frame.render_widget(container, area);

    if layout.is_minimal() {
        let warning = create_minimal_size_warning(termi, inner_area.width, inner_area.height);
        render(frame, &mut regions, warning, area);
        return regions;
    }

    let layout = create_layout(inner_area, termi);

    let mut fps_widget: Option<(Paragraph, Rect)> = None;
    if let Some(fps_value) = fps {
        let fps_text = format!("FPS: {:.0}", fps_value);
        let widget = Paragraph::new(fps_text)
            .style(Style::default().fg(theme.muted()))
            .add_modifier(Modifier::DIM)
            .alignment(Alignment::Right);

        let widget_width = 10;
        let fps_area = Rect::new(
            area.right().saturating_sub(widget_width + 1),
            area.top() + 1,
            widget_width,
            1,
        );
        fps_widget = Some((widget, fps_area));
    }

    let header = create_header(termi);

    match termi.tracker.status {
        Status::Typing => {
            let mode_bar = create_mode_bar(termi);
            render(frame, &mut regions, header, layout.section.header);

            let available_width = layout.section.typing_area.width as usize;

            let typing_area_width = available_width.min(TYPING_AREA_WIDTH as usize);

            let x_offset = layout
                .section
                .typing_area
                .width
                .saturating_sub(typing_area_width as u16)
                / 2;

            let x_pos = layout.section.typing_area.x + x_offset;

            let mode_bar_rect = Rect {
                x: x_pos,
                y: layout.section.mode_bar.y,
                width: typing_area_width as u16,
                height: layout.section.mode_bar.height,
            };

            render(frame, &mut regions, mode_bar, mode_bar_rect);
            render_typing_area(frame, termi, layout.section.typing_area);
        }
        Status::Completed => {
            render_results_screen(frame, termi, area, layout.show_small_results());
        }
        _ => {
            let mode_bar = create_mode_bar(termi);
            let command_bar = create_command_bar(termi);
            let footer = create_footer(termi);
            render(frame, &mut regions, mode_bar, layout.section.mode_bar);
            render(frame, &mut regions, header, layout.section.header);
            render_typing_area(frame, termi, layout.section.typing_area);
            // small width big enough height
            if layout.w_small() && !layout.h_small() {
                let menu_button = create_show_menu_button(termi);
                render(frame, &mut regions, menu_button, layout.section.action_bar);
                // TODO: there has to be a better way
                if layout.show_footer() {
                    render(frame, &mut regions, command_bar, layout.section.command_bar);
                    render(frame, &mut regions, footer, layout.section.footer);
                }
            } else if !layout.is_small() {
                let action_bar = create_action_bar(termi);
                render(frame, &mut regions, action_bar, layout.section.action_bar);
                // TODO: there has to be a better way
                if layout.show_footer() {
                    render(frame, &mut regions, command_bar, layout.section.command_bar);
                    render(frame, &mut regions, footer, layout.section.footer);
                }
            }
        }
    }

    if termi.menu.is_open() {
        render_menu(frame, termi, area);
    }

    if let Some(modal) = &termi.modal {
        if let Some(region) = render_modal(frame, termi, area, modal.clone()) {
            regions.add(region.0, region.1);
        }
    }

    if let Some((widget, fps_area)) = fps_widget {
        if fps_area.right() <= frame.area().right() && fps_area.bottom() <= frame.area().bottom() {
            frame.render_widget(widget, fps_area);
        }
    }

    regions
}

fn render(f: &mut Frame, cr: &mut TermiClickableRegions, elements: Vec<TermiElement>, area: Rect) {
    if elements.is_empty() {
        return;
    }

    if elements.len() == 1 {
        let element = &elements[0];
        let alignment = element.content.alignment.unwrap_or(Alignment::Left);
        let text_height = element.content.height() as u16;

        let centered_area = Rect {
            x: area.x,
            y: area.y + (area.height.saturating_sub(text_height)) / 2,
            width: area.width,
            height: area.height,
        };

        let paragraph = Paragraph::new(element.content.clone()).alignment(alignment);
        f.render_widget(paragraph, centered_area);

        if let Some(action) = element.action {
            cr.add(centered_area, action);
        }
    } else {
        let mut spans = Vec::new();
        let mut clickable_regions_to_add = Vec::new();
        let mut total_width: u16 = 0;

        let mut element_data = Vec::new();
        for element in &elements {
            let line_width = element.content.lines.first().map_or(0, |line| line.width()) as u16;
            total_width += line_width;
            element_data.push((line_width, element.action));
        }

        // if content fits, center it. left align otherwise
        let start_x = if total_width <= area.width {
            area.x + (area.width.saturating_sub(total_width)) / 2
        } else {
            area.x
        };

        let mut current_x_offset: u16 = 0;
        for (i, element) in elements.iter().enumerate() {
            let (element_width, action) = element_data[i];

            if let Some(line) = element.content.lines.first() {
                spans.extend(line.spans.clone());
            }

            if let Some(action) = action {
                let region_rect = Rect {
                    x: start_x + current_x_offset,
                    y: area.y,
                    width: element_width,
                    height: area.height.min(1),
                };
                if element_width > 0 {
                    clickable_regions_to_add.push((region_rect, action));
                }
            }
            current_x_offset += element_width;
        }

        let alignment = if total_width <= area.width {
            Alignment::Center
        } else {
            Alignment::Left
        };

        f.render_widget(Paragraph::new(Line::from(spans)).alignment(alignment), area);

        for (rect, action) in clickable_regions_to_add {
            cr.add(rect, action);
        }
    }
}

fn render_typing_area(frame: &mut Frame, termi: &Termi, area: Rect) {
    let available_width = area.width.min(TYPING_AREA_WIDTH) as usize;
    let line_count = termi.config.visible_lines as usize;
    let cursor_idx = termi.tracker.cursor_position;

    let word_positions = calculate_word_positions(&termi.words, available_width);

    if word_positions.is_empty() {
        frame.render_widget(Paragraph::new(Text::raw("")), area);
        return;
    }

    let current_word_pos = word_positions
        .iter()
        .filter(|pos| cursor_idx >= pos.start_index)
        .next_back()
        .unwrap_or_else(|| word_positions.first().unwrap());

    let current_line = current_word_pos.line;

    let scroll_offset = if line_count <= 1 {
        current_line
    } else {
        let half_visible = line_count / 2;
        if current_line < half_visible {
            0
        } else {
            current_line.saturating_sub(half_visible)
        }
    };

    let typing_text = create_typing_area(termi, scroll_offset, line_count, &word_positions);

    let text_height = typing_text.height();

    let area_width = available_width as u16;
    let area_padding = area.width.saturating_sub(area_width) / 2;
    let render_area = Rect {
        x: area.x + area_padding,
        y: area.y,
        width: area_width,
        height: area.height,
    };

    let paragraph = Paragraph::new(typing_text).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, render_area);

    // Menu overlap check logic
    let estimated_menu_area = calculate_menu_area(termi, frame.area());
    let show_cursor = (termi.tracker.status == Status::Idle
        || termi.tracker.status == Status::Typing)
        && (!termi.menu.is_open() || !estimated_menu_area.intersects(render_area))
        && termi.modal.is_none()
        && !termi.menu.is_theme_menu();

    if show_cursor {
        let offset_x = cursor_idx.saturating_sub(current_word_pos.start_index);
        let cursor_x = render_area.x + current_word_pos.col.saturating_add(offset_x) as u16;
        let cursor_y = render_area.y + current_line.saturating_sub(scroll_offset) as u16;

        // boundary check
        if cursor_x >= render_area.left()
            && cursor_x < render_area.right()
            && cursor_y >= render_area.top()
            && cursor_y < render_area.top() + text_height as u16
        {
            frame.set_cursor_position(Position {
                x: cursor_x,
                y: cursor_y,
            });
        }
    }
}

fn render_modal(
    frame: &mut Frame,
    termi: &Termi,
    area: Rect,
    modal: InputModal,
) -> Option<(Rect, TermiClickAction)> {
    let theme = termi.theme.clone();
    let modal_area = center_div(MODAL_WIDTH, MODAL_HEIGHT, area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.fg()))
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

    // ------ TITLE ------
    let title_area = layout[0];
    let title = Paragraph::new(modal.title.clone())
        .style(
            Style::default()
                .fg(theme.highlight())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    frame.render_widget(title, title_area);

    // ------ DESCRIPTION ------
    let desc_area = layout[1];
    let desc = Paragraph::new(modal.description.clone())
        .style(Style::default().fg(theme.muted()))
        .alignment(Alignment::Center);
    frame.render_widget(desc, desc_area);

    // ------ INPUT FIELD ------
    let input_area = layout[3];
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
    let padding = (input_area.width.saturating_sub(display_text_width)) / 2;
    let padding_span = Span::raw(" ".repeat(padding as usize));

    let input_spans = vec![
        padding_span,
        Span::styled(&input_text[..cursor_pos], input_style),
        Span::styled(" ", cursor_style),
        Span::styled(&input_text[cursor_pos..], input_style),
        Span::styled(suffix, suffix_style),
    ];

    let input_field = Paragraph::new(Line::from(input_spans));
    frame.render_widget(input_field, input_area);

    // ------ ERROR ------
    let error_area = layout[4];
    if let Some(error) = &modal.buffer.error_msg {
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(theme.error()))
            .alignment(Alignment::Center);
        frame.render_widget(error_text, error_area);
    }

    // ------ [ OK ] ------
    let ok_area = layout[6];
    let ok_button = Paragraph::new("[ OK ]")
        .style(Style::default().fg(theme.highlight()))
        .alignment(Alignment::Center);
    frame.render_widget(ok_button, ok_area);

    Some((ok_area, TermiClickAction::ModalConfirm))
}

fn render_menu(frame: &mut Frame, termi: &mut Termi, area: Rect) {
    let theme = termi.current_theme().clone();
    let menu_state = &mut termi.menu;

    let is_theme_picker = menu_state.is_theme_menu();
    let is_help_menu = menu_state.is_help_menu();
    let is_about_menu = menu_state.is_about_menu();
    let is_ascii_art_picker = menu_state.is_ascii_art_menu();

    let small_width = area.width <= MIN_THEME_PREVIEW_WIDTH;

    let picker_style = termi.config.resolve_picker_style();
    let base_rect = calculate_menu_area_from_parts(
        picker_style,
        is_theme_picker,
        is_help_menu,
        is_about_menu,
        is_ascii_art_picker,
        area,
    );

    // NOTE(ema): this is starting to get annoying. Find better way to determine this
    let (menu_area, preview_area) = if picker_style == crate::config::PickerStyle::Minimal {
        // TODO: need to decouple this from here as later adding more custom picker could be annoying
        // if we have a minimal picker don't fold the menu ever as we don't want previews
        (base_rect, None)
    } else if (is_theme_picker || is_ascii_art_picker) && !small_width {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(base_rect);
        (split[0], Some(split[1]))
    } else if (is_theme_picker || is_help_menu || is_about_menu || is_ascii_art_picker)
        && small_width
    {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(base_rect);
        (split[0], Some(split[1]))
    } else {
        (base_rect, None)
    };

    let menu_area = menu_area.intersection(area);

    frame.render_widget(Clear, menu_area);

    if let Some(preview) = preview_area {
        frame.render_widget(Clear, preview);
    }

    let hide_menu_footer = small_width && !menu_state.is_searching();
    let footer_len = if hide_menu_footer { 0 } else { 3 };
    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(footer_len)])
        .split(menu_area);
    let content_area = menu_layout[0];
    let footer_area = menu_layout[1];

    // FIXME: not a good idea to update menu state directly from here.
    // current menu height - (top border + bottom border)
    menu_state.ui_height = content_area.height.saturating_sub(2).max(1) as usize;

    if let Some(current_menu) = menu_state.current_menu() {
        let max_visible = content_area.height.saturating_sub(3) as usize;

        let total_items = current_menu.items().len();

        let scroll_offset = if total_items <= max_visible || max_visible == 0 {
            0
        } else {
            let halfway = max_visible / 2;
            let selected_index = current_menu.current_selection_index();
            if selected_index < halfway {
                0
            } else if selected_index >= total_items.saturating_sub(halfway) {
                total_items.saturating_sub(max_visible)
            } else {
                selected_index.saturating_sub(halfway)
            }
        };
        let menu_title = current_menu.title.clone();

        // hide description on the top fold
        let hide_description = (is_help_menu || is_about_menu) && small_width;
        let (list_items, total_items) =
            build_menu_items(termi, scroll_offset, max_visible, hide_description);

        let content_block = Block::default()
            .title(menu_title)
            .title_alignment(Alignment::Left)
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.bg()));

        let menu_widget = List::new(list_items)
            .block(content_block)
            .style(Style::default().bg(theme.bg()));

        frame.render_widget(menu_widget, content_area);

        if total_items > max_visible {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .style(Style::default().fg(theme.accent()));

            let scrollbar_area = content_area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            });

            let mut scrollbar_state = ScrollbarState::new(total_items).position(scroll_offset);

            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }

        // theme, description, or ascii art previews
        if let Some(preview_area) = preview_area {
            if is_theme_picker {
                render_theme_preview(frame, termi, preview_area);
            } else if is_ascii_art_picker {
                render_ascii_art_preview(frame, termi, preview_area);
            } else if (is_help_menu || is_about_menu) && small_width {
                render_description_preview(frame, termi, preview_area);
            }
        }
    }
    // don't render the menu footer text if we are in small width
    if !hide_menu_footer {
        let footer_text = create_menu_footer_text(termi);
        let footer_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.bg()));

        let footer_widget = Paragraph::new(footer_text)
            .block(footer_block)
            .style(Style::default().bg(theme.bg()))
            .alignment(Alignment::Left);

        frame.render_widget(footer_widget, footer_area);
    }
}

/// Renders the description portion of menu items with <key> <description> such as `Help` and `About` menus
fn render_description_preview(frame: &mut Frame, termi: &Termi, area: Rect) {
    let theme = termi.current_theme();
    frame.render_widget(Clear, area);

    let preview_block = Block::default()
        .title(" Description ")
        .title_alignment(Alignment::Left)
        .bg(theme.bg())
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(
            Style::default()
                .fg(theme.border())
                .add_modifier(Modifier::DIM),
        );

    let inner_area = preview_block.inner(area);
    frame.render_widget(preview_block, area);

    if let Some(menu) = termi.menu.current_menu() {
        if let Some(item) = menu.current_item() {
            let description_text = Text::from(item.label.as_str())
                .style(Style::default().fg(theme.fg()))
                .alignment(Alignment::Left);

            let paragraph = Paragraph::new(description_text)
                .wrap(Wrap { trim: false })
                .block(Block::default().padding(Padding::new(1, 1, 1, 1)));

            frame.render_widget(paragraph, inner_area);
        }
    }
}

fn render_theme_preview(frame: &mut Frame, termi: &Termi, area: Rect) {
    let theme = termi.current_theme();
    frame.render_widget(Clear, area);

    let preview_block = Block::default()
        .bg(theme.bg())
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(
            Style::default()
                .fg(theme.border())
                .add_modifier(Modifier::DIM),
        );

    let inner_area = preview_block.inner(area);
    frame.render_widget(preview_block, area);

    frame.render_widget(
        Block::default().style(Style::default().bg(theme.bg())),
        inner_area,
    );

    let preview_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // header + spacing
            Constraint::Length(1), // action bar
            Constraint::Length(3), // space
            Constraint::Min(5),    // test text
            Constraint::Length(6), // command bar
        ])
        .split(inner_area);

    // ------ TITLE ------
    let header_area = preview_layout[0];
    let header_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // p-top
            Constraint::Length(1), // title
            Constraint::Min(0),    // space
        ])
        .split(header_area);

    let title_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2), // p-left
            Constraint::Min(10),   // title
            Constraint::Min(0),    // space
        ])
        .split(header_layout[1]);

    let header = Paragraph::new(theme.id.clone())
        .style(Style::default().fg(theme.highlight()))
        .alignment(Alignment::Left);

    frame.render_widget(header, title_layout[1]);

    // ------ ACTION BAR ------
    let action_bar_centered = apply_horizontal_centering(preview_layout[1], 80);
    let action_bar = Line::from(vec![
        Span::styled("! ", Style::default().fg(theme.highlight())),
        Span::styled("punctuation ", Style::default().fg(theme.highlight())),
        Span::styled("# ", Style::default().fg(theme.muted())),
        Span::styled("numbers ", Style::default().fg(theme.muted())),
        Span::styled("@ ", Style::default().fg(theme.muted())),
        Span::styled("symbols ", Style::default().fg(theme.muted())),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(Paragraph::new(action_bar), action_bar_centered);

    // ------ TYPING AREA ------
    let typing_area_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // space + lang
            Constraint::Min(3),    // text
        ])
        .split(preview_layout[3]);

    let lang_centered = apply_horizontal_centering(typing_area_layout[0], 80);
    let lang_indicator = Paragraph::new("english")
        .style(
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(lang_indicator, lang_centered);

    let typing_area_centered = apply_horizontal_centering(typing_area_layout[1], 80);
    let sample_text = vec![
        Line::from(vec![
            Span::styled("terminal ", Style::default().fg(theme.success())),
            Span::styled("typing ", Style::default().fg(theme.error())),
            Span::styled(
                "at its finest",
                Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
            ),
        ]),
        Line::from(vec![Span::styled(
            "brought to you by termitype!",
            Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
        )]),
    ];

    frame.render_widget(
        Paragraph::new(sample_text).alignment(Alignment::Center),
        typing_area_centered,
    );

    // ------ COMMAND BAR ------
    let bottom_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // space
            Constraint::Length(1), // space
            Constraint::Length(1), // command bar
            Constraint::Length(2), // space
        ])
        .split(preview_layout[4]);

    let command_bar_centered = apply_horizontal_centering(bottom_layout[2], 80);
    let command_bar = vec![
        Line::from(vec![
            Span::styled(
                "tab",
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" + ", Style::default().fg(theme.muted())),
            Span::styled(
                "enter",
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - restart test", Style::default().fg(theme.muted())),
            // Span::styled(
            //     "esc",
            //     Style::default()
            //         .fg(theme.highlight())
            //         .add_modifier(Modifier::BOLD),
            // ),
            // Span::styled(" - menu", Style::default().fg(theme.muted())),
        ]),
        Line::from(vec![
            Span::styled(
                "ctrl",
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" + ", Style::default().fg(theme.muted())),
            Span::styled(
                "c",
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(theme.muted())),
            Span::styled(
                "ctrl",
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" + ", Style::default().fg(theme.muted())),
            Span::styled(
                "z",
                Style::default()
                    .fg(theme.highlight())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - to quit", Style::default().fg(theme.muted())),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(command_bar).alignment(Alignment::Center),
        command_bar_centered,
    );
}

fn render_ascii_art_preview(frame: &mut Frame, termi: &Termi, area: Rect) {
    let theme = termi.current_theme();
    frame.render_widget(Clear, area);

    let art_name = termi
        .preview_ascii_art
        .as_deref()
        .or(termi.config.ascii.as_deref())
        .unwrap_or(ascii::DEFAULT_ASCII_ART_NAME);

    let ascii_art = ascii::get_ascii_art_by_name(art_name);

    let title = format!(" Preview: {} ", art_name);
    let preview_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Left)
        .bg(theme.bg())
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(
            Style::default()
                .fg(theme.border())
                .add_modifier(Modifier::DIM),
        );

    let content_area = preview_block.inner(area);
    frame.render_widget(preview_block, area);

    // log_debug!("({},{})", content_area.width, content_area.height);
    if content_area.width == 0 || content_area.height == 0 {
        return;
    }

    if let Some(art) = ascii_art {
        let art_text = Text::from(art)
            .style(Style::default().fg(theme.fg()))
            .alignment(Alignment::Left);

        let width = art_text.width() as u16;
        let height = art_text.height() as u16;

        if width > 0 && height > 0 {
            let centered_rect = Rect {
                x: content_area
                    .x
                    .saturating_add(content_area.width.saturating_sub(width) / 2),
                y: content_area
                    .y
                    .saturating_add(content_area.height.saturating_sub(height) / 2),
                width: width.min(content_area.width),
                height: height.min(content_area.height),
            };

            let paragraph = Paragraph::new(art_text).wrap(Wrap { trim: false });
            frame.render_widget(paragraph, centered_rect);
        } else {
            let placeholder_text = Text::from("[empty art]")
                .style(Style::default().fg(theme.muted()))
                .alignment(Alignment::Center);
            let paragraph = Paragraph::new(placeholder_text)
                .block(Block::default().padding(Padding::uniform(1)));
            frame.render_widget(paragraph, content_area);
        }
    } else {
        let not_found_text = Text::from(format!("Art '{}' not found", art_name))
            .style(Style::default().fg(theme.error()))
            .alignment(Alignment::Center);
        let paragraph =
            Paragraph::new(not_found_text).block(Block::default().padding(Padding::uniform(1)));
        frame.render_widget(paragraph, content_area);
    }
}

pub fn render_results_screen(frame: &mut Frame, termi: &mut Termi, area: Rect, is_small: bool) {
    let results_style = termi.config.resolve_results_style();

    match results_style {
        crate::config::ResultsStyle::Neofetch => {
            render_neofetch_results_screen(frame, termi, area, is_small)
        }
        crate::config::ResultsStyle::Graph => {
            render_graph_results_screen(frame, termi, area, is_small)
        }
    }
}

fn render_neofetch_results_screen(
    frame: &mut Frame,
    termi: &mut Termi,
    area: Rect,
    is_small: bool,
) {
    let tracker = &termi.tracker;
    let theme = termi.current_theme();
    let config = &termi.config;
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let hostname = "termitype";

    let header = format!("{}@{}", username, hostname);
    let separator = "─".repeat(header.chars().count());

    let is_monochromatic = termi.config.monocrhomatic_results;

    let color_succes = if is_monochromatic {
        theme.highlight()
    } else {
        theme.success()
    };
    let color_fg = if is_monochromatic {
        theme.muted()
    } else {
        theme.fg()
    };

    let color_warning = if is_monochromatic {
        theme.highlight()
    } else {
        theme.warning()
    };

    let current_art_name = termi
        .preview_ascii_art
        .as_deref() // grab the preview if any first
        .or(config.ascii.as_deref())
        .unwrap_or(ascii::DEFAULT_ASCII_ART_NAME); // last resort

    let current_ascii_art = ascii::get_ascii_art_by_name(current_art_name).unwrap_or("");

    // TODO: improve the coloring of this to match fastfetch. They have the @ in a different color.
    let mut stats_lines = vec![
        Line::from(vec![
            Span::styled(username, Style::default().fg(color_succes)),
            Span::styled("@", Style::default().fg(color_fg)),
            Span::styled(hostname, Style::default().fg(color_succes)),
        ]),
        Line::from(Span::styled(separator, Style::default().fg(color_fg))),
    ];

    let add_stat = |label: &str, value: String| -> Line {
        Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(color_warning)),
            Span::styled(value, Style::default().fg(color_fg)),
        ])
    };

    let errors = tracker
        .total_keystrokes
        .saturating_sub(tracker.correct_keystrokes);
    let (min_wpm, max_wpm) = tracker
        .wpm_samples
        .iter()
        .fold((u32::MAX, 0), |(min, max), &val| {
            (min.min(val), max.max(val))
        });
    let wpm_range_str = if min_wpm == u32::MAX {
        None
    } else {
        Some(format!("{}-{}", min_wpm, max_wpm))
    };

    let mode_str = match config.current_mode() {
        Mode::Time { duration } => format!("Time ({}s)", duration),
        Mode::Words { count } => format!("Words ({})", count),
    };

    stats_lines.push(add_stat("OS", "termitype".to_string()));
    stats_lines.push(add_stat("Version", VERSION.to_string()));
    stats_lines.push(add_stat("Mode", mode_str));
    stats_lines.push(add_stat(
        "Lang",
        config.language.clone().unwrap_or_default(),
    ));
    stats_lines.push(add_stat("WPM", format!("{:.0}", tracker.wpm)));
    stats_lines.push(add_stat("Raw WPM", format!("{:.0}", tracker.raw_wpm)));

    if let Some(time) = tracker.completion_time {
        stats_lines.push(add_stat("Duration", format!("{:.1}s", time)));
    } else if let Mode::Time { duration } = config.current_mode() {
        stats_lines.push(add_stat("Duration", format!("{}s", duration)));
    }

    stats_lines.push(add_stat("Accuracy", format!("{}%", tracker.accuracy)));
    stats_lines.push(add_stat(
        "Consistency",
        format!("{:.0}%", tracker.calculate_consistency()),
    ));
    stats_lines.push(add_stat(
        "Keystrokes",
        format!("{} ({})", tracker.total_keystrokes, tracker.accuracy),
    ));
    stats_lines.push(add_stat(
        "Correct",
        format!("{}", tracker.correct_keystrokes),
    ));
    stats_lines.push(add_stat("Errors", format!("{}", errors)));
    stats_lines.push(add_stat(
        "Backspaces",
        format!("{}", tracker.backspace_count),
    ));
    if let Some(wpm_range) = wpm_range_str {
        stats_lines.push(add_stat("WPM Range", wpm_range));
    }
    stats_lines.push(Line::from(""));

    // only show color blocks if we show the ascii art
    if !is_small {
        let mut color_blocks = vec![];
        for color in [
            theme.cursor_text(),
            theme.error(),
            theme.success(),
            theme.warning(),
            theme.info(),
            theme.accent(),
            theme.highlight(),
            theme.fg(),
        ] {
            color_blocks.push(Span::styled("██", Style::default().fg(color)));
        }
        stats_lines.push(Line::from(color_blocks));
    }

    let art_text = Text::from(current_ascii_art).style(Style::default().fg(color_warning));
    let art_height = art_text.height() as u16;
    let art_width = art_text.width() as u16;

    let stats_text = Text::from(stats_lines);
    let stats_height = stats_text.height() as u16;
    let stats_width = stats_text
        .lines
        .iter()
        .map(|line| line.width())
        .max()
        .unwrap_or(0) as u16;

    if is_small {
        let centered_rect = Rect {
            x: area.x + area.width.saturating_sub(stats_width) / 2,
            y: area.y + area.height.saturating_sub(stats_height) / 2,
            width: stats_width.min(area.width),
            height: stats_height.min(area.height),
        };
        frame.render_widget(Paragraph::new(stats_text), centered_rect);
    } else {
        let total_needed_width = art_width + stats_width + 5;
        let horizontal_padding = area.width.saturating_sub(total_needed_width) / 2;
        let vertical_padding = area.height.saturating_sub(stats_height.max(art_height)) / 2;

        let layout_area = Rect {
            x: area.x + horizontal_padding,
            y: area.y + vertical_padding,
            width: total_needed_width.min(area.width),
            height: stats_height.max(art_height).min(area.height),
        };

        if layout_area.right() > area.right() || layout_area.width == 0 {
            let centered_rect = Rect {
                x: area.x + area.width.saturating_sub(stats_width) / 2,
                y: area.y + area.height.saturating_sub(stats_height) / 2,
                width: stats_width.min(area.width),
                height: stats_height.min(area.height),
            };
            frame.render_widget(Paragraph::new(stats_text), centered_rect);
        } else {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(art_width),
                        Constraint::Length(5), // padding
                        Constraint::Length(stats_width),
                    ]
                    .as_ref(),
                )
                .split(layout_area);

            let art_area = chunks[0];
            let stats_area = chunks[2];

            // if the art is shorter than stats then center it
            let art_y_padding = art_area.height.saturating_sub(art_height) / 2;
            let centered_art_area = Rect {
                y: art_area.y + art_y_padding,
                height: art_height.min(art_area.height),
                ..art_area
            };

            // do the same as above for the stats
            let stats_y_padding = stats_area.height.saturating_sub(stats_height) / 2;
            let centered_stats_area = Rect {
                y: stats_area.y + stats_y_padding,
                height: stats_height.min(stats_area.height),
                ..stats_area
            };

            frame.render_widget(Paragraph::new(art_text), centered_art_area);
            frame.render_widget(Paragraph::new(stats_text), centered_stats_area);
        }
    }

    // only show footer if we have enough space
    if !is_small {
        let footer_line = create_results_footer_text(theme);

        let restart_height: u16 = 4;
        if area.height > restart_height {
            let restart_area = Rect {
                x: area.x,
                y: area.bottom().saturating_sub(restart_height),
                width: area.width,
                height: restart_height,
            };
            frame.render_widget(Paragraph::new(footer_line), restart_area);
        }
    }
}

fn render_graph_results_screen(frame: &mut Frame, termi: &mut Termi, area: Rect, is_small: bool) {
    let tracker = &termi.tracker;
    let theme = termi.current_theme();
    let _config = &termi.config;

    let is_monochromatic = termi.config.monocrhomatic_results;

    let color_success = if is_monochromatic {
        theme.highlight()
    } else {
        theme.success()
    };
    let color_fg = if is_monochromatic {
        theme.muted()
    } else {
        theme.fg()
    };
    let color_muted = if is_monochromatic {
        theme.fg()
    } else {
        theme.muted()
    };

    // folds the results: graph top, stats mid, footer bottom
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if is_small {
            [Constraint::Percentage(100)].as_ref()
        } else {
            [
                Constraint::Percentage(60), // graph
                Constraint::Percentage(38), // stats
                Constraint::Length(2),      // footer
            ]
            .as_ref()
        })
        .split(area);

    let graph_area = main_layout[0];
    let stats_area = if is_small { area } else { main_layout[1] };
    let footer_area = if is_small { None } else { Some(main_layout[2]) };

    // if there's space show the graph
    if !is_small && !tracker.wpm_samples.is_empty() {
        render_wpm_graph(frame, termi, graph_area);
    }

    render_graph_stats(
        frame,
        termi,
        stats_area,
        is_small,
        color_success,
        color_fg,
        color_muted,
    );

    if let Some(footer_area) = footer_area {
        let footer_line = create_results_footer_text(theme);
        frame.render_widget(Paragraph::new(footer_line), footer_area);
    }
}

fn render_wpm_graph(frame: &mut Frame, termi: &Termi, area: Rect) {
    let theme = termi.current_theme();

    if termi.tracker.wpm_samples.is_empty() {
        return;
    }

    let wpm_data: Vec<(f64, f64)> = termi
        .tracker
        .wpm_samples
        .iter()
        .enumerate()
        .map(|(i, &wpm)| ((i + 1) as f64, wpm as f64))
        .collect();

    // clear with bg
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.bg())),
        area,
    );

    let max_wpm = termi.tracker.wpm_samples.iter().max().copied().unwrap_or(0) as f64;
    let min_wpm = termi.tracker.wpm_samples.iter().min().copied().unwrap_or(0) as f64;
    let max_time = if let Mode::Time { duration } = termi.config.current_mode() {
        (duration as f64).floor()
    } else {
        termi.tracker.wpm_samples.len() as f64
    };

    // padding
    let y_max = (max_wpm * 1.1).max(max_wpm + 10.0);
    let y_min = (min_wpm * 0.9).min(min_wpm - 10.0).max(0.0);

    let dataset = Dataset::default()
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.success()))
        .data(&wpm_data);

    let chart = Chart::new(vec![dataset])
        .style(Style::default().bg(theme.bg()))
        .block(
            Block::default()
                .title(" WPM Over Time ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(theme.border())
                        .add_modifier(Modifier::DIM),
                )
                .style(Style::default().bg(theme.bg())),
        )
        .x_axis(
            Axis::default()
                .title("Time (seconds)")
                .style(Style::default().fg(theme.muted()))
                .bounds([1.0, max_time])
                .labels(vec![
                    Span::styled("1", Style::default().fg(theme.muted())),
                    Span::styled(
                        format!("{:.0}", (1.0 + max_time) / 2.0),
                        Style::default().fg(theme.muted()),
                    ),
                    Span::styled(
                        format!("{:.0}", max_time),
                        Style::default().fg(theme.muted()),
                    ),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("WPM")
                .style(Style::default().fg(theme.muted()))
                .bounds([y_min, y_max])
                .labels(vec![
                    Span::styled(format!("{:.0}", y_min), Style::default().fg(theme.muted())),
                    Span::styled(
                        format!("{:.0}", (y_min + y_max) / 2.0),
                        Style::default().fg(theme.muted()),
                    ),
                    Span::styled(format!("{:.0}", y_max), Style::default().fg(theme.muted())),
                ]),
        );

    frame.render_widget(chart, area);
}

fn render_graph_stats(
    frame: &mut Frame,
    termi: &crate::termi::Termi,
    area: Rect,
    is_small: bool,
    color_success: ratatui::style::Color,
    color_fg: ratatui::style::Color,
    color_muted: ratatui::style::Color,
) {
    let tracker = &termi.tracker;
    let config = &termi.config;
    let theme = termi.current_theme();

    // smart folds based on available space
    let layout = if is_small {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area)
    };

    let perf_stats_area = layout[0];
    let details_stats_area = layout[1];

    // === Performance Stats ===
    let mut perf_lines = vec![
        Line::from(vec![
            Span::styled("WPM: ", Style::default().fg(color_muted)),
            Span::styled(
                format!("{:.0}", tracker.wpm),
                Style::default()
                    .fg(color_success)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Accuracy: ", Style::default().fg(color_muted)),
            Span::styled(
                format!("{}%", tracker.accuracy),
                Style::default()
                    .fg(color_success)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Raw WPM: ", Style::default().fg(color_muted)),
            Span::styled(
                format!("{:.0}", tracker.raw_wpm),
                Style::default().fg(color_fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Consistency: ", Style::default().fg(color_muted)),
            Span::styled(
                format!("{:.0}%", tracker.calculate_consistency()),
                Style::default().fg(color_fg),
            ),
        ]),
    ];

    if let Some(time) = tracker.completion_time {
        perf_lines.push(Line::from(vec![
            Span::styled("Duration: ", Style::default().fg(color_muted)),
            Span::styled(format!("{:.1}s", time), Style::default().fg(color_fg)),
        ]));
    }

    let perf_block = Block::default()
        .title(" Performance ")
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(theme.border())
                .add_modifier(Modifier::DIM),
        )
        .style(Style::default().bg(theme.bg()));

    let perf_paragraph = Paragraph::new(perf_lines)
        .block(perf_block)
        .wrap(Wrap { trim: false });

    frame.render_widget(perf_paragraph, perf_stats_area);

    // === Details Stats ===
    let errors = tracker
        .total_keystrokes
        .saturating_sub(tracker.correct_keystrokes);
    let (min_wpm, max_wpm) = tracker
        .wpm_samples
        .iter()
        .fold((u32::MAX, 0), |(min, max), &val| {
            (min.min(val), max.max(val))
        });

    let mode_str = match config.current_mode() {
        crate::config::Mode::Time { duration } => format!("Time ({}s)", duration),
        crate::config::Mode::Words { count } => format!("Words ({})", count),
    };

    let duration = if let Mode::Time { duration } = config.current_mode() {
        format!("{}s", duration)
    } else {
        let time = tracker.completion_time.unwrap_or(0.0);
        format!("{:.1}s", time)
    };

    let mut details_lines = vec![
        Line::from(vec![
            Span::styled("Mode: ", Style::default().fg(color_muted)),
            Span::styled(mode_str, Style::default().fg(color_fg)),
        ]),
        Line::from(vec![
            Span::styled("Language: ", Style::default().fg(color_muted)),
            Span::styled(
                config.language.clone().unwrap_or_default(),
                Style::default().fg(color_fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", Style::default().fg(color_muted)),
            Span::styled(duration, Style::default().fg(color_fg)),
        ]),
        Line::from(vec![
            Span::styled("Keystrokes: ", Style::default().fg(color_muted)),
            Span::styled(
                format!("{}", tracker.total_keystrokes),
                Style::default().fg(color_fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Correct: ", Style::default().fg(color_muted)),
            Span::styled(
                format!("{}", tracker.correct_keystrokes),
                Style::default().fg(color_success),
            ),
        ]),
        Line::from(vec![
            Span::styled("Errors: ", Style::default().fg(color_muted)),
            Span::styled(
                format!("{}", errors),
                Style::default().fg(if errors > 0 {
                    termi.current_theme().error()
                } else {
                    color_fg
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Backspaces: ", Style::default().fg(color_muted)),
            Span::styled(
                format!("{}", tracker.backspace_count),
                Style::default().fg(color_fg),
            ),
        ]),
    ];

    if min_wpm != u32::MAX {
        details_lines.push(Line::from(vec![
            Span::styled("WPM Range: ", Style::default().fg(color_muted)),
            Span::styled(
                format!("{}-{}", min_wpm, max_wpm),
                Style::default().fg(color_fg),
            ),
        ]));
    }

    let details_block = Block::default()
        .title(" Details ")
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(theme.border())
                .add_modifier(Modifier::DIM),
        )
        .style(Style::default().bg(theme.bg()));

    let details_paragraph = Paragraph::new(details_lines)
        .block(details_block)
        .wrap(Wrap { trim: false });

    frame.render_widget(details_paragraph, details_stats_area);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_word_position_basic() {
        let text = "hello world ";
        let available_width = 20;
        let positions = calculate_word_positions(text, available_width);

        assert_eq!(positions.len(), 2, "Should have positions for two words ");
        assert_eq!(positions[0].start_index, 0, "First word starts at 0");
        assert_eq!(positions[0].line, 0, "First word on line 0");
        assert_eq!(positions[0].col, 0, "First word at column 0");

        assert_eq!(
            positions[1].start_index, 6,
            "Second word starts after \"hello \""
        );
        assert_eq!(positions[1].line, 0, "Second word on line 0");
        assert_eq!(positions[1].col, 6, "Second word after first word + space");
    }

    #[test]
    fn test_word_position_wrapping() {
        let text = "hello world wrap";
        let available_width = 8; // force wrap after "hello"
        let positions = calculate_word_positions(text, available_width);

        assert_eq!(positions[0].line, 0, "First word on line 0");
        assert_eq!(positions[1].line, 1, "Second word should wrap to line 1");
        assert_eq!(positions[1].col, 0, "Wrapped word starts at column 0");
        assert_eq!(positions[2].line, 2, "Third word on line 2");
    }

    #[test]
    fn test_cursor_positions() {
        let text = "hello world next";
        let available_width = 20;
        let positions = calculate_word_positions(text, available_width);

        let test_positions = vec![
            (0, 0, "Start of text"),
            (5, 0, "End of first word"),
            (6, 1, "Start of second word"),
            (11, 1, "End of second word"),
            (12, 2, "Start of third word"),
        ];

        for (cursor_pos, expected_word_idx, description) in test_positions {
            let current_pos = positions
                .iter()
                .rev()
                .find(|pos| cursor_pos >= pos.start_index)
                .unwrap();

            assert_eq!(
                positions
                    .iter()
                    .position(|p| p.start_index == current_pos.start_index)
                    .unwrap(),
                expected_word_idx,
                "{}",
                description
            );
        }
    }

    #[test]
    fn test_empty_text() {
        let text = "";
        let available_width = 10;
        let positions = calculate_word_positions(text, available_width);
        assert!(positions.is_empty(), "Empty text should have no positions");
    }
}
