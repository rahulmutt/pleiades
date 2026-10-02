//! Inverse orchestrator: a TT/TDB `Instant` -> civil UTC/UT1 datetime with provenance.
//!
//! The instant is quantized once to a whole number of milliseconds. Leap
//! thresholds are exact integers on that axis, so the leap-second lookup is
//! integer comparison and a millisecond-aligned civil datetime survives the
//! round trip through `to_terrestrial` with identical fields.

use pleiades_types::{Instant, JulianDay, TimeScale, SECONDS_PER_DAY};

use super::{
    ConversionPath, ConversionProvenance, ConversionQuality, SOURCES, SUPPORT_END_JD,
    SUPPORT_START_JD,
};
use crate::calendar::CivilDateTime;
use crate::deltat::{self, DeltaTQuality};
use crate::error::CivilTimeError;
use crate::{leap, tdb};

const MS_PER_DAY: i64 = 86_400_000;
/// TT − TAI = 32.184 s, in milliseconds.
const TT_MINUS_TAI_MS: i64 = 32_184;

/// A civil datetime recovered from a TT/TDB instant, plus how it was produced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CivilConversion {
    /// The civil datetime in `scale`, rounded to the millisecond. `second` is
    /// in `[60, 61)` only for a UTC result inside an inserted leap second.
    pub civil: CivilDateTime,
    /// The civil scale of `civil`: `Utc` or `Ut1`.
    pub scale: TimeScale,
    /// How the datetime was produced, including its truthful quality tier.
    pub provenance: ConversionProvenance,
}

/// Milliseconds on a uniform axis whose days begin at midnight: the day
/// number `floor(JD + 0.5)` times [`MS_PER_DAY`], plus the rounded
/// millisecond of day. The caller bounds `jd` to the support window first,
/// so the product cannot overflow.
fn ms_from_jd(jd: f64) -> i64 {
    let shifted = jd + 0.5;
    let day = shifted.floor();
    let ms_of_day = ((shifted - day) * MS_PER_DAY as f64).round();
    day as i64 * MS_PER_DAY + ms_of_day as i64
}

/// The Julian day of a millisecond count from [`ms_from_jd`].
fn jd_from_ms(ms: i64) -> f64 {
    let day = ms.div_euclid(MS_PER_DAY);
    let ms_of_day = ms.rem_euclid(MS_PER_DAY);
    day as f64 - 0.5 + ms_of_day as f64 / MS_PER_DAY as f64
}

/// The civil datetime of a millisecond count. The date comes from the
/// calendar inverse at an exact midnight; the time of day is split from the
/// integer millisecond, so no field is rounded separately.
fn civil_from_ms(ms: i64) -> CivilDateTime {
    let day = ms.div_euclid(MS_PER_DAY);
    let ms_of_day = ms.rem_euclid(MS_PER_DAY);
    let date = CivilDateTime::from_julian_day(JulianDay::from_days(day as f64 - 0.5));
    let hour = ms_of_day / 3_600_000;
    let minute = (ms_of_day % 3_600_000) / 60_000;
    let second = (ms_of_day % 60_000) as f64 / 1_000.0;
    CivilDateTime::new(
        date.year,
        date.month,
        date.day,
        hour as u8,
        minute as u8,
        second,
    )
}

fn exact_provenance(tai_minus_utc: i32) -> ConversionProvenance {
    ConversionProvenance {
        path: ConversionPath::UtcLeapSecond,
        quality: ConversionQuality::Exact,
        delta_t_seconds: None,
        tai_minus_utc: Some(tai_minus_utc),
        sources: SOURCES,
    }
}

/// A civil result before the support-window check: the civil millisecond
/// count used for that check, the datetime, and its provenance.
type Solved = (i64, CivilDateTime, ConversionProvenance);

