# `veecle-orchestrator`

Veecle OS Orchestrator is a process manager for running multiple Veecle OS Runtime Instances (and eventually coordinating communication between them and other networked devices).

See the [Distributed Runtime SDD] for a bit more of an overview of the overall system design this fits into.

[Distributed Runtime SDD]: https://docs.google.com/document/d/1YaQgp74SfKklIBk3xiWg43np5_M8Ussjb67ckaPsDQk/edit?tab=t.0#heading=h.awa8ckheuyd

## Usage

In one terminal run the orchestrator:

```console
> export VEECLE_ORCHESTRATOR_SOCKET=$XDG_RUNTIME_DIR/veecle-orchestrator.socket
> cargo run -p veecle-orchestrator
2025-04-03T16:27:10.079509Z  INFO run: veecle_orchestrator::listener: listening socket="/run/user/1000/veecle-orchestrator.socket"
```

You can use the `VEECLE_ORCHESTRATOR_LOG` environment variable to control the log level.
Refer to [`EnvFilter` directives](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives) to learn how to use this variable.

Then in another terminal use the `veecle-os-cli` to interact with it:

```console
> export VEECLE_ORCHESTRATOR_SOCKET=$XDG_RUNTIME_DIR/veecle-orchestrator.socket
> cargo run -p veecle-orchestrator-cli -- version
server version: 0.1.0

# Previously built with `(cd veecle-os-examples/std && cargo build --bin ping_pong)`, any Veecle OS Runtime based binary should be usable.
> cargo run -p veecle-orchestrator-cli -- runtime add veecle-os-examples/std/target/debug/ping_pong
added instance 0195fc7b-33e6-70e3-bee1-ac515185fac7

> cargo run -p veecle-orchestrator-cli -- runtime start 0195fc7b-33e6-70e3-bee1-ac515185fac7
started instance 0195fc7b-33e6-70e3-bee1-ac515185fac7

> cargo run -p veecle-orchestrator-cli -- runtime stop 0195fc7b-33e6-70e3-bee1-ac515185fac7
stopped instance 0195fc7b-33e6-70e3-bee1-ac515185fac7

> cargo run -p veecle-orchestrator-cli -- runtime remove 0195fc7b-33e6-70e3-bee1-ac515185fac7
removed instance 0195fc7b-33e6-70e3-bee1-ac515185fac7
```

You should see logs from these actions in the orchestrator as well

```console
2025-04-03T16:27:51.615263Z  INFO run:connection: veecle_orchestrator::listener: handling new connection connection.id=0
2025-04-03T16:27:51.615443Z  INFO run:connection:handle_request: veecle_orchestrator::listener: processed request connection.id=0 request.parsed=Version response=Ok("0.1.0")
2025-04-03T16:28:07.526713Z  INFO run:connection: veecle_orchestrator::listener: handling new connection connection.id=1
2025-04-03T16:28:07.526874Z  INFO run:connection:handle_request: veecle_orchestrator::listener: processed request connection.id=1 request.parsed=Add(Instance { id: InstanceId(0195fc7b-33e6-70e3-bee1-ac515185fac7), path: "/home/wim/.cache/cargo/target/shared/x86_64-unknown-linux-gnu/debug/ping_pong" }) response=Ok(())
2025-04-03T16:28:18.778875Z  INFO run:connection: veecle_orchestrator::listener: handling new connection connection.id=2
2025-04-03T16:28:18.779490Z  INFO run:connection:handle_request: veecle_orchestrator::listener: processed request connection.id=2 request.parsed=Start(InstanceId(0195fc7b-33e6-70e3-bee1-ac515185fac7)) response=Ok(())
2025-04-03T16:28:23.130329Z  INFO run:connection: veecle_orchestrator::listener: handling new connection connection.id=3
2025-04-03T16:28:23.131090Z  INFO run:connection:handle_request:stop: veecle_orchestrator::conductor: child stop exit status Ok(ExitStatus(unix_wait_status(2))) connection.id=3 request.parsed=Stop(InstanceId(0195fc7b-33e6-70e3-bee1-ac515185fac7)) id=InstanceId(0195fc7b-33e6-70e3-bee1-ac515185fac7)
2025-04-03T16:28:23.131157Z  INFO run:connection:handle_request: veecle_orchestrator::listener: processed request connection.id=3 request.parsed=Stop(InstanceId(0195fc7b-33e6-70e3-bee1-ac515185fac7)) response=Ok(())
2025-04-03T16:28:26.516099Z  INFO run:connection: veecle_orchestrator::listener: handling new connection connection.id=4
2025-04-03T16:28:26.516221Z  INFO run:connection:handle_request: veecle_orchestrator::listener: processed request connection.id=4 request.parsed=Remove(InstanceId(0195fc7b-33e6-70e3-bee1-ac515185fac7)) response=Ok(())
```

### Linking example

You can use the binaries from `veecle-os-examples/orchestrator-ipc` to test the IPC linking:

```console
# Add the ipc examples to the orchestrator, recording their ids for later use.
$ cargo run -p veecle-orchestrator-cli runtime add .../ping
$ ping=...the instance id...
$ cargo run -p veecle-orchestrator-cli runtime add .../pong
$ pong=...the instance id...

# Define the links between the instances to route their data.
$ cargo run -p veecle-orchestrator-cli link add --from $ping --type veecle_os_examples_common::actors::ping_pong::Ping --to $pong
$ cargo run -p veecle-orchestrator-cli link add --from $pong --type veecle_os_examples_common::actors::ping_pong::Pong --to $ping

# In other terminals you can view the instance's logs.
$ cargo run -p veecle-orchestrator-cli runtime stdout $ping
$ cargo run -p veecle-orchestrator-cli runtime stdout $pong

# Then start the instances.
$ cargo run -p veecle-orchestrator-cli runtime start $pong
$ cargo run -p veecle-orchestrator-cli runtime start $ping
```

The `veecle-os-examples/orchestrator-ipc/run.sh` script will perform this whole process for you, running the binaries within a pair of orchestrators.
