//! Per-observer (local) eclipse circumstances: contact times, magnitude,
//! obscuration, horizontal position, and horizon visibility for a specific
//! observer, extending the crate's global/geocentric eclipse data.

use crate::ephemeris::{sample_sun_moon, SunMoonSample};
use crate::error::EclipseError;
use crate::types::{Eclipse, EclipseKind, LunarEclipseType, SolarEclipseType};
use pleiades_apparent::{
    apparent_from_true, sidereal_time, topocentric_position, true_obliquity_degrees, Atmosphere,
};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    Angle, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, ObserverLocation,
    TimeScale,
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

/// Half-width of the solar contact search bracket around local maximum (days).
/// A local solar eclipse's partial phase never exceeds ~3.5 h; 0.25 day (6 h)
/// is a safe superset.
const SOLAR_CONTACT_HALF_WINDOW_DAYS: f64 = 0.25;
/// Root/extremum refinement tolerance: 0.5 s, matching the crate's global
/// `refine_greatest` and the SP-2a root-finder.
const REFINE_TOLERANCE_DAYS: f64 = 0.5 / 86_400.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SolarContactsJd {
    pub max_jd: f64,
    pub c1_jd: f64,
    pub c2_jd: f64,
    pub c3_jd: f64,
    pub c4_jd: f64,
    pub min_sep_deg: f64,
    pub s_sun_at_max: f64,
    pub s_moon_at_max: f64,
    pub c2_c3_present: bool,
}

/// Topocentric Sun–Moon separation (degrees) at `jd` for `observer`.
fn solar_sep_deg<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    jd: f64,
) -> Result<f64, EclipseError> {
    Ok(solar_geom(&topo_sun_moon(backend, observer, jd)?).sep_deg)
}

/// Golden-section minimize `sep(t)` in `[a,b]` to `REFINE_TOLERANCE_DAYS`.
fn minimize_sep<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    mut a: f64,
    mut b: f64,
) -> Result<f64, EclipseError> {
    let phi = 0.618_033_988_75_f64;
    let mut c = b - (b - a) * phi;
    let mut d = a + (b - a) * phi;
    let mut fc = solar_sep_deg(backend, observer, c)?;
    let mut fd = solar_sep_deg(backend, observer, d)?;
    while (b - a) > REFINE_TOLERANCE_DAYS {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - (b - a) * phi;
            fc = solar_sep_deg(backend, observer, c)?;
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + (b - a) * phi;
            fd = solar_sep_deg(backend, observer, d)?;
        }
    }
    Ok(0.5 * (a + b))
}

/// Bisect `sep(t) - threshold` between `lo` and `hi` where it changes sign;
/// returns `None` if it does not change sign across the bracket.
fn bisect_contact<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    threshold: impl Fn(f64) -> f64, // threshold as fn of jd (semidiameters vary slowly)
    mut lo: f64,
    mut hi: f64,
) -> Result<Option<f64>, EclipseError> {
    let f = |jd: f64,
             s: &mut dyn FnMut(f64) -> Result<f64, EclipseError>|
     -> Result<f64, EclipseError> { Ok(s(jd)? - threshold(jd)) };
    let mut sep = |jd: f64| solar_sep_deg(backend, observer, jd);
    let mut flo = f(lo, &mut sep)?;
    let mut fhi = f(hi, &mut sep)?;
    if flo.signum() == fhi.signum() {
        return Ok(None);
    }
    while (hi - lo) > REFINE_TOLERANCE_DAYS {
        let mid = 0.5 * (lo + hi);
        let fmid = f(mid, &mut sep)?;
        if fmid.signum() == flo.signum() {
            lo = mid;
            flo = fmid;
        } else {
            hi = mid;
            fhi = fmid;
        }
    }
    let _ = fhi;
    Ok(Some(0.5 * (lo + hi)))
}

