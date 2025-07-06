use std::time::{Duration, Instant};

use crate::theme::Theme;

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

// TODO: all(), FromStr, etc

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

#[derive(Debug, Default)]
pub struct NotificationManager {
    max_count: usize,
    duration: Duration,
    position: NotificationPosition,
    notifications: Vec<Notification>,
}

impl NotificationManager {
    pub fn new() -> Self {
        const MAX_NOTIFICATION_COUNT: usize = 3;
        const DEFAULT_DURATION: Duration = Duration::from_secs(3);
        Self {
            notifications: Vec::new(),
            duration: DEFAULT_DURATION,
            max_count: MAX_NOTIFICATION_COUNT,
            position: NotificationPosition::default(),
        }
    }

    pub fn position(&self) -> NotificationPosition {
        self.position
    }

    pub fn max_count(&self) -> usize {
        self.max_count
    }

    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    pub fn get_notifications(&mut self) -> &[Notification] {
        self.cleanup();
        &self.notifications
    }

    pub fn info(&mut self, title: impl Into<String>, message: impl Into<String>) {
        let severity = NotificationSeverity::Info;
        self.add(Notification::new(title, message, severity, self.duration));
    }

    pub fn warning(&mut self, title: impl Into<String>, message: impl Into<String>) {
        let severity = NotificationSeverity::Warning;
        self.add(Notification::new(title, message, severity, self.duration));
    }

    pub fn error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        let severity = NotificationSeverity::Error;
        self.add(Notification::new(title, message, severity, self.duration));
    }

    fn add(&mut self, notification: Notification) {
        self.cleanup();
        if self.notifications.len() >= self.max_count {
            self.notifications.remove(0);
        }
        self.notifications.push(notification);
    }

    fn cleanup(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }
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
    fn test_notification_manager() {
        let mut manager = NotificationManager::new();
        assert_eq!(manager.notifications.len(), 0);
        assert_eq!(manager.max_count(), 3);

        manager.info("title", "msg");

        assert_eq!(manager.notifications.len(), 1);
        manager.info("title", "msg"); // 2
        manager.info("title", "msg"); // 3
        manager.info("title", "msg"); // 4 (should not be added due to max_count = 3)
        assert_eq!(manager.notifications.len(), 3);

        manager.clear();
        assert_eq!(manager.notifications.len(), 0);
    }
}
