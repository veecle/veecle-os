# Next

## Veecle OS

* **breaking** Telemetry functionality is now always compiled into the runtime.
  The `veecle-telemetry` feature flag has been removed from `veecle-os-runtime` and `telemetry` feature flag has been removed from `veecle-os`.
  Use the `telemetry-enable` feature flag on `veecle-os` to control telemetry behavior.
* **breaking** Removed `Span::root` method and `root_span!` macro; root spans should use `Span::new` and `span!` instead.
* **breaking** Replaced `SpanContext::from_span` with `Span::context` method.
* Added `ThreadAbstraction` trait to OSAL for querying current thread id.
* Updated MSRV to 1.90.
* Fixed `veecle_os::telemetry::instrument` macro to automatically resolve correct crate paths for the facade.

## Veecle Telemetry VSCode Extension

* **breaking** Removed.

# 0.1.0

* Initial release.
