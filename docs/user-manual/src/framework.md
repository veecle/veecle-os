# Framework

The Veecle framework provides a runtime and supporting tools for developing event-driven applications.

## Runtime

The runtime provides the means to build applications using an [actor model].
This allows the decomposition of functionality into multiple actors communicating via messages.
Messages are exchanged between actors via an integrated central communication hub of the application.
With the operating system abstraction layer (OSAL) developers can create OS-agnostic code.
This allows moving applications and actors between different target operating systems with minimal changes to the application.
An instance of the runtime provides the communication hub, and an executor to drive the actors.

[actor model]: https://en.wikipedia.org/wiki/Actor_model

### Actors

Actors are the basic building blocks of an application.
They can be reused across projects and composed to build complex applications.
An actor's interface is made up of message readers and writers and some optional initialization context.
The readers and writers are used to communicate between actors.
The initialization context allows passing data into an actor that might have to be set up before the executor can take over control (e.g. shared memory, peripherals, etc.).

At their core, actors are asynchronous functions.
They wait for events to happen and react accordingly.
When an event that an actor is waiting for occurs, the actor gets scheduled by the executor.
Events can for example be a timer expiring or a new value being written by another actor.

In Rust, asynchronous functions are inherently lazy and only do work if they are driven by an executor.
In practice, this means actors are polled by the executor if an event they are waiting for occurs.
The only exception is on startup of the executor, when the executor polls every actor once.
This ensures every actor gets a chance to initialize without an event occurring.
The order in which the actors are polled in is fixed, but actors without work to do are skipped to reduce resource usage.
Actors are expected not to exit, but to either indefinitely process events or to report themselves as not having work to do (in Rust terms: pending).
On fatal errors, actors can return an error to indicate their status to the runtime.
Within a single instance of the runtime, all actors are driven concurrently by a single executor.

Rust offers the official [Rust book] and [Rust async book] for more in-depth information regarding asynchronous programming in Rust.

[Rust book]: https://doc.rust-lang.org/stable/book/ch17-00-async-await.html

[Rust async book]: https://rust-lang.github.io/async-book/intro.html

### Communication hub

The runtime provides a central communication hub between actors, a place where actors can store and retrieve messages from.
For each message type, the hub internally uses a slot.
The slot represents an allotted space for this message type.
Readers and writers transparently allow access to the message in the slot.
With every message type being associated with a single slot, every message type within the store has to be unique.
Multiple readers for the same type of message are automatically using a single shared slot.

There must always be exactly one writer per message type, producing messages for at least one reader.
This prevents accidental misconfiguration where slots are not being written to or read from.
At startup, the runtime validates its configuration and exits (in Rust terms panics) if an invalid configuration is detected.
All slots before an actor writes to them are initially set to `None`.

Each instance of the runtime provides a single communication hub to be shared between all actors of the instance.

### Operating System Abstraction Layer (OSAL)

The operating system abstraction layer (OSAL) provides OS-agnostic interfaces to interact with OS-provided functionality (e.g. time).
By abstracting over the concrete OS interface, actors can make use of OS-provided functionality without requiring OS-specific code.
This allows developing actors portable between supported target operating systems.

Check out the [OSAL chapter](./osal.md) for further details.

## Tooling

<!-- TODO: tools -->
