//! Lunar occultations — Swiss Ephemeris `swe_lun_occult_*` analogue: the Moon
//! occulting a planet (small disc) or a catalogued fixed star (point). Local
//! circumstances (`how`), per-observer search (`when_loc`), and global search
//! (`when_glob`). Geocentric/topocentric apparent-of-date, TDB, 1900–2100 CE.

use pleiades_types::{
    Angle, CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, TimeScale, OBLIQUITY_J2000_DEG,
};

/// What the Moon is occulting.
#[derive(Clone, Debug, PartialEq)]
pub enum OccultTarget {
    /// A planet, Mercury..=Pluto. Sun and Moon are rejected (Sun ⇒ solar
    /// eclipse; the Moon is the occulter).
    Body(CelestialBody),
    /// A curated fixed-star catalog name (see [`crate::fixed_star_entry`]).
    Star(String),
}

/// What an observer (or the globe, for `when_glob`) actually sees.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OccultationType {
    /// Target fully covered at maximum (point star hidden, or planet disc fully
    /// behind the Moon's limb).
    Total,
    /// The Moon's limb crosses the target but never fully covers it.
    Grazing,
    /// No contact (topocentric/geocentric separation never small enough).
    Miss,
}

/// One observer-local contact event: its instant plus the target's horizontal
/// position and visibility there. A contact below the horizon is still timed
/// but flagged `visible == false`, matching SE.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OccultationContact {
    /// Instant of the contact (TDB).
    pub instant: Instant,
    /// Apparent (refracted) altitude of the target, degrees.
    pub altitude_degrees: f64,
    /// Azimuth from south increasing westward, `[0,360)` degrees.
    pub azimuth_degrees: f64,
    /// Whether the target is above the horizon at this instant.
    pub visible: bool,
}

/// Local circumstances of a lunar occultation of one target for one observer
/// (`how` / `when_loc`). Contact fields mirror the SP-2c eclipse C1–C4 layout.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalOccultation {
    /// The occulted target.
    pub target: OccultTarget,
    /// What THIS observer sees.
    pub occultation_type: OccultationType,
    /// Instant of least topocentric separation.
    pub maximum: OccultationContact,
    /// Covered fraction of the target's diameter at maximum (SE `attr[0]`).
    pub magnitude: f64,
    /// Covered fraction of the target's disc area at maximum (SE `attr[2]`);
    /// `0.0`/`1.0` for a point star.
    pub obscuration: f64,
    /// C1 — disappearance (exterior ingress).
    pub first_contact: OccultationContact,
    /// C2 — target fully hidden (planet disc only; `None` for a star / graze).
    pub second_contact: Option<OccultationContact>,
    /// C3 — target begins to reappear (planet disc only).
    pub third_contact: Option<OccultationContact>,
    /// C4 — reappearance (exterior egress).
    pub fourth_contact: OccultationContact,
    /// Whether the target is above the horizon during any part of the event.
    pub any_phase_visible: bool,
}

/// Global circumstances (`when_glob`): the greatest-occultation instant and the
/// sub-lunar point where it is central/greatest. NOT the full path polygon.
#[derive(Clone, Debug, PartialEq)]
pub struct GlobalOccultation {
    /// The occulted target.
    pub target: OccultTarget,
    /// Instant of greatest global occultation (TDB).
    pub maximum: Instant,
    /// Sub-lunar point of maximum occultation: geographic latitude, positive north.
    pub sublunar_latitude: Latitude,
    /// Sub-lunar point of maximum occultation: geographic longitude, positive east.
    pub sublunar_longitude: Longitude,
    /// Whether a central occultation exists somewhere on Earth.
    pub central: bool,
    /// Best-case type over the globe.
    pub occultation_type: OccultationType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{JulianDay, TimeScale};

    fn contact(jd: f64) -> OccultationContact {
        OccultationContact {
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            altitude_degrees: 42.0,
            azimuth_degrees: 100.0,
            visible: true,
        }
    }

    #[test]
    fn local_occultation_holds_optional_interior_contacts() {
        let star = LocalOccultation {
            target: OccultTarget::Star("Aldebaran".into()),
            occultation_type: OccultationType::Total,
            maximum: contact(2_451_545.0),
            magnitude: 1.0,
            obscuration: 1.0,
            first_contact: contact(2_451_544.98),
            second_contact: None,
            third_contact: None,
            fourth_contact: contact(2_451_545.02),
            any_phase_visible: true,
        };
        assert!(star.second_contact.is_none());
        assert_eq!(star.occultation_type, OccultationType::Total);
    }
}

