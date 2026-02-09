{{#include ../../../target/rustdoc_index.md}}

# Getting started

This document describes how to create and run a minimal Veecle OS application.

## Setting up a package

Create an empty package with `cargo new` and change to the package directory:

```
cargo new foo
cd foo
```

By structuring Veecle OS programs in a specific way, you can write Veecle OS programs that run on multiple platforms.
Because this example runs only on platforms that provide the Rust `std` library, the example uses a smaller structure that is not suitable for Veecle OS programs that run on multiple platforms.
(Refer to other examples to learn the structure for Veecle OS programs that can run on multiple platforms.)

Run the following commands to add the Veecle OS framework crate with the `std` operating system abstraction layer (OSAL):

```
cargo add veecle-os --features osal-std
```

## Writing Veecle OS code

Veecle OS programs contain actors.
Actors read inputs and write outputs using runtime channels.
The runtime stores marked Rust types.
Actors run concurrently by using async programming.

This example contains two actors:

* `sender_actor` writes a `Value`, then loops.
* `receiver_actor` waits to read a `Value`, prints the `Value`, then loops.

### Defining the storable types

First, define the `Value` type by copying the following code into `src/main.rs`:

```rust
{{#include ../crates/getting-started/src/main.rs:init}}
```

By implementing [`Storable`][`trait@veecle_os::runtime::Storable`] with the `data_type` `u32`, `Value` becomes an identifier for readers and writers.
Any reader or writer that uses `Value` will read or write the `data_type`, in this case `u32`.
The `data_type` must [implement the `Debug` trait](https://doc.rust-lang.org/stable/core/fmt/trait.Debug.html) so that logs can show its debug representation.

`Storable` can also be implemented manually.
See the [`veecle_os::runtime::Storable`][`trait@veecle_os::runtime::Storable`] documentation for more information.

### Implementing actors

You can implement actors in multiple ways.
This example uses the [`actor` macro][`attr@veecle_os::runtime::actor`], that defines an actor from a Rust function.

In this example, the actors use [`Reader`s][`struct@veecle_os::runtime::Reader`] and [`Writer`s][`struct@veecle_os::runtime::Writer`] to communicate with other actors.

Copy the following actors into `src/main.rs`:

```rust
{{#include ../crates/getting-started/src/main.rs:sender}}
```

```rust
{{#include ../crates/getting-started/src/main.rs:receiver}}
```

`receiver_actor` uses `std::process::exit` to exit the program when it receives 5.

Actors should never terminate.
In case of unrecoverable errors, actors should return an error.
This program ends so it always generates the same short output that is easy to verify.

### Implementing the Veecle OS program

Currently, Veecle OS programs have an entry point implemented according to the platform that the Veecle OS program runs on.
`std` Veecle OS programs require Tokio as their async runtime.
This is set up by the `veecle_os::osal::std::main` macro.

Copy the following code into `src/main.rs` to implement a Veecle OS program entry point:

```rust
{{#include ../crates/getting-started/src/main.rs:main}}
```

The `main` function uses [`veecle_os::runtime::execute!`][`macro@veecle_os::runtime::execute`] to create and execute a runtime instance that knows about the `Value` datatype and both actors.

## Running the Veecle OS program

(See the end of this document for a complete listing of `src/main.rs`.)

To execute the Veecle OS program, run the following command:

```
cargo run
```

## Appendix: the complete Veecle OS program

```rust
{{#include ../crates/getting-started/src/main.rs:full}}
```
