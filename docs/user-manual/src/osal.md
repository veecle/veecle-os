{{#include ../../../target/rustdoc_index.md}}

# OSAL

Building Veecle applications that can run on different operating systems requires the framework to define an API actors can rely on.
To achieve this, the framework provides an Operating System Abstraction Layer (OSAL), enabling to build OS agnostic Veecle applications and reuse its components across different environments.

## Crates

The Veecle OSAL is structured in two layers:

### Definitions

The [`veecle_os::osal::api`][`mod@veecle_os::osal::api`] module defines a collection of traits and data structures that abstract over functionality typically provided by the underlying operating system.

This functionality is meant for building actors and applications regardless of where they will run.
This ensures the components remain as decoupled as the framework, being portable across different environments.

### Implementations

If `veecle_os::osal::api` is the specification layer, the other modules within `veecle_os::osal` are the implementations for the different environments.

The available implementations include:

- [`veecle_os::osal::std`][`mod@veecle_os::osal::std`] for platforms supported by the Rust Standard Library.
- [`veecle_os::osal::freertos`][`mod@veecle_os::osal::freertos`] for FreeRTOS.
- [`veecle_os::osal::embassy`][`mod@veecle_os::osal::embassy`] for Embassy.

These are the environments supported by Veecle, but users may build their own implementations as needed.

OSAL implementations are meant for specifying the target platform during the runtime's construction.
This is usually done in the corresponding `main.rs` file.
The components of Veecle applications should never rely on concrete implementations, but on the [Definitions](#definitions) instead.

See the [time documentation][`mod@veecle_os::osal::api::time`] for a practical example.
