use crate::{
    notifications::{Notification, NotificationPosition, get_active_notifications},
    theme::Theme,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
};

const MIN_TERMINAL_HEIGHT: u16 = 15;

const NOTIFICATION_WIDTH: u16 = 35;
const NOTIFICATION_MARGIN_LEFT: u16 = 2;
const NOTIFICATION_MARGIN_RIGHT: u16 = 0;
const NOTIFICATION_PADDING: u16 = 1;

const ICON_INFO: &str = "i";
const ICON_WARNING: &str = "!";
const ICON_ERROR: &str = "x";

pub struct NotificationComponent;

impl NotificationComponent {
    pub fn render(frame: &mut Frame, theme: &Theme, area: Rect) {
        let notifications = get_active_notifications();
        if notifications.is_empty() {
            return;
        }

        if !Self::has_sufficient_space(area) {
            return;
        }

        let position = NotificationPosition::default();
        Self::render_notifications(frame, &notifications, theme, area, position);
    }

    /// Are we tall and wide enough to ride?
    fn has_sufficient_space(area: Rect) -> bool {
        area.width >= NOTIFICATION_WIDTH && area.height >= MIN_TERMINAL_HEIGHT
    }

    fn render_notifications(
        frame: &mut Frame,
        notifications: &[Notification],
        theme: &Theme,
        area: Rect,
        position: NotificationPosition,
    ) {
        let notification_width = NOTIFICATION_WIDTH.min(area.width.saturating_sub(4));

        let notification_data: Vec<_> = notifications
            .iter()
            .map(|n| {
                let height = Self::calculate_height(n, notification_width);
                (n, height)
            })
            .collect();

        let total_height: u16 = notification_data.iter().map(|(_, h)| *h).sum();

        let (start_x, start_y) = Self::calculate_start_position(
            area,
            position,
            notification_width,
            total_height,
            NOTIFICATION_MARGIN_LEFT,
            NOTIFICATION_MARGIN_RIGHT,
        );

        let mut current_y = start_y;
        for (notification, height) in notification_data {
            let notification_area = Rect {
                x: start_x.saturating_sub(1),
                y: current_y,
                width: notification_width,
                height,
            };

            if !Self::is_within_bounds(notification_area, area) {
                break;
            }

            Self::render_single_notification(frame, notification, notification_area, theme);
            current_y = current_y.saturating_add(height);
        }
    }

    /// Calculates the required height for a single notification
    fn calculate_height(notification: &Notification, width: u16) -> u16 {
        let content_width = width.saturating_sub(2 * (NOTIFICATION_PADDING + 1));

        let title_height = 1;

        let message_lines = if notification.message.is_empty() {
            0
        } else {
            Self::calculate_wrapped_lines(&notification.message, content_width).max(1)
        };

        // Border (2) + title (1) + spacing (1) + message lines
        2 + title_height + 1 + message_lines
    }

    /// Calculates how many lines are needed for wrapped text
    fn calculate_wrapped_lines(text: &str, max_width: u16) -> u16 {
        if max_width == 0 {
            return 1;
        }

        let mut total_lines = 0;
        for line in text.lines() {
            if line.is_empty() {
                total_lines += 1;
            } else {
                let line_len = line.chars().count() as u16;
                total_lines += line_len.div_ceil(max_width);
            }
        }
        total_lines.clamp(1, 4)
    }

    /// Calculates the starting position for notifications based on placement preference
    fn calculate_start_position(
        area: Rect,
        position: NotificationPosition,
        width: u16,
        total_height: u16,
        margin_left: u16,
        margin_right: u16,
    ) -> (u16, u16) {
        match position {
            NotificationPosition::TopLeft => (area.x + margin_left, area.y),
            NotificationPosition::TopRight => (
                area.x + area.width.saturating_sub(width + margin_right),
                area.y,
            ),
            NotificationPosition::BottomLeft => (
                area.x + margin_left,
                area.y + area.height.saturating_sub(total_height),
            ),
            NotificationPosition::BottomRight => (
                area.x + area.width.saturating_sub(width + margin_right),
                area.y + area.height.saturating_sub(total_height),
            ),
        }
    }

    /// Checks if a notification area is within the terminal bounds
    fn is_within_bounds(notification_area: Rect, parent_area: Rect) -> bool {
        notification_area.y + notification_area.height <= parent_area.y + parent_area.height
            && notification_area.x + notification_area.width <= parent_area.x + parent_area.width
    }

    /// Renders a single notification
    fn render_single_notification(
        frame: &mut Frame,
        notification: &Notification,
        area: Rect,
        theme: &Theme,
    ) {
        frame.render_widget(Clear, area);

        let notification_color = notification.color(theme);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(notification_color))
            .style(Style::default().bg(theme.bg()))
            .padding(Padding::horizontal(NOTIFICATION_PADDING));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // icon + title line + spacing
                Constraint::Min(0),    // message
            ])
            .split(inner_area);

        Self::render_title_section(frame, notification, sections[0], theme, notification_color);

        Self::render_message_section(frame, notification, sections[1], theme);
    }

    /// Renders the title section: icon + title
    fn render_title_section(
        frame: &mut Frame,
        notification: &Notification,
        area: Rect,
        theme: &Theme,
        color: Color,
    ) {
        use crate::notifications::NotificationSeverity;

        let icon = match notification.severity {
            NotificationSeverity::Info => ICON_INFO,
            NotificationSeverity::Warning => ICON_WARNING,
            NotificationSeverity::Error => ICON_ERROR,
        };

        let title_line = Line::from(vec![
            Span::styled(icon, Style::default().fg(color)),
            Span::raw("  "), // spacing
            Span::styled(
                notification.title.clone(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]);

        let title_paragraph = Paragraph::new(title_line)
            .style(Style::default().bg(theme.bg()))
            .alignment(Alignment::Left);

        frame.render_widget(title_paragraph, area);
    }

    /// Renders the message
    fn render_message_section(
        frame: &mut Frame,
        notification: &Notification,
        area: Rect,
        theme: &Theme,
    ) {
        if notification.message.is_empty() {
            return;
        }

        let message_paragraph = Paragraph::new(notification.message.as_str())
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        frame.render_widget(message_paragraph, area);
    }
}
