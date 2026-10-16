//! Numeric kernels checked against hand-evaluated values or Swiss Ephemeris
//! reference rows: IAU-2006 precession, the linear offset helper, and the
//! committed true-star and galactic cubics.

use pleiades_types::{Angle, Ayanamsa, Instant, JulianDay, TimeScale};

use crate::galactic::galactic_offset_degrees;
use crate::truestar::true_star_offset_degrees;
use crate::{general_precession_longitude_arcsec, AyanamsaDescriptor};

/// IAU 2006 (P03) general precession in longitude (Capitaine et al. 2003):
/// `pA(T) = 5028.796195 T + 1.1054348 T^2 + 0.00007964 T^3` arcseconds.
/// Each value below is the sum of the three terms worked out by hand.
#[test]
fn general_precession_matches_the_p03_polynomial() {
    for (t, expected) in [
        // 5028.796195 + 1.1054348 + 0.00007964
        (1.0, 5029.90170944),
        // -10057.59239 + 4.4217392 - 0.00063712
        (-2.0, -10053.17128792),
        // 15086.388585 + 9.9489132 + 0.00215028
        (3.0, 15096.33964848),
        // 50287.96195 + 110.54348 + 0.07964
        (10.0, 50398.58507),
    ] {
        let got = general_precession_longitude_arcsec(t);
        // Smallest mutant displacement over these epochs is 1.59e-4 arcsec
        // (cubic-term sign at T = 1); f64 rounding here is below 1e-11.
        assert!(
            (got - expected).abs() < 1.0e-9,
            "pA({t}) = {got}, expected {expected}"
        );
    }
}

fn at(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

/// The legacy linear path: `offset + centuries * 1.3969712777...` degrees per
/// Julian century, with `centuries = (jd - epoch) / 36525`. The rate is the
/// IAU 1976 general precession in longitude at J2000, 5029.0966 arcseconds per
/// Julian century (Lieske et al. 1977), divided by 3600. The expected values
/// below are worked out from that rate, not read back from the code.
#[test]
fn linear_offset_adds_the_precession_rate_per_julian_century() {
    let d = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Probe",
        &[],
        "Linear-offset probe",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(5.0)),
    );
    for (jd, expected) in [
        // two centuries after the epoch: 5 + 2 * 1.3969712777777778
        (2_451_545.0 + 73_050.0, 7.793_942_555_555_556),
        // one century before the epoch: 5 - 1.3969712777777778
        (2_451_545.0 - 36_525.0, 3.603_028_722_222_222),
        // half a century after the epoch: 5 + 0.5 * 1.3969712777777778
        (2_451_545.0 + 18_262.5, 5.698_485_638_888_889),
    ] {
        let got = d.offset_at(at(jd)).expect("offset").degrees();
        assert!(
            (got - expected).abs() < 1.0e-12,
            "offset at JD {jd} = {got}, expected {expected}"
        );
    }
}

/// Swiss Ephemeris mean-ayanamsa rows from
/// `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv`
/// (`mode_code,jd_tt,se_ayanamsa_deg`), copied here as literals.
/// `(mode, CSV line at JD 2451545, SE degrees, CSV line at JD 2488070, SE degrees)`.
///
/// JD 2451545 is T = 0 (pins the constant term) and JD 2488070 is T = 1 (the
/// edge of the fit window, where the T, T^2 and T^3 terms all carry weight).
type SeRow = (Ayanamsa, u32, f64, u32, f64);

const TRUE_STAR_ROWS: &[SeRow] = &[
    (Ayanamsa::TrueChitra, 48, 23.841361300, 51, 25.237369690),
    (Ayanamsa::TrueCitra, 58, 23.841361300, 61, 25.237369690),
    (Ayanamsa::TrueRevati, 238, 20.044211500, 241, 21.444596472),
    (Ayanamsa::TruePushya, 248, 22.721995234, 251, 24.120395986),
    (Ayanamsa::TrueMula, 258, 24.585708261, 261, 25.982698169),
    (Ayanamsa::TrueSheoran, 268, 25.229353017, 271, 26.627753769),
];

const GALACTIC_ROWS: &[SeRow] = &[
    (
        Ayanamsa::GalacticCenter,
        278,
        26.851701014,
        281,
        28.248872198,
    ),
    (
        Ayanamsa::GalacticCenterRgilbrand,
        288,
        22.474759997,
        291,
        23.871931181,
    ),
    (
        Ayanamsa::GalacticEquatorIau1958,
        298,
        30.023172469,
        301,
        31.427808983,
    ),
    (
        Ayanamsa::GalacticEquatorTrue,
        308,
        30.076092180,
        311,
        31.480726476,
    ),
    (
        Ayanamsa::GalacticEquatorMula,
        318,
        23.409425513,
        321,
        24.814059809,
    ),
    (
        Ayanamsa::GalacticCenterMardyks,
        328,
        30.017793877,
        331,
        31.414988833,
    ),
    (
        Ayanamsa::GalacticCenterMulaWilhelm,
        338,
        20.045135845,
        341,
        21.504219453,
    ),
    (
        Ayanamsa::GalacticCenterCochrane,
        348,
        356.851701014,
        351,
        358.248872198,
    ),
    (
        Ayanamsa::GalacticEquatorFiorenza,
        358,
        25.000019094,
        361,
        26.397214048,
    ),
];

/// Agreement required between a committed cubic and the SE row, in degrees.
///
/// The gate ceiling for these classes is 1.0 arcsecond (2.8e-4 degrees), which
/// is far too loose to see a sign flip on a cubic coefficient.
///
/// Floor: the largest fit residual against these rows is 6.73e-9 degrees
/// (Mula/Wilhelm at T = 1; 9-digit CSV resolution). Ceiling: the binding mutants
/// are the `delete -` flips on the galactic Equator `c3` (IAU 1958, True, Mula),
/// whose |mutant - SE| at T = 1 is 1.12e-7 to 1.13e-7. 2e-8 sits above the
/// floor and 5.6x below the ceiling.
///
/// Both JDs are nodes of the fit grid (T = 0 and T = 1, the window edge), where
/// the residuals are smallest. A refit with a different grid or window must
/// re-measure these residuals before this tolerance is trusted.
const FIT_ROW_TOL_DEG: f64 = 2.0e-8;

fn assert_rows_match(rows: &[SeRow], offset: impl Fn(&Ayanamsa, f64) -> Option<f64>) {
    for (mode, line_a, se_a, line_b, se_b) in rows {
        for (jd, line, se) in [(2_451_545.0, line_a, se_a), (2_488_070.0, line_b, se_b)] {
            let got = offset(mode, jd).unwrap_or_else(|| panic!("{mode:?} has no fit at {jd}"));
            assert!(
                (got - se).abs() < FIT_ROW_TOL_DEG,
                "{mode:?} at JD {jd} (ayanamsa.csv line {line}): got {got}, SE {se}"
            );
        }
    }
}

#[test]
fn true_star_cubics_reproduce_swiss_ephemeris_rows() {
    assert_rows_match(TRUE_STAR_ROWS, true_star_offset_degrees);
}

#[test]
fn galactic_cubics_reproduce_swiss_ephemeris_rows() {
    assert_rows_match(GALACTIC_ROWS, galactic_offset_degrees);
}
