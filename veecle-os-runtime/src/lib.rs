//! The Veecle OS runtime.
//!
//! This crate contains the main building blocks for any Veecle OS application.  Veecle OS applications are composed of [`Actor`]s,
//! that use the [`Reader`] and [`Writer`] types to communicate with each other within the runtime.
//!
//! This crate is meant to be used with asynchronous programming, which means that actors are expected to be async
//! functions. For example, it will be ensured that an actor does not update a value till all its readers had the
//! chance to read its latest state.
//!
//! # Example
//!
//! The following Veecle OS application consists of two actors, `PingActor` and `PongActor`, that communicate with each
//! other.
//!
//! ```rust
//! use std::fmt::Debug;
//!
//! use veecle_os_runtime::{Never, Reader, Storable, Writer};
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Default, Storable)]
//! pub struct Ping {
//!     value: u32,
//! }
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Default, Storable)]
//! pub struct Pong {
//!     value: u32,
//! }
//!
//! #[veecle_os_runtime::actor]
//! async fn ping_actor(mut ping: Writer<'_, Ping>, pong: Reader<'_, Pong>) -> Never {
//!     let mut value = 0;
//!     ping.write(Ping { value }).await;
//!
//!     let mut pong = pong.wait_init().await;
//!     loop {
//!         ping.write(Ping { value }).await;
//!         value += 1;
//!
//!         pong.wait_for_update().await.read(|pong| {
//!             println!("Pong: {}", pong.value);
//!         });
//! #       // Exit the application to allow doc-tests to complete.
//! #       std::process::exit(0);
//!     }
//! }
//!
//! #[veecle_os_runtime::actor]
//! async fn pong_actor(mut pong: Writer<'_, Pong>, ping: Reader<'_, Ping>) -> Never {
//!     let mut ping = ping.wait_init().await;
//!     loop {
//!         let ping = ping.wait_for_update().await.read_cloned();
//!         println!("Ping: {}", ping.value);
//!
//!         let data = Pong { value: ping.value };
//!         pong.write(data).await;
//!     }
//! }
//!
//! futures::executor::block_on(
//!     veecle_os_runtime::execute! {
//!         store: [Ping, Pong],
//!         actors: [
//!             PingActor,
//!             PongActor,
//!         ]
//!     }
//! )
//! ```
//!
//! ## Output
//!
//! The expected output for this example would be a sequence of ping/pong messages like the following:
//!
//! ```shell
//! Ping: 1
//! Pong: 1
//! Ping: 2
//! Pong: 2
//! Ping: 3
//! Pong: 3
//! ...
//! ```
//!
//! ## Execution
//!
//! See how the `PongActor` waits for `Ping` to be written by the `PingActor`.
//! If that were not the behavior, we would see `Ping: 0` as the very first output, since `Ping` defaults to zero.
//! The same would happen with the `PingActor`: if it were not waiting for `Pong` updates, its immediate action after
//! writing `Ping` would be to display `Pong: 0`. Waiting for updates ensures us that only written values are read.
//!
//! On the other hand, writing always yields for other woken futures to be executed before performing the write
//! operation. The only exception is the very first write, since there is no latest value for readers to read.

#![cfg_attr(docsrs, allow(internal_features))]
#![cfg_attr(docsrs, feature(rustdoc_internals))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![no_std]

#[cfg(test)]
extern crate std;

pub(crate) mod actor;
mod cons;
pub(crate) mod datastore;
mod execute;

mod heapfree_executor;

pub mod memory_pool;

pub use self::actor::{Actor, StoreRequest, actor};
pub use self::datastore::{
    CombinableReader, CombineReaders, ExclusiveReader, InitializedReader, Reader, Storable, Writer,
};

/// Internal exports for proc-macro and `macro_rules!` purposes.
#[doc(hidden)]
pub mod __exports {
    pub use crate::actor::{Datastore, IsActorResult};
    pub use crate::cons::{Cons, Nil, TupleConsToCons};
    pub use crate::execute::{execute_actor, make_store, validate_actors};
    pub use crate::heapfree_executor::{Executor, ExecutorShared};
}

/// A type that can never be constructed.
///
/// Used as the success type in `Result<Never, E>` to indicate that an operation
/// never returns successfully, only by error. This is semantically clearer than
/// using `Infallible` which suggests "cannot fail."
///
// TODO(https://github.com/rust-lang/rust/issues/35121)
/// This type will be replaced with the never type [`!`] once it is stabilized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Never {}

impl core::fmt::Display for Never {
    fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {}
    }
}

impl core::error::Error for Never {}
