//! The public eclipse search engine.

use crate::ephemeris::{apparent_sun_longitude_deg, sample_sun_moon};
use crate::error::{EclipseError, WINDOW_END_JD, WINDOW_START_JD};
use crate::geometry::{classify_lunar, classify_solar, sub_shadow_point};
use crate::local::{is_locally_visible, local_circumstances_for, LocalCircumstances};
use crate::saros::saros_series;
use crate::syzygy::{find_syzygies, Syzygy, STEP_DAYS};
use crate::types::{Eclipse, EclipseFilter, EclipseKind, EclipseType, Node};
use pleiades_apparent::Atmosphere;
use pleiades_backend::EphemerisBackend;
use pleiades_types::{Instant, JulianDay, Longitude, ObserverLocation, TimeScale};

/// Backstop cap on how many global eclipses `next/previous_local_eclipse`
/// inspect before returning `None` (a locally-visible eclipse always occurs
/// far sooner; this only guards against a pathological non-terminating walk).
const MAX_LOCAL_SEARCH: usize = 4000;

/// Searches for global/geocentric eclipses over a chosen [`EphemerisBackend`].
///
/// Backed by the packaged data, results are valid only within the
/// 1900-01-01..2100-01-01 window (see [`crate`]); out-of-window requests fail
/// closed with [`EclipseError::OutOfWindow`].
pub struct EclipseEngine<B> {
    backend: B,
}

impl<B: EphemerisBackend> EclipseEngine<B> {
    /// Creates an engine that draws Sun/Moon positions from `backend`.
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Returns every eclipse admitted by `filter` with greatest eclipse in
    /// `[start, end]`, in chronological order. Fails closed if either bound is
    /// outside the supported window.
    pub fn eclipses_in_range(
        &self,
        start: Instant,
        end: Instant,
        filter: EclipseFilter,
    ) -> Result<Vec<Eclipse>, EclipseError> {
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;

        // The syzygy scanner (`find_one`) probes one STEP_DAYS past its
        // requested end for sign-change detection, and `sample_sun_moon` issues a
        // light-time-retarded Sun query ~0.006 days before the nominal epoch.
        // Together these mean:
        //   - At the END: the scanner queries up to `scan_end + STEP_DAYS`; the
        //     packaged backend has no data beyond WINDOW_END_JD, so we clamp
        //     `scan_end` to `WINDOW_END_JD - STEP_DAYS`.
        //   - At the START: the retarded query falls `~light_time` before the
        //     first sample; clamping `scan_start` to `WINDOW_START_JD + STEP_DAYS`
        //     (> max light time ~0.006 d) keeps every retarded lookup within coverage.
        // Both clamps are safe: no corpus eclipse falls within 0.5 d of either bound.
        let scan_start = start_jd.max(WINDOW_START_JD + STEP_DAYS);
        let scan_end = end_jd.min(WINDOW_END_JD - STEP_DAYS);

        let mut out = Vec::new();
        for event in find_syzygies(&self.backend, scan_start, scan_end)? {
            if let Some(eclipse) = self.build(event.syzygy, event.julian_day)? {
                if filter.admits(eclipse.kind) {
                    out.push(eclipse);
                }
            }
        }
        Ok(out)
    }

    /// Returns the first eclipse admitted by `filter` whose greatest eclipse is
    /// strictly after `after`, or `None` if none remains before the window end.
    pub fn next_eclipse(
        &self,
        after: Instant,
        filter: EclipseFilter,
    ) -> Result<Option<Eclipse>, EclipseError> {
        let after_jd = after.julian_day.days();
        // `eclipses_in_range` clamps the scan end to WINDOW_END_JD - STEP_DAYS,
        // so passing WINDOW_END_JD directly is safe and correct.
        let end = Instant::new(JulianDay::from_days(WINDOW_END_JD), TimeScale::Tdb);
        Ok(self
            .eclipses_in_range(after, end, filter)?
            .into_iter()
            .find(|e| e.greatest_eclipse.julian_day.days() > after_jd))
    }

    /// Returns the last eclipse admitted by `filter` whose greatest eclipse is
    /// strictly before `before`, or `None` if none exists after the window start.
    pub fn previous_eclipse(
        &self,
        before: Instant,
        filter: EclipseFilter,
    ) -> Result<Option<Eclipse>, EclipseError> {
        let before_jd = before.julian_day.days();
        let start = Instant::new(JulianDay::from_days(WINDOW_START_JD), TimeScale::Tdb);
        Ok(self
            .eclipses_in_range(start, before, filter)?
            .into_iter()
            .rev()
            .find(|e| e.greatest_eclipse.julian_day.days() < before_jd))
    }

