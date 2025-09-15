# veecle-osal-std

Veecle OS operating system abstraction layer implementation for `std`.

## Overview

This crate provides the standard library (`std`) implementation of the Veecle OS operating system abstraction layer API.
It implements the traits defined in `veecle-osal-api` using Tokio and standard library features, suitable for desktop and server environments.

**Note**: Most users should depend on the [`veecle-os`](https://crates.io/crates/veecle-os) crate instead of using this crate directly.
The `veecle-os` crate re-exports this functionality and provides a more complete API for building Veecle OS applications.

For examples and more detailed usage information, please refer to the [repository](https://github.com/veecle/veecle-os).
