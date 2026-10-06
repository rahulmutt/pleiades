//! Station finder: the instants a body's longitude speed changes sign.
//!
//! A station is a sign change of the longitude speed
//! [`EventEngine::position_at`] reports, so it is exactly where the engine's
//! own direction flips, in every frame and zodiac.

use crate::crossings::{body_label, CrossingFrame, EventEngine};
use crate::error::{
    before_window_start, past_window_end, EventError, WINDOW_END_JD, WINDOW_START_JD,
};
use crate::position::place_and_motion;
use crate::reference::{check_supported, CrossingReference};
use crate::root::{crossings_in_range, first_crossing_after, last_crossing_before};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale, ZodiacMode};

/// Which way a body turns at a station.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum StationKind {
    /// Direct before the station, retrograde after it.
    TurnsRetrograde,
    /// Retrograde before the station, direct after it.
    TurnsDirect,
}

/// A station: an instant at which a body's longitude speed changes sign.
//
// `CelestialBody` is not `Copy`, so neither is this.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Station {
    /// The body that stations.
    pub body: CelestialBody,
    /// Instant of the station (TDB). It trails the sign change by less than
    /// the 0.5 s bisection tolerance and never precedes it.
    pub instant: Instant,
    /// Longitude at `instant`, in `frame` and `zodiac`.
    pub longitude: Longitude,
    /// Which way the body turns.
    pub kind: StationKind,
    /// The frame the speed was measured in.
    pub frame: CrossingFrame,
    /// The zodiac the speed and `longitude` are read in.
    pub zodiac: ZodiacMode,
}

/// Step used to bracket stations: well under the shortest interval between
/// two stations of the body. Mercury's shortest retrograde is about 19 days;
/// the lunar points' speeds oscillate within a month.
pub(crate) fn step_days(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Moon
        | CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => 0.25,
        CelestialBody::Sun | CelestialBody::Mercury | CelestialBody::Venus => 1.0,
        _ => 2.0,
    }
}

/// Whether `body` is known never to station in `frame`, so that a search can
/// answer without scanning the window (issue #167 (e)).
///
/// Geocentrically that is the Sun, the Moon and the mean lunar points;
/// heliocentrically it is the planets, whose longitude only ever increases.
/// Each one's speed keeps its sign well clear of zero over the whole window
/// (`the_bodies_answered_without_a_scan_keep_one_direction_all_window`), and
/// an ayanamsa's rate, about 50″ a year, is far too small to change it.
///
/// Every other body is scanned, including those that happen never to
/// station: an asteroid's or a fictitious body's heliocentric speed comes
/// from data that is not trusted to keep its sign (issue #158), and a station
/// is defined as a sign change of the speed the engine reports.
fn never_stations(body: &CelestialBody, frame: CrossingFrame) -> bool {
    use CelestialBody::{
        Jupiter, Mars, MeanApogee, MeanNode, MeanPerigee, Mercury, Moon, Neptune, Pluto, Saturn,
        Sun, Uranus, Venus,
    };
    match frame {
        CrossingFrame::Heliocentric => matches!(
            body,
            Mercury | Venus | Mars | Jupiter | Saturn | Uranus | Neptune | Pluto
        ),
        CrossingFrame::GeocentricApparentOfDate | CrossingFrame::GeocentricMeanOfDate => {
            matches!(body, Sun | Moon | MeanNode | MeanApogee | MeanPerigee)
        }
    }
}

/// The settled instant carries the post-station sign; `root::bisect` counts
/// zero as the negative side.
fn kind_of(settled_speed: f64) -> StationKind {
    if settled_speed > 0.0 {
        StationKind::TurnsDirect
    } else {
        StationKind::TurnsRetrograde
    }
}

/// A finite speed, or [`EventError::MissingSpeed`]. A NaN must not reach the
/// sign test, where it would read as "direct".
fn checked_speed(
    speed: Option<f64>,
    body: &CelestialBody,
    julian_day: f64,
) -> Result<f64, EventError> {
    speed
        .filter(|speed| speed.is_finite())
        .ok_or(EventError::MissingSpeed {
            body_label: body_label(body),
            julian_day,
        })
}

fn longitude_speed<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<f64, EventError> {
    let (_, motion) = place_and_motion(backend, body, reference, julian_day)?;
    checked_speed(motion.longitude_deg_per_day, body, julian_day)
}

