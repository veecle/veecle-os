//! Safe wrapper for SOME/IP test service.

mod config;
mod endpoint;
mod ipc;
mod subprocess;
mod test_service;

pub use config::test_service::{Config, LoggingLevel};
pub use test_service::TestService;

#[doc(hidden)]
/// Private API, do not use.
// Re-exports used in private binary exclusively by this crate.
pub mod reëxports {
    pub mod ipc {
        pub use crate::ipc::*;
    }
    pub mod config {
        pub use crate::config::common_api::Config as CommonApiConfig;
        pub use crate::config::vsomeip::Config as VSomeIpConfig;
    }
}
