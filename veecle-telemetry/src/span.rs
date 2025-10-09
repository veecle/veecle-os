//! Distributed tracing spans for tracking units of work.
//!
//! This module provides the core span implementation for distributed tracing.
//! Spans represent units of work within a trace and can be nested to show
//! relationships between different operations.
//!
//! # Key Concepts
//!
//! - **Span**: A unit of work within a trace, with a name and optional attributes
//! - **Span Context**: The trace and span IDs that identify a span within a trace
//! - **Span Guards**: RAII guards that automatically handle span entry/exit
//! - **Current Span**: Thread-local tracking of the currently active span
//!
//! # Basic Usage
//!
//! ```rust
//! use veecle_telemetry::{CurrentSpan, span};
//!
//! // Create and enter a span
//! let span = span!("operation", user_id = 123);
//! let _guard = span.entered();
//!
//! // Add events to the current span
//! CurrentSpan::add_event("checkpoint", &[]);
//!
//! // Span is automatically exited when guard is dropped
//! ```
//!
//! # Span Lifecycle
//!
//! 1. **Creation**: Spans are created with a name and optional attributes
//! 2. **Entry**: Spans are entered to make them the current active span
//! 3. **Events**: Events and attributes can be added to active spans
//! 4. **Exit**: Spans are exited when no longer active
//! 5. **Close**: Spans are closed when their work is complete
//!
//! # Nesting
//!
//! Spans can be nested to show relationships:
//!
//! ```rust
//! use veecle_telemetry::span;
//!
//! let parent = span!("parent_operation");
//! let _parent_guard = parent.entered();
//!
//! // This span will automatically be a child of the parent
//! let child = span!("child_operation");
//! let _child_guard = child.entered();
//! ```

#[cfg(feature = "enable")]
use core::cell::Cell;
use core::marker::PhantomData;
#[cfg(all(feature = "std", feature = "enable"))]
use std::thread_local;

use crate::SpanContext;
#[cfg(feature = "enable")]
use crate::collector::get_collector;
#[cfg(feature = "enable")]
use crate::id::SpanId;
#[cfg(feature = "enable")]
use crate::protocol::{
    SpanAddEventMessage, SpanAddLinkMessage, SpanCloseMessage, SpanCreateMessage, SpanEnterMessage,
    SpanExitMessage, SpanSetAttributeMessage,
};
#[cfg(feature = "enable")]
use crate::time::now;
use crate::value::KeyValue;

#[cfg(feature = "enable")]
thread_local! {
    pub(crate) static CURRENT_SPAN: Cell<Option<SpanId>> = const { Cell::new(None) };
}

/// A distributed tracing span representing a unit of work.
///
/// Spans are the fundamental building blocks of distributed tracing.
/// They represent a unit of work within a trace and can be nested to show relationships between different operations.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::{KeyValue, Span, Value};
///
/// // Create a span with attributes
/// let span = Span::new("database_query", &[
///     KeyValue::new("table", Value::String("users".into())),
///     KeyValue::new("operation", Value::String("SELECT".into())),
/// ]);
///
/// // Enter the span to make it active
/// let _guard = span.enter();
///
/// // Add events to the span
/// span.add_event("query_executed", &[]);
/// ```
///
/// # Conditional Compilation
///
/// When the `enable` feature is disabled, spans compile to no-ops with zero runtime overhead.
#[must_use]
#[derive(Default, Debug)]
pub struct Span {
    #[cfg(feature = "enable")]
    pub(crate) span_id: Option<SpanId>,
}

/// Utilities for working with the currently active span.
///
/// This struct provides static methods for interacting with the current span
/// in the thread-local context.
/// It allows adding events, links, and attributes to the currently active span without needing a direct reference to
/// it.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::{CurrentSpan, span};
///
/// let span = span!("operation");
/// let _guard = span.entered();
///
/// // Add an event to the current span
/// CurrentSpan::add_event("milestone", &[]);
/// ```
#[derive(Default, Debug)]
pub struct CurrentSpan;

impl Span {
    /// Creates a no-op span that performs no tracing operations.
    ///
    /// This is useful for creating spans that may be conditionally enabled
    /// or when telemetry is completely disabled.
    #[inline]
    pub fn noop() -> Self {
        Self {
            #[cfg(feature = "enable")]
            span_id: None,
        }
    }

