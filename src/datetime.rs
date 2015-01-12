// This is a part of rust-chrono.
// Copyright (c) 2014-2015, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!
 * ISO 8601 date and time with time zone.
 */

use std::{fmt, hash};
use std::cmp::Ordering;
use std::ops::{Add, Sub};

use {Weekday, Timelike, Datelike};
use offset::{TimeZone, Offset};
use duration::Duration;
use naive::datetime::NaiveDateTime;
use time::Time;
use date::Date;
use format::DelayedFormat;

/// ISO 8601 combined date and time with time zone.
#[derive(Clone)]
pub struct DateTime<Tz: TimeZone> {
    datetime: NaiveDateTime,
    offset: Tz::Offset,
}

impl<Tz: TimeZone> DateTime<Tz> {
    /// Makes a new `DateTime` with given *UTC* datetime and offset.
    /// The local datetime should be constructed via the `TimeZone` trait.
    //
    // note: this constructor is purposedly not named to `new` to discourage the direct usage.
    #[inline]
    pub fn from_utc(datetime: NaiveDateTime, offset: Tz::Offset) -> DateTime<Tz> {
        DateTime { datetime: datetime, offset: offset }
    }

    /// Retrieves a date component.
    #[inline]
    pub fn date(&self) -> Date<Tz> {
        Date::from_utc(self.datetime.date().clone(), self.offset.clone())
    }

    /// Retrieves a time component.
    #[inline]
    pub fn time(&self) -> Time<Tz> {
        Time::from_utc(self.datetime.time().clone(), self.offset.clone())
    }

    /// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC.
    #[inline]
    pub fn num_seconds_from_unix_epoch(&self) -> i64 {
        self.datetime.num_seconds_from_unix_epoch()
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
    /// This does not change the actual `DateTime` (but will change the string representation).
    #[inline]
    pub fn with_timezone<Tz2: TimeZone>(&self, tz: &Tz2) -> DateTime<Tz2> {
        tz.from_utc_datetime(&self.datetime)
    }

    /// Returns a view to the naive UTC datetime.
    #[inline]
    pub fn naive_utc(&self) -> NaiveDateTime {
        self.datetime
    }

    /// Returns a view to the naive local datetime.
    #[inline]
    pub fn naive_local(&self) -> NaiveDateTime {
        self.datetime + self.offset.local_minus_utc()
    }
}

/// Maps the local datetime to other datetime with given conversion function.
fn map_local<Tz: TimeZone, F>(dt: &DateTime<Tz>, mut f: F) -> Option<DateTime<Tz>>
        where F: FnMut(NaiveDateTime) -> Option<NaiveDateTime> {
    f(dt.naive_local()).and_then(|datetime| dt.timezone().from_local_datetime(&datetime).single())
}

impl<Tz: TimeZone> DateTime<Tz> where Tz::Offset: fmt::String {
    /// Formats the combined date and time in the specified format string.
    /// See the `format` module on the supported escape sequences.
    #[inline]
    pub fn format<'a>(&'a self, fmt: &'a str) -> DelayedFormat<'a> {
        let local = self.naive_local();
        DelayedFormat::new_with_offset(Some(local.date()), Some(local.time()), &self.offset, fmt)
    }
}

impl<Tz: TimeZone> Datelike for DateTime<Tz> {
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
    fn with_year(&self, year: i32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_year(year))
    }

    #[inline]
    fn with_month(&self, month: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_month(month))
    }

    #[inline]
    fn with_month0(&self, month0: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_month0(month0))
    }

    #[inline]
    fn with_day(&self, day: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_day(day))
    }

    #[inline]
    fn with_day0(&self, day0: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_day0(day0))
    }

    #[inline]
    fn with_ordinal(&self, ordinal: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_ordinal(ordinal))
    }

    #[inline]
    fn with_ordinal0(&self, ordinal0: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_ordinal0(ordinal0))
    }
}

impl<Tz: TimeZone> Timelike for DateTime<Tz> {
    #[inline] fn hour(&self) -> u32 { self.naive_local().hour() }
    #[inline] fn minute(&self) -> u32 { self.naive_local().minute() }
    #[inline] fn second(&self) -> u32 { self.naive_local().second() }
    #[inline] fn nanosecond(&self) -> u32 { self.naive_local().nanosecond() }

    #[inline]
    fn with_hour(&self, hour: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_hour(hour))
    }

    #[inline]
    fn with_minute(&self, min: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_minute(min))
    }

    #[inline]
    fn with_second(&self, sec: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_second(sec))
    }

    #[inline]
    fn with_nanosecond(&self, nano: u32) -> Option<DateTime<Tz>> {
        map_local(self, |datetime| datetime.with_nanosecond(nano))
    }
}

