//! Per-observer (local) eclipse circumstances: contact times, magnitude,
//! obscuration, horizontal position, and horizon visibility for a specific
//! observer, extending the crate's global/geocentric eclipse data.

// `TopoSunMoon`/`topo_sun_moon` are pub(crate) for upcoming eclipse-engine
// tasks; silence dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::ephemeris::{sample_sun_moon, SunMoonSample};
use crate::error::EclipseError;
use crate::types::{LunarEclipseType, SolarEclipseType};
use pleiades_apparent::{sidereal_time, topocentric_position, true_obliquity_degrees};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
};

/// One observer-local contact event: its instant plus the eclipsed body's
/// horizontal position and visibility there. A contact that occurs below the
/// horizon is still timed (`instant` present) but flagged `visible == false`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalContact {
    /// Instant of the contact (TDB).
    pub instant: Instant,
    /// Apparent (refracted) altitude of the eclipsed body, degrees.
    pub altitude_degrees: f64,
    /// Azimuth of the eclipsed body, measured from south increasing westward,
    /// `[0,360)` degrees (matches `swe_azalt`).
    pub azimuth_degrees: f64,
    /// Whether the body is above the horizon at this instant.
    pub visible: bool,
}

/// Local circumstances of a solar eclipse for one observer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalSolarCircumstances {
    /// What THIS observer sees (may differ from the global classification).
    pub local_type: SolarEclipseType,
    /// Instant of local greatest eclipse.
    pub maximum: LocalContact,
    /// Covered fraction of the Sun's diameter at local maximum.
    pub magnitude: f64,
    /// Covered fraction of the Sun's area at local maximum.
    pub obscuration: f64,
    /// First contact (C1): partial phase begins.
    pub first_contact: LocalContact,
    /// Second contact (C2): total/annular phase begins (central path only).
    pub second_contact: Option<LocalContact>,
    /// Third contact (C3): total/annular phase ends (central path only).
    pub third_contact: Option<LocalContact>,
    /// Fourth contact (C4): partial phase ends.
    pub fourth_contact: LocalContact,
    /// Whether the Sun is above the horizon during any part of the eclipse.
    pub any_phase_visible: bool,
}

/// Local circumstances of a lunar eclipse for one observer. Contact instants
/// are global (shared by all observers); the local content is horizon
/// visibility and the Moon's az/alt at each contact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalLunarCircumstances {
    /// Umbral/penumbral classification (identical to the global type).
    pub eclipse_type: LunarEclipseType,
    /// Local greatest eclipse.
    pub maximum: LocalContact,
    /// Umbral magnitude at greatest eclipse.
    pub umbral_magnitude: f64,
    /// Penumbral magnitude at greatest eclipse.
    pub penumbral_magnitude: f64,
    /// P1: penumbral phase begins.
    pub penumbral_begin: LocalContact,
    /// U1: partial (umbral) phase begins; `None` for penumbral-only eclipses.
    pub partial_begin: Option<LocalContact>,
    /// U2: total phase begins; `None` unless total.
    pub total_begin: Option<LocalContact>,
    /// U3: total phase ends; `None` unless total.
    pub total_end: Option<LocalContact>,
    /// U4: partial (umbral) phase ends; `None` for penumbral-only eclipses.
    pub partial_end: Option<LocalContact>,
    /// P4: penumbral phase ends.
    pub penumbral_end: LocalContact,
    /// Whether the Moon is above the horizon during any part of the eclipse.
    pub any_phase_visible: bool,
}

/// A tagged local result: either solar or lunar circumstances.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocalCircumstances {
    /// Solar eclipse local circumstances.
    Solar(LocalSolarCircumstances),
    /// Lunar eclipse local circumstances.
    Lunar(LocalLunarCircumstances),
}

/// Observer-relative (topocentric) Sun and Moon apparent ecliptic-of-date
/// geometry at one instant: the input to the two-circle solar contact geometry.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TopoSunMoon {
    pub sun_lon_deg: f64,
    pub sun_lat_deg: f64,
    pub sun_dist_au: f64,
    pub moon_lon_deg: f64,
    pub moon_lat_deg: f64,
    pub moon_dist_au: f64,
}

