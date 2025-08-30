use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame,
};

use crate::{
    actions::{MenuContext, TermiClickAction},
    ascii,
    constants::MIN_THEME_PREVIEW_WIDTH,
    styles,
    termi::Termi,
    ui::helpers::{LayoutHelper, MenuHelpers, TermiUtils},
};

use super::elements::TermiElement;

pub struct MenuComponent;

impl MenuComponent {
    pub fn render(frame: &mut Frame, termi: &mut Termi, area: Rect) {
        let menu_state = &mut termi.menu;

        let is_theme_picker = menu_state.is_current_ctx(MenuContext::Theme);
        let is_help_menu = menu_state.is_current_ctx(MenuContext::Help);
        let is_about_menu = menu_state.is_current_ctx(MenuContext::About);
        let is_ascii_art_picker = menu_state.is_current_ctx(MenuContext::AsciiArt);

        let small_width = area.width <= MIN_THEME_PREVIEW_WIDTH;

        let picker_style = termi.config.resolve_picker_style();
        let base_rect = LayoutHelper::calculate_menu_area_from_parts(
            picker_style,
            is_theme_picker,
            is_help_menu,
            is_about_menu,
            is_ascii_art_picker,
            area,
        );

        //
        let (menu_area, preview_area) = if picker_style == styles::PickerStyle::Minimal {
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

        Self::render_menu_content(frame, termi, menu_area, small_width);

        // render the preview if needed
        if let Some(preview_area) = preview_area {
            if is_theme_picker {
                Self::render_theme_preview(frame, termi, preview_area);
            } else if is_ascii_art_picker {
                Self::render_ascii_art_preview(frame, termi, preview_area);
            } else if (is_help_menu || is_about_menu) && small_width {
                Self::render_description_preview(frame, termi, preview_area);
            }
        }
    }

    /// Create a "show menu" button. Used on size constrained situations.
    pub fn create_show_menu_button(termi: &Termi) -> Vec<TermiElement<'_>> {
        let theme = termi.current_theme();
        let menu_text = "≡ Show Menu";
        let padding = " ".repeat((menu_text.len() / 2).max(1));

        vec![
            TermiElement::spacer(padding.len()),
            TermiElement::new(
                menu_text,
                termi.menu.is_open(),
                Some(TermiClickAction::ToggleMenu),
            )
            .to_styled(theme),
            TermiElement::spacer(padding.len()),
        ]
    }