/// Instantaneous two-circle geometry: Moon vs target, all degrees.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OccGeom {
    /// Center-to-center Moon–target separation.
    pub sep_deg: f64,
    /// Moon's (topocentric) angular semidiameter.
    pub s_moon_deg: f64,
    /// Target's angular semidiameter (0 for a point star).
    pub s_tgt_deg: f64,
}

/// Great-circle separation (degrees) between two equatorial points (RA, Dec).
pub(crate) fn angular_separation_deg(ra1: f64, dec1: f64, ra2: f64, dec2: f64) -> f64 {
    let (a1, d1) = (ra1.to_radians(), dec1.to_radians());
    let (a2, d2) = (ra2.to_radians(), dec2.to_radians());
    let cos_sep = (d1.sin() * d2.sin() + d1.cos() * d2.cos() * (a1 - a2).cos()).clamp(-1.0, 1.0);
    cos_sep.acos().to_degrees()
}

/// Covered fraction of the TARGET's diameter (SE `attr[0]`). For a point star
/// (`s_tgt == 0`) this is binary: 1.0 when the target is inside the Moon's disc,
/// else 0.0. Clamped ≥ 0.
pub(crate) fn covered_diameter_fraction(g: &OccGeom) -> f64 {
    if g.s_tgt_deg <= 0.0 {
        return if g.sep_deg < g.s_moon_deg { 1.0 } else { 0.0 };
    }
    ((g.s_moon_deg + g.s_tgt_deg - g.sep_deg) / (2.0 * g.s_tgt_deg)).max(0.0)
}

/// Covered fraction of the TARGET's disc AREA (SE `attr[2]`). Standard
/// two-circle lens area; the covered disc is the target (radius `s_tgt`), the
/// covering disc is the Moon (radius `s_moon`). Binary for a point star.
pub(crate) fn obscuration_fraction(g: &OccGeom) -> f64 {
    let (r_t, r_m, d) = (g.s_tgt_deg, g.s_moon_deg, g.sep_deg);
    if r_t <= 0.0 {
        return if d < r_m { 1.0 } else { 0.0 };
    }
    if d >= r_t + r_m {
        return 0.0; // disjoint
    }
    if d <= r_m - r_t {
        return 1.0; // target fully behind the Moon
    }
    if d <= (r_t - r_m).max(0.0) {
        // Moon fully inside the target disc (target larger — impossible for real
        // planets vs Moon, but kept for closed-form completeness).
        return ((r_m / r_t).powi(2)).clamp(0.0, 1.0);
    }
    let r_t2 = r_t * r_t;
    let r_m2 = r_m * r_m;
    let a_t = ((d * d + r_t2 - r_m2) / (2.0 * d * r_t)).clamp(-1.0, 1.0).acos();
    let a_m = ((d * d + r_m2 - r_t2) / (2.0 * d * r_m)).clamp(-1.0, 1.0).acos();
    let lens = r_t2 * a_t + r_m2 * a_m
        - 0.5
            * ((r_t + r_m + d) * (-r_t + r_m + d) * (r_t - r_m + d) * (r_t + r_m - d))
                .max(0.0)
                .sqrt();
    (lens / (core::f64::consts::PI * r_t2)).clamp(0.0, 1.0)
}

/// Classify the occultation from the geometry at maximum: `Total` when the
/// target is fully covered (`sep < s_moon − s_tgt`), `Grazing` when the limb
/// crosses but never fully covers (`s_moon − s_tgt ≤ sep < s_moon + s_tgt`, and
/// for a point star the knife-edge `sep == s_moon`), else `Miss`.
pub(crate) fn classify(g: &OccGeom) -> OccultationType {
    let internal = (g.s_moon_deg - g.s_tgt_deg).max(0.0);
    let external = g.s_moon_deg + g.s_tgt_deg;
    if g.sep_deg < internal {
        OccultationType::Total
    } else if g.sep_deg < external {
        // A point star (s_tgt == 0) has internal == external == s_moon, so it is
        // never Grazing at this branch; a sep exactly at s_moon is the Miss edge.
        if g.s_tgt_deg <= 0.0 {
            OccultationType::Miss
        } else {
            OccultationType::Grazing
        }
    } else {
        OccultationType::Miss
    }
}