/// UTC from TT. Inside the leap table the lookup runs on the TAI axis, where
/// row `i` takes effect at `effective_i + secs_i` and the second before each
/// later threshold is the inserted leap second.
fn utc_from_tt(jd_tt: f64) -> Result<Solved, CivilTimeError> {
    let stale = CivilTimeError::StaleTimeData {
        kind: "leap-second",
    };
    let rows = leap::table()?;
    let threshold =
        |&(effective, secs): &(f64, i32)| ms_from_jd(effective) + i64::from(secs) * 1_000;
    let (first, last) = match (rows.first(), rows.last()) {
        (Some(first), Some(last)) => (first, last),
        _ => return Err(stale),
    };
    let tai_ms = ms_from_jd(jd_tt) - TT_MINUS_TAI_MS;
    if tai_ms < threshold(first) {
        return Err(if jd_tt < SUPPORT_START_JD {
            CivilTimeError::BeyondHorizon { jd: jd_tt }
        } else {
            CivilTimeError::UtcBeforeLeapEpoch
        });
    }
    let horizon_ms = ms_from_jd(leap::VALID_THROUGH_JD) + i64::from(last.1) * 1_000;
    if tai_ms > horizon_ms {
        return civil_via_delta_t(jd_tt, TimeScale::Utc);
    }
    let index = rows
        .iter()
        .rposition(|row| tai_ms >= threshold(row))
        .ok_or(stale)?;
    let (_, secs) = rows[index];
    if let Some(next) = rows.get(index + 1) {
        let leap_start_ms = threshold(next) - 1_000;
        if tai_ms >= leap_start_ms {
            let last_second_ms = ms_from_jd(next.0) - 1_000;
            let mut civil = civil_from_ms(last_second_ms);
            civil.second = 60.0 + (tai_ms - leap_start_ms) as f64 / 1_000.0;
            return Ok((last_second_ms, civil, exact_provenance(secs)));
        }
    }
    let utc_ms = tai_ms - i64::from(secs) * 1_000;
    Ok((utc_ms, civil_from_ms(utc_ms), exact_provenance(secs)))
}

/// Solves `civil + ΔT(civil) = TT` for the civil Julian day, the equation the
/// forward conversion evaluates. ΔT changes by under 5e-8 s per second, so
/// each step shrinks the error by that factor and three steps are ample.
/// At the 0.216 s step in ΔT at the 2020 node the start value is already past
/// the node, so the solve settles on the post-node branch.
fn civil_via_delta_t(jd_tt: f64, target: TimeScale) -> Result<Solved, CivilTimeError> {
    let (mut delta_t, mut delta_t_quality) = deltat::delta_t(jd_tt)?;
    for _ in 0..3 {
        (delta_t, delta_t_quality) = deltat::delta_t(jd_tt - delta_t / SECONDS_PER_DAY)?;
    }
    let civil_ms = ms_from_jd(jd_tt - delta_t / SECONDS_PER_DAY);
    // Mirrors the forward: UTC reaches this path only past the leap horizon
    // and is always Predicted; UT1 reports the ΔT tier it used.
    let (path, quality) = match (target, delta_t_quality) {
        (TimeScale::Ut1, DeltaTQuality::Observed | DeltaTQuality::LeapSecondBound) => {
            (ConversionPath::Ut1DeltaT, ConversionQuality::Observed)
        }
        _ => (
            ConversionPath::FutureExtrapolated,
            ConversionQuality::Predicted,
        ),
    };
    Ok((
        civil_ms,
        civil_from_ms(civil_ms),
        ConversionProvenance {
            path,
            quality,
            delta_t_seconds: Some(delta_t),
            tai_minus_utc: None,
            sources: SOURCES,
        },
    ))
}

