# veecle-telemetry

Veecle OS telemetry.

## Overview

This crate provides telemetry collection and export capabilities for Veecle OS applications.
It supports distributed tracing, structured logging, and metrics collection with support for both `std` and `no_std` environments.

**Note**: Most users should depend on the [`veecle-os`](https://crates.io/crates/veecle-os) crate instead of using this crate directly.
The `veecle-os` crate re-exports this functionality and provides a more complete API for building Veecle OS applications.

For examples and more detailed usage information, please refer to the [repository](https://github.com/veecle/veecle-os).

## Features

- `enable` - Enable collecting and exporting telemetry data, should only be set in binary crates.
- `std` - Enable standard library support.
- `alloc` - Enable allocator support for dynamic data structures.
- `freertos` - Enable FreeRTOS support.
- `system_time` - Enable system time synchronization.
