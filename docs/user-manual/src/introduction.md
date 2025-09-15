{{#include ../../../target/rustdoc_index.md}}

# Introduction

Veecle OS is a programming framework that enables developers to write software for low-power embedded devices and high-powered systems alike.
Veecle OS uses features from the Rust programming language to help software developers avoid common complexities.
For example:

* With the actor model Veecle OS provides, developers can create concurrent programs on single-core processors.
* Veecle OS provides an operating system abstraction layer (OSAL) so that Veecle OS programs can run on different platforms, such as embedded devices or Unix derivatives.

Because Veecle OS programs are Rust programs, they gain some of the advantages that Rust provides:

* By confining unsafe operations to unsafe sections, memory and thread safety problems can only occur in restricted areas of the program, reducing review effort.

<!-- TODO: how to describe the other advantages of Rust vs. C/C++ in an objective way without saying "the tools and environment are nicer". -->

By using Veecle OS, a developer can start prototyping a program on their computer without the difficulties of developing on an embedded device.
As their program takes shape, the developer can add any necessary adaptations required for their embedded device target.
The developer can develop and troubleshoot many issues inside a full development environment, and only address embedded problems in the more limited embedded development environment.

<!-- TODO: suitability for light and heavy workloads -->

Veecle OS also provides other features:

<!-- TODO: cloud development, model-driven, ... -->

* Veecle OS provides [code](telemetry.md) that enables developers to collect structured telemetry in all environments.
  With `veecle-telemetry`, the Veecle OS framework and Veecle OS programs can record structured events with additional information about temporality and causality.
  Veecle OS provides tools to inspect traces.

* Veecle OS provides [utilities for writing tests for Veecle OS programs](testing.md).

* Veecle OS provides libraries for data formats and communication protocols such as:

  * [CAN][`mod@veecle_os::data_support::can`]
  * [SOME/IP][`mod@veecle_os::data_support::someip`] (work in progress)

* Veecle OS provides OSALs for platforms such as:

  * [`std`][`mod@veecle_os::osal::std`] for operating systems supported by Rust, such as Linux or macOS.
  * [FreeRTOS][`mod@veecle_os::osal::freertos`] for [FreeRTOS](https://www.freertos.org/).
  * [Embassy][`mod@veecle_os::osal::embassy`] for [Embassy](https://embassy.dev/).

Refer to [getting started](./getting-started.md) for a guide to write a minimal Veecle OS program.
[Framework](./framework.md) provides an overview over the Veecle framework.

Refer to [the Rust documentation for the `veecle-os` crate][`crate@veecle_os`] for further details on Veecle OS.
