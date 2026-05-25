/////////////////////////////////////////////////////
// Logging
/////////////////////////////////////////////////////
macro_rules! log {
    ($level:expr, $fmt:literal $(, $arg:expr)*) => {{
        use crate::logging::log_to_all_sinks;

        let now = chrono::Local::now().format("%H:%M:%S");

        match $level
        {
            LogLevel::Trace => log_to_all_sinks($level.clone(), format!("[{}] [TRACE] - {}", now, format_args!($fmt $(, $arg)*)).as_str()),
            LogLevel::Info  => log_to_all_sinks($level.clone(), format!("[{}] [INFO]  - {}", now, format_args!($fmt $(, $arg)*)).as_str()),
            LogLevel::Warn  => log_to_all_sinks($level.clone(), format!("[{}] [WARN]  - {}", now, format_args!($fmt $(, $arg)*)).as_str()),
            LogLevel::Error => log_to_all_sinks($level.clone(), format!("[{}] [ERROR] - {}", now, format_args!($fmt $(, $arg)*)).as_str()),
        };
    }};
}

#[macro_export]
macro_rules! trace { ($fmt:literal $(, $arg:expr)*) => {{ use crate::logging::LogLevel; log!(LogLevel::Trace, $fmt $(, $arg)*) }}; }
#[macro_export]
macro_rules! info  { ($fmt:literal $(, $arg:expr)*) => {{ use crate::logging::LogLevel; log!(LogLevel::Info,  $fmt $(, $arg)*) }}; }
#[macro_export]
macro_rules! warning  { ($fmt:literal $(, $arg:expr)*) => {{ use crate::logging::LogLevel; log!(LogLevel::Warn,  $fmt $(, $arg)*) }}; }
#[macro_export]
macro_rules! error { ($fmt:literal $(, $arg:expr)*) => {{ use crate::logging::LogLevel; log!(LogLevel::Error, $fmt $(, $arg)*) }}; }
