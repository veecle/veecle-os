use std::sync::{Arc, Mutex};
use std::vec::Vec;

use super::Export;
use crate::protocol::owned;
use crate::to_static::ToStatic;

/// An exporter for testing that stores all telemetry messages in memory.
///
/// This exporter is useful for unit tests and integration tests where you need
/// to verify that specific telemetry messages were generated.
#[derive(Debug)]
pub struct TestExporter {
    /// Shared vector storing all exported telemetry messages
    pub spans: Arc<Mutex<Vec<owned::InstanceMessage>>>,
}

impl TestExporter {
    /// Creates a new test exporter and returns both the exporter and a handle to the message storage.
    ///
    /// The returned tuple contains the exporter and a shared reference to the vector
    /// where all telemetry messages will be stored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::collector::TestExporter;
    ///
    /// let (exporter, messages) = TestExporter::new();
    /// // Use exporter for telemetry collection
    /// // Check messages for verification
    /// ```
    pub fn new() -> (Self, Arc<Mutex<Vec<owned::InstanceMessage>>>) {
        let spans = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                spans: spans.clone(),
            },
            spans,
        )
    }
}

impl Export for TestExporter {
    fn export(&self, message: crate::protocol::transient::InstanceMessage<'_>) {
        self.spans.lock().unwrap().push(message.to_static());
    }
}
