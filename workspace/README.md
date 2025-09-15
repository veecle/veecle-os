# `workspace`

This crate contains tests that perform validation such as running `cargo fmt --check`.

To view the full list of tests, run the following command:

```console
cargo nextest list -p workspace
```

Frequently, projects define multiple separate processes to run as part of validation, such as running tests, checking code formatting, and others.
These require developers to execute different processes for each kind of validation, and for automated validation systems to have additional different configuration for each kind of validation.

By integrating other kinds of validation as tests, the test runner becomes a single validation process.

Additionally, the workspace crate provides Rust project scaffolding, so each new validation process requires minimal implementation overhead and can use the Rust ecosystem facilities to work with Rust code.
