//! compatibility profile verification tests (white-box; moved verbatim from the former `tests.rs`).

use super::*;
use pleiades_core::{current_release_profile_identifiers, Ayanamsa, HouseSystem, JulianDay};

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
    assert!(rendered.contains("Deprecation policy:"));
}

#[test]
fn api_stability_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["api-stability-summary"]).expect("api stability summary should render");
    let release_profiles = current_release_profile_identifiers();
    let api_stability = current_api_stability_profile();
    assert!(rendered.contains("API stability summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains(&format!(
            "Summary line: API stability posture: {}; stable surfaces: {}; experimental surfaces: {}; deprecation policy items: {}; intentional limits: {}",
            release_profiles.api_stability_profile_id,
            api_stability.stable_surfaces.len(),
            api_stability.experimental_surfaces.len(),
            api_stability.deprecation_policy.len(),
            api_stability.intentional_limits.len()
        )));
    assert!(rendered.contains(&format!(
        "Compatibility profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains("Stable surfaces:"));
    assert!(rendered.contains("Experimental surfaces:"));
    assert!(rendered.contains("Deprecation policy items:"));
    assert!(rendered.contains("Intentional limits:"));
    assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));
}

#[test]
fn compatibility_profile_command_renders_the_full_profile() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    let release_profiles = current_release_profile_identifiers();
    assert!(rendered.contains(&format!(
        "Compatibility profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains("Stage 6 release profile:"));
    assert!(rendered.contains(&format!(
        "Unsupported modes: {}",
        unsupported_modes_summary_for_report()
    )));
    assert!(rendered.contains("Target compatibility catalog:"));
    assert!(rendered.contains(
            "the full Swiss-Ephemeris-class house-system catalog remains the long-term compatibility goal."
        ));
    assert!(rendered.contains("Target ayanamsa catalog:"));
    assert!(rendered.contains(
        "the full Swiss-Ephemeris-class ayanamsa catalog remains the long-term compatibility goal."
    ));
    assert!(rendered.contains("Release-specific coverage beyond baseline:"));
    assert!(rendered.contains("Alias mappings for built-in house systems:"));
    assert!(rendered.contains("Source-label aliases for built-in house systems:"));
    assert!(rendered.contains("Source-label aliases for built-in ayanamsas:"));
    assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
    assert!(rendered.contains("Polich Page"));
    assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
    assert!(rendered.contains("Poli-equatorial"));
    assert!(rendered.contains("Poli-Equatorial"));
    assert!(rendered.contains("horizon/azimuth"));
    assert!(rendered.contains("Meridian table of houses"));
    assert!(rendered.contains("Meridian house system"));
    assert!(rendered.contains("Horizon house system"));
    assert!(rendered.contains("Whole-sign"));
    assert!(rendered.contains("Equal Midheaven house system"));
    assert!(rendered.contains("Equal Quadrant"));
    assert!(rendered.contains("Horizontal house system"));
    assert!(rendered.contains("Azimuth house system"));
    assert!(rendered.contains("Azimuthal house system"));
    assert!(rendered.contains("Carter's poli-equatorial"));
    assert!(rendered.contains("Krusinski/Pisa/Goelzer house system"));
    assert!(rendered.contains("Babylonian Huber"));
    assert!(rendered.contains("Babylonian (House)"));
    assert!(rendered.contains("Babylonian (Sissy)"));
    assert!(rendered.contains("Babylonian (True Topc)"));
    assert!(rendered.contains("Babylonian (True Obs)"));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm)"));
    assert!(rendered.contains("True Balarama"));
    assert!(rendered.contains("Aphoric"));
    assert!(rendered.contains("Takra"));
    assert!(rendered.contains("Galactic Equator (True)"));
    assert!(rendered.contains("Galactic Equator mid-Mula, Mula galactic equator, Galactic equator Mula -> Galactic Equator (Mula)"));
    assert!(rendered.contains("Valens Moon ayanamsa"));
}

#[test]
fn compatibility_profile_command_surfaces_recent_release_profile_entries() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Equal (MC) table of houses"));
    assert!(rendered.contains("Equal (MC) house system"));
    assert!(rendered.contains("Equal Midheaven house system"));
    assert!(rendered.contains("Equal from MC"));
    assert!(rendered.contains("Equal (from MC)"));
    assert!(rendered.contains("Equal (from MC) table of houses"));
    assert!(rendered.contains("Equal (1=Aries) table of houses"));
    assert!(rendered.contains("Equal (1=Aries) house system"));
    assert!(rendered.contains("Equal MC"));
    assert!(rendered.contains("Equal Midheaven"));
    assert!(rendered.contains("Babylonian 1"));
    assert!(rendered.contains("Babylonian 2"));
    assert!(rendered.contains("Babylonian 3"));
    assert!(rendered.contains("Vehlow Equal table of houses"));
    assert!(rendered.contains("Vehlow Equal house system"));
    assert!(rendered.contains("Vehlow equal"));
    assert!(rendered.contains("V equal Vehlow, Vehlow, Vehlow equal, Vehlow house system, Vehlow Equal house system, Vehlow-equal, Vehlow-equal table of houses, Vehlow Equal table of houses -> Vehlow Equal"));
    assert!(rendered.contains("Topocentric house system"));
    assert!(rendered.contains("Meridian house system"));
    assert!(rendered.contains("Horizon house system"));
    assert!(rendered.contains("Horizontal house system"));
    assert!(rendered.contains("Azimuth house system"));
    assert!(rendered.contains("Azimuthal house system"));
    assert!(rendered.contains("Carter's poli-equatorial"));
    assert!(rendered.contains("Krusinski/Pisa/Goelzer house system"));
    assert!(rendered.contains("Albategnius"));
    assert!(rendered.contains("Gauquelin sectors"));
    assert!(rendered.contains("Equal table of houses"));
    assert!(rendered.contains("Equal (cusp 1 = Asc)"));
    assert!(rendered.contains("Whole Sign system"));
    assert!(rendered.contains("Whole Sign house system"));
    assert!(rendered.contains("Whole Sign (house 1 = Aries)"));
    assert!(rendered.contains("Morinus house system"));
    assert!(rendered.contains("Pullen SR (Sinusoidal Ratio) table of houses"));
    assert!(rendered.contains("Pullen SD (Sinusoidal Delta)"));
    assert!(rendered.contains("Pullen SD (Sinusoidal Delta) table of houses"));
    assert!(rendered.contains("Pullen SD (Neo-Porphyry) table of houses"));
    assert!(rendered.contains("Neo-Porphyry"));
    assert!(rendered.contains("WvA"));
    assert!(rendered.contains("Equal from MC"));
    assert!(rendered.contains("Equal (from MC)"));
    assert!(rendered.contains("Equal (from MC) table of houses"));
    assert!(rendered.contains("Makransky Sunshine"));
    assert!(rendered.contains("True Citra Paksha"));
    assert!(rendered.contains("True Chitra Paksha"));
    assert!(rendered.contains("True Chitrapaksha"));
    assert!(rendered.contains("Galactic Equator (Fiorenza)"));
    assert!(rendered.contains("Nick Anthony Fiorenza"));
    assert!(rendered.contains("Galactic Center (Cochrane)"));
    assert!(rendered.contains("Galactic Center (Gil Brand)"));
    assert!(rendered.contains("Gil Brand"));
    assert!(rendered.contains("P.V.R. Narasimha Rao"));
    assert!(rendered.contains("Bob Makransky"));
    assert!(rendered.contains("Sunshine table of houses, by Bob Makransky"));
    assert!(rendered.contains("Treindl Sunshine"));
    assert!(rendered.contains("Valens Moon"));
    assert!(rendered.contains("Babylonian (House Obs)"));
    assert!(rendered.contains("Sunil Sheoran / Vedic Sheoran / Sheoran ayanamsa spellings"));
    assert!(rendered.contains("True Sheoran"));
    assert!(rendered.contains("Lahiri (VP285)"));
    assert!(rendered.contains("Krishnamurti (VP291)"));
    assert!(rendered.contains("P.V.R. Narasimha Rao"));
    assert!(rendered.contains("B. V. Raman"));
    assert!(rendered.contains("Raman Ayanamsha"));
    assert!(rendered.contains("Raman ayanamsa"));
    assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
    assert!(rendered.contains("Polich/Page"));
    assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
    assert!(rendered.contains("T topocentric"));
    assert!(rendered.contains("Poli-equatorial"));
    assert!(rendered.contains("horizon/azimuth"));
    assert!(rendered.contains("horizon/azimut"));
    assert!(rendered.contains("Horizon/Azimuth table of houses"));
    assert!(rendered.contains("U krusinski-pisa-goelzer"));
    assert!(rendered.contains("X axial rotation system/ Meridian houses"));
    assert!(rendered.contains("Zariel"));
    assert!(rendered.contains("Babylonian Huber"));
    assert!(rendered.contains("Babylonian (True Topc)"));
    assert!(rendered.contains("Babylonian (True Obs)"));
    assert!(rendered.contains("Galactic Equator (True)"));
    assert!(rendered.contains("True galactic equator"));
    assert!(rendered.contains("Galactic equator true"));
    assert!(rendered.contains("Valens Moon ayanamsa"));
    assert!(rendered.contains("Lahiri (ICRC)"));
    assert!(rendered.contains("Lahiri (1940)"));
    assert!(rendered.contains("Yukteshwar"));
    assert!(rendered.contains("True Revati"));
    assert!(rendered.contains("True Pushya"));
    assert!(rendered.contains("Equal/MC = 10th"));
    assert!(rendered.contains("Equal Midheaven table of houses"));
    assert!(rendered.contains("Vehlow Equal table of houses"));
    assert!(rendered.contains("Vehlow equal"));
    assert!(rendered.contains("Wang"));
    assert!(rendered.contains("Aries houses"));
    assert!(rendered.contains("Fagan/Bradley"));
    assert!(rendered.contains("Usha Shashi"));
}

