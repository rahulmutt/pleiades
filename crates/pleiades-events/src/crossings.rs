//! The public longitude-crossing engine.

use crate::ephemeris::{geocentric_apparent_longitude_deg, heliocentric_longitude_deg};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::root::{crossings_in_range, first_crossing_after, last_crossing_before, wrap180};
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

/// Finds ephemeris events (longitude crossings today; rise/set/transit and
/// horizontal coordinates in sibling modules) over the packaged 1900–2100 TDB
/// window.
pub struct EventEngine<B> {
    pub(crate) backend: B,
}

impl<B: EphemerisBackend> EventEngine<B> {
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
        if matches!(frame, CrossingFrame::Heliocentric)
            && matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        {
            return Err(EventError::UnsupportedFrame {
                detail: format!("heliocentric crossings are undefined for {:?}", body),
            });
        }
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
    ///
    /// Early-terminating: this brackets and bisects forward from `after` and
    /// returns as soon as the first root is found, instead of scanning to
    /// `WINDOW_END`. The result is identical to
    /// `longitude_crossings_in_range(body, target, frame, after, WINDOW_END).first()`
    /// filtered to strictly-after `after` — same clamps, same step, same
    /// wrap-seam guard, same bisection tolerance.
    pub fn next_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        frame: CrossingFrame,
        after: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let after_jd = after.julian_day.days();
        // Preserve the window-check and heliocentric Sun/Moon guard exactly as
        // `longitude_crossings_in_range` applies them, before scanning.
        self.check_window(after_jd)?;
        self.check_window(WINDOW_END_JD)?;
        if matches!(frame, CrossingFrame::Heliocentric)
            && matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        {
            return Err(EventError::UnsupportedFrame {
                detail: format!("heliocentric crossings are undefined for {:?}", body),
            });
        }
        let step = Self::step_days(&body);
        // Same clamps as `longitude_crossings_in_range` over `[after, WINDOW_END]`.
        let scan_start = after_jd.max(WINDOW_START_JD + step);
        let scan_end = WINDOW_END_JD.min(WINDOW_END_JD - step);
        let target_deg = target.degrees();
        let root = first_crossing_after(
            |jd| Ok(wrap180(self.longitude_deg(&body, frame, jd)? - target_deg)),
            scan_start,
            scan_end,
            step,
        )?;
        Ok(root.filter(|&jd| jd > after_jd).map(|jd| Crossing {
            body: body.clone(),
            target_longitude: target,
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            frame,
        }))
    }

    /// The last crossing strictly before `before`, or `None`.
    ///
    /// Early-terminating: this brackets and bisects backward from `before` and
    /// returns as soon as the last (highest-JD) root is found, instead of
    /// scanning from `WINDOW_START`. The result is identical to
    /// `longitude_crossings_in_range(body, target, frame, WINDOW_START, before).last()`
    /// filtered to strictly-before `before` — same clamps, same step, same
    /// wrap-seam guard, same bisection tolerance.
    pub fn previous_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        frame: CrossingFrame,
        before: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let before_jd = before.julian_day.days();
        // Preserve the window-check and heliocentric Sun/Moon guard exactly as
        // `longitude_crossings_in_range` applies them, before scanning.
        self.check_window(WINDOW_START_JD)?;
        self.check_window(before_jd)?;
        if matches!(frame, CrossingFrame::Heliocentric)
            && matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        {
            return Err(EventError::UnsupportedFrame {
                detail: format!("heliocentric crossings are undefined for {:?}", body),
            });
        }
        let step = Self::step_days(&body);
        // Same clamps as `longitude_crossings_in_range` over `[WINDOW_START, before]`.
        let scan_start = WINDOW_START_JD.max(WINDOW_START_JD + step);
        let scan_end = before_jd.min(WINDOW_END_JD - step);
        let target_deg = target.degrees();
        let root = last_crossing_before(
            |jd| Ok(wrap180(self.longitude_deg(&body, frame, jd)? - target_deg)),
            scan_start,
            scan_end,
            step,
        )?;
        Ok(root.filter(|&jd| jd < before_jd).map(|jd| Crossing {
            body: body.clone(),
            target_longitude: target,
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            frame,
        }))
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

    /// Ecliptic longitude of `body` in `frame` at `instant` (TDB).
    ///
    /// Geocentric apparent tropical of date for
    /// [`CrossingFrame::GeocentricApparentOfDate`]; heliocentric of date for
    /// [`CrossingFrame::Heliocentric`]. Fails closed outside the packaged
    /// 1900–2100 window and for heliocentric Sun/Moon, matching the crossing
    /// entry points. This is the evaluator the `validate-crossings` parity tier
    /// uses to compare the engine's longitude against a reference crossing time.
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine};
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// let engine = EventEngine::new(packaged_backend());
    /// let t = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let lon = engine
    ///     .longitude_at(CelestialBody::Sun, CrossingFrame::GeocentricApparentOfDate, t)
    ///     .unwrap();
    /// assert!((0.0..360.0).contains(&lon.degrees()));
    /// ```
    pub fn longitude_at(
        &self,
        body: CelestialBody,
        frame: CrossingFrame,
        instant: Instant,
    ) -> Result<Longitude, EventError> {
        let jd = instant.julian_day.days();
        self.check_window(jd)?;
        if matches!(frame, CrossingFrame::Heliocentric)
            && matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        {
            return Err(EventError::UnsupportedFrame {
                detail: format!("heliocentric longitude is undefined for {:?}", body),
            });
        }
        let deg = self.longitude_deg(&body, frame, jd)?;
        Ok(Longitude::from_degrees(deg))
    }
}

