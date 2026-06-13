//! Tests for compatibility-profile, api-stability, and backend-matrix commands.

use pleiades_core::current_release_profile_identifiers;

use crate::cli::render_cli;
#[test]
fn profile_command_renders_catalogs() {
    let rendered = render_cli(&["compatibility-profile"]).expect("profile should render");
    let release_profiles = current_release_profile_identifiers();
    assert!(rendered.contains(&format!(
        "Compatibility profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains("Target compatibility catalog:"));
    assert!(rendered.contains("Baseline compatibility milestone:"));
    assert!(rendered.contains("Release-specific coverage beyond baseline:"));
    assert!(rendered.contains("Topocentric"));
    assert!(rendered.contains("Meridian house system"));
    assert!(rendered.contains("Horizon house system"));
    assert!(rendered.contains("Horizontal house system"));
    assert!(rendered.contains("Azimuth house system"));
    assert!(rendered.contains("Azimuthal house system"));
    assert!(rendered.contains("Whole Sign system"));
    assert!(rendered.contains("Whole Sign house system"));
    assert!(rendered.contains("Whole Sign (house 1 = Aries)"));
    assert!(rendered.contains("Whole-sign"));
    assert!(rendered.contains("Carter's poli-equatorial"));
    assert!(rendered.contains("Poli-equatorial"));
    assert!(rendered.contains("horizon/azimuth"));
    assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
    assert!(rendered.contains("Zariel"));
    assert!(rendered.contains("Krusinski/Pisa/Goelzer house system"));
    assert!(rendered.contains("Equal (cusp 1 = Asc)"));
    assert!(rendered.contains("Equal from MC"));
    assert!(rendered.contains("Equal (1=Aries) table of houses"));
    assert!(rendered.contains("Equal/MC = 10th"));
    assert!(rendered.contains("Equal Midheaven table of houses"));
    assert!(rendered.contains("Equal Midheaven house system"));
    assert!(rendered.contains("Vehlow equal"));
    assert!(rendered.contains("Equal/1=0 Aries"));
    assert!(rendered.contains("Equal (cusp 1 = 0° Aries)"));
    assert!(rendered.contains("WvA"));
    assert!(rendered.contains("Gauquelin table of sectors"));
    assert!(rendered.contains("Pullen SD (Neo-Porphyry) table of houses"));
    assert!(rendered.contains("Pullen SD (Sinusoidal Delta)"));
    assert!(rendered.contains("Makransky Sunshine"));
    assert!(rendered.contains("Sunshine table of houses, by Bob Makransky"));
    assert!(rendered.contains("Treindl Sunshine"));
    assert!(rendered.contains("Y APC houses"));
    assert!(rendered.contains("Wang"));
    assert!(rendered.contains("Aries houses"));
    assert!(rendered.contains("Fagan/Bradley"));
    assert!(rendered.contains("Usha Shashi"));
    assert!(rendered.contains("JN Bhasin"));
    assert!(rendered.contains("X, Meridian houses, Meridian table of houses, Meridian house system, ARMC, Axial Rotation, Axial rotation system, Zariel, X axial rotation system/ Meridian houses -> Meridian"));
    assert!(rendered.contains("Target ayanamsa catalog:"));
    assert!(rendered.contains("Alias mappings for built-in house systems:"));
    assert!(rendered.contains("Source-label aliases for built-in house systems:"));
    assert!(rendered.contains("Alias mappings for built-in ayanamsas:"));
    assert!(rendered.contains("Coverage summary:"));
    assert!(rendered.contains("ayanamsa sidereal metadata:"));
    assert!(rendered.contains("J2000"));
    assert!(rendered.contains("True Pushya"));
    assert!(rendered.contains("Djwhal Khul"));
    assert!(rendered.contains("True Revati"));
    assert!(rendered.contains("Babylonian (Eta Piscium)"));
    assert!(rendered.contains(
        "Babylonian/Kugler 2, Babylonian Kugler 2, Babylonian 2 -> Babylonian (Kugler 2)"
    ));
    assert!(rendered.contains(
        "Babylonian/Kugler 3, Babylonian Kugler 3, Babylonian 3 -> Babylonian (Kugler 3)"
    ));
    assert!(rendered.contains("Galactic Equator (Mula)"));
    assert!(rendered.contains("True Mula (Chandra Hari)"));
    assert!(rendered.contains("Galactic Equator (Fiorenza)"));
    assert!(rendered.contains("Galactic Equator (True)"));
    assert!(rendered.contains("Galactic Equator mid-Mula, Mula galactic equator, Galactic equator Mula -> Galactic Equator (Mula)"));
    assert!(rendered.contains("True galactic equator"));
    assert!(rendered.contains("Galactic equator true"));
    assert!(rendered.contains("Galactic Equator (IAU 1958)"));
    assert!(rendered.contains("Dhruva Galactic Center (Middle Mula)"));
    assert!(rendered.contains("Nick Anthony Fiorenza"));
    assert!(rendered.contains("Galactic Center (Cochrane)"));
    assert!(rendered.contains("Gal. Center = 0 Cap"));
    assert!(rendered.contains("Cochrane (Gal.Center = 0 Cap)"));
    assert!(rendered.contains("Galactic Center (Mardyks)"));
    assert!(rendered.contains("Skydram (Mardyks)"));
    assert!(rendered.contains("Mula Wilhelm"));
    assert!(rendered.contains("Wilhelm"));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm)"));
    assert!(rendered.contains("Galactic Center (Rgilbrand)"));
    assert!(rendered.contains("Galactic Center (Gil Brand)"));
    assert!(rendered.contains("Gil Brand"));
    assert!(rendered.contains("P.V.R. Narasimha Rao"));
    assert!(rendered.contains("Pullen SR (Sinusoidal Ratio) table of houses"));
    assert!(rendered.contains("True Citra Paksha"));
    assert!(rendered.contains("True Chitra Paksha"));
    assert!(rendered.contains("True Chitrapaksha"));
    assert!(rendered.contains("Babylonian (True Geoc)"));
    assert!(rendered.contains("Babylonian (True Topc)"));
    assert!(rendered.contains("Babylonian (True Obs)"));
    assert!(rendered.contains("Lahiri (VP285)"));
    assert!(rendered.contains("Krishnamurti (VP291)"));
    assert!(rendered.contains("Lahiri (ICRC)"));
    assert!(rendered.contains("Lahiri (1940)"));
    assert!(rendered.contains("Udayagiri"));
    assert!(rendered.contains("Valens Moon"));
    assert!(rendered.contains("Babylonian (House Obs)"));
    assert!(rendered.contains("B. V. Raman"));
    assert!(rendered.contains("Raman Ayanamsha"));
}

#[test]
fn api_stability_command_renders_the_posture() {
    let rendered = render_cli(&["api-stability"]).expect("api posture should render");
    let release_profiles = current_release_profile_identifiers();
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Stable consumer surfaces:"));
    assert!(rendered.contains("Experimental or operational surfaces:"));
}

#[test]
fn backend_matrix_command_renders_the_implemented_catalog() {
    let rendered = render_cli(&["backend-matrix"]).expect("backend matrix should render");
    assert!(rendered.contains("Implemented backend matrices"));
    assert!(rendered.contains("JPL snapshot reference backend"));
    assert!(rendered.contains("expected error classes:"));
    assert!(rendered.contains("required external data files:"));
    assert!(rendered.contains("VSOP87 planetary backend"));
    assert!(rendered.contains("ELP lunar backend"));
    assert!(rendered.contains("Packaged data backend"));
    assert!(rendered.contains("Composite routed backend"));
}
