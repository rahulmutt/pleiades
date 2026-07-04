//! Rise, set, and meridian-transit finding (`swe_rise_trans`, full-flag).

// `effective()` and some variants here are not yet consumed by an engine
// method; those land in Tasks 9-13. Silence the dead_code lint until then.
#![allow(dead_code)]

use crate::crossings::EventEngine;
use crate::ephemeris::geocentric_apparent_ecliptic;
use crate::error::EventError;
use crate::fixstar::fixed_star_apparent;
use pleiades_apparent::{sidereal_time, topocentric_position, true_obliquity_degrees};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    Angle, CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, TimeScale,
};

/// Which observer-local event to find.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiseSetEvent {
    /// Body crosses the horizon upward.
    Rise,
    /// Body crosses the horizon downward.
    Set,
    /// Upper (meridian) transit — hour angle 0.
    UpperTransit,
    /// Lower transit — hour angle ±12ʰ.
    LowerTransit,
}

/// Which point of the disc defines the event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscMode {
    /// Disc center.
    Center,
    /// Upper limb (SE default for rise/set).
    UpperLimb,
    /// Lower limb (`SE_BIT_DISC_BOTTOM`).
    LowerLimb,
}

/// The object whose event is sought.
#[derive(Clone, Debug)]
pub enum RiseSetTarget {
    /// A release-grade body.
    Body(CelestialBody),
    /// An arbitrary ecliptic point (longitude, latitude); pair with `no_ecl_lat`
    /// to force latitude 0 (rising of a zodiac degree).
    EclipticPoint(Longitude, Latitude),
    /// A curated fixed star by name.
    FixedStar(String),
}

/// `swe_rise_trans` flag bundle.
#[derive(Clone, Debug)]
pub struct RiseSetOptions {
    /// Disc convention.
    pub disc: DiscMode,
    /// Apply atmospheric refraction (`false` = `SE_BIT_NO_REFRACTION`).
    pub refraction: bool,
    /// Force ecliptic latitude 0 (`SE_BIT_GEOCTR_NO_ECL_LAT`).
    pub no_ecl_lat: bool,
    /// Hindu rising = `DISC_CENTER | NO_REFRACTION | GEOCTR_NO_ECL_LAT`.
    pub hindu: bool,
    /// Freeze semidiameter at mean distance (`SE_BIT_FIXED_DISC_SIZE`).
    pub fixed_disc_size: bool,
    /// Custom local horizon altitude, degrees (`swe_rise_trans_true_hor`).
    pub horizon_altitude_deg: Option<f64>,
}

impl Default for RiseSetOptions {
    fn default() -> Self {
        Self {
            disc: DiscMode::UpperLimb,
            refraction: true,
            no_ecl_lat: false,
            hindu: false,
            fixed_disc_size: false,
            horizon_altitude_deg: None,
        }
    }
}

