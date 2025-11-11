use crate::{
    actions::Action,
    app::App,
    log_debug,
    menu::{Menu, MenuAction, MenuItem},
    theme::{self, Theme, ThemeColor},
    tui::{helpers::menu_items_padding, layout::picker_overlay_area},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
    Frame,
};

fn theme_preview_colors() -> [ThemeColor; 3] {
    [
        ThemeColor::Highlight,
        ThemeColor::Warning,
        ThemeColor::Success,
    ]
}

/// Calculate the width of spans by summing character counts
fn calculate_spans_width(spans: &[Span]) -> usize {
    spans.iter().map(|s| s.content.chars().count()).sum()
}

pub fn render_command_palette(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let menu = &app.menu;

    if let Some(current_menu) = menu.current_menu() {
        let items = menu.current_items();
        let has_no_items = items.is_empty();

        let overlay_area = picker_overlay_area(area);

        frame.render_widget(Clear, overlay_area);

        let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
        let bg_style = Style::default().bg(theme.bg());
        let menu_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(dim_style)
            .title(format!(" {} ", current_menu.title))
            .title_alignment(Alignment::Center)
            .title_style(dim_style)
            .style(bg_style)
            .padding(Padding {
                left: 1,
                right: 0,
                top: 1,
                bottom: 0,
            });
        let inner_area = menu_block.inner(overlay_area);
        frame.render_widget(menu_block, overlay_area);

        let search_bar_height = 1u16;
        let spacer_height = 1u16;
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(search_bar_height),
                Constraint::Length(spacer_height),
                Constraint::Min(0),
            ])
            .split(inner_area);

        let search_bar_area = rows[0];
        let items_area = rows[2];

        render_search_bar(frame, search_bar_area, theme, menu);

        let items_area_height = items_area.height as usize;
        let mut scroll_offset = current_menu.scroll_offset;

        let mut lines: Vec<Line> = Vec::with_capacity(items_area_height);
        log_debug!("Lines len: {items_area_height}");

        if has_no_items {
            let no_items_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
            lines.push(Line::from(Span::styled("No items found", no_items_style)));
        } else {
            let current_index = menu.current_index().unwrap_or(0);

            if current_index < scroll_offset {
                scroll_offset = current_index;
            } else if current_index >= scroll_offset + items_area_height {
                scroll_offset = current_index.saturating_sub(items_area_height.saturating_sub(1));
            }

            let padding = menu_items_padding();
            let available_width = items_area
                .width
                .saturating_sub(padding.left)
                .saturating_sub(padding.right) as usize;

            for (idx, item) in items
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(items_area_height)
            {
                let is_selected = idx == current_index;
                let base_style = Style::default().fg(theme.fg());
                let selection_style = Style::default().fg(theme.bg()).bg(theme.fg());

                let label = item.get_description();
                // tag + label + preview + padding
                let mut spans = Vec::with_capacity(if item.tag.is_some() { 4 } else { 3 });
                spans.extend(create_item_spans(
                    item,
                    is_selected,
                    base_style,
                    theme,
                    label,
                ));

                let text_width = calculate_spans_width(&spans);

                // if we have a theme item we must render the theme dots
                let (preview_spans, preview_width) = if matches!(item.tag.as_deref(), Some("theme"))
                {
                    extract_theme_name(item)
                        .and_then(|name| theme::theme_manager().get_theme(name).ok())
                        .map(|preview_theme| render_theme_dots(&preview_theme, None))
                        .unwrap_or((vec![], 0))
                } else {
                    (vec![], 0)
                };

                // log_debug!(
                //     "pspans: {:?} and pwidth: {:?}",
                //     preview_spans,
                //     preview_width
                // );

                // Calculate spacer width
                let spacer_width = if preview_width > 0 {
                    let preview_start_pos = available_width
                        .saturating_sub(preview_width)
                        .saturating_sub(1); // allow bleed through of the selection line
                    preview_start_pos.saturating_sub(text_width)
                } else if is_selected {
                    available_width.saturating_sub(text_width)
                } else {
                    0
                };

                if spacer_width > 0 {
                    let spacer_style = if is_selected {
                        selection_style
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(" ".repeat(spacer_width), spacer_style));
                }

                // add the preview dots (if a theme entry)
                spans.extend(preview_spans);

                // make the selection bar fill the whole line
                if is_selected {
                    let current_width = calculate_spans_width(&spans);
                    if current_width < available_width {
                        let remaining_width = available_width - current_width;
                        spans.push(Span::styled(" ".repeat(remaining_width), selection_style));
                    }
                }

                lines.push(Line::from(spans));
            }
        }

        let items_paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .style(bg_style.fg(theme.fg()))
            .block(Block::default().padding(menu_items_padding()));

        frame.render_widget(items_paragraph, items_area);
    }
}

