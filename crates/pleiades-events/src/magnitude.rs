//! Swiss Ephemeris 2.10.03 `swe_pheno` apparent-magnitude and disc-diameter
//! models, transcribed verbatim from the vendored C source
//! (`libswisseph-sys-0.1.2/libswisseph/swecl.c:3876-4056`, tables at
//! `sweph.h:315-333` and `swecl.c:3762-3790`). Compile-time toggles in that
//! build are `MAG_MALLAMA_2018 = 1` and `MAG_MOON_VREIJS = 1`, so: Mercury–
//! Neptune use the Mallama-2018 analytic forms, the Moon uses the Vreijs law,
//! Pluto uses the classic `mag_elem` table (−1.00), and the Sun uses the
//! disc-ratio form from `mag_elem[0][0] = −26.86`. See the SP-5 plan §E1–§E9.

use pleiades_types::CelestialBody;

/// AU in metres (SE `AUNIT`, DE431 value; sweph.h:273).
pub(crate) const AUNIT_M: f64 = 1.495_978_707_00e11;
/// Speed of light, m/s (SE `CLIGHT`; sweph.h:274).
pub(crate) const CLIGHT_M_S: f64 = 2.997_924_58e8;
/// Earth equatorial radius, m (SE `EARTH_RADIUS`; sweph.h:282).
const EARTH_RADIUS_M: f64 = 6_378_136.6;
/// J2000.0 as a Julian Day.
const J2000_JD: f64 = 2_451_545.0;

/// SE `pla_diam` — full physical diameters in metres, Sun..Pluto
/// (sweph.h:315-324). Bodies without a disc datum return 0.
fn pla_diam_m(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Sun => 1_392_000_000.0,
        CelestialBody::Moon => 3_475_000.0,
        CelestialBody::Mercury => 2_439_400.0 * 2.0,
        CelestialBody::Venus => 6_051_800.0 * 2.0,
        CelestialBody::Mars => 3_389_500.0 * 2.0,
        CelestialBody::Jupiter => 69_911_000.0 * 2.0,
        CelestialBody::Saturn => 58_232_000.0 * 2.0,
        CelestialBody::Uranus => 25_362_000.0 * 2.0,
        CelestialBody::Neptune => 24_622_000.0 * 2.0,
        CelestialBody::Pluto => 1_188_300.0 * 2.0,
        _ => 0.0,
    }
}

/// Apparent disc diameter (degrees), SE `attr[3]` (swecl.c:3878-3887). Zero
/// when the body has no disc datum.
pub(crate) fn diameter_deg(body: &CelestialBody, delta_au: f64) -> f64 {
    let dd = pla_diam_m(body);
    if dd == 0.0 {
        return 0.0;
    }
    let sin_semi = dd / 2.0 / AUNIT_M / delta_au.max(1e-12);
    // Guard the asin domain (never-NaN convention).
    (sin_semi.clamp(-1.0, 1.0)).asin() * 2.0 * (180.0 / core::f64::consts::PI)
}

/// Inputs to the per-body magnitude model.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MagInputs {
    /// Phase angle, degrees.
    pub phase_angle_deg: f64,
    /// Heliocentric distance r, AU.
    pub r_helio_au: f64,
    /// Geocentric distance Δ, AU.
    pub delta_geo_au: f64,
    /// Apparent disc diameter, degrees (Sun only).
    pub diameter_deg: f64,
    /// Geocentric apparent ecliptic longitude/latitude, degrees (Saturn ring).
    pub geo_lon_deg: f64,
    pub geo_lat_deg: f64,
    /// Heliocentric ecliptic longitude/latitude, degrees (Saturn ring).
    pub helio_lon_deg: f64,
    pub helio_lat_deg: f64,
    /// TDB Julian Day (Neptune time term, Saturn ring epoch).
    pub jd_tdb: f64,
    /// Light-time, days (Saturn ring epoch `tjd − dt`).
    pub light_time_days: f64,
}

/// `5·log10(r·Δ)`, the common distance term.
fn dist_term(m: &MagInputs) -> f64 {
    5.0 * (m.r_helio_au * m.delta_geo_au).log10()
}

