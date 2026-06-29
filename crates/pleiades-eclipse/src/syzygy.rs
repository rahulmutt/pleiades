//! Locates new and full moons (eclipse candidates) by root-finding on the
//! Sun−Moon elongation.

// Items here are pub(crate) for upcoming eclipse-engine tasks; silence
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::ephemeris::{elongation_deg, sample_sun_moon};
use crate::error::EclipseError;
use pleiades_backend::EphemerisBackend;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Syzygy {
    NewMoon,
    FullMoon,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SyzygyEvent {
    pub syzygy: Syzygy,
    pub julian_day: f64,
}

const STEP_DAYS: f64 = 0.5;
const REFINE_TOLERANCE_DAYS: f64 = 0.5 / 86_400.0; // 0.5 s

/// Target function whose zero marks the syzygy: elongation for a new moon, and
/// the signed distance from 180° for a full moon. Both wrap into (-180, 180].
fn target<B: EphemerisBackend>(
    backend: &B,
    julian_day: f64,
    syzygy: Syzygy,
) -> Result<f64, EclipseError> {
    let sample = sample_sun_moon(backend, julian_day)?;
    let elong = elongation_deg(&sample);
    Ok(match syzygy {
        Syzygy::NewMoon => elong,
        Syzygy::FullMoon => {
            let mut d = elong - 180.0;
            d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
            d
        }
    })
}

fn bisect<B: EphemerisBackend>(
    backend: &B,
    syzygy: Syzygy,
    mut lo: f64,
    mut f_lo: f64,
    mut hi: f64,
) -> Result<f64, EclipseError> {
    while (hi - lo) > REFINE_TOLERANCE_DAYS {
        let mid = 0.5 * (lo + hi);
        let f_mid = target(backend, mid, syzygy)?;
        if (f_lo <= 0.0) == (f_mid <= 0.0) {
            lo = mid;
            f_lo = f_mid;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

fn find_one<B: EphemerisBackend>(
    backend: &B,
    syzygy: Syzygy,
    start_jd: f64,
    end_jd: f64,
    out: &mut Vec<SyzygyEvent>,
) -> Result<(), EclipseError> {
    let mut prev_jd = start_jd;
    let mut prev_f = target(backend, prev_jd, syzygy)?;
    let mut jd = start_jd + STEP_DAYS;
    while jd <= end_jd + STEP_DAYS {
        let f = target(backend, jd, syzygy)?;
        // A real syzygy crossing: sign change with the gap small enough that it
        // is the elongation passing through zero (not the ±180 wrap seam).
        if (prev_f <= 0.0) != (f <= 0.0) && (prev_f - f).abs() < 180.0 {
            let root = bisect(backend, syzygy, prev_jd, prev_f, jd)?;
            if root >= start_jd && root <= end_jd {
                out.push(SyzygyEvent {
                    syzygy,
                    julian_day: root,
                });
            }
        }
        prev_jd = jd;
        prev_f = f;
        jd += STEP_DAYS;
    }
    Ok(())
}

pub(crate) fn find_syzygies<B: EphemerisBackend>(
    backend: &B,
    start_jd: f64,
    end_jd: f64,
) -> Result<Vec<SyzygyEvent>, EclipseError> {
    let mut out = Vec::new();
    find_one(backend, Syzygy::NewMoon, start_jd, end_jd, &mut out)?;
    find_one(backend, Syzygy::FullMoon, start_jd, end_jd, &mut out)?;
    out.sort_by(|a, b| a.julian_day.partial_cmp(&b.julian_day).unwrap());
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn finds_the_new_moon_at_the_reference_epoch() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let events = find_syzygies(&backend, 2_451_549.0, 2_451_551.0).unwrap();
        let new_moon = events.iter().find(|e| e.syzygy == Syzygy::NewMoon).unwrap();
        assert!((new_moon.julian_day - 2_451_550.0).abs() < 1.0 / 86_400.0);
    }

    #[test]
    fn events_are_time_ordered() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let events = find_syzygies(&backend, 2_451_540.0, 2_451_600.0).unwrap();
        assert!(events
            .windows(2)
            .all(|w| w[0].julian_day <= w[1].julian_day));
        assert!(events.len() >= 4); // ~2 lunations → ≥2 new + ≥2 full
    }
}
