use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, HighlightSpacing, Padding, Paragraph, Row, Table,
        Wrap,
    },
    Frame,
};

use crate::{
    actions::TermiClickAction,
    db::SortOrder,
    leaderboard::{Leaderboard, SortColumn},
    termi::Termi,
    theme::Theme,
    ui::helpers::LayoutHelper,
};

pub struct LeaderboardComponent;

impl LeaderboardComponent {
    pub fn render(
        f: &mut Frame,
        termi: &mut Termi,
        area: Rect,
    ) -> Option<(Rect, TermiClickAction)> {
        if !termi.leaderboard.is_open() {
            return None;
        }

        let theme = termi.current_theme().clone();
        let width = (area.width as f32 * 0.85).clamp(70.0, 120.0) as u16;
        let height = (area.height as f32 * 0.80).clamp(15.0, 30.0) as u16;
        let container = LayoutHelper::center_rect(area, width, height);

        f.render_widget(Clear, container);

        let block = Block::default()
            .title(" Leaderboard ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.bg()));

        let inner_area = block.inner(container);
        f.render_widget(block, container);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // top spacing
                Constraint::Min(8),    // table
                Constraint::Length(1), // spacing
                Constraint::Length(2), // footer
            ])
            .split(inner_area);

        Self::render_table(f, &mut termi.leaderboard, chunks[1], &theme);
        Self::render_footer(f, &termi.leaderboard, chunks[3], &theme);

        Self::render_close(f, container)
    }

    fn render_table(frame: &mut Frame, leaderboard: &mut Leaderboard, area: Rect, theme: &Theme) {
        let items = leaderboard.items();
        if leaderboard.has_results() {
            let cols = SortColumn::all();
            let current_col_idx = leaderboard.current_sort_col_idx();
            let sort_order = leaderboard.sort_order();

            let headers: Vec<Cell> = cols
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let mut spans = Self::create_header_labels(col.to_display_name(), theme);

                    if i == current_col_idx {
                        let sort_symbol = match sort_order {
                            SortOrder::Ascending => " ↑",
                            SortOrder::Descending => " ↓",
                        };
                        spans.push(Span::styled(
                            sort_symbol,
                            Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                        ));
                    }

                    Cell::from(Line::from(spans))
                })
                .collect();

            let header_row = Row::new(headers).height(1);

            let rows: Vec<Row> = items
                .iter()
                .map(|result| {
                    let chars_text = format!(
                        "{}/{}/0/0",
                        result.correct_keystrokes,
                        result
                            .total_keystrokes
                            .saturating_sub(result.correct_keystrokes)
                    );

                    let cells = vec![
                        Cell::from(result.wpm.to_string()),
                        Cell::from(result.raw_wpm.to_string()),
                        Cell::from(format!("{}%", result.accuracy)),
                        Cell::from(chars_text),
                        Cell::from(result.language.clone()),
                        Cell::from(format!("{} {}", result.mode_type, result.mode_value)),
                        Cell::from(result.created_at.format("%d %b %Y %H:%M").to_string()),
                    ];

                    Row::new(cells).height(1)
                })
                .collect();

            let constraints = [
                Constraint::Length(6),  // WPM
                Constraint::Length(6),  // Raw WPM
                Constraint::Length(10), // Accuracy
                Constraint::Length(12), // Chars
                Constraint::Fill(1),    // Language
                Constraint::Length(12), // Mode
                Constraint::Fill(1),    // Date
            ];

            let table = Table::new(rows, constraints)
                .header(header_row)
                .row_highlight_style(
                    Style::default()
                        .bg(theme.selection_bg())
                        .fg(theme.selection_fg())
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_spacing(HighlightSpacing::Always);

            frame.render_stateful_widget(table, area, leaderboard.table());
        } else {
            let message = if leaderboard.error_message().is_some() {
                "Failed to load data"
            } else {
                "Loading..."
            };

            let loading_text = Paragraph::new(message)
                .style(Style::default().fg(theme.muted()))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            let loading_area = LayoutHelper::center_rect(area, 30, 3);
            frame.render_widget(loading_text, loading_area);
        }
    }

    fn create_header_labels(name: &str, theme: &Theme) -> Vec<Span<'static>> {
        match name {
            "WPM" => vec![
                Span::styled(
                    "W",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "PM",
                    Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                ),
            ],
            "Raw" => vec![
                Span::styled(
                    "R",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "aw",
                    Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                ),
            ],
            "Accuracy" => vec![
                Span::styled(
                    "A",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "ccuracy",
                    Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                ),
            ],
            "Chars" => vec![
                Span::styled(
                    "C",
                    Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "h",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "ars",
                    Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                ),
            ],
            "Language" => vec![
                Span::styled(
                    "L",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "anguage",
                    Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                ),
            ],
            "Mode" => vec![
                Span::styled(
                    "M",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "ode",
                    Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                ),
            ],
            "Date" => vec![
                Span::styled(
                    "D",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "ate",
                    Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                ),
            ],
            _ => vec![Span::styled(
                name.to_string(),
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )],
        }
    }

    fn render_footer(frame: &mut Frame, leaderboard: &Leaderboard, area: Rect, theme: &Theme) {
        let footer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // info
                Constraint::Length(1), // nav
            ])
            .split(area);

        let header_info_text = if leaderboard.has_results() {
            let sort_indicator = match leaderboard.sort_order() {
                SortOrder::Ascending => "↑",
                SortOrder::Descending => "↓",
            };

            let loading_text = if leaderboard.is_loading() {
                "  Loading more..."
            } else {
                ""
            };

            format!(
                "Sorted by {} {} • Showing {} of {} results{}",
                leaderboard.sort_col(),
                sort_indicator,
                leaderboard.items().len(),
                leaderboard.count(),
                loading_text
            )
        } else if let Some(err) = leaderboard.error_message() {
            format!("Error: {}", err)
        } else {
            "Loading...".to_string()
        };

        let header_info_paragraph = Paragraph::new(header_info_text)
            .style(Style::default().fg(theme.muted()))
            .alignment(Alignment::Center);
        frame.render_widget(header_info_paragraph, footer_chunks[1]);
    }

    fn render_close(frame: &mut Frame, area: Rect) -> Option<(Rect, TermiClickAction)> {
        let close_area = Rect {
            x: area.x + area.width - 3,
            y: area.y,
            width: 3,
            height: 1,
        };

        let close_button = Paragraph::new(" x ").alignment(Alignment::Center);

        frame.render_widget(close_button, close_area);
        Some((close_area, TermiClickAction::LeaderboardClose))
    }
}