/// Deprecated alias for [`EventEngine`]. Kept one release cycle for the SP-2a
/// crossing API; migrate to `EventEngine`.
#[deprecated(since = "0.3.1", note = "renamed to EventEngine")]
pub type CrossingEngine<B> = EventEngine<B>;

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
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
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
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
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
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
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

    #[test]
    fn heliocentric_rejects_sun_and_moon() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = tdb(2_451_545.0);
        for body in [CelestialBody::Sun, CelestialBody::Moon] {
            let err = engine
                .next_longitude_crossing(
                    body.clone(),
                    Longitude::from_degrees(0.0),
                    CrossingFrame::Heliocentric,
                    after,
                )
                .unwrap_err();
            assert!(
                matches!(err, EventError::UnsupportedFrame { .. }),
                "{body:?}"
            );
        }
    }

    #[test]
    fn longitude_at_matches_crossing_target() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = tdb(2_451_545.0);
        let target = Longitude::from_degrees(100.0);
        let c = engine.next_sun_crossing(target, after).unwrap().unwrap();
        // At the crossing instant the engine's longitude must equal the target.
        let lon = engine
            .longitude_at(
                CelestialBody::Sun,
                CrossingFrame::GeocentricApparentOfDate,
                c.instant,
            )
            .unwrap();
        assert!(
            wrap180(lon.degrees() - 100.0).abs() < 1e-3,
            "lon at crossing {}",
            lon.degrees()
        );
    }

    #[test]
    fn previous_equals_last_in_range_filtered_before() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let target = Longitude::from_degrees(100.0);
        // Sun sweeps ~0.9856 deg/day; over 800 days (~788 deg of travel) it
        // crosses any fixed target longitude more than once — confirm that
        // below rather than just asserting it in a comment.
        let before = tdb(WINDOW_START_JD + 800.0);
        let start = tdb(WINDOW_START_JD);
        let full = engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                target,
                CrossingFrame::GeocentricApparentOfDate,
                start,
                before,
            )
            .unwrap();
        assert!(
            full.len() >= 2,
            "expected multiple crossings in-window, got {}",
            full.len()
        );
        let expected = full
            .into_iter()
            .rfind(|c| c.instant.julian_day.days() < before.julian_day.days());
        let actual = engine
            .previous_longitude_crossing(
                CelestialBody::Sun,
                target,
                CrossingFrame::GeocentricApparentOfDate,
                before,
            )
            .unwrap();
        match (expected, actual) {
            (Some(e), Some(a)) => assert!(
                (e.instant.julian_day.days() - a.instant.julian_day.days()).abs() < 1e-6,
                "expected {}, got {}",
                e.instant.julian_day.days(),
                a.instant.julian_day.days()
            ),
            (None, None) => {}
            (e, a) => panic!("expected {e:?}, got {a:?}"),
        }
    }

    #[test]
    fn previous_none_when_no_earlier_crossing() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let target = Longitude::from_degrees(100.0);
        // Very early `before`, right at the start of the window (Sun's scan
        // step is 1.0 day): no crossing can precede it.
        let before = tdb(WINDOW_START_JD + 1.0);
        let actual = engine
            .previous_longitude_crossing(
                CelestialBody::Sun,
                target,
                CrossingFrame::GeocentricApparentOfDate,
                before,
            )
            .unwrap();
        assert!(actual.is_none(), "expected None, got {actual:?}");
    }

    #[test]
    fn longitude_at_fails_closed() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        // Heliocentric Sun is undefined.
        let err = engine
            .longitude_at(
                CelestialBody::Sun,
                CrossingFrame::Heliocentric,
                tdb(2_451_545.0),
            )
            .unwrap_err();
        assert!(matches!(err, EventError::UnsupportedFrame { .. }));
        // Out of the packaged window.
        let err = engine
            .longitude_at(
                CelestialBody::Sun,
                CrossingFrame::GeocentricApparentOfDate,
                tdb(2_000_000.0),
            )
            .unwrap_err();
        assert!(matches!(err, EventError::OutOfWindow { .. }));
    }
}
