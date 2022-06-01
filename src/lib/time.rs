use chrono::{DateTime, NaiveDateTime, Utc};

#[inline]
#[must_use]
pub fn datetime_unix(seconds: i64, nanoseconds: u32) -> DateTime<Utc> {
    DateTime::from_utc(NaiveDateTime::from_timestamp(seconds, nanoseconds), Utc)
}

#[inline]
#[must_use]
pub fn datetime_unix_seconds(seconds: i64) -> DateTime<Utc> {
    datetime_unix(seconds, 0)
}

#[inline]
#[must_use]
pub fn unix_epoch() -> DateTime<Utc> {
    datetime_unix(0, 0)
}
