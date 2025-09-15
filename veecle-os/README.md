# veecle-os

Veecle OS is an actor-based runtime framework designed for building reliable, concurrent systems across diverse platforms.

At its core, Veecle OS provides a programming model where independent actors communicate through type-safe channels.
The framework abstracts platform differences through its Operating System Abstraction Layer (OSAL).
This allows the same application logic to run on microcontrollers or full Linux environments.

## Key Concepts

### Actor-Based Architecture

Applications in Veecle OS are composed of asynchronous actors - independent units of computation that communicate exclusively through message passing.

### Type-Safe Communication

Actors communicate using statically-typed `Reader`s and `Writer`s.
The type system guarantees that only compatible data types can be exchanged between actors, catching mismatches at compile time rather than runtime.

### Platform Abstraction

The OSAL layer provides consistent APIs for system resources regardless of the underlying platform.
Whether your application needs networking, timers, or logging, the same interface works across all supported platforms.

## Components

### Runtime

The runtime module provides the core actor-based programming model.
Actors communicate through `Reader` and `Writer` types in an asynchronous, type-safe manner.

### Operating System Abstraction Layer (OSAL)

The OSAL provides platform-agnostic APIs for system resources like networking, time, and logging.
Multiple implementations are available through feature flags:

- **Standard**: Implementation for standard Rust environments with `std` support.
- **Embassy**: Implementation for Embassy async embedded framework.
- **FreeRTOS**: Implementation for FreeRTOS-based embedded systems.

### Telemetry

The telemetry module provides observability features including structured logging, distributed tracing, and metrics collection.
It supports both `std` and `no_std` environments with zero-cost abstractions when disabled.

### Data Support

Data support modules provide serialization, deserialization, and protocol handling for communication standards:

- **CAN**: Support for Controller Area Network messages, including frame handling and DBC file integration.
- **SOME/IP**: Support for serializing and deserializing data for the Scalable service-Oriented MiddlewarE over IP protocol.

## Features

The following feature flags enable specific functionality:

- `alloc` - Enable allocation support for telemetry.
- `osal-std` - Enable the standard library OSAL implementation.
- `osal-embassy` - Enable the Embassy OSAL implementation.
- `osal-freertos` - Enable the FreeRTOS OSAL implementation.
- `telemetry` - Enable telemetry collection and export support.
- `telemetry-enable` - Enable telemetry and activate collection (for binary crates).
- `data-support-can` - Enable CAN protocol support.
- `data-support-someip` - Enable SOME/IP protocol support.

## Examples

See the [repository](https://github.com/veecle/veecle-os/tree/main/veecle-os-examples) for examples.

## Platform Support

Veecle OS supports multiple platforms through its OSAL implementations:

- **Embedded Systems**: Use `osal-freertos` or `osal-embassy` for microcontroller targets.
- **Linux**: Use `osal-std` for desktop and server applications.

## Documentation

For more detailed information about specific components:

- Runtime documentation: See the [`veecle-os-runtime`][veecle-os-runtime-docs] crate.
- OSAL documentation: See the [`veecle-osal-api`][veecle-osal-api-docs] and implementation crates.
- Telemetry documentation: See the [`veecle-telemetry`][veecle-telemetry-docs] crate.
- Data support documentation: See the respective data support crates.

[veecle-os-runtime-docs]: https://docs.rs/veecle-os-runtime

[veecle-osal-api-docs]: https://docs.rs/veecle-osal-api

[veecle-telemetry-docs]: https://docs.rs/veecle-telemetry
