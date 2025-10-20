use crate::{
    app::App,
    db::{LeaderboardColumn, SortOrder},
    log_info,
    theme::Theme,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Row, Table},
    Frame,
};

#[derive(Debug, Clone)]
struct SortInfo {
    col: LeaderboardColumn,
    order: SortOrder,
}

#[derive(Debug, Clone, Copy)]
struct ShowRules {
    language: bool,
    mode: bool,
    errors: bool,
    consistency: bool,
    raw: bool,
    accuracy: bool,
}

pub struct LeaderboardOverlay;

impl LeaderboardOverlay {
    pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
        let Some(ref leaderboard) = app.leaderboard else {
            return;
        };

        if !leaderboard.is_open() {
            return;
        }

        let overlay_area = Self::calculate_leaderboard_overlay_area(area);

        frame.render_widget(Clear, overlay_area);

        let bg_block = Block::default().style(Style::default().bg(theme.bg()));
        frame.render_widget(bg_block, overlay_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(overlay_area);

        let content_area = chunks[0];
        let bottom_bar_area = chunks[1];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
            .title(" Leaderboard ")
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(theme.fg()).bold())
            .padding(Padding::symmetric(1, 1))
            .style(Style::default().bg(theme.bg()));

        let inner_area = block.inner(content_area);
        frame.render_widget(block, content_area);

