# veecle-telemetry-server-protocol

Message definition for the WebSocket tracing data protocol.

## Overview

This crate defines the protocol messages used for communication between the telemetry server and UI.
It provides the data structures for serializing and deserializing telemetry data over WebSocket connections.

**Note**: Most users should depend on the [`veecle-os`](https://crates.io/crates/veecle-os) crate instead of using this crate directly.
The `veecle-os` crate re-exports this functionality and provides a more complete API for building Veecle OS applications.

For examples and more detailed usage information, please refer to the [repository](https://github.com/veecle/veecle-os).
