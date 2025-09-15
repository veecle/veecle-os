# Veecle OS

Veecle OS is a programming framework that enables developers to write software for low-power embedded devices and high-powered systems alike.
Veecle OS uses features from the Rust programming language to help software developers avoid common complexities.

<!-- TODO:

Refer to:

* [User manual](https://user-manual-2bd.pages.dev/)
* [Private docs](https://private-docs-3bh.pages.dev/)

-->

Refer to [CONTRIBUTING](CONTRIBUTING.md) for build instructions and other development material.
After completing the setup instructions, go to [the examples](veecle-os-examples/) to run some Veecle OS example programs.

## Repository structure

<!-- TODO: to be removed/renamed?

nos-ui-vscode-extension
veecle-os-cli

-->

* [`docs`](docs/): source for the documentation.
* [`veecle-os`](veecle-os/): the main Veecle OS facade, exposing various components such as the runtime and OSAL.
* [`veecle-os-examples`](veecle-os-examples/): example code that uses Veecle OS.
  Check this directory to get started running some Veecle OS code.
* [`veecle-os-runtime`](veecle-os-runtime/) and [macros](veecle-os-runtime-macros): the Veecle OS runtime library with basic infrastructure, such as the store implementation.
* [`veecle-os-test`](veecle-os-test/): tools for testing Veecle OS actors.
* `veecle-telemetry-*`: a telemetry library for collecting and exporting observability data for Veecle OS code.
* [`veecle-osal-api`](veecle-osal-api/), `veecle-osal-*`: code to support running Veecle OS on different platforms.
* `veecle-os-data-support-*` and `*-someip-*`: code to support different data formats and transports, such as CAN.
* [`workspace`](workspace/): validation support.
* [`.vale`](.vale/): configuration for [Vale](https://vale.sh/), a prose linter for code and documentation.
* [`supply-chain`](supply-chain/): configuration for [`cargo-vet`](https://mozilla.github.io/cargo-vet/).
* [`external`](external/): code from external projects.
  The Veecle OS repository includes code from other projects, so that developers can make changes across repositories in a single commit.
* [`veecle-orchestrator`](veecle-orchestrator/), `veecle-orchestrator-*`, [`veecle-ipc`](veecle-ipc/), `veecle-ipc-*`: multi-runtime orchestrator prototype.
  Unpublished while it's still a prototype.

## License

This project is licensed under the [Apache License Version 2.0](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you shall be licensed as Apache License Version 2.0 without any additional terms or conditions.