/// Apparent visual magnitude for the ten majors; `None` otherwise.
/// Verbatim SE 2.10.03 branch structure (swecl.c:3891-4056).
pub(crate) fn apparent_magnitude(body: &CelestialBody, m: &MagInputs) -> Option<f64> {
    let a = m.phase_angle_deg;
    let a2 = a * a;
    Some(match body {
        CelestialBody::Sun => {
            // Disc-ratio form (swecl.c:3892-3896): mag = −26.86 − 2.5·log10(fac),
            // fac = (Δ-diameter / mean-diameter-at-1AU)².
            let mean = diameter_deg(&CelestialBody::Sun, 1.0);
            let fac = (m.diameter_deg / mean).powi(2);
            -26.86 - 2.5 * fac.log10()
        }
        CelestialBody::Moon => {
            // Vreijs (swecl.c:3900-3914).
            let base = if a <= 147.138_546_5 {
                -21.62 + 0.026 * a.abs() + 0.000_000_004 * a.powi(4)
            } else {
                -4.5444 - 2.5 * (180.0 - a).powi(3).log10()
            };
            base + 5.0 * (m.delta_geo_au * m.r_helio_au * AUNIT_M / EARTH_RADIUS_M).log10()
        }
        CelestialBody::Mercury => {
            // Mallama 2018 (swecl.c:3923-3927).
            let a3 = a2 * a;
            let a4 = a3 * a;
            let a5 = a4 * a;
            let a6 = a5 * a;
            (-0.613 + a * 6.3280e-2 - a2 * 1.6336e-3 + a3 * 3.3644e-5 - a4 * 3.4265e-7
                + a5 * 1.6893e-9
                - a6 * 3.0334e-12)
                + dist_term(m)
        }
        CelestialBody::Venus => {
            // Mallama 2018 (swecl.c:3928-3935).
            let base = if a <= 163.7 {
                -4.384 - a * 1.044e-3 + a2 * 3.687e-4 - a2 * a * 2.814e-6 + a2 * a2 * 8.938e-9
            } else {
                236.058_28 - a * 2.819_14 + a2 * 8.390_34e-3
            };
            base + dist_term(m)
        }
        CelestialBody::Mars => {
            // Mallama 2018 (swecl.c:3938-3956).
            let base = if a <= 50.0 {
                -1.601 + a * 0.02267 - a2 * 0.000_130_2
            } else {
                -0.367 - a * 0.02573 + a2 * 0.000_344_5
            };
            base + dist_term(m)
        }
        CelestialBody::Jupiter => {
            // Mallama 2018 (swecl.c:3957-3962).
            (-9.395 - a * 3.7e-4 + a2 * 6.16e-4) + dist_term(m)
        }
        CelestialBody::Saturn => {
            // Mallama 2018 + Meeus ring (swecl.c:3963-3983).
            let t = (m.jd_tdb - m.light_time_days - J2000_JD) / 36_525.0;
            let inc = (28.075216 - 0.012998 * t + 0.000004 * t * t).to_radians();
            let om = (169.508470 + 1.394681 * t + 0.000412 * t * t).to_radians();
            let sin_b = inc.sin()
                * m.geo_lat_deg.to_radians().cos()
                * (m.geo_lon_deg.to_radians() - om).sin()
                - inc.cos() * m.geo_lat_deg.to_radians().sin();
            let sin_b2 = inc.sin()
                * m.helio_lat_deg.to_radians().cos()
                * (m.helio_lon_deg.to_radians() - om).sin()
                - inc.cos() * m.helio_lat_deg.to_radians().sin();
            let sin_b = ((sin_b.clamp(-1.0, 1.0).asin() + sin_b2.clamp(-1.0, 1.0).asin()) / 2.0)
                .sin()
                .abs();
            (-8.914 - 1.825 * sin_b + 0.026 * a - 0.378 * sin_b * 2.7182818_f64.powf(-2.25 * a))
                + dist_term(m)
        }
        CelestialBody::Uranus => {
            // Mallama 2018, sub-Earth-latitude term dropped, −0.05 (swecl.c:3984-3994).
            (-7.110 + a * 6.587e-3 + a2 * 1.045e-4) + dist_term(m) - 0.05
        }
        CelestialBody::Neptune => {
            // Mallama 2018, time-dependent brightening (swecl.c:3995-4005).
            let base = if m.jd_tdb < 2_444_239.5 {
                -6.89
            } else if m.jd_tdb <= 2_451_544.5 {
                -6.89 - 0.0055 * (m.jd_tdb - 2_444_239.5) / 365.25
            } else {
                -7.00
            };
            base + dist_term(m)
        }
        CelestialBody::Pluto => {
            // Classic mag_elem[9] = {−1.00, 0, 0, 0} (swecl.c:4031-4036).
            -1.00 + dist_term(m)
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::CelestialBody;

    fn base_inputs() -> MagInputs {
        MagInputs {
            phase_angle_deg: 0.0,
            r_helio_au: 1.0,
            delta_geo_au: 1.0,
            diameter_deg: 0.0,
            geo_lon_deg: 0.0,
            geo_lat_deg: 0.0,
            helio_lon_deg: 0.0,
            helio_lat_deg: 0.0,
            jd_tdb: 2_451_545.0,
            light_time_days: 0.0,
        }
    }

    #[test]
    fn pluto_is_absolute_minus_one_at_unit_distances() {
        // 5*log10(r*Δ) + (-1.00); r=Δ=1 ⇒ log term 0 ⇒ -1.00.
        let m = apparent_magnitude(&CelestialBody::Pluto, &base_inputs()).unwrap();
        assert!((m + 1.00).abs() < 1e-9, "pluto {m}");
    }

    #[test]
    fn jupiter_at_zero_phase_unit_distance_is_minus_9_395() {
        let m = apparent_magnitude(&CelestialBody::Jupiter, &base_inputs()).unwrap();
        assert!((m + 9.395).abs() < 1e-9, "jupiter {m}");
    }

    #[test]
    fn distance_term_adds_five_log10_r_delta() {
        let mut i = base_inputs();
        i.r_helio_au = 5.0;
        i.delta_geo_au = 4.0;
        let m = apparent_magnitude(&CelestialBody::Pluto, &i).unwrap();
        let expected = -1.00 + 5.0 * (20.0_f64).log10();
        assert!((m - expected).abs() < 1e-9, "pluto far {m}");
    }

    #[test]
    fn sun_magnitude_is_about_minus_26_86_at_one_au() {
        let mut i = base_inputs();
        i.diameter_deg = diameter_deg(&CelestialBody::Sun, 1.0);
        let m = apparent_magnitude(&CelestialBody::Sun, &i).unwrap();
        // At 1 AU the disc equals its mean disc ⇒ fac=1 ⇒ mag = -26.86.
        assert!((m + 26.86).abs() < 1e-6, "sun {m}");
    }

    #[test]
    fn non_major_bodies_have_no_model() {
        let id = pleiades_types::CustomBodyId::new("asteroid", "2060-Chiron");
        let m = apparent_magnitude(&CelestialBody::Custom(id), &base_inputs());
        assert!(m.is_none());
    }

    #[test]
    fn sun_disc_diameter_is_about_half_a_degree() {
        let d = diameter_deg(&CelestialBody::Sun, 1.0);
        assert!((d - 0.533).abs() < 0.01, "sun diam {d}");
    }
}
