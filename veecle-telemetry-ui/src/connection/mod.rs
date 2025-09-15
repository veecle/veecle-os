//! Connection handling for `veecle-telemetry-ui`.

pub mod file;
pub mod file_contents;
pub mod pipe;
pub mod websocket;

/// A tracing data connection.
pub trait Connection: std::fmt::Display + std::fmt::Debug {
    /// Try to receive the next message.
    ///
    /// Returns `None` if all data has been received or there is no new data at the moment.
    fn try_recv(&mut self) -> Option<ConnectionMessage>;

    /// Returns `true` if the connection is to an active program, and we are expecting more data.
    fn is_continuous(&self) -> bool;

    /// Returns `true` if all data has been received.
    fn is_done(&self) -> bool;
}

/// Messages received from a connection.
#[derive(Debug)]
pub enum ConnectionMessage {
    /// A line of tracing data.
    Line(String),
    /// An error received from a connection.
    Error(anyhow::Error),
    /// Connection is done.
    Done,
    /// Restart the connection (clear the store).
    Restart,
}
