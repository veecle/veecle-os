# Examples

Each example is isolated in its own crate.
There may be specific cargo configuration required for each example, so you must compile them individually within their sub-directories.

### freertos-linux

Will compile and run on most host toolchains. Tested with:

- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-gnu`
- `aarch64-apple-darwin`

```bash
cd freertos-linux
cargo run --bin basic
cargo run --bin tracing | cargo run --manifest-path ../../Cargo.toml --bin veecle-os -- tracing
```

### freertos-stm32

The runner is configured in the workspace to be [probe-rs](https://probe.rs/docs/tools/probe-rs/).

The default board is `STM32-F767ZI`. To flash on a different board you can either override the configuration in `./cargo/config.toml` or the environment variable `PROBE_RS_CHIP`.

**Right now the example only works for `STM32F767`: to support different boards the code must be extended.**

To compile:

```bash
cd freertos-stm32
cargo build
```

To run on the real hardware:

```bash
[ PROBE_RS_PROBE=... ] cargo run --bin time
[ PROBE_RS_PROBE=... ] cargo run --bin tracing | ( cd ../.. && cargo run --bin veecle-os -- tracing )
```

### std

Targeting rust-std.

```bash
cd std
cargo run --bin ping_pong
cargo run --bin tracing | cargo run --manifest-path ../../Cargo.toml --bin veecle-os -- tracing
```

### embassy

#### std

Targeting rust-std.

```bash
cd embassy-std
cargo run
```

#### STM32

The runner is configured in the workspace to be [probe-rs](https://probe.rs/docs/tools/probe-rs/).

The default board is `STM32-F767ZI`.
To flash on a different board you can either override the configuration in `./cargo/config.toml` or the environment variable `PROBE_RS_CHIP`.
You also need to adapt the features in `Cargo.toml` to match the board.

```bash
cd embassy-stm32
cargo run
```


#### With IPC

The `ipc-*` binaries are designed to run as a combined system under the `veecle-orchestrator`.
See `veecle-orchestrator/README.md` for more details.

### docs

These are standalone examples that are currently being used by our docs, targeting `std`.