#[test]
fn compatibility_profile_command_surfaces_additional_equal_release_labels() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Equal/MC table of houses"));
    assert!(rendered.contains("Equal/MC house system"));
    assert!(rendered.contains("Equal/1=Aries table of houses"));
    assert!(rendered.contains("Equal/1=Aries house system"));
    assert!(rendered.contains("Equal/1=0 Aries"));
    assert!(rendered.contains("Equal (cusp 1 = 0° Aries)"));
    assert!(rendered.contains("Whole Sign (house 1 = Aries) table of houses"));
}

#[test]
fn compatibility_profile_command_surfaces_reference_frame_and_zero_point_entries() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Suryasiddhanta (499 CE)"));
    assert!(rendered.contains("Aryabhata (499 CE)"));
    assert!(rendered.contains("Sassanian"));
    assert!(rendered.contains("Sasanian"));
    assert!(rendered.contains("Zij al-Shah"));
    assert!(rendered.contains("DeLuce"));
    assert!(rendered.contains("Aryabhata (522 CE)"));
    assert!(rendered.contains("PVR Pushya-paksha"));
    assert!(rendered.contains("Galactic Center (Rgilbrand)"));
    assert!(rendered.contains("Galactic Center (Mardyks)"));
    assert!(rendered.contains("Skydram/Galactic Alignment"));
    assert!(rendered.contains("Skydram (Mardyks)"));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm)"));
    assert!(rendered.contains("Galactic Center (Cochrane)"));
    assert!(rendered.contains("Gal. Center = 0 Sag"));
    assert!(rendered.contains("Gal. Center = 0 Cap"));
}

#[test]
fn compatibility_profile_command_surfaces_additional_ayanamsa_transliterations() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Aryabhatan Kaliyuga"));
    assert!(rendered.contains("Krishnamurti-Senthilathiban"));
    assert!(rendered.contains("Sri Yukteshwar"));
    assert!(rendered.contains("Shri Yukteshwar"));
    assert!(rendered.contains("De Luce"));
}

#[test]
fn compatibility_profile_command_surfaces_additional_reference_mode_entries() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Babylonian (Britton)"));
    assert!(rendered.contains("Babylonian/Britton"));
    assert!(rendered.contains("Babylonian (Aldebaran)"));
    assert!(rendered.contains("Babylonian/Aldebaran = 15 Tau"));
    assert!(rendered.contains("Babylonian (Eta Piscium)"));
    assert!(rendered.contains("Babylonian/Eta Piscium"));
    assert!(rendered.contains("Babylonian Eta Piscium"));
    assert!(rendered.contains("Eta Piscium"));
    assert!(rendered.contains("Hipparchus"));
    assert!(rendered.contains("Djwhal Khul"));
    assert!(rendered.contains("Udayagiri"));
    assert!(rendered.contains("True Mula"));
    assert!(rendered.contains("Suryasiddhanta (Mean Sun)"));
    assert!(rendered.contains("Aryabhata (Mean Sun)"));
    assert!(rendered.contains("Galactic Equator (IAU 1958)"));
    assert!(rendered.contains("Galactic Equator (Mula)"));
}

