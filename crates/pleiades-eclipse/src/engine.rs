//! The public eclipse search engine.

use crate::ephemeris::{apparent_sun_longitude_deg, sample_sun_moon};
use crate::error::{EclipseError, WINDOW_END_JD, WINDOW_START_JD};
use crate::geometry::{classify_lunar, classify_solar, sub_shadow_point};
use crate::saros::saros_series;
use crate::syzygy::{find_syzygies, Syzygy};
use crate::types::{Eclipse, EclipseFilter, EclipseKind, EclipseType, Node};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{Instant, JulianDay, Longitude, TimeScale};

pub struct EclipseEngine<B> {
    backend: B,
}

impl<B: EphemerisBackend> EclipseEngine<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

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

        let mut out = Vec::new();
        for event in find_syzygies(&self.backend, start_jd, end_jd)? {
            if let Some(eclipse) = self.build(event.syzygy, event.julian_day)? {
                if filter.admits(eclipse.kind) {
                    out.push(eclipse);
                }
            }
        }
        Ok(out)
    }

    pub fn next_eclipse(
        &self,
        after: Instant,
        filter: EclipseFilter,
    ) -> Result<Option<Eclipse>, EclipseError> {
        let after_jd = after.julian_day.days();
        let end = Instant::new(JulianDay::from_days(WINDOW_END_JD), TimeScale::Tdb);
        Ok(self
            .eclipses_in_range(after, end, filter)?
            .into_iter()
            .find(|e| e.greatest_eclipse.julian_day.days() > after_jd))
    }

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
}
