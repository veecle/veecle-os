# veecle-os-data-support-can-codegen

Generates Veecle OS code from CAN DBC files.

## Overview

This crate provides code generation capabilities for converting CAN DBC files into Rust code compatible with Veecle OS.
It parses DBC files and generates Rust structures and implementations for CAN message handling.

**Note**: Most users should depend on the [`veecle-os`](https://crates.io/crates/veecle-os) crate instead of using this crate directly.
The `veecle-os` crate re-exports this functionality and provides a more complete API for building Veecle OS applications.
