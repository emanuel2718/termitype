use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Table, Wrap,
    },
    Frame,
};

use crate::{
    actions::TermiClickAction, db::SortOrder, leaderboard::Leaderboard, termi::Termi, theme::Theme,
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
            let cols = crate::leaderboard::get_sortable_columns();
            let current_col_idx = leaderboard.current_sort_col_idx();
            let sort_order = leaderboard.sort_order();

            let headers: Vec<Cell> = cols
                .iter()
                .enumerate()
                .map(|(i, (_, name))| {
                    let mut spans = vec![];

                    let (display_key, remaining_text) = Self::get_header_parts(name);

                    spans.push(Span::styled(
                        format!("[{}]", display_key.to_uppercase()),
                        Style::default().fg(theme.accent()),
                    ));

                    if !remaining_text.is_empty() {
                        spans.push(Span::styled(
                            remaining_text,
                            Style::default().fg(theme.muted()),
                        ));
                    }

                    if i == current_col_idx {
                        let sort_symbol = match sort_order {
                            SortOrder::Ascending => " ↑",
                            SortOrder::Descending => " ↓",
                        };
                        spans.push(Span::styled(
                            sort_symbol,
                            Style::default().fg(theme.highlight()),
                        ));
                    }

                    Cell::from(Text::from(Line::from(spans)))
                        .style(Style::default().add_modifier(Modifier::BOLD))
                })
                .collect();

            let header_row = Row::new(headers).height(1);

            let rows: Vec<Row> = items
                .iter()
                .enumerate()
                .map(|(row_idx, result)| {
                    let cells = vec![
                        Cell::from(result.wpm.to_string()),
                        Cell::from(format!("{}%", result.accuracy)),
                        Cell::from(format!("{:.1}%", result.consistency)),
                        Cell::from(format!("{} {}", result.mode_type, result.mode_value)),
                        Cell::from(result.language.clone()),
                        Cell::from(result.created_at.format("%m/%d %H:%M").to_string()),
                    ];

                    let row_style = if row_idx % 2 == 0 {
                        Style::default()
                    } else {
                        Style::default().fg(theme.muted())
                    };

                    Row::new(cells).height(1).style(row_style)
                })
                .collect();

            let constraints = [
                Constraint::Fill(1), // WPM
                Constraint::Fill(1), // Accuraccy
                Constraint::Fill(1), // Consistency
                Constraint::Fill(1), // Mode
                Constraint::Fill(1), // Language
                Constraint::Fill(1), // Date
            ];

            let table = Table::new(rows, constraints)
                .header(header_row)
                .row_highlight_style(
                    Style::default()
                        .bg(theme.selection_bg())
                        .fg(theme.selection_fg()),
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
        frame.render_widget(header_info_paragraph, footer_chunks[0]);

        let controls_text = "↑/j: Up  ↓/k: Down  W/A/C/M/L/D: Sort  Esc/q: Close";
        let controls_paragraph = Paragraph::new(controls_text)
            .style(Style::default().fg(theme.muted()))
            .alignment(Alignment::Center);
        frame.render_widget(controls_paragraph, footer_chunks[1]);
    }

    fn get_header_parts(name: &str) -> (char, String) {
        match name {
            "WPM" => ('W', "PM".to_string()),
            "Acc" => ('A', "cc".to_string()),
            "Cons" => ('C', "ons".to_string()),
            "Mode" => ('M', "ode".to_string()),
            "Language" => ('L', "anguage".to_string()),
            "Date" => ('D', "ate".to_string()),
            _ => {
                let first_char = name.chars().next().unwrap_or('?');
                let remaining: String = name.chars().skip(1).collect();
                (first_char, remaining)
            }
        }
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
