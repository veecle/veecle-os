//! Future instrumentation utilities for tracing async operations.
//!
//! This module provides utilities for instrumenting Rust futures with telemetry spans.
//! When a future is instrumented, the associated span is automatically entered every time
//! the future is polled.
//!
//! # Examples
//!
//! ```rust
//! use veecle_telemetry::future::FutureExt;
//! use veecle_telemetry::span;
//!
//! async fn example() {
//!     let span = span!("async_operation", user_id = 123);
//!
//!     some_async_work().with_span(span).await;
//! }
//!
//! async fn some_async_work() {
//!     // This work will be traced under the "async_operation" span
//! }
//! ```

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::Span;

impl<T> FutureExt for T where T: Future {}

/// Extension trait for instrumenting futures with telemetry spans.
///
/// This trait provides methods to attach telemetry spans to futures,
/// ensuring that the spans are entered every time the future is polled.
pub trait FutureExt: Future + Sized {
    /// Instruments a future with the provided [`Span`].
    ///
    /// The attached [`Span`] will be [entered] every time it is polled.
    ///
    /// [entered]: Span::enter()
    fn with_span(self, span: Span) -> WithSpan<Self> {
        WithSpan { inner: self, span }
    }
}

/// A future that has been instrumented with a telemetry span.
///
/// This future wrapper ensures that the associated span is entered every time
/// the future is polled.
///
/// Instances of this type are created using the [`FutureExt::with_span`] method.
#[pin_project::pin_project]
#[derive(Debug)]
pub struct WithSpan<T> {
    #[pin]
    inner: T,
    span: Span,
}

impl<T> Future for WithSpan<T>
where
    T: Future,
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _enter = this.span.enter();
        this.inner.poll(cx)
    }
}
