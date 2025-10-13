# capicxx-someip-sys

Provides SOME/IP subset of the [CommonAPI C++ Framework][capicxx-framework] as a Rust package.
It is meant to be used by other `-sys` packages that'd like to write SOME/IP applications with CommonAPI C++ SOME/IP Framework and therefore does not provide any Rust bindings.
The reason of having this as a separate package is to optimise a build time and caching, so that changing SOME/IP application's code is not causing CommonAPI C++ SOME/IP Framework to rebuild.

> [!WARNING]
> This package is meant to be used only on Linux.
> On all other OS's it will simply produce nothing and metadata field will point to the empty directory.
> This is because [`vsomeip`][vsomeip-repo] (which is a part of the CommonAPI C++ SOME/IP Framework) officially supports only [Linux][vsomeip-doc-linux-support] and [Android][vsomeip-doc-android-support].

## Configuration

1. Make sure [`boost`][boost] v1.66 is installed (use [this method][how-to-install-boost-ubuntu] if you are on Ubuntu).
2. Make sure [`CMake`][cmake] v3.28 or later is installed and available in the system `PATH`.
3. Make sure `VSOMEIP_PATH` environment variable is set and points to the folder with [`vsomeip v3.5.4`][required-vsomeip-version].
4. Make sure `COMMON_API_CORE_PATH` environment variable is set and points to the folder with internal fork of [`capicxx-core-runtime v3.2.4.patched`][required-core-runtime-version].
5. Make sure `COMMON_API_SOMEIP_PATH` environment variable is set and points to the folder with [`capicxx-someip-runtime v3.2.4`][required-someip-runtime-version].

> [!NOTE]
> You can use `[env]` section in `.cargo/config.toml` to set environment variables.

## Usage

> [!NOTE]
> This guide assumes you're already familiar with CommonAPI C++ SOME/IP Framework.
> If that's not the case, please read [this][capicxx-framework] and [this][someip-runtime-doc-quick-start] first.

First, add this package as a dependency.

```toml
# Cargo.toml

[dependencies]
capicxx-someip-sys = { workspace = true }
```

This package exposes a single metadata field called `install-path`.
Its value is a path where CommonAPI C++ SOME/IP Framework is installed.
You can read it in your build script as an environment variable.

```rust
// build.rs

let capicxx_someip_install_path = std::env::var("DEP_CAPICXX_SOMEIP_INSTALL_PATH")?;
```

At this path, you'll find the following directories.

```text
.
├── build/
├── etc/
│   └── vsomeip/
│       └── vsomeip*.json
├── include/
│   ├── CommonAPI-*/
│   ├── compat/
│   │   └── vsomeip/
│   └── vsomeip/
└── lib/
    ├── cmake/
    ├── pkgconfig/
    ├── libCommonAPI-SomeIP.so
    ├── libCommonAPI.so
    ├── libvsomeip3-cfg.so
    ├── libvsomeip3-e2e.so
    ├── libvsomeip3-sd.so
    └── libvsomeip3.so
```

Where:

- `build/` - build artifacts.
- `etc/vsomeip/` - reference configuration files.
- `include/` - C/C++ include headers.
- `lib/*.so` - framework's libraries.
- `lib/cmake/` - CMake configuration files.
- `lib/pkgconfig/` - pkg-config configuration files.

You can then use these files in your build script to build your SOME/IP application.

For example, if you're using CMake, add this.

```cmake
# CMakeLists.txt

# Discover CommonAPI C++ SOME/IP Frameework.
find_package(CommonAPI REQUIRED CONFIG)
find_package(CommonAPI-SomeIP REQUIRED CONFIG)

# Link your application and interface library with it.
target_link_libraries(application CommonAPI)
target_link_libraries(application-interface-someip CommonAPI-SomeIP)
```

Then tell CMake where it can find these packages by passing `DEP_CAPICXX_SOMEIP_INSTALL_PATH` to the `CMAKE_PREFIX_PATH`.

If you're using the [`cmake`][cmake-package] Rust crate to build your project, you can do it as follows.

```rust
// build.rs

let capicxx_someip_install_path = std::env::var("DEP_CAPICXX_SOMEIP_INSTALL_PATH")?;
let cmake_prefix_path = format!("-DCMAKE_PREFIX_PATH={capicxx_someip_install_path}");

cmake::Config::new("path/to/your/cmake_project").configure_arg(cmake_prefix_path).build();

println!("cargo:rustc-link-lib=CommonAPI");
println!("cargo:rustc-link-lib=CommonAPI-SomeIP");
```

[capicxx-framework]: https://covesa.github.io/capicxx-core-tools/
[vsomeip-repo]: https://github.com/COVESA/vsomeip
[vsomeip-doc-linux-support]: https://github.com/COVESA/vsomeip?tab=readme-ov-file#build-instructions-for-linux
[vsomeip-doc-android-support]: https://github.com/COVESA/vsomeip?tab=readme-ov-file#build-instructions-for-android
[boost]: https://www.boost.org
[cmake]: https://cmake.org
[how-to-install-boost-ubuntu]: https://stackoverflow.com/a/12578564
[required-vsomeip-version]: https://github.com/COVESA/vsomeip/tree/3.5.4
[required-core-runtime-version]: https://github.com/veecle/capicxx-core-runtime-fork/tree/3.2.4.patched
[required-someip-runtime-version]: https://github.com/COVESA/capicxx-someip-runtime/tree/3.2.4
[someip-runtime-doc-quick-start]: https://github.com/COVESA/capicxx-someip-tools/wiki/CommonAPI-C---SomeIP-in-10-minutes
[cmake-package]: https://crates.io/crates/cmake