impl<Tz: TimeZone, Tz2: TimeZone> PartialEq<DateTime<Tz2>> for DateTime<Tz> {
    fn eq(&self, other: &DateTime<Tz2>) -> bool { self.datetime == other.datetime }
}

impl<Tz: TimeZone> Eq for DateTime<Tz> {
}

impl<Tz: TimeZone> PartialOrd for DateTime<Tz> {
    fn partial_cmp(&self, other: &DateTime<Tz>) -> Option<Ordering> {
        self.datetime.partial_cmp(&other.datetime)
    }
}

impl<Tz: TimeZone> Ord for DateTime<Tz> {
    fn cmp(&self, other: &DateTime<Tz>) -> Ordering { self.datetime.cmp(&other.datetime) }
}

impl<Tz: TimeZone, H: hash::Hasher + hash::Writer> hash::Hash<H> for DateTime<Tz> {
    fn hash(&self, state: &mut H) { self.datetime.hash(state) }
}

impl<Tz: TimeZone> Add<Duration> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    fn add(self, rhs: Duration) -> DateTime<Tz> {
        DateTime { datetime: self.datetime + rhs, offset: self.offset }
    }
}

impl<Tz: TimeZone, Tz2: TimeZone> Sub<DateTime<Tz2>> for DateTime<Tz> {
    type Output = Duration;

    fn sub(self, rhs: DateTime<Tz2>) -> Duration { self.datetime - rhs.datetime }
}

impl<Tz: TimeZone> Sub<Duration> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    #[inline]
    fn sub(self, rhs: Duration) -> DateTime<Tz> { self.add(-rhs) }
}

impl<Tz: TimeZone> fmt::Show for DateTime<Tz> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{:?}", self.naive_local(), self.offset)
    }
}

impl<Tz: TimeZone> fmt::String for DateTime<Tz> where Tz::Offset: fmt::String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.naive_local(), self.offset)
    }
}

#[cfg(test)]
mod tests {
    use {Datelike};
    use duration::Duration;
    use offset::TimeZone;
    use offset::utc::UTC;
    use offset::local::Local;
    use offset::fixed::FixedOffset;

    #[test]
    #[allow(non_snake_case)]
    fn test_datetime_offset() {
        let EST = FixedOffset::east(5*60*60);
        let EDT = FixedOffset::east(4*60*60);

        assert_eq!(format!("{}", UTC.ymd(2014, 5, 6).and_hms(7, 8, 9)),
                   "2014-05-06 07:08:09 UTC");
        assert_eq!(format!("{}", EDT.ymd(2014, 5, 6).and_hms(7, 8, 9)),
                   "2014-05-06 07:08:09 +04:00");
        assert_eq!(format!("{:?}", UTC.ymd(2014, 5, 6).and_hms(7, 8, 9)),
                   "2014-05-06T07:08:09Z");
        assert_eq!(format!("{:?}", EDT.ymd(2014, 5, 6).and_hms(7, 8, 9)),
                   "2014-05-06T07:08:09+04:00");

        assert_eq!(UTC.ymd(2014, 5, 6).and_hms(7, 8, 9), EDT.ymd(2014, 5, 6).and_hms(11, 8, 9));
        assert_eq!(UTC.ymd(2014, 5, 6).and_hms(7, 8, 9) + Duration::seconds(3600 + 60 + 1),
                   UTC.ymd(2014, 5, 6).and_hms(8, 9, 10));
        assert_eq!(UTC.ymd(2014, 5, 6).and_hms(7, 8, 9) - EDT.ymd(2014, 5, 6).and_hms(10, 11, 12),
                   Duration::seconds(3600 - 3*60 - 3));

        assert_eq!(*UTC.ymd(2014, 5, 6).and_hms(7, 8, 9).offset(), UTC);
        assert_eq!(*EDT.ymd(2014, 5, 6).and_hms(7, 8, 9).offset(), EDT);
        assert!(*EDT.ymd(2014, 5, 6).and_hms(7, 8, 9).offset() != EST);
    }

    #[test]
    fn test_datetime_fmt_with_local() {
        // if we are not around the year boundary, local and UTC date should have the same year
        let dt = Local::now().with_month(5).unwrap();
        assert_eq!(dt.format("%Y").to_string(), dt.with_timezone(&UTC).format("%Y").to_string());
    }
}

