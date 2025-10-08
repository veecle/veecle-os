# Next

## Veecle OS

* **breaking** Telemetry execution id now includes thread id to uniquely identify thread/task combinations.
* Added `ThreadAbstraction` trait to OSAL for querying current thread id.
* Fixed `veecle_os::telemetry::instrument` macro to automatically resolve correct crate paths for the facade.

## Veecle Telemetry VSCode Extension

* **breaking** Removed.

# 0.1.0

* Initial release.