#[test]
fn compatibility_profile_command_surfaces_remaining_ayanamsa_and_reference_aliases() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("Suryasiddhanta (Revati)"));
    assert!(rendered.contains("Suryasiddhanta (Citra)"));
    assert!(rendered.contains("True Pushya (PVRN Rao)"));
    assert!(rendered.contains("Dhruva Galactic Center (Middle Mula)"));
    assert!(rendered.contains("Dhruva Galactic Center Middle Mula"));
    assert!(rendered.contains("Dhruva/Gal.Center/Mula (Wilhelm)"));
    assert!(rendered.contains("Mula Wilhelm"));
    assert!(rendered.contains("Wilhelm"));
    assert!(rendered.contains("Middle of Mula"));
}

#[test]
fn compatibility_profile_command_surfaces_house_table_code_spellings() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("A equal, E equal = A"));
    assert!(rendered.contains("D equal / MC"));
    assert!(rendered.contains("N, Equal/1=Aries"));
    assert!(rendered.contains("S, S sripati"));
    assert!(rendered.contains("I, I sunshine"));
    assert!(rendered.contains("W equal, whole sign"));
    assert!(rendered.contains("V equal Vehlow"));
    assert!(rendered.contains("T, Polich-Page"));
    assert!(rendered.contains("U, Krusinski"));
    assert!(rendered.contains("X, Meridian houses"));
    assert!(rendered.contains("Y APC houses"));
    assert!(rendered.contains("M, Morinus houses"));
    assert!(rendered.contains("G, Gauquelin"));
}

#[test]
fn compatibility_profile_command_surfaces_ayanamsa_code_spellings() {
    let rendered =
        render_cli(&["compatibility-profile"]).expect("compatibility profile should render");
    assert!(rendered.contains("J2000.0 -> J2000"));
    assert!(rendered.contains("J1900.0 -> J1900"));
    assert!(rendered.contains("B1950.0 -> B1950"));
    assert!(rendered.contains(
        "SS Revati, Suryasiddhanta Revati, Surya Siddhanta Revati -> Suryasiddhanta (Revati)"
    ));
    assert!(rendered.contains(
        "SS Citra, Suryasiddhanta Citra, Surya Siddhanta Citra -> Suryasiddhanta (Citra)"
    ));
    assert!(rendered.contains("Galact. Center = 0 Sag, Gal. Center = 0 Sag -> Galactic Center"));
    assert!(rendered.contains("Gal. Eq."));
}

