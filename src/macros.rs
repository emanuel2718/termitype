/// Logs a debug message. Only enabled in debug builds.
///
/// # Examples
///
/// ```
/// use termitype::log_debug;
/// let item = "example_item";
/// let status = "running";
/// log_debug!("Termitype artifact {}", item);
/// log_debug!("Status: {status}");
/// ```
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        $crate::log::write(
            $crate::log::Level::Debug,
            &format!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
        )
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{}};
}

/// Logs an info message.
///
/// # Examples
///
/// ```
/// use termitype::log_info;
/// let duration = 123;
/// log_info!("Termitype started");
/// log_info!("Test took {}ms", duration);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        $crate::log::write(
            $crate::log::Level::Info,
            &format!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
        )
    }};
}

/// Logs a warning message.
///
/// # Examples
///
/// ```
/// use termitype::log_warn;
/// let id = 456;
/// let reason = "timeout";
/// log_warn!("Failed to process item {}, retrying...", id);
/// log_warn!("Performance degraded: {reason}");
/// ```
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        $crate::log::write(
            $crate::log::Level::Warn,
            &format!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
        )
    }};
}

/// Logs an error message.
///
/// # Examples
///
/// ```
/// use termitype::log_error;
/// let err = "permission denied";
/// let msg = "Failed to open resource";
/// log_error!("Failed to save file: {}", err);
/// log_error!("Critical error occurred: {msg}");
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        $crate::log::write(
            $crate::log::Level::Error,
            &format!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
        )
    }};
}

/// Sends a notification of severity `NotificationSeverity::Info`.
///
/// # Examples
///
/// ```
/// use termitype::notify_info;
/// notify_info!("Test invalid - too short");
/// ```
#[macro_export]
macro_rules! notify_info {
    ($message:expr) => {{
        $crate::notifications::_notify_info($message)
    }};
}

/// Sends a notification of severity `NotificationSeverity::Warning`.
///
/// # Examples
///
/// ```
/// use termitype::notify_warning;
/// notify_warning!("Using fallback theme");
/// ```
#[macro_export]
macro_rules! notify_warning {
    ($message:expr) => {{
        $crate::notifications::_notify_warning($message)
    }};
}

/// Sends a notification of severity `NotificationSeverity::Error`.
///
/// # Examples
///
/// ```
/// use termitype::notify_error;
/// notify_error!("Could not load database");
/// ```
#[macro_export]
macro_rules! notify_error {
    ($message:expr) => {{
        $crate::notifications::_notify_error($message)
    }};
}