/// Applies diurnal parallax to the geocentric Sun/Moon sample for `observer`.
pub(crate) fn topo_sun_moon<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    jd: f64,
) -> Result<TopoSunMoon, EclipseError> {
    let sample: SunMoonSample = sample_sun_moon(backend, jd)?;
    let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let eps = true_obliquity_degrees(jd)
        .map_err(|e| EclipseError::Backend(format!("obliquity failed: {e}")))?;
    let lst = sidereal_time(at, observer.longitude).local_apparent_deg;

    let to_topo = |lon: f64, lat: f64, dist: f64| -> Result<(f64, f64, f64), EclipseError> {
        let ecl = EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(dist),
        );
        let topo = topocentric_position(ecl, observer, lst, eps)
            .map_err(|e| EclipseError::Backend(format!("topocentric failed: {e}")))?;
        Ok((
            topo.ecliptic.longitude.degrees(),
            topo.ecliptic.latitude.degrees(),
            topo.ecliptic.distance_au.unwrap_or(dist),
        ))
    };

    let (sun_lon_deg, sun_lat_deg, sun_dist_au) = to_topo(
        sample.sun_longitude_deg,
        sample.sun_latitude_deg,
        sample.sun_distance_au,
    )?;
    let (moon_lon_deg, moon_lat_deg, moon_dist_au) = to_topo(
        sample.moon_longitude_deg,
        sample.moon_latitude_deg,
        sample.moon_distance_au,
    )?;
    Ok(TopoSunMoon {
        sun_lon_deg,
        sun_lat_deg,
        sun_dist_au,
        moon_lon_deg,
        moon_lat_deg,
        moon_dist_au,
    })
}

#[cfg(test)]
mod topo_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn moon_parallax_shifts_topocentric_longitude() {
        // The analytic test backend places a new moon; an equatorial observer
        // sees the Moon shifted from its geocentric longitude by parallax.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let geo = sample_sun_moon(&backend, 2_451_550.0).unwrap();
        let topo = topo_sun_moon(&backend, &observer, 2_451_550.0).unwrap();
        let shift = (topo.moon_lon_deg - geo.moon_longitude_deg).abs();
        assert!(
            shift > 0.0,
            "expected a nonzero parallax shift, got {shift}"
        );
        assert!(topo.moon_dist_au.is_finite());
    }
}

/// Physical radii and unit conversion (mirrors `geometry::constants`; kept in
/// sync deliberately — do not diverge these values).
mod solar_consts {
    pub const R_SUN_KM: f64 = 696_000.0;
    pub const R_MOON_KM: f64 = 1_737.4;
    pub const AU_KM: f64 = 149_597_870.7;
}

/// Instantaneous topocentric two-circle geometry of a solar eclipse.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SolarGeom {
    /// Center-to-center Sun–Moon separation, degrees.
    pub sep_deg: f64,
    /// Sun's topocentric angular semidiameter, degrees.
    pub s_sun_deg: f64,
    /// Moon's topocentric angular semidiameter, degrees.
    pub s_moon_deg: f64,
}

/// Great-circle separation (degrees) between two ecliptic points.
fn angular_separation_deg(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let (l1, b1) = (lon1.to_radians(), lat1.to_radians());
    let (l2, b2) = (lon2.to_radians(), lat2.to_radians());
    let cos_sep = (b1.sin() * b2.sin() + b1.cos() * b2.cos() * (l1 - l2).cos()).clamp(-1.0, 1.0);
    cos_sep.acos().to_degrees()
}

/// Topocentric two-circle geometry at one instant.
pub(crate) fn solar_geom(t: &TopoSunMoon) -> SolarGeom {
    use solar_consts::*;
    let sep_deg =
        angular_separation_deg(t.sun_lon_deg, t.sun_lat_deg, t.moon_lon_deg, t.moon_lat_deg);
    let s_sun_deg = (R_SUN_KM / (t.sun_dist_au * AU_KM)).asin().to_degrees();
    let s_moon_deg = (R_MOON_KM / (t.moon_dist_au * AU_KM)).asin().to_degrees();
    SolarGeom {
        sep_deg,
        s_sun_deg,
        s_moon_deg,
    }
}

/// Covered fraction of the Sun's diameter (the eclipse "magnitude"), clamped ≥ 0.
pub(crate) fn covered_diameter_fraction(g: &SolarGeom) -> f64 {
    ((g.s_sun_deg + g.s_moon_deg - g.sep_deg) / (2.0 * g.s_sun_deg)).max(0.0)
}

