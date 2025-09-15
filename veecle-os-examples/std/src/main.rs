//! Examples for std rust.
use veecle_os::osal::std::time::{Time, TimeAbstraction};

#[veecle_os::osal::std::main(telemetry = true)]
pub async fn main() {
    veecle_os::telemetry::info!("Hello from std", timestamp = format!("{:?}", Time::now()));
}
