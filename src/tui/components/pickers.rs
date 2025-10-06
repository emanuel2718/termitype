use crate::{
    actions::Action,
    app::App,
    menu::{Menu, MenuAction, MenuVisualizer},
    theme::Theme,
    tui::helpers::{horizontally_center, menu_items_padding},
    tui::layout::{
        picker_overlay_area, picker_should_show_visualizer, picker_should_use_full_area,
    },
    variants::{CursorVariant, PickerVariant},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
    Frame,
};

pub struct Picker;

impl Picker {
    pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
        const MAX_MENU_HEIGHT: u16 = 20;
        let variant = app.config.current_picker_variant();
        let max_height = MAX_MENU_HEIGHT.min(area.height.saturating_sub(6));
        let menu_height = max_height.saturating_sub(2); // borders
        app.menu.ui_height = menu_height as usize;

        match variant {
            PickerVariant::Telescope => render_telescope_picker(frame, app, theme, area),
            _ => render_telescope_picker(frame, app, theme, area),
        }
    }
}

// TODO: have some sort of Visualizer trait an builder
// TODO: refactor so that is easy to add new pickers and have them share logic
// TODO: cleanup bunch of magic numbers
// TODO: cleanup logic for splits

fn calculate_total_menu_area(overlay_area: Rect, screen_area: Rect, has_visualizer: bool) -> Rect {
    if !has_visualizer {
        // single panel
        return overlay_area;
    }

    let bar_height = 3u16;
    let extended_height =
        (overlay_area.height + bar_height).min(screen_area.height.saturating_sub(overlay_area.y));

    Rect {
        x: overlay_area.x,
        y: overlay_area.y,
        width: overlay_area.width,
        height: extended_height,
    }
}

