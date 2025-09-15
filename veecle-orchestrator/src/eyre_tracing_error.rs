//! Adds [`SpanTrace`] captures to all [`eyre::Report`]s, with a lot less features and complications than
//! `color-eyre/spantrace`.
use std::fmt;

use eyre::{DefaultHandler, EyreHandler};
use tracing_error::{SpanTrace, SpanTraceStatus};

/// An [`EyreHandler`] that augments [`DefaultHandler`] with an additional [`SpanTrace`].
pub struct Handler {
    default_handler: Box<dyn EyreHandler>,
    span_trace: SpanTrace,
}

impl Handler {
    /// Provides a hook to install the handler via [`eyre::set_hook`].
    pub fn default_with(error: &(dyn std::error::Error + 'static)) -> Box<dyn EyreHandler> {
        Box::new(Handler {
            default_handler: DefaultHandler::default_with(error),
            span_trace: SpanTrace::capture(),
        })
    }
}

impl EyreHandler for Handler {
    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.default_handler.debug(error, f)?;
        if self.span_trace.status() == SpanTraceStatus::CAPTURED {
            write!(f, "\n\nSpans:\n{}", self.span_trace)?;
        }
        Ok(())
    }

    fn display(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.default_handler.display(error, f)?;
        Ok(())
    }

    fn track_caller(&mut self, location: &'static std::panic::Location<'static>) {
        self.default_handler.track_caller(location);
    }
}
