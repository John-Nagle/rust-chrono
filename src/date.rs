// This is a part of rust-chrono.
// Copyright (c) 2014-2015, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!
 * ISO 8601 calendar date with time zone.
 */

use std::{fmt, hash};
use std::cmp::Ordering;
use std::ops::{Add, Sub};

use {Weekday, Datelike};
use duration::Duration;
use offset::{TimeZone, Offset};
use offset::utc::UTC;
use naive;
use naive::date::NaiveDate;
use naive::time::NaiveTime;
use datetime::DateTime;
use format::DelayedFormat;

/// ISO 8601 calendar date with time zone.
#[derive(Clone)]
pub struct Date<Tz: TimeZone> {
    date: NaiveDate,
    offset: Tz::Offset,
}

/// The minimum possible `Date`.
pub const MIN: Date<UTC> = Date { date: naive::date::MIN, offset: UTC };
/// The maximum possible `Date`.
pub const MAX: Date<UTC> = Date { date: naive::date::MAX, offset: UTC };

impl<Tz: TimeZone> Date<Tz> {
    /// Makes a new `Date` with given *UTC* date and offset.
    /// The local date should be constructed via the `TimeZone` trait.
    //
    // note: this constructor is purposedly not named to `new` to discourage the direct usage.
    #[inline]
    pub fn from_utc(date: NaiveDate, offset: Tz::Offset) -> Date<Tz> {
        Date { date: date, offset: offset }
    }

    /// Makes a new `DateTime` from the current date and given `NaiveTime`.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid datetime.
    #[inline]
    pub fn and_time(&self, time: NaiveTime) -> Option<DateTime<Tz>> {
        let localdt = self.naive_local().and_time(time);
        self.timezone().from_local_datetime(&localdt).single()
    }

    /// Makes a new `DateTime` from the current date, hour, minute and second.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid hour, minute and/or second.
    #[inline]
    pub fn and_hms(&self, hour: u32, min: u32, sec: u32) -> DateTime<Tz> {
        self.and_hms_opt(hour, min, sec).expect("invalid time")
    }

