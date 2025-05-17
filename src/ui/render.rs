use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, List, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::{
    actions::TermiClickAction,
    config::Mode,
    constants::{
        ASCII_ART, MENU_HEIGHT, MIN_THEME_PREVIEW_WIDTH, MODAL_HEIGHT, MODAL_WIDTH,
        TYPING_AREA_WIDTH,
    },
    modal::InputModal,
    termi::Termi,
    tracker::Status,
    version::VERSION,
};

use super::{
    elements::{
        build_menu_items, create_action_bar, create_command_bar, create_footer, create_header,
        create_menu_footer_text, create_minimal_size_warning, create_mode_bar,
        create_show_menu_button, create_typing_area, TermiElement,
    },
    layout::create_layout,
    utils::{apply_horizontal_centering, calculate_word_positions, center_div},
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
                Padding::uniform(2)
            } else {
                Padding::symmetric(8, 2)
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

        let start_x = area.x + (area.width.saturating_sub(total_width)) / 2;

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

        f.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            area,
        );

        for (rect, action) in clickable_regions_to_add {
            cr.add(rect, action);
        }
    }
}

fn render_typing_area(frame: &mut Frame, termi: &Termi, area: Rect) {
    let available_width = area.width as usize;
    let line_count_from_config = termi.config.visible_lines as usize;
    let cursor_idx = termi.tracker.cursor_position;

    let effective_layout_width = available_width.min(TYPING_AREA_WIDTH as usize);
    let word_positions = calculate_word_positions(&termi.words, effective_layout_width);

    if word_positions.is_empty() {
        let (empty_text, _) =
            create_typing_area(termi, available_width, 0, line_count_from_config, &[]);

        let area_width = effective_layout_width as u16;
        let area_padding = area.width.saturating_sub(area_width) / 2;
        let render_area = Rect {
            x: area.x + area_padding,
            y: area.y,
            width: area_width,
            height: area.height,
        };
        frame.render_widget(Paragraph::new(empty_text), render_area);
        return;
    }

    let current_word_pos = word_positions
        .iter()
        .filter(|pos| cursor_idx >= pos.start_index)
        .next_back()
        .unwrap_or_else(|| word_positions.first().unwrap());

    let current_line = current_word_pos.line;

    let scroll_offset = if line_count_from_config <= 1 {
        current_line
    } else {
        let half_visible = line_count_from_config / 2;
        if current_line < half_visible {
            0
        } else {
            current_line.saturating_sub(half_visible)
        }
    };

    let (typing_text, _) = create_typing_area(
        termi,
        available_width,
        scroll_offset,
        line_count_from_config,
        &word_positions,
    );

    let text_height = typing_text.height();

    let area_width = effective_layout_width as u16;
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
    let menu_height = MENU_HEIGHT.min(frame.area().height);
    let estimated_menu_area = Rect {
        x: frame.area().x,
        y: frame.area().y,
        width: frame.area().width,
        height: menu_height,
    };
    let show_cursor = (termi.tracker.status == Status::Idle
        || termi.tracker.status == Status::Typing)
        && (!termi.menu.is_open() || !estimated_menu_area.intersects(render_area))
        && termi.modal.is_none()
        && !termi.menu.is_theme_menu();

    if show_cursor {
        let offset_x = cursor_idx.saturating_sub(current_word_pos.start_index);
        let cursor_x = render_area.x + current_word_pos.col.saturating_add(offset_x) as u16;
        let cursor_y = render_area.y + current_line.saturating_sub(scroll_offset) as u16;

        // boundary check.
        if cursor_x >= render_area.left()
            && cursor_x < render_area.right()
            && cursor_y >= render_area.top()
            && cursor_y < render_area.top() + (line_count_from_config.min(text_height) as u16)
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

    let small_width = area.width <= MIN_THEME_PREVIEW_WIDTH;
    let menu_height = if is_theme_picker && small_width {
        area.height
    } else {
        MENU_HEIGHT.min(area.height)
    };

    // split the menu in two folds if we are in the theme picker
    let (menu_area, preview_area) = if is_theme_picker && !small_width {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: menu_height,
            });
        (split[0], Some(split[1]))
    } else if is_theme_picker && small_width {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: menu_height,
            });
        (split[0], Some(split[1]))
    } else {
        (
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: menu_height,
            },
            None,
        )
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

        let (list_items, total_items) = build_menu_items(termi, scroll_offset, max_visible);

        let content_block = Block::default()
            .title(" Menu ")
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

        // theme preview
        if let Some(preview_area) = preview_area {
            render_theme_preview(frame, termi, preview_area);
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

pub fn render_results_screen(frame: &mut Frame, termi: &mut Termi, area: Rect, is_small: bool) {
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

    let color_muted = if is_monochromatic {
        theme.fg()
    } else {
        theme.muted()
    };

    let color_warning = if is_monochromatic {
        theme.highlight()
    } else {
        theme.warning()
    };

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
        stats_lines.push(add_stat("Time", format!("{:.1}s", time)));
    } else if let Mode::Time { duration } = config.current_mode() {
        stats_lines.push(add_stat("Time", format!("{}s", duration)));
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

    let art_text = Text::from(ASCII_ART).style(Style::default().fg(color_warning));
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

    let footer_line = Line::from(vec![
        Span::styled("[N]", Style::default().fg(color_warning)),
        Span::styled(
            "ew",
            Style::default().fg(color_muted).add_modifier(Modifier::DIM),
        ),
        Span::styled(" ", Style::default()),
        Span::styled("[R]", Style::default().fg(color_warning)),
        Span::styled(
            "edo",
            Style::default().fg(color_muted).add_modifier(Modifier::DIM),
        ),
        Span::styled(" ", Style::default()),
        Span::styled("[Q]", Style::default().fg(color_warning)),
        Span::styled(
            "uit",
            Style::default().fg(color_muted).add_modifier(Modifier::DIM),
        ),
    ])
    .alignment(Alignment::Center);

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
