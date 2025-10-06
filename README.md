# Veecle OS
[![version-badge][]][version] [![docs-badge][]][docs] [![msrv-badge][]][msrv] [![license-badge][]][license] [![validate-badge][]][validate] [![coverage-badge][]][coverage]

Veecle OS is a programming framework that enables developers to write software for low-power embedded devices and high-powered systems alike.
Veecle OS uses features from the Rust programming language to help software developers avoid common complexities.

Refer to the [user manual](https://veecle.github.io/veecle-os/user-manual/) to learn about Veecle OS usage.

Refer to [CONTRIBUTING](CONTRIBUTING.md) for build instructions and other development material.
After completing the setup instructions, go to [the examples](veecle-os-examples/) to run some Veecle OS example programs.

## Repository structure

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

<!-- Logo extracted from the smallest size in <https://crates.io/favicon.ico> and converted to `png`. -->
[version-badge]: https://img.shields.io/crates/v/veecle-os?style=flat-square&logo=image%2Fpng%3Bbase64%2CiVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAMAAAAoLQ9TAAAAIGNIUk0AAHomAACAhAAA%2BgAAAIDoAAB1MAAA6mAAADqYAAAXcJy6UTwAAAJwUExURQAAAOvCc%2BWyUuWyT%2Be1VOm7Y%2Be%2Fcue2V%2FHEb%2BGuSuSyUue6ZOi4W%2Bm5Xei2Vee0UOe2VtmgMOm5Xei3V%2Be2Vue6Y9aeMeKsQeWxS92nP%2BWvRuWrOeaxSOu7YNqiNOasOuCrQunFfua2WeWyUOe1VdyoQuatPN%2BrRNupRd2pQuKsQuCtSMyWK8%2BVJOKpOeStQsGLIMSKF9ukN%2BqzRbF9F7qADeGpOeexRX5fM5ZsJcaYTLmSXlpFL21UOHleOotkII5pOpt4UraSawAAABoUDQsIBjotIHpdOY9oK6NyFaR%2FUolmQXdXM3JWNwIBAQAAABENCUg4JTgsIEw5IZNtNYllOnpYNIhoRYdrSwAAAAAAACYdFHVbPqB%2BWX9eOmxTNue7Zee5X96oPuO0WeWyT%2Be0U%2Be3Wei7ZNujN%2BOuR%2Be2V%2Bi7Zei6Yee0Uue6Y%2Be7Y%2BKwTue9bOe2WOi0UOeySuSrPeauQNujNuCtSOCqQuavReKqO%2BGnNuatPearOOesN9GWItOZJdacLOGoOMmWMM2ZMueuP%2BmuO%2BivPuW4YOi8adilQseMGMqPHNCWJd2lN8OSMNOdNemwQemxROe2VtilQNynQNGcM8ONJMaQKMmPHeSqOd2mOeOsPuWvRuKrQNymOsiNGcuQHs2VKM2WKsiULcaPIuGqPeazUeKuRtSeNMORL76EEMCFE8WNHrqDGb%2BIHNCXKN%2BnOeWtPuWsOs6aM8mWMbSFLrmAEb%2BGFqp2EaZ0E8ePH%2BSqOOesOOesOeKpOtOfPat5HLF6DahzDMGIGeWrOOqxQNajRLSIQb6FFeKpOdWmUquDSLKJS%2F%2F%2F%2F1QhmtkAAABgdFJOUwAEK33X1HkmAlfh%2FPzbkD8MefK%2FYgd51hp5%2FtYbDZLWGwIpdsX21hsYyNcbHtnYHB3Z2Bwf2tgcL%2BblLx%2Bwzdz4tyIEHzh28vn697xZEQIKI2iBpvr0t1UPAxZKy8tdEFDhfOQAAAABYktHRM%2BD3sJpAAAAB3RJTUUH6QkRCQQ0UiO4XwAAAQBJREFUGNNjYAACRiZmFlY2dg4GGODk4k5I5OHl4xeACggmJaekpiWkCwmLiIIFxDIys7JzcrPy8sUlwAKSBYV5RcUlUqVl0jIgvqxceUVlVXVNbV29vAKDopKyimpDY1NzS2tbe4eaOoOGZmdXd09vX%2F%2BEiZMmT9HSZtDRnTpt%2BoyZs2bPmTtv%2FgI9fQYDw4WLFi9Zumz5ipWlq1YbGTOYmK5Zu279ho2bNm%2Bp3brNzJzBwnL7jp27du%2FZu29%2Fx4GDVtYMNrZ29ocOHzl67HjbiZMOjk4Mzi6ubu4enqdOnznr5e3jC3SWn39AYFBwyLnQsPAImG8jo6JjYuPiwWwAjKZWgqTCB7sAAAAldEVYdGRhdGU6Y3JlYXRlADIwMjUtMDktMTdUMDk6MDQ6NTIrMDA6MDCyCIoGAAAAJXRFWHRkYXRlOm1vZGlmeQAyMDI1LTA5LTE3VDA5OjA0OjUyKzAwOjAww1UyugAAACh0RVh0ZGF0ZTp0aW1lc3RhbXAAMjAyNS0wOS0xN1QwOTowNDo1MiswMDowMJRAE2UAAAAASUVORK5CYII%3D
[version]: https://crates.io/crates/veecle-os
[docs-badge]: https://img.shields.io/badge/docs.rs-veecle--os-teal?style=flat-square&logo=docs.rs
[docs]: https://docs.rs/veecle-os
[msrv-badge]: https://img.shields.io/crates/msrv/veecle-os?style=flat-square&logo=rust
[msrv]: ./rust-toolchain.toml
<!-- Logo is a generic "document" icon generated by Claude. -->
[license-badge]: https://img.shields.io/crates/l/veecle-os?style=flat-square&logo=image%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KICA8cGF0aCBkPSJNMTQgMkg2QzQuOSAyIDQgMi45IDQgNHYxNmMwIDEuMSAwLjkgMiAyIDJoMTJjMS4xIDAgMi0wLjkgMi0yVjhsLTYtNnoiIHN0cm9rZT0id2hpdGUiIHN0cm9rZS13aWR0aD0iMiIgZmlsbD0ibm9uZSIvPgogIDxwYXRoIGQ9Ik0xNCAydjZoNiIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLXdpZHRoPSIyIiBmaWxsPSJub25lIi8%2BCiAgPHBhdGggZD0iTTE2IDEzSDgiIHN0cm9rZT0id2hpdGUiIHN0cm9rZS13aWR0aD0iMiIvPgogIDxwYXRoIGQ9Ik0xNiAxN0g4IiBzdHJva2U9IndoaXRlIiBzdHJva2Utd2lkdGg9IjIiLz4KICA8cGF0aCBkPSJNMTAgOUg4IiBzdHJva2U9IndoaXRlIiBzdHJva2Utd2lkdGg9IjIiLz4KPC9zdmc%2B
[license]: ./LICENSE
[validate-badge]: https://img.shields.io/github/actions/workflow/status/veecle/veecle-os/.github%2Fworkflows%2Fvalidate.yaml?style=flat-square&logo=githubactions&logoColor=white
[validate]: https://github.com/veecle/veecle-os/actions/workflows/validate.yaml?query=branch%3Amain+event%3Apush
<!-- Uses an api request to get just the main component instead of the overall coverage of the `codecov` badge. -->
[coverage-badge]: https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.codecov.io%2Fapi%2Fv2%2Fgithub%2Fveecle%2Frepos%2Fveecle-os%2Ftotals%2F%3Fcomponent_id%3Dmain&query=totals.coverage&suffix=%25&style=flat-square&logo=codecov&logoColor=white&label=coverage%3Amain&color=green
[coverage]: https://app.codecov.io/gh/veecle/veecle-os?components%5B0%5D=main
