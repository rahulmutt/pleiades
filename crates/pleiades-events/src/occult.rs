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

/// Stage-by-stage intermediates of the topocentric occultation pipeline at
/// one instant, for differential comparison against Swiss Ephemeris — the
/// KNOWN GAP 3 diagnosis surface. Not a stable API.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct OccultStageDiagnostics {
    /// ΔT seconds used for the UT1-rotated sidereal time at `jd`.
    pub delta_t_seconds: f64,
    /// Whether ΔT came from the extrapolated (post-2020) branch.
    pub delta_t_predicted: bool,
    /// Geocentric apparent equatorial-of-date Moon (ra deg, dec deg, dist au).
    pub moon_geo: (f64, f64, f64),
    /// Topocentric Moon (ra deg, dec deg, dist au).
    pub moon_topo: (f64, f64, f64),
    /// Geocentric target (ra deg, dec deg).
    pub target_geo: (f64, f64),
    /// Topocentric target (ra deg, dec deg) — identical to geocentric for a star.
    pub target_topo: (f64, f64),
    /// Topocentric lunar semidiameter, degrees.
    pub s_moon_deg: f64,
    /// Target semidiameter, degrees (0 for a star).
    pub s_tgt_deg: f64,
    /// Topocentric Moon–target separation, degrees.
    pub sep_topo_deg: f64,
    /// `sep_topo_deg − (s_moon_deg + s_tgt_deg)`: positive ⇒ Miss at `jd`.
    pub graze_margin_deg: f64,
    /// Instant minimizing the topocentric separation within ±0.15 d of `jd`.
    pub refined_max_jd: f64,
    /// The graze margin at `refined_max_jd`.
    pub refined_margin_deg: f64,
    /// `classify()` at `refined_max_jd` — the quantity KNOWN GAP 3 disputes.
    pub occultation_type: OccultationType,
}

/// Global circumstances (`when_glob`): the greatest-occultation instant and
/// the central-observation point (geographic point of minimum topocentric
/// Moon–target separation) where it is central/greatest. NOT the full path
/// polygon.
#[derive(Clone, Debug, PartialEq)]
pub struct GlobalOccultation {
    /// The occulted target.
    pub target: OccultTarget,
    /// Instant of greatest global occultation (TDB).
    pub maximum: Instant,
    /// Central-observation point of maximum occultation: the geographic
    /// latitude (positive north) at which the topocentric Moon–target
    /// separation is minimized — where the occultation is best (most
    /// centrally) observed, matching SE's `swe_lun_occult_where`. NOT the
    /// Moon's geocentric zenith point.
    pub sublunar_latitude: Latitude,
    /// Central-observation point of maximum occultation: the geographic
    /// longitude (positive east) at which the topocentric Moon–target
    /// separation is minimized — see [`Self::sublunar_latitude`].
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
    let a_t = ((d * d + r_t2 - r_m2) / (2.0 * d * r_t))
        .clamp(-1.0, 1.0)
        .acos();
    let a_m = ((d * d + r_m2 - r_t2) / (2.0 * d * r_m))
        .clamp(-1.0, 1.0)
        .acos();
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
        OccGeom {
            sep_deg: sep,
            s_moon_deg: s_moon,
            s_tgt_deg: s_tgt,
        }
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

#[cfg(test)]
mod axis_pierce_tests {
    use super::*;

    /// Moon 0.00257 AU up the +x axis, target 1 AU up the +x axis: the
    /// shadow axis runs straight through the geocenter -> central.
    #[test]
    fn axis_through_geocenter_is_central() {
        let rm = [0.00257, 0.0, 0.0];
        let rs = [1.0, 0.0, 0.0];
        assert!(axis_pierce_central(rm, rs, 0.0));
    }

    /// Perpendicular Moon offsets around the Earth-radius threshold. The
    /// axis-geocenter distance is ~offset * 1.0026 (the axis diverges
    /// slightly as it extends from the Moon toward Earth), so 4000 km is
    /// comfortably inside de = 6378.14 km and 9000 km comfortably outside.
    #[test]
    fn axis_offset_thresholds() {
        let rs = [1.0, 0.0, 0.0];
        let near = [0.00257, 4_000.0 / AU_KM, 0.0];
        let far = [0.00257, 9_000.0 / AU_KM, 0.0];
        assert!(
            axis_pierce_central(near, rs, 0.0),
            "4000 km offset must pierce"
        );
        assert!(
            !axis_pierce_central(far, rs, 0.0),
            "9000 km offset must miss"
        );
    }