    /// Local (per-observer) circumstances for an already-found `eclipse`.
    ///
    /// Returns full circumstances even when the eclipse is not visible from
    /// `observer` (all contacts below the horizon); inspect `any_phase_visible`
    /// (via the returned variant) to test visibility. Solar contact instants are
    /// observer-dependent (topocentric); lunar contact instants are global with
    /// per-observer visibility.
    pub fn local_circumstances(
        &self,
        eclipse: &Eclipse,
        observer: &ObserverLocation,
        atmosphere: Atmosphere,
    ) -> Result<LocalCircumstances, EclipseError> {
        observer
            .validate()
            .map_err(|e| EclipseError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmosphere)?;
        local_circumstances_for(&self.backend, eclipse, observer, atmosphere)
    }

    /// The next eclipse admitted by `filter`, strictly after `after`, that is
    /// locally visible from `observer` (any phase above the horizon), paired with
    /// its local circumstances. Walks the global `next_eclipse` sequence and
    /// returns the first locally-visible one, so the result is a strict refinement
    /// of the global engine.
    pub fn next_local_eclipse(
        &self,
        after: Instant,
        observer: &ObserverLocation,
        filter: EclipseFilter,
        atmosphere: Atmosphere,
    ) -> Result<Option<(Eclipse, LocalCircumstances)>, EclipseError> {
        observer
            .validate()
            .map_err(|e| EclipseError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmosphere)?;
        let mut cursor = after;
        // Bounded walk: no more than MAX_LOCAL_SEARCH global eclipses inspected
        // before giving up (backstop; a locally-visible eclipse always occurs well
        // within the window). ~2 eclipses/year × 200 yr ≈ 1200 global eclipses max.
        for _ in 0..MAX_LOCAL_SEARCH {
            let Some(eclipse) = self.next_eclipse(cursor, filter)? else {
                return Ok(None);
            };
            let local = local_circumstances_for(&self.backend, &eclipse, observer, atmosphere)?;
            if is_locally_visible(&local) {
                return Ok(Some((eclipse, local)));
            }
            cursor = eclipse.greatest_eclipse;
        }
        Ok(None)
    }

    /// The previous eclipse admitted by `filter`, strictly before `before`, that
    /// is locally visible from `observer`, paired with its local circumstances.
    pub fn previous_local_eclipse(
        &self,
        before: Instant,
        observer: &ObserverLocation,
        filter: EclipseFilter,
        atmosphere: Atmosphere,
    ) -> Result<Option<(Eclipse, LocalCircumstances)>, EclipseError> {
        observer
            .validate()
            .map_err(|e| EclipseError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmosphere)?;
        let mut cursor = before;
        for _ in 0..MAX_LOCAL_SEARCH {
            let Some(eclipse) = self.previous_eclipse(cursor, filter)? else {
                return Ok(None);
            };
            let local = local_circumstances_for(&self.backend, &eclipse, observer, atmosphere)?;
            if is_locally_visible(&local) {
                return Ok(Some((eclipse, local)));
            }
            cursor = eclipse.greatest_eclipse;
        }
        Ok(None)
    }

    fn check_window(&self, jd: f64) -> Result<(), EclipseError> {
        if !(WINDOW_START_JD..=WINDOW_END_JD).contains(&jd) {
            Err(EclipseError::OutOfWindow { julian_day: jd })
        } else {
            Ok(())
        }
    }

