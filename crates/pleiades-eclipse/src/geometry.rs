//! Geocentric eclipse shadow-cone geometry and type classification.

use crate::ephemeris::SunMoonSample;
use crate::syzygy::Syzygy;
use crate::types::{LunarEclipseType, SolarEclipseType};

pub(crate) mod constants {
    pub const R_SUN_KM: f64 = 696_000.0;
    pub const R_MOON_KM: f64 = 1_737.4;
    pub const R_EARTH_KM: f64 = 6_378.137;
    pub const AU_KM: f64 = 149_597_870.7;
    /// Enlargement factor for Earth's umbral/penumbral shadow cone, modelling
    /// atmospheric extinction in the same way the NASA lunar canon does.
    ///
    /// Applied **only** to Earth's angular size (π_moon + π_sun); NOT to the
    /// Sun's semidiameter `s`, which sets the spread of the umbra/penumbra:
    ///   `earth_shadow = SHADOW_INFLATION · (π_moon + π_sun)`
    ///   `u = earth_shadow − s`  (umbral radius)
    ///   `p = earth_shadow + s`  (penumbral radius)
    ///
    /// The value 1.01 was empirically matched to the NASA Five Millennium Lunar
    /// Eclipse Canon (Espenak & Meeus).  A back-solve across the canon's
    /// penumbral magnitudes gives an implied factor of 1.00992–1.01013 (mean
    /// ≈ 1.010), confirming 1.01 as the correct value.
    ///
    /// **Do not change this to 1.02.**  Using 1.02 (or applying the factor to
    /// `(π+π±s)` rather than just `(π+π)`) biases the penumbral magnitude
    /// ~+0.028 high across the full canon and mis-classifies several near-
    /// boundary penumbrals as partials.
    pub const SHADOW_INFLATION: f64 = 1.01;
}

use constants::*;

