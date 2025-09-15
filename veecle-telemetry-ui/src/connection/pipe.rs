//! See [PipeConnection].

use std::sync::mpsc;

use crate::connection::{Connection, ConnectionMessage};

/// A connection via a channel, most likely a unix pipe,
/// could also just be a channel inside the binary.
#[derive(Debug)]
pub struct PipeConnection {
    /// Data receiver.
    receiver: mpsc::Receiver<String>,
    /// Whether the receiver has closed.
    done: bool,
}

impl PipeConnection {
    /// Create a new connection to a channel.
    pub fn new_boxed(receiver: mpsc::Receiver<String>) -> Box<dyn Connection> {
        Box::new(Self {
            receiver,
            done: false,
        })
    }
}

impl Connection for PipeConnection {
    fn try_recv(&mut self) -> Option<ConnectionMessage> {
        match self.receiver.try_recv() {
            Ok(line) => Some(ConnectionMessage::Line(line)),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.done = true;
                None
            }
        }
    }

    fn is_continuous(&self) -> bool {
        true
    }

    fn is_done(&self) -> bool {
        self.done
    }
}

impl std::fmt::Display for PipeConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("stdin pipe")
    }
}
