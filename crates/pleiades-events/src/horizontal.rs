//! Horizontal coordinates (`swe_azalt` / `swe_azalt_rev`): azimuth and altitude
//! of a target for a topocentric observer. Azimuth is measured from SOUTH,
//! increasing WESTWARD, degrees `[0,360)`, matching Swiss Ephemeris.

use crate::crossings::EventEngine;
use crate::error::EventError;
use crate::rise_trans::check_atmosphere;
use crate::time_scale::{local_apparent_sidereal_deg, tdb_jd};
use pleiades_apparent::{apparent_from_true, true_obliquity_degrees, Atmosphere};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{Angle, EclipticCoordinates, Instant, Latitude, Longitude, ObserverLocation};

/// Azimuth (from south, westward) plus true (geometric) and apparent (refracted)
/// altitude, all degrees.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Horizontal {
    /// Azimuth measured from south, increasing westward, `[0,360)` degrees.
    pub azimuth: f64,
    /// Geometric (unrefracted) altitude, degrees.
    pub true_altitude: f64,
    /// Apparent (refracted) altitude, degrees.
    pub apparent_altitude: f64,
}

/// Input coordinate for [`EventEngine::horizontal`].
#[derive(Clone, Copy, Debug)]
pub enum HorizontalInput {
    /// Tropical apparent ecliptic of date (`SE_ECL2HOR`): longitude, latitude.
    Ecliptic(Longitude, Latitude),
    /// Apparent equatorial of date (`SE_EQU2HOR`): right ascension, declination.
    Equatorial(Angle, Latitude),
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Azimuth/altitude of `input` for `observer` at `at`.
    ///
    /// `at` is read by its `TimeScale` tag (`Tdb`/`Tt` as TDB, `Ut1`/`Utc` with
    /// ΔT added; other scales fail closed with
    /// [`EventError::UnsupportedTimeScale`]), and the sky is rotated with local
    /// apparent sidereal time at the UT1 re-expression of that instant. The
    /// input coordinates themselves are taken as given (apparent of date).
    pub fn horizontal(
        &self,
        input: HorizontalInput,
        observer: ObserverLocation,
        atmos: Atmosphere,
        at: Instant,
    ) -> Result<Horizontal, EventError> {
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmos)?;
        let jd = tdb_jd(at)?;
        // Resolve to apparent equatorial RA/Dec (degrees).
        let (ra_deg, dec_deg) = match input {
            HorizontalInput::Equatorial(ra, dec) => (ra.degrees(), dec.degrees()),
            HorizontalInput::Ecliptic(lon, lat) => {
                let eps = true_obliquity_degrees(jd)
                    .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
                let ecl = EclipticCoordinates::new(lon, lat, None);
                let equ = ecl.to_equatorial(Angle::from_degrees(eps));
                (equ.right_ascension.degrees(), equ.declination.degrees())
            }
        };
        // Local apparent sidereal time (UT1 rotation) → local hour angle H = LST − RA.
        let lst = local_apparent_sidereal_deg(jd, observer.longitude)?;
        let h_deg = lst - ra_deg;
        let (h, dec, phi) = (
            h_deg.to_radians(),
            dec_deg.to_radians(),
            observer.latitude.degrees().to_radians(),
        );
        // Standard equatorial → horizontal rotation (azimuth from south, west +).
        let sin_alt = phi.sin() * dec.sin() + phi.cos() * dec.cos() * h.cos();
        let alt = sin_alt.clamp(-1.0, 1.0).asin();
        let az = (h.sin()).atan2(h.cos() * phi.sin() - dec.tan() * phi.cos());
        let true_altitude = alt.to_degrees();
        Ok(Horizontal {
            azimuth: az.to_degrees().rem_euclid(360.0),
            true_altitude,
            apparent_altitude: apparent_from_true(
                true_altitude,
                atmos.at_elevation(observer.elevation_m.unwrap_or(0.0)),
            ),
        })
    }
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Inverse of [`EventEngine::horizontal`] (`swe_azalt_rev`): horizontal →
    /// apparent equatorial of date. When `is_apparent` is true the altitude is
    /// de-refracted first. `at` is read by its `TimeScale` tag exactly as in
    /// [`EventEngine::horizontal`].
    pub fn horizontal_to_equatorial(
        &self,
        azimuth_deg: f64,
        altitude_deg: f64,
        is_apparent: bool,
        observer: ObserverLocation,
        atmos: Atmosphere,
        at: Instant,
    ) -> Result<(Angle, Latitude), EventError> {
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmos)?;
        let alt_deg = if is_apparent {
            pleiades_apparent::true_from_apparent(
                altitude_deg,
                atmos.at_elevation(observer.elevation_m.unwrap_or(0.0)),
            )
        } else {
            altitude_deg
        };
        let (az, alt, phi) = (
            azimuth_deg.to_radians(),
            alt_deg.to_radians(),
            observer.latitude.degrees().to_radians(),
        );
        let sin_dec = phi.sin() * alt.sin() - phi.cos() * alt.cos() * az.cos();
        let dec = sin_dec.clamp(-1.0, 1.0).asin();
        let h = (az.sin()).atan2(phi.sin() * az.cos() + phi.cos() * alt.tan());
        let lst = local_apparent_sidereal_deg(tdb_jd(at)?, observer.longitude)?;
        let ra = (lst - h.to_degrees()).rem_euclid(360.0);
        Ok((
            Angle::from_degrees(ra),
            Latitude::from_degrees(dec.to_degrees()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_apparent::Atmosphere;
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{
        Angle, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    };

    fn tdb(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }
    /// Local apparent sidereal time at Greenwich for a TDB instant, evaluated
    /// the way the engine does: at its UT1 re-expression.
    fn greenwich_last_deg(at: Instant) -> f64 {
        let ut1 = pleiades_apparent::ut1_instant(at).unwrap();
        pleiades_apparent::sidereal_time(ut1, Longitude::from_degrees(0.0)).local_apparent_deg
    }
    fn greenwich() -> ObserverLocation {
        ObserverLocation::new(
            Latitude::from_degrees(51.48),
            Longitude::from_degrees(0.0),
            None,
        )
    }

    #[test]
    fn object_on_local_meridian_has_azimuth_zero_or_180() {
        // A body whose RA equals the local apparent sidereal time is on the meridian:
        // hour angle 0 → azimuth 0 (south) if it is south of zenith.
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let at = tdb(2_451_545.0);
        let ra = Angle::from_degrees(greenwich_last_deg(at));
        let dec = Latitude::from_degrees(10.0); // south of a 51°N observer's zenith
        let h = engine
            .horizontal(
                HorizontalInput::Equatorial(ra, dec),
                greenwich(),
                Atmosphere::default(),
                at,
            )
            .unwrap();
        assert!(
            h.azimuth.abs() < 1e-3 || (h.azimuth - 360.0).abs() < 1e-3,
            "az {}",
            h.azimuth
        );
        assert!(
            h.apparent_altitude >= h.true_altitude,
            "refraction lifts the body"
        );
    }

    /// A zero pressure is SE's `atpress = 0`: estimated from the observer's
    /// elevation (1013.25 mbar at sea level, 898.707 mbar at 1000 m), not a
    /// vacuum (issue #242).
    #[test]
    fn zero_pressure_is_estimated_from_the_observer_elevation() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let at = tdb(2_451_545.0);
        let input = HorizontalInput::Equatorial(
            Angle::from_degrees(greenwich_last_deg(at) + 80.0),
            Latitude::from_degrees(0.0),
        );
        let apparent = |elevation_m, pressure_mbar| {
            let observer = ObserverLocation::new(
                Latitude::from_degrees(51.48),
                Longitude::from_degrees(0.0),
                Some(elevation_m),
            );
            let atmos = Atmosphere {
                pressure_mbar,
                temperature_c: 0.0,
            };
            let h = engine.horizontal(input, observer, atmos, at).unwrap();
            (h.apparent_altitude, h.true_altitude)
        };
        let (zero, true_alt) = apparent(0.0, 0.0);
        assert!(zero > true_alt + 0.1, "refracted: {zero} vs {true_alt}");
        assert_eq!(zero, apparent(0.0, 1013.25).0);
        let (high, _) = apparent(1000.0, 0.0);
        assert!((high - apparent(1000.0, 898.707).0).abs() < 1e-6);
        assert!(high < zero, "thinner air refracts less");
    }

    #[test]
    fn target_at_zenith_altitude_is_finite_not_nan() {
        // Declination equals the observer's latitude and hour angle ~0 (RA ==
        // local apparent sidereal time) puts the target exactly at zenith.
        // Floating-point rounding can push sin_alt fractionally outside
        // [-1,1] at this boundary, so asin must be fed a clamped value or it
        // silently returns NaN — a fail-closed violation. This exact
        // (jd, latitude) pair was found by scanning for the boundary case
        // where sin(phi)^2 + cos(phi)^2 rounds to a hair above 1.0.
        let observer = ObserverLocation::new(
            Latitude::from_degrees(-87.5),
            Longitude::from_degrees(0.0),
            None,
        );
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let at = tdb(2_451_000.0);
        let ra = Angle::from_degrees(greenwich_last_deg(at));
        let dec = Latitude::from_degrees(-87.5); // == observer's latitude
        let h = engine
            .horizontal(
                HorizontalInput::Equatorial(ra, dec),
                observer,
                Atmosphere::default(),
                at,
            )
            .unwrap();
        assert!(
            h.true_altitude.is_finite(),
            "altitude must never be NaN, got {}",
            h.true_altitude
        );
        assert!(
            (h.true_altitude - 90.0).abs() < 1e-6,
            "expected zenith (~90°), got {}",
            h.true_altitude
        );
    }

    #[test]
    fn altitude_never_exceeds_ninety() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let at = tdb(2_451_545.0);
        let h = engine
            .horizontal(
                HorizontalInput::Equatorial(
                    Angle::from_degrees(0.0),
                    Latitude::from_degrees(51.48),
                ),
                greenwich(),
                Atmosphere::default(),
                at,
            )
            .unwrap();
        assert!(h.true_altitude <= 90.0 + 1e-9);
    }

    #[test]
    fn azalt_round_trips_through_equatorial() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let at = tdb(2_451_545.0);
        let ra_in = Angle::from_degrees(123.0);
        let dec_in = Latitude::from_degrees(17.0);
        let h = engine
            .horizontal(
                HorizontalInput::Equatorial(ra_in, dec_in),
                greenwich(),
                Atmosphere::default(),
                at,
            )
            .unwrap();
        // Feed the TRUE altitude back (is_apparent = false) to invert the pure rotation.
        let (ra, dec) = engine
            .horizontal_to_equatorial(
                h.azimuth,
                h.true_altitude,
                false,
                greenwich(),
                Atmosphere::default(),
                at,
            )
            .unwrap();
        let dra = crate::root::wrap180(ra.degrees() - 123.0);
        assert!(dra.abs() < 1e-6, "ra back {}", ra.degrees());
        assert!(
            (dec.degrees() - 17.0).abs() < 1e-6,
            "dec back {}",
            dec.degrees()
        );
    }
}
