//! Supports inter-runtime-communication when running multiple runtime instances under the `veecle-orchestrator`.
//!
//! ## Usage
//!
//! Initialize a [`Connector`] and use [`veecle_ipc::Input`](data coming in to this runtime) and
//! [`veecle_ipc::Output`] (data going out from this runtime) to register the [`Storable`] data types you expect
//! to exchange with other runtimes.
//!
//! [`Storable`]: veecle_os_runtime::Storable
//!
//! ```no_run
//! # fn main() {
//! use veecle_os_runtime::{InitializedReader, Storable, Writer, Never};
//!
//! #[derive(Copy, Clone, Debug, Storable, serde::Deserialize)]
//! pub struct Ping(u8);
//!
//! #[derive(Copy, Clone, Debug, Storable, serde::Serialize)]
//! pub struct Pong(u8);
//!
//! #[veecle_os_runtime::actor]
//! async fn local_actor(
//!     mut ping: InitializedReader<'_, Ping>,
//!     mut pong: Writer<'_, Pong>,
//! ) -> Never {
//!     loop {
//!         let Ping(value) = ping.wait_for_update().await.read_cloned();
//!         pong.write(Pong(value)).await;
//!     }
//! }
//!
//! async fn main() {
//!     let connector = veecle_ipc::Connector::connect().await;
//!
//!     veecle_os_runtime::execute! {
//!         store: [Ping, Pong],
//!         actors: [
//!             veecle_ipc::Input<Ping>: &connector,
//!             veecle_ipc::Output<Pong>: (&connector).into(),
//!         ],
//!     }.await;
//! }
//! # }
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(doc)]
extern crate self as veecle_ipc;

mod actors;
mod connector;
mod send_policy;
mod telemetry;

pub use self::actors::{ControlHandler, Input, Output, OutputConfig};
pub use self::connector::Connector;
pub use self::send_policy::SendPolicy;
pub use self::telemetry::Exporter;
pub use veecle_ipc_protocol::{ControlRequest, ControlResponse, Uuid};
