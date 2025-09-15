# veecle-osal-embassy

Veecle OS operating system abstraction layer implementation for [Embassy](https://github.com/embassy-rs/embassy).

## Overview

This crate provides the Embassy implementation of the Veecle OS operating system abstraction layer API.
It implements the traits defined in `veecle-osal-api`.

**Note**: Most users should depend on the [`veecle-os`](https://crates.io/crates/veecle-os) crate instead of using this crate directly.
The `veecle-os` crate re-exports this functionality and provides a more complete API for building Veecle OS applications.

For examples and more detailed usage information, please refer to the [repository](https://github.com/veecle/veecle-os).

## Testing

### Time

Tests rely on Embassy's mock driver implementation.
The mock driver is shared within a binary, so every test must be a separate binary.

### Networking

Tests are implemented using a custom "loopback"-like driver.
