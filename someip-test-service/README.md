# SOME/IP Test Service

A Rust crate for testing SOME/IP services in integration tests against a pre-built external implementation.
For information about the underlying test service and its interfaces, see [`someip-test-service-sys`](../someip-test-service-sys/).

## Usage

Add this crate as a development dependency:

```toml
[dev-dependencies]
someip-test-service = { workspace = true }
```

Then use [this test](./tests/smoke_test.rs) as a reference.

## Important Notes

- Each service instance must have a unique unicast port.
- The service automatically stops when it goes out of scope.
- The library is intended for use in integration tests, not production code.
- Only Linux is currently supported.
- Consider adding a timeout to your tests to prevent hanging.
