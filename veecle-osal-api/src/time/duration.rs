//! This module implements a [`Duration`] with microsecond precision.

use core::fmt;
use core::num::TryFromIntError;
use core::ops::{Add, Div, Mul, Sub};

/// Duration represents a span of time.
///
/// Negative durations are not supported. [`Duration`] is not meant to be used
/// for math operations.
#[derive(
    Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Duration {
    micros: u64,
}

impl Duration {
    /// The largest value that can be represented by the `Duration` type.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::MAX, Duration::from_micros(u64::MAX));
    /// ```
    pub const MAX: Duration = Duration { micros: u64::MAX };

    /// A duration of zero time.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::ZERO, Duration::from_micros(0));
    /// ```
    pub const ZERO: Duration = Duration { micros: 0 };

    /// Factor of microseconds per second.
    const MICROS_PER_SECOND: u64 = 1_000_000;
    /// Factor of milliseconds per second.
    const MILLIS_PER_SECOND: u64 = 1_000;

    /// Creates a duration from the specified number of seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1), Duration::from_millis(1000));
    /// ```
    pub const fn from_secs(secs: u64) -> Duration {
        Duration {
            micros: secs * Self::MICROS_PER_SECOND,
        }
    }

    /// Creates a duration from the specified number of milliseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1), Duration::from_millis(1000));
    /// ```
    pub const fn from_millis(millis: u64) -> Duration {
        Duration {
            micros: millis * Self::MILLIS_PER_SECOND,
        }
    }

    /// Creates a duration from the specified number of microseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1), Duration::from_micros(1000000));
    /// ```
    pub const fn from_micros(micros: u64) -> Duration {
        Duration { micros }
    }

    /// Returns the total amount of seconds, rounded down.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_millis(1980).as_secs(), 1);
    /// ```
    pub const fn as_secs(&self) -> u64 {
        self.micros / Self::MICROS_PER_SECOND
    }

    /// Returns the total amount of milliseconds, rounded down.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_millis(1980).as_millis(), 1980);
    /// ```
    pub const fn as_millis(&self) -> u64 {
        self.micros / Self::MILLIS_PER_SECOND
    }

    /// Returns the total amount of microseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_millis(1980).as_micros(), 1980000);
    /// ```
    pub const fn as_micros(&self) -> u64 {
        self.micros
    }

    /// Adds one Duration to another, returning a new Duration or None in the event of an overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(
    ///     Duration::from_secs(1).checked_add(Duration::from_secs(1)),
    ///     Some(Duration::from_secs(2))
    /// );
    /// assert_eq!(Duration::MAX.checked_add(Duration::from_secs(1)), None);
    /// ```
    pub fn checked_add(self, rhs: Duration) -> Option<Duration> {
        self.micros
            .checked_add(rhs.micros)
            .map(|micros| Duration { micros })
    }

    /// Subtracts one Duration from another, returning a new Duration or None in the event of an underflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(
    ///     Duration::from_secs(2).checked_sub(Duration::from_secs(1)),
    ///     Some(Duration::from_secs(1))
    /// );
    /// assert_eq!(Duration::from_secs(1).checked_sub(Duration::from_secs(2)), None);
    /// ```
    pub fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        self.micros
            .checked_sub(rhs.micros)
            .map(|micros| Duration { micros })
    }

    /// Multiplies one Duration by a scalar `u32`, returning a new Duration or None in the event of an overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1).checked_mul(2), Some(Duration::from_secs(2)));
    /// assert_eq!(Duration::MAX.checked_mul(2), None);
    /// ```
    pub fn checked_mul(self, rhs: u32) -> Option<Duration> {
        self.micros
            .checked_mul(rhs as _)
            .map(|micros| Duration { micros })
    }

    /// Divides one Duration by a scalar `u32`, returning a new Duration or None if `rhs == 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1).checked_div(2), Some(Duration::from_millis(500)));
    /// assert_eq!(Duration::from_secs(1).checked_div(0), None);
    /// ```
    pub fn checked_div(self, rhs: u32) -> Option<Duration> {
        self.micros
            .checked_div(rhs as _)
            .map(|micros| Duration { micros })
    }

    /// Returns the absolute difference between self and rhs.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1).abs_diff(Duration::from_secs(2)), Duration::from_secs(1));
    /// assert_eq!(Duration::from_secs(2).abs_diff(Duration::from_secs(1)), Duration::from_secs(1));
    /// ```
    pub fn abs_diff(self, rhs: Self) -> Duration {
        Self {
            micros: self.micros.abs_diff(rhs.micros),
        }
    }

    /// Returns a duration of approximately 50 years.
    ///
    /// No leap-seconds/days have been taken into account.
    ///
    /// Can be used in place of [MAX][Self::MAX] to avoid overflows.
    pub(crate) fn max_no_overflow_alias() -> Self {
        // seconds * minutes * hours * days * weeks * years
        Self::from_secs(60 * 60 * 24 * 7 * 52 * 50)
    }
}

