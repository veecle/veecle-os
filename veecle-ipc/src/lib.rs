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
//! use core::convert::Infallible;
//!
//! use veecle_os_runtime::{InitializedReader, Storable, Writer};
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
//! ) -> Infallible {
//!     loop {
//!         let Ping(value) = ping.wait_for_update().await.read_cloned();
//!         pong.write(Pong(value)).await;
//!     }
//! }
//!
//! async fn main() {
//!     let connector = veecle_ipc::jsonl::Connector::connect().await;
//!
//!     veecle_os_runtime::execute! {
//!         store: [Ping, Pong],
//!         actors: [
//!             veecle_ipc::jsonl::Input<Ping>: &connector,
//!             veecle_ipc::jsonl::Output<Pong>: &connector,
//!         ],
//!     }.await;
//! }
//! # }
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(doc)]
extern crate self as veecle_ipc;

#[cfg(feature = "jsonl")]
pub mod jsonl;

#[cfg(feature = "iceoryx2")]
pub mod iceoryx2;

mod send_policy;

pub use self::send_policy::SendPolicy;
pub use veecle_ipc_protocol::{ControlRequest, ControlResponse, Uuid};
