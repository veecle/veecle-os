//! Macros for structured logging and telemetry.
//!
//! This module provides convenient macros for creating spans, adding events, and logging
//! messages with various severity levels.
//! The macros provide a more ergonomic interface compared to the lower-level functions and handle attribute creation
//! automatically.
//!
//! # Span Creation
//!
//! - `span!`: Creates a new span with optional attributes
//! - `event!`: Adds an event to the current span
//!
//! # Logging Macros
//!
//! - `log!`: Generic logging macro that accepts a severity level
//! - `trace!`: Logs trace-level messages (most verbose)
//! - `debug!`: Logs debug-level messages
//! - `info!`: Logs informational messages
//! - `warn!`: Logs warning messages
//! - `error!`: Logs error messages
//! - `fatal!`: Logs fatal error messages
//!
//! # Attribute Handling
//!
//! - `attributes!`: Creates a slice of key-value attributes
//! - `attribute!`: Creates a single key-value attribute
//!
//! All macros support flexible attribute syntax for adding contextual information.

/// Creates a new span.
///
/// A span represents a unit of work or operation that has a beginning and end.
/// It can contain attributes that provide additional context about the operation.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::span;
///
/// let span = span!("database_query");
/// ```
///
/// ```rust
/// use veecle_telemetry::span;
///
/// let user_id = 123;
/// let table_name = "users";
/// let span =
///     span!("database_query", user_id, table = table_name, "operation" = "select");
/// ```
#[macro_export]
macro_rules! span {
    ($name:literal $(, $($attributes:tt)*)?) => {
        $crate::Span::new($name, $crate::attributes!($($($attributes)*)?))
    };
}

/// Adds an event to the current span.
///
/// Events are timestamped occurrences that happen during the execution of a span.
/// They can include additional context through attributes.
///
/// # Examples
///
/// Add a simple event:
/// ```rust
/// use veecle_telemetry::event;
///
/// event!("cache_miss");
/// ```
///
/// Add an event with attributes:
/// ```rust
/// use veecle_telemetry::event;
///
/// let key = "user:123";
/// let cache_type = "redis";
/// event!("cache_miss", key = key, cache_type = cache_type, "retry_count" = 3);
/// ```
#[macro_export]
macro_rules! event {
    ($name:literal $(, $($attributes:tt)*)?) => {
        $crate::CurrentSpan::add_event($name, $crate::attributes!($($($attributes)*)?))
    };
}

/// Logs a message with the specified severity level.
///
/// This is the base logging macro that other severity-specific macros build upon.
/// It allows you to specify the severity level and optional attributes.
///
/// # Examples
///
/// Log a simple message:
/// ```rust
/// use veecle_telemetry::log;
/// use veecle_telemetry::protocol::Severity;
///
/// log!(Severity::Info, "Application started");
/// ```
///
/// Log a message with attributes:
/// ```rust
/// use veecle_telemetry::log;
/// use veecle_telemetry::protocol::Severity;
///
/// let port = 8080;
/// let version = "1.0.0";
/// log!(Severity::Info, "Server listening", port = port, version = version, "protocol" = "HTTP");
/// ```
#[macro_export]
macro_rules! log {
    ($severity:expr, $body:literal $(, $($attributes:tt)*)?) => {
        $crate::log::log($severity, $body, $crate::attributes!($($($attributes)*)?))
    };
}

/// Logs a trace-level message.
///
/// Trace messages are used for very detailed debugging information,
/// typically only enabled during development or deep troubleshooting.
///
/// # Examples
///
/// Simple trace message:
/// ```rust
/// use veecle_telemetry::trace;
///
/// trace!("Entering function");
/// ```
///
/// Trace message with context:
/// ```rust
/// use veecle_telemetry::trace;
///
/// let function_name = "process_request";
/// let request_id = "req-123";
/// trace!("Function entry", function = function_name, request_id = request_id);
/// ```
#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => {
        $crate::log!($crate::protocol::Severity::Trace, $($args)*);
    };
}

/// Logs a debug-level message.
///
/// Debug messages provide detailed information about the program's execution,
/// useful for diagnosing issues during development and testing.
///
/// # Examples
///
/// Simple debug message:
/// ```rust
/// use veecle_telemetry::debug;
///
/// debug!("Processing user request");
/// ```
///
/// Debug message with variables:
/// ```rust
/// use veecle_telemetry::debug;
///
/// let user_id = 456;
/// let action = "login";
/// debug!("User action processed", user_id = user_id, action = action, "success" = true);
/// ```
#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => {
        $crate::log!($crate::protocol::Severity::Debug, $($args)*);
    };
}

/// Logs an info-level message.
///
/// Info messages provide general information about the program's execution,
/// suitable for normal operational logging.
///
/// # Examples
///
/// Simple info message:
/// ```rust
/// use veecle_telemetry::info;
///
/// info!("Service started successfully");
/// ```
///
/// Info message with metadata:
/// ```rust
/// use veecle_telemetry::info;
///
/// let service_name = "web-server";
/// let version = "2.1.0";
/// info!(
///     "Service initialization complete",
///     service = service_name,
///     version = version,
///     "startup_time_ms" = 1250
/// );
/// ```
#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {
        $crate::log!($crate::protocol::Severity::Info, $($args)*);
    };
}