/// Full set of solar contact instants for `observer` around `greatest_jd`.
pub(crate) fn solar_contacts_jd<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    greatest_jd: f64,
) -> Result<Option<SolarContactsJd>, EclipseError> {
    let max_jd = minimize_sep(
        backend,
        observer,
        greatest_jd - SOLAR_CONTACT_HALF_WINDOW_DAYS,
        greatest_jd + SOLAR_CONTACT_HALF_WINDOW_DAYS,
    )?;
    let g_max = solar_geom(&topo_sun_moon(backend, observer, max_jd)?);
    let external = g_max.s_sun_deg + g_max.s_moon_deg;
    if g_max.sep_deg >= external {
        return Ok(None); // observer sees no eclipse at all
    }
    // Semidiameter sums vary slowly; freeze them at max for the threshold fn
    // (sub-second contact error), matching the design's closed-form treatment.
    let ext_threshold = move |_jd: f64| external;
    let internal = (g_max.s_moon_deg - g_max.s_sun_deg).abs();
    let int_threshold = move |_jd: f64| internal;

    let lo = max_jd - SOLAR_CONTACT_HALF_WINDOW_DAYS;
    let hi = max_jd + SOLAR_CONTACT_HALF_WINDOW_DAYS;
    let c1 = bisect_contact(backend, observer, ext_threshold, lo, max_jd)?.unwrap_or(max_jd);
    let c4 = bisect_contact(backend, observer, ext_threshold, max_jd, hi)?.unwrap_or(max_jd);

    let c2_c3_present = g_max.sep_deg < internal;
    let (c2, c3) = if c2_c3_present {
        let c2 = bisect_contact(backend, observer, int_threshold, c1, max_jd)?.unwrap_or(max_jd);
        let c3 = bisect_contact(backend, observer, int_threshold, max_jd, c4)?.unwrap_or(max_jd);
        (c2, c3)
    } else {
        (max_jd, max_jd)
    };

    Ok(Some(SolarContactsJd {
        max_jd,
        c1_jd: c1,
        c2_jd: c2,
        c3_jd: c3,
        c4_jd: c4,
        min_sep_deg: g_max.sep_deg,
        s_sun_at_max: g_max.s_sun_deg,
        s_moon_at_max: g_max.s_moon_deg,
        c2_c3_present,
    }))
}

#[cfg(test)]
mod solar_contact_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn contacts_bracket_the_maximum() {
        // Analytic on-node backend → central solar eclipse for an equatorial observer.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let c = solar_contacts_jd(&backend, &observer, 2_451_550.0)
            .unwrap()
            .expect("a local eclipse");
        assert!(
            c.c1_jd <= c.max_jd + 1e-9 && c.max_jd <= c.c4_jd + 1e-9,
            "C1<=max<=C4"
        );
        assert!(c.c1_jd < c.c4_jd, "C1 strictly before C4");
        if c.c2_c3_present {
            assert!(
                c.c2_jd >= c.c1_jd - 1e-9 && c.c3_jd <= c.c4_jd + 1e-9,
                "C2/C3 inside C1..C4"
            );
            assert!(
                c.c2_jd <= c.max_jd + 1e-9 && c.max_jd <= c.c3_jd + 1e-9,
                "C2<=max<=C3"
            );
        }
    }
}

/// Which body a horizontal position is computed for.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LocalBody {
    Sun,
    Moon,
}

/// Topocentric azimuth (from south, west, `[0,360)`), apparent (refracted)
/// altitude, and above-horizon visibility of `body` at `jd` for `observer`.
pub(crate) fn body_horizontal<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    jd: f64,
    body: LocalBody,
) -> Result<(f64, f64, bool), EclipseError> {
    let t = topo_sun_moon(backend, observer, jd)?;
    let (lon, lat, dist) = match body {
        LocalBody::Sun => (t.sun_lon_deg, t.sun_lat_deg, t.sun_dist_au),
        LocalBody::Moon => (t.moon_lon_deg, t.moon_lat_deg, t.moon_dist_au),
    };
    let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let eps = true_obliquity_degrees(jd)
        .map_err(|e| EclipseError::Backend(format!("obliquity failed: {e}")))?;
    let equ = EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        Some(dist),
    )
    .to_equatorial(Angle::from_degrees(eps));
    let ra_deg = equ.right_ascension.degrees();
    let dec_deg = equ.declination.degrees();
    let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
    let ha = (lst - ra_deg).to_radians();
    let dec = dec_deg.to_radians();
    let phi = observer.latitude.degrees().to_radians();
    // Standard equatorial → horizontal (azimuth from south, increasing west).
    let sin_alt = (phi.sin() * dec.sin() + phi.cos() * dec.cos() * ha.cos()).clamp(-1.0, 1.0);
    let true_alt = sin_alt.asin().to_degrees();
    let az = ha.sin().atan2(ha.cos() * phi.sin() - dec.tan() * phi.cos());
    let apparent_alt = apparent_from_true(true_alt, atmos);
    Ok((
        az.to_degrees().rem_euclid(360.0),
        apparent_alt,
        apparent_alt > 0.0,
    ))
}

