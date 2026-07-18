use serde::{Deserialize, Serialize};

use crate::errors::define;

/// A time (hour, minute, and second) within a single day.
/// Represented as a number of seconds from the beginning of the day.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct Time(u32);

impl Time {
    /// Creates a new `Time` from offset from the beginning of the day in seconds.
    ///
    /// # Errors
    ///
    /// Returns an error if got more than maximum seconds in a day.
    ///
    /// # Example
    ///
    /// ```
    /// # use wtimer_lib::domain::time::Time;
    /// # use wtimer_lib::domain::time::SecondsError;
    ///
    /// assert_eq!(Time::new(0).unwrap().seconds(), 0); // 00:00:00
    /// assert_eq!(Time::new(86_399).unwrap().seconds(), 86_399); // 23:59:59
    /// assert!(matches!(Time::new(86_400), Err(SecondsError)));
    /// ```
    pub fn new(seconds: u32) -> Result<Self, SecondsError> {
        if seconds >= SECONDS_PER_DAY {
            return Err(SecondsError);
        }
        Ok(Self(seconds))
    }

    /// Creates a new `Time` from specified anoumt of hours, minutes, and seconds.
    ///
    /// # Errors
    ///
    /// Returns an error on invalid amount of hours, minutes, or seconds.
    ///
    /// # Example
    ///
    /// ```
    /// # use wtimer_lib::domain::time::Time;
    /// # use wtimer_lib::domain::time::TimeError;
    ///
    /// assert_eq!(Time::from_hms(0, 0, 0).unwrap().seconds(), 0);
    /// assert_eq!(Time::from_hms(23, 59, 59).unwrap().seconds(), 86_399);
    /// assert!(matches!(Time::from_hms(24, 00, 00), Err(TimeError::Hours)));
    /// assert!(matches!(Time::from_hms(23, 60, 00), Err(TimeError::Minutes)));
    /// assert!(matches!(Time::from_hms(23, 59, 60), Err(TimeError::Seconds)));
    /// ```
    pub fn from_hms(h: u32, m: u32, s: u32) -> Result<Self, TimeError> {
        Self::validate(h, 23, TimeError::Hours)?;
        Self::validate(m, 59, TimeError::Minutes)?;
        Self::validate(s, 59, TimeError::Seconds)?;

        let seconds = h * SECONDS_PER_HOUR + m * SECONDS_PER_MINUTE + s;
        Ok(Self(seconds))
    }

    pub const MIN: Time = Time(0);
    pub const MAX: Time = Time(SECONDS_PER_DAY);

    pub fn seconds(&self) -> u32 {
        return self.0;
    }

    #[inline]
    fn validate(x: u32, max: u32, err: TimeError) -> Result<(), TimeError> {
        if x > max {
            return Err(err);
        }
        Ok(())
    }
}

define!(SecondsError, "exceeds the max number of seconds per day");

#[derive(Debug, thiserror::Error)]
pub enum TimeError {
    #[error("hours should be in [0..23]")]
    Hours,
    #[error("minutes should be in [0..59]")]
    Minutes,
    #[error("seconds should be in [0..59]")]
    Seconds,
}

const SECONDS_PER_DAY: u32 = 24 * SECONDS_PER_HOUR;
const SECONDS_PER_HOUR: u32 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_MINUTE: u32 = 60;

const _: () = {
    if SECONDS_PER_DAY != 86_400 {
        panic!("Seconds per day should equal to 86_400");
    }
};
