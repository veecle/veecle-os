# someip-test-service-macro

Provides macro that simplifies setup of the test service provided by [`someip-test-service`](../someip-test-service/).

## Usage

Add following dependencies:

```toml
[dev-dependencies]
ntest_timeout = { workspace = true } # Used internally by macro.
someip-test-service = { workspace = true }
someip-test-service-macro = { workspace = true }
```

Then use [this test](./tests/smoke_test.rs) as a reference.

Documentation can be found [here](./src/lib.rs).