    /// Creates a new span as a child of the current span.
    ///
    /// If there is no current span, this returns a new root span.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the span
    /// * `attributes` - Key-value attributes to attach to the span
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::{KeyValue, Span, Value};
    ///
    /// let span = Span::new("operation", &[KeyValue::new("user_id", Value::I64(123))]);
    /// ```
    pub fn new(name: &'static str, attributes: &'_ [KeyValue<'static>]) -> Self {
        #[cfg(not(feature = "enable"))]
        {
            let _ = (name, attributes);
            Self::noop()
        }

        #[cfg(feature = "enable")]
        {
            Self::new_inner(name, attributes)
        }
    }

    /// Creates a [`SpanContext`] from this [`Span`].
    /// For a noop span, this function will return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_telemetry::Span;
    ///
    /// let span = Span::new("root_span", &[]);
    /// assert!(span.context().is_some());
    /// ```
    pub fn context(&self) -> Option<SpanContext> {
        #[cfg(not(feature = "enable"))]
        {
            None
        }

        #[cfg(feature = "enable")]
        {
            self.span_id
                .map(|span_id| SpanContext::new(get_collector().process_id(), span_id))
        }
    }

    /// Enters this span, making it the current active span.
    ///
    /// This method returns a guard that will automatically exit the span when dropped.
    /// The guard borrows the span, so the span must remain alive while the guard exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::Span;
    ///
    /// let span = Span::new("operation", &[]);
    /// let _guard = span.enter();
    /// // span is now active
    /// // span is automatically exited when _guard is dropped
    /// ```
    pub fn enter(&'_ self) -> SpanGuardRef<'_> {
        #[cfg(not(feature = "enable"))]
        {
            SpanGuardRef::noop()
        }

        #[cfg(feature = "enable")]
        {
            let Some(span_id) = self.span_id else {
                return SpanGuardRef::noop();
            };

            self.do_enter();
            CURRENT_SPAN
                .try_with(|current| {
                    let parent = current.get();
                    current.set(Some(span_id));

                    SpanGuardRef::new(self, parent)
                })
                .unwrap_or(SpanGuardRef::noop())
        }
    }

    /// Enters this span by taking ownership of it.
    ///
    /// This method consumes the span and returns a guard that owns the span.
    /// The span will be automatically exited and closed when the guard is dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::Span;
    ///
    /// let span = Span::new("operation", &[]);
    /// let _guard = span.entered();
    /// // span is now active and owned by the guard
    /// // span is automatically exited and closed when _guard is dropped
    /// ```
    pub fn entered(self) -> SpanGuard {
        #[cfg(not(feature = "enable"))]
        {
            SpanGuard::noop()
        }

        #[cfg(feature = "enable")]
        {
            let Some(span_id) = self.span_id else {
                return SpanGuard::noop();
            };

            self.do_enter();
            CURRENT_SPAN
                .try_with(|current| {
                    let parent = current.get();
                    current.set(Some(span_id));

                    SpanGuard::new(self, parent)
                })
                .unwrap_or(SpanGuard::noop())
        }
    }

    /// Adds an event to this span.
    ///
    /// Events represent point-in-time occurrences within a span's lifetime.
    /// They can include additional attributes for context.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the event
    /// * `attributes` - Key-value attributes providing additional context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::{KeyValue, Span, Value};
    ///
    /// let span = Span::new("database_query", &[]);
    /// span.add_event("query_started", &[]);
    /// span.add_event("query_completed", &[KeyValue::new("rows_returned", Value::I64(42))]);
    /// ```
    pub fn add_event(&self, name: &'static str, attributes: &'_ [KeyValue<'static>]) {
        #[cfg(not(feature = "enable"))]
        {
            let _ = (name, attributes);
        }

        #[cfg(feature = "enable")]
        {
            if let Some(span_id) = self.span_id {
                get_collector().span_event(SpanAddEventMessage {
                    span_id,
                    name: name.into(),
                    time_unix_nano: now().as_nanos(),
                    attributes: attributes.into(),
                });
            }
        }
    }

    /// Creates a link from this span to another span.
    ///
    /// Links connect spans across different traces, allowing you to represent
    /// relationships between spans that are not parent-child relationships.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_telemetry::{Span, SpanContext, SpanId, ProcessId};
    ///
    /// let span = Span::new("my_span", &[]);
    /// let external_context = SpanContext::new(ProcessId::from_raw(0x123), SpanId(0x456));
    /// span.add_link(external_context);
    /// ```
    pub fn add_link(&self, link: SpanContext) {
        #[cfg(not(feature = "enable"))]
        {
            let _ = link;
        }

        #[cfg(feature = "enable")]
        {
            if let Some(span_id) = self.span_id {
                get_collector().span_link(SpanAddLinkMessage { span_id, link });
            }
        }
    }

    /// Adds an attribute to this span.
    ///
    /// Attributes provide additional context about the work being performed
    /// in the span. They can be set at any time during the span's lifetime.
    ///
    /// # Arguments
    ///
    /// * `attribute` - The key-value attribute to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::{KeyValue, Span, Value};
    ///
    /// let span = Span::new("user_operation", &[]);
    /// span.set_attribute(KeyValue::new("user_id", Value::I64(123)));
    /// span.set_attribute(KeyValue::new("operation_type", Value::String("update".into())));
    /// ```
    pub fn set_attribute(&self, attribute: KeyValue<'static>) {
        #[cfg(not(feature = "enable"))]
        {
            let _ = attribute;
        }

        #[cfg(feature = "enable")]
        {
            if let Some(span_id) = self.span_id {
                get_collector().span_attribute(SpanSetAttributeMessage { span_id, attribute });
            }
        }
    }
}

impl CurrentSpan {
    /// Adds an event to the current span.
    ///
    /// Events represent point-in-time occurrences within a span's lifetime.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the event
    /// * `attributes` - Key-value attributes providing additional context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::{CurrentSpan, KeyValue, Value, span};
    ///
    /// let _guard = span!("operation").entered();
    /// CurrentSpan::add_event("checkpoint", &[]);
    /// CurrentSpan::add_event("milestone", &[KeyValue::new("progress", 75)]);
    /// ```
    ///
    /// Does nothing if there's no active span.
    pub fn add_event(name: &'static str, attributes: &'_ [KeyValue<'static>]) {
        #[cfg(not(feature = "enable"))]
        {
            let _ = (name, attributes);
        }

        #[cfg(feature = "enable")]
        {
            if let Some(context) = SpanContext::current() {
                get_collector().span_event(SpanAddEventMessage {
                    span_id: context.span_id,
                    name: name.into(),
                    time_unix_nano: now().as_nanos(),
                    attributes: attributes.into(),
                });
            }
        }
    }

    /// Creates a link from the current span to another span.
    /// Does nothing if there's no active span.
    ///
    /// Links connect spans across different traces, allowing you to represent
    /// relationships between spans that are not parent-child relationships.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_telemetry::{CurrentSpan, Span, SpanContext, SpanId, ProcessId};
    ///
    /// let _guard = Span::new("my_span", &[]).entered();
    ///
    /// let external_context = SpanContext::new(ProcessId::from_raw(0x123), SpanId(0x456));
    /// CurrentSpan::add_link(external_context);
    /// ```
    pub fn add_link(link: SpanContext) {
        #[cfg(not(feature = "enable"))]
        {
            let _ = link;
        }

        #[cfg(feature = "enable")]
        {
            if let Some(context) = SpanContext::current() {
                get_collector().span_link(SpanAddLinkMessage {
                    span_id: context.span_id,
                    link,
                });
            }
        }
    }

    /// Sets an attribute on the current span.
    ///
    /// Attributes provide additional context about the work being performed
    /// in the span.
    ///
    /// # Arguments
    ///
    /// * `attribute` - The key-value attribute to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::{CurrentSpan, KeyValue, Value, span};
    ///
    /// let _guard = span!("operation").entered();
    /// CurrentSpan::set_attribute(KeyValue::new("user_id", 123));
    /// CurrentSpan::set_attribute(KeyValue::new("status", "success"));
    /// ```
    ///
    /// Does nothing if there's no active span.
    pub fn set_attribute(attribute: KeyValue<'static>) {
        #[cfg(not(feature = "enable"))]
        {
            let _ = attribute;
        }

        #[cfg(feature = "enable")]
        {
            if let Some(context) = SpanContext::current() {
                get_collector().span_attribute(SpanSetAttributeMessage {
                    span_id: context.span_id,
                    attribute,
                });
            }
        }
    }
}

#[cfg(feature = "enable")]
impl Span {
    fn new_inner(name: &'static str, attributes: &'_ [KeyValue<'static>]) -> Self {
        let span_id = SpanId::next_id();
        let parent_span_id = CURRENT_SPAN.get();

        get_collector().new_span(SpanCreateMessage {
            span_id,
            parent_span_id,
            name: name.into(),
            start_time_unix_nano: now().as_nanos(),
            attributes: attributes.into(),
        });

        Self {
            span_id: Some(span_id),
        }
    }

    fn do_enter(&self) {
        #[cfg(feature = "enable")]
        if let Some(span_id) = self.span_id {
            let timestamp = now();
            get_collector().enter_span(SpanEnterMessage {
                span_id,
                time_unix_nano: timestamp.0,
            });
        }
    }

    fn do_exit(&self) {
        #[cfg(feature = "enable")]
        if let Some(span_id) = self.span_id {
            let timestamp = now();
            get_collector().exit_span(SpanExitMessage {
                span_id,
                time_unix_nano: timestamp.0,
            });
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        #[cfg(feature = "enable")]
        if let Some(span_id) = self.span_id.take() {
            let timestamp = now();
            get_collector().close_span(SpanCloseMessage {
                span_id,
                end_time_unix_nano: timestamp.0,
            });
        }
    }
}

/// Exits and drops the span when this is dropped.
#[derive(Debug)]
pub struct SpanGuard {
    #[cfg(feature = "enable")]
    pub(crate) inner: Option<SpanGuardInner>,

    /// ```compile_fail
    /// use veecle_telemetry::span::*;
    /// trait AssertSend: Send {}
    ///
    /// impl AssertSend for SpanGuard {}
    /// ```
    _not_send: PhantomNotSend,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
pub(crate) struct SpanGuardInner {
    span: Span,
    parent: Option<SpanId>,
}

impl SpanGuard {
    pub(crate) fn noop() -> Self {
        Self {
            #[cfg(feature = "enable")]
            inner: None,
            _not_send: PhantomNotSend,
        }
    }

    #[cfg(feature = "enable")]
    pub(crate) fn new(span: Span, parent: Option<SpanId>) -> Self {
        Self {
            #[cfg(feature = "enable")]
            inner: Some(SpanGuardInner { span, parent }),
            _not_send: PhantomNotSend,
        }
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        #[cfg(feature = "enable")]
        if let Some(inner) = self.inner.take() {
            let _ = CURRENT_SPAN.try_with(|current| current.replace(inner.parent));
            inner.span.do_exit();
        }
    }
}

/// Exits the span when dropped.
#[derive(Debug)]
pub struct SpanGuardRef<'a> {
    #[cfg(feature = "enable")]
    pub(crate) inner: Option<SpanGuardRefInner<'a>>,

    _phantom: PhantomData<&'a ()>,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
pub(crate) struct SpanGuardRefInner<'a> {
    span: &'a Span,
    parent: Option<SpanId>,
}

impl<'a> SpanGuardRef<'a> {
    pub(crate) fn noop() -> Self {
        Self {
            #[cfg(feature = "enable")]
            inner: None,
            _phantom: PhantomData,
        }
    }

    #[cfg(feature = "enable")]
    pub(crate) fn new(span: &'a Span, parent: Option<SpanId>) -> Self {
        Self {
            #[cfg(feature = "enable")]
            inner: Some(SpanGuardRefInner { span, parent }),
            _phantom: PhantomData,
        }
    }
}

impl Drop for SpanGuardRef<'_> {
    fn drop(&mut self) {
        #[cfg(feature = "enable")]
        if let Some(inner) = self.inner.take() {
            let _ = CURRENT_SPAN.try_with(|current| current.replace(inner.parent));
            inner.span.do_exit();
        }
    }
}

/// Technically, `SpanGuard` _can_ implement both `Send` *and*
/// `Sync` safely. It doesn't, because it has a `PhantomNotSend` field,
/// specifically added in order to make it `!Send`.
///
/// Sending an `SpanGuard` guard between threads cannot cause memory unsafety.
/// However, it *would* result in incorrect behavior, so we add a
/// `PhantomNotSend` to prevent it from being sent between threads. This is
/// because it must be *dropped* on the same thread that it was created;
/// otherwise, the span will never be exited on the thread where it was entered,
/// and it will attempt to exit the span on a thread that may never have entered
/// it. However, we still want them to be `Sync` so that a struct holding an
/// `Entered` guard can be `Sync`.
///
/// Thus, this is totally safe.
#[derive(Debug)]
struct PhantomNotSend {
    ghost: PhantomData<*mut ()>,
}

#[allow(non_upper_case_globals)]
const PhantomNotSend: PhantomNotSend = PhantomNotSend { ghost: PhantomData };

/// # Safety:
///
/// Trivially safe, as `PhantomNotSend` doesn't have any API.
unsafe impl Sync for PhantomNotSend {}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::{ProcessId, SpanContext, SpanId};

    #[test]
    fn span_noop() {
        let span = Span::noop();
        assert!(span.span_id.is_none());
    }

    #[test]
    fn span_new_without_parent() {
        CURRENT_SPAN.set(None);

        let span = Span::new("test_span", &[]);
        assert!(span.span_id.is_some());
    }

    #[test]
    fn span_new_with_parent() {
        let parent_span_id = SpanId::next_id();
        CURRENT_SPAN.set(Some(parent_span_id));

        let span = Span::new("child_span", &[]);
        let span_id = span.span_id.unwrap();
        assert_ne!(span_id, parent_span_id);

        CURRENT_SPAN.set(None);
    }

    #[test]
    fn span_root() {
        let span = Span::new("root_span", &[]);
        let span_id = span.span_id.unwrap();
        assert_ne!(span_id, SpanId(0));
    }

    #[test]
    fn span_context_from_span() {
        let span = Span::new("test_span", &[]);

        let extracted_context = span.context();
        let context = extracted_context.unwrap();
        assert_eq!(context.process_id, get_collector().process_id());
    }

    #[test]
    fn span_context_from_noop_span() {
        let span = Span::noop();
        let extracted_context = span.context();
        assert!(extracted_context.is_none());
    }

    #[test]
    fn span_enter_and_current_context() {
        CURRENT_SPAN.set(None);

        assert!(SpanContext::current().is_none());

        let span = Span::new("test_span", &[]);

        {
            let _guard = span.enter();
            assert_eq!(SpanContext::current().unwrap(), span.context().unwrap());
        }

        // After guard is dropped, should be back to no current context
        assert!(SpanContext::current().is_none());
    }

    #[test]
    fn span_entered_guard() {
        CURRENT_SPAN.set(None);

        let span = Span::new("test_span", &[]);

        {
            let _guard = span.entered();
            // Should have current context while guard exists
            let current_context = SpanContext::current();
            assert!(current_context.is_some());
        }

        // Should be cleared after guard is dropped
        assert!(SpanContext::current().is_none());
    }

    #[test]
    fn noop_span_operations() {
        let noop_span = Span::noop();

        {
            let _guard = noop_span.enter();
            assert!(SpanContext::current().is_none());
        }

        let _entered_guard = noop_span.entered();
        assert!(SpanContext::current().is_none());
    }

    #[test]
    fn nested_spans() {
        CURRENT_SPAN.set(None);

        let root_span = Span::new("test_span", &[]);
        let root_context = root_span.context().unwrap();
        let _root_guard = root_span.entered();

        let child_span = Span::new("child", &[]);
        assert_ne!(child_span.context().unwrap().span_id, root_context.span_id);
    }

    #[test]
    fn span_event() {
        let span = Span::new("test_span", &[]);

        let event_attributes = [KeyValue::new("event_key", "event_value")];

        span.add_event("test_event", &event_attributes);

        let noop_span = Span::noop();
        noop_span.add_event("noop_event", &event_attributes);
    }

    #[test]
    fn span_link() {
        let span = Span::new("test_span", &[]);

        let link_context = SpanContext::new(ProcessId::from_raw(0), SpanId(0));
        span.add_link(link_context);

        let noop_span = Span::noop();
        noop_span.add_link(link_context);
    }

    #[test]
    fn span_attribute() {
        let span = Span::new("test_span", &[]);

        let attribute = KeyValue::new("test_key", "test_value");
        span.set_attribute(attribute.clone());

        let noop_span = Span::noop();
        noop_span.set_attribute(attribute);
    }

    #[test]
    fn span_methods_with_entered_span() {
        let span = Span::new("test_span", &[]);

        let _guard = span.enter();

        // All these should work while span is entered
        span.add_event("entered_event", &[]);
        span.add_link(SpanContext::new(ProcessId::from_raw(0), SpanId(0)));
        span.set_attribute(KeyValue::new("entered_key", true));
    }

    #[test]
    fn current_span_event_with_active_span() {
        CURRENT_SPAN.set(None);

        let _root_guard = Span::new("test_span", &[]).entered();

        let event_attributes = [KeyValue::new("current_event_key", "current_event_value")];
        CurrentSpan::add_event("current_test_event", &event_attributes);
    }

    #[test]
    fn current_span_link_with_active_span() {
        CURRENT_SPAN.set(None);

        let _root_guard = Span::new("test_span", &[]).entered();

        let link_context = SpanContext::new(ProcessId::from_raw(0), SpanId(0));
        CurrentSpan::add_link(link_context);
    }

    #[test]
    fn current_span_attribute_with_active_span() {
        CURRENT_SPAN.set(None);

        let span = Span::new("test_span", &[]);

        let _guard = span.enter();
        let attribute = KeyValue::new("current_attr_key", "current_attr_value");
        CurrentSpan::set_attribute(attribute);
    }
}
