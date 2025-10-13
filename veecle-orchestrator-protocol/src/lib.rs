//! Data definitions to communicate with a Veecle OS Orchestrator.
//!
//! The Veecle OS Orchestrator protocol consists of JSON Lines encoded messages transported over a Unix domain socket (UDS).
//! It is request-response oriented, a client connects to the server's UDS then transmits a [`Request`] and waits for a
//! [`Response`], it may then transmit another `Request` and repeat. Each `Request` variant documents what the expected
//! `Response` inner type is.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Identifies a runtime instance that has been added to a Veecle OS Orchestrator.
///
/// The same runtime binary may be added multiple times with unique ids.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct InstanceId(Uuid);

impl InstanceId {
    /// Creates a new randomized id.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for InstanceId {
    type Err = uuid::Error; // TODO: `impl Error`

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::from_str(s)?))
    }
}

/// Requests to send to a Veecle OS Orchestrator.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Request {
    /// Query the version of the server.
    ///
    /// Responds with <code>[Response]<[String]></code>.
    Version,

    /// Add a new runtime instance with the passed information.
    ///
    /// Responds with <code>[Response]<()></code>.
    Add {
        /// The id that will be used to interact with this instance later.
        id: InstanceId,

        /// The path to the binary that defines the instance.
        path: Utf8PathBuf,

        /// Whether this runtime is privileged and can send control messages.
        privileged: bool,
    },

    /// Add a new runtime instance with binary data sent after this command.
    ///
    /// The server should respond with <code>[Response]<()></code>, then the binary data of exactly
    /// `length` bytes should be sent, then the server should again respond with
    /// <code>[Response]<()></code>.
    ///
    /// The data will be validated against the provided SHA-256 `hash`.
    AddWithBinary {
        /// The id that will be used to interact with this instance later.
        id: InstanceId,

        /// The expected length of the binary data in bytes.
        length: usize,

        /// The SHA-256 hash of the expected binary data for validation.
        hash: [u8; 32],

        /// Whether this runtime is privileged and can send control messages.
        privileged: bool,
    },

    /// Remove the runtime instance with the passed id.
    ///
    /// Responds with <code>[Response]<()></code>.
    Remove(InstanceId),

    /// Start the runtime instance with the passed id.
    ///
    /// Responds with <code>[Response]<()></code>.
    Start(InstanceId),

    /// Stop the runtime instance with the passed id.
    ///
    /// Responds with <code>[Response]<()></code>.
    Stop(InstanceId),

    /// Link IPC for a data type identified by `type_name` to `to`.
    ///
    /// The same `type_name` can have multiple destinations, the data will be cloned to all.
    ///
    /// Responds with <code>[Response]<()></code>.
    Link {
        /// The type name identifying the data.
        type_name: String,
        /// A target instance that will receive the data.
        to: LinkTarget,
    },

    /// Query info about the current server state.
    ///
    /// Response with <code>[Response]<[Info]></code>
    Info,

    /// Stop all active runtimes and clear all orchestrator state.
    ///
    /// Responds with <code>[Response]<()></code>.
    Clear,
}

/// A local or remote instance for an IPC link target.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(untagged)]
pub enum LinkTarget {
    /// The instance is running on this orchestrator, identified by just its id.
    Local(InstanceId),

    /// The instance is running on another orchestrator, accessible at the given address.
    Remote(SocketAddr),
}

impl FromStr for LinkTarget {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InstanceId::from_str(s)
            .map(Self::Local)
            .or_else(|_| SocketAddr::from_str(s).map(Self::Remote))
            .map_err(|_| "could not parse as local or remote target")
    }
}

impl fmt::Display for LinkTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local(id) => id.fmt(f),
            Self::Remote(address) => address.fmt(f),
        }
    }
}

impl Request {
    /// Get the name of this value's variant.
    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::Version => "Version",
            Self::Add { .. } => "Add",
            Self::AddWithBinary { .. } => "AddWithBinary",
            Self::Remove(_) => "Remove",
            Self::Start(_) => "Start",
            Self::Stop(_) => "Stop",
            Self::Link { .. } => "Link",
            Self::Info => "Info",
            Self::Clear => "Clear",
        }
    }

    /// Creates a new `AddWithBinary` request from binary data.
    ///
    /// Automatically calculates the length and SHA-256 hash of the provided data.
    pub fn add_with_binary(id: InstanceId, data: &[u8], privileged: bool) -> Self {
        Self::AddWithBinary {
            id,
            length: data.len(),
            hash: Sha256::digest(data).into(),
            privileged,
        }
    }
}

/// A response from a Veecle OS Orchestrator.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Response<T> {
    /// A success response.
    Ok(T),

    /// A failure response.
    ///
    /// The error is returned as a series of messages that create the chain of context, similar to
    /// what recursively calling [`Error::source`] would give.
    Err(Vec<String>),
}

#[derive(Clone, Debug)]
struct StringError {
    message: String,
    source: Option<Box<StringError>>,
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for StringError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|error| error as _)
    }
}

#[derive(Clone, Debug)]
struct ServerError {
    source: Option<Box<StringError>>,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("server returned error")
    }
}

impl Error for ServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|error| error as _)
    }
}

impl<T> Response<T> {
    /// Converts any returned error messages into an [`Error`] implementer for use with other utilities.
    ///
    /// This re-attaches the messages into a chain via [`Error::source`], so it has the structure
    /// expected by error reporters.
    pub fn into_result(self) -> Result<T, impl Error> {
        match self {
            Response::Ok(value) => Ok(value),
            Response::Err(mut messages) => {
                let mut source = None;

                for message in messages.drain(..).rev() {
                    source = Some(Box::new(StringError { message, source }))
                }

                Err(ServerError { source })
            }
        }
    }

    /// Creates a response for an error, serializing its context.
    pub fn err(error: impl Error) -> Self {
        let mut messages = Vec::new();
        let mut source: Option<&dyn Error> = Some(&error);
        while let Some(error) = source {
            messages.push(error.to_string());
            source = error.source();
        }
        Self::Err(messages)
    }
}

/// Information about a runtime instance's state.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RuntimeInfo {
    /// Whether this instance is currently running.
    pub running: bool,

    /// The path to the binary for this instance.
    pub binary: Utf8PathBuf,

    /// Whether this runtime is privileged and can send control messages.
    pub privileged: bool,
}

/// Information about the current orchestrator state.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Info {
    /// The runtime instances that are currently registered.
    pub runtimes: BTreeMap<InstanceId, RuntimeInfo>,

    /// IPC links within and without this orchestrator.
    pub links: BTreeMap<String, Vec<LinkTarget>>,
}
