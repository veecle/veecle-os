#[cfg(feature = "enable")]
use crate::collector::get_collector;
use crate::protocol::transient;
#[cfg(feature = "enable")]
use crate::time::now;

/// Logs a message with the specified severity level and attributes.
///
/// Prefer using the macros.
///
/// This function creates a log message with the given severity, body text, and
/// key-value attributes.
/// If there is an active span context, the log message
/// will automatically be correlated with the trace and span IDs.
///
/// # Arguments
///
/// * `severity` - The severity level of the log message
/// * `body` - The main message text
/// * `attributes` - Key-value pairs providing additional context
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::span;
/// use veecle_telemetry::log::log;
/// use veecle_telemetry::protocol::transient::{Severity, KeyValue};
///
/// // Simple log message
/// log(Severity::Info, "Server started", &[]);
///
/// // Log with attributes
/// log(Severity::Warn, "High memory usage", &[
///     KeyValue::new("memory_usage_percent", 85),
///     KeyValue::new("available_mb", 512),
/// ]);
///
/// // Log within a span context
/// let span = span!("request_handler");
/// let _guard = span.entered();
/// log(Severity::Error, "Request failed", &[KeyValue::new("error_code", 500)]);
/// ```
///
/// # Conditional Compilation
///
/// When the `enable` feature is disabled, this function compiles to a no-op
/// and has zero runtime overhead.
pub fn log<'a>(
    severity: transient::Severity,
    body: &'a str,
    attributes: &'a [transient::KeyValue<'a>],
) {
    #[cfg(not(feature = "enable"))]
    {
        let _ = (severity, body, attributes);
    }

    #[cfg(feature = "enable")]
    {
        let log_message = transient::LogMessage {
            time_unix_nano: now().as_nanos(),
            severity,
            body,
            attributes,
        };

        get_collector().log_message(log_message);
    }
}