impl<B: EphemerisBackend> EventEngine<B> {
    fn station_at(
        &self,
        body: &CelestialBody,
        reference: &CrossingReference,
        julian_day: f64,
    ) -> Result<Station, EventError> {
        let (ecliptic, motion) = place_and_motion(&self.backend, body, reference, julian_day)?;
        let speed = checked_speed(motion.longitude_deg_per_day, body, julian_day)?;
        Ok(Station {
            body: body.clone(),
            instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb),
            longitude: ecliptic.longitude,
            kind: kind_of(speed),
            frame: reference.frame,
            zodiac: reference.zodiac.clone(),
        })
    }

    /// All stations of `body` in `[start, end]` (TDB), ascending.
    ///
    /// A station is a sign change of the longitude speed
    /// [`EventEngine::position_at`] reports in `reference`, so
    /// `position_at` just before and at a returned instant always disagree
    /// in direction. `reference` is a [`CrossingFrame`] (tropical zodiac) or
    /// a [`CrossingReference`] carrying a sidereal zodiac. The zodiac
    /// matters: a sidereal speed is lower by the ayanamsa's rate, which
    /// moves a slow planet's station by minutes to hours.
    ///
    /// A body that never stations in `reference` (the Sun, the Moon, the
    /// mean node, every body in the heliocentric frame) returns an empty
    /// list. For the Sun, the Moon, the mean lunar points and the
    /// heliocentric planets that answer comes without scanning the range.
    /// An empty or inverted range returns an empty list, but the scan
    /// still samples the speed at its start, so a body the backend cannot
    /// serve, or one with no longitude speed, still returns its error.
    ///
    /// The scan runs to both ends of the range. An apparent place of a body
    /// other than the Sun is read a light-time earlier, so a range starting
    /// within a light-time of the window's first instant is
    /// [`EventError::OutOfWindow`].
    ///
    /// # Accuracy
    ///
    /// The search steps by 0.25 day for the Moon and the lunar points, 1 day
    /// for the Sun, Mercury and Venus, and 2 days otherwise. Two stations
    /// closer together than the step are not reported. That happens only
    /// for the osculating lunar points: the true node's speed touches zero
    /// about every two weeks, and whether a touch crosses zero for a few
    /// hours depends on the ephemeris. On the packaged backend the node's
    /// speed is within 0.71″/day of the derivative of the node formed from
    /// the exact DE440 Moon (2017–2027), and its peaks in #108's short
    /// direct spells match Swiss Ephemeris within 0.1″/day.
    ///
    /// The 0.5 s bisection tolerance bounds how well the engine locates the
    /// zero of its own speed, not how well that zero matches another
    /// ephemeris: near a station the speed changes slowly, so a small speed
    /// difference is a large time difference. Measured against the sign
    /// changes of Swiss Ephemeris's longitude speed over 1900–2100 (geocentric
    /// apparent, 2026-10-02), the largest time difference runs from 9.3 s
    /// for Mercury to 1292.7 s (about 22 minutes) for Pluto, and every
    /// planet's longitude at the station agrees within 2.314″ (Neptune). The
    /// true node is only checked for the existence and kind of its
    /// well-separated stations, not for their timing. See the crate README
    /// for the per-body figures.
    ///
    /// # Errors
    ///
    /// The same as [`EventEngine::position_at`] —
    /// [`EventError::OutOfWindow`], [`EventError::UnsupportedFrame`],
    /// [`EventError::MissingCoordinates`], [`EventError::Backend`] — and
    /// [`EventError::MissingSpeed`] when the backend reports no finite
    /// longitude speed. A missing speed is never read as "no station".
    pub fn stations_in_range(
        &self,
        body: CelestialBody,
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<Station>, EventError> {
        let reference = reference.into();
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;
        check_supported(&body, &reference, start_jd, "stations are")?;
        let step = step_days(&body);
        if never_stations(&body, reference.frame) {
            // The one sample an empty range gets, for the same errors.
            longitude_speed(&self.backend, &body, &reference, start_jd)?;
            return Ok(Vec::new());
        }
        let roots = crossings_in_range(
            |jd| longitude_speed(&self.backend, &body, &reference, jd),
            start_jd,
            end_jd,
            step,
        )?;
        roots
            .into_iter()
            .map(|jd| self.station_at(&body, &reference, jd))
            .collect()
    }

    /// The first station of `body` strictly after `after`, or `None`.
    ///
    /// Identical to the first element of
    /// `stations_in_range(body, reference, after, WINDOW_END)` that is
    /// strictly after `after`, but stops at the first station found. A
    /// returned [`Station::instant`] can be handed back as `after`: the
    /// search then returns the following station, not the same one.
    ///
    /// The Sun, the Moon, the mean lunar points and, in the heliocentric
    /// frame, the planets never station and return `None` at once. Any other
    /// body that happens never to station (an asteroid in the heliocentric
    /// frame) is searched to the end of the 1900–2100 window first.
    ///
    /// Any other body is searched to the end of the 1900–2100 window; when
    /// the window ends first, the result is [`EventError::OutOfWindow`]
    /// naming the instant one step past it.
    ///
    /// The accuracy, step and errors are those of
    /// [`EventEngine::stations_in_range`].
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine, StationKind};
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// // Mercury's first station of 2000: it turns retrograde on 21 February.
    /// let engine = EventEngine::new(packaged_backend());
    /// let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let station = engine
    ///     .next_station(CelestialBody::Mercury, CrossingFrame::GeocentricApparentOfDate, after)
    ///     .unwrap()
    ///     .expect("Mercury stations six times a year");
    /// assert_eq!(station.kind, StationKind::TurnsRetrograde);
    /// let days = station.instant.julian_day.days() - 2_451_545.0;
    /// assert!((49.0..54.0).contains(&days), "{days}");
    /// ```
    pub fn next_station(
        &self,
        body: CelestialBody,
        reference: impl Into<CrossingReference>,
        after: Instant,
    ) -> Result<Option<Station>, EventError> {
        let reference = reference.into();
        let after_jd = after.julian_day.days();
        self.check_window(after_jd)?;
        check_supported(&body, &reference, after_jd, "stations are")?;
        let step = step_days(&body);
        if never_stations(&body, reference.frame) {
            longitude_speed(&self.backend, &body, &reference, after_jd)?;
            return Ok(None);
        }
        let root = first_crossing_after(
            |jd| longitude_speed(&self.backend, &body, &reference, jd),
            after_jd,
            WINDOW_END_JD,
            step,
        )?;
        let jd = root.ok_or_else(|| past_window_end(step))?;
        self.station_at(&body, &reference, jd).map(Some)
    }

    /// The last station of `body` that has happened by `before`, or `None`.
    ///
    /// Early-terminating: the search walks backward from `before` and stops
    /// at the first station found. It finds the station
    /// `stations_in_range(body, reference, WINDOW_START, before).last()`
    /// finds and agrees with it on the instant to within the 0.5 s bisection
    /// tolerance; its scan is anchored at `before`, the range's at its
    /// start, so the two are not bit-identical.
    ///
    /// Whether a station has happened by `before` is read from the sign of
    /// the speed AT `before`, as
    /// [`previous_longitude_crossing`](Self::previous_longitude_crossing)
    /// reads its crossings. A returned [`Station::instant`] trails its
    /// station by less than the tolerance and never precedes it, so handing
    /// it back as `before` returns that same station, at an instant no later
    /// than `before`. To step back to the station before it, move `before`
    /// back by a second.
    ///
    /// The Sun, the Moon, the mean lunar points and, in the heliocentric
    /// frame, the planets never station and return `None` at once. Any other
    /// body that happens never to station is searched to the start of the
    /// 1900–2100 window first.
    ///
    /// Any other body is searched to the start of the 1900–2100 window; when
    /// the window starts first, the result is [`EventError::OutOfWindow`]
    /// naming the instant one step before it.
    ///
    /// The accuracy, step and errors are those of
    /// [`EventEngine::stations_in_range`].
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine, StationKind};
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// // Mercury's last station before J2000: it turned direct on 25 November 1999.
    /// let engine = EventEngine::new(packaged_backend());
    /// let before = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let station = engine
    ///     .previous_station(CelestialBody::Mercury, CrossingFrame::GeocentricApparentOfDate, before)
    ///     .unwrap()
    ///     .expect("Mercury stations six times a year");
    /// assert_eq!(station.kind, StationKind::TurnsDirect);
    /// let days = 2_451_545.0 - station.instant.julian_day.days();
    /// assert!((35.0..40.0).contains(&days), "{days}");
    /// ```
    pub fn previous_station(
        &self,
        body: CelestialBody,
        reference: impl Into<CrossingReference>,
        before: Instant,
    ) -> Result<Option<Station>, EventError> {
        let reference = reference.into();
        let before_jd = before.julian_day.days();
        self.check_window(before_jd)?;
        check_supported(&body, &reference, before_jd, "stations are")?;
        let step = step_days(&body);
        if never_stations(&body, reference.frame) {
            longitude_speed(&self.backend, &body, &reference, before_jd)?;
            return Ok(None);
        }
        let root = last_crossing_before(
            |jd| longitude_speed(&self.backend, &body, &reference, jd),
            WINDOW_START_JD,
            before_jd,
            step,
        )?;
        let jd = root.ok_or_else(|| before_window_start(step))?;
        self.station_at(&body, &reference, jd).map(Some)
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod window_edge_tests;
