# Next

## Veecle OS

* **breaking** Removed `InitializedReader`.
  Use `Reader` with the new `read_updated` method instead.
  * `reader.wait_init().await` is no longer needed.
  * `reader.wait_for_update().await.read(|v| ...)` becomes `reader.read_updated(|v| ...).await`.
  * `reader.wait_for_update().await.read_cloned()` becomes `reader.read_updated_cloned().await`.
* **breaking** `Reader::read` and `ExclusiveReader::read` now take `&mut self` instead of `&self`.
* Added `read_updated` and `read_updated_cloned` methods to `Reader` and `ExclusiveReader`.
* Added `is_updated` method to `Reader`, `ExclusiveReader` and `CombineReaders` to check if an unseen value is available.
* Added `take_updated` method to `ExclusiveReader` to wait for an unseen value and take it.
* **breaking** The `execute!` macro no longer takes the `store` parameter.
  The `Storable` types used by the actors are now detected automatically.
* **breaking** Replaced `core::convert::Infallible` with custom `Never` enum for actor return types.
  * All actors using `Result<Infallible, E>` must change to `Result<Never, E>`.
  * All actors using bare `Infallible` must change to bare `Never`.
  * Import `Never` from `veecle_os::runtime::Never` or `veecle_os_runtime::Never`.
* **breaking** The `Storable` macro no longer takes a `data_type` attribute.
  * `#[storable(data_type = "Type")] struct MyType;` becomes `impl Storable for MyType { type DataType = Type; }`.
* **breaking** The `Storable` macro now requires a crate path without quotes.
  * `#[storable(crate = "::path")]` becomes `#[storable(crate = ::path)]`.
* **breaking** Telemetry functionality is now always compiled into the runtime.
  The `veecle-telemetry` feature flag has been removed from `veecle-os-runtime` and `telemetry` feature flag has been removed from `veecle-os`.
  Use the `telemetry-enable` feature flag on `veecle-os` to control telemetry behavior.
* **breaking** Removed `Span::root` method and `root_span!` macro; root spans should use `Span::new` and `span!` instead.
* **breaking** Replaced `SpanContext::from_span` with `Span::context` method.
* **breaking** Telemetry execution id is replaced by separate thread and process ids to uniquely identify thread/task combinations.
* **breaking** `ConsoleJsonExporter` is no longer a unit struct, replace usage with `ConsoleJsonExporter::DEFAULT`.
* **breaking** Removed `Writer::read` method.
* **breaking** `Writer::modify` closure must now return `bool` indicating whether the value was modified.
  Readers are only notified when the closure returns `true`.
* **breaking** `Reader`, `Writer`, and `ExclusiveReader` types are now exported from the `single_writer` module.
  * `use veecle_os_runtime::Reader` becomes `use veecle_os_runtime::single_writer::Reader`.
  * `use veecle_os::runtime::Writer` becomes `use veecle_os::runtime::single_writer::Writer`.
* Added custom serialization for telemetry ids with hex-encoded string format.
* Added `ThreadAbstraction` trait to OSAL for querying current thread id.
* Updated MSRV to 1.93.
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