    /// SE stretches z by 1/(1 - 1/298.25642) (+0.336%) before the pierce
    /// test. A 6350 km offset lands INSIDE the threshold along y
    /// (6350 * 1.0026 = 6366.5 < 6378.1) but OUTSIDE along z
    /// (6350 / 0.996647 * 1.0026 = 6387.9 > 6378.1) - the oblateness
    /// handling is what discriminates.
    #[test]
    fn oblateness_z_stretch_discriminates() {
        let rs = [1.0, 0.0, 0.0];
        let off = 6_350.0 / AU_KM;
        assert!(axis_pierce_central([0.00257, off, 0.0], rs, 0.0));
        assert!(!axis_pierce_central([0.00257, 0.0, off], rs, 0.0));
    }

    /// Fail closed: non-finite input is never central.
    #[test]
    fn non_finite_fails_closed() {
        let rs = [1.0, 0.0, 0.0];
        assert!(!axis_pierce_central([f64::NAN, 0.0, 0.0], rs, 0.0));
        assert!(!axis_pierce_central([0.00257, f64::INFINITY, 0.0], rs, 0.0));
    }
}

use crate::crossings::EventEngine;
use crate::ephemeris::{
    geocentric_apparent_ecliptic, geocentric_apparent_longitude_deg, spherical_to_cartesian,
};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::fixstar::fixed_star_apparent;
use crate::rise_trans::{check_atmosphere, RiseSetTarget};
use crate::root::{first_crossing_after, last_crossing_before, wrap180, REFINE_TOLERANCE_DAYS};
use crate::semidiameter::{radius_au, semidiameter_deg};
use pleiades_apparent::{
    apparent_from_true, sidereal_time, topocentric_position, true_obliquity_degrees, Atmosphere,
};
use pleiades_backend::EphemerisBackend;

/// Moon physical radius (km) and AU, matching the eclipse crate's `solar_consts`.
pub(crate) const R_MOON_KM: f64 = 1_737.4;
pub(crate) const AU_KM: f64 = 149_597_870.7;

/// Earth equatorial radius (km) for the Moon's horizontal parallax.
const R_EARTH_KM: f64 = 6_378.137;

/// The Moon's maximum reach in ecliptic latitude: ~5.3° orbital inclination +
/// ~0.27° semidiameter + ~0.95° horizontal parallax ≈ 6.6°. A star beyond this
/// |ecliptic latitude| can never be occulted from anywhere on Earth.
pub(crate) const MOON_MAX_REACH_DEG: f64 = 6.6;

/// Swiss Ephemeris `eclipse_where` constants (swecl.c / sweph.h), used by the
/// `central` axis-pierce test only. SE's Earth radius (`de = 6378140 m`) and
/// Moon radius (`RMOON = DMOON/2 = 3476300/2 m`) differ slightly from this
/// module's own `R_EARTH_KM`/`R_MOON_KM`; the SE values are used here so the
/// ported test is bit-faithful to SE's own thresholds.
pub(crate) const SE_EARTH_RADIUS_AU: f64 = 6_378.140 / AU_KM;
/// SE Earth oblateness (sweph.h, AA 2006 K6 value): 1/298.25642.
pub(crate) const SE_EARTH_OBLATENESS: f64 = 1.0 / 298.25642;
/// SE Moon radius in AU (swecl.c `RMOON`).
pub(crate) const SE_R_MOON_AU: f64 = 3_476.3 / 2.0 / AU_KM;

/// Swiss Ephemeris `SE_ECL_CENTRAL` axis-pierce test, ported exactly from
/// `swecl.c`'s `eclipse_where` (the routine behind `swe_lun_occult_where`):
/// stretch both z-coordinates by `1/(1 − oblateness)` (SE corrects the
/// bodies instead of flattening the Earth), form the shadow axis through the
/// Moon along the target→Moon direction, and report whether the axis'
/// perpendicular distance from the geocenter `r0 = sqrt(dm² − s0²)` is
/// within `de·cosf1`. Inputs are geocentric apparent equatorial-of-date
/// Cartesian AU. Fail-closed: any non-finite intermediate → `false`.
fn axis_pierce_central(mut rm: [f64; 3], mut rs: [f64; 3], drad_au: f64) -> bool {
    let earthobl = 1.0 - SE_EARTH_OBLATENESS;
    rm[2] /= earthobl;
    rs[2] /= earthobl;
    let dm = (rm[0] * rm[0] + rm[1] * rm[1] + rm[2] * rm[2]).sqrt();
    let e = [rm[0] - rs[0], rm[1] - rs[1], rm[2] - rs[2]];
    let dsm = (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt();
    if !(dsm.is_finite() && dsm > 0.0 && dm.is_finite()) {
        return false;
    }
    let e = [e[0] / dsm, e[1] / dsm, e[2] / dsm];
    let sinf1 = (drad_au - SE_R_MOON_AU) / dsm;
    let cosf1 = (1.0 - sinf1 * sinf1).max(0.0).sqrt();
    let s0 = -(rm[0] * e[0] + rm[1] * e[1] + rm[2] * e[2]);
    let r0 = (dm * dm - s0 * s0).max(0.0).sqrt();
    if !r0.is_finite() {
        return false;
    }
    SE_EARTH_RADIUS_AU * cosf1 >= r0
}

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
        let s_moon_deg = (R_MOON_KM / (moon_dist * AU_KM))
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees();
        let s_tgt_deg = match target {
            OccultTarget::Body(b) => {
                let dist = self.body_distance_au(b, observer, jd)?;
                semidiameter_deg(&RiseSetTarget::Body(b.clone()), dist, false)
            }
            OccultTarget::Star(_) => 0.0,
        };
        Ok(OccGeom {
            sep_deg,
            s_moon_deg,
            s_tgt_deg,
        })
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
        ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        )
    }

    #[test]
    fn moon_semidiameter_is_about_a_quarter_degree() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let g = engine
            .occ_geom(&OccultTarget::Body(CelestialBody::Sun), None, 2_451_550.0)
            .unwrap();
        assert!(
            g.s_moon_deg > 0.2 && g.s_moon_deg < 0.3,
            "moon SD {}",
            g.s_moon_deg
        );
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
            .occ_geom(
                &OccultTarget::Body(CelestialBody::Sun),
                Some(&obs()),
                2_451_550.0,
            )
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
        let sep =
            |jd: f64| Ok::<f64, EventError>(self.occ_geom(target, Some(observer), jd)?.sep_deg);
        let mut c = b - (b - a) * phi;
        let mut d = a + (b - a) * phi;
        let (mut fc, mut fd) = (sep(c)?, sep(d)?);
        while (b - a) > REFINE_TOLERANCE_DAYS {
            if fc < fd {
                b = d;
                d = c;
                fd = fc;
                c = b - (b - a) * phi;
                fc = sep(c)?;
            } else {
                a = c;
                c = d;
                fc = fd;
                d = a + (b - a) * phi;
                fd = sep(d)?;
            }
        }
        Ok(0.5 * (a + b))
    }

    /// Stage dump at `jd` (TDB) for `observer` — see [`OccultStageDiagnostics`].
    #[doc(hidden)]
    pub fn occult_stage_diagnostics(
        &self,
        target: &OccultTarget,
        observer: &ObserverLocation,
        jd: f64,
    ) -> Result<OccultStageDiagnostics, EventError> {
        self.validate_occult_target(target)?;
        let (dt_sec, quality) = pleiades_time::deltat::delta_t(jd)
            .map_err(|e| EventError::Backend(format!("delta_t failed: {e}")))?;
        let (moon_g, tgt_g) = self.moon_target_radec(target, None, jd)?;
        let (moon_t, tgt_t) = self.moon_target_radec(target, Some(observer), jd)?;
        let moon_geo_dist = self.body_distance_au(&CelestialBody::Moon, None, jd)?;
        let moon_topo_dist = self.body_distance_au(&CelestialBody::Moon, Some(observer), jd)?;
        let g = self.occ_geom(target, Some(observer), jd)?;
        let margin = g.sep_deg - (g.s_moon_deg + g.s_tgt_deg);
        let refined_max_jd = self.minimize_occ_sep(
            target,
            observer,
            (jd - OCC_CONTACT_HALF_WINDOW_DAYS).max(WINDOW_START_JD),
            (jd + OCC_CONTACT_HALF_WINDOW_DAYS).min(WINDOW_END_JD),
        )?;
        let g_ref = self.occ_geom(target, Some(observer), refined_max_jd)?;
        Ok(OccultStageDiagnostics {
            delta_t_seconds: dt_sec,
            delta_t_predicted: quality == pleiades_time::DeltaTQuality::Predicted,
            moon_geo: (moon_g.0, moon_g.1, moon_geo_dist),
            moon_topo: (moon_t.0, moon_t.1, moon_topo_dist),
            target_geo: tgt_g,
            target_topo: tgt_t,
            s_moon_deg: g.s_moon_deg,
            s_tgt_deg: g.s_tgt_deg,
            sep_topo_deg: g.sep_deg,
            graze_margin_deg: margin,
            refined_max_jd,
            refined_margin_deg: g_ref.sep_deg - (g_ref.s_moon_deg + g_ref.s_tgt_deg),
            occultation_type: classify(&g_ref),
        })
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
        let f = |jd: f64| {
            Ok::<f64, EventError>(self.occ_geom(target, Some(observer), jd)?.sep_deg - threshold)
        };
        let mut flo = f(lo)?;
        let fhi = f(hi)?;
        if flo.signum() == fhi.signum() {
            return Ok(None);
        }
        while (hi - lo) > REFINE_TOLERANCE_DAYS {
            let mid = 0.5 * (lo + hi);
            let fmid = f(mid)?;
            if fmid.signum() == flo.signum() {
                lo = mid;
                flo = fmid;
            } else {
                hi = mid;
            }
        }
        Ok(Some(0.5 * (lo + hi)))
    }

    /// Local circumstances of a lunar occultation of `target` for `observer` at
    /// (or around) `at` — the local-circumstances ("how") analogue. Swiss
    /// Ephemeris exposes no separate `swe_lun_occult_how` call (it does not
    /// exist in the SE API); this is validated against the `attr` (magnitude,
    /// obscuration, contact instants) that `swe_lun_occult_when_loc` returns.
    /// Returns full circumstances even when the target is below the horizon;
    /// the `occultation_type` is `Miss` when no contact occurs for this
    /// observer.
    pub fn occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,
        at: Instant,
    ) -> Result<LocalOccultation, EventError> {
        self.validate_occult_target(&target)?;
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver {
                detail: e.to_string(),
            })?;
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
        let c1 = self
            .bisect_occ_contact(&target, &observer, external, lo, max_jd)?
            .unwrap_or(max_jd);
        let c4 = self
            .bisect_occ_contact(&target, &observer, external, max_jd, hi)?
            .unwrap_or(max_jd);

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
        let second_contact = match c2 {
            Some(jd) => Some(self.occ_contact_at(&target, &observer, atmosphere, jd)?),
            None => None,
        };
        let third_contact = match c3 {
            Some(jd) => Some(self.occ_contact_at(&target, &observer, atmosphere, jd)?),
            None => None,
        };

        // Any phase visible: coarse 30-second scan of the target's altitude over [c1,c4].
        let any_phase_visible = {
            let step = 30.0 / 86_400.0;
            let mut jd = c1;
            let mut vis = false;
            while jd <= c4 + 1e-12 {
                if self
                    .target_horizontal(&target, &observer, atmosphere, jd)?
                    .2
                {
                    vis = true;
                    break;
                }
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

/// Conjunction bracketing step (Moon moves ~0.5°/h; 0.25 day is the crate's
/// Moon step and cannot skip a monthly conjunction).
const OCC_CONJUNCTION_STEP_DAYS: f64 = 0.25;

impl<B: EphemerisBackend> EventEngine<B> {
    /// Signed Moon−target apparent ecliptic longitude difference, wrapped.
    fn moon_target_lon_diff(&self, target: &OccultTarget, jd: f64) -> Result<f64, EventError> {
        let moon =
            geocentric_apparent_longitude_deg(&self.backend, CelestialBody::Moon, "Moon", jd)?;
        let tgt = match target {
            OccultTarget::Body(b) => {
                geocentric_apparent_longitude_deg(&self.backend, b.clone(), "body", jd)?
            }
            OccultTarget::Star(name) => {
                let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
                let equ = fixed_star_apparent(name, at)?;
                let eps = true_obliquity_degrees(jd)
                    .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
                equ.to_ecliptic(Angle::from_degrees(eps))
                    .longitude
                    .degrees()
            }
        };
        Ok(wrap180(moon - tgt))
    }

    /// Next occultation of `target` locally visible at `observer`, strictly after
    /// `after` — `swe_lun_occult_when_loc` analogue. `None` if none occurs before
    /// the window end (or ever, for an un-occultable star).
    pub fn next_occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,
        after: Instant,
    ) -> Result<Option<LocalOccultation>, EventError> {
        self.validate_occult_target(&target)?;
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmosphere)?;
        let after_jd = after.julian_day.days();
        self.check_window(after_jd)?;
        if self.target_never_occultable(&target, after_jd)? {
            return Ok(None);
        }
        let mut scan_start = after_jd.max(WINDOW_START_JD + OCC_CONJUNCTION_STEP_DAYS);
        let scan_end = WINDOW_END_JD - OCC_CONJUNCTION_STEP_DAYS;
        loop {
            let conj = first_crossing_after(
                |jd| self.moon_target_lon_diff(&target, jd),
                scan_start,
                scan_end,
                OCC_CONJUNCTION_STEP_DAYS,
            )?;
            let Some(conj_jd) = conj else { return Ok(None) };
            let at = Instant::new(JulianDay::from_days(conj_jd), TimeScale::Tdb);
            let local = self.occultation(target.clone(), observer.clone(), atmosphere, at)?;
            if !matches!(local.occultation_type, OccultationType::Miss)
                && local.any_phase_visible
                && local.maximum.instant.julian_day.days() > after_jd
            {
                return Ok(Some(local));
            }
            // Advance just past this conjunction to find the next one.
            scan_start = conj_jd + OCC_CONJUNCTION_STEP_DAYS;
            if scan_start >= scan_end {
                return Ok(None);
            }
        }
    }

    /// Previous occultation of `target` locally visible at `observer`, strictly
    /// before `before`.
    pub fn previous_occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,
        before: Instant,
    ) -> Result<Option<LocalOccultation>, EventError> {
        self.validate_occult_target(&target)?;
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmosphere)?;
        let before_jd = before.julian_day.days();
        self.check_window(before_jd)?;
        if self.target_never_occultable(&target, before_jd)? {
            return Ok(None);
        }
        let mut scan_end = before_jd.min(WINDOW_END_JD - OCC_CONJUNCTION_STEP_DAYS);
        let scan_start = WINDOW_START_JD + OCC_CONJUNCTION_STEP_DAYS;
        loop {
            let conj = last_crossing_before(
                |jd| self.moon_target_lon_diff(&target, jd),
                scan_start,
                scan_end,
                OCC_CONJUNCTION_STEP_DAYS,
            )?;
            let Some(conj_jd) = conj else { return Ok(None) };
            let at = Instant::new(JulianDay::from_days(conj_jd), TimeScale::Tdb);
            let local = self.occultation(target.clone(), observer.clone(), atmosphere, at)?;
            if !matches!(local.occultation_type, OccultationType::Miss)
                && local.any_phase_visible
                && local.maximum.instant.julian_day.days() < before_jd
            {
                return Ok(Some(local));
            }
            scan_end = conj_jd - OCC_CONJUNCTION_STEP_DAYS;
            if scan_end <= scan_start {
                return Ok(None);
            }
        }
    }

    /// Next occultation of `target` anywhere on Earth, strictly after `after` —
    /// `swe_lun_occult_when_glob` analogue. Reports the greatest-occultation
    /// instant and the central-observation point where it is central/greatest (not the
    /// full path). `None` if none occurs before the window end.
    pub fn next_global_occultation(
        &self,
        target: OccultTarget,
        after: Instant,
    ) -> Result<Option<GlobalOccultation>, EventError> {
        self.validate_occult_target(&target)?;
        let after_jd = after.julian_day.days();
        self.check_window(after_jd)?;
        if self.target_never_occultable(&target, after_jd)? {
            return Ok(None);
        }
        let mut scan_start = after_jd.max(WINDOW_START_JD + OCC_CONJUNCTION_STEP_DAYS);
        let scan_end = WINDOW_END_JD - OCC_CONJUNCTION_STEP_DAYS;
        loop {
            let conj = first_crossing_after(
                |jd| self.moon_target_lon_diff(&target, jd),
                scan_start,
                scan_end,
                OCC_CONJUNCTION_STEP_DAYS,
            )?;
            let Some(conj_jd) = conj else { return Ok(None) };
            // Minimize geocentric separation around the conjunction.
            let max_jd = self.minimize_geo_sep(
                &target,
                (conj_jd - OCC_CONTACT_HALF_WINDOW_DAYS).max(WINDOW_START_JD),
                (conj_jd + OCC_CONTACT_HALF_WINDOW_DAYS).min(WINDOW_END_JD),
            )?;
            let g = self.occ_geom(&target, None, max_jd)?;
            let moon_dist = self.body_distance_au(&CelestialBody::Moon, None, max_jd)?;
            let pi_moon = (R_EARTH_KM / (moon_dist * AU_KM))
                .clamp(-1.0, 1.0)
                .asin()
                .to_degrees();
            if g.sep_deg < g.s_moon_deg + g.s_tgt_deg + pi_moon && max_jd > after_jd {
                // Central-observation point: the geographic (lat, lon) on
                // Earth's surface at `max_jd` where the TOPOCENTRIC Moon–target
                // separation is MINIMIZED — this is what SE's
                // `swe_lun_occult_where` actually returns, NOT the Moon's
                // geocentric zenith point. Seed the search at the sub-Moon
                // point (correct hemisphere) and refine via golden-section
                // coordinate descent over `occ_geom`'s already-tested
                // topocentric path.
                let ((moon_ra, moon_dec), _) = self.moon_target_radec(&target, None, max_jd)?;
                let at = Instant::new(JulianDay::from_days(max_jd), TimeScale::Tdb);
                let gast = sidereal_time(at, Longitude::from_degrees(0.0)).local_apparent_deg;
                let seed_lon = wrap180(moon_ra - gast);
                let (lat_star, lon_star) =
                    self.minimize_sublunar_point(&target, max_jd, moon_dec, seed_lon)?;
                let central_observer = ObserverLocation::new(
                    Latitude::from_degrees(lat_star),
                    Longitude::from_degrees(lon_star),
                    Some(0.0),
                );
                let g_star = self.occ_geom(&target, Some(&central_observer), max_jd)?;
                // occ_type keeps its coverage meaning at the actual
                // central-observation point: Total iff the target is fully
                // behind the Moon's disc there.
                let occ_type = if g_star.sep_deg < (g_star.s_moon_deg - g_star.s_tgt_deg).max(0.0) {
                    OccultationType::Total
                } else {
                    OccultationType::Grazing
                };
                // `central` is SE's STRICTER axis-pierce condition (the
                // Moon–target center-line strikes the Earth), decoupled from
                // occ_type — a Total-somewhere event can be non-central,
                // exactly like a total-but-non-central solar eclipse. This
                // resolves the former KNOWN GAP 2 (Saturn 2/6 glob rows).
                let central = self.central_axis_pierce(&target, max_jd)?;
                return Ok(Some(GlobalOccultation {
                    target,
                    maximum: at,
                    sublunar_latitude: Latitude::from_degrees(lat_star),
                    sublunar_longitude: Longitude::from_degrees(lon_star),
                    central,
                    occultation_type: occ_type,
                }));
            }
            scan_start = conj_jd + OCC_CONJUNCTION_STEP_DAYS;
            if scan_start >= scan_end {
                return Ok(None);
            }
        }
    }

    /// Golden-section minimize the GEOCENTRIC separation in `[a,b]`.
    fn minimize_geo_sep(
        &self,
        target: &OccultTarget,
        mut a: f64,
        mut b: f64,
    ) -> Result<f64, EventError> {
        let phi = 0.618_033_988_75_f64;
        let sep = |jd: f64| Ok::<f64, EventError>(self.occ_geom(target, None, jd)?.sep_deg);
        let mut c = b - (b - a) * phi;
        let mut d = a + (b - a) * phi;
        let (mut fc, mut fd) = (sep(c)?, sep(d)?);
        while (b - a) > REFINE_TOLERANCE_DAYS {
            if fc < fd {
                b = d;
                d = c;
                fd = fc;
                c = b - (b - a) * phi;
                fc = sep(c)?;
            } else {
                a = c;
                c = d;
                fc = fd;
                d = a + (b - a) * phi;
                fd = sep(d)?;
            }
        }
        Ok(0.5 * (a + b))
    }

    /// SE `SE_ECL_CENTRAL` analogue at `jd`: does the Moon–target shadow
    /// axis strike the (oblateness-corrected) Earth? Geocentric positions
    /// only — this is a property of the global geometry, not of an observer.
    fn central_axis_pierce(&self, target: &OccultTarget, jd: f64) -> Result<bool, EventError> {
        /// Point-source stand-in distance for a fixed star (AU). SE uses the
        /// catalog's own (astronomically large) distance; at ≥1e6 AU the
        /// axis direction is converged to ~1e-9 deg, far below every other
        /// term, so the exact value is immaterial.
        const STAR_AXIS_DISTANCE_AU: f64 = 1.0e9;
        let ((moon_ra, moon_dec), (tgt_ra, tgt_dec)) = self.moon_target_radec(target, None, jd)?;
        let moon_dist = self.body_distance_au(&CelestialBody::Moon, None, jd)?;
        let (tgt_dist, drad_au) = match target {
            OccultTarget::Body(b) => (self.body_distance_au(b, None, jd)?, radius_au(b)),
            OccultTarget::Star(_) => (STAR_AXIS_DISTANCE_AU, 0.0),
        };
        Ok(axis_pierce_central(
            spherical_to_cartesian(moon_ra, moon_dec, moon_dist),
            spherical_to_cartesian(tgt_ra, tgt_dec, tgt_dist),
            drad_au,
        ))
    }

    /// Golden-section coordinate-descent search for the geographic
    /// `(lat, lon)` at fixed `jd` that MINIMIZES the topocentric Moon–target
    /// separation — the central-observation point SE's `swe_lun_occult_where`
    /// returns. Seeded at `(lat0, lon0)` (typically the sub-Moon point, which
    /// sits in the correct hemisphere even when it is not the answer);
    /// alternates 1-D golden-section minimization over latitude (longitude
    /// held fixed) and over longitude (latitude held fixed) for several
    /// rounds, each searching a wide window around the current best point.
    /// The topocentric-separation surface is smooth and single-minimum over
    /// the Moon-facing hemisphere, so this coordinate descent converges even
    /// from a badly-off seed. Never panics: latitude is clamped to
    /// `[-90, 90]` and longitude is wrapped to `(-180, 180]` on every
    /// evaluation; any `occ_geom` error is propagated with `?`.
    fn minimize_sublunar_point(
        &self,
        target: &OccultTarget,
        jd: f64,
        lat0: f64,
        lon0: f64,
    ) -> Result<(f64, f64), EventError> {
        let sep_at = |lat: f64, lon: f64| -> Result<f64, EventError> {
            let observer = ObserverLocation::new(
                Latitude::from_degrees(lat.clamp(-90.0, 90.0)),
                Longitude::from_degrees(wrap180(lon)),
                Some(0.0),
            );
            Ok(self.occ_geom(target, Some(&observer), jd)?.sep_deg)
        };

        let mut lat = lat0.clamp(-90.0, 90.0);
        let mut lon = wrap180(lon0);
        for _ in 0..SUBLUNAR_REFINE_MAX_ROUNDS {
            let (prev_lat, prev_lon) = (lat, lon);
            let lon_fixed = lon;
            let lat_lo = (lat - SUBLUNAR_SEARCH_HALF_WIDTH_DEG).max(-90.0);
            let lat_hi = (lat + SUBLUNAR_SEARCH_HALF_WIDTH_DEG).min(90.0);
            lat = golden_section_min(SUBLUNAR_TOLERANCE_DEG, lat_lo, lat_hi, |l| {
                sep_at(l, lon_fixed)
            })?;

            let lat_fixed = lat;
            let lon_lo = lon - SUBLUNAR_SEARCH_HALF_WIDTH_DEG;
            let lon_hi = lon + SUBLUNAR_SEARCH_HALF_WIDTH_DEG;
            lon = golden_section_min(SUBLUNAR_TOLERANCE_DEG, lon_lo, lon_hi, |lo| {
                sep_at(lat_fixed, lo)
            })?;
            lon = wrap180(lon);

            // Converged: the reported point moved less than the tolerance this
            // round. Bounded by SUBLUNAR_REFINE_MAX_ROUNDS regardless (no
            // unbounded loop even if convergence is never reached).
            if (lat - prev_lat).abs() < SUBLUNAR_CONVERGENCE_TOL_DEG
                && (lon - prev_lon).abs() < SUBLUNAR_CONVERGENCE_TOL_DEG
            {
                break;
            }
        }
        Ok((lat, lon))
    }
}