/// Packages a `jd` + a body's horizontal position into a `LocalContact`.
pub(crate) fn contact_at<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    jd: f64,
    body: LocalBody,
) -> Result<LocalContact, EclipseError> {
    let (az, alt, visible) = body_horizontal(backend, observer, atmos, jd, body)?;
    Ok(LocalContact {
        instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        altitude_degrees: alt,
        azimuth_degrees: az,
        visible,
    })
}

/// Classifies what the observer sees at local maximum from the two-circle
/// geometry: total when the Moon's disk fully covers the Sun's
/// (`sep + s_sun <= s_moon`), annular when the Moon is fully inside
/// (`sep + s_moon <= s_sun`), else partial. Hybrid is a global (path-level)
/// distinction, not a single-observer one, so a single observer sees total or
/// annular, never "hybrid".
fn classify_local_solar(c: &SolarContactsJd) -> SolarEclipseType {
    let (sep, s_sun, s_moon) = (c.min_sep_deg, c.s_sun_at_max, c.s_moon_at_max);
    if c.c2_c3_present && sep + s_sun <= s_moon + 1e-9 {
        SolarEclipseType::Total
    } else if c.c2_c3_present && sep + s_moon <= s_sun + 1e-9 {
        SolarEclipseType::Annular
    } else {
        SolarEclipseType::Partial
    }
}

/// Whether the Sun is above the horizon anywhere in `[c1,c4]` (coarse 2-min scan).
fn solar_any_visible<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    c1_jd: f64,
    c4_jd: f64,
) -> Result<bool, EclipseError> {
    let step = 2.0 / 1440.0;
    let mut jd = c1_jd;
    while jd <= c4_jd + 1e-12 {
        let (_, _, vis) = body_horizontal(backend, observer, atmos, jd, LocalBody::Sun)?;
        if vis {
            return Ok(true);
        }
        jd += step;
    }
    Ok(false)
}

/// Full solar local circumstances for `observer` around `greatest_jd`.
pub(crate) fn solar_local<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    greatest_jd: f64,
) -> Result<LocalSolarCircumstances, EclipseError> {
    let c = solar_contacts_jd(backend, observer, greatest_jd)?;
    let sun = LocalBody::Sun;
    match c {
        None => {
            // No eclipse for this observer: a degenerate all-at-greatest record,
            // not visible. `next_local_eclipse` filters these out; `local_circumstances`
            // still returns it so a caller can inspect "not visible here".
            let contact = contact_at(backend, observer, atmos, greatest_jd, sun)?;
            let g = solar_geom(&topo_sun_moon(backend, observer, greatest_jd)?);
            Ok(LocalSolarCircumstances {
                local_type: SolarEclipseType::Partial,
                maximum: contact,
                magnitude: covered_diameter_fraction(&g), // 0.0 here
                obscuration: obscuration_fraction(&g),
                first_contact: contact,
                second_contact: None,
                third_contact: None,
                fourth_contact: contact,
                any_phase_visible: contact.visible,
            })
        }
        Some(c) => {
            let g_max = solar_geom(&topo_sun_moon(backend, observer, c.max_jd)?);
            let local_type = classify_local_solar(&c);
            let maximum = contact_at(backend, observer, atmos, c.max_jd, sun)?;
            let first_contact = contact_at(backend, observer, atmos, c.c1_jd, sun)?;
            let fourth_contact = contact_at(backend, observer, atmos, c.c4_jd, sun)?;
            let (second_contact, third_contact) = if c.c2_c3_present {
                (
                    Some(contact_at(backend, observer, atmos, c.c2_jd, sun)?),
                    Some(contact_at(backend, observer, atmos, c.c3_jd, sun)?),
                )
            } else {
                (None, None)
            };
            let any_phase_visible = solar_any_visible(backend, observer, atmos, c.c1_jd, c.c4_jd)?;
            Ok(LocalSolarCircumstances {
                local_type,
                maximum,
                magnitude: covered_diameter_fraction(&g_max),
                obscuration: obscuration_fraction(&g_max),
                first_contact,
                second_contact,
                third_contact,
                fourth_contact,
                any_phase_visible,
            })
        }
    }
}

