//! Proleptic-Gregorian civil datetime <-> Julian Day (Meeus, ch. 7).

use pleiades_types::JulianDay;

use crate::error::CivilTimeError;

/// A civil calendar datetime in the proleptic Gregorian calendar. Scale-agnostic:
/// the caller tags it UTC or UT1 at conversion time.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CivilDateTime {
    /// Proleptic-Gregorian calendar year (astronomical numbering; may be negative).
    pub year: i32,
    /// Calendar month, 1–12 (validated in `to_julian_day`).
    pub month: u8,
    /// Day of month, 1–31 (validated in `to_julian_day`).
    pub day: u8,
    /// Hour of day, 0–23.
    pub hour: u8,
    /// Minute of hour, 0–59.
    pub minute: u8,
    /// Second of minute, in `[0.0, 61.0)` (the upper bound admits leap seconds).
    pub second: f64,
}

impl CivilDateTime {
    /// Creates a civil datetime without validation. Validation happens in `to_julian_day`.
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: f64) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    fn validate(&self) -> Result<(), CivilTimeError> {
        if !(1..=12).contains(&self.month) {
            return Err(CivilTimeError::InvalidCivilDate { field: "month" });
        }
        if !(1..=31).contains(&self.day) {
            return Err(CivilTimeError::InvalidCivilDate { field: "day" });
        }
        if self.hour > 23 {
            return Err(CivilTimeError::InvalidCivilDate { field: "hour" });
        }
        if self.minute > 59 {
            return Err(CivilTimeError::InvalidCivilDate { field: "minute" });
        }
        if !self.second.is_finite() || self.second < 0.0 || self.second >= 61.0 {
            return Err(CivilTimeError::InvalidCivilDate { field: "second" });
        }
        Ok(())
    }

    /// Converts to a Julian Day using the proleptic-Gregorian Meeus formula.
    pub fn to_julian_day(&self) -> Result<JulianDay, CivilTimeError> {
        self.validate()?;
        let day_frac = self.day as f64
            + (self.hour as f64 + self.minute as f64 / 60.0 + self.second / 3600.0) / 24.0;
        let (y, m) = if self.month <= 2 {
            (self.year - 1, self.month as i32 + 12)
        } else {
            (self.year, self.month as i32)
        };
        let a = (y as f64 / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();
        let jd = (365.25 * (y as f64 + 4716.0)).floor()
            + (30.6001 * (m as f64 + 1.0)).floor()
            + day_frac
            + b
            - 1524.5;
        Ok(JulianDay::from_days(jd))
    }

    /// Inverse: reconstructs a civil datetime from a Julian Day (proleptic Gregorian).
    pub fn from_julian_day(jd: JulianDay) -> Self {
        let jd = jd.days() + 0.5;
        let z = jd.floor();
        let f = jd - z;
        let alpha = ((z - 1867216.25) / 36524.25).floor();
        let a = z + 1.0 + alpha - (alpha / 4.0).floor();
        let b = a + 1524.0;
        let c = ((b - 122.1) / 365.25).floor();
        let d = (365.25 * c).floor();
        let e = ((b - d) / 30.6001).floor();
        let day_frac = b - d - (30.6001 * e).floor() + f;
        let day = day_frac.floor();
        let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
        let year = if month > 2.0 { c - 4716.0 } else { c - 4715.0 };
        // Decompose the intra-day remainder via total seconds-of-day, rounding
        // the whole quantity to the nearest millisecond (the inverse conversion
        // carries ~0.1 ms of floating-point noise) BEFORE splitting into h/m/s.
        // Rounding the aggregate, then flooring each component, reconstructs a
        // round-tripped HH:MM:00 cleanly without the per-component-round bug
        // that corrupted sub-minute times (e.g. 19:20:30 -> 19:21:-30).
        let mut rem_seconds = ((day_frac - day) * 86_400.0 * 1_000.0).round() / 1_000.0;
        if rem_seconds >= 86_400.0 {
            rem_seconds = 86_399.999; // absorb the sub-ms round-up into the last representable second
        }
        let hour = (rem_seconds / 3_600.0).floor();
        rem_seconds -= hour * 3_600.0;
        let minute = (rem_seconds / 60.0).floor();
        let second = rem_seconds - minute * 60.0;
        Self {
            year: year as i32,
            month: month as u8,
            day: day as u8,
            hour: hour as u8,
            minute: minute as u8,
            second,
        }
    }
}

#[cfg(test)]
mod tests;