/// Logs a warning-level message.
///
/// Warning messages indicate potential issues or unusual conditions
/// that don't prevent the program from continuing but should be noted.
///
/// # Examples
///
/// Simple warning message:
/// ```rust
/// use veecle_telemetry::warn;
///
/// warn!("Rate limit approaching");
/// ```
///
/// Warning with context:
/// ```rust
/// use veecle_telemetry::warn;
///
/// let current_requests = 950;
/// let limit = 1000;
/// warn!(
///     "High request rate detected",
///     current_requests = current_requests,
///     limit = limit,
///     "utilization_percent" = 95
/// );
/// ```
#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => {
        $crate::log!($crate::protocol::Severity::Warn, $($args)*);
    };
}

/// Logs an error-level message.
///
/// Error messages indicate serious problems that have occurred
/// but allow the program to continue running.
///
/// # Examples
///
/// Simple error message:
/// ```rust
/// use veecle_telemetry::error;
///
/// error!("Database connection failed");
/// ```
///
/// Error with details:
/// ```rust
/// use veecle_telemetry::error;
///
/// let db_host = "localhost:5432";
/// let error_code = 1001;
/// error!(
///     "Database operation failed",
///     host = db_host,
///     error_code = error_code,
///     "retry_attempted" = true
/// );
/// ```
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        $crate::log!($crate::protocol::Severity::Error, $($args)*);
    };
}

/// Logs a fatal-level message.
///
/// Fatal messages indicate critical errors that will likely cause
/// the program to terminate or become unusable.
///
/// # Examples
///
/// Simple fatal message:
/// ```rust
/// use veecle_telemetry::fatal;
///
/// fatal!("Critical system failure");
/// ```
///
/// Fatal error with context:
/// ```rust
/// use veecle_telemetry::fatal;
///
/// let component = "memory_allocator";
/// let error_type = "out_of_memory";
/// fatal!(
///     "System component failure",
///     component = component,
///     error_type = error_type,
///     "available_memory_mb" = 0
/// );
/// ```
#[macro_export]
macro_rules! fatal {
    ($($args:tt)*) => {
        $crate::log!($crate::protocol::Severity::Fatal, $($args)*);
    };
}

/// Constructs a slice of `KeyValue` attributes.
///
/// This macro is primarily used when manually constructing spans.
///
/// # Syntax
///
/// The macro supports several attribute formats:
/// - `identifier = value` - Uses the identifier as the key name
/// - `"literal" = value` - Uses the literal string as the key name
/// - `identifier` - Uses the identifier as both key and value
/// - `field.subfield` - Simple dot notation for field access
///
/// # Examples
///
/// Basic usage with mixed attribute types:
/// ```rust
/// use veecle_telemetry::attributes;
///
/// let user_id = 123;
/// let service_name = "auth-service";
/// let attrs = attributes!(user_id = user_id, "service" = service_name, "version" = "1.0.0");
/// ```
///
/// Using identifiers as both key and value:
/// ```rust
/// use veecle_telemetry::attributes;
///
/// let database = "postgresql";
/// let timeout = 30;
/// let attrs = attributes!(
///     database, // equivalent to database = database
///     timeout = timeout,
///     "connection_pool" = "primary"
/// );
/// ```
///
/// Span construction with attributes:
/// ```rust
/// use veecle_telemetry::{Span, attributes};
///
/// let operation = "user_login";
/// let user_id = 456;
/// let span = Span::new(
///     "authentication",
///     attributes!(operation = operation, user_id = user_id, "security_level" = "high"),
/// );
/// ```
///
/// Empty attributes:
/// ```rust
/// use veecle_telemetry::attributes;
/// use veecle_telemetry::value::KeyValue;
///
/// let attrs: &[KeyValue] = attributes!(); // Creates an empty slice
/// ```
#[macro_export]
macro_rules! attributes {
    ({ $($kvs:tt)* }) => {
        $crate::attributes_inner!(@ { }, { $($kvs)* })
    };
    ($($kvs:tt)*) => {
        $crate::attributes_inner!(@ { }, { $($kvs)* })
    };
}

/// The actual implementation of `attributes!`, separated out to avoid accidentally recursing into
/// the `$($tt)*` case from the inner cases.
#[doc(hidden)]
#[macro_export]
macro_rules! attributes_inner {
    // Base case, remaining tokens is empty.
    (@ { $($val:expr,)* }, { } ) => {
        &[ $($val,)* ]
    };

    // Recursive cases, take one key-value pair, add it to the output, and recurse on the remaining
    // tokens.
    (@ { $($out:expr,)* }, { $($key:ident).+ $(, $($rest:tt)*)? }) => {
        $crate::attributes_inner!(
            @ { $($out,)* $crate::attribute!($($key).+), },
            { $($($rest)*)? }
        )
    };
    (@ { $($out:expr,)* }, { $key:ident = $value:expr $(, $($rest:tt)*)? }) => {
        $crate::attributes_inner!(
            @ { $($out,)* $crate::attribute!($key = $value), },
            { $($($rest)*)? }
        )
    };
    (@ { $($out:expr,)* }, { $key:literal = $value:expr $(, $($rest:tt)*)? }) => {
        $crate::attributes_inner!(
            @ { $($out,)* $crate::attribute!($key = $value), },
            { $($($rest)*)? }
        )
    };
}

/// Constructs a single attribute `KeyValue` pair.
#[macro_export]
macro_rules! attribute {
    ($($key:ident)+ = $value:expr) => {
        $crate::value::KeyValue::new(::core::stringify!($($key).+), $value)
    };
    ($key:literal = $value:expr) => {
        $crate::value::KeyValue::new($key, $value)
    };
    ($($key:ident)+) => {
        $crate::value::KeyValue::new(::core::stringify!($($key).+), $($key).+)
    };
}
