# `supply-chain`

Currently our configuration reuses the `safe-to-run` and `safe-to-deploy` default policies.

The configuration uses `safe-to-deploy` for code that will run in "production" (e.g. `veecle-os-runtime`) and `safe-to-run` for code that won't (e.g. `veecle-telemetry-ui`).

## Tricks and tips

### Finding unnecessary vetting

`cargo vet` considers all crate dependencies, including Windows dependencies.
If a `safe-to-deploy` crate depends on a dependency such as `windows-sys`, then `windows-sys` must be audited for `safe-to-deploy`.
You can view this by running `cargo vet suggest` or reviewing the exemptions on `config.toml`.

In general, you might find dependencies that your code does not use, but that `cargo vet` requires vetting.
Finding the dependencies that cause this issue can be tedious.
This section shows some commands that can help.

`cargo tree --target all --invert $crate@$version` shows the dependency tree rooted on a specific version of an unwanted crate.
(Configuration changes must be done for each version of the unwanted crate.)
You can prune any `safe-to-run` crate that appears in the graph for clarity.
You can also exclude dev-dependencies.

```console
$ cargo tree --target all -i windows-sys@0.52.0 -e no-dev --prune veecle-telemetry-ui --prune veecle-ipc --prune veecle-orchestrator --prune veecle-telemetry-server --prune veecle-ipc-protocol
windows-sys v0.52.0
├── glutin v0.32.3
│   ├── eframe v0.31.1
│   └── glutin-winit v0.5.0
│       └── eframe v0.31.1 (*)
├── glutin_egl_sys v0.7.1
...
```

The first subtree ends in `eframe`, because `safe-to-run` crates have been pruned.
This subtree can be safely ignored.

```
├── mio v1.0.3
│   └── tokio v1.45.0
...
│       ├── veecle-osal-std v0.1.0 (/home/alex/git/veecle-os/veecle-osal-std)
...
│       ├── tokio-stream v0.1.17
│       │   ├── veecle-osal-std v0.1.0 (/home/alex/git/veecle-os/veecle-osal-std)
...
├── socket2 v0.5.9
│   ├── hyper-util v0.1.11 (*)
│   └── tokio v1.45.0 (*)
├── tokio v1.45.0 (*)
...
```

Note that `(*)` means that a subtree is duplicated and has been omitted from the output.

`mio`, `tokio` and `socket2` depend on `windows-sys`, and are depended by `veecle-osal-std`.

To make `windows-sys` only require `safe-to-run` auditing, add the following policies to `config.toml`:

```
[policy."mio:1.0.3"]
dependency-criteria = { windows-sys = [] }
notes = "We do not audit Windows for safe-to-deploy."

[policy."socket2:0.5.9"]
dependency-criteria = { windows-sys = [] }
notes = "We do not audit Windows for safe-to-deploy."

[policy."tokio:1.45.0"]
dependency-criteria = { windows-sys = [] }
notes = "We do not audit Windows for safe-to-deploy."
```

Run `cargo vet regenerate exemptions` to regenerate the exemptions and verify that `windows-sys:0.52.0` is only `safe-to-run`.
