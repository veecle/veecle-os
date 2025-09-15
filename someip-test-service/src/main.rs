//! Test service launcher.
//!
//! Intended to be launched by the library to ensure each service has a unique environment, do not run directly.

use std::env::var as env_var;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, anyhow, bail};
use signal_hook::consts::SIGTERM;
use signal_hook::flag;
use someip_test_service::reëxports::{config, ipc};
use someip_test_service_sys::{launch, terminate};

/// Launches the test service and waits for it to start. Once ready, sends
/// a confirmation message via IPC and waits for a SIGTERM signal to terminate.
fn main() {
    let terminated = Arc::new(AtomicBool::new(false));

    flag::register(SIGTERM, Arc::clone(&terminated)).expect("SIGTERM handler should be registered");

    validate_vsomeip_config().expect("vsomeip config should be provided and valid");
    validate_common_api_config().expect("common api config should be provided and valid");

    // SAFETY: We verify that the environment variables are set before calling the launch function.
    unsafe {
        launch();
    }

    let mut client = ipc::create_client().expect("ipc client should be created");
    ipc::send_message(&mut client, "Test service successfully launched")
        .expect("message should be sent");
    drop(client);

    while !terminated.load(Ordering::Relaxed) {
        sleep(Duration::from_millis(500));
    }

    // SAFETY: There's no safety restriction for calling the terminate function.
    unsafe {
        terminate();
    }
}

fn validate_vsomeip_config() -> anyhow::Result<()> {
    let env_var_name = "VSOMEIP_CONFIGURATION";
    let file_path = path_from_env_var(env_var_name)
        .context(format!("failed to obtain path from {env_var_name}"))?;
    let file_content = read_to_string(&file_path)
        .context(format!("failed to read file {}", file_path.display()))?;
    serde_json::from_str::<config::VSomeIpConfig>(&file_content).context(format!(
        "configuration file provided via {env_var_name} is invalid"
    ))?;
    Ok(())
}

fn validate_common_api_config() -> anyhow::Result<()> {
    let env_var_name = "COMMONAPI_CONFIG";
    let file_path = path_from_env_var(env_var_name)
        .context(format!("failed to obtain path from {env_var_name}"))?;
    let file_content = read_to_string(&file_path)
        .context(format!("failed to read file {}", file_path.display()))?;
    serde_ini::from_str::<config::CommonApiConfig>(&file_content).context(format!(
        "configuration file provided via {env_var_name} is invalid"
    ))?;
    Ok(())
}

fn path_from_env_var(name: &str) -> anyhow::Result<PathBuf> {
    let path = env_var(name).map_err(|_| anyhow!("environment variable {name} not set"))?;
    if !Path::new(&path).is_file() {
        bail!("path {path} does not point to a file");
    }
    Ok(PathBuf::from(path))
}
