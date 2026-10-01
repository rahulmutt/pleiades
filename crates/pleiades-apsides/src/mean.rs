//! Mean lunar orbit: the Moon's mean node and mean perigee as slowly varying
//! orbital elements, and the constant mean shape of the orbit.
//!
//! The two longitudes are the Meeus (Astronomical Algorithms, ch. 47)
//! polynomials, referred to the **mean equinox and ecliptic of date**. They are
//! *elements*, not points on the sky: the perigee longitude is the node
//! longitude plus the argument of perigee measured in the orbit plane. To get
//! the point Swiss Ephemeris reports as `SE_MEAN_APOG`, pass
//! [`mean_lunar_elements_of_date`] to [`crate::points_from_elements`], which
//! places the apsis on the inclined orbit (up to about 7′ in longitude and
//! 5.1° in latitude away from the raw element).

use crate::KeplerianElements;

const J2000_JD: f64 = 2_451_545.0;
const DAYS_PER_JULIAN_CENTURY: f64 = 36_525.0;
const METERS_PER_AU: f64 = 1.495_978_707_00e11;

/// Mean inclination of the lunar orbit to the ecliptic, degrees (Swiss
/// Ephemeris `MOON_MEAN_INCL`).
pub const MOON_MEAN_INCLINATION_DEG: f64 = 5.145_396_4;
/// Mean eccentricity of the lunar orbit (Swiss Ephemeris `MOON_MEAN_ECC`).
pub const MOON_MEAN_ECCENTRICITY: f64 = 0.054_900_489;
/// Mean semi-major axis of the lunar orbit, AU (384 400 km, Swiss Ephemeris
/// `MOON_MEAN_DIST`).
pub const MOON_MEAN_SEMI_MAJOR_AU: f64 = 384_400_000.0 / METERS_PER_AU;

fn julian_centuries(jd_tt: f64) -> f64 {
    (jd_tt - J2000_JD) / DAYS_PER_JULIAN_CENTURY
}

/// Mean longitude of the Moon's ascending node, degrees in `[0, 360)`, mean
/// equinox of date, at the TT Julian day `jd_tt`. Non-finite input yields NaN.
pub fn mean_lunar_node_longitude_of_date(jd_tt: f64) -> f64 {
    let t = julian_centuries(jd_tt);
    (125.044_547_9
        + (-1_934.136_289_1 + (0.002_075_4 + (1.0 / 476_441.0 - t / 60_616_000.0) * t) * t) * t)
        .rem_euclid(360.0)
}

/// Mean longitude of the Moon's perigee (node longitude plus in-plane argument
/// of perigee), degrees in `[0, 360)`, mean equinox of date, at the TT Julian
/// day `jd_tt`. Non-finite input yields NaN.
pub fn mean_lunar_perigee_longitude_of_date(jd_tt: f64) -> f64 {
    let t = julian_centuries(jd_tt);
    (83.353_246_5
        + (4_069.013_728_7 + (-0.010_32 + (-1.0 / 80_053.0 + t / 18_999_000.0) * t) * t) * t)
        .rem_euclid(360.0)
}

/// The Moon's mean Keplerian elements in the mean ecliptic of date at the TT
/// Julian day `jd_tt`, ready for [`crate::points_from_elements`].
pub fn mean_lunar_elements_of_date(jd_tt: f64) -> KeplerianElements {
    KeplerianElements {
        node_deg: mean_lunar_node_longitude_of_date(jd_tt),
        peri_lon_deg: mean_lunar_perigee_longitude_of_date(jd_tt),
        incl_deg: MOON_MEAN_INCLINATION_DEG,
        eccentricity: MOON_MEAN_ECCENTRICITY,
        semi_major_au: MOON_MEAN_SEMI_MAJOR_AU,
    }
}

#[cfg(test)]
mod tests;
