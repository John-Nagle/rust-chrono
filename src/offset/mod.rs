// This is a part of rust-chrono.
// Copyright (c) 2014-2015, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!
 * Offsets from the local time to UTC.
 *
 * There are three operations provided by the `Offset` trait:
 *
 * 1. Converting the local `NaiveDateTime` to `DateTime<Offset>`
 * 2. Converting the UTC `NaiveDateTime` to `DateTime<Offset>`
 * 3. Converting `DateTime<Offset>` to the local `NaiveDateTime`
 *
 * 1 is used for constructors. 2 is used for the `with_offset` method of date and time types.
 * 3 is used for other methods, e.g. `year()` or `format()`, and provided by an associated type
 * which implements `OffsetState` (which then passed to `Offset` for actual implementations).
 * Technically speaking `Offset` has a total knowledge about given timescale,
 * but `OffsetState` is used as a cache to avoid the repeated conversion
 * and provides implementations for 1 and 3.
 * An `Offset` instance can be reconstructed from the corresponding `OffsetState` instance.
 */

use std::fmt;

use Weekday;
use duration::Duration;
use naive::date::NaiveDate;
use naive::time::NaiveTime;
use naive::datetime::NaiveDateTime;
use date::Date;
use time::Time;
use datetime::DateTime;

/// The conversion result from the local time to the timezone-aware datetime types.
#[derive(Clone, PartialEq, Show)]
pub enum LocalResult<T> {
    /// Given local time representation is invalid.
    /// This can occur when, for example, the positive timezone transition.
    None,
    /// Given local time representation has a single unique result.
    Single(T),
    /// Given local time representation has multiple results and thus ambiguous.
    /// This can occur when, for example, the negative timezone transition.
    Ambiguous(T /*min*/, T /*max*/),
}

impl<T> LocalResult<T> {
    /// Returns `Some` only when the conversion result is unique, or `None` otherwise.
    pub fn single(self) -> Option<T> {
        match self { LocalResult::Single(t) => Some(t), _ => None }
    }

    /// Returns `Some` for the earliest possible conversion result, or `None` if none.
    pub fn earliest(self) -> Option<T> {
        match self { LocalResult::Single(t) | LocalResult::Ambiguous(t,_) => Some(t), _ => None }
    }

    /// Returns `Some` for the latest possible conversion result, or `None` if none.
    pub fn latest(self) -> Option<T> {
        match self { LocalResult::Single(t) | LocalResult::Ambiguous(_,t) => Some(t), _ => None }
    }

    /// Maps a `LocalResult<T>` into `LocalResult<U>` with given function.
    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> LocalResult<U> {
        match self {
            LocalResult::None => LocalResult::None,
            LocalResult::Single(v) => LocalResult::Single(f(v)),
            LocalResult::Ambiguous(min, max) => LocalResult::Ambiguous(f(min), f(max)),
        }
    }
}

impl<Off: Offset> LocalResult<Date<Off>> {
    /// Makes a new `DateTime` from the current date and given `NaiveTime`.
    /// The offset in the current date is preserved.
    ///
    /// Propagates any error. Ambiguous result would be discarded.
    #[inline]
    pub fn and_time(self, time: NaiveTime) -> LocalResult<DateTime<Off>> {
        match self {
            LocalResult::Single(d) => d.and_time(time)
                                       .map_or(LocalResult::None, LocalResult::Single),
            _ => LocalResult::None,
        }
    }

