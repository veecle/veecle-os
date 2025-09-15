//! This module implements an [`Instant`] with microsecond precision.

use core::fmt;
use core::num::NonZeroU64;
use core::ops::{Add, Sub};

use super::Duration;

/// An Instant in time. Instants should be always increasing and are
/// generally obtainable through the operating system time driver.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    micros: NonZeroU64,
}

impl Instant {
    /// The largest value that can be represented by the [`Instant`] type.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    /// use veecle_osal_std::time::Time;
    ///
    /// let now = Time::now();
    ///
    /// assert!(Instant::MAX > now);
    /// ```
    pub const MAX: Instant = Instant {
        micros: NonZeroU64::MAX,
    };

    /// The smallest value that can be represented by the [`Instant`] type.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    /// use veecle_osal_std::time::Time;
    ///
    /// let now = Time::now();
    ///
    /// assert!(Instant::MIN <= now);
    /// ```
    pub const MIN: Instant = Instant {
        micros: NonZeroU64::MIN,
    };

    /// Returns the [`Duration`] between this [`Instant`] and the give one if, and only if,
    /// the given one is earlier than this, otherwise returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    /// use veecle_osal_std::time::Time;
    ///
    /// let begin = Time::now();
    ///
    /// std::thread::sleep(core::time::Duration::from_millis(1));
    ///
    /// let end = Time::now();
    /// assert!(end.duration_since(begin).unwrap() > Duration::ZERO);
    /// assert_eq!(begin.duration_since(end), None);
    /// ```
    pub const fn duration_since(&self, earlier: Instant) -> Option<Duration> {
        if self.micros.get() < earlier.micros.get() {
            return None;
        }

        Some(Duration::from_micros(
            self.micros.get() - earlier.micros.get(),
        ))
    }

    /// Adds one [`Duration`] to self, returning a new [`Instant`] or None in the event of an overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    /// use veecle_osal_std::time::Time;
    ///
    /// let now = Time::now();
    ///
    /// assert!(now.checked_add(Duration::from_secs(1)).unwrap() > now);
    /// assert_eq!(now.checked_add(Duration::MAX), None);
    /// ```
    pub fn checked_add(self, rhs: Duration) -> Option<Instant> {
        self.micros
            .checked_add(rhs.as_micros())
            .map(|micros| Instant { micros })
    }

    /// Subs one [`Duration`] from self, returning a new [`Instant`] or None in the event of an underflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    /// use veecle_osal_std::time::Time;
    ///
    /// let begin = Time::now();
    ///
    /// std::thread::sleep(core::time::Duration::from_millis(1));
    ///
    /// let end = Time::now();
    ///
    /// assert!(end.checked_sub(Duration::from_millis(1)).unwrap() < end);
    /// assert_eq!(end.checked_sub(Duration::MAX), None);
    /// ```
    pub fn checked_sub(&self, rhs: Duration) -> Option<Instant> {
        self.micros
            .get()
            .checked_sub(rhs.as_micros())
            .and_then(NonZeroU64::new)
            .map(|micros| Instant { micros })
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    /// # Panics
    ///
    /// This function may panic if the resulting instant overflows. See [`Instant::checked_add`] for a version
    /// without panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    /// use veecle_osal_std::time::Time;
    ///
    /// let now = Time::now();
    ///
    /// assert!(now + Duration::from_secs(1) > now);
    /// ```
    ///
    /// ```should_panic
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    /// use veecle_osal_std::time::Time;
    ///
    /// let now = Time::now();
    ///
    /// let _ = now + Duration::MAX;
    /// ```
    fn add(self, rhs: Duration) -> Self::Output {
        let Some(result) = self.checked_add(rhs) else {
            panic!("overflow when adding a duration to an instant");
        };

        result
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;

    /// # Panics
    ///
    /// This function may panic if the resulting instant underflows. See [`Instant::checked_sub`] for a
    /// version without panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    /// use veecle_osal_std::time::Time;
    ///
    /// let begin = Time::now();
    ///
    /// std::thread::sleep(core::time::Duration::from_millis(1));
    ///
    /// let end = Time::now();
    ///
    /// assert!(end - Duration::from_millis(1) < end);
    /// ```
    ///
    /// ```should_panic
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    /// use veecle_osal_std::time::Time;
    ///
    /// let now = Time::now();
    ///
    /// let _ = now - Duration::MAX;
    /// ```
    fn sub(self, rhs: Duration) -> Self::Output {
        let Some(result) = self.checked_sub(rhs) else {
            panic!("underflow when subtracting a duration from an instant");
        };

        result
    }
}

impl fmt::Debug for Instant {
    /// # Examples
    ///
    /// ```
    /// use veecle_osal_api::time::{Duration, Instant, TimeAbstraction};
    ///
    /// let instant = Instant::MIN + Duration::from_millis(1980);
    /// assert_eq!(format!("{instant:?}"), "1s.980000us");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.duration_since(Self::MIN)
            .expect("instant should be at least Instant::MIN")
            .fmt(f)
    }
}
