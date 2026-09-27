//! Time-scale adapters for the observer-local surfaces (rise/set/transit and
//! horizontal coordinates), which mix two clocks: body positions are sampled
//! in TDB, while Earth rotation (sidereal time, hour angle, diurnal parallax)
//! is a function of UT1. The engine works in TDB throughout and converts to
//! UT1 only where rotation is evaluated (issue #74).

use crate::error::EventError;
use pleiades_apparent::sidereal_time;
use pleiades_types::{Instant, JulianDay, Longitude, TimeScale};

const SECONDS_PER_DAY: f64 = 86_400.0;

/// Re-expresses a caller-supplied instant as a TDB Julian Day, honouring its
/// `TimeScale` tag.
///
/// - `Tdb` and `Tt`: taken as-is. TT and TDB differ by under 2 ms, far below
///   the 0.5 s bisection tolerance of every event search here.
/// - `Ut1` and `Utc`: ΔT from `pleiades-time` is added. UTC is treated as UT1
///   (|UT1 − UTC| < 0.9 s; the workspace carries no DUT1 table), mirroring
///   `pleiades_apparent::ut1_instant`.
///
/// # Errors
///
/// `EventError::UnsupportedTimeScale` for a scale this adapter has no
/// conversion for (`TimeScale` is `#[non_exhaustive]`), and
/// `EventError::Backend` if the packaged ΔT table cannot be read.
pub(crate) fn tdb_jd(instant: Instant) -> Result<f64, EventError> {
    let jd = instant.julian_day.days();
    match instant.scale {
        TimeScale::Tdb | TimeScale::Tt => Ok(jd),
        TimeScale::Ut1 | TimeScale::Utc => Ok(jd + delta_t_days(jd)?),
        other => Err(EventError::UnsupportedTimeScale { scale: other }),
    }
}

/// The UT1 Julian Day for a TDB Julian Day (`UT1 = TDB − ΔT`, TDB taken as
/// TT).
pub(crate) fn ut1_jd(jd_tdb: f64) -> Result<f64, EventError> {
    Ok(jd_tdb - delta_t_days(jd_tdb)?)
}

/// Local apparent sidereal time (degrees, `[0, 360)`) at a TDB Julian Day for
/// an observer east longitude, with Earth rotation evaluated at the UT1
/// re-expression of that day.
pub(crate) fn local_apparent_sidereal_deg(
    jd_tdb: f64,
    observer_longitude: Longitude,
) -> Result<f64, EventError> {
    let at_ut1 = Instant::new(JulianDay::from_days(ut1_jd(jd_tdb)?), TimeScale::Ut1);
    Ok(sidereal_time(at_ut1, observer_longitude).local_apparent_deg)
}

/// ΔT in days at `jd`. The lookup epoch is read in whichever scale `jd`
/// carries; ΔT's sub-second sensitivity to that choice is negligible here.
fn delta_t_days(jd: f64) -> Result<f64, EventError> {
    let (delta_t_seconds, _quality) = pleiades_time::deltat::delta_t(jd)
        .map_err(|e| EventError::Backend(format!("delta_t failed: {e}")))?;
    Ok(delta_t_seconds / SECONDS_PER_DAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    const JD: f64 = 2_451_545.0;

    #[test]
    fn tt_and_tdb_tags_pass_the_day_through() {
        for scale in [TimeScale::Tt, TimeScale::Tdb] {
            let jd = tdb_jd(Instant::new(JulianDay::from_days(JD), scale)).unwrap();
            assert_eq!(jd, JD, "{scale:?}");
        }
    }

    #[test]
    fn ut1_and_utc_tags_add_delta_t() {
        let expected = JD + pleiades_time::deltat::delta_t(JD).unwrap().0 / SECONDS_PER_DAY;
        for scale in [TimeScale::Ut1, TimeScale::Utc] {
            let jd = tdb_jd(Instant::new(JulianDay::from_days(JD), scale)).unwrap();
            assert!(
                (jd - expected).abs() < 1e-12,
                "{scale:?}: {jd} vs {expected}"
            );
        }
    }

    #[test]
    fn ut1_round_trips_a_ut1_tagged_day_to_within_delta_t_feedback() {
        // UT1 → TDB (+ΔT at the UT1 epoch) → UT1 (−ΔT at the TDB epoch): the
        // two lookups differ only by ΔT's drift over ~64 s, well under 1 ms.
        let tdb = tdb_jd(Instant::new(JulianDay::from_days(JD), TimeScale::Ut1)).unwrap();
        let back = ut1_jd(tdb).unwrap();
        assert!((back - JD).abs() * SECONDS_PER_DAY < 1e-3);
    }

    #[test]
    fn sidereal_time_uses_the_ut1_day() {
        let via_helper = local_apparent_sidereal_deg(JD, Longitude::from_degrees(0.0)).unwrap();
        let ut1 = Instant::new(JulianDay::from_days(ut1_jd(JD).unwrap()), TimeScale::Ut1);
        let direct = sidereal_time(ut1, Longitude::from_degrees(0.0)).local_apparent_deg;
        assert_eq!(via_helper, direct);
        // And it differs from the raw-day evaluation by ΔT of rotation.
        let raw = sidereal_time(
            Instant::new(JulianDay::from_days(JD), TimeScale::Tdb),
            Longitude::from_degrees(0.0),
        )
        .local_apparent_deg;
        assert!(
            (raw - via_helper).abs() > 0.2,
            "raw {raw} vs ut1 {via_helper}"
        );
    }
}