fn calculate_cursor_position(left_area: Rect, query: &str) -> Position {
    const BASE_OFFSET: u16 = 2; // "> "
    let qlen = query.chars().count() as u16;
    let mut x = left_area.x + BASE_OFFSET + qlen;
    if x >= left_area.x + left_area.width.saturating_sub(1) {
        x = left_area.x + left_area.width.saturating_sub(2);
    }
    let position = Position { x, y: left_area.y };
    log_debug!("Cursor position: {position:?}");
    position
}

fn render_search_bar(frame: &mut Frame, area: Rect, theme: &Theme, menu: &Menu) {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);

    let left_area = sections[0];
    let right_area = sections[1];

    let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let highlight_style = Style::default().fg(theme.fg());

    let mut spans = Vec::with_capacity(3);
    spans.push(Span::styled(">", highlight_style));

    if !menu.has_search_query() {
        spans.push(Span::styled(" Search...", dim_style));
    }

    let query = menu.search_query();
    if !query.is_empty() {
        spans.push(Span::styled(format!(" {}", query), highlight_style));
    }

    let left = Paragraph::new(vec![Line::from(spans)]);
    frame.render_widget(left, left_area);

    frame.set_cursor_position(calculate_cursor_position(left_area, query));

    let (m, n) = menu
        .current_menu()
        .map(|_| {
            let items = menu.current_items();
            let n = items.len();
            let current_index = menu.current_index().unwrap_or(0);
            (current_index.saturating_add(1), n)
        })
        .unwrap_or((0, 0));

    let right = Paragraph::new(format!("{}/{}", if n == 0 { 0 } else { m }, n))
        .style(dim_style)
        .alignment(Alignment::Right);
    frame.render_widget(right, right_area);
}

fn render_theme_dots(theme: &Theme, selection_bg: Option<Color>) -> (Vec<Span<'static>>, usize) {
    let colors = theme_preview_colors();
    const CONTAINER_PADDING: usize = 1;
    const NUM_DOTS: usize = 3;

    // width: padding + (dots + spaces between) + padding
    const EXPECTED_WIDTH: usize = CONTAINER_PADDING * 2 + NUM_DOTS * 2 - 1;

    let container_bg = selection_bg.unwrap_or_else(|| theme.get(ThemeColor::Background));
    let container_style = Style::default().bg(container_bg);

    let mut spans = Vec::with_capacity(EXPECTED_WIDTH);

    // leading padding
    for _ in 0..CONTAINER_PADDING {
        spans.push(Span::styled(" ", container_style));
    }

    // theme dots
    for (i, &color) in colors.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" ", container_style));
        }

        let dot_color = theme.get(color);
        let dot_style = Style::default()
            .fg(dot_color)
            .bg(container_bg)
            .remove_modifier(Modifier::DIM);

        // https://www.compart.com/en/unicode/U+25C9
        spans.push(Span::styled("◉", dot_style));
    }

    // trailing padding
    for _ in 0..CONTAINER_PADDING {
        spans.push(Span::styled(" ", container_style));
    }

    // Width: CONTAINER_PADDING * 2 + NUM_DOTS + (NUM_DOTS - 1)
    let width = CONTAINER_PADDING * 2 + NUM_DOTS + (NUM_DOTS - 1);

    (spans, width)
}

fn extract_theme_name(item: &MenuItem) -> Option<&str> {
    if let MenuAction::Action(Action::SetTheme(name)) = &item.action {
        Some(name.as_str())
    } else {
        None
    }
}

fn create_item_spans(
    item: &MenuItem,
    is_selected: bool,
    base_style: Style,
    theme: &Theme,
    label: String,
) -> Vec<Span<'static>> {
    let label_style = if is_selected {
        base_style.fg(theme.bg()).bg(theme.fg())
    } else if item.is_disabled {
        base_style.add_modifier(Modifier::DIM)
    } else {
        base_style
    };

    if let Some(tag) = &item.tag {
        let tag_style = if is_selected {
            Style::default()
                .fg(theme.bg())
                .bg(theme.fg())
                .add_modifier(Modifier::DIM)
        } else {
            Style::default().fg(theme.fg()).add_modifier(Modifier::DIM)
        };

        vec![
            Span::styled(format!("{tag}: "), tag_style),
            Span::styled(label, label_style),
        ]
    } else {
        vec![Span::styled(label, label_style)]
    }
}
