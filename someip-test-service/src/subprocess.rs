//! Functions to spawn a test service in a separate process.

use std::process::{Child, Command};

use anyhow::Context;

use crate::config::common_api::TempConfig as CommonApiTempConfig;
use crate::config::vsomeip::TempConfig as VSomIpTempConfig;
use crate::ipc;

/// Spawns a new test service in a separate process.
///
/// Running a test service in a separate process is required since it can only be configured through environment
/// variables. Because setting environment variables from within the same process is unsafe, we launch a separate
/// process for each test service instance and pass configuration through environment variables.
pub fn spawn(
    common_api_config: &CommonApiTempConfig,
    vsomeip_config: &VSomIpTempConfig,
) -> anyhow::Result<Child> {
    let listener = ipc::create_listener()?;

    let cwd = std::env::current_dir()?;
    let args = ["run", "--package", env!("CARGO_PKG_NAME")];
    let env = [
        ("IPC_LISTENER_PATH", listener.path()),
        ("COMMONAPI_CONFIG", common_api_config.path()),
        ("VSOMEIP_CONFIGURATION", vsomeip_config.path()),
    ];
    let child_process = Command::new("cargo")
        .current_dir(cwd)
        .args(args)
        .envs(env)
        .spawn()
        .context("failed to spawn test service subprocess")?;

    ipc::wait_for_message(&listener, "Test service successfully launched")?;

    Ok(child_process)
}