/// Converts a TT or TDB instant to a civil datetime in `target` (UTC or UT1):
/// the inverse of [`to_terrestrial`](super::to_terrestrial).
///
/// The source scale is read from `instant.scale`. The result is rounded to
/// the millisecond, and converting it back with `to_terrestrial` returns the
/// starting instant to within 1 ms. In the other direction, a civil datetime
/// passed through `to_terrestrial` and back returns to within 1 ms, except
/// UT1 in the 0.216 s before the 2020-01-01 node, which comes back as the
/// post-node datetime (see below), and the last 0.5 ms of 2100, which rounds
/// out of range (see Errors). The provenance uses the same vocabulary and the
/// same epoch tiers as the forward conversion: UTC from 1972 through the
/// leap-second table is `Exact`; UTC beyond the table and UT1 use the Delta-T
/// model and are `Observed` or `Predicted`.
///
/// # Leap seconds
///
/// A UTC result inside an inserted leap second has `second` in `[60, 61)` on
/// the day the second was appended to, with the `TAI − UTC` in force before
/// the insertion. The result is a [`CivilDateTime`] rather than a UTC-tagged
/// `Instant` because a Julian day cannot represent that second.
///
/// # UT1 near 2020-01-01
///
/// Delta-T steps down by 0.216 s at the last observed node (2020-01-01), so
/// TT instants in a 0.216 s window there correspond to two UT1 datetimes.
/// This function returns the one at or after the node.
///
/// # Errors
///
/// - [`CivilTimeError::UnsupportedScale`] unless the instant is TT or TDB and
///   `target` is UTC or UT1.
/// - [`CivilTimeError::NonFiniteOffset`] for a non-finite Julian day.
/// - [`CivilTimeError::BeyondHorizon`] when the civil datetime falls outside
///   the 1900–2100 support window. The check applies after rounding to the
///   millisecond, so an instant within the last 0.5 ms of the window rounds
///   to 2101-01-01T00:00:00.000 and is out of range. The error carries the
///   civil Julian day, except for an instant more than a day outside the
///   window, which is rejected before conversion and carries the TT Julian
///   day.
/// - [`CivilTimeError::UtcBeforeLeapEpoch`] for a UTC target before
///   1972-01-01; use UT1 there.
/// - [`CivilTimeError::StaleTimeData`] if a pinned table fails its checksum.
///
/// # Examples
///
/// ```
/// use pleiades_time::{from_terrestrial, tt_from_utc_civil, CivilDateTime};
/// use pleiades_types::TimeScale;
///
/// // Half a second into the leap second inserted at the end of 2016.
/// let leap = CivilDateTime::new(2016, 12, 31, 23, 59, 60.5);
/// let tt = tt_from_utc_civil(leap).unwrap();
/// let back = from_terrestrial(tt.instant, TimeScale::Utc).unwrap();
/// assert_eq!(back.civil, leap);
/// assert_eq!(back.scale, TimeScale::Utc);
/// assert_eq!(back.provenance.tai_minus_utc, Some(36));
/// ```
pub fn from_terrestrial(
    instant: Instant,
    target: TimeScale,
) -> Result<CivilConversion, CivilTimeError> {
    let source = instant.scale;
    if !matches!(source, TimeScale::Tt | TimeScale::Tdb)
        || !matches!(target, TimeScale::Utc | TimeScale::Ut1)
    {
        return Err(CivilTimeError::UnsupportedScale { source, target });
    }
    let jd = instant.julian_day.days();
    if !jd.is_finite() {
        return Err(CivilTimeError::NonFiniteOffset);
    }
    // The forward evaluates the periodic term at the TT day; evaluating it at
    // the TDB day differs by under 1e-12 s.
    let jd_tt = if source == TimeScale::Tdb {
        jd - tdb::tdb_minus_tt_seconds(jd) / SECONDS_PER_DAY
    } else {
        jd
    };
    // TT and civil time differ by minutes at most, so anything more than a
    // day outside the window is out of range. Rejecting it here also keeps
    // the millisecond count inside i64.
    if !(SUPPORT_START_JD - 1.0..SUPPORT_END_JD + 1.0).contains(&jd_tt) {
        return Err(CivilTimeError::BeyondHorizon { jd: jd_tt });
    }
    let (civil_ms, civil, provenance) = if target == TimeScale::Utc {
        utc_from_tt(jd_tt)?
    } else {
        civil_via_delta_t(jd_tt, target)?
    };
    if !(ms_from_jd(SUPPORT_START_JD)..ms_from_jd(SUPPORT_END_JD)).contains(&civil_ms) {
        return Err(CivilTimeError::BeyondHorizon {
            jd: jd_from_ms(civil_ms),
        });
    }
    Ok(CivilConversion {
        civil,
        scale: target,
        provenance,
    })
}

fn from_scale(
    instant: Instant,
    source: TimeScale,
    target: TimeScale,
) -> Result<CivilConversion, CivilTimeError> {
    if instant.scale != source {
        return Err(CivilTimeError::UnsupportedScale {
            source: instant.scale,
            target,
        });
    }
    from_terrestrial(instant, target)
}

/// Convenience: TT instant -> UTC civil. Rejects an instant not tagged TT.
pub fn utc_civil_from_tt(instant: Instant) -> Result<CivilConversion, CivilTimeError> {
    from_scale(instant, TimeScale::Tt, TimeScale::Utc)
}
/// Convenience: TDB instant -> UTC civil. Rejects an instant not tagged TDB.
pub fn utc_civil_from_tdb(instant: Instant) -> Result<CivilConversion, CivilTimeError> {
    from_scale(instant, TimeScale::Tdb, TimeScale::Utc)
}
/// Convenience: TT instant -> UT1 civil. Rejects an instant not tagged TT.
pub fn ut1_civil_from_tt(instant: Instant) -> Result<CivilConversion, CivilTimeError> {
    from_scale(instant, TimeScale::Tt, TimeScale::Ut1)
}
/// Convenience: TDB instant -> UT1 civil. Rejects an instant not tagged TDB.
pub fn ut1_civil_from_tdb(instant: Instant) -> Result<CivilConversion, CivilTimeError> {
    from_scale(instant, TimeScale::Tdb, TimeScale::Ut1)
}

#[cfg(test)]
mod tests;
