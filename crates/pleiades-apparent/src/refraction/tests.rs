//! White-box unit tests for the refraction module.
//!
//! Relocated out of `refraction.rs` per AGENTS.md ("keep large inline test
//! suites out of the file under test"). These remain white-box unit tests with
//! access to the module's private helpers — they are deliberately not converted
//! into black-box integration tests.

use super::*;

#[test]
fn default_atmosphere_is_se_standard() {
    let a = Atmosphere::default();
    assert_eq!(a.pressure_mbar, 1013.25);
    assert_eq!(a.temperature_c, 15.0);
}

#[test]
fn refraction_at_horizon_is_about_34_arcmin() {
    // Bennett, evaluated ON the true altitude at h=0 with standard
    // atmosphere, gives a true→apparent lift of ~29' (0.4752° ≈ 28.5').
    // The ~34' figure (0.567°) belongs to the *other* direction — see
    // `true_from_apparent_at_horizon_is_about_negative_34_arcmin` below,
    // which evaluates Saemundsson on the apparent altitude at h=0. Assert
    // the apparent altitude sits ~29' above 0 within a loose band.
    let app = apparent_from_true(0.0, Atmosphere::default());
    assert!(
        (app - 0.4752).abs() < 0.05,
        "apparent horizon altitude {app}"
    );
}

#[test]
fn refraction_vanishes_at_zenith() {
    let app = apparent_from_true(90.0, Atmosphere::default());
    assert!((app - 90.0).abs() < 1e-4, "zenith {app}");
}

#[test]
fn saemundsson_inverts_bennett_within_a_few_arcsec() {
    // Round-trip: for altitudes above the horizon the two formulae are near-inverses.
    for h in [5.0, 15.0, 45.0, 80.0] {
        let app = apparent_from_true(h, Atmosphere::default());
        let back = true_from_apparent(app, Atmosphere::default());
        assert!((back - h).abs() < 0.01, "round-trip h={h} back={back}");
    }
}

#[test]
fn true_from_apparent_at_horizon_is_about_negative_34_arcmin() {
    // A body seen ON the apparent horizon (h_app=0) is geometrically ~34' below it.
    let t = true_from_apparent(0.0, Atmosphere::default());
    assert!(
        (t + 0.5667).abs() < 0.02,
        "true altitude at apparent horizon {t}"
    );
}

#[test]
fn refraction_matches_se_below_horizon() {
    // Pinned from `crates/pleiades-validate/data/rise-trans-corpus/azalt.csv`
    // (`se_true_alt_deg < 0` rows; standard atmosphere, `swe_azalt` ->
    // `swe_refrac_extended` ground truth). SE reports `se_apparent_alt_deg
    // == se_true_alt_deg` (refraction fully suppressed) for every one of
    // these — see `apparent_from_true_below_horizon`'s doc for why this
    // module approximates rather than exactly reproduces SE's own
    // (discontinuous) below-horizon model. The shallowest row (-9.96 deg)
    // sits right at the edge of the fade and is pinned within 15 arcsec;
    // every deeper row is pinned within a fraction of an arcsec. Both are
    // a large improvement over the pre-fix ~282 arcsec worst case.
    let atmos = Atmosphere::default();
    for (true_alt, se_apparent_alt, tolerance_arcsec) in [
        (-9.964249, -9.964249, 15.0),
        (-15.874977, -15.874977, 0.01),
        (-34.289902, -34.289902, 0.01),
        (-43.529313, -43.529313, 0.01),
        (-60.896360, -60.896360, 0.01),
        (-64.642565, -64.642565, 0.01),
        (-70.739219, -70.739219, 0.01),
    ] {
        let app = apparent_from_true(true_alt, atmos);
        let residual_arcsec = (app - se_apparent_alt).abs() * 3600.0;
        assert!(
            residual_arcsec < tolerance_arcsec,
            "true={true_alt} app={app} se={se_apparent_alt} residual={residual_arcsec}\""
        );
    }
}