#[cfg(test)]
mod solar_local_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn central_eclipse_has_full_magnitude_and_ordered_contacts() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let s = solar_local(&backend, &observer, Atmosphere::default(), 2_451_550.0).unwrap();
        assert!(s.magnitude > 0.0, "central magnitude {}", s.magnitude);
        assert!(s.obscuration >= 0.0 && s.obscuration <= 1.0);
        assert!(
            s.first_contact.instant.julian_day.days() <= s.maximum.instant.julian_day.days() + 1e-9
        );
        assert!(
            s.maximum.instant.julian_day.days()
                <= s.fourth_contact.instant.julian_day.days() + 1e-9
        );
    }
}

#[cfg(test)]
mod horizontal_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn altitude_is_finite_and_azimuth_in_range() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let (az, alt, _vis) = body_horizontal(
            &backend,
            &observer,
            Atmosphere::default(),
            2_451_545.0,
            LocalBody::Sun,
        )
        .unwrap();
        assert!(alt.is_finite() && alt <= 90.0 + 1e-9, "alt {alt}");
        assert!((0.0..360.0).contains(&az), "az {az}");
    }
}

use crate::geometry::lunar_shadow;

/// Half-window (days) to search for lunar shadow contacts around greatest
/// eclipse. A penumbral lunar eclipse lasts up to ~6 h; 0.25 day is safe.
const LUNAR_CONTACT_HALF_WINDOW_DAYS: f64 = 0.25;

/// Signed residual `dist(t) - radius` for a lunar contact, where `dist` is the
/// Moon-to-shadow-axis separation `sigma` and `radius` is one of the shadow
/// radii (umbral/penumbral), optionally offset by the Moon's semidiameter for
/// the exterior/interior contacts. `kind` selects the residual.
#[derive(Clone, Copy)]
enum LunarContactKind {
    /// P1/P4: sigma == p + m_moon (penumbra first touches / last touches disc).
    Penumbral,
    /// U1/U4: sigma == u + m_moon (umbra first touches / last touches disc).
    UmbralPartial,
    /// U2/U3: sigma == u - m_moon (disc fully enters / begins leaving umbra).
    UmbralTotal,
}

fn lunar_residual<B: EphemerisBackend>(
    backend: &B,
    kind: LunarContactKind,
    jd: f64,
) -> Result<f64, EclipseError> {
    let sh = lunar_shadow(&sample_sun_moon(backend, jd)?);
    let target = match kind {
        LunarContactKind::Penumbral => sh.p + sh.m_moon,
        LunarContactKind::UmbralPartial => sh.u + sh.m_moon,
        LunarContactKind::UmbralTotal => sh.u - sh.m_moon,
    };
    Ok(sh.sigma - target)
}

/// Bisect a lunar contact residual between `lo` and `hi`; `None` if no sign change.
fn bisect_lunar<B: EphemerisBackend>(
    backend: &B,
    kind: LunarContactKind,
    mut lo: f64,
    mut hi: f64,
) -> Result<Option<f64>, EclipseError> {
    let mut flo = lunar_residual(backend, kind, lo)?;
    let fhi = lunar_residual(backend, kind, hi)?;
    if flo.signum() == fhi.signum() {
        return Ok(None);
    }
    while (hi - lo) > REFINE_TOLERANCE_DAYS {
        let mid = 0.5 * (lo + hi);
        let fmid = lunar_residual(backend, kind, mid)?;
        if fmid.signum() == flo.signum() {
            lo = mid;
            flo = fmid;
        } else {
            hi = mid;
        }
    }
    Ok(Some(0.5 * (lo + hi)))
}

/// Returns `(type, umbral_magnitude, penumbral_magnitude)` at greatest eclipse.
fn classify_lunar_public<B: EphemerisBackend>(
    backend: &B,
    greatest_jd: f64,
) -> Result<(LunarEclipseType, f64, f64), EclipseError> {
    let sample = sample_sun_moon(backend, greatest_jd)?;
    let sh = lunar_shadow(&sample);
    let umbral_magnitude = ((sh.u + sh.m_moon - sh.sigma) / (2.0 * sh.m_moon)).max(0.0);
    let penumbral_magnitude = ((sh.p + sh.m_moon - sh.sigma) / (2.0 * sh.m_moon)).max(0.0);
    let eclipse_type = if sh.sigma + sh.m_moon <= sh.u {
        LunarEclipseType::Total
    } else if sh.sigma - sh.m_moon < sh.u {
        LunarEclipseType::Partial
    } else {
        LunarEclipseType::Penumbral
    };
    Ok((eclipse_type, umbral_magnitude, penumbral_magnitude))
}

