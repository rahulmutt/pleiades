//! IAU-2006 (P03) general precession in longitude, used as the drift term for
//! offset-defined ayanamsa modes.
#![forbid(unsafe_code)]

/// Accumulated general precession in longitude pA(T) from J2000.0, in arcseconds.
/// T is Julian centuries of TT from J2000.0. IAU 2006 (Capitaine et al. 2003).
pub(crate) fn general_precession_longitude_arcsec(t: f64) -> f64 {
    5028.796195 * t + 1.1054348 * t * t + 0.000_079_64 * t * t * t
}

/// Precession accumulated between two instants, expressed in degrees of longitude.
pub(crate) fn precession_delta_degrees(jd_tt: f64, epoch_jd_tt: f64) -> f64 {
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    let t0 = (epoch_jd_tt - 2_451_545.0) / 36_525.0;
    (general_precession_longitude_arcsec(t) - general_precession_longitude_arcsec(t0)) / 3600.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precession_rate_is_about_one_point_four_degrees_per_century() {
        // ~50.29"/yr ≈ 1.3969°/century near J2000.
        let one_century = precession_delta_degrees(2_451_545.0 + 36_525.0, 2_451_545.0);
        assert!((one_century - 1.396_9).abs() < 0.001, "got {one_century}");
    }

    #[test]
    fn precession_delta_is_zero_at_epoch() {
        assert_eq!(precession_delta_degrees(2_440_000.0, 2_440_000.0), 0.0);
    }

    #[test]
    fn precession_is_nonlinear_over_the_window() {
        // The quadratic term makes a +1 century delta differ from a -1 century delta.
        let fwd = precession_delta_degrees(2_451_545.0 + 36_525.0, 2_451_545.0);
        let bwd = precession_delta_degrees(2_451_545.0, 2_451_545.0 - 36_525.0);
        assert!((fwd - bwd).abs() > 1.0e-5, "expected nonlinearity, fwd={fwd} bwd={bwd}");
    }
}
