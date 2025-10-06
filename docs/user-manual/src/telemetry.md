{{#include ../../../target/rustdoc_index.md}}

# Telemetry

Veecle OS has features to learn about the execution of Veecle OS applications.

The [`veecle-telemetry` crate](crate@veecle-telemetry) provides functions to add events and spans to Rust programs.

The Veecle OS crates themselves are instrumented, and you can instrument your code with the `veecle_os::telemetry` macros.
Veecle OS applications can use a [collector][`fn@veecle_os::telemetry::collector::set_exporter`] to define where these events and spans are sent.

For example, with a Veecle OS application that emits serialized traces, you can use [`probe-rs`](https://probe.rs/) to run the application on an embedded device and pipe the serialized data for further processing.

## Configuring applications to emit serialized traces

For `std` applications, annotate your `main` function as follows:

```rust
{{#include ../crates/traces-serialization/src/main.rs:setup}}
```

The [`main`][`attr@veecle_os::osal::std::main`] macro configures `veecle_os::telemetry` to send traces as logs to the log target implementation from the `std` OSAL.
This log target prints logs to the standard output.
The result is that the Veecle OS application prints serialized traces to standard output.

<!-- Skipping other adapters because we do not focus on them at the moment -->

## Viewing serialized traces

See [installing the `veecle-telemetry-ui` graphical telemetry viewer](/crates-and-tools.md#installing-the-veecle-telemetry-ui-graphical-telemetry-viewer).

If `cargo run ...` starts a program that emits serialized traces, then you can pipe the traces into `veecle-telemetry-ui` to read and display telemetry in real time:

```
cargo run ... | veecle-telemetry-ui
```

Alternatively, if `x.trace.jsonl` contains serialized traces, then you can use the `veecle-telemetry-ui`to visualize the telemetry:

```
veecle-telemetry-ui x.trace.jsonl
```

<!-- TODO: Should we document `veecle-telemetry-server/`? -->