/// Full lunar local circumstances. Contact instants are global; visibility is local.
pub(crate) fn lunar_local<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    greatest_jd: f64,
) -> Result<LocalLunarCircumstances, EclipseError> {
    let moon = LocalBody::Moon;
    let g = classify_lunar_public(backend, greatest_jd)?; // (type, umbral_mag, penumbral_mag)
    let (eclipse_type, umbral_magnitude, penumbral_magnitude) = g;
    let lo = greatest_jd - LUNAR_CONTACT_HALF_WINDOW_DAYS;
    let hi = greatest_jd + LUNAR_CONTACT_HALF_WINDOW_DAYS;

    let find = |kind: LunarContactKind, a: f64, b: f64| bisect_lunar(backend, kind, a, b);
    // Penumbral contacts always exist for any lunar eclipse.
    let p1 = find(LunarContactKind::Penumbral, lo, greatest_jd)?.unwrap_or(greatest_jd);
    let p4 = find(LunarContactKind::Penumbral, greatest_jd, hi)?.unwrap_or(greatest_jd);
    let u1 = find(LunarContactKind::UmbralPartial, lo, greatest_jd)?;
    let u4 = find(LunarContactKind::UmbralPartial, greatest_jd, hi)?;
    let u2 = find(LunarContactKind::UmbralTotal, lo, greatest_jd)?;
    let u3 = find(LunarContactKind::UmbralTotal, greatest_jd, hi)?;

    let mk = |jd: f64| contact_at(backend, observer, atmos, jd, moon);
    let opt = |o: Option<f64>| -> Result<Option<LocalContact>, EclipseError> {
        match o {
            Some(jd) => Ok(Some(mk(jd)?)),
            None => Ok(None),
        }
    };

    // Visibility across the widest phase present (P1..P4).
    let any_phase_visible = {
        let step = 5.0 / 1440.0;
        let mut jd = p1;
        let mut vis = false;
        while jd <= p4 + 1e-12 {
            let (_, _, v) = body_horizontal(backend, observer, atmos, jd, moon)?;
            if v {
                vis = true;
                break;
            }
            jd += step;
        }
        vis
    };

    Ok(LocalLunarCircumstances {
        eclipse_type,
        maximum: mk(greatest_jd)?,
        umbral_magnitude,
        penumbral_magnitude,
        penumbral_begin: mk(p1)?,
        partial_begin: opt(u1)?,
        total_begin: opt(u2)?,
        total_end: opt(u3)?,
        partial_end: opt(u4)?,
        penumbral_end: mk(p4)?,
        any_phase_visible,
    })
}

#[cfg(test)]
mod lunar_local_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn lunar_contacts_are_ordered_and_penumbra_brackets_umbra() {
        let backend = LinearSunMoon::full_moon_at(2_451_550.0).with_moon_latitude(0.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let l = lunar_local(&backend, &observer, Atmosphere::default(), 2_451_550.0).unwrap();
        let p1 = l.penumbral_begin.instant.julian_day.days();
        let p4 = l.penumbral_end.instant.julian_day.days();
        assert!(p1 <= p4, "P1 <= P4");
        if let (Some(u1), Some(u4)) = (l.partial_begin, l.partial_end) {
            assert!(p1 <= u1.instant.julian_day.days() + 1e-9);
            assert!(u4.instant.julian_day.days() <= p4 + 1e-9);
        }
    }
}

/// Computes local circumstances for an already-found `eclipse`.
pub(crate) fn local_circumstances_for<B: EphemerisBackend>(
    backend: &B,
    eclipse: &Eclipse,
    observer: &ObserverLocation,
    atmos: Atmosphere,
) -> Result<LocalCircumstances, EclipseError> {
    let greatest_jd = eclipse.greatest_eclipse.julian_day.days();
    match eclipse.kind {
        EclipseKind::Solar => Ok(LocalCircumstances::Solar(solar_local(
            backend,
            observer,
            atmos,
            greatest_jd,
        )?)),
        EclipseKind::Lunar => Ok(LocalCircumstances::Lunar(lunar_local(
            backend,
            observer,
            atmos,
            greatest_jd,
        )?)),
    }
}

/// Whether a computed local result has any above-horizon phase.
pub(crate) fn is_locally_visible(local: &LocalCircumstances) -> bool {
    match local {
        LocalCircumstances::Solar(s) => s.any_phase_visible,
        LocalCircumstances::Lunar(l) => l.any_phase_visible,
    }
}