pub fn render_telescope_picker(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let menu = &app.menu;

    if let Some(current_menu) = menu.current_menu() {
        let items = menu.current_items();
        let has_no_items = items.is_empty();

        let title = current_menu.title.clone();
        let overlay_area = picker_overlay_area(area);

        // don't show visualizer on small height. Doesnt look good
        let has_visualizer = current_menu.has_visualizer() && picker_should_show_visualizer(area);
        let total_menu_area = calculate_total_menu_area(overlay_area, area, has_visualizer);

        frame.render_widget(Clear, total_menu_area);

        // has visualizer: two panels
        // no visualizer: single panel
        let is_low_width = overlay_area.width <= 75;

        let (items_area, visualizer_area, bottom_bar_anchor): (Rect, Rect, Rect) = if has_visualizer
        {
            if is_low_width {
                let bar_height: u16 = 3;
                let half_top_len = overlay_area.height / 2;

                let rows = if menu.is_searching() {
                    let top_len = half_top_len.saturating_sub(bar_height);
                    Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(top_len),
                            Constraint::Length(bar_height),
                            Constraint::Min(0),
                        ])
                        .split(total_menu_area)
                } else {
                    Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(half_top_len), Constraint::Min(0)])
                        .split(total_menu_area)
                };

                let top_panel = rows[0];
                let bottom_panel_base = rows[rows.len() - 1];

                // items (top)
                let top_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .title(format!(" {} ", title))
                    .title_alignment(Alignment::Center)
                    .title_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .style(Style::default().bg(theme.bg()));
                let top_inner = top_block.inner(top_panel);
                frame.render_widget(top_block, top_panel);

                // visualizer (bot)
                let bottom_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .style(Style::default().bg(theme.bg()));
                let bottom_inner = bottom_block.inner(bottom_panel_base);
                frame.render_widget(bottom_block, bottom_panel_base);

                (top_inner, bottom_inner, top_panel)
            } else {
                let columns = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(45), // left
                        Constraint::Length(0),
                        Constraint::Percentage(55), // right
                    ])
                    .split(overlay_area);

                let left_panel = columns[0];
                let right_panel_base = columns[2];

                // items (left)
                let left_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .title(format!(" {} ", title))
                    .title_alignment(Alignment::Center)
                    .title_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .style(Style::default().bg(theme.bg()));
                let left_inner = left_block.inner(left_panel);
                frame.render_widget(left_block, left_panel);

                let bar_height: u16 = 3; // TODO: refactor magic number for menu_bar height
                let right_panel = Rect {
                    x: right_panel_base.x,
                    y: overlay_area.y,
                    width: right_panel_base.width,
                    height: (overlay_area.height + bar_height)
                        .min(area.height.saturating_sub(overlay_area.y)),
                };

                // right (visualizer)
                let right_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(theme.border())
                            .bg(theme.bg())
                            .remove_modifier(Modifier::all()),
                    )
                    .style(Style::default().bg(theme.bg()));
                let right_inner = right_block.inner(right_panel);
                frame.render_widget(right_block, right_panel);

                (left_inner, right_inner, left_panel)
            }
        } else {
            // single panel (no visualizer)
            let menu_block = Block::default()
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(theme.border())
                        .bg(theme.bg())
                        .remove_modifier(Modifier::all()),
                )
                .title(format!(" {} ", title))
                .title_alignment(Alignment::Center)
                .title_style(
                    Style::default()
                        .fg(theme.border())
                        .bg(theme.bg())
                        .remove_modifier(Modifier::all()),
                )
                .style(Style::default().bg(theme.bg()));
            let inner_area = menu_block.inner(overlay_area);
            frame.render_widget(menu_block, overlay_area);
            (inner_area, Rect::default(), overlay_area)
        };

        let items_area_height = items_area.height as usize;
        let mut scroll_offset = current_menu.scroll_offset;

        let mut lines: Vec<Line> = Vec::new();

        if has_no_items {
            let no_items_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
            lines.push(Line::from(Span::styled("No items found", no_items_style)));
        } else {
            let current_index = if menu.has_search_query() {
                if let Some(curr) = current_menu.current_item() {
                    items.iter().position(|&item| item == curr).unwrap_or(0)
                } else {
                    0
                }
            } else {
                current_menu.current_index
            };

            // ensure the current index is visible
            if current_index < scroll_offset {
                scroll_offset = current_index;
            } else if current_index >= scroll_offset + items_area_height {
                scroll_offset = current_index.saturating_sub(items_area_height.saturating_sub(1));
            }

            let visible_items = items
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(items_area_height)
                .collect::<Vec<_>>();

            for (idx, item) in visible_items {
                let is_selected = idx == current_index;
                let is_toggle = matches!(&item.action, MenuAction::Action(Action::Toggle(_)));
                let mut style = Style::default().fg(theme.fg());

                if is_selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                if item.is_disabled {
                    style = style.add_modifier(Modifier::DIM);
                }

                if is_toggle {
                    if let MenuAction::Action(Action::Toggle(setting)) = &item.action {
                        let enabled = app.config.is_enabled(setting.clone());
                        if !enabled {
                            style = style.add_modifier(Modifier::DIM);
                        } else {
                            style = style.fg(theme.success())
                        }
                    }
                }

                let indicator = if is_toggle {
                    if let MenuAction::Action(Action::Toggle(setting)) = &item.action {
                        let enabled = app.config.is_enabled(setting.clone());
                        if enabled {
                            "[x]"
                        } else {
                            "[ ]"
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    match is_selected {
                        true => ">",
                        false => " ",
                    }
                };

                let label = format!("{} {}", indicator, item.label());

                let spans = vec![Span::styled(label, style)];

                lines.push(Line::from(spans));
            }
        }

        let items_paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .block(Block::default().padding(menu_items_padding()));

        // render the items of the menu
        frame.render_widget(items_paragraph, items_area);

        // render visualization
        if let Some(menu) = menu.current_menu() {
            if menu.has_visualizer() && menu.visualizer.is_some() {
                let visualizer = menu.visualizer.as_ref().unwrap();
                render_menu_visualizer(frame, theme, visualizer, visualizer_area, app);
            }
        }

        // Bottom bar should span the width of the items area panel when a visualizer exists.
        // Otherwise, it spans the classic overlay width.
        // In low-width mode, hide the bar unless searching; when searching, place it between
        // the stacked panels (anchored to the top/items panel).
        // In full area mode, always show the bar since space is reserved for it, except for vertical splits.
        if !is_low_width
            || menu.is_searching()
            || picker_should_use_full_area(area) && !has_visualizer
        {
            render_menu_bottom_bar(frame, bottom_bar_anchor, area, theme, menu, has_visualizer);
        }
    }
}