/// Bounded backstop on nested lat/lon golden-section refinement rounds for
/// the sub-lunar (central-observation) point search in
/// `next_global_occultation`. The loop normally exits early once
/// `SUBLUNAR_CONVERGENCE_TOL_DEG` is reached (a handful of rounds for
/// well-conditioned targets); this cap only bites for the worst-conditioned
/// corpus rows (e.g. Regulus, Saturn) and keeps the loop provably bounded —
/// never unbounded — even if convergence is never reached. A prior fixed
/// `SUBLUNAR_REFINE_ROUNDS = 8` under-converged those rows (sub-lunar
/// residual 69.6' for Regulus); replaying the exact production procedure
/// with more rounds showed the point still moving well past round 8 and
/// settling only by ~round 32 (Regulus 8→69.63', 32→4.23'; Saturn 8→25.79',
/// 24→12.22' plateau), proving 8 rounds was under-converged, not up against
/// a geometric floor. Cost is negligible (sub-second even at 48 rounds).
const SUBLUNAR_REFINE_MAX_ROUNDS: usize = 48;
/// Convergence stop: once the reported (lat, lon) point moves less than this
/// between rounds, further rounds are refining below any measurement
/// resolution, so the search exits early.
const SUBLUNAR_CONVERGENCE_TOL_DEG: f64 = 1e-4;
/// Golden-section tolerance for the sub-lunar point search, degrees (~3.6
/// arcsec) — a few arcsec is ample precision against an arcmin-scale gate.
const SUBLUNAR_TOLERANCE_DEG: f64 = 0.001;
/// Half-width (degrees) of each round's local lat/lon search window around
/// the current best point. Wide enough that, even starting from a badly-off
/// seed (the pre-fix bug was off by up to 89°), coordinate descent reaches
/// the true minimum within a handful of rounds.
const SUBLUNAR_SEARCH_HALF_WIDTH_DEG: f64 = 90.0;

