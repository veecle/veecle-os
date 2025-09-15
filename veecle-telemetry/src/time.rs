//! Time utilities for telemetry timestamping.
//!
//! This module provides time-related functionality for telemetry data collection.
//! It abstracts over different time sources depending on the platform and
//! feature configuration.
//!
//! # Platform Support
//!
//! - `std`: Uses system time with high precision
//! - `freertos`: Uses FreeRTOS time utilities
//! - `no_std`: Uses monotonic time sources
//!
//! # Timestamp Format
//!
//! All timestamps are represented as nanoseconds since an epoch.
//! The specific epoch depends on the platform and configuration:
//!
//! - With `system_time` feature: Unix epoch (1970-01-01 00:00:00 UTC)
//! - Without `system_time` feature: Arbitrary system start time

#[cfg(all(feature = "enable", feature = "freertos", not(feature = "std")))]
pub(crate) use veecle_osal_freertos::time::*;
#[cfg(all(feature = "enable", feature = "std"))]
pub(crate) use veecle_osal_std::time::*;

/// A timestamp with nanosecond resolution.
///
/// The value might be relative to the UNIX epoch or an arbitrary moment in time if the system time has not been synced
/// yet.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn as_nanos(&self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(feature = "enable")]
pub(crate) fn now() -> Timestamp {
    #[cfg(feature = "system_time")]
    let timestamp_micros = match Time::duration_since_epoch() {
        Ok(duration) => duration.as_micros(),
        Err(SystemTimeError::Unsynchronized) => Time::now()
            .duration_since(Instant::MIN)
            .expect("should be able to get a duration since the MIN value")
            .as_micros(),
        Err(SystemTimeError::EpochIsLaterThanStartTime) => {
            panic!(
                "Failed to get duration since epoch: {:?}",
                SystemTimeError::EpochIsLaterThanStartTime
            );
        }
    };

    #[cfg(not(feature = "system_time"))]
    let timestamp_micros = Time::now()
        .duration_since(Instant::MIN)
        .expect("should be able to get a duration since the MIN value")
        .as_micros();

    Timestamp(timestamp_micros * 1000)
}
