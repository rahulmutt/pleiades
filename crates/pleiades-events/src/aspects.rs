//! Aspect finder: the instants the ecliptic separation of two bodies equals
//! a given angle.
//!
//! With `d = wrap180(lon(first) − lon(second))`, an event is a sign change
//! of `d − angle` or of `d + angle`. The scan splits each step at the
//! turning points of `d`, so two exact moments inside one step are both
//! found.

use crate::crossings::{body_label, CrossingFrame, EventEngine};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::reference::{check_supported, ecliptic_in, CrossingReference};
use crate::root::{
    first_level_crossing_after, last_level_crossing_before, level_crossings_in_range, wrap180,
};
use crate::stations::step_days;
use pleiades_backend::EphemerisBackend;
use pleiades_types::{Angle, CelestialBody, Instant, JulianDay, Longitude, TimeScale, ZodiacMode};

/// An exact aspect: an instant at which the ecliptic separation of two
/// bodies equals the requested angle.
///
/// The two longitudes say which body is ahead: `first_longitude −
/// second_longitude` is `+angle` or `−angle`, to within the bodies' motion
/// over the 0.5 s bisection tolerance.
//
// `CelestialBody` is not `Copy`, so neither is this.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct AspectEvent {
    /// The first body of the pair, as passed to the finder.
    pub first: CelestialBody,
    /// The second body of the pair, as passed to the finder.
    pub second: CelestialBody,
    /// The requested separation, in 0–180 degrees.
    pub angle: Angle,
    /// Instant the aspect is exact (TDB). It trails the exact moment by less
    /// than the 0.5 s bisection tolerance and never precedes it.
    pub instant: Instant,
    /// Longitude of `first` at `instant`, in `frame` and `zodiac`.
    pub first_longitude: Longitude,
    /// Longitude of `second` at `instant`, in `frame` and `zodiac`.
    pub second_longitude: Longitude,
    /// The frame the longitudes are measured in.
    pub frame: CrossingFrame,
    /// The zodiac the longitudes are read in.
    pub zodiac: ZodiacMode,
}

fn invalid(detail: String) -> EventError {
    EventError::InvalidAspect { detail }
}

/// The values of the signed separation at which the aspect is exact: one for
/// a conjunction or an opposition, otherwise one on each side.
fn levels_for(angle: Angle) -> Result<Vec<f64>, EventError> {
    let degrees = angle.degrees();
    // A NaN is in no range.
    if !(0.0..=180.0).contains(&degrees) {
        return Err(invalid(format!(
            "the angle must be a separation between 0 and 180 degrees, got {degrees}"
        )));
    }
    if degrees == 0.0 || degrees == 180.0 {
        Ok(vec![degrees])
    } else {
        Ok(vec![degrees, -degrees])
    }
}

/// Step used to bracket aspects: the smaller of the two bodies' station
/// steps, because the separation turns where either body's speed changes.
fn search_step(first: &CelestialBody, second: &CelestialBody) -> f64 {
    step_days(first).min(step_days(second))
}

/// A finite longitude, or [`EventError::MissingCoordinates`]. A NaN must not
/// reach the sign tests, where it would read as "no event".
fn finite_longitude(
    degrees: f64,
    body: &CelestialBody,
    julian_day: f64,
) -> Result<f64, EventError> {
    if degrees.is_finite() {
        Ok(degrees)
    } else {
        Err(EventError::MissingCoordinates {
            body_label: body_label(body),
            julian_day,
        })
    }
}

fn longitude<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<f64, EventError> {
    let (degrees, _, _) = ecliptic_in(backend, body, reference, julian_day)?;
    finite_longitude(degrees, body, julian_day)
}