    /// Makes a new `DateTime` from the current date, hour, minute and second.
    /// The offset in the current date is preserved.
    ///
    /// Returns `None` on invalid hour, minute and/or second.
    #[inline]
    pub fn and_hms_opt(&self, hour: u32, min: u32, sec: u32) -> Option<DateTime<Tz>> {
        NaiveTime::from_hms_opt(hour, min, sec).and_then(|time| self.and_time(time))
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and millisecond.
    /// The millisecond part can exceed 1,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid hour, minute, second and/or millisecond.
    #[inline]
    pub fn and_hms_milli(&self, hour: u32, min: u32, sec: u32, milli: u32) -> DateTime<Tz> {
        self.and_hms_milli_opt(hour, min, sec, milli).expect("invalid time")
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and millisecond.
    /// The millisecond part can exceed 1,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Returns `None` on invalid hour, minute, second and/or millisecond.
    #[inline]
    pub fn and_hms_milli_opt(&self, hour: u32, min: u32, sec: u32,
                             milli: u32) -> Option<DateTime<Tz>> {
        NaiveTime::from_hms_milli_opt(hour, min, sec, milli).and_then(|time| self.and_time(time))
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and microsecond.
    /// The microsecond part can exceed 1,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid hour, minute, second and/or microsecond.
    #[inline]
    pub fn and_hms_micro(&self, hour: u32, min: u32, sec: u32, micro: u32) -> DateTime<Tz> {
        self.and_hms_micro_opt(hour, min, sec, micro).expect("invalid time")
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and microsecond.
    /// The microsecond part can exceed 1,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Returns `None` on invalid hour, minute, second and/or microsecond.
    #[inline]
    pub fn and_hms_micro_opt(&self, hour: u32, min: u32, sec: u32,
                             micro: u32) -> Option<DateTime<Tz>> {
        NaiveTime::from_hms_micro_opt(hour, min, sec, micro).and_then(|time| self.and_time(time))
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and nanosecond.
    /// The nanosecond part can exceed 1,000,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid hour, minute, second and/or nanosecond.
    #[inline]
    pub fn and_hms_nano(&self, hour: u32, min: u32, sec: u32, nano: u32) -> DateTime<Tz> {
        self.and_hms_nano_opt(hour, min, sec, nano).expect("invalid time")
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and nanosecond.
    /// The nanosecond part can exceed 1,000,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Returns `None` on invalid hour, minute, second and/or nanosecond.
    #[inline]
    pub fn and_hms_nano_opt(&self, hour: u32, min: u32, sec: u32,
                            nano: u32) -> Option<DateTime<Tz>> {
        NaiveTime::from_hms_nano_opt(hour, min, sec, nano).and_then(|time| self.and_time(time))
    }

    /// Makes a new `Date` for the next date.
    ///
    /// Fails when `self` is the last representable date.
    #[inline]
    pub fn succ(&self) -> Date<Tz> {
        self.succ_opt().expect("out of bound")
    }

    /// Makes a new `Date` for the next date.
    ///
    /// Returns `None` when `self` is the last representable date.
    #[inline]
    pub fn succ_opt(&self) -> Option<Date<Tz>> {
        self.date.succ_opt().map(|date| Date::from_utc(date, self.offset.clone()))
    }

    /// Makes a new `Date` for the prior date.
    ///
    /// Fails when `self` is the first representable date.
    #[inline]
    pub fn pred(&self) -> Date<Tz> {
        self.pred_opt().expect("out of bound")
    }

    /// Makes a new `Date` for the prior date.
    ///
    /// Returns `None` when `self` is the first representable date.
    #[inline]
    pub fn pred_opt(&self) -> Option<Date<Tz>> {
        self.date.pred_opt().map(|date| Date::from_utc(date, self.offset.clone()))
    }

    /// Retrieves an associated offset from UTC.
    #[inline]
    pub fn offset<'a>(&'a self) -> &'a Tz::Offset {
        &self.offset
    }

    /// Retrieves an associated time zone.
    #[inline]
    pub fn timezone(&self) -> Tz {
        TimeZone::from_offset(&self.offset)
    }

    /// Changes the associated time zone.
    /// This does not change the actual `Date` (but will change the string representation).
    #[inline]
    pub fn with_timezone<Tz2: TimeZone>(&self, tz: &Tz2) -> Date<Tz2> {
        tz.from_utc_date(&self.date)
    }

    /// Returns a view to the naive UTC date.
    #[inline]
    pub fn naive_utc(&self) -> NaiveDate {
        self.date
    }

    /// Returns a view to the naive local date.
    #[inline]
    pub fn naive_local(&self) -> NaiveDate {
        self.date + self.offset.local_minus_utc()
    }
}

/// Maps the local date to other date with given conversion function.
fn map_local<Tz: TimeZone, F>(d: &Date<Tz>, mut f: F) -> Option<Date<Tz>>
        where F: FnMut(NaiveDate) -> Option<NaiveDate> {
    f(d.naive_local()).and_then(|date| d.timezone().from_local_date(&date).single())
}

impl<Tz: TimeZone> Date<Tz> where Tz::Offset: fmt::String {
    /// Formats the date in the specified format string.
    /// See the `format` module on the supported escape sequences.
    #[inline]
    pub fn format<'a>(&'a self, fmt: &'a str) -> DelayedFormat<'a> {
        DelayedFormat::new_with_offset(Some(self.naive_local()), None, &self.offset, fmt)
    }
}

impl<Tz: TimeZone> Datelike for Date<Tz> {
    #[inline] fn year(&self) -> i32 { self.naive_local().year() }
    #[inline] fn month(&self) -> u32 { self.naive_local().month() }
    #[inline] fn month0(&self) -> u32 { self.naive_local().month0() }
    #[inline] fn day(&self) -> u32 { self.naive_local().day() }
    #[inline] fn day0(&self) -> u32 { self.naive_local().day0() }
    #[inline] fn ordinal(&self) -> u32 { self.naive_local().ordinal() }
    #[inline] fn ordinal0(&self) -> u32 { self.naive_local().ordinal0() }
    #[inline] fn weekday(&self) -> Weekday { self.naive_local().weekday() }
    #[inline] fn isoweekdate(&self) -> (i32, u32, Weekday) { self.naive_local().isoweekdate() }

    #[inline]
    fn with_year(&self, year: i32) -> Option<Date<Tz>> {
        map_local(self, |date| date.with_year(year))
    }

    #[inline]
    fn with_month(&self, month: u32) -> Option<Date<Tz>> {
        map_local(self, |date| date.with_month(month))
    }

    #[inline]
    fn with_month0(&self, month0: u32) -> Option<Date<Tz>> {
        map_local(self, |date| date.with_month0(month0))
    }

    #[inline]
    fn with_day(&self, day: u32) -> Option<Date<Tz>> {
        map_local(self, |date| date.with_day(day))
    }

    #[inline]
    fn with_day0(&self, day0: u32) -> Option<Date<Tz>> {
        map_local(self, |date| date.with_day0(day0))
    }

    #[inline]
    fn with_ordinal(&self, ordinal: u32) -> Option<Date<Tz>> {
        map_local(self, |date| date.with_ordinal(ordinal))
    }

    #[inline]
    fn with_ordinal0(&self, ordinal0: u32) -> Option<Date<Tz>> {
        map_local(self, |date| date.with_ordinal0(ordinal0))
    }
}

impl<Tz: TimeZone, Tz2: TimeZone> PartialEq<Date<Tz2>> for Date<Tz> {
    fn eq(&self, other: &Date<Tz2>) -> bool { self.date == other.date }
}

impl<Tz: TimeZone> Eq for Date<Tz> {
}

impl<Tz: TimeZone> PartialOrd for Date<Tz> {
    fn partial_cmp(&self, other: &Date<Tz>) -> Option<Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

impl<Tz: TimeZone> Ord for Date<Tz> {
    fn cmp(&self, other: &Date<Tz>) -> Ordering { self.date.cmp(&other.date) }
}

impl<Tz: TimeZone, H: hash::Hasher + hash::Writer> hash::Hash<H> for Date<Tz> {
    fn hash(&self, state: &mut H) { self.date.hash(state) }
}

impl<Tz: TimeZone> Add<Duration> for Date<Tz> {
    type Output = Date<Tz>;

    fn add(self, rhs: Duration) -> Date<Tz> {
        Date { date: self.date + rhs, offset: self.offset }
    }
}

impl<Tz: TimeZone, Tz2: TimeZone> Sub<Date<Tz2>> for Date<Tz> {
    type Output = Duration;

    fn sub(self, rhs: Date<Tz2>) -> Duration { self.date - rhs.date }
}

impl<Tz: TimeZone> Sub<Duration> for Date<Tz> {
    type Output = Date<Tz>;

    #[inline]
    fn sub(self, rhs: Duration) -> Date<Tz> { self.add(-rhs) }
}

impl<Tz: TimeZone> fmt::Show for Date<Tz> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{:?}", self.naive_local(), self.offset)
    }
}

impl<Tz: TimeZone> fmt::String for Date<Tz> where Tz::Offset: fmt::String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.naive_local(), self.offset)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use duration::Duration;
    use naive::date::NaiveDate;
    use naive::time::NaiveTime;
    use naive::datetime::NaiveDateTime;
    use offset::{TimeZone, Offset, LocalResult};

    #[derive(Copy, Clone, PartialEq, Eq)]
    struct UTC1y; // same to UTC but with an offset of 365 days

    #[derive(Copy, Clone, PartialEq, Eq)]
    struct OneYear;

    impl TimeZone for UTC1y {
        type Offset = OneYear;

        fn from_offset(_offset: &OneYear) -> UTC1y { UTC1y }

        fn offset_from_local_date(&self, _local: &NaiveDate) -> LocalResult<OneYear> {
            LocalResult::Single(OneYear)
        }
        fn offset_from_local_time(&self, _local: &NaiveTime) -> LocalResult<OneYear> {
            LocalResult::Single(OneYear)
        }
        fn offset_from_local_datetime(&self, _local: &NaiveDateTime) -> LocalResult<OneYear> {
            LocalResult::Single(OneYear)
        }

        fn offset_from_utc_date(&self, _utc: &NaiveDate) -> OneYear { OneYear }
        fn offset_from_utc_time(&self, _utc: &NaiveTime) -> OneYear { OneYear }
        fn offset_from_utc_datetime(&self, _utc: &NaiveDateTime) -> OneYear { OneYear }
    }

    impl Offset for OneYear {
        fn local_minus_utc(&self) -> Duration { Duration::days(365) }
    }

    impl fmt::Show for OneYear {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "+8760:00") }
    }

    #[test]
    fn test_date_weird_offset() {
        assert_eq!(format!("{:?}", UTC1y.ymd(2012, 2, 29)),
                   "2012-02-29+8760:00".to_string());
        assert_eq!(format!("{:?}", UTC1y.ymd(2012, 2, 29).and_hms(5, 6, 7)),
                   "2012-02-29T05:06:07+8760:00".to_string());
        assert_eq!(format!("{:?}", UTC1y.ymd(2012, 3, 4)),
                   "2012-03-04+8760:00".to_string());
        assert_eq!(format!("{:?}", UTC1y.ymd(2012, 3, 4).and_hms(5, 6, 7)),
                   "2012-03-04T05:06:07+8760:00".to_string());
    }
}

