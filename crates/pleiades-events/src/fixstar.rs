//! Curated fixed-star apparent place. A bounded set of astrologically-used
//! stars, astrometry matched to Swiss Ephemeris `sefstars.txt`, parsed once from
//! a committed CSV at build time.

use crate::error::EventError;
use pleiades_apparent::aberration::annual_aberration;
use pleiades_apparent::nutation::nutation;
use pleiades_apparent::{apparent_equatorial_of_date, precess_ecliptic_j2000_to_date};
use pleiades_types::{
    Angle, EclipticCoordinates, EquatorialCoordinates, Instant, Latitude, Longitude,
};

/// Julian Day number of the J2000.0 epoch (TT).
const J2000_JD: f64 = 2_451_545.0;
/// Mean obliquity of the ecliptic at J2000.0, degrees (IAU 1976).
const MEAN_OBLIQUITY_J2000_DEG: f64 = 23.439_291_1;

/// Sun's geometric (true) ecliptic longitude of date, degrees, via the Meeus
/// low-precision solar theory (Astronomical Algorithms, ch. 25).
///
/// Computes the geometric mean longitude `L0`, the mean anomaly `M`, and the
/// equation of the center `C`, then returns the true longitude `L0 + C`. Accuracy
/// is ~0.01°, which is far more than enough for annual aberration: the offset is
/// at most κ ≈ 20.5″ and its sensitivity to the Sun's longitude is a fraction of
/// that per degree. This is deliberately backend-free — `fixed_star_apparent`
/// carries no ephemeris, and the aberration term only needs ⊙ to arcsecond-scale
/// effect. Input `jd` is the (TT/TDB) Julian Day of date.
fn sun_true_longitude_of_date_deg(jd: f64) -> f64 {
    let t = (jd - J2000_JD) / 36_525.0; // Julian centuries since J2000.0
                                        // Geometric mean longitude of the Sun (Meeus 25.2).
    let l0 = 280.466_46 + 36_000.769_83 * t + 0.000_303_2 * t * t;
    // Mean anomaly of the Sun (Meeus 25.3).
    let m = (357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t).to_radians();
    // Equation of the center (Meeus, ch. 25).
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    (l0 + c).rem_euclid(360.0)
}

/// One catalog row: J2000 ICRS position + space motion.
#[derive(Clone, Copy, Debug)]
pub struct FixedStarEntry {
    /// SE-compatible star name.
    pub name: &'static str,
    /// Right ascension at J2000, degrees.
    pub ra_j2000_deg: f64,
    /// Declination at J2000, degrees.
    pub dec_j2000_deg: f64,
    /// Proper motion in RA (mas/yr, already `·cosδ`-folded per sefstars convention).
    pub pm_ra_mas_yr: f64,
    /// Proper motion in Dec (mas/yr).
    pub pm_dec_mas_yr: f64,
    /// Parallax, milliarcseconds.
    pub parallax_mas: f64,
    /// Radial velocity, km/s.
    pub rv_km_s: f64,
}

const RAW: &str = include_str!("../data/fixstars-catalog.csv");

// Parsed at first use; the file is tiny (~30 rows). Build a static via a parse fn
// invoked in a `LazyLock` (std, no extra deps).
use std::sync::LazyLock;
pub(crate) static CATALOG: LazyLock<Vec<FixedStarEntry>> = LazyLock::new(parse_catalog);

fn parse_catalog() -> Vec<FixedStarEntry> {
    RAW.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("name") && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            FixedStarEntry {
                name: Box::leak(f[0].to_string().into_boxed_str()),
                ra_j2000_deg: f[1].parse().unwrap(),
                dec_j2000_deg: f[2].parse().unwrap(),
                pm_ra_mas_yr: f[3].parse().unwrap(),
                pm_dec_mas_yr: f[4].parse().unwrap(),
                parallax_mas: f[5].parse().unwrap(),
                rv_km_s: f[6].parse().unwrap(),
            }
        })
        .collect()
}

/// Looks up a curated fixed star by SE-compatible name (case-insensitive).
///
/// Returns [`EventError::UnknownFixedStar`] (fail closed, no placeholder) when the
/// name is not present in the curated catalog.
pub fn fixed_star_entry(name: &str) -> Result<&'static FixedStarEntry, EventError> {
    CATALOG
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| EventError::UnknownFixedStar {
            name: name.to_string(),
        })
}