    /// Makes a new `DateTime` from the current date, hour, minute and second.
    /// The offset in the current date is preserved.
    ///
    /// Propagates any error. Ambiguous result would be discarded.
    #[inline]
    pub fn and_hms_opt(self, hour: u32, min: u32, sec: u32) -> LocalResult<DateTime<Off>> {
        match self {
            LocalResult::Single(d) => d.and_hms_opt(hour, min, sec)
                                       .map_or(LocalResult::None, LocalResult::Single),
            _ => LocalResult::None,
        }
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and millisecond.
    /// The millisecond part can exceed 1,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Propagates any error. Ambiguous result would be discarded.
    #[inline]
    pub fn and_hms_milli_opt(self, hour: u32, min: u32, sec: u32,
                             milli: u32) -> LocalResult<DateTime<Off>> {
        match self {
            LocalResult::Single(d) => d.and_hms_milli_opt(hour, min, sec, milli)
                                       .map_or(LocalResult::None, LocalResult::Single),
            _ => LocalResult::None,
        }
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and microsecond.
    /// The microsecond part can exceed 1,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Propagates any error. Ambiguous result would be discarded.
    #[inline]
    pub fn and_hms_micro_opt(self, hour: u32, min: u32, sec: u32,
                             micro: u32) -> LocalResult<DateTime<Off>> {
        match self {
            LocalResult::Single(d) => d.and_hms_micro_opt(hour, min, sec, micro)
                                       .map_or(LocalResult::None, LocalResult::Single),
            _ => LocalResult::None,
        }
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and nanosecond.
    /// The nanosecond part can exceed 1,000,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Propagates any error. Ambiguous result would be discarded.
    #[inline]
    pub fn and_hms_nano_opt(self, hour: u32, min: u32, sec: u32,
                            nano: u32) -> LocalResult<DateTime<Off>> {
        match self {
            LocalResult::Single(d) => d.and_hms_nano_opt(hour, min, sec, nano)
                                       .map_or(LocalResult::None, LocalResult::Single),
            _ => LocalResult::None,
        }
    }

}

impl<T: fmt::Show> LocalResult<T> {
    /// Returns the single unique conversion result, or fails accordingly.
    pub fn unwrap(self) -> T {
        match self {
            LocalResult::None => panic!("No such local time"),
            LocalResult::Single(t) => t,
            LocalResult::Ambiguous(t1,t2) => {
                panic!("Ambiguous local time, ranging from {:?} to {:?}", t1, t2)
            }
        }
    }
}

/// The offset state.
pub trait OffsetState: Sized + Clone + fmt::Show {
    /// Returns the offset from UTC to the local time stored in the offset state.
    fn local_minus_utc(&self) -> Duration;
}

/// The offset from the local time to UTC.
pub trait Offset: Sized {
    type State: OffsetState;

    /// Makes a new `Date` from year, month, day and the current offset.
    /// This assumes the proleptic Gregorian calendar, with the year 0 being 1 BCE.
    ///
    /// The offset normally does not affect the date (unless it is between UTC-24 and UTC+24),
    /// but it will propagate to the `DateTime` values constructed via this date.
    ///
    /// Fails on the out-of-range date, invalid month and/or day.
    fn ymd(&self, year: i32, month: u32, day: u32) -> Date<Self> {
        self.ymd_opt(year, month, day).unwrap()
    }

    /// Makes a new `Date` from year, month, day and the current offset.
    /// This assumes the proleptic Gregorian calendar, with the year 0 being 1 BCE.
    ///
    /// The offset normally does not affect the date (unless it is between UTC-24 and UTC+24),
    /// but it will propagate to the `DateTime` values constructed via this date.
    ///
    /// Returns `None` on the out-of-range date, invalid month and/or day.
    fn ymd_opt(&self, year: i32, month: u32, day: u32) -> LocalResult<Date<Self>> {
        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(d) => self.from_local_date(&d),
            None => LocalResult::None,
        }
    }

    /// Makes a new `Date` from year, day of year (DOY or "ordinal") and the current offset.
    /// This assumes the proleptic Gregorian calendar, with the year 0 being 1 BCE.
    ///
    /// The offset normally does not affect the date (unless it is between UTC-24 and UTC+24),
    /// but it will propagate to the `DateTime` values constructed via this date.
    ///
    /// Fails on the out-of-range date and/or invalid DOY.
    fn yo(&self, year: i32, ordinal: u32) -> Date<Self> {
        self.yo_opt(year, ordinal).unwrap()
    }

    /// Makes a new `Date` from year, day of year (DOY or "ordinal") and the current offset.
    /// This assumes the proleptic Gregorian calendar, with the year 0 being 1 BCE.
    ///
    /// The offset normally does not affect the date (unless it is between UTC-24 and UTC+24),
    /// but it will propagate to the `DateTime` values constructed via this date.
    ///
    /// Returns `None` on the out-of-range date and/or invalid DOY.
    fn yo_opt(&self, year: i32, ordinal: u32) -> LocalResult<Date<Self>> {
        match NaiveDate::from_yo_opt(year, ordinal) {
            Some(d) => self.from_local_date(&d),
            None => LocalResult::None,
        }
    }

    /// Makes a new `Date` from ISO week date (year and week number), day of the week (DOW) and
    /// the current offset.
    /// This assumes the proleptic Gregorian calendar, with the year 0 being 1 BCE.
    /// The resulting `Date` may have a different year from the input year.
    ///
    /// The offset normally does not affect the date (unless it is between UTC-24 and UTC+24),
    /// but it will propagate to the `DateTime` values constructed via this date.
    ///
    /// Fails on the out-of-range date and/or invalid week number.
    fn isoywd(&self, year: i32, week: u32, weekday: Weekday) -> Date<Self> {
        self.isoywd_opt(year, week, weekday).unwrap()
    }

    /// Makes a new `Date` from ISO week date (year and week number), day of the week (DOW) and
    /// the current offset.
    /// This assumes the proleptic Gregorian calendar, with the year 0 being 1 BCE.
    /// The resulting `Date` may have a different year from the input year.
    ///
    /// The offset normally does not affect the date (unless it is between UTC-24 and UTC+24),
    /// but it will propagate to the `DateTime` values constructed via this date.
    ///
    /// Returns `None` on the out-of-range date and/or invalid week number.
    fn isoywd_opt(&self, year: i32, week: u32, weekday: Weekday) -> LocalResult<Date<Self>> {
        match NaiveDate::from_isoywd_opt(year, week, weekday) {
            Some(d) => self.from_local_date(&d),
            None => LocalResult::None,
        }
    }

    /// Makes a new `Time` from hour, minute, second and the current offset.
    ///
    /// Fails on invalid hour, minute and/or second.
    fn hms(&self, hour: u32, min: u32, sec: u32) -> Time<Self> {
        self.hms_opt(hour, min, sec).unwrap()
    }

    /// Makes a new `Time` from hour, minute, second and the current offset.
    ///
    /// Returns `None` on invalid hour, minute and/or second.
    fn hms_opt(&self, hour: u32, min: u32, sec: u32) -> LocalResult<Time<Self>> {
        match NaiveTime::from_hms_opt(hour, min, sec) {
            Some(t) => self.from_local_time(&t),
            None => LocalResult::None,
        }
    }

    /// Makes a new `Time` from hour, minute, second, millisecond and the current offset.
    /// The millisecond part can exceed 1,000 in order to represent the leap second.
    ///
    /// Fails on invalid hour, minute, second and/or millisecond.
    fn hms_milli(&self, hour: u32, min: u32, sec: u32, milli: u32) -> Time<Self> {
        self.hms_milli_opt(hour, min, sec, milli).unwrap()
    }

    /// Makes a new `Time` from hour, minute, second, millisecond and the current offset.
    /// The millisecond part can exceed 1,000 in order to represent the leap second.
    ///
    /// Returns `None` on invalid hour, minute, second and/or millisecond.
    fn hms_milli_opt(&self, hour: u32, min: u32, sec: u32, milli: u32) -> LocalResult<Time<Self>> {
        match NaiveTime::from_hms_milli_opt(hour, min, sec, milli) {
            Some(t) => self.from_local_time(&t),
            None => LocalResult::None,
        }
    }

    /// Makes a new `Time` from hour, minute, second, microsecond and the current offset.
    /// The microsecond part can exceed 1,000,000 in order to represent the leap second.
    ///
    /// Fails on invalid hour, minute, second and/or microsecond.
    fn hms_micro(&self, hour: u32, min: u32, sec: u32, micro: u32) -> Time<Self> {
        self.hms_micro_opt(hour, min, sec, micro).unwrap()
    }

    /// Makes a new `Time` from hour, minute, second, microsecond and the current offset.
    /// The microsecond part can exceed 1,000,000 in order to represent the leap second.
    ///
    /// Returns `None` on invalid hour, minute, second and/or microsecond.
    fn hms_micro_opt(&self, hour: u32, min: u32, sec: u32, micro: u32) -> LocalResult<Time<Self>> {
        match NaiveTime::from_hms_micro_opt(hour, min, sec, micro) {
            Some(t) => self.from_local_time(&t),
            None => LocalResult::None,
        }
    }

    /// Makes a new `Time` from hour, minute, second, nanosecond and the current offset.
    /// The nanosecond part can exceed 1,000,000,000 in order to represent the leap second.
    ///
    /// Fails on invalid hour, minute, second and/or nanosecond.
    fn hms_nano(&self, hour: u32, min: u32, sec: u32, nano: u32) -> Time<Self> {
        self.hms_nano_opt(hour, min, sec, nano).unwrap()
    }

    /// Makes a new `Time` from hour, minute, second, nanosecond and the current offset.
    /// The nanosecond part can exceed 1,000,000,000 in order to represent the leap second.
    ///
    /// Returns `None` on invalid hour, minute, second and/or nanosecond.
    fn hms_nano_opt(&self, hour: u32, min: u32, sec: u32, nano: u32) -> LocalResult<Time<Self>> {
        match NaiveTime::from_hms_nano_opt(hour, min, sec, nano) {
            Some(t) => self.from_local_time(&t),
            None => LocalResult::None,
        }
    }

    /// Reconstructs the offset from the offset state.
    fn from_state(state: &Self::State) -> Self;

    /// Creates the offset state(s) for given local `NaiveDate` if possible.
    fn state_from_local_date(&self, local: &NaiveDate) -> LocalResult<Self::State>;

    /// Creates the offset state(s) for given local `NaiveTime` if possible.
    fn state_from_local_time(&self, local: &NaiveTime) -> LocalResult<Self::State>;

    /// Creates the offset state(s) for given local `NaiveDateTime` if possible.
    fn state_from_local_datetime(&self, local: &NaiveDateTime) -> LocalResult<Self::State>;

    /// Converts the local `NaiveDate` to the timezone-aware `Date` if possible.
    fn from_local_date(&self, local: &NaiveDate) -> LocalResult<Date<Self>> {
        self.state_from_local_date(local).map(|state| {
            Date::from_utc(*local - state.local_minus_utc(), state)
        })
    }

    /// Converts the local `NaiveTime` to the timezone-aware `Time` if possible.
    fn from_local_time(&self, local: &NaiveTime) -> LocalResult<Time<Self>> {
        self.state_from_local_time(local).map(|state| {
            Time::from_utc(*local - state.local_minus_utc(), state)
        })
    }

    /// Converts the local `NaiveDateTime` to the timezone-aware `DateTime` if possible.
    fn from_local_datetime(&self, local: &NaiveDateTime) -> LocalResult<DateTime<Self>> {
        self.state_from_local_datetime(local).map(|state| {
            DateTime::from_utc(*local - state.local_minus_utc(), state)
        })
    }

    /// Creates the offset state for given UTC `NaiveDate`. This cannot fail.
    fn state_from_utc_date(&self, utc: &NaiveDate) -> Self::State;

    /// Creates the offset state for given UTC `NaiveTime`. This cannot fail.
    fn state_from_utc_time(&self, utc: &NaiveTime) -> Self::State;

    /// Creates the offset state for given UTC `NaiveDateTime`. This cannot fail.
    fn state_from_utc_datetime(&self, utc: &NaiveDateTime) -> Self::State;

    /// Converts the UTC `NaiveDate` to the local time.
    /// The UTC is continuous and thus this cannot fail (but can give the duplicate local time).
    fn from_utc_date(&self, utc: &NaiveDate) -> Date<Self> {
        Date::from_utc(utc.clone(), self.state_from_utc_date(utc))
    }

    /// Converts the UTC `NaiveTime` to the local time.
    /// The UTC is continuous and thus this cannot fail (but can give the duplicate local time).
    fn from_utc_time(&self, utc: &NaiveTime) -> Time<Self> {
        Time::from_utc(utc.clone(), self.state_from_utc_time(utc))
    }

    /// Converts the UTC `NaiveDateTime` to the local time.
    /// The UTC is continuous and thus this cannot fail (but can give the duplicate local time).
    fn from_utc_datetime(&self, utc: &NaiveDateTime) -> DateTime<Self> {
        DateTime::from_utc(utc.clone(), self.state_from_utc_datetime(utc))
    }
}

pub mod utc;
pub mod fixed;
pub mod local;