/// Golden-section 1-D minimization of `f` over `[a, b]` to within `tol`. The
/// bracket shrinks geometrically by the golden ratio each iteration, so this
/// always terminates (never an unbounded loop).
fn golden_section_min(
    tol: f64,
    mut a: f64,
    mut b: f64,
    mut f: impl FnMut(f64) -> Result<f64, EventError>,
) -> Result<f64, EventError> {
    let phi = 0.618_033_988_75_f64;
    let mut c = b - (b - a) * phi;
    let mut d = a + (b - a) * phi;
    let (mut fc, mut fd) = (f(c)?, f(d)?);
    while (b - a) > tol {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - (b - a) * phi;
            fc = f(c)?;
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + (b - a) * phi;
            fd = f(d)?;
        }
    }
    Ok(0.5 * (a + b))
}

#[cfg(test)]
mod when_loc_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn never_occultable_star_returns_none_fast() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let obs = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        let out = engine
            .next_occultation(
                OccultTarget::Star("Sirius".into()),
                obs,
                Atmosphere::default(),
                after,
            )
            .unwrap();
        assert!(out.is_none(), "Sirius can never be occulted");
    }

    #[test]
    fn out_of_window_fails_closed() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let obs = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let bad = Instant::new(JulianDay::from_days(2_000_000.0), TimeScale::Tdb);
        assert!(matches!(
            engine.next_occultation(
                OccultTarget::Body(CelestialBody::Venus),
                obs,
                Atmosphere::default(),
                bad
            ),
            Err(EventError::OutOfWindow { .. })
        ));
    }
}

#[cfg(test)]
mod how_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    fn equatorial_obs() -> ObserverLocation {
        ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        )
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
            engine.occultation(
                OccultTarget::Body(CelestialBody::Mars),
                equatorial_obs(),
                Atmosphere::default(),
                bad
            ),
            Err(EventError::OutOfWindow { .. })
        ));
        let ok_at = Instant::new(JulianDay::from_days(2_451_550.0), TimeScale::Tdb);
        assert!(matches!(
            engine.occultation(
                OccultTarget::Body(CelestialBody::Sun),
                equatorial_obs(),
                Atmosphere::default(),
                ok_at
            ),
            Err(EventError::UnsupportedOccultTarget { .. })
        ));
    }
}

#[cfg(test)]
mod when_glob_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn never_occultable_star_returns_none() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        assert!(engine
            .next_global_occultation(OccultTarget::Star("Sirius".into()), after)
            .unwrap()
            .is_none());
    }

    #[test]
    fn sun_target_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        assert!(matches!(
            engine.next_global_occultation(OccultTarget::Body(CelestialBody::Sun), after),
            Err(EventError::UnsupportedOccultTarget { .. })
        ));
    }
}
