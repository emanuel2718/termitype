use crate::{
    actions::Action,
    app::App,
    menu::{MenuAction, MenuVisualizer},
    theme::Theme,
    tui::helpers::horizontally_center,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_menu_visualizer(
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
            MenuAction::Action(Action::SetCursorVariant(variant)) => *variant,
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
    variant: &crate::variants::CursorVariant,
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
    _variant: &crate::variants::CursorVariant,
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
        .wrap(ratatui::widgets::Wrap { trim: true });
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
