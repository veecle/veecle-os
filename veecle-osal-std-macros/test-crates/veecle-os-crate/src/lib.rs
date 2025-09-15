//! Test that derive macros can be used while depending only on `veecle-os`.
#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {}
