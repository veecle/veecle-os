//! Utilities for testing actors.
//!
//! [`veecle_os_test::execute!`](crate::execute) can be used in place of [`veecle_os::runtime::execute!`](veecle_os_runtime::execute!)
//! to write tests for actors.
//!
//! [`veecle_os_test::block_on_future`](block_on_future) can be used to block the current thread until the future resolves.
//!
//! The following example shows how to implement a test:
//!
//! ```
//! #[derive(Debug, Default, veecle_os_runtime::Storable)]
//! pub struct Number(usize);
//!
//! #[derive(Debug, Default, veecle_os_runtime::Storable)]
//! pub struct Total(usize);
//! #
//! # use veecle_os_runtime::{InitializedReader, Never, Reader, Writer};
//!
//! // `total_actor` reads numbers from a `Number` reader, keeps a running
//! // total, and writes running totals to a `Total` writer.
//! #[veecle_os_runtime::actor]
//! async fn total_actor(
//!     mut total: Writer<'_, Total>,
//!     mut numbers: InitializedReader<'_, Number>,
//! ) -> Never {
//!     let mut sum: usize = 0;
//!     loop {
//!         numbers.wait_for_update().await.read(|value| {
//!             sum += value.0;
//!         });
//!
//!         total.write(Total(sum)).await;
//!     }
//! }
//!
//! // This test writes 1 and 2 to the `Number` slot and verifies that
//! // `total_actor` writes 1 and 3 as the running totals.
//!
//! veecle_os_test::block_on_future(
//!     veecle_os_test::execute! {
//!         actors: [TotalActor],
//!         validation: async |mut total_reader: Reader<'_, Total>, mut numbers_writer: Writer<'_, Number>| {
//!             numbers_writer.write(Number(0)).await;
//!             let mut total_reader = total_reader.wait_init().await;
//!             total_reader.wait_for_update().await.read(|value| {
//!                 assert_eq!(value.0, 0);
//!             });
//!             numbers_writer.write(Number(1)).await;
//!             total_reader.wait_for_update().await.read(|value| {
//!                 assert_eq!(value.0, 1);
//!             });
//!             numbers_writer.write(Number(2)).await;
//!             total_reader.wait_for_update().await.read(|value| {
//!                 assert_eq!(value.0, 3);
//!             });
//!         }
//!     }
//! );
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[doc(hidden)]
mod execute;

/// Reexport of [`futures::executor::block_on`] for convenience.
pub use futures::executor::block_on as block_on_future;

/// Internal exports for `macro_rules!` purposes.
#[doc(hidden)]
pub mod __exports {
    pub use ::veecle_os_runtime;
    pub use futures;
    pub use veecle_os_runtime::Never;
}