/// Covered fraction of the Sun's disk AREA (obscuration), clamped to [0,1].
/// Standard two-circle lens area with radii `r_sun`, `r_moon` and center
/// distance `d` (all in the same angular units).
pub(crate) fn obscuration_fraction(g: &SolarGeom) -> f64 {
    let (r_s, r_m, d) = (g.s_sun_deg, g.s_moon_deg, g.sep_deg);
    if d >= r_s + r_m {
        return 0.0; // disjoint
    }
    if d <= r_m - r_s {
        return 1.0; // Sun fully covered (Moon disk envelops Sun disk)
    }
    if d <= (r_s - r_m).max(0.0) {
        // Moon fully inside the Sun (annular): area ratio (r_m/r_s)^2.
        return ((r_m / r_s).powi(2)).clamp(0.0, 1.0);
    }
    let r_s2 = r_s * r_s;
    let r_m2 = r_m * r_m;
    let a_s = ((d * d + r_s2 - r_m2) / (2.0 * d * r_s))
        .clamp(-1.0, 1.0)
        .acos();
    let a_m = ((d * d + r_m2 - r_s2) / (2.0 * d * r_m))
        .clamp(-1.0, 1.0)
        .acos();
    let lens_area = r_s2 * a_s + r_m2 * a_m
        - 0.5
            * ((r_s + r_m + d) * (-r_s + r_m + d) * (r_s - r_m + d) * (r_s + r_m - d))
                .max(0.0)
                .sqrt();
    (lens_area / (core::f64::consts::PI * r_s2)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod solar_geom_tests {
    use super::*;

    fn geom(sep: f64, s_sun: f64, s_moon: f64) -> SolarGeom {
        SolarGeom {
            sep_deg: sep,
            s_sun_deg: s_sun,
            s_moon_deg: s_moon,
        }
    }

    #[test]
    fn magnitude_is_one_when_centers_coincide_and_moon_larger() {
        let g = geom(0.0, 0.26, 0.28);
        let mag = covered_diameter_fraction(&g);
        assert!(mag >= 1.0, "central total magnitude {mag}");
    }

    #[test]
    fn magnitude_zero_outside_contact() {
        let g = geom(1.0, 0.26, 0.26); // sep > s_sun + s_moon
        assert_eq!(covered_diameter_fraction(&g), 0.0);
    }

    #[test]
    fn obscuration_full_when_sun_fully_covered() {
        let g = geom(0.0, 0.26, 0.30);
        assert!((obscuration_fraction(&g) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn obscuration_zero_when_disjoint() {
        let g = geom(1.0, 0.26, 0.26);
        assert_eq!(obscuration_fraction(&g), 0.0);
    }

    #[test]
    fn obscuration_between_zero_and_one_partial() {
        let g = geom(0.30, 0.26, 0.26);
        let o = obscuration_fraction(&g);
        assert!(o > 0.0 && o < 1.0, "partial obscuration {o}");
    }

    #[test]
    fn annular_obscuration_is_area_ratio() {
        // Moon fully inside Sun (annular): d + r_m <= r_s.
        let g = geom(0.0, 0.28, 0.26);
        let o = obscuration_fraction(&g);
        let expected = (0.26_f64 / 0.28).powi(2);
        assert!(
            (o - expected).abs() < 1e-6,
            "annular obscuration {o} vs {expected}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{JulianDay, TimeScale};

    fn contact(jd: f64) -> LocalContact {
        LocalContact {
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            altitude_degrees: 30.0,
            azimuth_degrees: 180.0,
            visible: true,
        }
    }

    #[test]
    fn local_circumstances_tags_solar_and_lunar() {
        let solar = LocalCircumstances::Solar(LocalSolarCircumstances {
            local_type: SolarEclipseType::Partial,
            maximum: contact(2_451_545.0),
            magnitude: 0.5,
            obscuration: 0.4,
            first_contact: contact(2_451_544.9),
            second_contact: None,
            third_contact: None,
            fourth_contact: contact(2_451_545.1),
            any_phase_visible: true,
        });
        assert!(matches!(solar, LocalCircumstances::Solar(_)));
    }
}