fn render_menu_bottom_bar(
    frame: &mut Frame,
    overlay_area: Rect,
    area: Rect,
    theme: &Theme,
    menu: &Menu,
    has_visualizer: bool,
) {
    let bar_height = 3u16;
    let bar_area = Rect {
        x: overlay_area.x,
        y: overlay_area.y + overlay_area.height,
        width: overlay_area.width,
        height: bar_height,
    };
    if bar_area.y + bar_area.height <= area.y + area.height {
        frame.render_widget(Clear, bar_area);
        let border_block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(theme.border())
                    .bg(theme.bg())
                    .remove_modifier(Modifier::all()),
            )
            .style(Style::default().bg(theme.bg()))
            .padding(Padding {
                left: 1,
                right: 1,
                top: 0,
                bottom: 0,
            });
        let inner = border_block.inner(bar_area);
        frame.render_widget(border_block, bar_area);

        // left 70%
        // right 30%
        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(inner);

        let left_area = sections[0];
        let right_area = sections[1];

        let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);

        // left
        if menu.is_searching() {
            let highlight_style = Style::default().fg(theme.highlight());
            let left = Paragraph::new(vec![Line::from(vec![
                Span::styled(">", highlight_style),
                Span::styled(format!(" {}", menu.search_query()), dim_style),
            ])]);
            frame.render_widget(left, left_area);

            // Hide cursor for cursor menu when visualizer is not shown
            let is_cursor_menu_without_visualizer = if let Some(current_menu) = menu.current_menu()
            {
                matches!(
                    current_menu.visualizer,
                    Some(MenuVisualizer::CursorVisualizer)
                ) && !has_visualizer
            } else {
                false
            };

            if !is_cursor_menu_without_visualizer {
                let base_offset: u16 = 2; // "> "
                let qlen = menu.search_query().chars().count() as u16;
                let mut x = left_area.x + base_offset + qlen;
                if x >= left_area.x + left_area.width.saturating_sub(1) {
                    x = left_area.x + left_area.width.saturating_sub(2);
                }
                let y = left_area.y;
                frame.set_cursor_position(Position { x, y });
            }
        } else {
            let left = Paragraph::new("> ")
                .style(dim_style)
                .alignment(Alignment::Left);
            frame.render_widget(left, left_area);
        }

        // right <m>/<N>
        let (m, n) = if let Some(current_menu) = menu.current_menu() {
            let items = menu.current_items();
            let n = items.len();
            let current_index = if menu.has_search_query() {
                if let Some(curr) = current_menu.current_item() {
                    items.iter().position(|&item| item == curr).unwrap_or(0)
                } else {
                    0
                }
            } else {
                current_menu.current_index
            };
            (current_index.saturating_add(1), n)
        } else {
            (0, 0)
        };

        let right_text = Paragraph::new(format!("{}/{}", if n == 0 { 0 } else { m }, n))
            .style(dim_style)
            .alignment(Alignment::Right);
        frame.render_widget(right_text, right_area);
    }
}

fn render_menu_visualizer(
    frame: &mut Frame,
    theme: &Theme,
    vis: &MenuVisualizer,
    area: Rect,
    app: &App,
) {
    match vis {
        MenuVisualizer::ThemeVisualizer => render_theme_visualizer(frame, theme, area),
        MenuVisualizer::CursorVisualizer => render_cursor_visualizer(frame, theme, area, app),
    }
}

fn render_theme_visualizer(frame: &mut Frame, theme: &Theme, area: Rect) {
    let visualizer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // header + spacing
            Constraint::Length(1), // action bar
            Constraint::Length(8), // space
            Constraint::Min(5),    // typing area
            Constraint::Length(4), // bottom section
        ])
        .split(area);

    render_theme_header_visualizer(frame, theme, visualizer_layout[0]);
    render_theme_mode_bar_visualizer(frame, theme, visualizer_layout[1]);
    render_theme_typing_area_visualizer(frame, theme, visualizer_layout[3]);
    render_theme_cmd_bar_visualizer(frame, theme, visualizer_layout[4]);
}

fn render_theme_header_visualizer(frame: &mut Frame, theme: &Theme, area: Rect) {
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

fn render_theme_mode_bar_visualizer(frame: &mut Frame, theme: &Theme, area: Rect) {
    let centered = horizontally_center(area, 80);
    let highlight_style = Style::default().fg(theme.highlight());
    let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let mode_bar = Line::from(vec![
        Span::styled("! ", highlight_style),
        Span::styled("punctuation ", highlight_style),
        Span::styled("# ", dim_style),
        Span::styled("numbers ", dim_style),
    ]);
    let mode_bar = Paragraph::new(mode_bar).alignment(Alignment::Center);
    frame.render_widget(mode_bar, centered);
}

fn render_theme_typing_area_visualizer(frame: &mut Frame, theme: &Theme, area: Rect) {
    let typing_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // space + lang
            Constraint::Min(3),    // text
        ])
        .split(area);

    let lang_centered = horizontally_center(typing_layout[0], 80);
    let lang_indicator = Paragraph::new("english")
        .style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
        .alignment(Alignment::Center);
    frame.render_widget(lang_indicator, lang_centered);

    let typing_centered = horizontally_center(typing_layout[1], 80);
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
    let typing_area = Paragraph::new(sample_text).alignment(Alignment::Center);
    frame.render_widget(typing_area, typing_centered);
}