/// Limiting |γ| (axis distance from Earth's center, equatorial radii) for a
/// central (total/annular) eclipse to reach the surface. The classic 0.9972
/// value accounts for Earth's polar flattening; above it the axis misses the
/// ellipsoid and the greatest eclipse is grazing/partial.
const CENTRAL_GAMMA_LIMIT: f64 = 0.9972;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SolarCircumstances {
    pub eclipse_type: SolarEclipseType,
    pub magnitude: f64,
    pub gamma: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LunarCircumstances {
    pub eclipse_type: LunarEclipseType,
    pub magnitude: f64,
    pub gamma: f64,
}

/// Great-circle separation (radians) between Sun and Moon centers.
fn separation_rad(sample: &SunMoonSample) -> f64 {
    let (l1, b1) = (
        sample.sun_longitude_deg.to_radians(),
        sample.sun_latitude_deg.to_radians(),
    );
    let (l2, b2) = (
        sample.moon_longitude_deg.to_radians(),
        sample.moon_latitude_deg.to_radians(),
    );
    let cos_sep = (b1.sin() * b2.sin() + b1.cos() * b2.cos() * (l1 - l2).cos()).clamp(-1.0, 1.0);
    cos_sep.acos()
}

/// Classifies a solar eclipse from a new-moon sample.
///
/// NASA's published solar-eclipse magnitude and total/annular/hybrid type are
/// **topocentric**, evaluated at the point of greatest eclipse on Earth's
/// surface — not geocentric. The geocentric quantities (`s`, `m`, `parallax`,
/// `sigma`) determine *whether* and *where* an eclipse occurs; the magnitude and
/// type are then taken at the sub-shadow surface point, where the Moon is roughly
/// one Earth radius nearer the observer (so its apparent semidiameter `m_topo` is
/// larger) and the shadow axis passes through the observer (so the topocentric
/// Sun–Moon separation `sigma_topo` is ~0 for central eclipses).
///
/// * Central eclipse (axis meets Earth, |γ| ≤ 1): the greatest-eclipse observer
///   sits on the axis, ~`R_earth·sqrt(1-γ²)` nearer the Moon. With Sun and Moon
///   centers coincident there, the magnitude is the ratio of apparent diameters
///   `m_topo / s`. Type is **Total** if the Moon's disk also covers the Sun
///   *geocentrically* (`m_geo ≥ s`); **Hybrid** if it covers only after the
///   topocentric enlargement tips it over (`m_geo < s ≤ m_topo`, i.e. annular at
///   the path ends but total near the sub-solar point — the defining geometry of
///   an annular-total eclipse); **Annular** otherwise (`m_topo < s`).
/// * Partial only (axis misses Earth, |γ| > 1): the greatest-eclipse observer is
///   the surface point closest to the axis, at perpendicular distance
///   `(|γ|-1)·R_earth`. The topocentric separation there is
///   `sigma_topo = (|γ|-1)·R_earth·(1/d_topo − 1/d_sun)`, and the magnitude is the
///   penetration fraction `(s + m_topo − sigma_topo)/(2s)`.
pub(crate) fn classify_solar(sample: &SunMoonSample) -> Option<SolarCircumstances> {
    let sun_dist_km = sample.sun_distance_au * AU_KM;
    let moon_dist_km = sample.moon_distance_au * AU_KM;
    let s = (R_SUN_KM / sun_dist_km).asin();
    let m_geo = (R_MOON_KM / moon_dist_km).asin();
    let parallax = (R_EARTH_KM / moon_dist_km).asin();
    let sigma = separation_rad(sample);

    if sigma >= parallax + s + m_geo {
        return None; // penumbra misses Earth — no eclipse
    }

    let gamma = (sigma / parallax) * sample.moon_latitude_deg.signum();

    // Exact topocentric geometry at the greatest-eclipse surface point. Build
    // geocentric rectangular vectors (AU); the shadow axis is the Sun→Moon line.
    let s_vec = rect(
        sample.sun_longitude_deg,
        sample.sun_latitude_deg,
        sample.sun_distance_au,
    );
    let m_vec = rect(
        sample.moon_longitude_deg,
        sample.moon_latitude_deg,
        sample.moon_distance_au,
    );
    let r_earth_au = R_EARTH_KM / AU_KM;

    // Spherical axis miss-distance (linear γ, equatorial radii) — used only to
    // choose the magnitude regime (central diameter-ratio vs grazing penetration).
    // Keeping this on the sphere makes the regime boundary stable; the observer
    // geometry below uses the true ellipsoid.
    let gamma_linear = {
        let ax = {
            let d = sub(m_vec, s_vec);
            scale(d, 1.0 / norm(d))
        };
        norm(sub(m_vec, scale(ax, dot(m_vec, ax)))) / r_earth_au
    };

    // The greatest-eclipse surface point lies on Earth's true (oblate) ellipsoid.
    // Working in an equatorial frame whose polar axis is stretched by 1/(1-f)
    // turns the ellipsoid into a sphere of equatorial radius, so the spherical
    // axis/closest-point construction applies; the observer is then mapped back.
    // Oblateness matters at the ~3″ level for high-latitude grazing eclipses,
    // which decides annular-vs-partial / annular-vs-hybrid at the boundary.
    let sf = flatten_to_sphere(s_vec);
    let mf = flatten_to_sphere(m_vec);
    let axis = {
        let d = sub(mf, sf);
        scale(d, 1.0 / norm(d))
    };
    // Foot of the perpendicular from Earth's center onto the axis (flattened
    // frame); its distance is the linear analogue of γ in equatorial radii.
    let foot = sub(mf, scale(axis, dot(mf, axis)));
    let perp = norm(foot);

    let observer_flat = if perp < r_earth_au {
        // Axis pierces Earth: the greatest-eclipse observer sits on the axis on
        // the Sun-facing (day) side, where the eclipse is actually visible.
        // |foot ± t·axis| = R_earth with foot ⟂ axis ⇒ t = sqrt(R_earth² − perp²);
        // pick the intersection nearer the Sun (the axis points anti-solar).
        let t = (r_earth_au * r_earth_au - perp * perp).sqrt();
        let cand_a = add(foot, scale(axis, t));
        let cand_b = sub(foot, scale(axis, t));
        if norm(sub(sf, cand_a)) < norm(sub(sf, cand_b)) {
            cand_a
        } else {
            cand_b
        }
    } else {
        // Axis misses Earth's ellipsoid: closest surface point lies along the
        // perpendicular. (The umbra/antumbra may still reach the surface here for
        // a grazing total/annular eclipse — the disk-overlap test below decides.)
        scale(foot, r_earth_au / perp)
    };
    let observer = unflatten_from_sphere(observer_flat);

    let to_moon = sub(m_vec, observer);
    let to_sun = sub(s_vec, observer);
    let d_moon_topo = norm(to_moon);
    let d_sun_topo = norm(to_sun);
    let m_topo = ((R_MOON_KM / AU_KM) / d_moon_topo).asin();
    let s_topo = ((R_SUN_KM / AU_KM) / d_sun_topo).asin();
    let cos_sep = (dot(to_moon, to_sun) / (d_moon_topo * d_sun_topo)).clamp(-1.0, 1.0);
    let sigma_topo = cos_sep.acos();

    // NASA reports the magnitude at the greatest-eclipse point on Earth's surface.
    // When the shadow axis truly reaches the surface (|γ| < CENTRAL_GAMMA_LIMIT,
    // the classic central limit set by Earth's flattening) the Sun and Moon centers
    // coincide there and NASA's magnitude is the ratio of apparent diameters. When
    // the axis only grazes or misses (|γ| ≥ limit) the greatest-eclipse point sees
    // a partial overlap, and NASA's magnitude is the covered-diameter fraction
    // (penetration). The type, however, is still total/annular when the umbra or
    // antumbra reaches the surface (the disks fully overlap at the observer).
    let (eclipse_type, magnitude) = if gamma_linear < CENTRAL_GAMMA_LIMIT {
        // Central: diameter ratio. Hybrid when only the topocentric enlargement
        // near the sub-solar point tips an otherwise-annular eclipse to total
        // (annular geocentrically, total topocentrically — annular at the path ends).
        let mag = m_topo / s_topo;
        let etype = if m_topo >= s_topo {
            if m_geo >= s {
                SolarEclipseType::Total
            } else {
                SolarEclipseType::Hybrid
            }
        } else {
            SolarEclipseType::Annular
        };
        (etype, mag)
    } else {
        // Grazing/non-central: covered-diameter fraction at the surface point.
        let mag = ((s_topo + m_topo - sigma_topo) / (2.0 * s_topo)).max(0.0);
        let etype = if sigma_topo + s_topo <= m_topo {
            SolarEclipseType::Total
        } else if sigma_topo + m_topo <= s_topo {
            SolarEclipseType::Annular
        } else {
            SolarEclipseType::Partial
        };
        (etype, mag)
    };

    Some(SolarCircumstances {
        eclipse_type,
        magnitude,
        gamma,
    })
}

/// Separation (radians) of the Moon from the antisolar (shadow-axis) point.
fn shadow_axis_separation_rad(sample: &SunMoonSample) -> f64 {
    let anti_lon = (sample.sun_longitude_deg + 180.0).rem_euclid(360.0);
    let anti = SunMoonSample {
        sun_longitude_deg: anti_lon,
        sun_latitude_deg: -sample.sun_latitude_deg,
        ..*sample
    };
    // Reuse the Sun↔Moon separation routine against the antisolar point.
    separation_rad(&anti)
}

pub(crate) fn classify_lunar(sample: &SunMoonSample) -> Option<LunarCircumstances> {
    let s = (R_SUN_KM / (sample.sun_distance_au * AU_KM)).asin();
    let m_moon = (R_MOON_KM / (sample.moon_distance_au * AU_KM)).asin();
    let pi_moon = (R_EARTH_KM / (sample.moon_distance_au * AU_KM)).asin();
    let pi_sun = (R_EARTH_KM / (sample.sun_distance_au * AU_KM)).asin();

    // Earth's geometric shadow is enlarged by SHADOW_INFLATION (atmospheric
    // extinction, the value the NASA canon uses) — applied to Earth's angular
    // size (π_moon + π_sun) only, NOT to the Sun's semidiameter s, which sets the
    // umbra/penumbra spread. (Enlarging s too, as `INFLATION·(π+π±s)` would,
    // biases the penumbral magnitude ~+0.028 high.)
    let earth_shadow = SHADOW_INFLATION * (pi_moon + pi_sun);
    let u = earth_shadow - s;
    let p = earth_shadow + s;
    let sigma = shadow_axis_separation_rad(sample);

    if sigma >= p + m_moon {
        return None;
    }

    let gamma = (sigma / pi_moon) * sample.moon_latitude_deg.signum();
    let (eclipse_type, magnitude) = if sigma + m_moon <= u {
        (
            LunarEclipseType::Total,
            (u + m_moon - sigma) / (2.0 * m_moon),
        )
    } else if sigma - m_moon < u {
        (
            LunarEclipseType::Partial,
            (u + m_moon - sigma) / (2.0 * m_moon),
        )
    } else {
        (
            LunarEclipseType::Penumbral,
            (p + m_moon - sigma) / (2.0 * m_moon),
        )
    };

    Some(LunarCircumstances {
        eclipse_type,
        magnitude,
        gamma,
    })
}

/// Geocentric rectangular position (AU) from ecliptic longitude/latitude/distance.
fn rect(lon_deg: f64, lat_deg: f64, dist_au: f64) -> [f64; 3] {
    let l = lon_deg.to_radians();
    let b = lat_deg.to_radians();
    [
        dist_au * b.cos() * l.cos(),
        dist_au * b.cos() * l.sin(),
        dist_au * b.sin(),
    ]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// WGS-84 flattening of Earth (polar radius = equatorial·(1 − f)).
const EARTH_FLATTENING: f64 = 1.0 / 298.257_223_563;
/// Mean obliquity of the ecliptic (J2000), radians. Its ~0.013°/century drift is
/// negligible for orienting the flattening (a ~0.3% geometric correction).
const OBLIQUITY_RAD: f64 = 0.409_092_804_222_329; // 23.439291°

/// Rotate an ecliptic vector to the equatorial frame (about the +x/equinox axis).
fn ecliptic_to_equatorial(v: [f64; 3]) -> [f64; 3] {
    let (c, s) = (OBLIQUITY_RAD.cos(), OBLIQUITY_RAD.sin());
    [v[0], v[1] * c - v[2] * s, v[1] * s + v[2] * c]
}

/// Inverse of [`ecliptic_to_equatorial`].
fn equatorial_to_ecliptic(v: [f64; 3]) -> [f64; 3] {
    let (c, s) = (OBLIQUITY_RAD.cos(), OBLIQUITY_RAD.sin());
    [v[0], v[1] * c + v[2] * s, -v[1] * s + v[2] * c]
}

/// Map an ecliptic position into the equatorial frame with the polar axis
/// stretched by 1/(1−f), so Earth's oblate ellipsoid becomes a sphere of
/// equatorial radius. The inverse is [`unflatten_from_sphere`].
fn flatten_to_sphere(v: [f64; 3]) -> [f64; 3] {
    let e = ecliptic_to_equatorial(v);
    [e[0], e[1], e[2] / (1.0 - EARTH_FLATTENING)]
}

/// Inverse of [`flatten_to_sphere`]: undo the polar stretch, return to ecliptic.
fn unflatten_from_sphere(v: [f64; 3]) -> [f64; 3] {
    equatorial_to_ecliptic([v[0], v[1], v[2] * (1.0 - EARTH_FLATTENING)])
}

/// Quantity that greatest eclipse minimizes, per syzygy type.
///
/// NASA's "greatest eclipse" is the instant the shadow **axis** passes closest
/// to Earth's center (solar) or the instant the Moon passes closest to the
/// umbral axis (lunar) — a minimum of a *linear* perpendicular distance, not of
/// the geocentric *angular* separation. The two minima differ by tens of seconds
/// (the bodies' geocentric distances change across the conjunction, re-weighting
/// the angular separation), which at the Sun's ~0.04″/s motion is ~1.5″ of
/// `eclipsed_longitude` — enough to blow the 1″ gate. We therefore refine on the
/// linear distance.
///
/// * Solar: perpendicular distance from Earth's center (origin) to the line
///   through the Sun and Moon = |s × m| / |m − s|.
/// * Lunar: perpendicular distance from the Moon to the umbral axis (the line
///   through Earth's center and the Sun) = |m × s| / |s|.
pub(crate) fn separation_for(syzygy: Syzygy, sample: &SunMoonSample) -> f64 {
    let s = rect(
        sample.sun_longitude_deg,
        sample.sun_latitude_deg,
        sample.sun_distance_au,
    );
    let m = rect(
        sample.moon_longitude_deg,
        sample.moon_latitude_deg,
        sample.moon_distance_au,
    );
    match syzygy {
        Syzygy::NewMoon => {
            let diff = [m[0] - s[0], m[1] - s[1], m[2] - s[2]];
            norm(cross(s, m)) / norm(diff)
        }
        Syzygy::FullMoon => norm(cross(m, s)) / norm(s),
    }
}

/// Geocentric sub-shadow point: latitude = Moon's equatorial declination,
/// longitude = Moon RA − GMST (east-positive), both from the mean obliquity.
pub(crate) fn sub_shadow_point(
    sample: &SunMoonSample,
    greatest_jd: f64,
) -> crate::types::GeoLocation {
    use pleiades_types::{Angle, EclipticCoordinates, Latitude, Longitude};
    let coords = EclipticCoordinates::new(
        Longitude::from_degrees(sample.moon_longitude_deg),
        Latitude::from_degrees(sample.moon_latitude_deg),
        Some(sample.moon_distance_au),
    );
    // Mean obliquity of date (degrees) via the standard IAU polynomial.
    let t = (greatest_jd - 2_451_545.0) / 36_525.0;
    let eps_deg = 23.439_291 - 0.013_004_2 * t;
    let equatorial = coords.to_equatorial(Angle::from_degrees(eps_deg));
    let declination = equatorial.declination.degrees();
    let ra_deg = equatorial.right_ascension.degrees();
    // GMST in degrees (IAU 1982), then geographic longitude = RA − GMST.
    let gmst_deg =
        (280.460_618_37 + 360.985_647_366_29 * (greatest_jd - 2_451_545.0)).rem_euclid(360.0);
    let mut lon = (ra_deg - gmst_deg + 180.0).rem_euclid(360.0) - 180.0;
    if lon <= -180.0 {
        lon += 360.0;
    }
    crate::types::GeoLocation {
        latitude_degrees: declination,
        longitude_degrees: lon,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ephemeris::SunMoonSample;

    fn sample(moon_lat_deg: f64, moon_dist_au: f64, sun_dist_au: f64) -> SunMoonSample {
        SunMoonSample {
            sun_longitude_deg: 100.0,
            sun_latitude_deg: 0.0,
            sun_distance_au: sun_dist_au,
            moon_longitude_deg: 100.0,
            moon_latitude_deg: moon_lat_deg,
            moon_distance_au: moon_dist_au,
        }
    }

    #[test]
    fn central_close_moon_is_total() {
        // Moon near perigee (close → large disk), on the node → total.
        let c = classify_solar(&sample(0.0, 0.00238, 1.000)).unwrap();
        assert_eq!(c.eclipse_type, SolarEclipseType::Total);
        assert!(c.magnitude >= 1.0);
        assert!(c.gamma.abs() < 0.1);
    }

    #[test]
    fn central_far_moon_is_annular() {
        // Moon near apogee (far → small disk), on the node → annular.
        let c = classify_solar(&sample(0.0, 0.00271, 1.000)).unwrap();
        assert_eq!(c.eclipse_type, SolarEclipseType::Annular);
        assert!(c.magnitude < 1.0);
    }

    #[test]
    fn far_from_node_is_no_eclipse() {
        assert!(classify_solar(&sample(1.5, 0.00257, 1.000)).is_none());
    }

    fn full_moon_sample(moon_lat_deg: f64, moon_dist_au: f64) -> SunMoonSample {
        // Full moon: Moon opposite the Sun in longitude.
        SunMoonSample {
            sun_longitude_deg: 100.0,
            sun_latitude_deg: 0.0,
            sun_distance_au: 1.000,
            moon_longitude_deg: 280.0,
            moon_latitude_deg: moon_lat_deg,
            moon_distance_au: moon_dist_au,
        }
    }

    #[test]
    fn central_full_moon_is_total_lunar() {
        let c = classify_lunar(&full_moon_sample(0.0, 0.00257)).unwrap();
        assert_eq!(c.eclipse_type, LunarEclipseType::Total);
        assert!(c.magnitude >= 1.0);
    }

    #[test]
    fn distant_latitude_full_moon_is_no_eclipse() {
        assert!(classify_lunar(&full_moon_sample(1.6, 0.00257)).is_none());
    }
}
