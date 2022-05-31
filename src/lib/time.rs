use chrono::{DateTime, NaiveDateTime, Utc};

pub fn unix_epoch() -> DateTime<Utc> {
    DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
}
