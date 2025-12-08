# Next

## Veecle OS

* **breaking** Telemetry functionality is now always compiled into the runtime.
  The `veecle-telemetry` feature flag has been removed from `veecle-os-runtime` and `telemetry` feature flag has been removed from `veecle-os`.
  Use the `telemetry-enable` feature flag on `veecle-os` to control telemetry behavior.
* **breaking** Removed `Span::root` method and `root_span!` macro; root spans should use `Span::new` and `span!` instead.
* **breaking** Replaced `SpanContext::from_span` with `Span::context` method.
* **breaking** Telemetry execution id is replaced by separate thread and process ids to uniquely identify thread/task combinations.
* **breaking** `ConsoleJsonExporter` is no longer a unit struct, replace usage with `ConsoleJsonExporter::DEFAULT`.
* Added custom serialization for telemetry ids with hex-encoded string format.
* Added `ThreadAbstraction` trait to OSAL for querying current thread id.
* Updated MSRV to 1.91.
* Fixed `veecle_os::telemetry::instrument` macro to automatically resolve correct crate paths for the facade.
* Implemented `stable_deref_trait::StableDeref` for `Chunk` to allow usage in `yoke`.

## Veecle Telemetry

* Added `ConsolePrettyExporter` for pretty printed telemetry output for non-production use-cases.

## Veecle Telemetry VSCode Extension

* **breaking** Removed.

# 0.1.0

* Initial release.
