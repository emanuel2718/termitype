use std::{
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use crate::theme::Theme;

const MAX_NOTIFICATION_COUNT: usize = 3;
const DEFAULT_DURATION: Duration = Duration::from_secs(3);

static GLOBAL_NOTIFICATIONS: OnceLock<Arc<Mutex<Vec<Notification>>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for NotificationPosition {
    fn default() -> Self {
        Self::TopRight
    }
}

impl NotificationPosition {
    pub fn label(&self) -> &'static str {
        match self {
            NotificationPosition::TopLeft => "Top Left",
            NotificationPosition::TopRight => "Top Right",
            NotificationPosition::BottomLeft => "Bottom Left",
            NotificationPosition::BottomRight => "Bottom Right",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Notification {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        severity: NotificationSeverity,
        duration: Duration,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            severity,
            created_at: Instant::now(),
            duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    pub fn color(&self, theme: &Theme) -> ratatui::style::Color {
        match self.severity {
            NotificationSeverity::Info => theme.info(),
            NotificationSeverity::Warning => theme.warning(),
            NotificationSeverity::Error => theme.error(),
        }
    }
}

fn get_notifications() -> &'static Arc<Mutex<Vec<Notification>>> {
    GLOBAL_NOTIFICATIONS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

fn add_notification(notification: Notification) {
    if let Ok(mut notifications) = get_notifications().lock() {
        notifications.retain(|n| !n.is_expired());

        if notifications.len() >= MAX_NOTIFICATION_COUNT {
            notifications.remove(0);
        }

        notifications.push(notification);
    }
}

/// Non expired notifications
pub fn get_active_notifications() -> Vec<Notification> {
    if let Ok(mut notifications) = get_notifications().lock() {
        notifications.retain(|n| !n.is_expired());
        notifications.clone()
    } else {
        Vec::new()
    }
}

/// Clears all notifications
pub fn clear_notifications() {
    if let Ok(mut notifications) = get_notifications().lock() {
        notifications.clear();
    }
}

/// Interna use only
pub fn _notify_info(message: impl Into<String>) {
    let notification = Notification::new(
        "Notice",
        message,
        NotificationSeverity::Info,
        DEFAULT_DURATION,
    );
    add_notification(notification);
}

/// Interna use only
pub fn _notify_warning(message: impl Into<String>) {
    let notification = Notification::new(
        "Warning",
        message,
        NotificationSeverity::Warning,
        DEFAULT_DURATION,
    );
    add_notification(notification);
}

/// Internal use only
pub fn _notify_error(message: impl Into<String>) {
    let notification = Notification::new(
        "Error",
        message,
        NotificationSeverity::Error,
        DEFAULT_DURATION,
    );
    add_notification(notification);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let severity = NotificationSeverity::Info;
        let duration = Duration::from_secs(3);
        let notification = Notification::new("title", "msg", severity, duration);
        assert_eq!(notification.title, "title");
        assert_eq!(notification.message, "msg");
        assert_eq!(notification.severity, severity);
        assert_eq!(notification.duration, duration);
    }

    #[test]
    fn test_global_notifications() {
        clear_notifications();

        _notify_info("msg");
        _notify_warning("msg");
        _notify_error("msg");

        let notifications = get_active_notifications();
        assert_eq!(notifications.len(), 3);

        clear_notifications();
        let notifications = get_active_notifications();
        assert_eq!(notifications.len(), 0);
    }
}
