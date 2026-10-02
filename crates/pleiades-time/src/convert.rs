//! Orchestrator: civil datetime + scales -> tagged TT/TDB Instant with provenance.

use core::fmt;

use pleiades_types::{Instant, JulianDay, TimeScale, SECONDS_PER_DAY};

use crate::calendar::CivilDateTime;
use crate::deltat::{self, DeltaTQuality, TT_MINUS_TAI};
use crate::error::CivilTimeError;
use crate::leap;
use crate::tdb;

mod inverse;

pub use inverse::{
    from_terrestrial, ut1_civil_from_tdb, ut1_civil_from_tt, utc_civil_from_tdb, utc_civil_from_tt,
    CivilConversion,
};

/// Start of the supported civil window (1900-01-01 00:00).
pub const SUPPORT_START_JD: f64 = 2415020.5;
/// End of the supported civil window: an exclusive upper bound at the start of
/// 2101 (JD 2488434.5). The last accepted instant is 2100-12-31T23:59:59.x;
/// 2101-01-01T00:00:00 is rejected as `BeyondHorizon`.
pub const SUPPORT_END_JD: f64 = 2488434.5;

const SOURCES: &str =
    "leap-seconds.csv (IERS Bulletin C); delta-t-observed.csv (IERS/USNO + Espenak–Meeus); as-of 2026-06";

/// Which path the orchestrator took.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConversionPath {
    /// UTC input converted via the leap-second table (exact).
    UtcLeapSecond,
    /// UT1 input converted via the observed Delta-T table.
    Ut1DeltaT,
    /// Input beyond a validated table, converted via Delta-T extrapolation.
    FutureExtrapolated,
}

impl fmt::Display for ConversionPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UtcLeapSecond => "utc-leap-second",
            Self::Ut1DeltaT => "ut1-delta-t",
            Self::FutureExtrapolated => "future-extrapolated",
        })
    }
}

/// Overall conversion quality, the truthful-claims marker on every result.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConversionQuality {
    /// Leap-second-exact (no Delta-T model error).
    Exact,
    /// From the observed Delta-T table.
    Observed,
    /// From Delta-T extrapolation.
    Predicted,
}

impl fmt::Display for ConversionQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Exact => "exact",
            Self::Observed => "observed",
            Self::Predicted => "predicted",
        })
    }
}

/// Provenance describing how a civil instant was converted.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConversionProvenance {
    /// Which orchestrator path produced the result.
    pub path: ConversionPath,
    /// Truthful accuracy tier of the result (`exact`/`observed`/`predicted`).
    pub quality: ConversionQuality,
    /// `ΔT = TT − UT1` in seconds when a Delta-T model was used; `None` on the
    /// leap-second-exact UTC path.
    pub delta_t_seconds: Option<f64>,
    /// `TAI − UTC` in whole seconds when the leap-second table was used; `None`
    /// on the UT1/Delta-T and extrapolated paths.
    pub tai_minus_utc: Option<i32>,
    /// Human-readable identification of the underlying data tables and as-of date.
    pub sources: &'static str,
}

