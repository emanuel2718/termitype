/// Logs a debug message. Only enabled in debug builds.
///
/// # Examples
///
/// ```
/// use termitype::debug;
/// let item = "example_item";
/// let status = "running";
/// debug!("Termitype artifact {}", item);
/// debug!("Status: {status}");
/// ```
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        $crate::log::write(
            $crate::log::Level::Debug,
            &format!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
        )
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{}};
}

/// Logs an info message.
///
/// # Examples
///
/// ```
/// use termitype::info;
/// let duration = 123;
/// info!("Termitype started");
/// info!("Test took {}ms", duration);
/// ```
#[macro_export]
macro_rules! info {
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
/// use termitype::warn;
/// let id = 456;
/// let reason = "timeout";
/// warn!("Failed to process item {}, retrying...", id);
/// warn!("Performance degraded: {reason}");
/// ```
#[macro_export]
macro_rules! warn {
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
/// use termitype::error;
/// let err = "permission denied";
/// let msg = "Failed to open resource";
/// error!("Failed to save file: {}", err);
/// error!("Critical error occurred: {msg}");
/// ```
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        $crate::log::write(
            $crate::log::Level::Error,
            &format!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
        )
    }};
}