    fn build(&self, syzygy: Syzygy, syzygy_jd: f64) -> Result<Option<Eclipse>, EclipseError> {
        let greatest_jd = self.refine_greatest(syzygy, syzygy_jd)?;
        let sample = sample_sun_moon(&self.backend, greatest_jd)?;
        let greatest_eclipse = Instant::new(JulianDay::from_days(greatest_jd), TimeScale::Tdb);
        // Compute the apparent geocentric solar longitude of date for eclipsed_longitude.
        // Geometric sampling (separation, classification) uses the Mean frame — that is
        // correct and unchanged. But eclipsed_longitude must be apparent-of-date per the
        // spec (Task 10 gate: ≤1 arcsecond); mean would be ~20–25″ off.
        let apparent_sun_lon = apparent_sun_longitude_deg(&self.backend, greatest_jd)?;
        let eclipsed_longitude = match syzygy {
            Syzygy::NewMoon => Longitude::from_degrees(apparent_sun_lon),
            // For lunar eclipses the eclipsed body is the Moon, which is opposite the Sun;
            // eclipsed_longitude is the apparent solar longitude + 180° (corpus MANIFEST).
            Syzygy::FullMoon => Longitude::from_degrees(apparent_sun_lon + 180.0),
        };
        // Node: ascending (North) if the Moon's latitude is increasing through 0.
        let later = sample_sun_moon(&self.backend, greatest_jd + 0.01)?;
        let near_node = if later.moon_latitude_deg >= sample.moon_latitude_deg {
            Node::North
        } else {
            Node::South
        };

        let eclipse = match syzygy {
            Syzygy::NewMoon => {
                let Some(c) = classify_solar(&sample) else {
                    return Ok(None);
                };
                Eclipse {
                    kind: EclipseKind::Solar,
                    eclipse_type: EclipseType::Solar(c.eclipse_type),
                    greatest_eclipse,
                    magnitude: c.magnitude,
                    gamma: c.gamma,
                    saros_series: saros_series(EclipseKind::Solar, greatest_jd),
                    eclipsed_longitude,
                    near_node,
                    greatest_eclipse_location: Some(sub_shadow_point(&sample, greatest_jd)),
                }
            }
            Syzygy::FullMoon => {
                let Some(c) = classify_lunar(&sample) else {
                    return Ok(None);
                };
                Eclipse {
                    kind: EclipseKind::Lunar,
                    eclipse_type: EclipseType::Lunar(c.eclipse_type),
                    greatest_eclipse,
                    magnitude: c.magnitude,
                    gamma: c.gamma,
                    saros_series: saros_series(EclipseKind::Lunar, greatest_jd),
                    eclipsed_longitude,
                    near_node,
                    greatest_eclipse_location: None,
                }
            }
        };
        Ok(Some(eclipse))
    }

    /// Golden-section minimize the Sun–Moon (or Moon–antisolar) separation in a
    /// ±0.25-day bracket around the syzygy to find greatest eclipse.
    fn refine_greatest(&self, syzygy: Syzygy, syzygy_jd: f64) -> Result<f64, EclipseError> {
        use crate::geometry::separation_for;
        let phi = 0.618_033_988_75_f64;
        let (mut a, mut b) = (syzygy_jd - 0.25, syzygy_jd + 0.25);
        let mut c = b - (b - a) * phi;
        let mut d = a + (b - a) * phi;
        let mut fc = separation_for(syzygy, &sample_sun_moon(&self.backend, c)?);
        let mut fd = separation_for(syzygy, &sample_sun_moon(&self.backend, d)?);
        while (b - a) > 0.5 / 86_400.0 {
            if fc < fd {
                b = d;
                d = c;
                fd = fc;
                c = b - (b - a) * phi;
                fc = separation_for(syzygy, &sample_sun_moon(&self.backend, c)?);
            } else {
                a = c;
                c = d;
                fc = fd;
                d = a + (b - a) * phi;
                fd = separation_for(syzygy, &sample_sun_moon(&self.backend, d)?);
            }
        }
        Ok(0.5 * (a + b))
    }
}

fn check_atmosphere(atmos: Atmosphere) -> Result<(), EclipseError> {
    if !atmos.pressure_mbar.is_finite() || !atmos.temperature_c.is_finite() {
        return Err(EclipseError::InvalidAtmosphere {
            detail: format!(
                "pressure={} temp={}",
                atmos.pressure_mbar, atmos.temperature_c
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, TimeScale};

    fn at(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }

    #[test]
    fn out_of_window_start_fails_closed() {
        let engine = EclipseEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let err = engine
            .eclipses_in_range(at(2_400_000.0), at(2_451_551.0), EclipseFilter::All)
            .unwrap_err();
        assert!(matches!(err, EclipseError::OutOfWindow { .. }));
    }

    #[test]
    fn filter_excludes_lunar() {
        // The on-node analytic backend yields a solar eclipse at every new moon
        // and a lunar one at every full moon; SolarOnly must drop the lunar ones.
        let engine =
            EclipseEngine::new(LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0));
        let solar = engine
            .eclipses_in_range(at(2_451_549.0), at(2_451_551.0), EclipseFilter::SolarOnly)
            .unwrap();
        assert!(solar.iter().all(|e| e.kind == EclipseKind::Solar));
    }

    #[test]
    fn local_circumstances_returns_solar_for_a_solar_eclipse() {
        use pleiades_apparent::Atmosphere;
        use pleiades_types::{Latitude, Longitude, ObserverLocation};
        let engine =
            EclipseEngine::new(LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0));
        let eclipse = engine
            .next_eclipse(at(2_451_549.0), EclipseFilter::SolarOnly)
            .unwrap()
            .expect("a solar eclipse");
        let observer = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let local = engine
            .local_circumstances(&eclipse, &observer, Atmosphere::default())
            .unwrap();
        assert!(matches!(local, crate::LocalCircumstances::Solar(_)));
    }
}