impl ConversionProvenance {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        format!(
            "civil-time path={} quality={} delta_t={} tai_minus_utc={}",
            self.path,
            self.quality,
            self.delta_t_seconds
                .map(|d| format!("{d:.3}s"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.tai_minus_utc
                .map(|t| format!("{t}s"))
                .unwrap_or_else(|| "n/a".to_string()),
        )
    }
}

impl fmt::Display for ConversionProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A converted instant plus the provenance describing how it was produced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CivilInstant {
    /// The converted instant, tagged with its target time scale (TT or TDB).
    pub instant: Instant,
    /// How the instant was produced, including its truthful quality tier.
    pub provenance: ConversionProvenance,
}

fn finite(jd: f64) -> Result<(), CivilTimeError> {
    if jd.is_finite() {
        Ok(())
    } else {
        Err(CivilTimeError::NonFiniteOffset)
    }
}

/// Builds a TT instant (and provenance) from a civil Julian Day tagged UTC or UT1.
fn to_tt(
    jd_civil: f64,
    source: TimeScale,
    target: TimeScale,
) -> Result<(f64, ConversionProvenance), CivilTimeError> {
    if !(SUPPORT_START_JD..SUPPORT_END_JD).contains(&jd_civil) {
        return Err(CivilTimeError::BeyondHorizon { jd: jd_civil });
    }
    match source {
        TimeScale::Utc => {
            if jd_civil < leap::LEAP_EPOCH_JD {
                return Err(CivilTimeError::UtcBeforeLeapEpoch);
            }
            if let Some(tai_minus_utc) = leap::tai_minus_utc(jd_civil)? {
                let offset = tai_minus_utc as f64 + TT_MINUS_TAI;
                let jd_tt = jd_civil + offset / SECONDS_PER_DAY;
                finite(jd_tt)?;
                return Ok((
                    jd_tt,
                    ConversionProvenance {
                        path: ConversionPath::UtcLeapSecond,
                        quality: ConversionQuality::Exact,
                        delta_t_seconds: None,
                        tai_minus_utc: Some(tai_minus_utc),
                        sources: SOURCES,
                    },
                ));
            }
            // Future UTC beyond the leap table: fall back to Delta-T extrapolation.
            let (dt, _q) = deltat::delta_t(jd_civil)?;
            let jd_tt = jd_civil + dt / SECONDS_PER_DAY;
            finite(jd_tt)?;
            Ok((
                jd_tt,
                ConversionProvenance {
                    path: ConversionPath::FutureExtrapolated,
                    quality: ConversionQuality::Predicted,
                    delta_t_seconds: Some(dt),
                    tai_minus_utc: None,
                    sources: SOURCES,
                },
            ))
        }
        TimeScale::Ut1 => {
            let (dt, q) = deltat::delta_t(jd_civil)?;
            let jd_tt = jd_civil + dt / SECONDS_PER_DAY;
            finite(jd_tt)?;
            // The leap-second bound is observed data with a sub-second error
            // bound, so it reports as Observed; the three-tier vocabulary
            // (exact/observed/predicted) is unchanged.
            let (path, quality) = match q {
                DeltaTQuality::Observed | DeltaTQuality::LeapSecondBound => {
                    (ConversionPath::Ut1DeltaT, ConversionQuality::Observed)
                }
                DeltaTQuality::Predicted => (
                    ConversionPath::FutureExtrapolated,
                    ConversionQuality::Predicted,
                ),
            };
            Ok((
                jd_tt,
                ConversionProvenance {
                    path,
                    quality,
                    delta_t_seconds: Some(dt),
                    tai_minus_utc: None,
                    sources: SOURCES,
                },
            ))
        }
        other => Err(CivilTimeError::UnsupportedScale {
            source: other,
            target,
        }),
    }
}

/// Anchors the last two seconds of a leap-second day at `23:59:59.0`.
///
/// A Julian day cannot express the 86,401st second of a day: `23:59:60.x`
/// aliases the next day's `00:00:00.x`. Near 2.46e6 it also resolves only
/// about 40 µs, so `23:59:59.99998` and later round to the next midnight's
/// Julian day and would pick up the new `TAI − UTC`. For a UTC input at
/// `23:59` with `second` in `[59, 61)` on a day that ends with an inserted
/// leap second, this returns `23:59:59.0` plus `second − 59` seconds, so the
/// caller looks up the offset at an instant that cannot round across
/// midnight and then adds the seconds back. Any other `second` in `[60, 61)`
/// is invalid; every other input is returned unchanged with nothing to add.
fn anchor_leap_second_day(
    civil: CivilDateTime,
    source: TimeScale,
) -> Result<(CivilDateTime, f64), CivilTimeError> {
    if !(59.0..61.0).contains(&civil.second) {
        return Ok((civil, 0.0));
    }
    let in_leap_second = civil.second >= 60.0;
    let invalid = CivilTimeError::InvalidCivilDate { field: "second" };
    if source != TimeScale::Utc || civil.hour != 23 || civil.minute != 59 {
        return if in_leap_second {
            Err(invalid)
        } else {
            Ok((civil, 0.0))
        };
    }
    let midnight = CivilDateTime::new(civil.year, civil.month, civil.day, 0, 0, 0.0);
    let next_midnight_jd = midnight.to_julian_day()?.days() + 1.0;
    if !leap::is_insertion_day_end(next_midnight_jd)? {
        return if in_leap_second {
            Err(invalid)
        } else {
            Ok((civil, 0.0))
        };
    }
    Ok((
        CivilDateTime {
            second: 59.0,
            ..civil
        },
        civil.second - 59.0,
    ))
}

/// Converts a civil datetime tagged `source` (UTC or UT1) to `target` (TT or TDB).
///
/// # Leap seconds
///
/// A UTC `second` in `[60, 61)` is accepted only at `23:59` on a day that
/// ends with an inserted leap second. On such a day, any UTC `second` in
/// `[59, 61)` at `23:59` converts as `23:59:59.0` under the `TAI − UTC` in
/// force before the insertion, plus `second − 59` seconds; the Julian day
/// cannot resolve the last ~20 µs before midnight, so a lookup at the input
/// itself could pick up the new offset. A `second` in `[60, 61)` on any other
/// day or minute, and for UT1 input, is [`CivilTimeError::InvalidCivilDate`]
/// with `field: "second"`.
///
/// # Examples
///
/// ```
/// use pleiades_time::{to_terrestrial, CivilDateTime};
/// use pleiades_types::TimeScale;
///
/// let civil = CivilDateTime::new(2017, 1, 1, 0, 0, 0.0);
/// let out = to_terrestrial(civil, TimeScale::Utc, TimeScale::Tt).unwrap();
/// assert_eq!(out.instant.scale, TimeScale::Tt);
/// assert_eq!(out.provenance.tai_minus_utc, Some(37));
/// ```
pub fn to_terrestrial(
    civil: CivilDateTime,
    source: TimeScale,
    target: TimeScale,
) -> Result<CivilInstant, CivilTimeError> {
    if !matches!(source, TimeScale::Utc | TimeScale::Ut1) {
        return Err(CivilTimeError::UnsupportedScale { source, target });
    }
    if !matches!(target, TimeScale::Tt | TimeScale::Tdb) {
        return Err(CivilTimeError::UnsupportedScale { source, target });
    }
    let (civil, seconds_after) = anchor_leap_second_day(civil, source)?;
    let jd_civil = civil.to_julian_day()?.days();
    let (jd_tt, provenance) = to_tt(jd_civil, source, target)?;
    let jd_tt = jd_tt + seconds_after / SECONDS_PER_DAY;
    let (jd_out, scale) = match target {
        TimeScale::Tt => (jd_tt, TimeScale::Tt),
        TimeScale::Tdb => {
            let jd_tdb = jd_tt + tdb::tdb_minus_tt_seconds(jd_tt) / SECONDS_PER_DAY;
            finite(jd_tdb)?;
            (jd_tdb, TimeScale::Tdb)
        }
        _ => unreachable!("guarded above"),
    };
    Ok(CivilInstant {
        instant: Instant::new(JulianDay::from_days(jd_out), scale),
        provenance,
    })
}

/// Convenience: UTC civil -> TT.
pub fn tt_from_utc_civil(civil: CivilDateTime) -> Result<CivilInstant, CivilTimeError> {
    to_terrestrial(civil, TimeScale::Utc, TimeScale::Tt)
}
/// Convenience: UTC civil -> TDB.
pub fn tdb_from_utc_civil(civil: CivilDateTime) -> Result<CivilInstant, CivilTimeError> {
    to_terrestrial(civil, TimeScale::Utc, TimeScale::Tdb)
}
/// Convenience: UT1 civil -> TT.
pub fn tt_from_ut1_civil(civil: CivilDateTime) -> Result<CivilInstant, CivilTimeError> {
    to_terrestrial(civil, TimeScale::Ut1, TimeScale::Tt)
}
/// Convenience: UT1 civil -> TDB.
pub fn tdb_from_ut1_civil(civil: CivilDateTime) -> Result<CivilInstant, CivilTimeError> {
    to_terrestrial(civil, TimeScale::Ut1, TimeScale::Tdb)
}

/// Returns the UT1 Julian day for a Terrestrial Time Julian day, using the
/// Delta-T table (`UT1 = TT - ΔT`). The Julian-day argument is interpreted in
/// the TT scale; the ΔT lookup uses it directly (the sub-second feedback of ΔT
/// on its own lookup epoch is negligible for sidereal time).
pub fn ut1_jd_from_tt(jd_tt: f64) -> Result<f64, crate::error::CivilTimeError> {
    let (delta_t_seconds, _quality) = crate::deltat::delta_t(jd_tt)?;
    Ok(jd_tt - delta_t_seconds / 86_400.0)
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
