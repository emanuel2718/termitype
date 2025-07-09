use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
    Frame,
};

use crate::{
    constants::{MIN_TERM_HEIGHT, MIN_WIDTH_FOR_NOTIFICATIONS},
    notifications::{get_active_notifications, NotificationPosition},
    termi::Termi,
    ui::helpers::TermiUtils,
};

pub struct NotificationComponent;

impl NotificationComponent {
    pub fn render(frame: &mut Frame, termi: &Termi, area: Rect) {
        if termi.config.hide_notifications {
            return;
        }

        let notifications = get_active_notifications();
        if notifications.is_empty() {
            return;
        }

        // terminal screen too smool
        if area.width < MIN_WIDTH_FOR_NOTIFICATIONS || area.height < MIN_TERM_HEIGHT {
            return;
        }

        let theme = termi.current_theme();
        let position = NotificationPosition::default(); // TOOD: this needs to be configurable
        let symbols = TermiUtils::get_symbols(theme.supports_unicode());

        let notification_width = 35.min(area.width.saturating_sub(4));
        let margin_left = 2;
        let margin_right = 1;

        let mut total_height = 0;
        let mut notification_heights = Vec::new();
        for notification in &notifications {
            let height = Self::calculate_notification_height(notification, notification_width);
            notification_heights.push(height);
            total_height += height;
        }

        let (start_x, start_y) = match position {
            NotificationPosition::TopLeft => (area.x + margin_left, area.y),
            NotificationPosition::TopRight => (
                area.x + area.width.saturating_sub(notification_width + margin_right),
                area.y,
            ),
            NotificationPosition::BottomLeft => (
                area.x + margin_left,
                area.y + area.height.saturating_sub(total_height),
            ),
            NotificationPosition::BottomRight => (
                area.x + area.width.saturating_sub(notification_width + margin_right),
                area.y + area.height.saturating_sub(total_height),
            ),
        };

        // render the notifications
        let mut current_y = start_y;
        for (i, notification) in notifications.iter().enumerate() {
            let notification_height = notification_heights[i];
            let notification_area = Rect {
                x: start_x.saturating_sub(1),
                y: current_y,
                width: notification_width,
                height: notification_height,
            };

            // bounds checkin
            if notification_area.y + notification_area.height > area.y + area.height
                || notification_area.x + notification_area.width > area.x + area.width
            {
                break;
            }

            Self::render_notification(frame, notification, notification_area, theme, &symbols);
            current_y += notification_height;
        }
    }

    /// Calculate the height needed for a notification based on its content
    fn calculate_notification_height(
        notification: &crate::notifications::Notification,
        notification_width: u16,
    ) -> u16 {
        let available_width = notification_width.saturating_sub(4);

        let title_height = 1;

        let message_lines = if notification.message.is_empty() {
            1
        } else {
            (notification.message.len() as u16)
                .div_ceil(available_width)
                .clamp(1, 4)
        };

        // y-borders + title + lork space + message lines
        2 + title_height + 1 + message_lines
    }

    /// Renders a single notification
    fn render_notification(
        frame: &mut Frame,
        notification: &crate::notifications::Notification,
        area: Rect,
        theme: &crate::theme::Theme,
        symbols: &crate::ui::helpers::TermiSymbols,
    ) {
        frame.render_widget(Clear, area);

        let notification_color = notification.color(theme);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(notification_color))
            .style(Style::default().bg(theme.bg()))
            .padding(Padding::horizontal(1));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // icon + title
                Constraint::Max(4),    // message
            ])
            .split(inner_area);

        // <icon><spacing><title>
        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1), // icon
                Constraint::Length(2), // spacing
                Constraint::Min(0),    // title
            ])
            .split(vertical_layout[0]);

        let icon_area = top_layout[0];
        let title_area = top_layout[2];

        let icon = match notification.severity {
            crate::notifications::NotificationSeverity::Info => symbols.info,
            crate::notifications::NotificationSeverity::Warning => symbols.warning,
            crate::notifications::NotificationSeverity::Error => symbols.error,
        };

        let icon_paragraph = Paragraph::new(icon)
            .style(Style::default().fg(notification_color))
            .alignment(Alignment::Center);
        frame.render_widget(icon_paragraph, icon_area);

        let title_style = Style::default()
            .fg(notification_color)
            .add_modifier(Modifier::BOLD);
        let title_paragraph = Paragraph::new(notification.title.clone())
            .style(title_style)
            .alignment(Alignment::Left);
        frame.render_widget(title_paragraph, title_area);

        let message_style = Style::default().fg(theme.fg());
        let message_paragraph = Paragraph::new(notification.message.clone())
            .style(message_style)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(message_paragraph, vertical_layout[1]);
    }
}
