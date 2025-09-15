# `actions`

This directory contains reusable actions for workflows:

* `setup-arm-gnu-toolchain`
* `setup-cmake`
* `setup-fnm`
* `setup-miri`
* `setup-rustup`
* `setup-tools` (uses the [`cargo-quickinstall`](https://github.com/cargo-bins/cargo-quickinstall) that provides binaries of many tools implemented in Rust)

`setup-*` actions install tools required in workflows under `$RUNNER_TEMP` for better build isolation.
They add environment variables to the GitHub environment, including extending the `PATH` variable.
