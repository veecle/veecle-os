# Next

## Veecle OS

* **breaking** Removed `TraceId` concept; `SpanContext` now uses `ProcessId` directly for span identification.
* **breaking** Removed `Span::root` method and `root_span!` macro; root spans should use `Span::new` and `span!` instead.
* **breaking** Replaced `SpanContext::from_span` with `Span::context` method.
* **breaking** Telemetry execution id now includes thread id to uniquely identify thread/task combinations.
* Added custom serialization for telemetry ids with hex-encoded string format.
* Added `ThreadAbstraction` trait to OSAL for querying current thread id.
* Fixed `veecle_os::telemetry::instrument` macro to automatically resolve correct crate paths for the facade.

## Veecle Telemetry VSCode Extension

* **breaking** Removed.

# 0.1.0

* Initial release.
