use crate::{
    app::App,
    menu::{Menu, MenuVisualizer},
    theme::Theme,
    tui::{
        elements::create_menu_search_bar,
        utils::{horizontally_center, menu_items_padding},
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

pub fn render_telescope_picker(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let menu = &app.menu;

    if let Some(current_menu) = menu.current_menu() {
        let items = menu.current_items();
        if items.is_empty() {
            return;
        }

        let max_width = 70.min(area.width.saturating_sub(6));
        let max_height = 25.min(area.height.saturating_sub(6)).max(12);
        let title = current_menu.title.clone();

        let overlay_area = Rect {
            x: (area.width - max_width) / 2,
            y: (area.height - max_height) / 2,
            width: max_width,
            height: max_height,
        };

        let menu_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD))
            .title(format!(" {} ", title))
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(theme.bg()));

        let inner_area = menu_block.inner(overlay_area);

        frame.render_widget(Clear, inner_area);
        frame.render_widget(menu_block, overlay_area);

        let content_area = inner_area;

        let [items_area, visualizer_area] = {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(content_area);
            [columns[0], columns[1]]
        };

        let items_area_height = items_area.height as usize;
        let items_count = items.len();
        let scroll_offset = current_menu.scroll_offset;
        let visible_items = items
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(items_area_height)
            .collect::<Vec<_>>();

        let mut lines: Vec<Line> = Vec::new();
        let current_index = if menu.has_search_query() {
            if let Some(curr) = current_menu.current_item() {
                items.iter().position(|&item| item == curr).unwrap_or(0)
            } else {
                0
            }
        } else {
            current_menu.current_index
        };

        for (idx, item) in visible_items {
            let is_selected = idx == current_index;
            let mut style = Style::default().fg(theme.fg());

            if is_selected {
                style = style.add_modifier(Modifier::REVERSED);
            }

            if item.is_disabled {
                style = style.add_modifier(Modifier::DIM);
            }

            let label = item.label();

            lines.push(Line::from(vec![Span::styled(label, style)]));
        }

        let items_paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(Block::default().padding(menu_items_padding()));

        // render the items of the menu
        frame.render_widget(items_paragraph, items_area);

        // TODO: move to utils
        // render the scrollbar if needed
        if items_count > items_area_height {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::default()
                .content_length(items_count)
                .viewport_content_length(items_area_height)
                .position(scroll_offset);

            frame.render_stateful_widget(scrollbar, items_area, &mut scrollbar_state);
        }

        // render visualization preview on the right if any
        if let Some(menu) = menu.current_menu() {
            if menu.has_visualizer() && menu.visualizer.is_some() {
                let block = Block::default();
                frame.render_widget(block, visualizer_area);
                let visualizer = menu.visualizer.as_ref().unwrap();
                render_menu_visualizer(frame, theme, visualizer, visualizer_area);
            }
        }

        render_menu_bottom_bar(frame, overlay_area, area, theme, menu);
    }
}

fn render_menu_bottom_bar(
    frame: &mut Frame,
    overlay_area: Rect,
    area: Rect,
    theme: &Theme,
    menu: &Menu,
) {
    let bar_height = 3u16;
    let bar_area = Rect {
        x: overlay_area.x,
        y: overlay_area.y + overlay_area.height,
        width: overlay_area.width,
        height: bar_height,
    };
    if bar_area.y + bar_area.height <= area.y + area.height {
        let block = Block::default().bg(theme.bg());
        frame.render_widget(block, bar_area);
        let bar = create_menu_search_bar(theme, menu.is_searching(), menu.search_query());
        frame.render_widget(bar, bar_area);
        if menu.is_searching() {
            let base_offset: u16 = 8; // "Search: "
            let qlen = menu.search_query().chars().count() as u16;
            let mut x = bar_area.x + 1 + 1 + base_offset + qlen; // +1 for left border, +1 for left padding
            if x >= bar_area.x + bar_area.width.saturating_sub(1) {
                x = bar_area.x + bar_area.width.saturating_sub(2);
            }
            let y = bar_area.y + 1; // inside the bordered box
            frame.set_cursor_position(Position { x, y });
        }
    }
}

fn render_menu_visualizer(frame: &mut Frame, theme: &Theme, vis: &MenuVisualizer, area: Rect) {
    match vis {
        MenuVisualizer::ThemeVisualizer => render_theme_visualizer(frame, theme, area),
    }
}

fn render_theme_visualizer(frame: &mut Frame, theme: &Theme, area: Rect) {
    let visualizer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header + spacing
            Constraint::Length(1), // action bar
            Constraint::Length(4), // space (increased from 2)
            Constraint::Min(5),    // typing area
            Constraint::Length(4), // bottom section
        ])
        .split(area);

    render_theme_header_visualizer(frame, theme, visualizer_layout[0]);
    render_theme_action_bar_visualizer(frame, theme, visualizer_layout[1]);
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

fn render_theme_action_bar_visualizer(frame: &mut Frame, theme: &Theme, area: Rect) {
    let centered = horizontally_center(area, 80);
    let highlight_style = Style::default().fg(theme.highlight());
    let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
    let action_bar = Line::from(vec![
        Span::styled("! ", highlight_style),
        Span::styled("punctuation ", highlight_style),
        Span::styled("# ", dim_style),
        Span::styled("numbers ", dim_style),
    ]);
    let action_bar = Paragraph::new(action_bar).alignment(Alignment::Center);
    frame.render_widget(action_bar, centered);
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
