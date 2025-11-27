//! Provides a simple IPC mechanism used to synchronize test service from a child process with the main process.

#![allow(
    missing_docs,
    reason = "private module re-exported for use in crate's private binary"
)]

use std::env::var as env_var;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};

use anyhow::Context;
use tempfile::{Builder, NamedTempFile};

/// Creates [`UnixListener`] bound to a temporary socket.
///
/// Temporary socket is deleted once listener goes out of scope.
pub fn create_listener() -> anyhow::Result<NamedTempFile<UnixListener>> {
    Builder::new()
        // Make it more unique because the hash from `tempfile` is short.
        .prefix(&format!("someip-test-service-{}-", rand::random::<u64>()))
        .suffix(".sock")
        .make(|path| UnixListener::bind(path))
        .context("failed to create a tempory IPC socket")
}

pub fn wait_for_message(
    listener: &NamedTempFile<UnixListener>,
    expected_message: &str,
) -> anyhow::Result<()> {
    let (mut stream, _) = listener
        .as_file()
        .accept()
        .context("failed to accept ipc socket")?;
    let mut message = String::new();
    while !message.contains(expected_message) {
        stream
            .read_to_string(&mut message)
            .context("failed to read message")?;
    }
    Ok(())
}

pub fn create_client() -> anyhow::Result<UnixStream> {
    let socket_path = env_var("IPC_LISTENER_PATH").context("failed to get ipc socket path")?;
    let stream = UnixStream::connect(&socket_path).context("failed to connect to ipc socket")?;
    Ok(stream)
}

pub fn send_message(stream: &mut UnixStream, message: &str) -> anyhow::Result<()> {
    stream
        .write_all(message.as_bytes())
        .context("failed to send message")?;
    Ok(())
}
