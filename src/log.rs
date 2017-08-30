use chrono::Utc;

pub fn current_rfc2882_string() -> String {
    Utc::now().to_rfc2822()
}

/// Makes a log entry, by prepending the time and date of the entry to what's {{{1
/// provided to the function.
#[macro_export]
macro_rules! make_log_entry {
    ($kind:expr, $arg:expr) => (make_log_entry!($kind, "{}", $arg));
    ($kind:expr, $fmt:expr, $($arg:expr),*) => {{
        let timestamp = $crate::log::current_rfc2882_string();
        let entry = format!($fmt, $($arg),*);
        println!("[{} at {}] {}", $kind, timestamp, entry);
    }};
}

#[macro_export]
macro_rules! log_info {
    ($($arg:expr),+) => (make_log_entry!("Info", $($arg),+));
}

#[macro_export]
macro_rules! log_init {
    ($($arg:expr),+) => (make_log_entry!("Init", $($arg),+));
}

#[macro_export]
macro_rules! log_error {
    ($($arg:expr),+) => (make_log_entry!("Error", $($arg),+));
}

#[macro_export]
macro_rules! log_prefix {
    ($($arg:expr),+) => (make_log_entry!("Prefix", $($arg),+));
}

#[macro_export]
macro_rules! log_status {
    ($($arg:expr),+) => (make_log_entry!("Status", $($arg),+));
}
