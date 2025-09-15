//! Provides functions to work with an endpoint used to communicate with a test service.

use std::net::UdpSocket;

use anyhow::Context;

use crate::config::test_service::Config as TestServiceConfig;

pub fn create(config: &TestServiceConfig) -> anyhow::Result<UdpSocket> {
    let socket = UdpSocket::bind((config.unicast_address.as_str(), 0))
        .context("failed to create a socket")?;
    socket
        .connect((config.unicast_address.as_str(), config.unicast_port))
        .context("failed to connect to a socket")?;
    Ok(socket)
}

/// Sends a message on the socket to the remote address to which it was connected.
/// On success, returns the number of bytes written.
pub fn send(socket: &UdpSocket, message: &[u8]) -> anyhow::Result<usize> {
    let bytes_written = socket.send(message).context("failed to send data")?;
    Ok(bytes_written)
}

/// Receives a single message on the socket from the remote address to which it was connected.
/// On success, returns the number of bytes read.
pub fn receive(socket: &UdpSocket, message: &mut [u8]) -> anyhow::Result<usize> {
    let bytes_read = socket.recv(message).context("failed to receive data")?;
    Ok(bytes_read)
}
