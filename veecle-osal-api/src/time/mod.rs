//! Abstractions for time-based operations.
//!
//! The main purpose of this module is to provide the definition of [`TimeAbstraction`], the trait that has to be
//! implemented to interact with the underlying operating system when performing time-based operations.
//! In order to keep the time abstractions as decoupled as possible from the running environment, this module provides
//! its own [`Instant`] and [`Duration`] types.
//!
//! # Example
//!
//! Runtime actors using time-based operations should rely on the [`TimeAbstraction`] trait, and never use specific
//! implementations. It is during the runtime's setup that the concrete implementation for the targeted environment has
//! to be specified.
//!
//! ```rust
//! use veecle_os_runtime::Never;
//!
//! use veecle_osal_api::time::{Duration, TimeAbstraction};
//! use veecle_osal_std::time::Time;
//! use veecle_os_runtime::{Reader, Storable, Writer};
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Default, Storable)]
//! pub struct Tick {
//!     since_epoch: Duration,
//! }
//!
//! #[veecle_os_runtime::actor]
//! async fn tick_writer<T>(mut writer: Writer<'_, Tick>) -> Never
//! where
//!     T: TimeAbstraction,
//! {
//!     let epoch = T::now();
//!
//!     loop {
//!         let _ = T::sleep_until(T::now() + Duration::from_secs(1)).await;
//!         writer
//!             .write(Tick {
//!                 since_epoch: T::now()
//!                     .duration_since(epoch)
//!                     .expect("now should be later than epoch"),
//!             })
//!             .await;
//!     }
//! }
//!
//! #[veecle_os_runtime::actor]
//! async fn tick_reader(mut reader: Reader<'_, Tick>) -> Never {
//!     loop {
//!         reader.wait_for_update().await.read(|tick| {
//!             println!("[READER TASK] Tick received: {tick:?}");
//! #           // Exit the application to allow doc-tests to complete.
//! #           std::process::exit(0);
//!         })
//!     }
//! }
//!
//! # let mut rt = tokio::runtime::Runtime::new().unwrap();
//! # rt.block_on(async move {
//! #
//! veecle_os_runtime::execute! {
//!     actors: [
//!         TickWriter<Time>,
//!         TickReader,
//!     ]
//! }.await;
//!
//! unreachable!("the runtime instance does not return");
//! # })
//! ```

#![allow(async_fn_in_trait, reason = "auto-bounds are not necessary here")]

use core::future::IntoFuture;

use futures::future::Either;

mod duration;
mod instant;
mod system_time;
mod timeout;

pub use self::duration::Duration;
pub use self::instant::Instant;
pub use self::system_time::{SystemTime, SystemTimeError, SystemTimeSync};
pub use self::timeout::Exceeded;
use crate::Error;

/// A stream of periodic ticks, created by [`TimeAbstraction::interval`].
pub trait Interval {
    /// Completes when the next period has been reached (unless there is an error).
    ///
    /// If the stream consumer falls behind and multiple periods go by between reading from the stream, the stream will
    /// keep track of the missed periods and instantly yield them until caught up.
    async fn tick(&mut self) -> Result<(), Error>;
}

/// `TimeAbstraction` is used to perform time-related operations in a platform-agnostic manner.
pub trait TimeAbstraction {
    /// Retrieves the current time.
    fn now() -> Instant;

    /// Returns a future that resolves successfully at the specified `deadline` (or earlier with an error).
    async fn sleep_until(deadline: Instant) -> Result<(), Error>;

    /// Returns a future that resolves successfully after the specified `duration` (or earlier with an error).
    ///
    /// If the `duration` overflows `Instant`, the method sleeps for an unspecified time.
    async fn sleep(duration: Duration) -> Result<(), Error> {
        match Self::now().checked_add(duration) {
            Some(deadline) => Self::sleep_until(deadline).await,
            None => Self::sleep_until(Self::now() + Duration::max_no_overflow_alias()).await,
        }
    }

    /// Returns a future that will resolve when: the wrapped future resolves, the `deadline` is reached, or there is an
    /// error.
    async fn timeout_at<F>(
        deadline: Instant,
        future: F,
    ) -> Result<F::Output, Either<Exceeded, Error>>
    where
        Self: Sized,
        F: IntoFuture,
    {
        self::timeout::timeout_at::<Self, _>(deadline, future.into_future()).await
    }

    /// Returns an [`Interval`] that will yield an item straight away and then once every `period` (unless there is an error).
    ///
    /// If the stream consumer falls behind and multiple periods go by between reading from the stream, the stream will
    /// keep track of the missed periods and instantly yield them until caught up.
    #[must_use]
    fn interval(period: Duration) -> impl Interval;
}