/// Atmosphere crafted so `scale` is exactly `1.0`: `1010/1010 = 1` and
/// `283/(273+10) = 1`. Lets a refraction literal be compared without any
/// scaling factor folded in.
const EXACT: Atmosphere = Atmosphere {
    pressure_mbar: 1010.0,
    temperature_c: 10.0,
};

/// Atmosphere where BOTH scale factors differ from 1 and from each other
/// (`2020/1010 = 2`, `283/298 = 0.9497`), so no operator swap inside `scale`
/// can alias another and still produce the right answer.
const DENSE: Atmosphere = Atmosphere {
    pressure_mbar: 2020.0,
    temperature_c: 25.0,
};

/// Tolerance for degree-valued altitude assertions. Values here are O(1) and
/// f64 carries ~1e-16 relative precision; 1e-11 absorbs any last-ULP `tan()`
/// variation between platform libm implementations while staying far tighter
/// than the smallest mutant-induced shift (~1e-3 deg).
// Not consumed by this task's tests; produced here for the degree-valued
// assertions Tasks 3 and 4 add on top of this file. Allowed narrowly rather
// than dropped so the two tolerance constants stay defined together at the
// point their rationale is documented.
#[allow(dead_code)]
const TOL_DEG: f64 = 1e-11;

/// Same rationale, for arcminute-valued refraction assertions (values O(10)).
const TOL_ARCMIN: f64 = 1e-10;

#[test]
fn scale_matches_independent_pressure_temperature_ratio() {
    // scale = (p/1010) * (283/(273+t)), evaluated independently.
    // EXACT is constructed so both factors are exactly 1.
    assert_eq!(scale(EXACT), 1.0);
    // Pressure doubled, temperature factor still exactly 1.
    assert_eq!(
        scale(Atmosphere {
            pressure_mbar: 2020.0,
            temperature_c: 10.0,
        }),
        2.0
    );
    // Both factors non-unit: 2 * (283/298) = 1.8993288590604027. This case is
    // what distinguishes `*` from `/` between the two factors — with a unity
    // second factor the swap would be invisible.
    assert!(
        (scale(DENSE) - 1.899_328_859_060_402_7).abs() < 1e-15,
        "dense scale {}",
        scale(DENSE)
    );
}

#[test]
fn bennett_matches_independently_evaluated_formula() {
    // R = scale * 1.02 / tan(h + 10.3/(h + 5.11)) arcmin, h in degrees,
    // evaluated outside this crate from the published Bennett (1982) formula.
    for (h, atmos, expected_arcmin) in [
        (-0.5, EXACT, 33.687_796_094_672_83),
        (-1.0, EXACT, 38.794_837_252_861_49),
        (-1.0, DENSE, 73.684_153_976_911_42),
    ] {
        let got = bennett_refraction_arcmin(h, atmos);
        assert!(
            (got - expected_arcmin).abs() < TOL_ARCMIN,
            "bennett h={h} got={got} expected={expected_arcmin}"
        );
    }
}

#[test]
fn saemundsson_matches_independently_evaluated_formula() {
    // R = scale * 1.0 / tan(h + 7.31/(h + 4.4)) arcmin, h in degrees,
    // evaluated outside this crate from the published Saemundsson (1986)
    // formula.
    //
    // Documented equivalent mutant: `replace * with / in
    // saemundsson_refraction_arcmin` cannot be killed here or anywhere. The
    // operand is the literal `1.0`, so `scale * 1.0` and `scale / 1.0` are
    // bit-identical for every input, including non-finite ones. The `* 1.0` is
    // kept in the source because it mirrors the published coefficient in the
    // formula the rustdoc cites; it is not dead weight.
    for (h, atmos, expected_arcmin) in [
        (-0.5, EXACT, 41.681_097_299_305_71),
        (-1.0, EXACT, 49.815_726_359_405_96),
        (-1.0, DENSE, 94.616_446_709_475_74),
    ] {
        let got = saemundsson_refraction_arcmin(h, atmos);
        assert!(
            (got - expected_arcmin).abs() < TOL_ARCMIN,
            "saemundsson h={h} got={got} expected={expected_arcmin}"
        );
    }
}
