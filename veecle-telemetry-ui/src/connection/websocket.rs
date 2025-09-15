//! See [WebSocketConnection].

use std::collections::VecDeque;

use ewebsock::{WsEvent, WsMessage};
use veecle_telemetry_server_protocol::TracingMessage;

use crate::connection::{Connection, ConnectionMessage};
use anyhow::Context;

/// Represents a `WebSocket` connection.
pub struct WebSocketConnection {
    url: String,

    // When this is dropped, the connection gets closed.
    #[expect(unused)]
    sender: ewebsock::WsSender,
    receiver: ewebsock::WsReceiver,

    buffer: VecDeque<String>,
    total: usize,
    done: bool,
}

impl WebSocketConnection {
    /// Create a new `WebSocket` connection.
    pub fn new_boxed(url: String, egui_ctx: egui::Context) -> anyhow::Result<Box<dyn Connection>> {
        let wake_up = move || egui_ctx.request_repaint(); // wake up UI thread on new message

        let (sender, receiver) =
            ewebsock::connect_with_wakeup(url.clone(), Default::default(), wake_up)
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("connecting to {url}"))?;

        Ok(Box::new(Self {
            url,

            sender,
            receiver,

            buffer: Default::default(),
            total: Default::default(),
            done: Default::default(),
        }))
    }
}

impl Connection for WebSocketConnection {
    fn try_recv(&mut self) -> Option<ConnectionMessage> {
        if let Some(line) = self.buffer.pop_front() {
            return Some(ConnectionMessage::Line(line));
        }

        let event = self.receiver.try_recv()?;
        let message = match event {
            WsEvent::Message(message) => message,
            WsEvent::Error(error) => {
                return Some(ConnectionMessage::Error(
                    anyhow::anyhow!(error).context("WebSocket Error"),
                ));
            }
            WsEvent::Closed => {
                return Some(ConnectionMessage::Error(anyhow::anyhow!(
                    "WebSocket connection closed."
                )));
            }
            WsEvent::Opened => return None,
        };

        let message = match message {
            WsMessage::Text(text) => text,
            WsMessage::Ping(_) => {
                return None;
            }

            message => {
                log::warn!("WebSocket received unexpected message: {message:?}");
                return None;
            }
        };

        let message: TracingMessage = match serde_json::from_str(&message) {
            Ok(message) => message,
            Err(error) => {
                return Some(ConnectionMessage::Error(
                    anyhow::anyhow!(error).context("WebSocket Error: Deserializing message failed"),
                ));
            }
        };

        self.total = message.total;
        self.done = message.done;

        self.buffer.extend(message.lines);

        self.buffer.pop_front().map(ConnectionMessage::Line)
    }

    fn is_continuous(&self) -> bool {
        true
    }

    fn is_done(&self) -> bool {
        self.done
    }
}

impl std::fmt::Display for WebSocketConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "websocket ({})", &self.url)
    }
}

impl std::fmt::Debug for WebSocketConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketConnection")
            .field("receiver", &"WsReceiver { rx: ... }".to_string())
            .field("buffer", &self.buffer)
            .field("total", &self.total)
            .field("done", &self.done)
            .finish()
    }
}