        Self::render_table(frame, app, theme, inner_area);
        Self::render_bottom_bar(frame, app, theme, bottom_bar_area);
    }

    fn calculate_leaderboard_overlay_area(area: Rect) -> Rect {
        let width = area.width.saturating_sub(6).min(140);
        let height = area.height.saturating_sub(6).min(35);
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;

        Rect {
            x,
            y,
            width,
            height,
        }
    }

    fn render_table(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
        let Some(ref leaderboard) = app.leaderboard else {
            return;
        };

        let (sort_col, sort_order) = leaderboard.current_sort();
        let data = leaderboard.data();
        let width = area.width;

        // We don't have any results to show
        if data.is_empty() {
            let message = Paragraph::new("No results found")
                .style(Style::default().fg(theme.fg()).add_modifier(Modifier::DIM))
                .alignment(Alignment::Center);

            let vertical_padding = area.height / 2;
            let message_area = Rect {
                x: area.x,
                y: area.y + vertical_padding,
                width: area.width,
                height: 1,
            };

            frame.render_widget(message, message_area);
            return;
        }

        let sort_info = SortInfo {
            col: sort_col.clone(),
            order: sort_order.clone(),
        };

        // TODO: improve how we do this, this is a tad bit hardcoded
        // NOTE: this is the order of importance when hiding due to size limitations:
        // Order: WPM > Date > Accuracy > Raw > Consistency > Errors > Mode > Language
        log_info!("The width: {width}");
        let show_rules = ShowRules {
            language: width >= 115,
            mode: width >= 98,
            errors: width >= 87,
            consistency: width >= 75,
            raw: width >= 57,
            accuracy: width >= 42,
        };
        let headers = Self::build_leaderboard_headers(theme, sort_info, show_rules);

        // TODO: do this in a more idiomatic rust way...
        let rows: Vec<Row> = data
            .iter()
            .map(|result| {
                let mode_display = format!("{}({})", result.mode_kind, result.mode_value);
                let date_format = if width >= 30 {
                    "%d %b %Y %H:%M"
                } else {
                    "%d %b %Y"
                };
                // format: "01 Jan 2001 11:11"
                let date_display = result.created_at.format(date_format).to_string();

                let mut cells = vec![result.wpm.to_string()];

                if show_rules.accuracy {
                    cells.push(format!("{}%", result.accuracy));
                }

                if show_rules.raw {
                    cells.push(result.raw_wpm.to_string());
                }

                if show_rules.consistency {
                    cells.push(format!("{}%", result.consistency));
                }

                if show_rules.errors {
                    cells.push(result.error_count.to_string());
                }

                if show_rules.mode {
                    cells.push(mode_display);
                }

                if show_rules.language {
                    cells.push(result.language.clone());
                }

                cells.push(date_display);

                Row::new(cells)
                    .style(Style::default().fg(theme.fg()))
                    .height(1)
            })
            .collect();

        // columns widths
        let widths: Vec<Constraint> = if show_rules.language {
            vec![
                Constraint::Length(10), // WPM
                Constraint::Length(14), // Accuracy
                Constraint::Length(10), // Raw
                Constraint::Length(16), // Consistency
                Constraint::Length(12), // Errors
                Constraint::Min(12),    // Mode
                Constraint::Min(16),    // Language
                Constraint::Length(18), // Date
            ]
        } else if show_rules.mode {
            log_info!("WE ARE SHOWING THE MODE");
            vec![
                Constraint::Length(8),  // WPM
                Constraint::Length(12), // Accuracy
                Constraint::Length(8),  // Raw
                Constraint::Length(16), // Consistency
                Constraint::Length(12), // Errors
                Constraint::Min(16),    // Mode (rem)
                Constraint::Length(18), // Date
            ]
        } else if show_rules.errors {
            log_info!(">>>>>> WE ARE SHOWING THE ERRORS");
            vec![
                Constraint::Length(10), // WPM
                Constraint::Length(16), // Accuracy
                Constraint::Length(10), // Raw
                Constraint::Length(18), // Consistency
                Constraint::Min(12),    // Errors (rem)
                Constraint::Length(18), // Date
            ]
        } else if show_rules.consistency {
            log_info!("!!!!!!!! WE ARE SHOWING THE CONSISTENCY");
            vec![
                Constraint::Length(10), // WPM
                Constraint::Length(16), // Accuracy
                Constraint::Length(10), // Raw
                Constraint::Min(14),    // Consistency (rem)
                Constraint::Length(18), // Date
            ]
        } else if show_rules.raw {
            log_info!("***** WE ARE SHOWING THE RAW");
            vec![
                Constraint::Length(12), // WPM
                Constraint::Length(18), // Accuracy
                Constraint::Min(12),    // Raw (rem)
                Constraint::Length(18), // Date
            ]
        } else if show_rules.accuracy {
            vec![
                Constraint::Length(12), // WPM
                Constraint::Min(14),    // Accuracy (rem)
                Constraint::Length(18), // Date
            ]
        } else {
            vec![
                Constraint::Percentage(30), // WPM
                Constraint::Percentage(70), // Date
            ]
        };

        let table = Table::new(rows, widths)
            .header(headers)
            .row_highlight_style(
                Style::default()
                    .bg(theme.selection_bg())
                    .fg(theme.selection_fg())
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("  ")
            .column_spacing(1);

        frame.render_stateful_widget(table, area, &mut app.leaderboard.as_mut().unwrap().table);
    }

    fn build_leaderboard_headers<'a>(theme: &Theme, sort: SortInfo, show: ShowRules) -> Row<'a> {
        let sort_indicator = match sort.order {
            SortOrder::Descending => "↓",
            SortOrder::Ascending => "↑",
        };

        let mut columns = vec![("WPM [w]", "wpm")];

        if show.accuracy {
            columns.push(("Accuracy [a]", "accuracy"));
        }

        if show.raw {
            columns.push(("Raw [r]", "raw_wpm"));
        }

        if show.consistency {
            columns.push(("Consistency [c]", "consistency"));
        }

        if show.errors {
            columns.push(("Errors [e]", "error_count"));
        }

        if show.mode {
            columns.push(("Mode [m]", "mode_kind"));
        }

        if show.language {
            columns.push(("Language [l]", "language"));
        }

        columns.push(("Date [d]", "created_at"));

        let header_cells: Vec<Line> = columns
            .iter()
            .map(|(label, col_str)| {
                let is_active = sort.col.to_value() == *col_str;
                let style = if is_active {
                    Style::default()
                        .fg(theme.success())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg()).add_modifier(Modifier::DIM)
                };

                let text = if is_active {
                    format!("{} {}", label, sort_indicator)
                } else {
                    label.to_string()
                };

                Line::from(Span::styled(text, style))
            })
            .collect();

        Row::new(header_cells)
            .style(Style::default().bg(theme.bg()))
            .height(2)
            .bottom_margin(1)
    }

    fn render_bottom_bar(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
        let Some(ref leaderboard) = app.leaderboard else {
            return;
        };

        let (sort_col, sort_order) = leaderboard.current_sort();

        let current_selection = leaderboard.table.selected().map(|i| i + 1).unwrap_or(0);
        let total_count = if let Some(state) = &leaderboard.state {
            state.count
        } else {
            0
        };

        let sort_name = match sort_col.to_value() {
            "mode_kind" => "Mode",
            "language" => "Language",
            "wpm" => "WPM",
            "raw_wpm" => "Raw",
            "accuracy" => "Accuracy",
            "consistency" => "Consistency",
            "error_count" => "Errors",
            "created_at" => "Date",
            _ => "Unknown",
        };
        let sort_indicator = match sort_order {
            SortOrder::Descending => "↓",
            SortOrder::Ascending => "↑",
        };

        let status_text = if total_count == 0 {
            "No results".to_string()
        } else {
            format!(
                "{}/{} results  |  Sort: {} {}  ",
                current_selection, total_count, sort_name, sort_indicator
            )
        };

        let status_paragraph = Paragraph::new(status_text)
            .style(
                Style::default()
                    .fg(theme.fg())
                    .bg(theme.bg())
                    .add_modifier(Modifier::DIM),
            )
            .alignment(Alignment::Right);

        frame.render_widget(status_paragraph, area);
    }
}