impl Add for Duration {
    type Output = Self;

    /// # Panics
    ///
    /// This function may panic if the resulting duration overflows. See [`Duration::checked_add`] for a version
    /// without panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1) + Duration::from_secs(1), Duration::from_secs(2));
    /// ```
    ///
    /// ```should_panic
    /// use veecle_osal_api::time::Duration;
    ///
    /// let _ = Duration::MAX + Duration::from_secs(1);
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        let Some(result) = self.checked_add(rhs) else {
            panic!("overflow when adding two durations");
        };

        result
    }
}

impl Sub for Duration {
    type Output = Self;

    /// # Panics
    ///
    /// This function may panic if the resulting duration underflows. See [`Duration::checked_sub`] for a
    /// version without panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(2) - Duration::from_secs(1), Duration::from_secs(1));
    /// ```
    ///
    /// ```should_panic
    /// use veecle_osal_api::time::Duration;
    ///
    /// let _ = Duration::from_secs(1) - Duration::from_secs(2);
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        let Some(result) = self.checked_sub(rhs) else {
            panic!("underflow when subtracting two durations");
        };

        result
    }
}

impl Mul<u32> for Duration {
    type Output = Self;

    /// # Panics
    ///
    /// This function may panic if the resulting duration overflows. See [`Duration::checked_mul`] for a version
    /// without panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1) * 2, Duration::from_secs(2));
    /// ```
    ///
    /// ```should_panic
    /// use veecle_osal_api::time::Duration;
    ///
    /// let _ = Duration::MAX * 2;
    /// ```
    fn mul(self, rhs: u32) -> Self::Output {
        let Some(result) = self.checked_mul(rhs) else {
            panic!("overflow when multiplying a duration by a scalar");
        };

        result
    }
}

impl Mul<Duration> for u32 {
    type Output = Duration;

    /// # Panics
    ///
    /// This function may panic if the resulting duration overflows. See [`Duration::checked_mul`] for a version
    /// without panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(2 * Duration::from_secs(1), Duration::from_secs(2));
    /// ```
    ///
    /// ```should_panic
    /// use veecle_osal_api::time::Duration;
    ///
    /// let _ = 2 * Duration::MAX;
    /// ```
    fn mul(self, rhs: Duration) -> Self::Output {
        rhs * self
    }
}

impl Div<u32> for Duration {
    type Output = Self;

    /// # Panics
    ///
    /// This function may panic if the duration is divided by zero. See [`Duration::checked_div`] for a version
    /// without panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1) / 2, Duration::from_millis(500));
    /// ```
    ///
    /// ```should_panic
    /// use veecle_osal_api::time::Duration;
    ///
    /// let _ = Duration::from_secs(1) / 0;
    /// ```
    fn div(self, rhs: u32) -> Self::Output {
        let Some(result) = self.checked_div(rhs) else {
            panic!("divided a duration by zero");
        };

        result
    }
}

impl fmt::Debug for Duration {
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// let duration = Duration::from_millis(1980);
    /// assert_eq!(format!("{duration:?}"), "1s.980000us");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}s.{}us",
            self.as_secs(),
            self.as_micros() % Self::MICROS_PER_SECOND
        )
    }
}

impl TryFrom<core::time::Duration> for Duration {
    type Error = TryFromIntError;

    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(Duration::try_from(core::time::Duration::from_secs(1)), Ok(Duration::from_secs(1)));
    /// ```
    fn try_from(value: core::time::Duration) -> Result<Self, Self::Error> {
        value.as_micros().try_into().map(Self::from_micros)
    }
}

impl From<Duration> for core::time::Duration {
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::Duration;
    ///
    /// assert_eq!(
    ///     core::time::Duration::from(Duration::from_secs(1)),
    ///     core::time::Duration::from_secs(1)
    /// );
    /// ```
    fn from(value: Duration) -> Self {
        Self::from_micros(value.as_micros())
    }
}
