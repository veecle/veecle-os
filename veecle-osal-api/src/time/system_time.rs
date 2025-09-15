//! Provides system time functionality with support for time synchronization.

use crate::time::Duration;

/// Provides a measurement of the system time.
pub trait SystemTime {
    /// Calculates the [`Duration`] elapsed since epoch.
    ///
    /// ## Notes
    ///
    /// OSAL implementations might require time to be synchronized by using the [`SystemTimeSync`] before using this
    /// function. Review the documentation on the implementation you are using to learn platform-specific information.
    ///
    /// ## Errors
    ///
    /// - [`SystemTimeError::Unsynchronized`] when system was not synchronized previously.
    fn duration_since_epoch() -> Result<Duration, SystemTimeError>;
}

/// Allows manual synchronization of [`SystemTime`].
pub trait SystemTimeSync {
    /// Updates the current system time.
    ///
    /// Use this to synchronize with real-world clock (e.g., provide time obtained from NTP).
    ///
    /// ## Errors
    ///
    /// - [`SystemTimeError::EpochIsLaterThanStartTime`] when the provided duration is less than the program's execution
    ///   time.
    fn set_system_time(elapsed_from_epoch: Duration) -> Result<(), SystemTimeError>;
}

/// An error that may happen while working with System Time.
#[derive(Debug, Eq, PartialEq)]
pub enum SystemTimeError {
    /// Occurs when an attempt is made to get system time, but it was not synchronized earlier.
    Unsynchronized,
    /// Occurs when an attempt is made to synchronize using an epoch time that is later than the program's start time.
    EpochIsLaterThanStartTime,
}

impl core::error::Error for SystemTimeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            SystemTimeError::Unsynchronized => None,
            SystemTimeError::EpochIsLaterThanStartTime => None,
        }
    }
}

impl core::fmt::Display for SystemTimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SystemTimeError::Unsynchronized => write!(f, "{self:?}"),
            SystemTimeError::EpochIsLaterThanStartTime => write!(f, "{self:?}"),
        }
    }
}
