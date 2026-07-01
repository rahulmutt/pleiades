//! Apparent equatorial of date: rotate the tropical apparent ecliptic of date
//! into RA/Dec using the true obliquity of date (mean obliquity + Δε). Pure;
//! the only fallible step is the nutation table lookup.

use pleiades_types::{Angle, EclipticCoordinates, EquatorialCoordinates};

use crate::error::ApparentPlaceError;
use crate::nutation::{mean_obliquity_degrees, nutation};

/// True obliquity of date in degrees: mean obliquity (Meeus 22.2) plus
/// nutation-in-obliquity Δε.
pub fn true_obliquity_degrees(jd_tt: f64) -> Result<f64, ApparentPlaceError> {
    let delta_eps_arcsec = nutation(jd_tt)?.delta_eps_arcsec;
    Ok(mean_obliquity_degrees(jd_tt) + delta_eps_arcsec / 3600.0)
}

/// Apparent equatorial of date (RA/Dec) from the tropical apparent ecliptic of
/// date. The ecliptic must already carry of-date corrections (precession,
/// nutation-in-longitude, aberration, and any topocentric shift); this rotates
/// it into the equatorial frame of date. Distance is preserved.
pub fn apparent_equatorial_of_date(
    tropical_apparent_ecliptic: EclipticCoordinates,
    jd_tt: f64,
) -> Result<EquatorialCoordinates, ApparentPlaceError> {
    let eps = true_obliquity_degrees(jd_tt)?;
    Ok(tropical_apparent_ecliptic.to_equatorial(Angle::from_degrees(eps)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{Latitude, Longitude};

    fn ecl(lon: f64, lat: f64, dist: Option<f64>) -> EclipticCoordinates {
        EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            dist,
        )
    }

    #[test]
    fn true_obliquity_is_mean_plus_delta_eps() {
        let jd = 2_451_545.0; // J2000
        let mean = mean_obliquity_degrees(jd);
        let delta_eps_deg = nutation(jd).unwrap().delta_eps_arcsec / 3600.0;
        let got = true_obliquity_degrees(jd).unwrap();
        assert!((got - (mean + delta_eps_deg)).abs() < 1e-12, "got {got}");
        // Near 23.44° at J2000.
        assert!((got - 23.4392).abs() < 0.01, "true obliquity {got}");
    }

    #[test]
    fn composes_rotation_with_true_obliquity() {
        // Regression lock against independently pinned RA/Dec (captured once by
        // running the code, not recomputed from the function under test). This
        // ensures the output stays stable across refactors. Rotation-direction
        // correctness is covered separately by `solstice_point_maps_to_ra90_dec_obliquity`.
        let jd = 2_433_283.0;
        let e = ecl(123.456, 1.234, Some(0.987));
        let got = apparent_equatorial_of_date(e, jd).unwrap();

        const EXPECTED_RA_DEG: f64 = 126.070243353872;
        const EXPECTED_DEC_DEG: f64 = 20.589323321908;

        assert!(
            (got.right_ascension.degrees() - EXPECTED_RA_DEG).abs() < 1e-6,
            "RA drifted: {} vs {}",
            got.right_ascension.degrees(),
            EXPECTED_RA_DEG
        );
        assert!(
            (got.declination.degrees() - EXPECTED_DEC_DEG).abs() < 1e-6,
            "Dec drifted: {} vs {}",
            got.declination.degrees(),
            EXPECTED_DEC_DEG
        );
        assert_eq!(
            got.distance_au,
            Some(0.987),
            "distance must be preserved exactly"
        );
    }

    #[test]
    fn solstice_point_maps_to_ra90_dec_obliquity() {
        // Ecliptic longitude 90°, latitude 0° → RA 90°, Dec = +obliquity.
        let jd = 2_451_545.0;
        let eps = true_obliquity_degrees(jd).unwrap();
        let got = apparent_equatorial_of_date(ecl(90.0, 0.0, None), jd).unwrap();
        assert!(
            (got.right_ascension.degrees() - 90.0).abs() < 1e-6,
            "ra {}",
            got.right_ascension.degrees()
        );
        assert!(
            (got.declination.degrees() - eps).abs() < 1e-6,
            "dec {}",
            got.declination.degrees()
        );
    }

    #[test]
    fn roundtrips_through_to_ecliptic() {
        let jd = 2_469_807.5;
        let e = ecl(305.0, -4.5, Some(2.0));
        let eps = true_obliquity_degrees(jd).unwrap();
        let eq = apparent_equatorial_of_date(e, jd).unwrap();
        let back = eq.to_ecliptic(Angle::from_degrees(eps));
        assert!((back.longitude.degrees() - e.longitude.degrees()).abs() < 1e-7);
        assert!((back.latitude.degrees() - e.latitude.degrees()).abs() < 1e-7);
    }
}
