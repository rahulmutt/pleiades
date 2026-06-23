//! Greenwich mean sidereal time (IAU-1982, Meeus 12.4) from a UT1 Julian day.

/// Greenwich mean sidereal time in degrees, normalized to `[0, 360)`.
///
/// `jd_ut1` is the Julian day in the UT1 time scale. Formula: Meeus,
/// *Astronomical Algorithms*, eq. 12.4.
pub fn gmst_degrees(jd_ut1: f64) -> f64 {
    let t = (jd_ut1 - 2_451_545.0) / 36_525.0;
    let theta =
        280.460_618_37 + 360.985_647_366_29 * (jd_ut1 - 2_451_545.0) + 0.000_387_933 * t * t
            - (t * t * t) / 38_710_000.0;
    theta.rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmst_matches_meeus_example_12a() {
        // Meeus Example 12.a: 1987 April 10, 0h UT -> JD 2446895.5,
        // GMST = 13h10m46.3668s = 197.693195 deg.
        let gmst = gmst_degrees(2_446_895.5);
        assert!((gmst - 197.693_195).abs() < 1e-4, "gmst {gmst}");
    }

    #[test]
    fn gmst_is_normalized() {
        for jd in [2_415_020.5_f64, 2_451_545.0, 2_488_069.5] {
            let g = gmst_degrees(jd);
            assert!(
                (0.0..360.0).contains(&g),
                "gmst {g} out of range at jd {jd}"
            );
        }
    }
}
