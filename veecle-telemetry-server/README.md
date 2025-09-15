# veecle-telemetry-server

Sends tracing data piped in over a WebSocket connection to `veecle-telemetry-ui`.

## Overview

This crate provides a telemetry server that receives tracing data from Veecle OS applications and forwards it to the telemetry UI.
It acts as a bridge between instrumented applications and the visualization interface.

**Note**: Most users should depend on the [`veecle-os`](https://crates.io/crates/veecle-os) crate instead of using this crate directly.
The `veecle-os` crate re-exports this functionality and provides a more complete API for building Veecle OS applications.

For examples and more detailed usage information, please refer to the [repository](https://github.com/veecle/veecle-os).
