//! Orchestrator: civil datetime + scales -> tagged TT/TDB Instant with provenance.

use core::fmt;

use pleiades_types::{Instant, JulianDay, TimeScale, SECONDS_PER_DAY};

use crate::calendar::CivilDateTime;
use crate::deltat::{self, DeltaTQuality};
use crate::error::CivilTimeError;
use crate::leap;
use crate::tdb;

/// Start of the supported civil window (1900-01-01 00:00).
pub const SUPPORT_START_JD: f64 = 2415020.5;
/// End of the supported civil window: an exclusive upper bound at the start of
/// 2101 (JD 2488434.5). The last accepted instant is 2100-12-31T23:59:59.x;
/// 2101-01-01T00:00:00 is rejected as `BeyondHorizon`.
pub const SUPPORT_END_JD: f64 = 2488434.5;

/// TT − TAI, in seconds (fixed by definition).
const TT_MINUS_TAI: f64 = 32.184;

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
    pub path: ConversionPath,
    pub quality: ConversionQuality,
    pub delta_t_seconds: Option<f64>,
    pub tai_minus_utc: Option<i32>,
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
    pub instant: Instant,
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
            let (path, quality) = match q {
                DeltaTQuality::Observed => (ConversionPath::Ut1DeltaT, ConversionQuality::Observed),
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

/// Converts a civil datetime tagged `source` (UTC or UT1) to `target` (TT or TDB).
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
    let jd_civil = civil.to_julian_day()?.days();
    let (jd_tt, provenance) = to_tt(jd_civil, source, target)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_modern_is_exact() {
        // 2017-01-01 00:00 UTC -> TT. Offset = 37 + 32.184 = 69.184 s.
        let civil = CivilDateTime::new(2017, 1, 1, 0, 0, 0.0);
        let out = tt_from_utc_civil(civil).unwrap();
        assert_eq!(out.instant.scale, TimeScale::Tt);
        assert_eq!(out.provenance.quality, ConversionQuality::Exact);
        assert_eq!(out.provenance.tai_minus_utc, Some(37));
        let expected_jd = civil.to_julian_day().unwrap().days() + 69.184 / SECONDS_PER_DAY;
        assert!((out.instant.julian_day.days() - expected_jd).abs() < 1e-9);
    }

    #[test]
    fn ut1_historical_is_observed() {
        let civil = CivilDateTime::new(1950, 1, 1, 0, 0, 0.0);
        let out = tt_from_ut1_civil(civil).unwrap();
        assert_eq!(out.provenance.quality, ConversionQuality::Observed);
        assert!(out.provenance.delta_t_seconds.unwrap() > 28.0);
    }

    #[test]
    fn future_utc_is_predicted() {
        let civil = CivilDateTime::new(2090, 6, 1, 0, 0, 0.0);
        let out = tt_from_utc_civil(civil).unwrap();
        assert_eq!(out.provenance.quality, ConversionQuality::Predicted);
        assert_eq!(out.provenance.path, ConversionPath::FutureExtrapolated);
    }

    #[test]
    fn pre_1972_utc_is_rejected() {
        let civil = CivilDateTime::new(1965, 1, 1, 0, 0, 0.0);
        assert_eq!(
            tt_from_utc_civil(civil),
            Err(CivilTimeError::UtcBeforeLeapEpoch)
        );
    }

    #[test]
    fn outside_window_is_rejected() {
        let civil = CivilDateTime::new(1880, 1, 1, 0, 0, 0.0);
        assert!(matches!(
            tt_from_ut1_civil(civil),
            Err(CivilTimeError::BeyondHorizon { .. })
        ));
    }

    #[test]
    fn bad_target_scale_is_rejected() {
        let civil = CivilDateTime::new(2000, 1, 1, 0, 0, 0.0);
        assert_eq!(
            to_terrestrial(civil, TimeScale::Utc, TimeScale::Ut1),
            Err(CivilTimeError::UnsupportedScale {
                source: TimeScale::Utc,
                target: TimeScale::Ut1
            })
        );
    }

    #[test]
    fn end_of_2100_is_accepted() {
        let civil = CivilDateTime::new(2100, 12, 31, 23, 59, 59.0);
        assert!(tt_from_ut1_civil(civil).is_ok());
    }

    #[test]
    fn start_of_2101_ut1_is_rejected() {
        let civil = CivilDateTime::new(2101, 1, 1, 0, 0, 0.0);
        assert!(matches!(
            tt_from_ut1_civil(civil),
            Err(CivilTimeError::BeyondHorizon { .. })
        ));
    }

    #[test]
    fn start_of_2101_utc_is_rejected() {
        let civil = CivilDateTime::new(2101, 1, 1, 0, 0, 0.0);
        assert!(matches!(
            tt_from_utc_civil(civil),
            Err(CivilTimeError::BeyondHorizon { .. })
        ));
    }

    #[test]
    fn tdb_differs_from_tt_sub_millisecond() {
        let civil = CivilDateTime::new(2000, 4, 1, 0, 0, 0.0);
        let tt = tt_from_utc_civil(civil).unwrap().instant.julian_day.days();
        let tdb = tdb_from_utc_civil(civil).unwrap().instant.julian_day.days();
        let diff_s = (tdb - tt).abs() * SECONDS_PER_DAY;
        assert!(diff_s < 0.002 && diff_s > 0.0, "diff {diff_s}s");
    }
}
