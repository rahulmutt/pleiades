//! The public longitude-crossing engine.

use crate::error::{
    before_window_start, past_window_end, EventError, WINDOW_END_JD, WINDOW_START_JD,
};
use crate::reference::{check_supported, ecliptic_in, CrossingReference};
use crate::rise_trans::PlaceCache;
use crate::root::{crossings_in_range, first_crossing_after, last_crossing_before, wrap180};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale, ZodiacMode};

/// The coordinate/center convention a crossing is computed in.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum CrossingFrame {
    /// Geocentric apparent tropical ecliptic of date (SE `solcross`/`mooncross`).
    GeocentricApparentOfDate,
    /// Heliocentric ecliptic (SE `helio_cross`); planets only.
    Heliocentric,
    /// Geocentric geometric place in the mean ecliptic and equinox of date:
    /// the backend's J2000 place precessed to date, with no light-time, no
    /// aberration and no nutation (SE `SEFLG_TRUEPOS | SEFLG_NOABERR |
    /// SEFLG_NOGDEFL | SEFLG_NONUT`). This is not the J2000 longitude a
    /// `pleiades-core` mean chart reports.
    GeocentricMeanOfDate,
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
    /// The ecliptic longitude that was crossed, in `zodiac`.
    pub target_longitude: Longitude,
    /// Instant of the crossing (TDB).
    pub instant: Instant,
    /// The frame the crossing was computed in.
    pub frame: CrossingFrame,
    /// The zodiac `target_longitude` is read in.
    #[cfg_attr(
        feature = "serde",
        serde(default = "crate::reference::tropical_zodiac")
    )]
    pub zodiac: ZodiacMode,
}

