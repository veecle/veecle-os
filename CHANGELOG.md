# Next

## Veecle OS

* **breaking** The `Storable` macro now requires type paths and crate paths without quotes.
  * `#[storable(data_type = "Type")]` becomes `#[storable(data_type = Type)]`.
  * `#[storable(crate = "::path")]` becomes `#[storable(crate = ::path)]`.
* **breaking** Telemetry functionality is now always compiled into the runtime.
  The `veecle-telemetry` feature flag has been removed from `veecle-os-runtime` and `telemetry` feature flag has been removed from `veecle-os`.
  Use the `telemetry-enable` feature flag on `veecle-os` to control telemetry behavior.
* **breaking** Removed `Span::root` method and `root_span!` macro; root spans should use `Span::new` and `span!` instead.
* **breaking** Replaced `SpanContext::from_span` with `Span::context` method.
* **breaking** Telemetry execution id is replaced by separate thread and process ids to uniquely identify thread/task combinations.
* **breaking** `ConsoleJsonExporter` is no longer a unit struct, replace usage with `ConsoleJsonExporter::DEFAULT`.
* Added custom serialization for telemetry ids with hex-encoded string format.
* Added `ThreadAbstraction` trait to OSAL for querying current thread id.
* Updated MSRV to 1.92.
* Fixed `veecle_os::telemetry::instrument` macro to automatically resolve correct crate paths for the facade.
* Implemented `stable_deref_trait::StableDeref` for `Chunk` to allow usage in `yoke`.

## Veecle Telemetry

* **breaking** Telemetry protocol types (`InstanceMessage`, `TelemetryMessage`, `LogMessage`, etc.) are now generic over value types to support formatting in `no_std` environments.
  Use `transient::*` type aliases for local telemetry operations (supports `format_args!`) and `owned::*` aliases for deserialization and cross-thread communication.
  The `Export` trait now accepts `transient::InstanceMessage<'_>` instead of `InstanceMessage<'_>`.
* Added `ConsolePrettyExporter` for pretty printed telemetry output for non-production use-cases.

## Veecle Telemetry VSCode Extension

* **breaking** Removed.

## Veecle OS Data Support SOME/IP

* **breaking** Change return type of `veecle_os_data_support_someip::serialize::SerializeExt::serialize` to match its documentation.
* Add `serialize_with_serializable` to `veecle_os_data_support_someip::header::Header` to allow serializing without intermediate buffer.

## Veecle OSAL API

* **breaking** Updated `embedded-io*` to version `0.7`.

## Veecle OSAL Embassy

* **breaking** Updated `embassy-net` to version `0.8.0`.

# 0.1.0

* Initial release.
