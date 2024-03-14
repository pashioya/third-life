use chrono::{NaiveDate, NaiveTime};

pub fn to_timestamp(date: &NaiveDate) -> i64 {
    date.and_time(NaiveTime::default()).timestamp_nanos_opt().unwrap()
}
