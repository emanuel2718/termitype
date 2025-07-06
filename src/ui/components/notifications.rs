use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
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

        // notification dimensions
        let notification_width = 35.min(area.width.saturating_sub(4));
        let notification_height = 4;
        let margin = 1;

        let (start_x, start_y) = match position {
            NotificationPosition::TopLeft => (area.x + margin, area.y + margin),
            NotificationPosition::TopRight => (
                area.x + area.width.saturating_sub(notification_width + margin),
                area.y + margin,
            ),
            NotificationPosition::BottomLeft => (
                area.x + margin,
                area.y
                    + area.height.saturating_sub(
                        (notification_height + margin) * notifications.len() as u16 + margin,
                    ),
            ),
            NotificationPosition::BottomRight => (
                area.x + area.width.saturating_sub(notification_width + margin),
                area.y
                    + area.height.saturating_sub(
                        (notification_height + margin) * notifications.len() as u16 + margin,
                    ),
            ),
        };

        // render the notifications
        for (i, notification) in notifications.iter().enumerate() {
            let notification_area = Rect {
                x: start_x,
                y: start_y + (i as u16 * (notification_height + margin)),
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
        }
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
                Constraint::Length(1), // icon + title
                Constraint::Length(1), // message
            ])
            .split(inner_area);

        // <icon><spacing><title>
        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1), // icon
                Constraint::Length(1), // spacing
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
            .alignment(Alignment::Left);
        frame.render_widget(message_paragraph, vertical_layout[1]);
    }
}
