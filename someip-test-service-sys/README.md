# someip-test-service-sys

Provides pre-made SOME/IP test service and unsafe Rust FFI bindings to it.
It is meant to be used by higher-level packages that'll provide safe high-level abstractions suitable for integration into a test suite.
The reason of having this package is to provide bindings to the reference SOME/IP service implementation and use it to validate other SOME/IP implementations.

> [!WARNING]
> This package is meant to be used only on Linux.
> On all other operating systems it will compile to the [empty stub](./cpp/service_stub/).
> See [`capicxx-someip-sys`][capicxx-someip-sys] documentation for details.

## Configuration

1. Make sure you followed configuration instructions from [`capicxx-someip-sys`](../capicxx-someip-sys/) package.
2. Install dependencies required by [`bindgen`](https://rust-lang.github.io/rust-bindgen/requirements.html).
3. Make sure `COMMONAPI_CONFIG` environment variable is set and points to the valid Common API configuration file.
   Minimal configuration example can be found in [tests](./tests).
   Read `./cpp/interface.hpp` for more details. 
4. Optionally, make sure `VSOMEIP_CONFIGURATION` environment variable is set and points to the valid vsomeip configuration file.
   Minimal configuration example can be found in [tests](./tests).
   Read `./cpp/interface.hpp` for more details. 

## Usage

This package provides two unsafe functions - `launch()` and `terminate()`, which start and shut down the pre-built SOME/IP test service.

```rust
unsafe {
   launch();
   terminate();
}
```

You can find more details on these functions in `./cpp/interface.hpp`.

See [`./fild`](./fidl/) directory and [`./cpp/service/src/service.hpp`](./cpp/service/src/service.hpp) to understand which methods SOME/IP test service provides and how they behave.

## Contribution

See [`CONTRIBUTING.md`](./CONTRIBUTING.md).
