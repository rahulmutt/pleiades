//! Station finder: the instants a body's longitude speed changes sign.
//!
//! A station is a sign change of the longitude speed
//! [`EventEngine::position_at`] reports, so it is exactly where the engine's
//! own direction flips, in every frame and zodiac.

use crate::crossings::{body_label, CrossingFrame, EventEngine};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::position::place_and_motion;
use crate::reference::{check_supported, CrossingReference};
use crate::root::{crossings_in_range, first_crossing_after};
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
    /// list. An empty or inverted range returns an empty list, but the scan
    /// still samples the speed at its start, so a body the backend cannot
    /// serve, or one with no longitude speed, still returns its error.
    ///
    /// As for the crossings, the scan is clamped one step inside each end of
    /// the 1900–2100 window so that its bracketing samples stay in the
    /// window: a station within one step of either window end is not
    /// reported.
    ///
    /// # Accuracy
    ///
    /// The search steps by 0.25 day for the Moon and the lunar points, 1 day
    /// for the Sun, Mercury and Venus, and 2 days otherwise. Two stations
    /// closer together than the step are not reported. That happens only
    /// for the osculating lunar points: the true node's speed touches zero
    /// about every two weeks, and whether a touch crosses zero for a few
    /// hours depends on the ephemeris.
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
        // Clamp like the crossings: keep the bracketing samples in-window.
        let scan_start = start_jd.max(WINDOW_START_JD + step);
        let scan_end = end_jd.min(WINDOW_END_JD - step);
        let roots = crossings_in_range(
            |jd| longitude_speed(&self.backend, &body, &reference, jd),
            scan_start,
            scan_end,
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
    /// For a body that never stations the search runs to the end of the
    /// 1900–2100 window before returning `None`.
    ///
    /// The window-edge clamp (a station within one step of either end of
    /// the window is not reported), accuracy, step and errors are those of
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
        // Same clamps as `stations_in_range` over `[after, WINDOW_END]`.
        let scan_start = after_jd.max(WINDOW_START_JD + step);
        let scan_end = WINDOW_END_JD - step;
        let root = first_crossing_after(
            |jd| longitude_speed(&self.backend, &body, &reference, jd),
            scan_start,
            scan_end,
            step,
        )?;
        root.filter(|&jd| jd > after_jd)
            .map(|jd| self.station_at(&body, &reference, jd))
            .transpose()
    }
}

#[cfg(test)]
mod tests;
