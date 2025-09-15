# veecle-os-runtime

This crate contains the core runtime for Veecle OS applications.
It provides the actor-based programming model where applications are composed of actors that communicate using Reader and Writer types.

**Note**: Most users should depend on the [`veecle-os`](https://crates.io/crates/veecle-os) crate instead of using this crate directly.
The `veecle-os` crate re-exports this functionality and provides a more complete API for building Veecle OS applications.

For examples and more detailed usage information, please refer to the [repository](https://github.com/veecle/veecle-os).

## Features

- `veecle-telemetry` - Enables telemetry support for runtime instrumentation.