#[test]
fn compatibility_profile_summary_command_renders_the_summary() {
    let rendered = render_cli(&["compatibility-profile-summary"])
        .expect("compatibility profile summary should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains("Compatibility profile summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    let coverage = metadata_coverage();
    assert!(rendered.contains("House systems:"));
    assert!(rendered.contains("House systems: 25 total (12 baseline, 13 release-specific)"));
    assert!(rendered.contains(&format!(
        "Latitude-sensitive house constraints: {}",
        profile.latitude_sensitive_house_constraints_summary_line()
    )));
    assert!(rendered.contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(rendered.contains("Ayanamsas:"));
    assert!(rendered.contains("Compatibility caveats documented:"));
    assert!(rendered.contains(&format!(
        "Unsupported modes: {}",
        unsupported_modes_summary_for_report()
    )));
    assert!(rendered.contains(profile.known_gaps[0]));
    assert!(rendered.contains(profile.known_gaps[1]));
    assert!(rendered.contains("ayanamsa sidereal metadata: 53/59 entries with both a reference epoch and offset; custom-definition-only=6 labels: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs); missing-sidereal-metadata=none"));
    assert!(rendered.contains(&format!(
        "House formula families: {}",
        profile.house_formula_families_summary_line()
    )));
    let catalog_inventory_summary = render_cli(&["catalog-inventory-summary"])
        .expect("catalog inventory summary should render");
    assert_eq!(
        catalog_inventory_summary,
        profile
            .validated_catalog_inventory_summary_line()
            .expect("catalog inventory summary should validate")
    );
    assert_eq!(
        catalog_inventory_summary,
        validated_catalog_inventory_summary_for_report()
            .expect("catalog inventory summary helper should validate")
    );
    assert_eq!(
        render_cli(&["catalog-inventory"]).expect("catalog inventory alias should render"),
        catalog_inventory_summary
    );
    let catalog_inventory_summary_error = render_cli(&["catalog-inventory-summary", "extra"])
        .expect_err("catalog inventory summary should reject extra arguments");
    assert_eq!(
        catalog_inventory_summary_error,
        "catalog-inventory-summary does not accept extra arguments"
    );
    let catalog_inventory_alias_error = render_cli(&["catalog-inventory", "extra"])
        .expect_err("catalog inventory alias should reject extra arguments");
    assert_eq!(
        catalog_inventory_alias_error,
        "catalog-inventory does not accept extra arguments"
    );
    let catalog_posture_summary =
        render_cli(&["catalog-posture-summary"]).expect("catalog posture summary should render");
    assert_eq!(
        catalog_posture_summary,
        profile
            .validated_catalog_posture_summary_line()
            .expect("catalog posture summary should validate")
    );
    assert_eq!(
        catalog_posture_summary,
        core_validated_catalog_posture_summary_for_report()
            .expect("catalog posture summary helper should validate")
    );
    assert_eq!(
        render_cli(&["catalog-posture"]).expect("catalog posture alias should render"),
        catalog_posture_summary
    );
    assert_eq!(
        render_cli(&["catalog-posture-summary", "extra"])
            .expect_err("catalog posture summary should reject extra arguments"),
        "catalog-posture-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["catalog-posture", "extra"])
            .expect_err("catalog posture alias should reject extra arguments"),
        "catalog-posture does not accept extra arguments"
    );
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_house_scope.join("; ")));
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_ayanamsa_scope.join("; ")));
    assert!(rendered.contains(&coverage.summary_line()));
    assert!(rendered.contains("ayanamsa catalog validation: ok"));
    let caveats_summary = render_cli(&["compatibility-caveats-summary"])
        .expect("compatibility caveats summary should render");
    let profile = current_compatibility_profile();
    assert!(caveats_summary.contains("Compatibility caveats summary"));
    assert!(caveats_summary.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(caveats_summary.contains("Compatibility caveats: 2"));
    assert!(caveats_summary.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
    assert!(caveats_summary.contains("Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"));
    assert!(caveats_summary.contains("Latitude-sensitive house constraints: 8 ("));
    assert!(caveats_summary.contains("Placidus ["));
    assert!(caveats_summary.contains("Koch ["));
    assert!(caveats_summary.contains("Topocentric ["));
    assert!(caveats_summary.contains("Gauquelin sectors ["));
    assert!(caveats_summary.contains("Descriptor-only ayanamsa labels: 6 (Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs))"));
    assert!(caveats_summary.contains(profile.known_gaps[0]));
    assert!(caveats_summary.contains(profile.known_gaps[1]));
    assert_eq!(
        render_cli(&["compatibility-caveats"]).expect("compatibility caveats alias should render"),
        caveats_summary
    );
    assert_eq!(
        render_cli(&["compatibility-caveats-summary", "extra"]).unwrap_err(),
        "compatibility-caveats-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["compatibility-caveats", "extra"]).unwrap_err(),
        "compatibility-caveats does not accept extra arguments"
    );
    let known_gaps_summary =
        render_cli(&["known-gaps-summary"]).expect("known gaps summary should render");
    assert_eq!(
        known_gaps_summary,
        format!("Known gaps: {}", profile.known_gaps_summary_line())
    );
    assert_eq!(
        render_cli(&["known-gaps"]).expect("known gaps alias should render"),
        known_gaps_summary
    );
    assert_eq!(
        render_cli(&["known-gaps-summary", "extra"]).unwrap_err(),
        "known-gaps-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["known-gaps", "extra"]).unwrap_err(),
        "known-gaps does not accept extra arguments"
    );
    let ayanamsa_metadata_coverage_summary = render_cli(&["ayanamsa-metadata-coverage-summary"])
        .expect("ayanamsa metadata coverage summary should render");
    assert_eq!(
        ayanamsa_metadata_coverage_summary,
        super::format_ayanamsa_metadata_coverage_for_report()
    );
    assert!(rendered.contains("Ayanamsa reference offsets: representative zero-point examples:"));
    assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
    assert!(rendered.contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
    assert!(rendered.contains("Usha Shashi: epoch=JD 2415020.5; offset=18.66096111111111°"));
    assert!(rendered.contains("Raman: epoch=JD 2415020; offset=21.01444°"));
    assert!(rendered.contains("Krishnamurti: epoch=JD 2415020; offset=22.363889°"));
    assert!(rendered.contains("Fagan/Bradley: epoch=JD 2433282.42346; offset=24.042044444°"));
    assert!(rendered.contains("True Chitra: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("True Revati: epoch=JD 1926902.658267; offset=0°"));
    assert!(rendered.contains("True Mula: epoch=JD 1805889.671313; offset=0°"));
    assert!(rendered.contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
    assert!(rendered.contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
    assert!(rendered.contains("Yukteshwar: epoch=JD 2451545; offset=22.6288889°"));
    assert!(rendered.contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
    assert!(rendered.contains("J2000: epoch=JD 2451545; offset=23.85317778°"));
    assert!(rendered.contains("J1900: epoch=JD 2415020; offset=0°"));
    assert!(rendered.contains("B1950: epoch=JD 2433281.5; offset=0°"));
    assert!(rendered.contains("Babylonian (Kugler 2): epoch=JD 1797039.20682; offset=0°"));
    assert!(rendered.contains("Babylonian (Kugler 3): epoch=JD 1774637.420172; offset=0°"));
    assert!(
        rendered.contains("Babylonian (Huber): epoch=JD 1721171.5; offset=-0.12055555555555555°")
    );
    assert!(rendered.contains("Babylonian (Eta Piscium): epoch=JD 1807871.964797; offset=0°"));
    assert!(rendered.contains("Babylonian (Aldebaran): epoch=JD 1801643.133503; offset=0°"));
    assert!(rendered.contains("Lahiri (VP285): epoch=JD 1825235.164583; offset=0°"));
    assert!(rendered.contains("Krishnamurti (VP291): epoch=JD 1827424.663554; offset=0°"));
    assert!(rendered.contains("Sheoran: epoch=JD 1789947.090881; offset=0°"));
    assert!(rendered.contains("True Sheoran: epoch="));
    assert!(rendered.contains("Hipparchus: epoch=JD 1674484; offset=-9.333333333333334°"));
    assert!(rendered.contains("Djwhal Khul: epoch=JD 1706703.948006; offset=0°"));
    assert!(rendered.contains("Galactic Center: epoch="));
    assert!(rendered.contains("Galactic Center (Rgilbrand): epoch="));
    assert!(rendered.contains("Galactic Center (Mardyks): epoch="));
    assert!(rendered.contains("Galactic Center (Cochrane): epoch="));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm): epoch="));
    assert!(rendered.contains("Dhruva Galactic Center (Middle Mula): epoch="));
    assert!(rendered.contains("Galactic Equator (IAU 1958): epoch=JD 1667118.376332; offset=0°"));
    assert!(rendered.contains("Galactic Equator (True): epoch=JD 1665728.603158; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Mula): epoch=JD 1840527.426262; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
    assert!(rendered.contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
    assert!(rendered.contains("Suryasiddhanta (499 CE): epoch=JD 1903396.8128653935; offset=0°"));
    assert!(rendered.contains("Suryasiddhanta (Mean Sun): epoch=JD 1909045.584433; offset=0°"));
    assert!(rendered.contains("Aryabhata (Mean Sun): epoch=JD 1909650.815331; offset=0°"));
    assert!(rendered.contains("Aryabhata (522 CE): epoch=JD 1911797.740782; offset=0°"));
    assert!(rendered.contains("Release-specific house-system canonical names: 13 (Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, Gauquelin sectors)"));
    assert!(rendered.contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(rendered.contains("Release-specific ayanamsa canonical names:"));
    assert!(rendered.contains("Release-specific ayanamsa canonical names: 54 (True Citra, J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), Galactic Center, Galactic Equator, True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Valens Moon)"));
    assert!(rendered.contains("Custom-definition labels: 9"));
    assert!(rendered.contains(&format!(
        "Custom-definition label names: {}",
        profile.custom_definition_labels.join(", ")
    )));
    assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
    assert!(rendered.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
    assert!(rendered.contains("Compatibility caveats: 2"));
    assert!(rendered.contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));
}

#[test]
fn compatibility_profile_summary_text_validation_rejects_split_drift() {
    let profile = validated_compatibility_profile_for_report()
        .expect("compatibility profile should validate");
    let release_profiles = validated_release_profile_identifiers_for_report()
        .expect("release profile identifiers should validate");
    let rendered = render_compatibility_profile_summary_text();
    let expected_house_line = format!(
        "House systems: {} total ({} baseline, {} release-specific)",
        profile.house_systems.len(),
        profile.baseline_house_systems.len(),
        profile.release_house_systems.len()
    );
    let expected_ayanamsa_line = format!(
        "Ayanamsas: {} total ({} baseline, {} release-specific)",
        profile.ayanamsas.len(),
        profile.baseline_ayanamsas.len(),
        profile.release_ayanamsas.len()
    );

    assert!(
        validate_compatibility_profile_summary_text(&rendered, &profile, &release_profiles,)
            .is_ok()
    );

    let tampered = rendered.replacen(&expected_house_line, "House systems: split omitted", 1);
    let error = validate_compatibility_profile_summary_text(&tampered, &profile, &release_profiles)
        .expect_err("tampered compatibility profile summary should fail validation");
    assert!(error.contains("baseline/release split mismatch"));

    let tampered = rendered.replacen(&expected_ayanamsa_line, "Ayanamsas: split omitted", 1);
    let error = validate_compatibility_profile_summary_text(&tampered, &profile, &release_profiles)
        .expect_err("tampered compatibility profile summary should fail validation");
    assert!(error.contains("baseline/release split mismatch"));
}

#[test]
fn compatibility_profile_report_helper_validates_the_current_profile() {
    let profile = validated_compatibility_profile_for_report()
        .expect("compatibility profile should validate");
    assert_eq!(
        profile.profile_id,
        current_compatibility_profile().profile_id
    );
}

#[test]
fn compatibility_profile_verification_command_checks_the_catalogs() {
    let rendered = render_cli(&["verify-compatibility-profile"])
        .expect("compatibility profile verification should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains("Compatibility profile verification"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains("House systems verified: 25 descriptors, 181 labels"));
    assert!(rendered.contains(&format!(
        "House code aliases verified: {} short-form labels",
        profile.house_code_alias_count()
    )));
    assert!(rendered.contains(&format!(
            "Alias uniqueness checks: house={} aliases, ayanamsa={} aliases; exact and case-insensitive labels verified",
            profile
                .house_systems
                .iter()
                .map(|entry| 1 + entry.aliases.len())
                .sum::<usize>()
                - profile.house_systems.len(),
            profile
                .ayanamsas
                .iter()
                .map(|entry| 1 + entry.aliases.len())
                .sum::<usize>()
                - profile.ayanamsas.len()
        )));
    assert!(rendered.contains(
            "Latitude-sensitive house systems verified: 8 descriptors, 8 labels (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        ));
    assert!(rendered.contains("Ayanamsas verified: 59 descriptors, 245 labels"));
    assert!(rendered.contains("House formula families verified: Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign"));
    assert!(rendered.contains(
            "Ayanamsa reference metadata verified: 53 descriptors with epoch/offset metadata, 6 metadata gaps"
        ));
    assert!(rendered.contains(&format!(
            "Catalog posture: house systems=25 descriptors (8 constrained, 17 unconstrained); ayanamsas=59 descriptors (53 metadata-bearing, 6 descriptor-only); ayanamsa alias-bearing entries={}; ayanamsa metadata gaps=6; custom-definition labels=9; custom-definition ayanamsa labels=6; known gaps={}",
            profile
                .ayanamsas
                .iter()
                .filter(|entry| !entry.aliases.is_empty())
                .count(),
            profile.known_gaps_summary_line()
        )));
    assert!(rendered.contains(&format!(
        "Custom-definition labels verified: {} labels, all remain custom-definition territory",
        profile.custom_definition_labels.len()
    )));
    assert!(rendered.contains("Baseline/release slices:"));
    assert!(rendered.contains("Release-specific house-system canonical names verified: 13 (Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, Gauquelin sectors)"));
    assert!(rendered.contains("Release-specific ayanamsa canonical names verified: 54 (True Citra, J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), Galactic Center, Galactic Equator, True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Valens Moon)"));
    assert!(rendered.contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"));
    assert!(rendered.contains(&format!(
        "Custom-definition label names verified: {}",
        profile.custom_definition_labels.join(", ")
    )));
    assert!(rendered.contains(&format!(
            "Custom-definition ayanamsa labels verified: {} labels, all remain custom-definition territory",
            profile.custom_definition_ayanamsa_labels().len()
        )));
    assert!(rendered.contains(&format!(
        "Custom-definition ayanamsa label names verified: {}",
        profile.custom_definition_ayanamsa_labels().join(", ")
    )));
    assert!(rendered.contains(&format!(
        "Release notes documented: {} entries",
        profile.release_notes.len()
    )));
    assert!(rendered.contains(&format!(
        "Validation reference points documented: {} entries",
        profile.validation_reference_points.len()
    )));
    assert!(rendered.contains(&format!(
        "Custom-definition labels verified: {} labels, all remain custom-definition territory",
        profile.custom_definition_labels.len()
    )));
    assert!(rendered.contains(&format!(
        "Custom-definition label names verified: {}",
        profile.custom_definition_labels.join(", ")
    )));
    assert!(rendered.contains(&format!(
        "Compatibility caveats documented: {}",
        profile.known_gaps.len()
    )));
}

#[test]
fn compatibility_profile_verification_summary_renders_consistently() {
    let summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();

    assert_eq!(
        summary.profile_id,
        release_profiles.compatibility_profile_id
    );
    assert_eq!(
        summary.house_system_descriptor_count,
        profile.house_systems.len()
    );
    assert_eq!(
        summary.house_code_alias_count,
        profile.house_code_alias_count()
    );
    assert_eq!(
        summary.house_code_aliases_summary,
        profile.house_code_aliases_summary_line()
    );
    assert_eq!(
        summary.house_system_alias_count,
        profile
            .house_systems
            .iter()
            .map(|entry| 1 + entry.aliases.len())
            .sum::<usize>()
            - profile.house_systems.len()
    );
    assert_eq!(summary.ayanamsa_descriptor_count, profile.ayanamsas.len());
    assert_eq!(
        summary.baseline_house_system_count,
        profile.baseline_house_systems.len()
    );
    assert_eq!(
        summary.release_house_system_count,
        profile.release_house_systems.len()
    );
    assert_eq!(
        summary.baseline_ayanamsa_count,
        profile.baseline_ayanamsas.len()
    );
    assert_eq!(
        summary.release_ayanamsa_count,
        profile.release_ayanamsas.len()
    );
    assert_eq!(summary.release_note_count, profile.release_notes.len());
    assert_eq!(
        summary.validation_reference_point_count,
        profile.validation_reference_points.len()
    );
    assert_eq!(
        summary.custom_definition_label_count,
        profile.custom_definition_labels.len()
    );
    assert_eq!(
        summary.custom_definition_label_names,
        profile.custom_definition_labels.join(", ")
    );
    assert_eq!(
        summary.custom_definition_ayanamsa_label_count,
        profile.custom_definition_ayanamsa_labels().len()
    );
    assert_eq!(
        summary.custom_definition_ayanamsa_label_names,
        profile.custom_definition_ayanamsa_labels().join(", ")
    );
    assert_eq!(
        summary.ayanamsa_alias_count,
        profile
            .ayanamsas
            .iter()
            .map(|entry| 1 + entry.aliases.len())
            .sum::<usize>()
            - profile.ayanamsas.len()
    );
    assert_eq!(
        summary.house_formula_family_names,
        profile.house_formula_family_names().join(", ")
    );
    assert_eq!(
        summary.ayanamsa_metadata_count,
        profile
            .ayanamsas
            .iter()
            .filter(|entry| entry.has_sidereal_metadata())
            .count()
    );
    assert_eq!(
        summary.ayanamsa_metadata_gap_count,
        profile.ayanamsas.len() - summary.ayanamsa_metadata_count
    );
    assert_eq!(summary.compatibility_caveat_count, profile.known_gaps.len());
    summary
        .validate()
        .expect("fresh compatibility profile verification summary should validate");
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_summary_line().unwrap(),
        summary.summary_line()
    );
    assert_eq!(
        verify_compatibility_profile().unwrap(),
        summary.summary_line()
    );
    assert!(summary
        .summary_line()
        .contains("Compatibility profile verification"));
    assert!(summary.summary_line().contains(&format!(
        "House code aliases verified: {} short-form labels",
        profile.house_code_alias_count()
    )));
    assert!(summary.summary_line().contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(summary.summary_line().contains(&format!(
            "Alias uniqueness checks: house={} aliases, ayanamsa={} aliases; exact and case-insensitive labels verified",
            summary.house_system_alias_count,
            summary.ayanamsa_alias_count
        )));
    assert!(summary
            .summary_line()
            .contains("House formula families verified: Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign"));
    assert!(summary.summary_line().contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"));
    assert!(summary.summary_line().contains(&format!(
            "Ayanamsa reference metadata verified: {} descriptors with epoch/offset metadata, {} metadata gaps",
            profile
                .ayanamsas
                .iter()
                .filter(|entry| entry.has_sidereal_metadata())
                .count(),
            profile.ayanamsas.len() - profile
                .ayanamsas
                .iter()
                .filter(|entry| entry.has_sidereal_metadata())
                .count()
        )));
    assert!(summary.summary_line().contains(&format!(
            "Catalog posture: house systems=25 descriptors (8 constrained, 17 unconstrained); ayanamsas=59 descriptors (53 metadata-bearing, 6 descriptor-only); ayanamsa alias-bearing entries={}; ayanamsa metadata gaps=6; custom-definition labels=9; custom-definition ayanamsa labels=6; known gaps={}",
            profile
                .ayanamsas
                .iter()
                .filter(|entry| !entry.aliases.is_empty())
                .count(),
            profile.known_gaps_summary_line()
        )));
    assert!(
            summary
                .summary_line()
                .contains("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented")
        );
}

#[test]
fn compatibility_profile_verification_summary_validation_rejects_stale_fields() {
    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.release_ayanamsa_canonical_names = "stale summary".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("stale compatibility profile verification summary should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("release ayanamsa canonical names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.release_ayanamsa_canonical_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale compatibility profile verification summary should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("release ayanamsa canonical names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.release_house_canonical_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale release house-system canonical names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("release house-system canonical names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.house_system_alias_count = 0;

    let error = summary
        .validate()
        .expect_err("stale house-system alias count should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house-system alias count mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.house_code_alias_count = 0;

    let error = summary
        .validate()
        .expect_err("stale house-code alias count should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house-code alias count mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.house_code_aliases_summary = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale house-code alias summary should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house-code aliases mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.custom_definition_label_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale custom-definition label names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("custom-definition label names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.custom_definition_ayanamsa_label_count = 0;

    let error = summary
        .validate()
        .expect_err("stale custom-definition ayanamsa label count should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("custom-definition ayanamsa label count mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.custom_definition_ayanamsa_label_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale custom-definition ayanamsa label names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("custom-definition ayanamsa label names mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.house_formula_family_names = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale house formula families should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house formula families mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.release_posture = "stale summary".to_string();

    let error = summary
        .validate()
        .expect_err("stale release posture should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("release posture mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.ayanamsa_alias_count = 0;

    let error = summary
        .validate()
        .expect_err("stale ayanamsa alias count should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("ayanamsa alias count mismatch"));

    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.ayanamsa_metadata_count = 0;

    let error = summary
        .validate()
        .expect_err("stale ayanamsa metadata counts should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("ayanamsa metadata count mismatch"));
}

#[test]
fn compatibility_profile_verification_summary_validation_rejects_stale_latitude_sensitive_house_systems(
) {
    let mut summary = compatibility_profile_verification_summary()
        .expect("compatibility profile verification summary should render");
    summary.latitude_sensitive_house_systems.reverse();

    let error = summary
        .validate()
        .expect_err("stale latitude-sensitive house systems should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("latitude-sensitive house systems mismatch"));
}

#[test]
fn descriptor_names_summary_validation_rejects_blank_entries() {
    let summary = DescriptorNamesSummary {
        names: vec!["Equal (MC)", "   "],
    };

    let error = summary
        .validate()
        .expect_err("blank descriptor names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("blank name"));
}

#[test]
fn descriptor_names_summary_validation_rejects_case_insensitive_duplicates() {
    let summary = DescriptorNamesSummary {
        names: vec!["Equal (MC)", "equal (mc)"],
    };

    let error = summary
        .validate()
        .expect_err("case-insensitive duplicate descriptor names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("case-insensitive duplicate name"));
}

#[test]
fn validate_name_sequence_rejects_whitespace_padded_owned_names() {
    let names = [String::from("Equal (MC)"), String::from(" release family ")];

    let error = validate_name_sequence(
        "compatibility profile house formula families",
        names.iter().map(String::as_str),
    )
    .expect_err("whitespace-padded owned names should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("surrounding whitespace"));
}

#[test]
fn compatibility_profile_partition_checks_cover_the_current_catalog() {
    let profile = current_compatibility_profile();

    verify_profile_catalog_partitions_are_disjoint(
        "house-system",
        profile.baseline_house_systems,
        profile.release_house_systems,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect("the current house catalog partitions should remain disjoint");

    verify_profile_catalog_partitions_are_disjoint(
        "ayanamsa",
        profile.baseline_ayanamsas,
        profile.release_ayanamsas,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect("the current ayanamsa catalog partitions should remain disjoint");
}

#[test]
fn compatibility_profile_partition_checks_reject_overlapping_labels() {
    let house_baseline = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus house system"],
        "Quadrant system used for partition-overlap coverage.",
        true,
    )];
    let house_release = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Koch,
        "Koch",
        &["Placidus"],
        "Quadrant system used for partition-overlap coverage.",
        true,
    )];

    let error = verify_profile_catalog_partitions_are_disjoint(
        "house-system",
        &house_baseline,
        &house_release,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect_err("overlapping house-system labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("baseline and release slices overlap on label 'Placidus'"));

    let ayanamsa_baseline = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &["Lahiri ayanamsa"],
        "Sidereal mode used for partition-overlap coverage.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
    )];
    let ayanamsa_release = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::Raman,
        "Raman",
        &["Lahiri"],
        "Sidereal mode used for partition-overlap coverage.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(pleiades_core::Angle::from_degrees(21.014_44)),
    )];

    let error = verify_profile_catalog_partitions_are_disjoint(
        "ayanamsa",
        &ayanamsa_baseline,
        &ayanamsa_release,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect_err("overlapping ayanamsa labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("baseline and release slices overlap on label 'Lahiri'"));
}

#[test]
fn compatibility_profile_partition_checks_reject_case_normalized_alias_overlaps() {
    #[derive(Clone, Copy)]
    struct Entry {
        canonical_name: &'static str,
        aliases: &'static [&'static str],
    }

    let baseline = [Entry {
        canonical_name: "Lahiri",
        aliases: &["Lahiri ayanamsa"],
    }];
    let release = [Entry {
        canonical_name: "Raman",
        aliases: &["lahiri"],
    }];

    let error = verify_profile_catalog_partitions_are_disjoint(
        "ayanamsa",
        &baseline,
        &release,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )
    .expect_err("case-normalized overlapping alias labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("baseline and release slices overlap on label 'lahiri'"));
}

#[test]
fn descriptor_names_summary_formats_empty_single_and_multiple_entries() {
    #[derive(Clone, Copy)]
    struct Item(&'static str);

    let empty = summarize_descriptor_names(&[] as &[Item], |item| item.0);
    assert_eq!(empty.summary_line(), "0 (none)");
    assert_eq!(empty.to_string(), "0 (none)");

    let single = summarize_descriptor_names(&[Item("Alpha")], |item| item.0);
    assert_eq!(single.summary_line(), "1 (Alpha)");
    assert_eq!(single.to_string(), "1 (Alpha)");

    let multiple = summarize_descriptor_names(&[Item("Alpha"), Item("Beta")], |item| item.0);
    assert_eq!(multiple.summary_line(), "2 (Alpha, Beta)");
    assert_eq!(multiple.to_string(), "2 (Alpha, Beta)");
}

#[test]
fn compatibility_profile_verification_rejects_duplicate_house_labels() {
    let descriptors = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus"],
        "Quadrant system used for duplicate-label verification coverage.",
        true,
    )];

    let error = verify_house_system_aliases(&descriptors)
        .expect_err("duplicate labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("house-system labels are not unique"));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_house_labels() {
    let descriptors = [
        pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::Placidus,
            "Placidus",
            &[],
            "Quadrant system used for case-insensitive duplicate-label coverage.",
            true,
        ),
        pleiades_houses::HouseSystemDescriptor::new(
            HouseSystem::Koch,
            "placidus",
            &[],
            "Quadrant system used for case-insensitive duplicate-label coverage.",
            true,
        ),
    ];

    let error = verify_house_system_aliases(&descriptors)
        .expect_err("case-insensitive duplicate labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("labels are not unique ignoring case"));
}

#[test]
fn compatibility_profile_verification_allows_case_insensitive_duplicate_house_aliases_within_entry()
{
    let descriptors = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["placidus"],
        "Quadrant system used for intra-entry duplicate-label coverage.",
        true,
    )];

    let checked = verify_house_system_aliases(&descriptors)
        .expect("case-insensitive duplicate aliases within one descriptor should remain allowed");
    assert_eq!(checked, 2);
}

#[test]
fn compatibility_profile_verification_uses_display_labels_for_alias_mismatches() {
    let house_descriptors = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::EqualAries,
        "Equal (MC)",
        &[],
        "Quadrant system used for display-label mismatch coverage.",
        false,
    )];

    let error = verify_house_system_aliases(&house_descriptors)
        .expect_err("mismatched house labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("canonical label 'Equal (MC)' should resolve to Equal (1=Aries)"));
    assert!(!error.message.contains("EqualMidheaven"));
    assert!(!error.message.contains("EqualAries"));

    let ayanamsa_descriptors = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::TrueCitra,
        "True Chitra",
        &[],
        "Sidereal mode used for display-label mismatch coverage.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(pleiades_core::Angle::from_degrees(23.0)),
    )];

    let error = verify_ayanamsa_aliases(&ayanamsa_descriptors)
        .expect_err("mismatched ayanamsa labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("canonical label 'True Chitra' should resolve to True Citra"));
    assert!(!error.message.contains("TrueChitra"));
    assert!(!error.message.contains("TrueCitra"));
}

#[test]
fn compatibility_profile_verification_rejects_missing_descriptor_notes() {
    let descriptors = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[],
        " ",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
    )];

    let error = verify_ayanamsa_aliases(&descriptors)
        .expect_err("missing descriptor notes should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("missing notes metadata"));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_canonical_names() {
    let error = ensure_profile_descriptor_metadata(
        "house-system",
        " Placidus ",
        "Quadrant system used for whitespace-padded metadata coverage.",
    )
    .expect_err("whitespace-padded canonical names should fail profile verification");

    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its canonical name"));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_notes() {
    let error = ensure_profile_descriptor_metadata(
        "ayanamsa",
        "Lahiri",
        " whitespace-padded notes metadata ",
    )
    .expect_err("whitespace-padded notes should fail profile verification");

    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its notes metadata"));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_house_aliases() {
    let descriptors = [pleiades_houses::HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &[" Placidus alias "],
        "Quadrant system used for whitespace-padded alias coverage.",
        true,
    )];

    let error = verify_house_system_aliases(&descriptors)
        .expect_err("whitespace-padded house aliases should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its label"));
    assert!(error.message.contains(" Placidus alias "));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_ayanamsa_aliases() {
    let descriptors = [pleiades_ayanamsa::AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[" Lahiri alias "],
        "Ayanamsa used for whitespace-padded alias coverage.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
    )];

    let error = verify_ayanamsa_aliases(&descriptors)
        .expect_err("whitespace-padded ayanamsa aliases should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its label"));
    assert!(error.message.contains(" Lahiri alias "));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_labels() {
    let mut seen_labels = BTreeSet::new();
    let mut seen_labels_case_insensitive = BTreeMap::new();
    let error = ensure_unique_profile_label(
        "custom-definition",
        "  custom delta  ",
        "custom delta",
        &mut seen_labels,
        &mut seen_labels_case_insensitive,
    )
    .expect_err("whitespace-padded labels should fail profile verification");

    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("contains surrounding whitespace in its label"));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_ayanamsa_labels() {
    let descriptors = [
        pleiades_ayanamsa::AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Lahiri",
            &[],
            "Sidereal mode used for case-insensitive duplicate-label coverage.",
            Some(JulianDay::from_days(2_435_553.5)),
            Some(pleiades_core::Angle::from_degrees(23.245_524_743)),
        ),
        pleiades_ayanamsa::AyanamsaDescriptor::new(
            Ayanamsa::TrueRevati,
            "lahiri",
            &[],
            "Sidereal mode used for case-insensitive duplicate-label coverage.",
            Some(JulianDay::from_days(2_444_907.5)),
            Some(pleiades_core::Angle::from_degrees(0.0)),
        ),
    ];

    let error = verify_ayanamsa_aliases(&descriptors)
        .expect_err("case-insensitive duplicate labels should fail profile verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("labels are not unique ignoring case"));
}

#[test]
fn compatibility_profile_verification_rejects_custom_definition_labels_that_resolve_to_builtins() {
    let labels = ["Placidus"];

    let error = verify_custom_definition_labels(&labels)
        .expect_err("custom-definition labels should stay outside built-ins");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("should remain unresolved as a built-in house system or ayanamsa"));
}

#[test]
fn compatibility_profile_verification_rejects_custom_definition_labels_that_resolve_to_ayanamsas() {
    let labels = ["Lahiri"];

    let error = verify_custom_definition_labels(&labels)
        .expect_err("custom-definition labels should stay outside built-in ayanamsas");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("should remain unresolved as a built-in house system or ayanamsa"));
}

#[test]
fn compatibility_profile_verification_allows_intentional_ayanamsa_homographs() {
    let labels = INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS;

    let checked = verify_custom_definition_labels(labels)
        .expect("intentional custom-definition homographs should remain allowed");
    assert_eq!(checked, labels.len());
    assert!(is_intentional_custom_definition_ayanamsa_homograph(
        labels[0]
    ));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_custom_definition_labels()
{
    let labels = ["custom delta", "Custom Delta"];

    let error = verify_custom_definition_labels(&labels)
        .expect_err("case-insensitive duplicate custom-definition labels should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("custom-definition entries are not unique"));
}

#[test]
fn compatibility_profile_verification_rejects_blank_release_note_entries() {
    let entries = ["release note", "   "];

    let error = verify_profile_text_section("release-note", &entries)
        .expect_err("blank release-note entries should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("entry is blank"));
}

#[test]
fn compatibility_profile_verification_rejects_duplicate_compatibility_caveats() {
    let entries = ["known gap", "known gap"];

    let error = verify_profile_text_section("compatibility-caveat", &entries)
        .expect_err("duplicate caveat entries should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("entries are not unique"));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_release_notes() {
    let entries = ["shared release text", "Shared Release Text"];

    let error = verify_profile_text_section("release-note", &entries)
        .expect_err("case-insensitive duplicate release notes should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("entries are not unique ignoring case"));
    assert!(error.message.contains("Shared Release Text"));
}

#[test]
fn compatibility_profile_verification_rejects_duplicate_text_across_sections() {
    let error = verify_profile_text_sections_are_disjoint(&[
        ("release-note", &["shared release text"]),
        ("compatibility-caveat", &["shared release text"]),
    ])
    .expect_err("duplicate prose across sections should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains(
            "duplicate entry 'shared release text' appears in both release-note and compatibility-caveat"
        ));
}

#[test]
fn compatibility_profile_verification_rejects_case_insensitive_duplicate_text_across_sections() {
    let error = verify_profile_text_sections_are_disjoint(&[
        ("release-note", &["shared release text"]),
        ("compatibility-caveat", &["Shared Release Text"]),
    ])
    .expect_err("case-insensitive duplicate prose across sections should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("not unique ignoring case"));
    assert!(error.message.contains("release-note"));
    assert!(error.message.contains("compatibility-caveat"));
}

#[test]
fn compatibility_profile_verification_rejects_whitespace_padded_text_across_sections() {
    let error = verify_profile_text_sections_are_disjoint(&[
        ("release-note", &["shared release text "]),
        ("compatibility-caveat", &["shared release text"]),
    ])
    .expect_err("whitespace-padded prose across sections should fail verification");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error.message.contains("contains surrounding whitespace"));
    assert!(error.message.contains("release-note"));
}

#[test]
fn compatibility_profile_verification_validates_target_scope_sections() {
    let profile = current_compatibility_profile();

    verify_profile_text_section("target-house-scope", profile.target_house_scope)
        .expect("target house scope should validate");
    verify_profile_text_section("target-ayanamsa-scope", profile.target_ayanamsa_scope)
        .expect("target ayanamsa scope should validate");
    verify_profile_text_sections_are_disjoint(&[
        ("target-house-scope", profile.target_house_scope),
        ("target-ayanamsa-scope", profile.target_ayanamsa_scope),
        ("release-note", profile.release_notes),
        (
            "validation-reference-point",
            profile.validation_reference_points,
        ),
        ("compatibility-caveat", profile.known_gaps),
    ])
    .expect("target scope prose should remain disjoint from release prose");
    assert_eq!(
        profile
            .validated_target_house_scope_summary_line()
            .expect("target house scope summary should validate"),
        profile.target_house_scope.join("; ")
    );
    assert_eq!(
        profile
            .validated_target_ayanamsa_scope_summary_line()
            .expect("target ayanamsa scope summary should validate"),
        profile.target_ayanamsa_scope.join("; ")
    );
}
