# Contributing

Essentially, this repository is an unsafe FFI wrapper around [SOME/IP test service](./cpp/service/) built with [CommonAPI C++ Framework][capicxx-framework].

## CommonAPI C++ Framework

CommonAPI C++ Framework is a framework for IPC communication.
It uses D-Bus and/or SOME/IP as an underlying middleware communication protocols.
Since we're building service to test only SOME/IP, we'll skip the D-Bus part since CommonAPI C++ Framework is modular and allows to completely omit it.

When we're concerned only about SOME/IP, CommonAPI C++ Framework consists of:

- [`vsomeip`][vsomeip] - implements SOME/IP protocol.
- [`CommonAPI C++ Core Runtime`][capicxx-core-runtime] - provides protocol-independent runtime.
- [`CommonAPI C++ SOME/IP Runtime`][capicxx-someip-runtime] - bridges protocol and runtime.
- [`CommonAPI C++ Core Tools`][capicxx-core-tools] - takes [Franca definition file][franca-idl] (`*.fidl`) and generates protocol-independent service (called `Stub`) and client (called `Proxy`).
- [`CommonAPI C++ SOME/IP Tools`][capicxx-someip-tools] - takes [Franca deployment file][franca-depl-reference] (`*.fdepl`) and generates SOME/IP-specific glue-code which consist of adapters for the service (called `SomeIPStubAdapter`) and client (called `SomeIPProxy`).

The basic workflow of CommonAPI C++ Framework is:

1. Write service definition in `*.fidl` file.
2. Write service deployment in `*.fdepl` file.
3. Generate `Stub` and `Proxy` from `*.fidl` files.
4. Generate `SomeIPStub` and `SomeIPProxy` from `*.fdepl` files.
5. Provide an implementation of methods for the `Stub` (via class inheritance and method override).
   > [!NOTE]
   > `SomeIPStub` is left untouched as it's just an adapter used by runtime internally.
6. Build so-called "interface library" from files generated on step 4. Link it with `CommonAPI C++ SOME/IP Runtime`.
7. Build your SOME/IP service. Link it with `CommonAPI C++ Core Runtime`.

This repository already has all these steps done and partially automated.

The following guide describes how you can make modifications.

## Writing Franca files

Franca definition and deployment files are located in `./fidl/`.

Use examples from [here][franca-examples-internal] and [here][franca-examples-external] as a references and [this][franca-reference] as an entry for documentation.

## Generating stubs and proxies

Make sure `Docker` is installed (use [official guide](https://docs.docker.com/engine/install/)).

> [!NOTE]
> If you're on Mac, make sure your Docker Desktop is configured to run `x86` images since generators are only available for this architecture.
> See [this](https://docs.docker.com/desktop/features/vmm/) to understand how to configure it.

Then, run:

```sh
just generate
```

## Adjusting stub implementation

Modify `./cpp/service/service.hpp` - add/remove/update methods/names as needed.
Use existing implementation as a reference.

## Auto-formatting C/C++ files

Make sure `clang-format` is installed.

Then, run:

```sh
just fmt_cpp
```

## Testing

```sh
cargo test
```

## Considering `interface.hpp`

`./cpp/interface.hpp` is used by `bindgen` to generate Rust bindings.
However, due to how C/C++ works, it is impossible to verify in compile time that both `./cpp/service/` and `./cpp/service_stub/` implement this interface.
Therefore, if you are making changes there, please make sure you update both of them.

[capicxx-framework]: https://covesa.github.io/capicxx-core-tools/
[vsomeip]: https://github.com/COVESA/vsomeip
[capicxx-core-runtime]: https://github.com/COVESA/capicxx-core-runtime
[capicxx-someip-runtime]: https://github.com/COVESA/capicxx-someip-runtime
[capicxx-core-tools]: https://github.com/COVESA/capicxx-core-tools
[capicxx-someip-tools]: https://github.com/COVESA/capicxx-someip-tools
[capicxx-core-tools-release]: https://github.com/COVESA/capicxx-core-tools/releases/tag/3.2.15
[capicxx-someip-tools-release]: https://github.com/COVESA/capicxx-someip-tools/releases/tag/3.2.15
[franca-idl]: https://github.com/franca/franca/
[franca-examples-internal]: https://github.com/veecle/franca/tree/main/examples
[franca-examples-external]: https://github.com/COVESA/capicxx-someip-tools/tree/master/test/fidl
[franca-reference]: https://github.com/bmwcarit/joynr/blob/master/wiki/franca.md
[franca-depl-reference]: https://github.com/COVESA/capicxx-core-tools/blob/master/org.genivi.commonapi.core/deployment/CommonAPI_deployment_spec.fdepl
[incus-getting-started]: https://github.com/veecle/infrastructure/blob/main/docs/virtualization.md
