# Next

## Veecle OS

* **breaking** Telemetry functionality is now always compiled into the runtime.
  The `veecle-telemetry` feature flag has been removed from `veecle-os-runtime`.
  Use the `telemetry-enable` feature flag on `veecle-os` to control telemetry behavior.
* **breaking** Replaced `SpanContext::from_span` with `Span::context` method.
* Added `ThreadAbstraction` trait to OSAL for querying current thread id.
* Fixed `veecle_os::telemetry::instrument` macro to automatically resolve correct crate paths for the facade.

## Veecle Telemetry VSCode Extension

* **breaking** Removed.

# 0.1.0

* Initial release.
