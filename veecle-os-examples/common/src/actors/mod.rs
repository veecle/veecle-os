//! Actors shared between multiple examples.

// `telemetry` includes `alloc`, but requiring both is more explicit and less likely to get mistakenly refactored.
#[cfg(all(feature = "alloc", feature = "telemetry"))]
pub mod alloc;

pub mod ping_pong;
pub mod tcp;
pub mod time;
pub mod udp;