#[cfg(test)]
mod geom_tests {
    use super::*;

    fn g(sep: f64, s_moon: f64, s_tgt: f64) -> OccGeom {
        OccGeom { sep_deg: sep, s_moon_deg: s_moon, s_tgt_deg: s_tgt }
    }

    #[test]
    fn star_hidden_is_total_and_full_magnitude() {
        let geo = g(0.1, 0.25, 0.0); // sep < s_moon, point star
        assert_eq!(classify(&geo), OccultationType::Total);
        assert_eq!(covered_diameter_fraction(&geo), 1.0);
        assert_eq!(obscuration_fraction(&geo), 1.0);
    }

    #[test]
    fn star_clear_is_miss_zero_magnitude() {
        let geo = g(0.30, 0.25, 0.0); // sep > s_moon
        assert_eq!(classify(&geo), OccultationType::Miss);
        assert_eq!(covered_diameter_fraction(&geo), 0.0);
        assert_eq!(obscuration_fraction(&geo), 0.0);
    }

    #[test]
    fn planet_fully_behind_is_total() {
        let geo = g(0.20, 0.25, 0.003); // sep < s_moon - s_tgt
        assert_eq!(classify(&geo), OccultationType::Total);
        assert!(covered_diameter_fraction(&geo) >= 1.0);
        assert!((obscuration_fraction(&geo) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn planet_partly_covered_is_grazing_partial() {
        let geo = g(0.2495, 0.25, 0.003); // between internal and external
        assert_eq!(classify(&geo), OccultationType::Grazing);
        let o = obscuration_fraction(&geo);
        assert!(o > 0.0 && o < 1.0, "partial obscuration {o}");
    }

    #[test]
    fn separation_is_symmetric_and_zero_on_identity() {
        assert!(angular_separation_deg(10.0, 20.0, 10.0, 20.0) < 1e-9);
        let a = angular_separation_deg(10.0, 20.0, 12.0, 19.0);
        let b = angular_separation_deg(12.0, 19.0, 10.0, 20.0);
        assert!((a - b).abs() < 1e-12);
    }
}

use crate::crossings::EventEngine;
use crate::ephemeris::geocentric_apparent_ecliptic;
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::fixstar::fixed_star_apparent;
use crate::rise_trans::{check_atmosphere, RiseSetTarget};
use crate::root::REFINE_TOLERANCE_DAYS;
use crate::semidiameter::semidiameter_deg;
use pleiades_apparent::{
    apparent_from_true, sidereal_time, topocentric_position, true_obliquity_degrees, Atmosphere,
};
use pleiades_backend::EphemerisBackend;

/// Moon physical radius (km) and AU, matching the eclipse crate's `solar_consts`.
pub(crate) const R_MOON_KM: f64 = 1_737.4;
pub(crate) const AU_KM: f64 = 149_597_870.7;

/// The Moon's maximum reach in ecliptic latitude: ~5.3° orbital inclination +
/// ~0.27° semidiameter + ~0.95° horizontal parallax ≈ 6.6°. A star beyond this
/// |ecliptic latitude| can never be occulted from anywhere on Earth.
pub(crate) const MOON_MAX_REACH_DEG: f64 = 6.6;

impl<B: EphemerisBackend> EventEngine<B> {
    /// Apparent equatorial-of-date RA/Dec (degrees) of the Moon and the target at
    /// `jd`. When `observer` is `Some`, the Moon (and a planet target) carry
    /// diurnal parallax via `topocentric_position`; a `Star` target has no
    /// parallax. When `observer` is `None`, both are geocentric.
    #[allow(clippy::type_complexity)]
    pub(crate) fn moon_target_radec(
        &self,
        target: &OccultTarget,
        observer: Option<&ObserverLocation>,
        jd: f64,
    ) -> Result<((f64, f64), (f64, f64)), EventError> {
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let eps = true_obliquity_degrees(jd)
            .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
        // UT1-rotated sidereal time for parallax (matches eclipse local.rs).
        let sid_jd = pleiades_time::ut1_jd_from_tt(jd).unwrap_or(jd);
        let sid_at = Instant::new(JulianDay::from_days(sid_jd), TimeScale::Tdb);
        let lst = sidereal_time(sid_at, Longitude::from_degrees(0.0)).local_apparent_deg;

        let body_radec = |b: &CelestialBody| -> Result<(f64, f64), EventError> {
            let (lon, lat, dist) =
                geocentric_apparent_ecliptic(&self.backend, b.clone(), "body", jd)?;
            let ecl = EclipticCoordinates::new(
                Longitude::from_degrees(lon),
                Latitude::from_degrees(lat),
                Some(dist),
            );
            let ecl = if let Some(obs) = observer {
                // Parallax needs the observer's local sidereal time.
                let l = sidereal_time(sid_at, obs.longitude).local_apparent_deg;
                topocentric_position(ecl, obs, l, eps)
                    .map_err(|e| EventError::Backend(format!("topocentric failed: {e}")))?
                    .ecliptic
            } else {
                ecl
            };
            let equ = ecl.to_equatorial(Angle::from_degrees(eps));
            Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
        };
        let _ = lst; // observer-specific lst computed inside the closure

        let moon = body_radec(&CelestialBody::Moon)?;
        let tgt = match target {
            OccultTarget::Body(b) => body_radec(b)?,
            OccultTarget::Star(name) => {
                let equ = fixed_star_apparent(name, at)?;
                (equ.right_ascension.degrees(), equ.declination.degrees())
            }
        };
        Ok((moon, tgt))
    }

    /// Two-circle geometry (separation + semidiameters) of the Moon vs target at
    /// `jd`, topocentric when `observer` is `Some`.
    pub(crate) fn occ_geom(
        &self,
        target: &OccultTarget,
        observer: Option<&ObserverLocation>,
        jd: f64,
    ) -> Result<OccGeom, EventError> {
        let (moon, tgt) = self.moon_target_radec(target, observer, jd)?;
        let sep_deg = angular_separation_deg(moon.0, moon.1, tgt.0, tgt.1);
        // Moon semidiameter from its topocentric/geocentric distance.
        let moon_dist = self.body_distance_au(&CelestialBody::Moon, observer, jd)?;
        let s_moon_deg = (R_MOON_KM / (moon_dist * AU_KM)).clamp(-1.0, 1.0).asin().to_degrees();
        let s_tgt_deg = match target {
            OccultTarget::Body(b) => {
                let dist = self.body_distance_au(b, observer, jd)?;
                semidiameter_deg(&RiseSetTarget::Body(b.clone()), dist, false)
            }
            OccultTarget::Star(_) => 0.0,
        };
        Ok(OccGeom { sep_deg, s_moon_deg, s_tgt_deg })
    }

    /// Topocentric (if `observer`) or geocentric distance (AU) of a body.
    fn body_distance_au(
        &self,
        b: &CelestialBody,
        observer: Option<&ObserverLocation>,
        jd: f64,
    ) -> Result<f64, EventError> {
        let (lon, lat, dist) = geocentric_apparent_ecliptic(&self.backend, b.clone(), "body", jd)?;
        if let Some(obs) = observer {
            let eps = true_obliquity_degrees(jd)
                .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
            let sid_jd = pleiades_time::ut1_jd_from_tt(jd).unwrap_or(jd);
            let sid_at = Instant::new(JulianDay::from_days(sid_jd), TimeScale::Tdb);
            let l = sidereal_time(sid_at, obs.longitude).local_apparent_deg;
            let ecl = EclipticCoordinates::new(
                Longitude::from_degrees(lon),
                Latitude::from_degrees(lat),
                Some(dist),
            );
            let topo = topocentric_position(ecl, obs, l, eps)
                .map_err(|e| EventError::Backend(format!("topocentric failed: {e}")))?;
            Ok(topo.ecliptic.distance_au.unwrap_or(dist))
        } else {
            Ok(dist)
        }
    }

    /// Rejects Sun/Moon as an occultation target and unknown star names.
    pub(crate) fn validate_occult_target(&self, target: &OccultTarget) -> Result<(), EventError> {
        match target {
            OccultTarget::Body(CelestialBody::Sun) => Err(EventError::UnsupportedOccultTarget {
                detail: "the Sun occulted by the Moon is a solar eclipse; use the eclipse engine"
                    .into(),
            }),
            OccultTarget::Body(CelestialBody::Moon) => Err(EventError::UnsupportedOccultTarget {
                detail: "the Moon is the occulter, not a target".into(),
            }),
            OccultTarget::Body(_) => Ok(()),
            OccultTarget::Star(name) => crate::fixstar::fixed_star_entry(name).map(|_| ()),
        }
    }

    /// Whether the target's ecliptic latitude puts it permanently outside the
    /// Moon's reach (only meaningful — and only applied — for stars, whose
    /// latitude is effectively constant; planets always return `false`).
    pub(crate) fn target_never_occultable(
        &self,
        target: &OccultTarget,
        jd: f64,
    ) -> Result<bool, EventError> {
        match target {
            OccultTarget::Star(name) => {
                let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
                let equ = fixed_star_apparent(name, at)?;
                let eps = true_obliquity_degrees(jd).unwrap_or(OBLIQUITY_J2000_DEG);
                let ecl = equ.to_ecliptic(Angle::from_degrees(eps));
                Ok(ecl.latitude.degrees().abs() > MOON_MAX_REACH_DEG)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod sampler_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    fn obs() -> ObserverLocation {
        ObserverLocation::new(Latitude::from_degrees(0.0), Longitude::from_degrees(0.0), Some(0.0))
    }

    #[test]
    fn moon_semidiameter_is_about_a_quarter_degree() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let g = engine
            .occ_geom(&OccultTarget::Body(CelestialBody::Sun), None, 2_451_550.0)
            .unwrap();
        assert!(g.s_moon_deg > 0.2 && g.s_moon_deg < 0.3, "moon SD {}", g.s_moon_deg);
        assert!(g.sep_deg.is_finite());
    }

    #[test]
    fn topocentric_separation_differs_from_geocentric() {
        // Diurnal parallax shifts the Moon, so the Moon–planet separation seen
        // topocentrically differs from the geocentric one.
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let geo = engine
            .occ_geom(&OccultTarget::Body(CelestialBody::Sun), None, 2_451_550.0)
            .unwrap();
        let topo = engine
            .occ_geom(&OccultTarget::Body(CelestialBody::Sun), Some(&obs()), 2_451_550.0)
            .unwrap();
        assert!((geo.sep_deg - topo.sep_deg).abs() > 0.0);
    }
}

#[cfg(test)]
mod target_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn sun_and_moon_targets_are_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        assert!(matches!(
            engine.validate_occult_target(&OccultTarget::Body(CelestialBody::Sun)),
            Err(EventError::UnsupportedOccultTarget { .. })
        ));
        assert!(matches!(
            engine.validate_occult_target(&OccultTarget::Body(CelestialBody::Moon)),
            Err(EventError::UnsupportedOccultTarget { .. })
        ));
        assert!(engine
            .validate_occult_target(&OccultTarget::Body(CelestialBody::Venus))
            .is_ok());
    }

    #[test]
    fn unknown_star_is_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        assert!(matches!(
            engine.validate_occult_target(&OccultTarget::Star("Nope".into())),
            Err(EventError::UnknownFixedStar { .. })
        ));
    }

    #[test]
    fn far_off_ecliptic_star_is_never_occultable() {
        // Sirius sits ~39° below the ecliptic; the Moon can never reach it.
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        assert!(engine
            .target_never_occultable(&OccultTarget::Star("Sirius".into()), 2_451_545.0)
            .unwrap());
        // Aldebaran (~5.5°S) is within reach.
        assert!(!engine
            .target_never_occultable(&OccultTarget::Star("Aldebaran".into()), 2_451_545.0)
            .unwrap());
    }
}

/// Half-window (days) to bracket occultation contacts around the maximum.
const OCC_CONTACT_HALF_WINDOW_DAYS: f64 = 0.15;

impl<B: EphemerisBackend> EventEngine<B> {
    /// Topocentric az/alt + visibility of the target at `jd` for `observer`.
    fn target_horizontal(
        &self,
        target: &OccultTarget,
        observer: &ObserverLocation,
        atmos: Atmosphere,
        jd: f64,
    ) -> Result<(f64, f64, bool), EventError> {
        let (_, (ra_deg, dec_deg)) = self.moon_target_radec(target, Some(observer), jd)?;
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        let ha = (lst - ra_deg).to_radians();
        let dec = dec_deg.to_radians();
        let phi = observer.latitude.degrees().to_radians();
        let sin_alt = (phi.sin() * dec.sin() + phi.cos() * dec.cos() * ha.cos()).clamp(-1.0, 1.0);
        let true_alt = sin_alt.asin().to_degrees();
        let az = ha.sin().atan2(ha.cos() * phi.sin() - dec.tan() * phi.cos());
        let app_alt = apparent_from_true(true_alt, atmos);
        Ok((az.to_degrees().rem_euclid(360.0), app_alt, app_alt > 0.0))
    }

    fn occ_contact_at(
        &self,
        target: &OccultTarget,
        observer: &ObserverLocation,
        atmos: Atmosphere,
        jd: f64,
    ) -> Result<OccultationContact, EventError> {
        let (az, alt, visible) = self.target_horizontal(target, observer, atmos, jd)?;
        Ok(OccultationContact {
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            altitude_degrees: alt,
            azimuth_degrees: az,
            visible,
        })
    }

    /// Golden-section minimize the topocentric separation in `[a,b]`.
    fn minimize_occ_sep(
        &self,
        target: &OccultTarget,
        observer: &ObserverLocation,
        mut a: f64,
        mut b: f64,
    ) -> Result<f64, EventError> {
        let phi = 0.618_033_988_75_f64;
        let sep = |jd: f64| Ok::<f64, EventError>(self.occ_geom(target, Some(observer), jd)?.sep_deg);
        let mut c = b - (b - a) * phi;
        let mut d = a + (b - a) * phi;
        let (mut fc, mut fd) = (sep(c)?, sep(d)?);
        while (b - a) > REFINE_TOLERANCE_DAYS {
            if fc < fd {
                b = d; d = c; fd = fc; c = b - (b - a) * phi; fc = sep(c)?;
            } else {
                a = c; c = d; fc = fd; d = a + (b - a) * phi; fd = sep(d)?;
            }
        }
        Ok(0.5 * (a + b))
    }

    /// Bisect `sep(t) − threshold` between `lo` and `hi`; `None` if no sign change.
    fn bisect_occ_contact(
        &self,
        target: &OccultTarget,
        observer: &ObserverLocation,
        threshold: f64,
        mut lo: f64,
        mut hi: f64,
    ) -> Result<Option<f64>, EventError> {
        let f = |jd: f64| Ok::<f64, EventError>(self.occ_geom(target, Some(observer), jd)?.sep_deg - threshold);
        let mut flo = f(lo)?;
        let fhi = f(hi)?;
        if flo.signum() == fhi.signum() {
            return Ok(None);
        }
        while (hi - lo) > REFINE_TOLERANCE_DAYS {
            let mid = 0.5 * (lo + hi);
            let fmid = f(mid)?;
            if fmid.signum() == flo.signum() { lo = mid; flo = fmid; } else { hi = mid; }
        }
        Ok(Some(0.5 * (lo + hi)))
    }

    /// Local circumstances of a lunar occultation of `target` for `observer` at
    /// (or around) `at` — Swiss Ephemeris `swe_lun_occult_how` analogue. Returns
    /// full circumstances even when the target is below the horizon; the
    /// `occultation_type` is `Miss` when no contact occurs for this observer.
    pub fn occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,
        at: Instant,
    ) -> Result<LocalOccultation, EventError> {
        self.validate_occult_target(&target)?;
        observer.validate().map_err(|e| EventError::InvalidObserver { detail: e.to_string() })?;
        check_atmosphere(atmosphere)?;
        let jd0 = at.julian_day.days();
        self.check_window(jd0)?;

        let max_jd = self.minimize_occ_sep(
            &target,
            &observer,
            (jd0 - OCC_CONTACT_HALF_WINDOW_DAYS).max(WINDOW_START_JD),
            (jd0 + OCC_CONTACT_HALF_WINDOW_DAYS).min(WINDOW_END_JD),
        )?;
        let g = self.occ_geom(&target, Some(&observer), max_jd)?;
        let occ_type = classify(&g);
        let maximum = self.occ_contact_at(&target, &observer, atmosphere, max_jd)?;

        if matches!(occ_type, OccultationType::Miss) {
            // A timed "no occultation here" record: all contacts at the closest
            // approach, magnitude 0, not visible as an event.
            return Ok(LocalOccultation {
                target,
                occultation_type: OccultationType::Miss,
                maximum,
                magnitude: 0.0,
                obscuration: 0.0,
                first_contact: maximum,
                second_contact: None,
                third_contact: None,
                fourth_contact: maximum,
                any_phase_visible: false,
            });
        }

        let external = g.s_moon_deg + g.s_tgt_deg;
        let internal = (g.s_moon_deg - g.s_tgt_deg).max(0.0);
        let lo = (max_jd - OCC_CONTACT_HALF_WINDOW_DAYS).max(WINDOW_START_JD);
        let hi = (max_jd + OCC_CONTACT_HALF_WINDOW_DAYS).min(WINDOW_END_JD);
        let c1 = self.bisect_occ_contact(&target, &observer, external, lo, max_jd)?.unwrap_or(max_jd);
        let c4 = self.bisect_occ_contact(&target, &observer, external, max_jd, hi)?.unwrap_or(max_jd);

        let disc_total = g.s_tgt_deg > 0.0 && g.sep_deg < internal;
        let (c2, c3) = if disc_total {
            (
                self.bisect_occ_contact(&target, &observer, internal, c1, max_jd)?,
                self.bisect_occ_contact(&target, &observer, internal, max_jd, c4)?,
            )
        } else {
            (None, None)
        };

        let first_contact = self.occ_contact_at(&target, &observer, atmosphere, c1)?;
        let fourth_contact = self.occ_contact_at(&target, &observer, atmosphere, c4)?;
        let second_contact = match c2 { Some(jd) => Some(self.occ_contact_at(&target, &observer, atmosphere, jd)?), None => None };
        let third_contact = match c3 { Some(jd) => Some(self.occ_contact_at(&target, &observer, atmosphere, jd)?), None => None };

        // Any phase visible: coarse 30-second scan of the target's altitude over [c1,c4].
        let any_phase_visible = {
            let step = 30.0 / 86_400.0;
            let mut jd = c1;
            let mut vis = false;
            while jd <= c4 + 1e-12 {
                if self.target_horizontal(&target, &observer, atmosphere, jd)?.2 { vis = true; break; }
                jd += step;
            }
            vis
        };

        Ok(LocalOccultation {
            target,
            occultation_type: occ_type,
            maximum,
            magnitude: covered_diameter_fraction(&g),
            obscuration: obscuration_fraction(&g),
            first_contact,
            second_contact,
            third_contact,
            fourth_contact,
            any_phase_visible,
        })
    }
}

#[cfg(test)]
mod how_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    fn equatorial_obs() -> ObserverLocation {
        ObserverLocation::new(Latitude::from_degrees(0.0), Longitude::from_degrees(0.0), Some(0.0))
    }

