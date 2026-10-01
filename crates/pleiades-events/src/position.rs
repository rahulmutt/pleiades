//! Ecliptic position and speed of a body in a [`CrossingFrame`].
//!
//! Both frames follow one pattern. A *base* place has a speed known from the
//! backend; the reported place is that base place moved by a small, smooth
//! correction; and the reported speed is the base speed plus the rate of the
//! correction, differenced over ±[`HALF_SPAN_DAYS`].
//!
//! | Frame | Base place and speed | Corrected place |
//! |---|---|---|
//! | geocentric apparent of date | backend mean J2000 place and its motion | apparent place of date |
//! | geocentric mean of date | backend mean J2000 place and its motion | precessed to the mean equinox of date |
//! | heliocentric | planet minus Sun in J2000, rates from Cartesian velocities | true equinox of date |
//!
//! The heliocentric place is geometric — no light-time, no aberration — in the
//! true ecliptic and equinox of date, i.e. Swiss Ephemeris
//! `SEFLG_HELCTR | SEFLG_TRUEPOS`. Plain `SEFLG_HELCTR` output is retarded by
//! the heliocentric light-time and differs from it by up to ≈ 41″ (Mercury).

use crate::crossings::{body_label, CrossingFrame, EventEngine};
use crate::ephemeris::{
    geocentric_apparent_ecliptic, geocentric_mean_of_date_ecliptic, heliocentric_j2000,
    heliocentric_of_date, j2000_spherical, read_mean_ecliptic_with_motion,
};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::state_vector::spherical_rates;
use pleiades_apparent::motion::{apparent_motion, Correction, CorrectionSample, HALF_SPAN_DAYS};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{CelestialBody, EclipticCoordinates, Instant, Latitude, Longitude, Motion};

/// Ecliptic position and speed of a body at one instant.
//
// `CelestialBody` is not `Copy`, so neither is this.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct EclipticPosition {
    /// The body.
    pub body: CelestialBody,
    /// The frame `ecliptic` and `motion` are expressed in.
    pub frame: CrossingFrame,
    /// The instant as given; its Julian day is read as TDB.
    pub instant: Instant,
    /// Ecliptic longitude and latitude (degrees) and distance (AU, always
    /// `Some`): from the Earth's centre for the geocentric frame, from the Sun
    /// for the heliocentric frame.
    pub ecliptic: EclipticCoordinates,
    /// Speed of `ecliptic`: longitude and latitude in degrees per day,
    /// distance in AU per day. A channel is `None` when the backend reports no
    /// speed to derive it from.
    pub motion: Motion,
}

const NO_MOTION: Motion = Motion::new(None, None, None);

/// The base and corrected place of a body at one instant, with the base speed.
struct Sample {
    base: EclipticCoordinates,
    corrected: EclipticCoordinates,
    base_motion: Option<Motion>,
}

impl Sample {
    fn correction_at(&self, julian_day: f64) -> CorrectionSample {
        CorrectionSample {
            julian_day,
            correction: Correction::between(&self.corrected, &self.base),
        }
    }
}

fn coordinates((lon_deg, lat_deg, distance_au): (f64, f64, f64)) -> EclipticCoordinates {
    EclipticCoordinates::new(
        Longitude::from_degrees(lon_deg),
        Latitude::from_degrees(lat_deg),
        Some(distance_au),
    )
}

fn sample<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    frame: CrossingFrame,
    julian_day: f64,
) -> Result<Sample, EventError> {
    let label = body_label(body);
    match frame {
        CrossingFrame::GeocentricApparentOfDate => {
            let (mean, base_motion) =
                read_mean_ecliptic_with_motion(backend, body.clone(), label, julian_day)?;
            let apparent = geocentric_apparent_ecliptic(backend, body.clone(), label, julian_day)?;
            Ok(Sample {
                base: coordinates(mean),
                corrected: coordinates(apparent),
                base_motion,
            })
        }
        CrossingFrame::GeocentricMeanOfDate => {
            let (mean, base_motion) =
                read_mean_ecliptic_with_motion(backend, body.clone(), label, julian_day)?;
            let of_date =
                geocentric_mean_of_date_ecliptic(backend, body.clone(), label, julian_day)?;
            Ok(Sample {
                base: coordinates(mean),
                corrected: coordinates(of_date),
                base_motion,
            })
        }
        CrossingFrame::Heliocentric => {
            let helio = heliocentric_j2000(backend, body.clone(), label, julian_day)?;
            let of_date = heliocentric_of_date(helio.position, julian_day)?;
            Ok(Sample {
                base: coordinates(j2000_spherical(helio.position)),
                corrected: coordinates(of_date),
                base_motion: helio
                    .velocity
                    .map(|velocity| spherical_rates(helio.position, velocity)),
            })
        }
    }
}