/// Finds ephemeris events (longitude crossings today; rise/set/transit and
/// horizontal coordinates in sibling modules) over the packaged 1900–2100 TDB
/// window.
///
/// # Reuse one engine across a sweep
///
/// An engine remembers the body places its rise, set and transit searches
/// have read, and later searches reuse them: the three searches of a daily
/// sunrise bracket share their samples, and a sweep of daily brackets on one
/// engine reads only the samples each new day adds. Build one engine for a
/// sweep and pass it by reference, rather than one per search.
///
/// What the engine remembers never changes an answer. A remembered place is
/// the one a fresh engine would read at the same instant, and it does not
/// depend on the observer, the atmosphere or the options. The memory is
/// bounded, and the engine can be shared between threads.
pub struct EventEngine<B> {
    pub(crate) backend: B,
    /// Body samples shared by the rise/set and transit searches.
    pub(crate) places: PlaceCache,
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Wraps a backend.
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            places: PlaceCache::new(),
        }
    }

    /// An engine whose sample cache holds at most `capacity` entries.
    #[cfg(test)]
    pub(crate) fn with_place_cache_capacity(backend: B, capacity: usize) -> Self {
        Self {
            backend,
            places: PlaceCache::with_capacity(capacity),
        }
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

    pub(crate) fn check_window(&self, jd: f64) -> Result<(), EventError> {
        if !(WINDOW_START_JD..=WINDOW_END_JD).contains(&jd) {
            return Err(EventError::OutOfWindow { julian_day: jd });
        }
        Ok(())
    }

    fn crossing(
        body: &CelestialBody,
        target: Longitude,
        reference: &CrossingReference,
        jd: f64,
    ) -> Crossing {
        Crossing {
            body: body.clone(),
            target_longitude: target,
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            frame: reference.frame,
            zodiac: reference.zodiac.clone(),
        }
    }

    /// All crossings of `target` by `body` from `start` to `end` (TDB),
    /// ascending.
    ///
    /// Each end decides membership by the body's longitude AT that instant,
    /// not by comparing a refined instant with it: a crossing is in range
    /// when it has not yet happened at `start` and has happened by `end`.
    /// Ranges that share an end therefore split the crossings between them
    /// with none repeated and none dropped, whatever that end is. In
    /// particular a returned [`Crossing::instant`] (which trails its crossing
    /// by less than the 0.5 s bisection tolerance) used as `end` includes
    /// that crossing, and used as `start` excludes it.
    ///
    /// `reference` is a [`CrossingFrame`] (tropical zodiac) or a
    /// [`CrossingReference`] carrying a sidereal zodiac; `target` is read in
    /// that zodiac. An ayanamsa with no offset data is
    /// [`EventError::UnsupportedFrame`].
    ///
    /// The scan runs to both ends of the range, which may be the window's own
    /// limits. An apparent place of a body other than the Sun is read a
    /// light-time earlier, so a range starting within a light-time of the
    /// window's first instant is [`EventError::OutOfWindow`].
    pub fn longitude_crossings_in_range(
        &self,
        body: CelestialBody,
        target: Longitude,
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<Crossing>, EventError> {
        let reference = reference.into();
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;
        check_supported(&body, &reference, start_jd, "crossings are")?;
        let step = Self::step_days(&body);
        let target_deg = target.degrees();
        let roots = crossings_in_range(
            |jd| {
                Ok(wrap180(
                    ecliptic_in(&self.backend, &body, &reference, jd)?.0 - target_deg,
                ))
            },
            start_jd,
            end_jd,
            step,
        )?;
        Ok(roots
            .into_iter()
            .map(|jd| Self::crossing(&body, target, &reference, jd))
            .collect())
    }

    /// The first crossing strictly after `after`.
    ///
    /// Early-terminating: this brackets and bisects forward from `after` and
    /// returns as soon as the first root is found, instead of scanning to
    /// `WINDOW_END`. The result is identical to
    /// `longitude_crossings_in_range(body, target, reference, after, WINDOW_END).first()`
    /// filtered to strictly-after `after` — same step, same
    /// wrap-seam guard, same bisection tolerance.
    ///
    /// A returned [`Crossing::instant`] trails its crossing by less than the
    /// 0.5 s bisection tolerance and never precedes it, so handing it back as
    /// `after` returns the following crossing, not the same one again.
    ///
    /// `reference` is a [`CrossingFrame`] (tropical zodiac) or a
    /// [`CrossingReference`] carrying a sidereal zodiac; `target` is read in
    /// that zodiac. An ayanamsa with no offset data is
    /// [`EventError::UnsupportedFrame`].
    ///
    /// When the window ends before the next crossing, the result is
    /// [`EventError::OutOfWindow`] naming the instant one step past the
    /// window's end. Every body crosses every longitude, so this search never
    /// returns `Ok(None)`. An `after` within a light-time of the window's first
    /// instant in the apparent frame, for a body other than the Sun, is
    /// `OutOfWindow` too: the read at `after` fails.
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, CrossingReference, EventEngine};
    /// use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, Longitude, TimeScale};
    ///
    /// // The Sun's next entry into sidereal Aries (Lahiri), after J2000.
    /// let engine = EventEngine::new(packaged_backend());
    /// let lahiri =
    ///     CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::Lahiri);
    /// let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let ingress = engine
    ///     .next_longitude_crossing(CelestialBody::Sun, Longitude::from_degrees(0.0), lahiri, after)
    ///     .unwrap()
    ///     .expect("the Sun enters sidereal Aries every year");
    /// // Mid-April, about 24 days after the tropical equinox.
    /// let days = ingress.instant.julian_day.days() - 2_451_545.0;
    /// assert!((100.0..110.0).contains(&days), "{days}");
    /// ```
    pub fn next_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        reference: impl Into<CrossingReference>,
        after: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let reference = reference.into();
        let after_jd = after.julian_day.days();
        // Same checks, in the same order, as `longitude_crossings_in_range`.
        self.check_window(after_jd)?;
        self.check_window(WINDOW_END_JD)?;
        check_supported(&body, &reference, after_jd, "crossings are")?;
        let step = Self::step_days(&body);
        let target_deg = target.degrees();
        let root = first_crossing_after(
            |jd| {
                Ok(wrap180(
                    ecliptic_in(&self.backend, &body, &reference, jd)?.0 - target_deg,
                ))
            },
            after_jd,
            WINDOW_END_JD,
            step,
        )?;
        let jd = root.ok_or_else(|| past_window_end(step))?;
        Ok(Some(Self::crossing(&body, target, &reference, jd)))
    }

    /// The last crossing that has happened by `before`.
    ///
    /// Early-terminating: this brackets and bisects backward from `before` and
    /// returns as soon as the last (highest-JD) root is found, instead of
    /// scanning from `WINDOW_START`. It finds the crossing
    /// `longitude_crossings_in_range(body, target, reference, WINDOW_START, before).last()`
    /// finds — same step, same wrap-seam guard — and agrees with
    /// it on the instant to within the 0.5 s bisection tolerance. Its scan is
    /// anchored at `before`, the range's at its start, so the two are not
    /// bit-identical.
    ///
    /// Whether a crossing has happened by `before` is read from the body's
    /// longitude AT `before`, as [`previous_rise_set`](Self::previous_rise_set)
    /// reads its events. A returned [`Crossing::instant`] trails its crossing
    /// by less than the bisection tolerance and never precedes it, so handing
    /// it back as `before` returns that same crossing, at an instant no later
    /// than `before` (issue #159). To step back to the crossing before it,
    /// move `before` back by a second.
    ///
    /// `reference` is a [`CrossingFrame`] (tropical zodiac) or a
    /// [`CrossingReference`] carrying a sidereal zodiac; `target` is read in
    /// that zodiac. An ayanamsa with no offset data is
    /// [`EventError::UnsupportedFrame`].
    ///
    /// When the window starts after the previous crossing, the result is
    /// [`EventError::OutOfWindow`] naming the instant one step before the
    /// window's start. Every body crosses every longitude, so this search
    /// never returns `Ok(None)`.
    ///
    /// When the search reaches the window's first light-time in the apparent
    /// frame, for a body other than the Sun, the error comes from that read
    /// and names its instant instead.
    pub fn previous_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        reference: impl Into<CrossingReference>,
        before: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let reference = reference.into();
        let before_jd = before.julian_day.days();
        // Same checks, in the same order, as `longitude_crossings_in_range`.
        self.check_window(WINDOW_START_JD)?;
        self.check_window(before_jd)?;
        check_supported(&body, &reference, before_jd, "crossings are")?;
        let step = Self::step_days(&body);
        let target_deg = target.degrees();
        let root = last_crossing_before(
            |jd| {
                Ok(wrap180(
                    ecliptic_in(&self.backend, &body, &reference, jd)?.0 - target_deg,
                ))
            },
            WINDOW_START_JD,
            before_jd,
            step,
        )?;
        let jd = root.ok_or_else(|| before_window_start(step))?;
        Ok(Some(Self::crossing(&body, target, &reference, jd)))
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

    /// Ecliptic longitude of `body` in `reference` at `instant` (TDB).
    ///
    /// `reference` is a [`CrossingFrame`] (tropical zodiac) or a
    /// [`CrossingReference`] carrying a sidereal zodiac. A sidereal zodiac in
    /// the heliocentric frame, and an ayanamsa with no offset data, are
    /// [`EventError::UnsupportedFrame`].
    ///
    /// Geocentric apparent tropical of date for
    /// [`CrossingFrame::GeocentricApparentOfDate`]; geocentric geometric, mean
    /// equinox of date for [`CrossingFrame::GeocentricMeanOfDate`]; heliocentric
    /// of date for [`CrossingFrame::Heliocentric`]. Fails closed outside the
    /// packaged 1900–2100 window and for a heliocentric Sun, Moon or lunar
    /// orbit point, matching the crossing entry points. This is the evaluator
    /// the `validate-crossings` parity tier uses to compare the engine's
    /// longitude against a reference crossing time.
    ///
    /// A lunar orbit point (mean or true node, apogee or perigee) is a
    /// direction of the lunar orbit, not a body: in the geocentric frames it is
    /// precessed, and in the apparent frame also rotated by nutation in
    /// longitude, with no light-time and no aberration, as the `pleiades-core`
    /// chart layer does. A backend may serve it without a distance (issue
    /// #118).
    ///
    /// An apparent place of a body other than the Sun is read a light-time
    /// before `instant`, so within a light-time of the window start it is
    /// [`EventError::OutOfWindow`] (issue #163).
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
        reference: impl Into<CrossingReference>,
        instant: Instant,
    ) -> Result<Longitude, EventError> {
        let reference = reference.into();
        let jd = instant.julian_day.days();
        self.check_window(jd)?;
        check_supported(&body, &reference, jd, "longitude is")?;
        let (deg, _, _) = ecliptic_in(&self.backend, &body, &reference, jd)?;
        Ok(Longitude::from_degrees(deg))
    }
}

