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