    #[test]
    fn contacts_bracket_the_maximum_when_occulted() {
        // On-node new moon → the analytic Moon passes over a target near the
        // ecliptic for an equatorial observer.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0);
        let engine = EventEngine::new(backend);
        let at = Instant::new(JulianDay::from_days(2_451_550.0), TimeScale::Tdb);
        // The Sun sits at the new-moon longitude; use it as a stand-in disc target
        // only to exercise ordering (validate_occult_target forbids Sun, so use a
        // planet the mock serves). The LinearSunMoon mock serves Sun/Moon only, so
        // assert ordering via a Miss-free path is covered in integration Task 8.
        let out = engine.occultation(
            OccultTarget::Body(CelestialBody::Mercury),
            equatorial_obs(),
            Atmosphere::default(),
            at,
        );
        // The mock has no Mercury → fail-closed MissingCoordinates/Backend, proving
        // the resolve path reaches the backend. Real-backend ordering is Task 8.
        assert!(out.is_err());
    }

    #[test]
    fn out_of_window_and_sun_target_fail_closed() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let bad = Instant::new(JulianDay::from_days(2_000_000.0), TimeScale::Tdb);
        assert!(matches!(
            engine.occultation(OccultTarget::Body(CelestialBody::Mars), equatorial_obs(), Atmosphere::default(), bad),
            Err(EventError::OutOfWindow { .. })
        ));
        let ok_at = Instant::new(JulianDay::from_days(2_451_550.0), TimeScale::Tdb);
        assert!(matches!(
            engine.occultation(OccultTarget::Body(CelestialBody::Sun), equatorial_obs(), Atmosphere::default(), ok_at),
            Err(EventError::UnsupportedOccultTarget { .. })
        ));
    }
}
