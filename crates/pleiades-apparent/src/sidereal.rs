//! Sidereal time (GMST/GAST, Greenwich and local) for the of-date chart layer.
//!
//! Sidereal time is a function of UT1 (Earth rotation), not TT/TDB. The
//! `Instant`'s Julian Day is used as supplied; pass a UT1-scale instant for
//! rigorous results (see `docs/time-observer-policy.md`).

use pleiades_types::{Angle, Instant, Longitude};

use crate::nutation::{mean_obliquity_degrees, nutation};

/// Greenwich Mean Sidereal Time in degrees (unnormalized).
///
/// Delegates to `pleiades_time::gmst_degrees_raw` — the single source of the
/// GMST polynomial — so the coefficients live in exactly one crate. Still
/// returns the unnormalized value; `sidereal_time` normalizes downstream.
pub fn greenwich_mean_sidereal_time_degrees(jd: f64) -> f64 {
    pleiades_time::gmst_degrees_raw(jd)
}

/// Equation of the equinoxes in degrees: `Δψ · cos(ε_true)`.
///
/// Falls back to `0.0` if the nutation table is unavailable (a development-time
/// artifact — a stale checksum — not a runtime condition), matching the prior
/// behavior in `pleiades-houses`.
pub fn equation_of_equinoxes_degrees(jd: f64) -> f64 {
    nutation(jd)
        .map(|n| {
            let delta_psi_deg = n.delta_psi_arcsec / 3600.0;
            let true_obl_rad =
                (mean_obliquity_degrees(jd) + n.delta_eps_arcsec / 3600.0).to_radians();
            delta_psi_deg * true_obl_rad.cos()
        })
        .unwrap_or(0.0)
}

/// Sidereal time in mean/apparent form, at Greenwich and locally.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SiderealTime {
    /// Greenwich Mean Sidereal Time, degrees `[0,360)`.
    pub gmst_deg: f64,
    /// Greenwich Apparent Sidereal Time (GMST + equation of equinoxes), degrees `[0,360)`.
    pub gast_deg: f64,
    /// Local Mean Sidereal Time (GMST + east longitude), degrees `[0,360)`.
    pub local_mean_deg: f64,
    /// Local Apparent Sidereal Time (GAST + east longitude), degrees `[0,360)`.
    pub local_apparent_deg: f64,
}

impl SiderealTime {
    /// GMST in hours `[0,24)`.
    pub fn gmst_hours(&self) -> f64 {
        self.gmst_deg / 15.0
    }
    /// GAST in hours `[0,24)`.
    pub fn gast_hours(&self) -> f64 {
        self.gast_deg / 15.0
    }
    /// Local mean sidereal time in hours `[0,24)`.
    pub fn local_mean_hours(&self) -> f64 {
        self.local_mean_deg / 15.0
    }
    /// Local apparent sidereal time in hours `[0,24)`.
    pub fn local_apparent_hours(&self) -> f64 {
        self.local_apparent_deg / 15.0
    }
}

/// Computes sidereal time for an instant and observer east longitude.
pub fn sidereal_time(instant: Instant, observer_longitude: Longitude) -> SiderealTime {
    let jd = instant.julian_day.days();
    let gmst = greenwich_mean_sidereal_time_degrees(jd);
    let ee = equation_of_equinoxes_degrees(jd);
    let lon = observer_longitude.degrees();
    let norm = |d: f64| Angle::from_degrees(d).normalized_0_360().degrees();
    SiderealTime {
        gmst_deg: norm(gmst),
        gast_deg: norm(gmst + ee),
        local_mean_deg: norm(gmst + lon),
        local_apparent_deg: norm(gmst + ee + lon),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{Instant, JulianDay, Longitude, TimeScale};

    fn j2000() -> Instant {
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
    }

    #[test]
    fn gmst_at_j2000_is_about_280_46_degrees() {
        let gmst = greenwich_mean_sidereal_time_degrees(2_451_545.0);
        // GMST at J2000.0 ≈ 280.4606°.
        assert!(
            (gmst.rem_euclid(360.0) - 280.4606).abs() < 1e-3,
            "got {gmst}"
        );
    }

    #[test]
    fn local_apparent_equals_gast_plus_east_longitude() {
        let st = sidereal_time(j2000(), Longitude::from_degrees(90.0));
        let expected = (st.gast_deg + 90.0).rem_euclid(360.0);
        assert!((st.local_apparent_deg - expected).abs() < 1e-9, "{st:?}");
    }

    #[test]
    fn all_fields_normalized_and_hours_consistent() {
        let st = sidereal_time(j2000(), Longitude::from_degrees(-123.4));
        for v in [
            st.gmst_deg,
            st.gast_deg,
            st.local_mean_deg,
            st.local_apparent_deg,
        ] {
            assert!((0.0..360.0).contains(&v), "not normalized: {v}");
        }
        assert!((st.gmst_hours() - st.gmst_deg / 15.0).abs() < 1e-12);
    }

    #[test]
    fn equation_of_equinoxes_is_small() {
        // EE is at most a couple of arcseconds ≈ a few×1e-4 degrees.
        assert!(equation_of_equinoxes_degrees(2_451_545.0).abs() < 0.01);
    }

    #[test]
    fn apparent_gmst_matches_pleiades_time_source() {
        for jd in [
            2_415_020.5_f64,
            2_433_283.0,
            2_451_545.0,
            2_469_807.0,
            2_488_069.5,
        ] {
            let apparent = greenwich_mean_sidereal_time_degrees(jd);
            // Un-normalized apparent value must equal the pleiades-time source exactly.
            assert_eq!(apparent, pleiades_time::gmst_degrees_raw(jd), "raw jd {jd}");
            // Reducing to [0,360) must match the normalized public fn.
            assert!(
                (apparent.rem_euclid(360.0) - pleiades_time::gmst_degrees(jd)).abs() < 1e-9,
                "normalized jd {jd}"
            );
        }
    }
}