/// Apparent equatorial of date for a curated fixed star. Applies space motion to
/// epoch, precession (IAU-1976), nutation-in-longitude (IAU-1980), and annual
/// aberration — no light-time or light-deflection, matching `swe_fixstar`'s
/// default flags. Backend-free: the Sun's true longitude needed by the aberration
/// term is computed analytically ([`sun_true_longitude_of_date_deg`]).
pub fn fixed_star_apparent(name: &str, at: Instant) -> Result<EquatorialCoordinates, EventError> {
    let e = fixed_star_entry(name)?;
    let jd = at.julian_day.days();
    let years = (jd - J2000_JD) / 365.25;

    // 1. Space motion to epoch (RA/Dec, degrees). PM in mas/yr; the RA pm is on
    //    the great circle, so recover the coordinate rate by dividing by cosδ
    //    (sefstars convention already folds cosδ into `pm_ra_mas_yr`).
    let dec0 = e.dec_j2000_deg.to_radians();
    let ra = e.ra_j2000_deg + (e.pm_ra_mas_yr / 3_600_000.0) * years / dec0.cos();
    let dec = e.dec_j2000_deg + (e.pm_dec_mas_yr / 3_600_000.0) * years;

    // 2. Equatorial J2000 → ecliptic J2000 for the precession/nutation helpers,
    //    which operate in ecliptic coordinates.
    let equ_j2000 = EquatorialCoordinates::new(
        Angle::from_degrees(ra.rem_euclid(360.0)),
        Latitude::from_degrees(dec),
        None,
    );
    let ecl_j2000 = equ_j2000.to_ecliptic(Angle::from_degrees(MEAN_OBLIQUITY_J2000_DEG));

    // 3. Precession J2000 → of date (ecliptic).
    let precessed = precess_ecliptic_j2000_to_date(
        ecl_j2000.longitude.degrees(),
        ecl_j2000.latitude.degrees(),
        jd,
    )
    .map_err(|err| EventError::Backend(format!("star precession failed: {err}")))?;

    // 4. Nutation in longitude (mean → true equinox) + annual aberration. The
    //    aberration term needs the Sun's true longitude of date, computed
    //    analytically since this path carries no ephemeris backend.
    let nut =
        nutation(jd).map_err(|err| EventError::Backend(format!("star nutation failed: {err}")))?;
    let sun_lon = sun_true_longitude_of_date_deg(jd);
    let ab = annual_aberration(precessed.longitude_deg, precessed.latitude_deg, sun_lon, jd);
    let lon_true =
        precessed.longitude_deg + nut.delta_psi_arcsec / 3600.0 + ab.d_lambda_arcsec / 3600.0;
    let lat_true = precessed.latitude_deg + ab.d_beta_arcsec / 3600.0;

    // 5. True ecliptic of date → apparent equatorial of date (uses true obliquity).
    let ecl_true = EclipticCoordinates::new(
        Longitude::from_degrees(lon_true.rem_euclid(360.0)),
        Latitude::from_degrees(lat_true),
        None,
    );
    apparent_equatorial_of_date(ecl_true, jd)
        .map_err(|err| EventError::Backend(format!("star apparent equatorial failed: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_star_resolves() {
        let e = fixed_star_entry("Aldebaran").unwrap();
        assert!((e.ra_j2000_deg - 68.98).abs() < 0.1);
        assert!((e.dec_j2000_deg - 16.51).abs() < 0.1);
    }

    #[test]
    fn unknown_star_fails_closed() {
        let err = fixed_star_entry("Nonesuch").unwrap_err();
        assert!(matches!(err, EventError::UnknownFixedStar { .. }));
    }

    #[test]
    fn aldebaran_apparent_is_near_catalog_and_of_date() {
        use pleiades_types::{Instant, JulianDay, TimeScale};
        let at = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb); // J2000
        let equ = fixed_star_apparent("Aldebaran", at).unwrap();
        // At J2000 the of-date apparent RA/Dec differ from the catalog J2000 values
        // only by nutation + aberration (< ~40"). Assert the small, bounded shift.
        assert!(
            (equ.right_ascension.degrees() - 68.98).abs() < 0.02,
            "ra {}",
            equ.right_ascension.degrees()
        );
        assert!(
            (equ.declination.degrees() - 16.51).abs() < 0.02,
            "dec {}",
            equ.declination.degrees()
        );
    }

    #[test]
    fn unknown_star_apparent_fails_closed() {
        use pleiades_types::{Instant, JulianDay, TimeScale};
        let at = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        assert!(matches!(
            fixed_star_apparent("Nope", at).unwrap_err(),
            EventError::UnknownFixedStar { .. }
        ));
    }

    #[test]
    fn catalog_is_nonempty_and_finite() {
        assert!(CATALOG.len() >= 25);
        for e in CATALOG.iter() {
            assert!(e.ra_j2000_deg.is_finite() && (0.0..360.0).contains(&e.ra_j2000_deg));
            assert!((-90.0..=90.0).contains(&e.dec_j2000_deg));
        }
    }
}