impl RiseSetOptions {
    /// Resolves `hindu` into its component flags (SE composition).
    pub(crate) fn effective(&self) -> Self {
        if self.hindu {
            Self {
                disc: DiscMode::Center,
                refraction: false,
                no_ecl_lat: true,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }
}

/// A located rise/set/transit event (TDB).
#[derive(Clone, Debug)]
pub struct RiseSet {
    /// Which event this is.
    pub event: RiseSetEvent,
    /// Instant of the event (TDB).
    pub instant: Instant,
    /// The target the event is for.
    pub target: RiseSetTarget,
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Topocentric right ascension / declination (degrees, apparent-of-date) of
    /// `target` for `observer` at `jd` (TDB Julian Day).
    ///
    /// - `FixedStar`: the curated catalog's apparent equatorial place (already
    ///   geocentric to the precision the catalog supports; no topocentric
    ///   correction is applied since stars have no meaningful parallax here).
    /// - `EclipticPoint`: a pure geocentric ecliptic → equatorial rotation using
    ///   the true obliquity of date; `opts.no_ecl_lat` forces latitude to 0.
    /// - `Body`: the geocentric apparent ecliptic position (from
    ///   `geocentric_apparent_ecliptic`), with `no_ecl_lat` applied, then
    ///   diurnal parallax + diurnal aberration via `topocentric_position`
    ///   before rotating to equatorial.
    pub(crate) fn target_equatorial(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        jd: f64,
    ) -> Result<(f64, f64), EventError> {
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let eps = true_obliquity_degrees(jd)
            .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        match target {
            RiseSetTarget::FixedStar(name) => {
                let equ = fixed_star_apparent(name, at)?;
                Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
            }
            RiseSetTarget::EclipticPoint(lon, lat) => {
                let lat = if opts.no_ecl_lat {
                    Latitude::from_degrees(0.0)
                } else {
                    *lat
                };
                let equ = EclipticCoordinates::new(*lon, lat, None)
                    .to_equatorial(Angle::from_degrees(eps));
                Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
            }
            RiseSetTarget::Body(b) => {
                let (lon, lat, dist) =
                    geocentric_apparent_ecliptic(&self.backend, b.clone(), "body", jd)?;
                let lat = if opts.no_ecl_lat { 0.0 } else { lat };
                let ecl = EclipticCoordinates::new(
                    Longitude::from_degrees(lon),
                    Latitude::from_degrees(lat),
                    Some(dist),
                );
                let topo = topocentric_position(ecl, observer, lst, eps)
                    .map_err(|e| EventError::Backend(format!("topocentric failed: {e}")))?;
                let equ = topo.ecliptic.to_equatorial(Angle::from_degrees(eps));
                Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_are_upper_limb_refracted() {
        let o = RiseSetOptions::default();
        assert!(matches!(o.disc, DiscMode::UpperLimb));
        assert!(o.refraction);
        assert!(!o.hindu && !o.no_ecl_lat && !o.fixed_disc_size);
        assert!(o.horizon_altitude_deg.is_none());
    }

    #[test]
    fn target_equatorial_matches_horizontal_for_a_star() {
        use pleiades_backend::test_backend::LinearSunMoon;
        use pleiades_types::{
            Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
        };
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let obs = ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(0.0),
            None,
        );
        let jd = 2_451_545.0;
        let (ra, dec) = engine
            .target_equatorial(
                &RiseSetTarget::FixedStar("Aldebaran".into()),
                &obs,
                &RiseSetOptions::default(),
                jd,
            )
            .unwrap();
        let equ = crate::fixstar::fixed_star_apparent(
            "Aldebaran",
            Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        )
        .unwrap();
        assert!((ra - equ.right_ascension.degrees()).abs() < 1e-9);
        assert!((dec - equ.declination.degrees()).abs() < 1e-9);
    }

    #[test]
    fn ecliptic_point_no_ecl_lat_forces_latitude_zero() {
        use pleiades_backend::test_backend::LinearSunMoon;
        use pleiades_types::{Latitude, Longitude, ObserverLocation};
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let obs = ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(0.0),
            None,
        );
        let opts = RiseSetOptions {
            no_ecl_lat: true,
            ..RiseSetOptions::default()
        };
        // A point at longitude 90° latitude 30° with no_ecl_lat behaves like latitude 0.
        let (_, dec_forced) = engine
            .target_equatorial(
                &RiseSetTarget::EclipticPoint(
                    Longitude::from_degrees(90.0),
                    Latitude::from_degrees(30.0),
                ),
                &obs,
                &opts,
                2_451_545.0,
            )
            .unwrap();
        let (_, dec_zero) = engine
            .target_equatorial(
                &RiseSetTarget::EclipticPoint(
                    Longitude::from_degrees(90.0),
                    Latitude::from_degrees(0.0),
                ),
                &obs,
                &opts,
                2_451_545.0,
            )
            .unwrap();
        assert!(
            (dec_forced - dec_zero).abs() < 1e-9,
            "no_ecl_lat should ignore supplied latitude"
        );
    }
}
