//! Backend-free low-precision Sun, for the places that need the Sun's
//! longitude only as the argument of the annual-aberration term.

use crate::mean_elements::J2000_JD;

/// Sun's geometric (true) ecliptic longitude of date, degrees, via the Meeus
/// low-precision solar theory (Astronomical Algorithms, ch. 25).
///
/// Computes the geometric mean longitude `L0`, the mean anomaly `M`, and the
/// equation of the center `C`, then returns the true longitude `L0 + C`. Accuracy
/// is ~0.01°, which is far more than enough for annual aberration: the offset is
/// at most κ ≈ 20.5″ and its sensitivity to the Sun's longitude is a fraction of
/// that per degree. Being backend-free, it costs nothing next to an ephemeris
/// query: the fixed-star path carries no ephemeris, and the event engine's
/// apparent place uses it for the provenance-only aberration estimate instead
/// of querying the backend for the Sun on every sample (issue #128). Input `jd`
/// is the (TT/TDB) Julian Day of date.
pub(crate) fn sun_true_longitude_of_date_deg(jd: f64) -> f64 {
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
