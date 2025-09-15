# `veecle-telemetry-ui`

A GUI application to visualize Veecle OS telemetry data.

![](/veecle-telemetry-ui/screenshot.jpg)

## Running

### Native

```shell
cargo run --package veecle-telemetry-ui
```

### Web (WASM)

Install `trunk`: https://github.com/trunk-rs/trunk
(e.g., `cargo binstall trunk`)

```shell
trunk serve --open
```

## Usage

### Trace Files

1. Write raw `veecle-telemetry` output to file. Example:
    ```sh
    cargo run --package veecle-telemetry-ui --example remote > spans.jsonl
    ```
2. Open the file with `veecle-telemetry-ui` via `File > Open`
   > Note: The native binary will look for a file called `spans.jsonl` in the CWD and open it by default.
   - Alternatively you can also drag and drop a file onto `veecle-telemetry-ui`.

### WebSocket Connection

1. Pipe telemetry data into `veecle-telemetry-server`.
    ```shell
    cargo run --package veecle-telemetry-ui --example remote | cargo run --package veecle-telemetry-server
    ```
2. Connect to the WebSocket in `veecle-telemetry-ui` via `File > Connect`

### Pipe

> Note: Only available for the native build.

1. Pipe telemetry data into `veecle-telemetry-ui`.
    ```shell
    cargo run --package veecle-telemetry-ui --example remote | cargo run --package veecle-telemetry-ui
    ```
2. `veecle-telemetry-ui` will automatically parse and display the data as it comes in.
