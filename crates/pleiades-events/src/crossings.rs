//! The public longitude-crossing engine.

use crate::ephemeris::{geocentric_apparent_longitude_deg, heliocentric_longitude_deg};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::root::{crossings_in_range, wrap180};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

/// The coordinate/center convention a crossing is computed in.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum CrossingFrame {
    /// Geocentric apparent tropical ecliptic of date (SE `solcross`/`mooncross`).
    GeocentricApparentOfDate,
    /// Heliocentric ecliptic (SE `helio_cross`); planets only.
    Heliocentric,
}

/// A single longitude crossing.
//
// `CelestialBody` is not `Copy` (it carries `Custom(CustomBodyId)`), so `Crossing`
// cannot derive `Copy`; it is `Clone` only.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Crossing {
    /// The body that crossed the target longitude.
    pub body: CelestialBody,
    /// The ecliptic longitude that was crossed.
    pub target_longitude: Longitude,
    /// Instant of the crossing (TDB).
    pub instant: Instant,
    /// The frame the crossing was computed in.
    pub frame: CrossingFrame,
}

/// Finds longitude crossings of a body over the packaged 1900–2100 TDB window.
pub struct CrossingEngine<B> {
    pub(crate) backend: B,
}

impl<B: EphemerisBackend> CrossingEngine<B> {
    /// Wraps a backend.
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Step used to bracket crossings, scaled by body speed so no crossing is
    /// skipped (fast Moon → small step; slow outer planets → larger step).
    fn step_days(body: &CelestialBody) -> f64 {
        match body {
            CelestialBody::Moon => 0.25,
            CelestialBody::Sun | CelestialBody::Mercury | CelestialBody::Venus => 1.0,
            _ => 2.0,
        }
    }

    fn check_window(&self, jd: f64) -> Result<(), EventError> {
        if !(WINDOW_START_JD..=WINDOW_END_JD).contains(&jd) {
            return Err(EventError::OutOfWindow { julian_day: jd });
        }
        Ok(())
    }

    fn longitude_deg(
        &self,
        body: &CelestialBody,
        frame: CrossingFrame,
        jd: f64,
    ) -> Result<f64, EventError> {
        match frame {
            CrossingFrame::GeocentricApparentOfDate => {
                geocentric_apparent_longitude_deg(&self.backend, body.clone(), body_label(body), jd)
            }
            CrossingFrame::Heliocentric => {
                heliocentric_longitude_deg(&self.backend, body.clone(), body_label(body), jd)
            }
        }
    }

    /// All crossings of `target` by `body` in `[start, end]` (TDB), ascending.
    pub fn longitude_crossings_in_range(
        &self,
        body: CelestialBody,
        target: Longitude,
        frame: CrossingFrame,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<Crossing>, EventError> {
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;
        let step = Self::step_days(&body);
        // Clamp like the eclipse engine: keep retarded/aberration queries in-window.
        let scan_start = start_jd.max(WINDOW_START_JD + step);
        let scan_end = end_jd.min(WINDOW_END_JD - step);
        let target_deg = target.degrees();
        let roots = crossings_in_range(
            |jd| Ok(wrap180(self.longitude_deg(&body, frame, jd)? - target_deg)),
            scan_start,
            scan_end,
            step,
        )?;
        Ok(roots
            .into_iter()
            .map(|jd| Crossing {
                body: body.clone(),
                target_longitude: target,
                instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
                frame,
            })
            .collect())
    }

    /// The first crossing strictly after `after`, or `None`.
    pub fn next_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        frame: CrossingFrame,
        after: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let end = Instant::new(JulianDay::from_days(WINDOW_END_JD), TimeScale::Tdb);
        let after_jd = after.julian_day.days();
        Ok(self
            .longitude_crossings_in_range(body, target, frame, after, end)?
            .into_iter()
            .find(|c| c.instant.julian_day.days() > after_jd))
    }

    /// The last crossing strictly before `before`, or `None`.
    pub fn previous_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        frame: CrossingFrame,
        before: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let start = Instant::new(JulianDay::from_days(WINDOW_START_JD), TimeScale::Tdb);
        let before_jd = before.julian_day.days();
        Ok(self
            .longitude_crossings_in_range(body, target, frame, start, before)?
            .into_iter()
            .rev()
            .find(|c| c.instant.julian_day.days() < before_jd))
    }

    /// `swe_solcross`: next geocentric apparent Sun crossing of `target`.
    pub fn next_sun_crossing(
        &self,
        target: Longitude,
        after: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        self.next_longitude_crossing(
            CelestialBody::Sun,
            target,
            CrossingFrame::GeocentricApparentOfDate,
            after,
        )
    }

    /// `swe_mooncross`: next geocentric apparent Moon crossing of `target`.
    pub fn next_moon_crossing(
        &self,
        target: Longitude,
        after: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        self.next_longitude_crossing(
            CelestialBody::Moon,
            target,
            CrossingFrame::GeocentricApparentOfDate,
            after,
        )
    }
}

fn body_label(body: &CelestialBody) -> &'static str {
    match body {
        CelestialBody::Sun => "Sun",
        CelestialBody::Moon => "Moon",
        CelestialBody::Mercury => "Mercury",
        CelestialBody::Venus => "Venus",
        CelestialBody::Mars => "Mars",
        CelestialBody::Jupiter => "Jupiter",
        CelestialBody::Saturn => "Saturn",
        CelestialBody::Uranus => "Uranus",
        CelestialBody::Neptune => "Neptune",
        CelestialBody::Pluto => "Pluto",
        _ => "body",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    fn tdb(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }

    #[test]
    fn finds_a_sun_crossing_in_range() {
        let engine = CrossingEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        // Sun sweeps ~1°/day; over 400 days it crosses any target at least once.
        let start = tdb(2_451_545.0);
        let end = tdb(2_451_545.0 + 400.0);
        let out = engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                Longitude::from_degrees(100.0),
                CrossingFrame::GeocentricApparentOfDate,
                start,
                end,
            )
            .unwrap();
        assert!(!out.is_empty(), "expected at least one Sun crossing");
        for c in &out {
            let lon = geocentric_apparent_longitude_deg(
                &engine.backend,
                CelestialBody::Sun,
                "Sun",
                c.instant.julian_day.days(),
            )
            .unwrap();
            assert!(
                wrap180(lon - 100.0).abs() < 1e-3,
                "residual at crossing {lon}"
            );
        }
    }

    #[test]
    fn next_equals_first_in_range() {
        let engine = CrossingEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = tdb(2_451_545.0);
        let end = tdb(2_451_545.0 + 400.0);
        let next = engine
            .next_longitude_crossing(
                CelestialBody::Sun,
                Longitude::from_degrees(100.0),
                CrossingFrame::GeocentricApparentOfDate,
                after,
            )
            .unwrap()
            .unwrap();
        // `Crossing` is not `Copy`, so bind a reference into the Vec rather than
        // moving element `[0]` out of it.
        let in_range = engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                Longitude::from_degrees(100.0),
                CrossingFrame::GeocentricApparentOfDate,
                after,
                end,
            )
            .unwrap();
        let first = &in_range[0];
        assert!((next.instant.julian_day.days() - first.instant.julian_day.days()).abs() < 1e-6);
    }

    #[test]
    fn out_of_window_fails_closed() {
        let engine = CrossingEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let err = engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                Longitude::from_degrees(0.0),
                CrossingFrame::GeocentricApparentOfDate,
                tdb(2_000_000.0),
                tdb(2_100_000.0),
            )
            .unwrap_err();
        assert!(matches!(err, EventError::OutOfWindow { .. }));
    }
}