    /// Renders the main menu.
    fn render_menu_content(frame: &mut Frame, termi: &mut Termi, area: Rect, small_width: bool) {
        let picker_style = termi.config.resolve_picker_style();

        let hide_menu_footer = (small_width && !termi.menu.is_searching())
            || (picker_style == styles::PickerStyle::Minimal);
        let footer_len = if hide_menu_footer { 0 } else { 3 };
        let menu_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(footer_len)])
            .split(area);
        let content_area = menu_layout[0];
        let footer_area = menu_layout[1];

        termi.menu.ui_height = content_area.height.saturating_sub(2).max(1) as usize;

        let theme = termi.current_theme();
        if let Some(current_menu) = termi.menu.current_menu() {
            let max_visible = content_area.height.saturating_sub(3) as usize;
            let total_items = current_menu.items().len();

            let scroll_offset =
                Self::calculate_scroll_offset(total_items, max_visible, current_menu);
            let menu_title = current_menu.title.clone();

            let hide_description = (termi.menu.is_current_ctx(MenuContext::Help)
                || termi.menu.is_current_ctx(MenuContext::About))
                && small_width;
            let (list_items, total_items) =
                MenuHelpers::build_menu_items(termi, scroll_offset, max_visible, hide_description);

            let content_block = Block::default()
                .title(menu_title)
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border()))
                .style(Style::default().bg(theme.bg()));

            let menu_widget = List::new(list_items)
                .block(content_block)
                .style(Style::default().bg(theme.bg()));

            frame.render_widget(menu_widget, content_area);

            if total_items > max_visible {
                Self::render_scrollbar(frame, content_area, total_items, scroll_offset, theme);
            }
        }

        if !hide_menu_footer {
            Self::render_menu_footer(frame, termi, footer_area, theme);
        }
    }

    /// Calculates the scroll offset for the menu itemsjk.
    fn calculate_scroll_offset(
        total_items: usize,
        max_visible: usize,
        current_menu: &crate::menu::Menu,
    ) -> usize {
        if total_items <= max_visible || max_visible == 0 {
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
        }
    }

    /// Render scrollbar for menu overlay.
    fn render_scrollbar(
        frame: &mut Frame,
        content_area: Rect,
        total_items: usize,
        scroll_offset: usize,
        theme: &crate::theme::Theme,
    ) {
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

    fn render_menu_footer(
        frame: &mut Frame,
        termi: &Termi,
        footer_area: Rect,
        theme: &crate::theme::Theme,
    ) {
        let footer_text = TermiUtils::create_menu_footer_text(termi);
        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.bg()));

        let footer_widget = Paragraph::new(footer_text)
            .block(footer_block)
            .style(Style::default().bg(theme.bg()))
            .alignment(Alignment::Left);

        frame.render_widget(footer_widget, footer_area);
    }

    /// Render description preview. Used for the help/about menus.
    fn render_description_preview(frame: &mut Frame, termi: &Termi, area: Rect) {
        let theme = termi.current_theme();
        frame.render_widget(Clear, area);

        let preview_block = Block::default()
            .title(" Description ")
            .title_alignment(Alignment::Left)
            .style(Style::default().bg(theme.bg()))
            .borders(Borders::ALL)
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

    // TODO: maybe we might need to move and refactor this menu previews to a separate file
    //       and make it easier to create different kinds of menu previews

    // ==== THEME PREVIEW MENU RENDERING ====

    fn render_theme_preview(frame: &mut Frame, termi: &Termi, area: Rect) {
        let theme = termi.current_theme();
        frame.render_widget(Clear, area);

        let preview_block = Block::default()
            .style(Style::default().bg(theme.bg()))
            .borders(Borders::ALL)
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

        Self::render_theme_preview_content(frame, termi, inner_area);
    }

    fn render_theme_preview_content(frame: &mut Frame, termi: &Termi, area: Rect) {
        let theme = termi.current_theme();

        let preview_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // header + spacing
                Constraint::Length(1), // action bar
                Constraint::Length(3), // space
                Constraint::Min(5),    // test text
                Constraint::Length(6), // bottom section
            ])
            .split(area);

        // header
        Self::render_theme_header(frame, theme, preview_layout[0]);

        // action bar
        Self::render_theme_action_bar(frame, theme, preview_layout[1]);

        // typing area
        Self::render_theme_typing_area(frame, theme, preview_layout[3]);

        // bottom section
        Self::render_theme_command_bar(frame, theme, preview_layout[4]);
    }

    fn render_theme_header(frame: &mut Frame, theme: &crate::theme::Theme, area: Rect) {
        let header_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // p-top
                Constraint::Length(1), // title
                Constraint::Min(0),    // space
            ])
            .split(area);

        let title_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2), // p-left
                Constraint::Min(10),   // title
                Constraint::Min(0),    // space
            ])
            .split(header_layout[1]);

        let header = Paragraph::new(theme.id.as_ref())
            .style(Style::default().fg(theme.highlight()))
            .alignment(Alignment::Left);

        frame.render_widget(header, title_layout[1]);
    }

    fn render_theme_action_bar(frame: &mut Frame, theme: &crate::theme::Theme, area: Rect) {
        let action_bar_centered = LayoutHelper::apply_horizontal_centering(area, 80);
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
    }

    fn render_theme_typing_area(frame: &mut Frame, theme: &crate::theme::Theme, area: Rect) {
        let typing_area_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // space + lang
                Constraint::Min(3),    // text
            ])
            .split(area);

        let lang_centered = LayoutHelper::apply_horizontal_centering(typing_area_layout[0], 80);
        let lang_indicator = Paragraph::new("english")
            .style(
                Style::default()
                    .fg(theme.muted())
                    .add_modifier(Modifier::DIM),
            )
            .alignment(Alignment::Center);
        frame.render_widget(lang_indicator, lang_centered);

        let typing_area_centered =
            LayoutHelper::apply_horizontal_centering(typing_area_layout[1], 80);
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
    }

    fn render_theme_command_bar(frame: &mut Frame, theme: &crate::theme::Theme, area: Rect) {
        let bottom_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // space
                Constraint::Length(1), // space
                Constraint::Length(1), // command bar
                Constraint::Length(2), // space
            ])
            .split(area);

        let command_bar_centered = LayoutHelper::apply_horizontal_centering(bottom_layout[2], 80);
        let command_bar = vec![
            Line::from(vec![
                Span::styled(
                    "tab",
                    Style::default()
                        .fg(theme.highlight())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" + ", Style::default().fg(theme.muted())).add_modifier(Modifier::DIM),
                Span::styled(
                    "enter",
                    Style::default()
                        .fg(theme.highlight())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" - restart test", Style::default().fg(theme.muted()))
                    .add_modifier(Modifier::DIM),
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

    // ==== ASCII ART MENU PREVIEW RENDERING ====

    fn render_ascii_art_preview(frame: &mut Frame, termi: &Termi, area: Rect) {
        let theme = termi.current_theme();
        frame.render_widget(Clear, area);

        let art_name = termi
            .preview_ascii_art
            .as_deref()
            .or(termi.config.ascii.as_deref())
            .unwrap_or(ascii::DEFAULT_ASCII_ART_NAME);

        let ascii_art = ascii::get_ascii_art_by_name(art_name);

        let title = format!(" Preview: {art_name} ");
        let preview_block = Block::default()
            .title(title)
            .title_alignment(Alignment::Left)
            .style(Style::default().bg(theme.bg()))
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(theme.border())
                    .add_modifier(Modifier::DIM),
            );

        let content_area = preview_block.inner(area);
        frame.render_widget(preview_block, area);

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
            let not_found_text = Text::from(format!("Art '{art_name}' not found"))
                .style(Style::default().fg(theme.error()))
                .alignment(Alignment::Center);
            let paragraph =
                Paragraph::new(not_found_text).block(Block::default().padding(Padding::uniform(1)));
            frame.render_widget(paragraph, content_area);
        }
    }
}