/// `lon(first) − lon(second)`, wrapped into (-180, 180].
fn separation<B: EphemerisBackend>(
    backend: &B,
    first: &CelestialBody,
    second: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<f64, EventError> {
    let first_deg = longitude(backend, first, reference, julian_day)?;
    let second_deg = longitude(backend, second, reference, julian_day)?;
    Ok(wrap180(first_deg - second_deg))
}

/// What both finders settle before scanning.
struct Search {
    levels: Vec<f64>,
    step: f64,
    /// Earliest and latest Julian day a scan may start and end at: two steps
    /// inside the window, because the scanner samples one step before its
    /// start and up to two past its end.
    earliest: f64,
    latest: f64,
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Validates an aspect request. The angle is checked first, then the
    /// bodies, the window, and the frame and zodiac for each body.
    fn aspect_search(
        &self,
        first: &CelestialBody,
        second: &CelestialBody,
        angle: Angle,
        reference: &CrossingReference,
        instants_jd: [f64; 2],
    ) -> Result<Search, EventError> {
        let levels = levels_for(angle)?;
        if first == second {
            return Err(invalid(format!(
                "an aspect needs two different bodies, got {} twice",
                body_label(first)
            )));
        }
        for jd in instants_jd {
            self.check_window(jd)?;
        }
        check_supported(first, reference, instants_jd[0], "aspects are")?;
        check_supported(second, reference, instants_jd[0], "aspects are")?;
        let step = search_step(first, second);
        Ok(Search {
            levels,
            step,
            earliest: WINDOW_START_JD + 2.0 * step,
            latest: WINDOW_END_JD - 2.0 * step,
        })
    }

    fn aspect_at(
        &self,
        first: &CelestialBody,
        second: &CelestialBody,
        angle: Angle,
        reference: &CrossingReference,
        julian_day: f64,
    ) -> Result<AspectEvent, EventError> {
        let first_deg = longitude(&self.backend, first, reference, julian_day)?;
        let second_deg = longitude(&self.backend, second, reference, julian_day)?;
        Ok(AspectEvent {
            first: first.clone(),
            second: second.clone(),
            angle,
            instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb),
            first_longitude: Longitude::from_degrees(first_deg),
            second_longitude: Longitude::from_degrees(second_deg),
            frame: reference.frame,
            zodiac: reference.zodiac.clone(),
        })
    }

    /// All instants in `[start, end]` (TDB) at which the ecliptic separation
    /// of `first` and `second` equals `angle`, ascending.
    ///
    /// `angle` is an unsigned separation in 0–180 degrees. An angle strictly
    /// between the two is found on both sides: asking for 90 degrees returns
    /// the moments `first` is 90 degrees ahead of `second` and the moments
    /// it is 90 degrees behind. The two longitudes of each [`AspectEvent`]
    /// say which. The moment a pair enters a 3-degree orb of a square is the
    /// exact moment of the 87-degree (or 93-degree) separation.
    ///
    /// `reference` is a [`CrossingFrame`] (tropical zodiac) or a
    /// [`CrossingReference`] carrying a sidereal zodiac. The zodiac changes
    /// the reported longitudes only: the ayanamsa comes off both longitudes
    /// and cancels in the separation.
    ///
    /// An event is in the range when it has not happened at `start` and has
    /// happened by `end`, judged by the separation at each end and not by
    /// comparing a returned instant with it. Two ranges that share an end
    /// therefore hold each event exactly once, wherever that end falls,
    /// including on an instant this engine returned.
    ///
    /// A pair that approaches the angle and turns back before reaching it
    /// returns no event; that is not an error. An empty or inverted range
    /// returns an empty list, but the scan still samples both bodies at its
    /// start, so a body the backend cannot serve still returns its error.
    ///
    /// # Accuracy
    ///
    /// The search steps by the smaller of the two bodies' steps: 0.25 day
    /// for the Moon and the lunar points, 1 day for the Sun, Mercury and
    /// Venus, and 2 days otherwise. Each step is split at the turning points
    /// of the separation, so two exact moments inside one step, around a
    /// station of either body, are both found. Three limits remain:
    ///
    /// - two turning points of the separation within two steps of each other
    ///   may go unseen, and with them a pair of exact moments between them;
    /// - a pair whose separation passes the angle by less than the noise of
    ///   the ephemeris may be found or not;
    /// - an event within two steps of either end of the 1900–2100 window is
    ///   not reported, because the scan keeps its samples inside the window.
    ///
    /// The 0.5 s bisection tolerance bounds how well the engine locates the
    /// exact moment in its own longitudes, not how well that moment matches
    /// another ephemeris. The time difference is the difference in the
    /// separation divided by the pair's relative speed, and the relative
    /// speed falls to zero where the separation turns: an aspect instant is
    /// firm for a fast pair and soft for a slow pair near a station. See the
    /// crate README for the figures measured against Swiss Ephemeris.
    ///
    /// # Errors
    ///
    /// [`EventError::InvalidAspect`] for a non-finite angle, an angle outside
    /// 0–180 degrees, or the same body twice; otherwise the same as
    /// [`EventEngine::longitude_at`] for either body:
    /// [`EventError::OutOfWindow`], [`EventError::UnsupportedFrame`],
    /// [`EventError::MissingCoordinates`], [`EventError::Backend`].
    pub fn aspects_in_range(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angle: Angle,
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<AspectEvent>, EventError> {
        let reference = reference.into();
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        let search = self.aspect_search(&first, &second, angle, &reference, [start_jd, end_jd])?;
        let roots = level_crossings_in_range(
            |jd| separation(&self.backend, &first, &second, &reference, jd),
            &search.levels,
            start_jd.max(search.earliest),
            end_jd.min(search.latest),
            search.step,
        )?;
        roots
            .into_iter()
            .map(|jd| self.aspect_at(&first, &second, angle, &reference, jd))
            .collect()
    }

    /// The first instant strictly after `after` at which the ecliptic
    /// separation of `first` and `second` equals `angle`, or `None`.
    ///
    /// Identical to the first element of
    /// `aspects_in_range(first, second, angle, reference, after, WINDOW_END)`
    /// that is strictly after `after`, but stops at the first event found. A
    /// returned [`AspectEvent::instant`] can be handed back as `after`: the
    /// search then returns the following event, not the same one.
    ///
    /// For a pair that never reaches the angle (the Sun and Mercury at 60
    /// degrees) the search runs to the end of the 1900–2100 window before
    /// returning `None`.
    ///
    /// The meaning of `angle`, the accuracy, the limits and the errors are
    /// those of [`EventEngine::aspects_in_range`].
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine};
    /// use pleiades_types::{Angle, CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// // The Jupiter–Saturn great conjunction of 28 May 2000.
    /// let engine = EventEngine::new(packaged_backend());
    /// let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let conjunction = engine
    ///     .next_aspect(
    ///         CelestialBody::Jupiter,
    ///         CelestialBody::Saturn,
    ///         Angle::from_degrees(0.0),
    ///         CrossingFrame::GeocentricApparentOfDate,
    ///         after,
    ///     )
    ///     .unwrap()
    ///     .expect("Jupiter and Saturn meet every twenty years");
    /// let days = conjunction.instant.julian_day.days() - 2_451_545.0;
    /// assert!((148.0..148.4).contains(&days), "{days}");
    /// let apart = conjunction.first_longitude.degrees() - conjunction.second_longitude.degrees();
    /// assert!(apart.abs() < 1e-4, "{apart}");
    /// ```
    pub fn next_aspect(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angle: Angle,
        reference: impl Into<CrossingReference>,
        after: Instant,
    ) -> Result<Option<AspectEvent>, EventError> {
        let reference = reference.into();
        let after_jd = after.julian_day.days();
        let search =
            self.aspect_search(&first, &second, angle, &reference, [after_jd, after_jd])?;
        let root = first_level_crossing_after(
            |jd| separation(&self.backend, &first, &second, &reference, jd),
            &search.levels,
            after_jd.max(search.earliest),
            search.latest,
            search.step,
        )?;
        root.filter(|&jd| jd > after_jd)
            .map(|jd| self.aspect_at(&first, &second, angle, &reference, jd))
            .transpose()
    }

    /// The last instant at or before `before` at which the ecliptic
    /// separation of `first` and `second` equals `angle`, or `None`.
    ///
    /// The event `aspects_in_range(first, second, angle, reference,
    /// WINDOW_START, before).last()` finds, located without scanning the
    /// whole range: the search walks backward from `before` in chunks and
    /// stops at the first chunk that holds an event. The two agree on the
    /// instant to within the 0.5 s bisection tolerance, not bit for bit.
    ///
    /// "At or before" is decided by the separation at `before`, as the
    /// crossing and station searches decide it: an event counts when it has
    /// already happened there. Given an [`AspectEvent::instant`] this engine
    /// returned, the result is that same event, since a returned instant
    /// trails its event by less than the tolerance and never precedes it. To
    /// step back to the event before it, move `before` back by more than the
    /// tolerance, a second say.
    ///
    /// For a pair that never reaches the angle (the Sun and Mercury at 60
    /// degrees) the search runs to the start of the 1900–2100 window before
    /// returning `None`.
    ///
    /// The meaning of `angle`, the accuracy, the limits and the errors are
    /// those of [`EventEngine::aspects_in_range`].
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine};
    /// use pleiades_types::{Angle, CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// // The last full Moon before J2000: 22 December 1999.
    /// let engine = EventEngine::new(packaged_backend());
    /// let before = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let full_moon = engine
    ///     .previous_aspect(
    ///         CelestialBody::Sun,
    ///         CelestialBody::Moon,
    ///         Angle::from_degrees(180.0),
    ///         CrossingFrame::GeocentricApparentOfDate,
    ///         before,
    ///     )
    ///     .unwrap()
    ///     .expect("the Moon is full every month");
    /// let days = 2_451_545.0 - full_moon.instant.julian_day.days();
    /// assert!((9.5..10.5).contains(&days), "{days}");
    /// ```
    pub fn previous_aspect(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angle: Angle,
        reference: impl Into<CrossingReference>,
        before: Instant,
    ) -> Result<Option<AspectEvent>, EventError> {
        let reference = reference.into();
        let before_jd = before.julian_day.days();
        let search =
            self.aspect_search(&first, &second, angle, &reference, [before_jd, before_jd])?;
        let root = last_level_crossing_before(
            |jd| separation(&self.backend, &first, &second, &reference, jd),
            &search.levels,
            search.earliest,
            before_jd.min(search.latest),
            search.step,
        )?;
        root.map(|jd| self.aspect_at(&first, &second, angle, &reference, jd))
            .transpose()
    }
}

#[cfg(test)]
mod tests;