fn render_theme_cmd_bar_visualizer(frame: &mut Frame, theme: &Theme, area: Rect) {
    let bottom_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // space
            Constraint::Length(1), // space
            Constraint::Length(2), // cmd bar
            Constraint::Length(1), // space
        ])
        .split(area);

    let command_bar_centered = horizontally_center(bottom_layout[2], 80);
    let highlight_style = Style::default().fg(theme.highlight());
    let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let command_bar = vec![
        Line::from(vec![
            Span::styled("tab", highlight_style),
            Span::styled(" + ", dim_style),
            Span::styled("enter", highlight_style),
            Span::styled(" - restart test", dim_style),
        ]),
        Line::from(vec![
            Span::styled("ctrl", highlight_style),
            Span::styled(" + ", dim_style),
            Span::styled("c", highlight_style),
            Span::styled(" or ", dim_style),
            Span::styled("ctrl", highlight_style),
            Span::styled(" + ", dim_style),
            Span::styled("z", highlight_style),
            Span::styled(" - to quit", dim_style),
        ]),
    ];

    let cmd_bar = Paragraph::new(command_bar).alignment(Alignment::Center);
    frame.render_widget(cmd_bar, command_bar_centered);
}

fn render_cursor_visualizer(frame: &mut Frame, theme: &Theme, area: Rect, app: &App) {
    use crate::{actions::Action, menu::MenuAction, variants::CursorVariant};

    let cursor_variant = if let Some(item) = app.menu.current_item() {
        match &item.action {
            MenuAction::Action(Action::SetCursor(variant)) => *variant,
            _ => CursorVariant::default(),
        }
    } else {
        CursorVariant::default()
    };

    let visualizer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // variant name
            Constraint::Min(5),    // cursor preview
        ])
        .split(area);

    render_cursor_variant_header(frame, theme, visualizer_layout[0], &cursor_variant);

    render_cursor_preview_text(frame, theme, visualizer_layout[1], &cursor_variant);
}

fn render_cursor_variant_header(
    frame: &mut Frame,
    theme: &Theme,
    area: Rect,
    variant: &CursorVariant,
) {
    let header_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // padding top
            Constraint::Length(1), // variant
            Constraint::Min(0),    // space
        ])
        .split(area);

    let title_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2), // padding left
            Constraint::Min(10),   // title
            Constraint::Min(0),    // space
        ])
        .split(header_layout[1]);

    let variant_text = format!("Variant: {}", variant.label());
    let header = Paragraph::new(variant_text)
        .style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
        .alignment(Alignment::Left);
    frame.render_widget(header, title_layout[1]);
}

fn render_cursor_preview_text(
    frame: &mut Frame,
    theme: &Theme,
    area: Rect,
    _variant: &CursorVariant,
) {
    let vertical_padding = area.height.saturating_sub(3) / 2;
    let preview_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding), // top padding
            Constraint::Length(2),                // preview text
            Constraint::Min(0),                   // bottom space
        ])
        .split(area);

    let text_area = preview_layout[1];

    let preview_line = create_cursor_preview_line(theme);

    let preview = Paragraph::new(preview_line)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(preview, text_area);

    let full_text = "terminal typing at its finest";
    let cursor_position = "terminal typing".len();

    let full_text_width = full_text.len() as u16;
    let centered_x = (text_area.width.saturating_sub(full_text_width)) / 2;
    let cursor_x = text_area.x + centered_x + cursor_position as u16;
    let cursor_y = text_area.y;

    frame.set_cursor_position(Position {
        x: cursor_x,
        y: cursor_y,
    });
}

fn create_cursor_preview_line(theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            "terminal ".to_string(),
            Style::default().fg(theme.success()),
        ),
        Span::styled("typing ".to_string(), Style::default().fg(theme.error())),
        Span::styled(
            "at its finest".to_string(),
            Style::default().fg(theme.fg()).add_modifier(Modifier::DIM),
        ),
    ])
}
