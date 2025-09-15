# veecle-osal-freertos

Veecle OS operating system abstraction layer implementation for [FreeRTOS](https://www.freertos.org/).

## Overview

This crate provides the FreeRTOS implementation of the Veecle OS operating system abstraction layer API.
It implements the traits defined in `veecle-osal-api`.

**Note**: Most users should depend on the [`veecle-os`](https://crates.io/crates/veecle-os) crate instead of using this crate directly.
The `veecle-os` crate re-exports this functionality and provides a more complete API for building Veecle OS applications.

For examples and more detailed usage information, please refer to the [repository](https://github.com/veecle/veecle-os).

## Tests

All tests are written using the FreeRTOS POSIX port.

Tests are only guaranteed to be sound using ports where the following holds true:

- `vTaskEndScheduler` is available and has not further requirements on the caller.
- `taskYIELD` is interrupt-safe.

### Adding new tests

New tests should be added as separate integration tests in [tests](tests).
Each test must be placed in a separate file to ensure one test per binary.

Starting and stopping the FreeRTOS scheduler from multiple tests in parallel leads to interference between the tests.
The FreeRTOS memory allocator also interacts with the scheduler globals so it must not be used in a multi-threaded binary.
Because of that, integration tests are used where each file in the [tests](tests) directory will be compiled as a separate binary.

Every test must include `pub mod common;`.
Marking the module as `pub` avoids Clippy warnings about unused code for common functionality not used by the specific test.
While `pub mod common;` allows access to shared functionality, the main reason is to use the FreeRTOS-allocator as the global allocator.