/// Speed of the corrected place at `julian_day`, whose sample is `centre`.
///
/// The correction is differenced centrally over ±[`HALF_SPAN_DAYS`]. A
/// neighbouring instant outside the engine's window, or one the backend cannot
/// serve, is dropped and the difference becomes one-sided. With no base speed,
/// or with neither neighbour, the speed is unknown: every channel is `None`
/// rather than a speed that describes another place.
fn motion<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    frame: CrossingFrame,
    julian_day: f64,
    centre: &Sample,
) -> Motion {
    let Some(base) = centre.base_motion else {
        return NO_MOTION;
    };
    let neighbour = |jd: f64| {
        if !(WINDOW_START_JD..=WINDOW_END_JD).contains(&jd) {
            return None;
        }
        sample(backend, body, frame, jd)
            .ok()
            .map(|sample| sample.correction_at(jd))
    };
    let centre = centre.correction_at(julian_day);
    let (earlier, later) = match (
        neighbour(julian_day - HALF_SPAN_DAYS),
        neighbour(julian_day + HALF_SPAN_DAYS),
    ) {
        (Some(earlier), Some(later)) => (earlier, later),
        (Some(earlier), None) => (earlier, centre),
        (None, Some(later)) => (centre, later),
        (None, None) => return NO_MOTION,
    };
    apparent_motion(base, &earlier, &later)
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Ecliptic position and speed of `body` in `frame` at `instant` (TDB).
    ///
    /// The longitude is exactly the one [`EventEngine::longitude_at`] returns,
    /// so a position read here is consistent with the crossings the engine
    /// finds in the same frame.
    ///
    /// - [`CrossingFrame::GeocentricApparentOfDate`]: the apparent place
    ///   (light-time with annual aberration, precession, nutation) in the true
    ///   ecliptic of date, from the Earth's centre. The speed is the one a
    ///   `pleiades-core` apparent chart reports.
    /// - [`CrossingFrame::GeocentricMeanOfDate`]: the geometric place from the
    ///   Earth's centre (no light-time, no aberration, no nutation) in the
    ///   mean ecliptic and equinox of date.
    /// - [`CrossingFrame::Heliocentric`]: the geometric place from the Sun (no
    ///   light-time, no aberration) in the true ecliptic and equinox of date,
    ///   i.e. Swiss Ephemeris `SEFLG_HELCTR | SEFLG_TRUEPOS`. Plain
    ///   `SEFLG_HELCTR` output is retarded by the heliocentric light-time and
    ///   differs from this place by up to ≈ 41″ (Mercury).
    ///
    /// Longitude and latitude are degrees, distance is AU; speeds are per day.
    /// The speed is the backend's own speed plus the rate of the frame or
    /// apparent-place correction, differenced over ±0.5 day. The difference is
    /// one-sided at the edges of the window and, more generally, whenever a
    /// neighbouring instant cannot be served by the backend. A speed channel is
    /// `None` when the backend reports no speed to derive it from.
    ///
    /// # Errors
    ///
    /// The same as [`EventEngine::longitude_at`]:
    /// [`EventError::OutOfWindow`] outside the packaged 1900–2100 window,
    /// [`EventError::UnsupportedFrame`] for a heliocentric Sun or Moon,
    /// [`EventError::MissingCoordinates`] when the backend returns no ecliptic
    /// place or no distance, and [`EventError::Backend`] for a backend failure.
    /// In the geocentric frame an instant within a light-time of the start of
    /// the packaged range can fail with [`EventError::Backend`], exactly as
    /// `longitude_at` does there, because the light-time re-query leaves the
    /// backend's range (for example Mars or the Moon at JD 2415020.5).
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine};
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// let engine = EventEngine::new(packaged_backend());
    /// let t = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    ///
    /// let helio = engine
    ///     .position_at(CelestialBody::Mars, CrossingFrame::Heliocentric, t)
    ///     .unwrap();
    /// // Mars is 1.38–1.67 AU from the Sun and always moves direct around it.
    /// assert!((1.38..1.67).contains(&helio.ecliptic.distance_au.unwrap()));
    /// assert!(helio.motion.longitude_deg_per_day.unwrap() > 0.4);
    ///
    /// let geo = engine
    ///     .position_at(CelestialBody::Mars, CrossingFrame::GeocentricApparentOfDate, t)
    ///     .unwrap();
    /// let lon = engine
    ///     .longitude_at(CelestialBody::Mars, CrossingFrame::GeocentricApparentOfDate, t)
    ///     .unwrap();
    /// assert_eq!(geo.ecliptic.longitude, lon);
    /// ```
    pub fn position_at(
        &self,
        body: CelestialBody,
        frame: CrossingFrame,
        instant: Instant,
    ) -> Result<EclipticPosition, EventError> {
        let jd = instant.julian_day.days();
        self.check_window(jd)?;
        if matches!(frame, CrossingFrame::Heliocentric)
            && matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        {
            return Err(EventError::UnsupportedFrame {
                detail: format!("heliocentric position is undefined for {:?}", body),
            });
        }
        let centre = sample(&self.backend, &body, frame, jd)?;
        let motion = motion(&self.backend, &body, frame, jd, &centre);
        Ok(EclipticPosition {
            body,
            frame,
            instant,
            ecliptic: centre.corrected,
            motion,
        })
    }
}

#[cfg(test)]
mod tests;
