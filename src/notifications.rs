use crate::theme::Theme;
use ratatui::style::Color;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const MAX_NOTIFICATION_COUNT: usize = 3;
const DEFAULT_DUATION: Duration = Duration::from_secs(3);

type NotificationStore = Arc<Mutex<Vec<Notification>>>;

#[cfg(not(test))]
static NOTIFICATIONS: std::sync::OnceLock<NotificationStore> = std::sync::OnceLock::new();

#[cfg(test)]
thread_local! {
    static NOTIFICATIONS: std::cell::RefCell<Option<NotificationStore>> = const { std::cell::RefCell::new(None) };
}

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
    #[rustfmt::skip]
    pub fn new(title: impl Into<String>, msg: impl Into<String>, s: NotificationSeverity) -> Self {
        Self {
            title: title.into(),
            message: msg.into(),
            severity: s,
            created_at: Instant::now(),
            duration: DEFAULT_DUATION,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    pub fn color(&self, theme: &Theme) -> Color {
        match self.severity {
            NotificationSeverity::Info => theme.info(),
            NotificationSeverity::Warning => theme.warning(),
            NotificationSeverity::Error => theme.error(),
        }
    }
}

#[cfg(not(test))]
fn get_notifications() -> &'static Arc<Mutex<Vec<Notification>>> {
    NOTIFICATIONS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

#[cfg(test)]
fn get_notifications() -> Arc<Mutex<Vec<Notification>>> {
    NOTIFICATIONS.with(|n| {
        n.borrow_mut()
            .get_or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
            .clone()
    })
}

pub fn get_active_notifications() -> Vec<Notification> {
    if let Ok(mut notifications) = get_notifications().lock() {
        notifications.retain(|n| !n.is_expired());
        notifications.clone()
    } else {
        Vec::new()
    }
}

pub fn has_any() -> bool {
    !get_active_notifications().is_empty()
}

pub fn clear_notifications() {
    if let Ok(mut notifications) = get_notifications().lock() {
        notifications.clear()
    }
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

#[doc(hidden)]
pub fn notify(severity: NotificationSeverity, msg: impl Into<String>) {
    let title = match severity {
        NotificationSeverity::Info => "Notice",
        NotificationSeverity::Warning => "Warning",
        NotificationSeverity::Error => "Error",
    };

    let notification = Notification::new(title.to_string(), msg, severity);
    add_notification(notification);
}

/// Sends an Info notification
///
/// # Examples
///
/// ```
/// use termitype::notify_info;
/// use termitype::notifications::NotificationSeverity;
/// notify_info!("Example notification");
/// ```
#[macro_export]
macro_rules! notify_info {
    ($message:expr) => {{
        $crate::notifications::notify($crate::notifications::NotificationSeverity::Info, $message)
    }};
}

/// Sends an Warning notification
///
/// # Examples
///
/// ```
/// use termitype::notify_warning;
/// use termitype::notifications::NotificationSeverity;
/// notify_warning!("Using test theme");
/// ```
#[macro_export]
macro_rules! notify_warning {
    ($message:expr) => {{
        $crate::notifications::notify(
            $crate::notifications::NotificationSeverity::Warning,
            $message,
        )
    }};
}

/// Sends an Error notification
///
/// # Examples
///
/// ```
/// use termitype::notify_error;
/// use termitype::notifications::NotificationSeverity;
/// notify_error!("Fatal error");
/// ```
#[macro_export]
macro_rules! notify_error {
    ($message:expr) => {{
        $crate::notifications::notify($crate::notifications::NotificationSeverity::Error, $message)
    }};
}

// NOTE: we need this to avoid to have to run the test with `--test-threads=1`
#[cfg(test)]
fn reset_notifications() {
    NOTIFICATIONS.with(|n| {
        *n.borrow_mut() = None;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_notification() {
        let severity = NotificationSeverity::Info;
        let notification = Notification::new("termitype", "random msg", severity);

        assert_eq!(notification.title, "termitype");
        assert_eq!(notification.message, "random msg");
        assert_eq!(notification.severity, severity);
        assert_eq!(notification.duration, DEFAULT_DUATION);
    }

    #[test]
    fn test_notification_position_default() {
        assert_eq!(
            NotificationPosition::default(),
            NotificationPosition::TopRight
        );
    }

    #[test]
    fn test_notification_position_label() {
        assert_eq!(NotificationPosition::TopLeft.label(), "Top Left");
        assert_eq!(NotificationPosition::TopRight.label(), "Top Right");
        assert_eq!(NotificationPosition::BottomLeft.label(), "Bottom Left");
        assert_eq!(NotificationPosition::BottomRight.label(), "Bottom Right");
    }

    #[test]
    fn test_notification_is_expired() {
        let mut notification = Notification::new("title", "msg", NotificationSeverity::Info);
        assert!(!notification.is_expired());

        notification.created_at = Instant::now() - Duration::from_secs(4);
        assert!(notification.is_expired());
    }

    #[test]
    fn test_notification_color() {
        let theme = Theme::default();
        let i_n = Notification::new("title", "msg", NotificationSeverity::Info);
        let w_n = Notification::new("title", "msg", NotificationSeverity::Warning);
        let e_n = Notification::new("title", "msg", NotificationSeverity::Error);

        assert_eq!(i_n.color(&theme), theme.info());
        assert_eq!(w_n.color(&theme), theme.warning());
        assert_eq!(e_n.color(&theme), theme.error());
    }

    #[test]
    fn test_notify() {
        reset_notifications();
        notify(NotificationSeverity::Info, "test message".to_string());
        let active = get_active_notifications();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "Notice");
        assert_eq!(active[0].message, "test message");
        assert_eq!(active[0].severity, NotificationSeverity::Info);
    }

    #[test]
    fn test_clear_notifications() {
        reset_notifications();
        notify(NotificationSeverity::Info, "msg".to_string());
        notify(NotificationSeverity::Warning, "warn".to_string());
        assert_eq!(get_active_notifications().len(), 2);
        clear_notifications();
        assert_eq!(get_active_notifications().len(), 0);
    }

    #[test]
    fn test_max_notification_count() {
        reset_notifications();
        for i in 0..5 {
            notify(NotificationSeverity::Info, format!("msg {}", i));
        }
        let active = get_active_notifications();
        assert_eq!(active.len(), 3); // `MAX_NOTIFICATION_COUNT`
        assert_eq!(active[0].message, "msg 2");
        assert_eq!(active[1].message, "msg 3");
        assert_eq!(active[2].message, "msg 4");
    }

    #[test]
    fn test_has_any() {
        reset_notifications();
        assert!(!has_any());
        notify(NotificationSeverity::Info, "test message");
        assert!(has_any());
    }
}
