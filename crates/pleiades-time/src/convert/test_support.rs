//! Shared setup for the forward and inverse conversion tests.

use pleiades_types::JulianDay;

use crate::calendar::CivilDateTime;
use crate::leap;

/// One inserted leap second, derived from the leap table.
pub(crate) struct Insertion {
    /// The UTC day that ends with the leap second, at `00:00:00`.
    pub last_day: CivilDateTime,
    /// The following UTC day, at `00:00:00`.
    pub next_day: CivilDateTime,
    /// `TAI − UTC` in force up to and including the leap second.
    pub offset_before: i32,
}

/// Every insertion in the leap table, ascending.
pub(crate) fn insertions() -> Vec<Insertion> {
    leap::table()
        .unwrap()
        .windows(2)
        .map(|pair| Insertion {
            last_day: CivilDateTime::from_julian_day(JulianDay::from_days(pair[1].0 - 1.0)),
            next_day: CivilDateTime::from_julian_day(JulianDay::from_days(pair[1].0)),
            offset_before: pair[0].1,
        })
        .collect()
}

/// `date` with its time of day replaced.
pub(crate) fn at(date: CivilDateTime, hour: u8, minute: u8, second: f64) -> CivilDateTime {
    CivilDateTime::new(date.year, date.month, date.day, hour, minute, second)
}