/// Deprecated alias for [`EventEngine`]. Kept one release cycle for the SP-2a
/// crossing API; migrate to `EventEngine`.
#[deprecated(since = "0.3.1", note = "renamed to EventEngine")]
pub type CrossingEngine<B> = EventEngine<B>;

pub(crate) fn body_label(body: &CelestialBody) -> &'static str {
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
        CelestialBody::MeanNode => "mean node",
        CelestialBody::TrueNode => "true node",
        CelestialBody::MeanApogee => "mean apogee",
        CelestialBody::TrueApogee => "true apogee",
        CelestialBody::MeanPerigee => "mean perigee",
        CelestialBody::TruePerigee => "true perigee",
        _ => "body",
    }
}

#[cfg(test)]
mod window_edge_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ephemeris::geocentric_apparent_longitude_deg;
    use crate::root::REFINE_TOLERANCE_DAYS;
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
        let expected = full.into_iter().next_back();
        let actual = engine
            .previous_longitude_crossing(
                CelestialBody::Sun,
                target,
                CrossingFrame::GeocentricApparentOfDate,
                before,
            )
            .unwrap();
        match (expected, actual) {
            // The two scans are anchored at opposite ends of the range.
            (Some(e), Some(a)) => assert!(
                (e.instant.julian_day.days() - a.instant.julian_day.days()).abs()
                    < REFINE_TOLERANCE_DAYS,
                "expected {}, got {}",
                e.instant.julian_day.days(),
                a.instant.julian_day.days()
            ),
            (None, None) => {}
            (e, a) => panic!("expected {e:?}, got {a:?}"),
        }
    }

    #[test]
    fn previous_is_out_of_window_when_no_earlier_crossing_in_window() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let target = Longitude::from_degrees(100.0);
        // Very early `before`, right at the start of the window (Sun's scan
        // step is 1.0 day): the search reaches the window's start, so it is
        // cut short rather than known to be empty.
        let before = tdb(WINDOW_START_JD + 1.0);
        let actual = engine.previous_longitude_crossing(
            CelestialBody::Sun,
            target,
            CrossingFrame::GeocentricApparentOfDate,
            before,
        );
        assert!(
            matches!(actual, Err(EventError::OutOfWindow { .. })),
            "expected OutOfWindow, got {actual:?}"
        );
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

    // Issue #80's family: a crossing instant handed back as `after` must not
    // find the same crossing again. Every returned instant is the settled
    // end of its bisection bracket (see `root::bisect`), so the search reads
    // the post-crossing sign there. Targets spread round the zodiac put the
    // roots on both halves of the final bracket.
    #[test]
    fn next_after_a_returned_crossing_is_the_following_one() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let frame = CrossingFrame::GeocentricApparentOfDate;
        for target_deg in (0..24).map(|i| 7.3 + 15.0 * f64::from(i)) {
            let target = Longitude::from_degrees(target_deg);
            let found = engine
                .previous_longitude_crossing(
                    CelestialBody::Moon,
                    target,
                    frame,
                    tdb(2_451_545.0 + 60.0),
                )
                .unwrap()
                .expect("the Moon crosses every longitude monthly");
            let following = engine
                .next_longitude_crossing(CelestialBody::Moon, target, frame, found.instant)
                .unwrap()
                .expect("the Moon crosses every longitude monthly");
            let gap = following.instant.julian_day.days() - found.instant.julian_day.days();
            assert!(
                (27.0..28.0).contains(&gap),
                "target {target_deg}: {gap} days between a crossing and the next"
            );
        }
    }

    // Issue #159: the backward search and the range ends read the body's
    // longitude at the query instant, as the forward search does.

    const SECOND_DAYS: f64 = 1.0 / 86_400.0;

    /// Moon crossings of targets spread round the zodiac, each found forward
    /// from the same instant.
    fn moon_crossings(engine: &EventEngine<LinearSunMoon>) -> Vec<Crossing> {
        (0..24)
            .map(|i| {
                engine
                    .next_longitude_crossing(
                        CelestialBody::Moon,
                        Longitude::from_degrees(7.3 + 15.0 * f64::from(i)),
                        CrossingFrame::GeocentricApparentOfDate,
                        tdb(2_451_545.0 + 60.0),
                    )
                    .unwrap()
                    .expect("the Moon crosses every longitude monthly")
            })
            .collect()
    }

    fn days_between(earlier: &Crossing, later: &Crossing) -> f64 {
        later.instant.julian_day.days() - earlier.instant.julian_day.days()
    }

    #[test]
    fn previous_before_a_returned_crossing_is_that_crossing() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        for found in moon_crossings(&engine) {
            let again = engine
                .previous_longitude_crossing(
                    CelestialBody::Moon,
                    found.target_longitude,
                    found.frame,
                    found.instant,
                )
                .unwrap()
                .expect("the crossing has happened by its returned instant");
            let gap = days_between(&again, &found);
            assert!(
                (0.0..=REFINE_TOLERANCE_DAYS).contains(&gap),
                "target {:?}: {gap} days from the crossing to its returned instant",
                found.target_longitude
            );
        }
    }

    #[test]
    fn previous_a_second_before_a_returned_crossing_is_the_one_before() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        for found in moon_crossings(&engine) {
            let earlier = engine
                .previous_longitude_crossing(
                    CelestialBody::Moon,
                    found.target_longitude,
                    found.frame,
                    tdb(found.instant.julian_day.days() - SECOND_DAYS),
                )
                .unwrap()
                .expect("the Moon crosses every longitude monthly");
            let gap = days_between(&earlier, &found);
            assert!(
                (27.0..28.0).contains(&gap),
                "target {:?}: {gap} days between a crossing and the one before",
                found.target_longitude
            );
        }
    }

    #[test]
    fn ranges_split_near_a_crossing_hold_it_exactly_once() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        for found in moon_crossings(&engine) {
            let at = found.instant.julian_day.days();
            let count = |from: f64, to: f64| {
                engine
                    .longitude_crossings_in_range(
                        CelestialBody::Moon,
                        found.target_longitude,
                        found.frame,
                        tdb(from),
                        tdb(to),
                    )
                    .unwrap()
                    .len()
            };
            // The returned instant trails the crossing by under half a
            // second, so these splits fall on both sides of it.
            for offset_s in [-2.0, -0.6, -0.4, -0.2, -0.05, 0.0, 0.3, 2.0] {
                let split = at + offset_s * SECOND_DAYS;
                assert_eq!(
                    count(at - 5.0, split) + count(split, at + 5.0),
                    1,
                    "target {:?} split {offset_s} s from the returned instant",
                    found.target_longitude
                );
            }
            // Ending a range at the returned instant includes the crossing;
            // starting one there excludes it.
            assert_eq!(count(at - 5.0, at), 1);
            assert_eq!(count(at, at + 5.0), 0);
        }
    }
}
