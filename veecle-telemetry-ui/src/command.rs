//! TODO: document

// Some of this is adapted from `rerun`.
//
// Copyright (c) 2022 Rerun Technologies AB <opensource@rerun.io>
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::collections::HashSet;
use std::sync::mpsc;

use veecle_telemetry::protocol::ThreadId;

use crate::connection::Connection;
use crate::store::Level;

#[derive(Debug)]
pub enum UICommand {
    /// Open a file.
    Open,
    /// Connect to WebSocket (open modal).
    Connect,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,

    ToggleFilterPanel,
    ToggleSelectionPanel,
}

#[derive(Debug)]
pub enum SystemCommand {
    Connect(Box<dyn Connection>),

    ClearFilter,
    SetLevelFilter(HashSet<Level>),
    SetTargetFilter(String),
    SetFileFilter(String),
    SetActorFilter(HashSet<String>),
    SetMessageFilter(String),
    SetThreadFilter(HashSet<ThreadId>),
}

#[derive(Debug, Clone)]
pub struct CommandSender {
    ui_sender: mpsc::Sender<UICommand>,
    system_sender: mpsc::Sender<SystemCommand>,
}

#[derive(Debug)]
pub struct CommandReceiver {
    ui_receiver: mpsc::Receiver<UICommand>,
    system_receiver: mpsc::Receiver<SystemCommand>,
}

impl CommandReceiver {
    /// Receive a [`SystemCommand`] to be executed if any is queued.
    pub fn recv_system(&self) -> Option<SystemCommand> {
        // The only way this can fail (other than being empty)
        // is if the sender has been dropped.
        self.system_receiver.try_recv().ok()
    }

    /// Receive a [`UICommand`] to be executed if any is queued.
    pub fn recv_ui(&self) -> Option<UICommand> {
        // The only way this can fail (other than being empty)
        // is if the sender has been dropped.
        self.ui_receiver.try_recv().ok()
    }
}

impl CommandSender {
    /// Send a command to be executed.
    pub fn send_system(&self, command: SystemCommand) {
        // The only way this can fail is if the receiver has been dropped.
        self.system_sender.send(command).ok();
    }

    /// Send a command to be executed.
    pub fn send_ui(&self, command: UICommand) {
        // The only way this can fail is if the receiver has been dropped.
        self.ui_sender.send(command).ok();
    }
}

pub fn command_channel() -> (CommandSender, CommandReceiver) {
    let (ui_sender, ui_receiver) = mpsc::channel();
    let (system_sender, system_receiver) = mpsc::channel();

    (
        CommandSender {
            ui_sender,
            system_sender,
        },
        CommandReceiver {
            ui_receiver,
            system_receiver,
        },
    )
}
