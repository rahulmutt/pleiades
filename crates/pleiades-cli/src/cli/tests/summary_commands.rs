//! Tests for the many compact summary commands.

use pleiades_core::{current_compatibility_profile, current_release_profile_identifiers};
use pleiades_validate::{house_validation_summary_for_report, render_cli as validate_render_cli};

use super::super::test_support::packaged_artifact_access_report_line;
use crate::cli::render_cli;

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn summary_commands_render_compact_reports() {
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();

    let compatibility = render_cli(&["compatibility-profile-summary"])
        .expect("compatibility summary should render");
    assert_eq!(render_cli(&["profile-summary"]).unwrap(), compatibility);
    assert!(compatibility.contains("Compatibility profile summary"));
    assert!(compatibility.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(compatibility.contains("House systems: 25 total"));
    assert!(compatibility.contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(compatibility.contains("Compatibility caveats documented:"));
    assert!(compatibility.contains(profile.known_gaps[0]));
    assert!(compatibility.contains(profile.known_gaps[1]));
    assert!(
        compatibility.contains("Compatibility profile verification: verify-compatibility-profile")
    );
    assert!(compatibility.lines().any(|line| {
        line == "Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"
    }));
    assert!(compatibility.contains("Release notes summary: release-notes-summary"));
    assert!(compatibility.contains("Release summary: release-summary"));
    assert!(compatibility.contains("Release checklist summary: release-checklist-summary"));
    assert!(compatibility.contains("Release bundle verification: verify-release-bundle"));
    let caveats = render_cli(&["compatibility-caveats-summary"])
        .expect("compatibility caveats summary should render");
    assert!(caveats.contains("Compatibility caveats summary"));
    assert!(caveats.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(caveats.contains("Compatibility caveats: 2"));
    assert!(caveats.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
    assert!(caveats.contains("Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"));
    assert!(caveats.contains("Descriptor-only ayanamsa labels: 6 (Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs))"));
    assert!(caveats.contains(profile.known_gaps[0]));
    assert!(caveats.contains(profile.known_gaps[1]));
    assert_eq!(render_cli(&["compatibility-caveats"]).unwrap(), caveats);
    assert_eq!(
        render_cli(&["compatibility-caveats-summary", "extra"]).unwrap_err(),
        "compatibility-caveats-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["compatibility-caveats", "extra"]).unwrap_err(),
        "compatibility-caveats does not accept extra arguments"
    );
    assert!(compatibility.contains("Babylonian (Eta Piscium)"));
    assert!(compatibility.contains("Galactic Equator (Mula)"));
    assert!(compatibility.contains("Galactic Equator (Fiorenza)"));
    assert!(compatibility.contains("JN Bhasin"));
    assert!(compatibility.contains("Lahiri (ICRC)"));
    assert!(compatibility.contains("Udayagiri"));
    assert!(compatibility.contains("Valens Moon"));
    assert!(compatibility.contains("Babylonian (House Obs)"));
    assert!(compatibility.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));

    let verification = render_cli(&["verify-compatibility-profile"])
        .expect("compatibility profile verification should render");
    assert!(verification.contains("Compatibility profile verification"));
    assert!(verification.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(verification.contains("House systems verified:"));
    assert!(verification.contains(&format!(
        "House code aliases verified: {} short-form labels",
        profile.house_code_alias_count()
    )));
    assert!(verification.contains("Ayanamsas verified:"));
    assert!(verification.contains(
        "House formula families verified: Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign"
    ));
    assert!(verification.contains(&format!(
        "Custom-definition label names verified: {}",
        profile.custom_definition_labels.join(", ")
    )));
    assert!(verification.contains(&format!(
        "Custom-definition ayanamsa labels verified: {} labels, all remain custom-definition territory",
        profile.custom_definition_ayanamsa_labels().len()
    )));
    assert!(verification.contains(&format!(
        "Custom-definition ayanamsa label names verified: {}",
        profile.custom_definition_ayanamsa_labels().join(", ")
    )));
    assert!(verification.contains("Ayanamsa reference metadata verified: "));
    assert!(verification.contains(
        "Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented"
    ));

    let backend_matrix =
        render_cli(&["backend-matrix-summary"]).expect("backend matrix summary should render");
    assert!(backend_matrix.contains("Backend matrix summary"));
    assert!(backend_matrix.contains("Backends: 5"));
    assert!(backend_matrix.contains("Accuracy classes: Exact: 1"));
    assert!(backend_matrix.contains("Reference snapshot dense boundary day:"));
    assert!(backend_matrix.contains("Reference major-body bridge evidence:"));
    assert!(backend_matrix.contains("Selected asteroid bridge evidence:"));
    assert!(backend_matrix.contains("JPL source corpus contract: JPL evidence classification:"));
    assert!(backend_matrix.contains("Release notes summary: release-notes-summary"));
    assert!(
        backend_matrix.contains("Compatibility profile verification: verify-compatibility-profile")
    );
    assert!(backend_matrix.contains("Release bundle verification: verify-release-bundle"));
    assert!(backend_matrix
        .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
    assert!(backend_matrix.contains("Release checklist summary: release-checklist-summary"));

    let comparison_corpus = render_cli(&["comparison-corpus-summary"])
        .expect("comparison corpus summary should render");
    assert!(comparison_corpus.contains("Comparison corpus summary"));
    assert!(comparison_corpus.contains("name: JPL Horizons release-grade comparison window"));
    assert!(
        comparison_corpus.contains("release-grade guard: Pluto excluded from tolerance evidence")
    );
    assert_eq!(
        comparison_corpus,
        validate_render_cli(&["comparison-corpus-summary"])
            .expect("comparison corpus summary should match validation output")
    );
    assert_eq!(
        render_cli(&["comparison-corpus"]).expect("comparison corpus alias should render"),
        comparison_corpus
    );
    assert_eq!(
        validate_render_cli(&["comparison-corpus"])
            .expect("comparison corpus alias should match validation output"),
        comparison_corpus
    );
    assert_eq!(
        render_cli(&["comparison-corpus-summary", "extra"])
            .expect_err("comparison corpus summary should reject extra arguments"),
        "comparison-corpus-summary does not accept extra arguments"
    );

    let comparison_guard = render_cli(&["comparison-corpus-release-guard-summary"])
        .expect("comparison corpus release guard summary should render");
    assert!(comparison_guard.contains("Comparison corpus release-grade guard summary"));
    assert!(
        comparison_guard.contains("Release-grade guard: Pluto excluded from tolerance evidence")
    );
    assert_eq!(
        comparison_guard,
        validate_render_cli(&["comparison-corpus-release-guard-summary"])
            .expect("comparison corpus release guard summary should match validation output")
    );
    assert_eq!(
        render_cli(&["comparison-corpus-release-guard"])
            .expect("comparison corpus release guard short alias should render"),
        comparison_guard
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard-summary"])
            .expect("comparison corpus guard alias should render"),
        comparison_guard
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard"])
            .expect("comparison corpus guard short alias should render"),
        comparison_guard
    );
    assert_eq!(
        render_cli(&["comparison-corpus-release-guard-summary", "extra"])
            .expect_err("comparison corpus release guard summary should reject extra arguments"),
        "comparison-corpus-release-guard-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-corpus-release-guard", "extra"]).expect_err(
            "comparison corpus release guard short alias should reject extra arguments"
        ),
        "comparison-corpus-release-guard does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard-summary", "extra"])
            .expect_err("comparison corpus guard alias should reject extra arguments"),
        "comparison-corpus-guard-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard", "extra"])
            .expect_err("comparison corpus guard short alias should reject extra arguments"),
        "comparison-corpus-guard does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-corpus", "extra"])
            .expect_err("comparison corpus alias should reject extra arguments"),
        "comparison-corpus-summary does not accept extra arguments"
    );

    let benchmark_corpus =
        render_cli(&["benchmark-corpus-summary"]).expect("benchmark corpus summary should render");
    assert!(benchmark_corpus.contains("Benchmark corpus summary"));
    assert!(benchmark_corpus.contains("name: Representative 1600-2600 window"));
    assert!(benchmark_corpus.contains("epoch labels: JD 2268559.0 (TT)"));
    assert!(benchmark_corpus.contains("JD 2451545.0 (TT)"));
    assert!(benchmark_corpus.contains("JD 2634532.0 (TT)"));
    assert_eq!(
        benchmark_corpus,
        validate_render_cli(&["benchmark-corpus-summary"])
            .expect("benchmark corpus summary should match validation output")
    );
    let chart_benchmark_corpus = render_cli(&["chart-benchmark-corpus-summary"])
        .expect("chart benchmark corpus summary should render");
    assert!(chart_benchmark_corpus.contains("Chart benchmark corpus summary"));
    assert!(chart_benchmark_corpus.contains("name: Representative chart validation scenarios"));
    assert!(chart_benchmark_corpus.contains("requests: 9"));
    assert!(chart_benchmark_corpus.contains("epochs: 1"));
    assert!(chart_benchmark_corpus.contains("epoch labels: JD 2451545.0 (TT)"));
    assert!(chart_benchmark_corpus.contains("bodies: 10"));
    assert_eq!(
        chart_benchmark_corpus,
        validate_render_cli(&["chart-benchmark-corpus-summary"])
            .expect("chart benchmark corpus summary should match validation output")
    );
    assert_eq!(
        render_cli(&["benchmark-corpus-summary", "extra"])
            .expect_err("benchmark corpus summary should reject extra arguments"),
        "benchmark-corpus-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["chart-benchmark-corpus-summary", "extra"])
            .expect_err("chart benchmark corpus summary should reject extra arguments"),
        "chart-benchmark-corpus-summary does not accept extra arguments"
    );

    let api_stability =
        render_cli(&["api-stability-summary"]).expect("api stability summary should render");
    assert!(api_stability.contains("API stability summary"));
    assert!(api_stability.contains("Summary line: API stability posture:"));
    assert!(api_stability.contains("Stable surfaces: 6"));
    assert!(api_stability.contains(&format!(
        "Compatibility profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(api_stability.contains("Release notes summary: release-notes-summary"));
    assert!(api_stability.contains("Release checklist summary: release-checklist-summary"));
    assert!(api_stability.contains("Release bundle verification: verify-release-bundle"));

    let release_notes = render_cli(&["release-notes"]).expect("release notes should render");
    assert!(release_notes.contains("Release notes"));
    assert!(release_notes.contains("Release notes summary: release-notes-summary"));
    assert!(release_notes.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(release_notes.contains("Artifact validation: validate-artifact"));
    assert!(release_notes.lines().any(|line| {
        line == "Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"
    }));
    assert!(release_notes.contains("Release checklist summary: release-checklist-summary"));
    assert!(release_notes.contains("Release bundle verification: verify-release-bundle"));
    assert!(
        release_notes.contains("Compatibility profile verification: verify-compatibility-profile")
    );
    assert!(release_notes.contains("API stability posture:"));
    assert!(release_notes.contains("Bundle provenance:"));
    assert!(release_notes.contains("True Mula (Chandra Hari)"));
    assert!(release_notes.contains("Babylonian (Eta Piscium)"));
    assert!(release_notes.contains("Babylonian (Kugler 2)"));
    assert!(release_notes.contains("Babylonian (Kugler 3)"));
    assert!(release_notes.contains("Galactic Equator (Mula)"));
    assert!(release_notes.contains("Galactic Equator (Fiorenza)"));
    assert!(release_notes.contains("JN Bhasin"));
    assert!(release_notes.contains("Galactic Equator (True)"));
    assert!(release_notes.contains("True galactic equator"));
    assert!(release_notes.contains("Galactic equator true"));
    assert!(release_notes.contains("Galactic Center (Mardyks)"));
    assert!(release_notes.contains("Gal. Center = 0 Cap"));
    assert!(release_notes.contains("Skydram (Mardyks)"));
    assert!(release_notes.contains("Mula Wilhelm"));
    assert!(release_notes.contains("Wilhelm"));
    assert!(release_notes.contains("Galactic Center (Rgilbrand)"));
    assert!(release_notes.contains("Babylonian (True Geoc)"));
    assert!(release_notes.contains("Babylonian (True Topc)"));
    assert!(release_notes.contains("Babylonian (True Obs)"));
    assert!(release_notes.contains("Pullen SD (Sinusoidal Delta)"));
    assert!(release_notes.contains("Equal/MC house system"));
    assert!(release_notes.contains("Equal Midheaven house system"));
    assert!(release_notes.contains("True Balarama"));
    assert!(release_notes.contains("Aphoric"));
    assert!(release_notes.contains("Takra"));

    let release_notes_summary =
        render_cli(&["release-notes-summary"]).expect("release notes summary should render");
    assert!(release_notes_summary.contains("Release notes summary"));
    assert!(release_notes_summary
        .contains("Comparison tolerance policy: backend family=Composite; scopes=6"));
    assert!(release_notes_summary.contains("Pluto fallback (approximate)"));
    assert!(release_notes_summary.contains(&format!(
        "House code aliases: {}",
        current_compatibility_profile().house_code_aliases_summary_line()
    )));
    assert!(release_notes_summary.contains("API stability summary line:"));
    assert!(release_notes_summary
        .contains(&pleiades_jpl::selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(release_notes_summary.contains(
        &pleiades_jpl::reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()
    ));
    assert!(release_notes_summary.contains(profile.target_house_scope.join("; ").as_str()));
    assert!(release_notes_summary.contains(profile.target_ayanamsa_scope.join("; ").as_str()));
    assert!(release_notes_summary.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(release_notes_summary.lines().any(|line| {
        line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
    }));
    assert!(release_notes_summary.lines().any(|line| {
        line == "VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"
    }));
    assert!(release_notes_summary.lines().any(|line| {
        line == "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
    }));
    assert!(release_notes_summary.lines().any(|line| {
        line == "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
    }));
    assert!(release_notes_summary.contains("Artifact validation: validate-artifact"));
    assert!(release_notes_summary.lines().any(|line| {
        line == "Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"
    }));
    assert!(release_notes_summary.contains("Release notes: release-notes"));
    assert!(release_notes_summary.contains("Packaged-artifact storage/reconstruction:"));
    assert!(release_notes_summary
        .lines()
        .any(|line| line == packaged_artifact_access_report_line()));
    assert!(release_notes_summary.lines().any(|line| {
        line == format!(
            "Packaged-artifact generation policy: {}",
            pleiades_data::packaged_artifact_generation_policy_summary_for_report()
        )
    }));
    assert!(release_notes_summary.contains("Packaged request policy:"));
    assert!(release_notes_summary.contains("Packaged lookup epoch policy:"));
    assert!(release_notes_summary.lines().any(|line| {
        line == format!(
            "Packaged batch parity: {}",
            pleiades_data::packaged_mixed_tt_tdb_batch_parity_summary_for_report()
        )
    }));
    assert!(release_notes_summary.contains("Packaged batch parity:"));
    assert!(release_notes_summary
        .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
    assert!(release_notes_summary
        .contains("Artifact summary: artifact-summary / artifact-posture-summary"));
    assert!(release_notes_summary.contains("Release checklist summary: release-checklist-summary"));
    assert!(release_notes_summary.contains("Release bundle verification: verify-release-bundle"));
    assert!(release_notes_summary
        .contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(release_notes_summary
        .contains("See release-notes for the full maintainer-facing artifact."));
    assert!(release_notes_summary
        .contains("See release-summary for the compact one-screen release overview."));
    assert!(release_notes_summary.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));

    let release_checklist =
        render_cli(&["release-checklist"]).expect("release checklist should render");
    assert!(release_checklist.contains("Release checklist"));
    assert!(release_checklist.contains("Release notes summary: release-notes-summary"));
    assert!(release_checklist.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(release_checklist.contains("API stability summary: api-stability-summary"));
    assert!(release_checklist.contains("Artifact validation: validate-artifact"));
    assert!(release_checklist.lines().any(|line| {
        line == "Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"
    }));
    assert!(release_checklist.contains("Repository-managed release gates:"));
    assert!(release_checklist
        .contains("[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile"));
    assert!(release_checklist.contains("bundle-release --out /tmp/pleiades-release"));
    assert!(release_checklist.contains("release-checklist-summary.txt"));

    let release_checklist_summary = render_cli(&["release-checklist-summary"])
        .expect("release checklist summary should render");
    assert!(release_checklist_summary.contains("Release checklist summary"));
    assert!(release_checklist_summary.contains("Release notes summary: release-notes-summary"));
    assert!(release_checklist_summary.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(release_checklist_summary.contains("API stability summary: api-stability-summary"));
    assert!(release_checklist_summary
        .contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(release_checklist_summary.contains("Artifact validation: validate-artifact"));
    assert!(
        release_checklist_summary.contains("Release bundle verification: verify-release-bundle")
    );
    assert!(release_checklist_summary.contains("Workspace audit: workspace-audit / audit"));
    assert!(release_checklist_summary.contains("Release summary: release-summary"));
    assert!(release_checklist_summary.lines().any(|line| {
        line == "Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"
    }));
    assert!(release_checklist_summary.contains("Repository-managed release gates: 10 items"));
    let release_checklist_summary_details = pleiades_validate::release_checklist_summary();
    assert!(release_checklist_summary.contains(&format!(
        "Manual bundle workflow: {} items",
        release_checklist_summary_details.manual_bundle_workflow_items
    )));
    assert!(release_checklist_summary.contains("Bundle contents: 25 items"));
    assert!(release_checklist_summary.contains("External publishing reminders: 3 items"));
    assert!(release_checklist_summary
        .contains("See release-summary for the compact one-screen release overview."));

    let release_gate = render_cli(&["release-gate"]).expect("release gate should render");
    let release_gate_summary =
        render_cli(&["release-gate-summary"]).expect("release gate summary should render");
    assert_eq!(release_gate, release_checklist);
    assert_eq!(release_gate_summary, release_checklist_summary);

    let compatibility_profile = current_compatibility_profile();
    let release_summary = render_cli(&["release-summary"]).expect("release summary should render");
    assert!(release_summary.contains("Release summary"));
    assert!(release_summary.contains("House systems:"));
    assert!(release_summary.contains(&format!(
        "House-code aliases: {}",
        compatibility_profile.house_code_alias_count()
    )));
    assert!(release_summary.contains(&format!(
        "House code aliases: {}",
        compatibility_profile.house_code_aliases_summary_line()
    )));
    assert!(release_summary.contains(&compatibility_profile.catalog_inventory_summary_line()));
    assert!(release_summary
        .contains("Comparison corpus release-grade guard: Pluto excluded from tolerance evidence"));
    assert!(release_summary.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
    assert!(release_summary.lines().any(|line| {
        line == "Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.7.0, api-stability=pleiades-api-stability/0.2.0"
    }));
    assert!(release_summary.contains("API stability summary line: API stability posture: pleiades-api-stability/0.2.0; stable surfaces: 6; experimental surfaces: 3; deprecation policy items: 4; intentional limits: 3"));
    assert!(release_summary.lines().any(|line| {
        line == "Time-scale policy: direct backend requests accept TT/TDB; civil UTC/UT1 inputs convert via the pleiades-time crate or caller-supplied offsets; the ephemeris backends carry no internal Delta T or UTC convenience model"
    }));
    assert!(release_summary.lines().any(|line| {
        line == "Delta T policy: built-in Delta T modeling is now provided by the pleiades-time crate for civil UTC/UT1 inputs over 1900-2100, tagged observed/predicted; direct backend requests still accept TT/TDB"
    }));
    assert!(release_summary.lines().any(|line| {
        line == "Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported"
    }));
    assert!(release_summary.lines().any(|line| {
        line == "Apparentness policy: backends remain mean-only and J2000 at the backend boundary; apparent place of date (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, release-grade bodies; gravitational light-deflection omitted"
    }));
    assert!(release_summary.lines().any(|line| {
        line == "Native sidereal policy: native sidereal backend output remains unsupported unless a backend explicitly advertises it"
    }));
    assert!(release_summary.lines().any(|line| {
        line == "Request policy: time-scale=direct backend requests accept TT/TDB; civil UTC/UT1 inputs convert via the pleiades-time crate or caller-supplied offsets; the ephemeris backends carry no internal Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported; apparentness=backends remain mean-only and J2000 at the backend boundary; apparent place of date (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, release-grade bodies; gravitational light-deflection omitted; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
    }));

    let request_surface_summary =
        render_cli(&["request-surface-summary"]).expect("request surface summary should render");
    assert_eq!(
        request_surface_summary,
        render_cli(&["request-surface"]).expect("request surface alias should render")
    );
    assert!(request_surface_summary.contains("Request surface summary"));
    assert!(request_surface_summary.contains("Primary request surfaces:"));
    assert!(request_surface_summary
        .contains("pleiades-types::Instant (tagged instant plus caller-supplied retagging)"));
    assert!(request_surface_summary
        .contains("pleiades-core::ChartRequest (chart assembly plus house-observer preflight)"));
    assert!(request_surface_summary.contains(
        "pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight)"
    ));
    assert!(request_surface_summary
        .contains("pleiades-houses::HouseRequest (house-only observer calculations)"));
    assert!(
        request_surface_summary.contains("pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)")
    );
    assert_eq!(
        request_surface_summary,
        pleiades_validate::render_request_surface_summary()
    );
    assert_eq!(
        render_cli(&["request-surface-summary", "extra"])
            .expect_err("request surface summary should reject extra arguments"),
        "request-surface-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["request-surface", "extra"])
            .expect_err("request surface alias should reject extra arguments"),
        "request-surface does not accept extra arguments"
    );

    let request_policy_summary =
        render_cli(&["request-policy-summary"]).expect("request policy summary should render");
    assert!(request_policy_summary.contains("Request policy summary"));
    assert!(request_policy_summary.contains("Time-scale policy: direct backend requests accept TT/TDB; civil UTC/UT1 inputs convert via the pleiades-time crate or caller-supplied offsets; the ephemeris backends carry no internal Delta T or UTC convenience model"));
    assert!(request_policy_summary.contains("UTC convenience policy: built-in UTC convenience conversion is now provided by the pleiades-time crate (civil UTC/UT1 to TT/TDB, leap-second-exact UTC, tiered exact/observed/predicted, 1900-2100); direct backends still consume TT/TDB"));
    assert!(request_policy_summary.contains("Delta T policy: built-in Delta T modeling is now provided by the pleiades-time crate for civil UTC/UT1 inputs over 1900-2100, tagged observed/predicted; direct backend requests still accept TT/TDB"));
    assert!(request_policy_summary.contains("Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported"));
    assert!(request_policy_summary.contains("Apparentness policy: backends remain mean-only and J2000 at the backend boundary; apparent place of date (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, release-grade bodies; gravitational light-deflection omitted"));
    assert!(request_policy_summary.contains("Request policy: time-scale=direct backend requests accept TT/TDB; civil UTC/UT1 inputs convert via the pleiades-time crate or caller-supplied offsets; the ephemeris backends carry no internal Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported; apparentness=backends remain mean-only and J2000 at the backend boundary; apparent place of date (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, release-grade bodies; gravitational light-deflection omitted; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    let request_policy_alias =
        render_cli(&["request-policy"]).expect("request policy alias should render");
    assert_eq!(request_policy_alias, request_policy_summary);

    let request_semantics_summary = render_cli(&["request-semantics-summary"])
        .expect("request semantics summary should render");
    assert!(request_semantics_summary.contains("Request semantics summary"));

    let request_semantics_alias =
        render_cli(&["request-semantics"]).expect("request semantics alias should render");
    assert!(request_semantics_alias.contains("Request semantics summary"));
    assert_eq!(request_semantics_alias, request_semantics_summary);

    let unsupported_modes_summary = render_cli(&["unsupported-modes-summary"])
        .expect("unsupported-modes summary should render");
    assert!(unsupported_modes_summary.contains("Unsupported modes summary"));
    assert!(unsupported_modes_summary.contains("Unsupported modes:"));
    assert_eq!(
        render_cli(&["unsupported-modes"]).expect("unsupported-modes alias should render"),
        unsupported_modes_summary
    );

    let request_policy_error = render_cli(&["request-policy", "extra"])
        .expect_err("request policy alias should reject extra arguments");
    assert_eq!(
        request_policy_error,
        "request-policy does not accept extra arguments"
    );
    let request_policy_summary_error = render_cli(&["request-policy-summary", "extra"])
        .expect_err("request policy summary should reject extra arguments");
    assert_eq!(
        request_policy_summary_error,
        "request-policy-summary does not accept extra arguments"
    );
    let request_semantics_error = render_cli(&["request-semantics-summary", "extra"])
        .expect_err("request semantics alias should reject extra arguments");
    assert_eq!(
        request_semantics_error,
        "request-semantics-summary does not accept extra arguments"
    );

    let request_semantics_alias_error = render_cli(&["request-semantics", "extra"])
        .expect_err("request-semantics alias should reject extra arguments");
    assert_eq!(
        request_semantics_alias_error,
        "request-semantics does not accept extra arguments"
    );

    for (args, expected) in [
        (
            ["catalog-inventory-summary", "extra"],
            "catalog-inventory-summary does not accept extra arguments",
        ),
        (
            ["catalog-inventory", "extra"],
            "catalog-inventory does not accept extra arguments",
        ),
        (
            ["backend-matrix", "extra"],
            "backend-matrix does not accept extra arguments",
        ),
        (
            ["compatibility-profile", "extra"],
            "compatibility-profile does not accept extra arguments",
        ),
        (
            ["api-posture-summary", "extra"],
            "api-stability-summary does not accept extra arguments",
        ),
        (
            ["api-stability", "extra"],
            "api-stability does not accept extra arguments",
        ),
        (
            ["release-checklist", "extra"],
            "release-checklist does not accept extra arguments",
        ),
        (
            ["release-summary", "extra"],
            "release-summary does not accept extra arguments",
        ),
        (
            ["release-notes", "extra"],
            "release-notes does not accept extra arguments",
        ),
        (
            ["artifact-posture-summary", "extra"],
            "artifact-summary does not accept extra arguments",
        ),
    ] {
        assert_eq!(
            render_cli(&args).expect_err("summary command should reject extra arguments"),
            expected
        );
    }

    let comparison_tolerance_policy_summary = render_cli(&["comparison-tolerance-policy-summary"])
        .expect("comparison tolerance policy summary should render");
    assert_eq!(
        comparison_tolerance_policy_summary,
        validate_render_cli(&["comparison-tolerance-policy-summary"])
            .expect("comparison tolerance policy summary should match validate CLI")
    );

    let comparison_tolerance_summary = render_cli(&["comparison-tolerance-summary"])
        .expect("comparison tolerance summary alias should render");
    assert_eq!(
        comparison_tolerance_summary, comparison_tolerance_policy_summary,
        "comparison tolerance summary alias should match the canonical command"
    );

    let comparison_tolerance_scope_coverage_summary =
        render_cli(&["comparison-tolerance-scope-coverage-summary"])
            .expect("comparison tolerance scope coverage summary should render");
    assert_eq!(
        comparison_tolerance_scope_coverage_summary,
        validate_render_cli(&["comparison-tolerance-scope-coverage-summary"])
            .expect("comparison tolerance scope coverage summary should match validate CLI")
    );
    assert_eq!(
        render_cli(&["comparison-tolerance-scope-coverage"])
            .expect("comparison tolerance scope coverage alias should render"),
        comparison_tolerance_scope_coverage_summary,
        "comparison tolerance scope coverage alias should match the canonical command"
    );
    assert_eq!(
        render_cli(&["comparison-tolerance-scope-coverage-summary", "extra"]).expect_err(
            "comparison tolerance scope coverage summary should reject extra arguments"
        ),
        "comparison-tolerance-scope-coverage-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-tolerance-scope-coverage", "extra"])
            .expect_err("comparison tolerance scope coverage alias should reject extra arguments"),
        "comparison-tolerance-scope-coverage does not accept extra arguments"
    );

    let comparison_body_class_tolerance_summary =
        render_cli(&["comparison-body-class-tolerance-summary"])
            .expect("comparison body-class tolerance summary should render");
    assert_eq!(
        comparison_body_class_tolerance_summary,
        validate_render_cli(&["comparison-body-class-tolerance-summary"])
            .expect("comparison body-class tolerance summary should match validate CLI")
    );

    let comparison_body_class_tolerance_alias = render_cli(&["comparison-body-class-tolerance"])
        .expect("comparison body-class tolerance alias should render");
    assert_eq!(
        comparison_body_class_tolerance_alias, comparison_body_class_tolerance_summary,
        "comparison body-class tolerance alias should match the canonical command"
    );

    let comparison_body_class_tolerance_posture_summary =
        render_cli(&["comparison-body-class-tolerance-posture-summary"])
            .expect("comparison body-class tolerance posture summary should render");
    assert_eq!(
        comparison_body_class_tolerance_posture_summary,
        validate_render_cli(&["comparison-body-class-tolerance-posture-summary"])
            .expect("comparison body-class tolerance posture summary should match validate CLI")
    );

    let comparison_body_class_tolerance_posture_alias =
        render_cli(&["comparison-body-class-tolerance-posture"])
            .expect("comparison body-class tolerance posture alias should render");
    assert_eq!(
        comparison_body_class_tolerance_posture_alias,
        comparison_body_class_tolerance_posture_summary,
        "comparison body-class tolerance posture alias should match the canonical command"
    );

    let comparison_envelope_summary = render_cli(&["comparison-envelope-summary"])
        .expect("comparison envelope summary should render");
    assert_eq!(
        comparison_envelope_summary,
        validate_render_cli(&["comparison-envelope-summary"])
            .expect("comparison envelope summary should match validate CLI")
    );

    let comparison_envelope_alias =
        render_cli(&["comparison-envelope"]).expect("comparison envelope alias should render");
    assert_eq!(comparison_envelope_alias, comparison_envelope_summary);

    let comparison_body_class_error_envelope_summary =
        render_cli(&["comparison-body-class-error-envelope-summary"])
            .expect("comparison body-class error envelope summary should render");
    assert_eq!(
        comparison_body_class_error_envelope_summary,
        validate_render_cli(&["comparison-body-class-error-envelope-summary"])
            .expect("comparison body-class error envelope summary should match validate CLI")
    );
    let comparison_body_class_error_envelope_alias =
        render_cli(&["comparison-body-class-error-envelope"])
            .expect("comparison body-class error envelope alias should render");
    assert_eq!(
        comparison_body_class_error_envelope_alias,
        comparison_body_class_error_envelope_summary
    );

    let comparison_tolerance_alias_error = render_cli(&["comparison-tolerance-summary", "extra"])
        .expect_err("comparison tolerance alias should reject extra arguments");
    assert!(comparison_tolerance_alias_error
        .contains("comparison-tolerance-summary does not accept extra arguments"));

    let comparison_body_class_tolerance_alias_error =
        render_cli(&["comparison-body-class-tolerance", "extra"])
            .expect_err("comparison body-class tolerance alias should reject extra arguments");
    assert!(comparison_body_class_tolerance_alias_error
        .contains("comparison-body-class-tolerance does not accept extra arguments"));

    let comparison_body_class_tolerance_posture_alias_error =
        render_cli(&["comparison-body-class-tolerance-posture", "extra"]).expect_err(
            "comparison body-class tolerance posture alias should reject extra arguments",
        );
    assert!(comparison_body_class_tolerance_posture_alias_error
        .contains("comparison-body-class-tolerance-posture does not accept extra arguments"));

    let comparison_envelope_alias_error = render_cli(&["comparison-envelope", "extra"])
        .expect_err("comparison envelope alias should reject extra arguments");
    assert!(comparison_envelope_alias_error
        .contains("comparison-envelope does not accept extra arguments"));

    let release_body_claims_summary = render_cli(&["release-body-claims-summary"])
        .expect("release body claims summary should render");
    assert_eq!(
        release_body_claims_summary,
        validate_render_cli(&["release-body-claims-summary"])
            .expect("release body claims summary should match validate CLI")
    );
    assert_eq!(
        render_cli(&["body-claims-summary"]).expect("body claims alias should render"),
        release_body_claims_summary
    );
    assert_eq!(
        render_cli(&["body-claims-summary", "extra"])
            .expect_err("body claims alias should reject extra arguments"),
        "body-claims-summary does not accept extra arguments"
    );

    let body_date_channel_claims_summary = render_cli(&["body-date-channel-claims-summary"])
        .expect("body/date/channel claims summary should render");
    assert_eq!(
        body_date_channel_claims_summary,
        validate_render_cli(&["body-date-channel-claims-summary"])
            .expect("body/date/channel claims summary should match validate CLI")
    );
    assert_eq!(
        render_cli(&["body-date-channel-claims"])
            .expect("body/date/channel claims alias should render"),
        body_date_channel_claims_summary
    );
    assert_eq!(
        render_cli(&["body-date-channel-claims", "extra"])
            .expect_err("body/date/channel claims alias should reject extra arguments"),
        "body-date-channel-claims does not accept extra arguments"
    );

    let pluto_fallback_summary =
        render_cli(&["pluto-fallback-summary"]).expect("Pluto fallback summary should render");
    assert_eq!(
        pluto_fallback_summary,
        validate_render_cli(&["pluto-fallback-summary"])
            .expect("Pluto fallback summary should match validate CLI")
    );
    assert_eq!(
        render_cli(&["pluto-fallback"]).expect("Pluto fallback alias should render"),
        pluto_fallback_summary
    );
    assert_eq!(
        render_cli(&["pluto-fallback", "extra"])
            .expect_err("Pluto fallback alias should reject extra arguments"),
        "pluto-fallback does not accept extra arguments"
    );

    let jpl_batch_error_taxonomy_summary = render_cli(&["jpl-batch-error-taxonomy-summary"])
        .expect("JPL batch error taxonomy summary should render");
    assert_eq!(
        jpl_batch_error_taxonomy_summary,
        validate_render_cli(&["jpl-batch-error-taxonomy-summary"])
            .expect("validation JPL batch error taxonomy summary should render")
    );

    let jpl_snapshot_evidence_summary = render_cli(&["jpl-snapshot-evidence-summary"])
        .expect("JPL snapshot evidence summary should render");
    assert!(jpl_snapshot_evidence_summary
        .contains(&pleiades_jpl::jpl_source_posture_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary.contains(
        &pleiades_jpl::reference_snapshot_2451914_major_body_bridge_day_summary_for_report()
    ));
    assert_eq!(
        jpl_snapshot_evidence_summary,
        validate_render_cli(&["jpl-snapshot-evidence-summary"])
            .expect("validation JPL snapshot evidence summary should render")
    );

    let jpl_source_corpus_contract_summary = render_cli(&["jpl-source-corpus-contract-summary"])
        .expect("JPL source corpus contract summary should render");
    assert!(jpl_source_corpus_contract_summary.contains("JPL source corpus contract:"));
    assert!(jpl_source_corpus_contract_summary
        .contains(&pleiades_jpl::jpl_source_corpus_contract_summary_for_report()));
    assert_eq!(
        jpl_source_corpus_contract_summary,
        validate_render_cli(&["jpl-source-corpus-contract-summary"])
            .expect("validation JPL source corpus contract summary should render")
    );
    assert_eq!(
        jpl_source_corpus_contract_summary,
        render_cli(&["jpl-source-corpus-contract"])
            .expect("JPL source corpus contract alias should render")
    );
    assert_eq!(
        jpl_source_corpus_contract_summary,
        validate_render_cli(&["jpl-source-corpus-contract"])
            .expect("validation JPL source corpus contract alias should render")
    );
    assert_eq!(
        render_cli(&["jpl-source-corpus-contract", "extra"])
            .expect_err("JPL source corpus contract alias should reject extra arguments"),
        "jpl-source-corpus-contract does not accept extra arguments"
    );
    let jpl_source_posture_summary = render_cli(&["jpl-source-posture-summary"])
        .expect("JPL source posture summary should render");
    assert_eq!(
        jpl_source_posture_summary,
        pleiades_jpl::jpl_source_posture_summary_for_report()
    );
    assert_eq!(
        jpl_source_posture_summary,
        render_cli(&["jpl-source-posture"]).expect("JPL source posture alias should render")
    );
    assert_eq!(
        render_cli(&["jpl-source-posture-summary", "extra"])
            .expect_err("JPL source posture summary should reject extra arguments"),
        "jpl-source-posture-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["jpl-source-posture", "extra"])
            .expect_err("JPL source posture alias should reject extra arguments"),
        "jpl-source-posture-summary does not accept extra arguments"
    );

    let jpl_provenance_only_summary = render_cli(&["jpl-provenance-only-summary"])
        .expect("JPL provenance-only summary should render");
    assert_eq!(
        jpl_provenance_only_summary,
        pleiades_jpl::jpl_provenance_only_summary_for_report()
    );
    assert_eq!(
        jpl_provenance_only_summary,
        render_cli(&["jpl-provenance-only"]).expect("JPL provenance-only alias should render")
    );
    assert_eq!(
        jpl_provenance_only_summary,
        validate_render_cli(&["jpl-provenance-only-summary"])
            .expect("validation JPL provenance-only summary should render")
    );
    assert_eq!(
        render_cli(&["jpl-provenance-only-summary", "extra"])
            .expect_err("JPL provenance-only summary should reject extra arguments"),
        "jpl-provenance-only-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["jpl-provenance-only", "extra"])
            .expect_err("JPL provenance-only alias should reject extra arguments"),
        "jpl-provenance-only does not accept extra arguments"
    );

    let source_corpus_summary =
        render_cli(&["source-corpus-summary"]).expect("source corpus summary should render");
    assert_eq!(
        source_corpus_summary,
        validate_render_cli(&["source-corpus-summary"])
            .expect("validation source corpus summary should render")
    );
    assert!(source_corpus_summary.contains("comparison corpus release-grade guard:"));
    assert!(source_corpus_summary.contains("JPL source corpus contract:"));
    assert!(source_corpus_summary.contains("phase-2 corpus alignment:"));
    assert_eq!(
        source_corpus_summary,
        render_cli(&["source-corpus"]).expect("source corpus alias should render")
    );
    assert_eq!(
        source_corpus_summary,
        validate_render_cli(&["source-corpus"])
            .expect("validation source corpus alias should render")
    );
    assert_eq!(
        source_corpus_summary,
        render_cli(&["source-corpus-posture-summary"])
            .expect("source corpus posture summary should render")
    );
    assert_eq!(
        source_corpus_summary,
        render_cli(&["source-corpus-posture"]).expect("source corpus posture alias should render")
    );
    assert_eq!(
        source_corpus_summary,
        validate_render_cli(&["source-corpus-posture-summary"])
            .expect("validation source corpus posture summary should render")
    );
    assert_eq!(
        source_corpus_summary,
        validate_render_cli(&["source-corpus-posture"])
            .expect("validation source corpus posture alias should render")
    );
    assert_eq!(
        render_cli(&["source-corpus", "extra"])
            .expect_err("source corpus alias should reject extra arguments"),
        "source-corpus does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["source-corpus-posture-summary", "extra"])
            .expect_err("source corpus posture summary should reject extra arguments"),
        "source-corpus-posture-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["source-corpus-posture", "extra"])
            .expect_err("source corpus posture alias should reject extra arguments"),
        "source-corpus-posture does not accept extra arguments"
    );

    let packaged_lookup_epoch_policy_summary =
        render_cli(&["packaged-lookup-epoch-policy-summary"])
            .expect("packaged lookup epoch policy summary should render");
    assert!(packaged_lookup_epoch_policy_summary
        .contains("TT-grid retag without relativistic correction"));
    assert_eq!(
        packaged_lookup_epoch_policy_summary,
        pleiades_data::packaged_lookup_epoch_policy_summary_for_report()
    );
    assert_eq!(
        packaged_lookup_epoch_policy_summary,
        render_cli(&["packaged-lookup-epoch-policy"])
            .expect("packaged lookup epoch policy alias should render")
    );
    assert_eq!(
        packaged_lookup_epoch_policy_summary,
        render_cli(&["packaged-artifact-lookup-epoch-policy-summary"])
            .expect("packaged artifact lookup epoch policy summary alias should render")
    );
    assert_eq!(
        packaged_lookup_epoch_policy_summary,
        render_cli(&["packaged-artifact-lookup-epoch-policy"])
            .expect("packaged artifact lookup epoch policy alias should render")
    );

    let production_generation_boundary_summary =
        render_cli(&["production-generation-boundary-summary"])
            .expect("production generation boundary summary should render");
    assert!(
        production_generation_boundary_summary.contains("Production generation boundary overlay:")
    );
    assert_eq!(
        production_generation_boundary_summary,
        pleiades_jpl::production_generation_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-boundary"])
            .expect("production generation boundary alias should render"),
        pleiades_jpl::production_generation_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-boundary", "extra"]).unwrap_err(),
        "production-generation-boundary-summary does not accept extra arguments"
    );
    let production_generation_summary = render_cli(&["production-generation-summary"])
        .expect("production generation summary should render");
    assert!(production_generation_summary.contains("Production generation coverage:"));
    assert_eq!(
        production_generation_summary,
        pleiades_jpl::production_generation_snapshot_summary_for_report()
    );
    let production_generation_quarter_day_boundary_summary =
        render_cli(&["production-generation-quarter-day-boundary-summary"])
            .expect("production generation quarter-day boundary summary should render");
    assert!(production_generation_quarter_day_boundary_summary
        .contains("Production generation quarter-day boundary samples:"));
    assert_eq!(
        production_generation_quarter_day_boundary_summary,
        pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report()
    );
    let production_generation_quarter_day_boundary_alias =
        render_cli(&["production-generation-quarter-day-boundary"])
            .expect("production generation quarter-day boundary alias should render");
    assert_eq!(
        production_generation_quarter_day_boundary_alias,
        pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report()
    );
    let production_generation_alias =
        render_cli(&["production-generation"]).expect("production generation alias should render");
    assert_eq!(
        production_generation_alias,
        pleiades_jpl::production_generation_snapshot_summary_for_report()
    );
    let alias_error = render_cli(&["production-generation", "extra"])
        .expect_err("production generation alias should reject extra arguments");
    assert!(alias_error.contains("production-generation does not accept extra arguments"));
    assert_eq!(
        render_cli(&["production-generation-boundary-summary", "extra"]).unwrap_err(),
        "production-generation-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["production-generation-summary", "extra"]).unwrap_err(),
        "production-generation-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["production-generation-source-summary", "extra"]).unwrap_err(),
        "production-generation-source-summary does not accept extra arguments"
    );
    let production_generation_boundary_request_corpus_summary =
        render_cli(&["production-generation-boundary-request-corpus-summary"])
            .expect("production generation boundary request corpus summary should render");
    assert!(production_generation_boundary_request_corpus_summary
        .contains("Production generation boundary request corpus:"));
    assert_eq!(
        production_generation_boundary_request_corpus_summary,
        pleiades_jpl::production_generation_boundary_request_corpus_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-boundary-request-corpus"])
            .expect("production generation boundary request corpus alias should render"),
        pleiades_jpl::production_generation_boundary_request_corpus_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-boundary-request-corpus", "extra"]).unwrap_err(),
        "production-generation-boundary-request-corpus-summary does not accept extra arguments"
    );
    let production_generation_boundary_request_corpus_equatorial_summary =
        render_cli(&["production-generation-boundary-request-corpus-equatorial-summary"]).expect(
            "production generation boundary request corpus equatorial summary should render",
        );
    assert!(
        production_generation_boundary_request_corpus_equatorial_summary
            .contains("Production generation boundary request corpus:")
    );
    assert_eq!(
        production_generation_boundary_request_corpus_equatorial_summary,
        pleiades_jpl::production_generation_boundary_request_corpus_equatorial_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-boundary-request-corpus-equatorial"])
            .expect("production generation boundary request corpus equatorial alias should render"),
        pleiades_jpl::production_generation_boundary_request_corpus_equatorial_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-boundary-request-corpus-equatorial-summary", "extra"]).expect_err(
            "production generation boundary request corpus equatorial summary should reject extra arguments"
        ),
        "production-generation-boundary-request-corpus-equatorial-summary does not accept extra arguments"
    );
    let production_generation_source_window_summary =
        render_cli(&["production-generation-source-window-summary"])
            .expect("production generation source window summary should render");
    assert!(production_generation_source_window_summary
        .contains("Production generation source windows:"));
    assert_eq!(
        production_generation_source_window_summary,
        pleiades_jpl::production_generation_snapshot_window_summary_for_report()
    );
    assert_eq!(
        production_generation_source_window_summary,
        validate_render_cli(&["production-generation-source-window-summary"])
            .expect("validation production generation source window summary should render")
    );
    assert_eq!(
        render_cli(&["production-generation-source-window"])
            .expect("production generation source window alias should render"),
        production_generation_source_window_summary
    );
    assert_eq!(
        render_cli(&["production-generation-source-window", "extra"])
            .expect_err("production generation source window alias should reject extra arguments"),
        "production-generation-source-window does not accept extra arguments"
    );
    let production_generation_source_revision_summary =
        render_cli(&["production-generation-source-revision-summary"])
            .expect("production generation source revision summary should render");
    assert!(production_generation_source_revision_summary.contains("source revision="));
    assert_eq!(
        production_generation_source_revision_summary,
        pleiades_jpl::production_generation_source_revision_summary_for_report()
    );
    let production_generation_source_revision_alias =
        render_cli(&["production-generation-source-revision"])
            .expect("production generation source revision alias should render");
    assert_eq!(
        production_generation_source_revision_alias,
        pleiades_jpl::production_generation_source_revision_summary_for_report()
    );
    let production_generation_manifest_summary =
        render_cli(&["production-generation-manifest-summary"])
            .expect("production generation manifest summary should render");
    assert_eq!(
        production_generation_manifest_summary,
        validate_render_cli(&["production-generation-manifest-summary"])
            .expect("validation production generation manifest summary should render")
    );
    let production_generation_manifest_alias = render_cli(&["production-generation-manifest"])
        .expect("production generation manifest alias should render");
    assert_eq!(
        production_generation_manifest_alias,
        production_generation_manifest_summary
    );
    let production_generation_manifest_checksum_summary =
        render_cli(&["production-generation-manifest-checksum-summary"])
            .expect("production generation manifest checksum summary should render");
    assert_eq!(
        production_generation_manifest_checksum_summary,
        validate_render_cli(&["production-generation-manifest-checksum-summary"])
            .expect("validation production generation manifest checksum summary should render")
    );
    let production_generation_manifest_checksum_alias =
        render_cli(&["production-generation-manifest-checksum"])
            .expect("production generation manifest checksum alias should render");
    assert_eq!(
        production_generation_manifest_checksum_alias,
        production_generation_manifest_checksum_summary
    );
    assert_eq!(
        render_cli(&["production-generation-source-revision-summary", "extra"]).expect_err(
            "production generation source revision summary should reject extra arguments"
        ),
        "production-generation-source-revision-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["production-generation-source-revision", "extra"]).expect_err(
            "production generation source revision alias should reject extra arguments"
        ),
        "production-generation-source-revision-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["production-generation-manifest-summary", "extra"])
            .expect_err("production generation manifest summary should reject extra arguments"),
        "production-generation-manifest-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["production-generation-manifest", "extra"])
            .expect_err("production generation manifest alias should reject extra arguments"),
        "production-generation-manifest-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["production-generation-manifest-checksum-summary", "extra"]).expect_err(
            "production generation manifest checksum summary should reject extra arguments"
        ),
        "production-generation-manifest-checksum-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["production-generation-manifest-checksum", "extra"]).expect_err(
            "production generation manifest checksum alias should reject extra arguments"
        ),
        "production-generation-manifest-checksum-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["production-generation-source-window-summary", "extra"]).expect_err(
            "production generation source window summary should reject extra arguments"
        ),
        "production-generation-source-window-summary does not accept extra arguments"
    );
    let production_generation_body_class_coverage_summary =
        render_cli(&["production-generation-body-class-coverage-summary"])
            .expect("production generation body-class coverage summary should render");
    assert!(production_generation_body_class_coverage_summary
        .contains("Production generation body-class coverage:"));
    let production_body_class_coverage_summary =
        render_cli(&["production-body-class-coverage-summary"])
            .expect("production body-class coverage summary alias should render");
    assert_eq!(
        production_body_class_coverage_summary,
        production_generation_body_class_coverage_summary
    );
    assert_eq!(
        production_generation_body_class_coverage_summary,
        validate_render_cli(&["production-generation-body-class-coverage-summary"])
            .expect("validation production generation body-class coverage summary should render")
    );
    let comparison_body_class_coverage_summary =
        render_cli(&["comparison-body-class-coverage-summary"])
            .expect("comparison body-class coverage summary alias should render");
    assert_eq!(
        comparison_body_class_coverage_summary,
        validate_render_cli(&["comparison-snapshot-body-class-coverage-summary"])
            .expect("validation comparison snapshot body-class coverage summary should render")
    );
    let reference_body_class_coverage_summary =
        render_cli(&["reference-body-class-coverage-summary"])
            .expect("reference body-class coverage summary alias should render");
    assert_eq!(
        reference_body_class_coverage_summary,
        validate_render_cli(&["reference-snapshot-body-class-coverage-summary"])
            .expect("validation reference snapshot body-class coverage summary should render")
    );
    let holdout_body_class_coverage_summary = render_cli(&["holdout-body-class-coverage-summary"])
        .expect("holdout body-class coverage summary alias should render");
    assert_eq!(
        holdout_body_class_coverage_summary,
        validate_render_cli(&["independent-holdout-body-class-coverage-summary"])
            .expect("validation independent hold-out body-class coverage summary should render")
    );
    let comparison_snapshot_source_window_summary =
        render_cli(&["comparison-snapshot-source-window-summary"])
            .expect("comparison snapshot source window summary should render");
    assert!(
        comparison_snapshot_source_window_summary.contains("Comparison snapshot source windows:")
    );
    assert_eq!(
        comparison_snapshot_source_window_summary,
        pleiades_jpl::comparison_snapshot_source_window_summary_for_report()
    );
    let comparison_snapshot_source_window_alias =
        render_cli(&["comparison-snapshot-source-window"])
            .expect("comparison snapshot source window alias should render");
    assert_eq!(
        comparison_snapshot_source_window_alias,
        comparison_snapshot_source_window_summary
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-source-window", "extra"])
            .expect_err("comparison snapshot source window alias should reject extra arguments"),
        "comparison-snapshot-source-window does not accept extra arguments"
    );
    let comparison_snapshot_source_summary = render_cli(&["comparison-snapshot-source-summary"])
        .expect("comparison snapshot source summary should render");
    assert!(comparison_snapshot_source_summary.contains("Comparison snapshot source:"));
    assert_eq!(
        comparison_snapshot_source_summary,
        pleiades_jpl::comparison_snapshot_source_summary_for_report()
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-source-summary", "extra"])
            .expect_err("comparison snapshot source summary should reject extra arguments"),
        "comparison-snapshot-source-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-source"])
            .expect("comparison snapshot source alias should render"),
        comparison_snapshot_source_summary
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-source", "extra"])
            .expect_err("comparison snapshot source alias should reject extra arguments"),
        "comparison-snapshot-source does not accept extra arguments"
    );
    let reference_snapshot_source_window_summary =
        render_cli(&["reference-snapshot-source-window-summary"])
            .expect("reference snapshot source window summary should render");
    assert!(reference_snapshot_source_window_summary.contains("Reference snapshot source windows:"));
    assert_eq!(
        reference_snapshot_source_window_summary,
        pleiades_jpl::reference_snapshot_source_window_summary_for_report()
    );
    let reference_snapshot_source_window_alias = render_cli(&["reference-snapshot-source-window"])
        .expect("reference snapshot source window alias should render");
    assert_eq!(
        reference_snapshot_source_window_alias,
        reference_snapshot_source_window_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-source-window", "extra"])
            .expect_err("reference snapshot source window alias should reject extra arguments"),
        "reference-snapshot-source-window does not accept extra arguments"
    );
    let production_generation_boundary_source_summary =
        render_cli(&["production-generation-boundary-source-summary"])
            .expect("production generation boundary source summary should render");
    assert!(production_generation_boundary_source_summary
        .contains("Production generation boundary overlay source:"));
    assert!(production_generation_boundary_source_summary.contains("boundary overlay source"));
    assert_eq!(
        production_generation_boundary_source_summary,
        pleiades_jpl::production_generation_boundary_source_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-boundary-source"])
            .expect("production generation boundary source alias should render"),
        production_generation_boundary_source_summary,
    );
    let production_generation_boundary_window_summary =
        render_cli(&["production-generation-boundary-window-summary"])
            .expect("production generation boundary window summary should render");
    assert!(production_generation_boundary_window_summary
        .contains("Production generation boundary windows:"));
    assert!(production_generation_boundary_window_summary.contains("source-backed samples"));
    assert_eq!(
        production_generation_boundary_window_summary,
        pleiades_jpl::production_generation_boundary_window_summary_for_report()
    );
    let production_generation_boundary_window_alias =
        render_cli(&["production-generation-boundary-window"])
            .expect("production generation boundary window alias should render");
    assert_eq!(
        production_generation_boundary_window_alias,
        pleiades_jpl::production_generation_boundary_window_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-boundary-window", "extra"]).expect_err(
            "production generation boundary window alias should reject extra arguments"
        ),
        "production-generation-boundary-window does not accept extra arguments"
    );
    let production_generation_corpus_shape_summary =
        render_cli(&["production-generation-corpus-shape-summary"])
            .expect("production generation corpus shape summary should render");
    assert!(
        production_generation_corpus_shape_summary.contains("Production generation corpus shape:")
    );
    assert_eq!(
        production_generation_corpus_shape_summary,
        pleiades_jpl::production_generation_corpus_shape_summary_for_report()
    );
    let production_generation_corpus_shape_alias =
        render_cli(&["production-generation-corpus-shape"])
            .expect("production generation corpus shape alias should render");
    assert_eq!(
        production_generation_corpus_shape_alias,
        pleiades_jpl::production_generation_corpus_shape_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-corpus-shape", "extra"])
            .expect_err("production generation corpus shape alias should reject extra arguments"),
        "production-generation-corpus-shape-summary does not accept extra arguments"
    );
    let production_generation_source_summary =
        render_cli(&["production-generation-source-summary"])
            .expect("production generation source summary should render");
    assert!(production_generation_source_summary.contains("Production generation source:"));
    assert_eq!(
        production_generation_source_summary,
        pleiades_jpl::production_generation_source_summary_for_report()
    );
    let production_generation_source_alias = render_cli(&["production-generation-source"])
        .expect("production generation source alias should render");
    assert_eq!(
        production_generation_source_alias,
        pleiades_jpl::production_generation_source_summary_for_report()
    );
    assert_eq!(
        render_cli(&["production-generation-source", "extra"])
            .expect_err("production generation source alias should reject extra arguments"),
        "production-generation-source does not accept extra arguments"
    );
    let reference_snapshot_lunar_boundary_summary =
        render_cli(&["reference-snapshot-lunar-boundary-summary"])
            .expect("reference snapshot lunar boundary summary should render");
    assert!(
        reference_snapshot_lunar_boundary_summary.contains("Reference lunar boundary evidence:")
    );
    assert_eq!(
        reference_snapshot_lunar_boundary_summary,
        pleiades_jpl::reference_snapshot_lunar_boundary_summary_for_report()
    );
    let lunar_boundary_summary = render_cli(&["lunar-boundary-summary"])
        .expect("lunar boundary summary alias should render");
    assert!(lunar_boundary_summary.contains("Reference lunar boundary evidence:"));
    assert_eq!(
        lunar_boundary_summary,
        pleiades_jpl::reference_snapshot_lunar_boundary_summary_for_report()
    );
    let reference_snapshot_2451910_major_body_boundary_summary =
        render_cli(&["reference-snapshot-2451910-major-body-boundary-summary"])
            .expect("reference snapshot 2451910 major-body boundary summary should render");
    assert!(reference_snapshot_2451910_major_body_boundary_summary
        .contains("Reference 2451910 major-body boundary evidence:"));
    assert_eq!(
        reference_snapshot_2451910_major_body_boundary_summary,
        pleiades_jpl::reference_snapshot_2451910_major_body_boundary_summary_for_report()
    );
    let reference_snapshot_2451910_major_body_boundary_alias =
        render_cli(&["2451910-major-body-boundary-summary"])
            .expect("2451910 major-body boundary alias should render");
    assert_eq!(
        reference_snapshot_2451910_major_body_boundary_alias,
        reference_snapshot_2451910_major_body_boundary_summary
    );
    let reference_snapshot_2451911_major_body_boundary_summary =
        render_cli(&["reference-snapshot-2451911-major-body-boundary-summary"])
            .expect("reference snapshot 2451911 major-body boundary summary should render");
    assert!(reference_snapshot_2451911_major_body_boundary_summary
        .contains("Reference 2451911 major-body boundary evidence:"));
    assert_eq!(
        reference_snapshot_2451911_major_body_boundary_summary,
        pleiades_jpl::reference_snapshot_2451911_major_body_boundary_summary_for_report()
    );
    let reference_snapshot_2451911_major_body_boundary_alias =
        render_cli(&["2451911-major-body-boundary-summary"])
            .expect("2451911 major-body boundary alias should render");
    assert_eq!(
        reference_snapshot_2451911_major_body_boundary_alias,
        reference_snapshot_2451911_major_body_boundary_summary
    );
    let comparison_snapshot_manifest_summary =
        render_cli(&["comparison-snapshot-manifest-summary"])
            .expect("comparison snapshot manifest summary should render");
    assert!(comparison_snapshot_manifest_summary.contains("Comparison snapshot manifest:"));
    assert_eq!(
        comparison_snapshot_manifest_summary,
        pleiades_jpl::validated_comparison_snapshot_manifest_summary_for_report()
            .expect("comparison snapshot manifest summary should validate")
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-manifest"])
            .expect("comparison snapshot manifest alias should render"),
        comparison_snapshot_manifest_summary
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-manifest", "extra"])
            .expect_err("comparison snapshot manifest alias should reject extra arguments"),
        "comparison-snapshot-manifest does not accept extra arguments"
    );
    let comparison_snapshot_summary = render_cli(&["comparison-snapshot-summary"])
        .expect("comparison snapshot summary should render");
    assert!(comparison_snapshot_summary.contains("Comparison snapshot summary"));
    assert_eq!(
        comparison_snapshot_summary,
        format!(
            "Comparison snapshot summary\n{}\n",
            pleiades_jpl::comparison_snapshot_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["j2000-snapshot"]).expect("j2000 snapshot alias should render"),
        comparison_snapshot_summary
    );
    assert_eq!(
        render_cli(&["comparison-snapshot"]).expect("comparison snapshot alias should render"),
        comparison_snapshot_summary
    );
    assert_eq!(
        render_cli(&["j2000-snapshot", "extra"])
            .expect_err("j2000 snapshot alias should reject extra arguments"),
        "j2000-snapshot does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-summary", "extra"])
            .expect_err("comparison snapshot summary should reject extra arguments"),
        "comparison-snapshot-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-snapshot", "extra"])
            .expect_err("comparison snapshot alias should reject extra arguments"),
        "comparison-snapshot does not accept extra arguments"
    );
    let comparison_snapshot_batch_parity_summary =
        render_cli(&["comparison-snapshot-batch-parity-summary"])
            .expect("comparison snapshot batch parity summary should render");
    assert_eq!(
        comparison_snapshot_batch_parity_summary,
        validate_render_cli(&["comparison-snapshot-batch-parity-summary"])
            .expect("validation comparison snapshot batch parity summary should render")
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-batch-parity"])
            .expect("comparison snapshot batch parity alias should render"),
        comparison_snapshot_batch_parity_summary
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-batch-parity", "extra"])
            .expect_err("comparison snapshot batch parity alias should reject extra arguments"),
        "comparison-snapshot-batch-parity-summary does not accept extra arguments"
    );
    let reference_snapshot_manifest_summary = render_cli(&["reference-snapshot-manifest-summary"])
        .expect("reference snapshot manifest summary should render");
    assert!(reference_snapshot_manifest_summary.contains("Reference snapshot manifest:"));
    assert_eq!(
        reference_snapshot_manifest_summary,
        pleiades_jpl::reference_snapshot_manifest_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-manifest"])
            .expect("reference snapshot manifest alias should render"),
        reference_snapshot_manifest_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-manifest", "extra"])
            .expect_err("reference snapshot manifest alias should reject extra arguments"),
        "reference-snapshot-manifest does not accept extra arguments"
    );
    let reference_snapshot_source_summary = render_cli(&["reference-snapshot-source-summary"])
        .expect("reference snapshot source summary should render");
    assert!(reference_snapshot_source_summary.contains("Reference snapshot source:"));
    assert_eq!(
        reference_snapshot_source_summary,
        pleiades_jpl::reference_snapshot_source_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-source-summary", "extra"])
            .expect_err("reference snapshot source summary should reject extra arguments"),
        "reference-snapshot-source-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-source"])
            .expect("reference snapshot source alias should render"),
        reference_snapshot_source_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-source", "extra"])
            .expect_err("reference snapshot source alias should reject extra arguments"),
        "reference-snapshot-source does not accept extra arguments"
    );
    let reference_snapshot_summary = render_cli(&["reference-snapshot-summary"])
        .expect("reference snapshot summary should render");
    assert!(reference_snapshot_summary.contains("Reference snapshot summary"));
    assert!(reference_snapshot_summary.contains(
        &pleiades_jpl::reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()
    ));
    assert!(reference_snapshot_summary.contains(
        &pleiades_jpl::reference_snapshot_2451916_major_body_boundary_summary_for_report()
    ));
    assert!(reference_snapshot_summary.contains(
        &pleiades_jpl::reference_snapshot_2451916_major_body_boundary_summary_for_report()
    ));
    assert!(reference_snapshot_summary.contains(
        &pleiades_jpl::reference_snapshot_2451918_major_body_boundary_summary_for_report()
    ));
    assert!(reference_snapshot_summary.contains(
        &pleiades_jpl::reference_snapshot_2451919_major_body_boundary_summary_for_report()
    ));
    assert!(reference_snapshot_summary.contains(
        &pleiades_jpl::reference_snapshot_2451914_major_body_bridge_day_summary_for_report()
    ));
    assert!(reference_snapshot_summary
        .contains(&pleiades_jpl::selected_asteroid_boundary_summary_for_report()));
    assert!(reference_snapshot_summary
        .contains(&pleiades_jpl::selected_asteroid_bridge_summary_for_report()));
    assert!(reference_snapshot_summary
        .contains(&pleiades_jpl::selected_asteroid_dense_boundary_summary_for_report()));
    assert!(reference_snapshot_summary
        .contains(&pleiades_jpl::selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(reference_snapshot_summary
        .contains(&pleiades_jpl::selected_asteroid_source_evidence_summary_for_report()));
    assert!(reference_snapshot_summary
        .contains(&pleiades_jpl::selected_asteroid_source_window_summary_for_report()));
    assert!(!reference_snapshot_summary.contains("JPL independent hold-out:"));
    assert!(!reference_snapshot_summary.contains("Reference/hold-out overlap:"));
    assert_eq!(
        reference_snapshot_summary,
        format!(
            "Reference snapshot summary\n{}\n",
            pleiades_jpl::reference_snapshot_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["reference-snapshot"]).expect("reference snapshot alias should render"),
        reference_snapshot_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-summary", "extra"])
            .expect_err("reference snapshot summary should reject extra arguments"),
        "reference-snapshot-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot", "extra"])
            .expect_err("reference snapshot alias should reject extra arguments"),
        "reference-snapshot does not accept extra arguments"
    );

    let reference_snapshot_exact_j2000 =
        render_cli(&["reference-snapshot-exact-j2000-evidence-summary"])
            .expect("reference snapshot exact J2000 evidence should render");
    assert!(
        reference_snapshot_exact_j2000.contains("Reference snapshot exact J2000 evidence summary")
    );
    assert!(reference_snapshot_exact_j2000.contains(
        "Reference snapshot exact J2000 evidence: 16 exact J2000 samples at JD 2451545.0"
    ));
    assert_eq!(
        reference_snapshot_exact_j2000,
        format!(
            "Reference snapshot exact J2000 evidence summary\n{}\n",
            pleiades_jpl::reference_snapshot_exact_j2000_evidence_summary_for_report()
        )
    );
    let exact_j2000_evidence =
        render_cli(&["exact-j2000-evidence"]).expect("exact J2000 evidence alias should render");
    assert_eq!(exact_j2000_evidence, reference_snapshot_exact_j2000);
    let reference_snapshot_exact_j2000_alias =
        render_cli(&["reference-snapshot-exact-j2000-evidence"])
            .expect("reference snapshot exact J2000 evidence alias should render");
    assert_eq!(
        reference_snapshot_exact_j2000_alias,
        reference_snapshot_exact_j2000
    );
    assert_eq!(
        render_cli(&["exact-j2000-evidence", "extra"])
            .expect_err("exact J2000 evidence alias should reject extra arguments"),
        "exact-j2000-evidence does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-exact-j2000-evidence", "extra"]).expect_err(
            "reference snapshot exact J2000 evidence alias should reject extra arguments"
        ),
        "reference-snapshot-exact-j2000-evidence does not accept extra arguments"
    );
    let reference_snapshot_batch_parity_summary =
        render_cli(&["reference-snapshot-batch-parity-summary"])
            .expect("reference snapshot batch parity summary should render");
    assert_eq!(
        reference_snapshot_batch_parity_summary,
        validate_render_cli(&["reference-snapshot-batch-parity-summary"])
            .expect("validation reference snapshot batch parity summary should render")
    );
    let reference_snapshot_batch_parity_alias = render_cli(&["reference-snapshot-batch-parity"])
        .expect("reference snapshot batch parity alias should render");
    assert_eq!(
        reference_snapshot_batch_parity_alias,
        reference_snapshot_batch_parity_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-batch-parity", "extra"])
            .expect_err("reference snapshot batch parity alias should reject extra arguments"),
        "reference-snapshot-batch-parity does not accept extra arguments"
    );
    let reference_snapshot_mixed_time_scale_batch_parity_summary =
        render_cli(&["reference-snapshot-mixed-time-scale-batch-parity-summary"])
            .expect("reference snapshot mixed TT/TDB batch parity summary should render");
    assert_eq!(
        reference_snapshot_mixed_time_scale_batch_parity_summary,
        validate_render_cli(&["reference-snapshot-mixed-time-scale-batch-parity-summary"]).expect(
            "validation reference snapshot mixed TT/TDB batch parity summary should render"
        )
    );
    let reference_snapshot_mixed_tt_tdb_batch_parity_summary =
        render_cli(&["reference-snapshot-mixed-tt-tdb-batch-parity-summary"])
            .expect("reference snapshot mixed TT/TDB batch parity alias should render");
    assert_eq!(
        reference_snapshot_mixed_tt_tdb_batch_parity_summary,
        reference_snapshot_mixed_time_scale_batch_parity_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-mixed-tt-tdb-batch-parity"])
            .expect("reference snapshot mixed TT/TDB batch parity short alias should render"),
        reference_snapshot_mixed_time_scale_batch_parity_summary
    );
    let reference_snapshot_equatorial_parity_summary =
        render_cli(&["reference-snapshot-equatorial-parity-summary"])
            .expect("reference snapshot equatorial parity summary should render");
    assert_eq!(
        reference_snapshot_equatorial_parity_summary,
        validate_render_cli(&["reference-snapshot-equatorial-parity-summary"])
            .expect("validation reference snapshot equatorial parity summary should render")
    );
    let reference_snapshot_equatorial_parity_alias =
        render_cli(&["reference-snapshot-equatorial-parity"])
            .expect("reference snapshot equatorial parity alias should render");
    assert_eq!(
        reference_snapshot_equatorial_parity_alias,
        reference_snapshot_equatorial_parity_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-equatorial-parity", "extra"])
            .expect_err("reference snapshot equatorial parity alias should reject extra arguments"),
        "reference-snapshot-equatorial-parity does not accept extra arguments"
    );
    let time_scale_policy_summary = render_cli(&["time-scale-policy-summary"])
        .expect("time-scale policy summary should render");
    assert!(time_scale_policy_summary.contains("Time-scale policy summary"));
    assert!(time_scale_policy_summary.contains("Time-scale policy: direct backend requests accept TT/TDB; civil UTC/UT1 inputs convert via the pleiades-time crate or caller-supplied offsets; the ephemeris backends carry no internal Delta T or UTC convenience model"));
    assert_eq!(
        render_cli(&["time-scale-policy"]).expect("time-scale policy alias should render"),
        time_scale_policy_summary
    );
    let utc_convenience_policy_summary = render_cli(&["utc-convenience-policy-summary"])
        .expect("UTC convenience policy summary should render");
    assert!(utc_convenience_policy_summary.contains("UTC convenience policy summary"));
    assert!(utc_convenience_policy_summary.contains("UTC convenience policy: built-in UTC convenience conversion is now provided by the pleiades-time crate (civil UTC/UT1 to TT/TDB, leap-second-exact UTC, tiered exact/observed/predicted, 1900-2100); direct backends still consume TT/TDB"));
    assert_eq!(
        render_cli(&["utc-convenience-policy"])
            .expect("UTC convenience policy alias should render"),
        utc_convenience_policy_summary
    );
    let delta_t_policy_summary =
        render_cli(&["delta-t-policy-summary"]).expect("delta T policy summary should render");
    assert!(delta_t_policy_summary.contains("Delta T policy summary"));
    assert!(delta_t_policy_summary.contains("Delta T policy: built-in Delta T modeling is now provided by the pleiades-time crate for civil UTC/UT1 inputs over 1900-2100, tagged observed/predicted; direct backend requests still accept TT/TDB"));
    assert_eq!(
        render_cli(&["delta-t-policy"]).expect("delta T policy alias should render"),
        delta_t_policy_summary
    );
    let zodiac_policy_summary =
        render_cli(&["zodiac-policy-summary"]).expect("zodiac policy summary should render");
    assert!(zodiac_policy_summary.contains("Zodiac policy summary"));
    assert!(zodiac_policy_summary.contains("Zodiac policy: tropical only"));
    assert_eq!(
        render_cli(&["zodiac-policy"]).expect("zodiac policy alias should render"),
        zodiac_policy_summary
    );
    let observer_policy_summary =
        render_cli(&["observer-policy-summary"]).expect("observer policy summary should render");
    assert!(observer_policy_summary.contains("Observer policy summary"));
    assert!(observer_policy_summary.contains("Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported"));
    assert_eq!(
        render_cli(&["observer-policy"]).expect("observer policy alias should render"),
        observer_policy_summary
    );
    let apparentness_policy_summary = render_cli(&["apparentness-policy-summary"])
        .expect("apparentness policy summary should render");
    assert!(apparentness_policy_summary.contains("Apparentness policy summary"));
    assert!(apparentness_policy_summary.contains("Apparentness policy: backends remain mean-only and J2000 at the backend boundary; apparent place of date (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, release-grade bodies; gravitational light-deflection omitted"));
    assert_eq!(
        render_cli(&["apparentness-policy"]).expect("apparentness policy alias should render"),
        apparentness_policy_summary
    );
    let native_sidereal_policy_summary = render_cli(&["native-sidereal-policy-summary"])
        .expect("native sidereal policy summary should render");
    assert!(native_sidereal_policy_summary.contains("Native sidereal policy summary"));
    assert!(native_sidereal_policy_summary.contains("Native sidereal policy: native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    assert_eq!(
        render_cli(&["native-sidereal-policy"])
            .expect("native sidereal policy alias should render"),
        native_sidereal_policy_summary
    );
    let interpolation_posture_summary = render_cli(&["interpolation-posture-summary"])
        .expect("interpolation posture summary should render");
    assert_eq!(
        interpolation_posture_summary,
        validate_render_cli(&["interpolation-posture-summary"])
            .expect("validation interpolation posture summary should render")
    );
    assert_eq!(
        render_cli(&["interpolation-posture"]).expect("interpolation posture alias should render"),
        interpolation_posture_summary
    );
    assert_eq!(
        render_cli(&["interpolation-posture", "extra"])
            .expect_err("interpolation posture alias should reject extra arguments"),
        "interpolation-posture does not accept extra arguments"
    );
    let mean_obliquity_frame_round_trip_summary =
        render_cli(&["mean-obliquity-frame-round-trip-summary"])
            .expect("mean-obliquity frame round-trip summary should render");
    assert_eq!(
        mean_obliquity_frame_round_trip_summary,
        validate_render_cli(&["mean-obliquity-frame-round-trip-summary"])
            .expect("validation mean-obliquity frame round-trip summary should render")
    );
    assert_eq!(
        render_cli(&["mean-obliquity-frame-round-trip"])
            .expect("mean-obliquity frame round-trip alias should render"),
        mean_obliquity_frame_round_trip_summary
    );
    assert_eq!(
        render_cli(&["mean-obliquity-frame-round-trip", "extra"])
            .expect_err("mean-obliquity frame round-trip alias should reject extra arguments"),
        "mean-obliquity-frame-round-trip does not accept extra arguments"
    );
    let interpolation_quality_kind_coverage_summary =
        render_cli(&["interpolation-quality-kind-coverage-summary"])
            .expect("interpolation quality kind coverage summary should render");
    assert!(interpolation_quality_kind_coverage_summary
        .contains("JPL interpolation quality kind coverage:"));
    let interpolation_quality_request_corpus_summary =
        render_cli(&["interpolation-quality-request-corpus-summary"])
            .expect("interpolation quality request corpus summary should render");
    assert!(interpolation_quality_request_corpus_summary
        .contains("Interpolation-quality sample request corpus:"));
    assert_eq!(
        interpolation_quality_request_corpus_summary,
        pleiades_jpl::interpolation_quality_sample_request_corpus_summary_for_report()
    );
    assert_eq!(
        render_cli(&["interpolation-quality-request-corpus"])
            .expect("interpolation quality request corpus alias should render"),
        interpolation_quality_request_corpus_summary
    );
    let lunar_reference_error_envelope_summary =
        render_cli(&["lunar-reference-error-envelope-summary"])
            .expect("lunar reference error envelope summary should render");
    assert!(
        lunar_reference_error_envelope_summary.contains("Lunar reference error envelope summary")
    );
    assert_eq!(
        lunar_reference_error_envelope_summary,
        format!(
            "Lunar reference error envelope summary\n{}\n",
            pleiades_elp::lunar_reference_evidence_envelope_for_report()
        )
    );
    assert_eq!(
        render_cli(&["lunar-reference-error-envelope"]).unwrap(),
        lunar_reference_error_envelope_summary
    );
    let lunar_reference_evidence_summary = render_cli(&["lunar-reference-evidence-summary"])
        .expect("lunar reference evidence summary should render");
    assert!(lunar_reference_evidence_summary.contains("Lunar reference evidence summary"));
    assert_eq!(
        lunar_reference_evidence_summary,
        format!(
            "Lunar reference evidence summary\n{}\n",
            pleiades_elp::lunar_reference_evidence_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["lunar-reference-evidence"]).unwrap(),
        lunar_reference_evidence_summary
    );
    let lunar_equatorial_reference_error_envelope_summary =
        render_cli(&["lunar-equatorial-reference-error-envelope-summary"])
            .expect("lunar equatorial reference error envelope summary should render");
    assert!(lunar_equatorial_reference_error_envelope_summary
        .contains("Lunar equatorial reference error envelope summary"));
    assert_eq!(
        lunar_equatorial_reference_error_envelope_summary,
        format!(
            "Lunar equatorial reference error envelope summary\n{}\n",
            pleiades_elp::lunar_equatorial_reference_evidence_envelope_for_report()
        )
    );
    assert_eq!(
        render_cli(&["lunar-equatorial-reference-error-envelope"]).unwrap(),
        lunar_equatorial_reference_error_envelope_summary
    );
    let lunar_apparent_comparison_summary = render_cli(&["lunar-apparent-comparison-summary"])
        .expect("lunar apparent comparison summary should render");
    assert!(lunar_apparent_comparison_summary.contains("Lunar apparent comparison summary"));
    assert_eq!(
        lunar_apparent_comparison_summary,
        format!(
            "Lunar apparent comparison summary\n{}\n",
            pleiades_elp::lunar_apparent_comparison_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["lunar-apparent-comparison"]).unwrap(),
        lunar_apparent_comparison_summary
    );
    let lunar_source_window_summary = render_cli(&["lunar-source-window-summary"])
        .expect("lunar source window summary should render");
    assert!(lunar_source_window_summary.contains("lunar source windows"));
    assert_eq!(
        lunar_source_window_summary,
        pleiades_elp::lunar_source_window_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-lunar-source-window-summary"])
            .expect("reference snapshot lunar source window summary should render"),
        lunar_source_window_summary
    );
    assert_eq!(
        render_cli(&["lunar-source-window"]).expect("lunar source window alias should render"),
        lunar_source_window_summary
    );
    let lunar_reference_mixed_time_scale_batch_parity_summary =
        render_cli(&["lunar-reference-mixed-time-scale-batch-parity-summary"])
            .expect("lunar reference mixed TT/TDB batch parity summary should render");
    assert!(lunar_reference_mixed_time_scale_batch_parity_summary
        .contains("lunar reference mixed TT/TDB batch parity"));
    assert_eq!(
        lunar_reference_mixed_time_scale_batch_parity_summary,
        pleiades_elp::lunar_reference_batch_parity_summary_for_report()
    );
    assert_eq!(
        render_cli(&["lunar-reference-mixed-tt-tdb-batch-parity-summary"])
            .expect("lunar reference mixed TT/TDB batch parity alias should render"),
        lunar_reference_mixed_time_scale_batch_parity_summary
    );
    assert_eq!(
        render_cli(&["lunar-reference-mixed-tt-tdb-batch-parity"])
            .expect("lunar reference mixed TT/TDB batch parity short alias should render"),
        lunar_reference_mixed_time_scale_batch_parity_summary
    );
    assert_eq!(
        render_cli(&[
            "lunar-reference-mixed-time-scale-batch-parity-summary",
            "extra"
        ])
        .expect_err(
            "lunar reference mixed TT/TDB batch parity summary should reject extra arguments"
        ),
        "lunar-reference-mixed-time-scale-batch-parity-summary does not accept extra arguments"
    );
    let lunar_theory_request_policy_summary = render_cli(&["lunar-theory-request-policy-summary"])
        .expect("lunar theory request policy summary should render");
    assert_eq!(
        lunar_theory_request_policy_summary,
        pleiades_elp::lunar_theory_request_policy_summary()
    );
    assert_eq!(
        render_cli(&["lunar-theory-request-policy"])
            .expect("lunar theory request policy alias should render"),
        lunar_theory_request_policy_summary
    );
    assert_eq!(
        render_cli(&["lunar-theory-request-policy", "extra"])
            .expect_err("lunar theory request policy alias should reject extra arguments"),
        "lunar-theory-request-policy does not accept extra arguments"
    );
    let lunar_theory_frame_treatment_summary =
        render_cli(&["lunar-theory-frame-treatment-summary"])
            .expect("lunar theory frame treatment summary should render");
    assert_eq!(
        lunar_theory_frame_treatment_summary,
        pleiades_elp::lunar_theory_frame_treatment_summary_for_report()
    );
    assert_eq!(
        render_cli(&["lunar-theory-frame-treatment"])
            .expect("lunar theory frame treatment alias should render"),
        lunar_theory_frame_treatment_summary
    );
    assert_eq!(
        render_cli(&["lunar-theory-frame-treatment", "extra"])
            .expect_err("lunar theory frame treatment alias should reject extra arguments"),
        "lunar-theory-frame-treatment does not accept extra arguments"
    );
    let lunar_theory_limitations_summary = render_cli(&["lunar-theory-limitations-summary"])
        .expect("lunar theory limitations summary should render");
    assert_eq!(
        lunar_theory_limitations_summary,
        pleiades_elp::lunar_theory_limitations_summary_for_report()
    );
    assert_eq!(
        render_cli(&["lunar-theory-limitations"]).unwrap(),
        pleiades_elp::lunar_theory_limitations_summary_for_report()
    );
    let lunar_theory_summary =
        render_cli(&["lunar-theory-summary"]).expect("lunar theory summary should render");
    assert!(lunar_theory_summary.contains("ELP lunar theory specification:"));
    assert_eq!(
        lunar_theory_summary,
        pleiades_elp::lunar_theory_summary_for_report()
    );
    let lunar_theory_capability_summary = render_cli(&["lunar-theory-capability-summary"])
        .expect("lunar theory capability summary should render");
    assert!(lunar_theory_capability_summary.contains("lunar capability summary:"));
    assert_eq!(
        lunar_theory_capability_summary,
        pleiades_elp::lunar_theory_capability_summary_for_report()
    );
    let lunar_theory_source_summary = render_cli(&["lunar-theory-source-summary"])
        .expect("lunar theory source summary should render");
    assert!(lunar_theory_source_summary.contains("lunar source selection:"));
    assert_eq!(
        lunar_theory_source_summary,
        pleiades_elp::lunar_theory_source_summary_for_report()
    );
    let lunar_theory_source_selection_summary =
        render_cli(&["lunar-theory-source-selection-summary"])
            .expect("lunar theory source selection summary should render");
    assert!(lunar_theory_source_selection_summary.contains("lunar source selection:"));
    assert_eq!(
        lunar_theory_source_selection_summary,
        pleiades_elp::lunar_theory_source_selection_summary_for_report()
    );
    assert_eq!(
        render_cli(&["lunar-theory-source-selection"]).unwrap(),
        lunar_theory_source_selection_summary
    );
    assert_eq!(
        render_cli(&["lunar-theory-source-selection", "extra"])
            .expect_err("lunar theory source selection alias should reject extra arguments"),
        "lunar-theory-source-selection-summary does not accept extra arguments"
    );
    let lunar_theory_source_family_summary = render_cli(&["lunar-theory-source-family-summary"])
        .expect("lunar theory source family summary should render");
    assert!(lunar_theory_source_family_summary.contains("lunar source family:"));
    assert_eq!(
        lunar_theory_source_family_summary,
        pleiades_elp::lunar_theory_source_family_summary_for_report()
    );
    assert_eq!(
        render_cli(&["lunar-theory-source-family"]).unwrap(),
        lunar_theory_source_family_summary
    );
    let lunar_theory_catalog_summary = render_cli(&["lunar-theory-catalog-summary"])
        .expect("lunar theory catalog summary should render");
    assert_eq!(
        lunar_theory_catalog_summary,
        pleiades_elp::lunar_theory_catalog_summary_for_report()
    );
    let lunar_theory_catalog_validation_summary =
        render_cli(&["lunar-theory-catalog-validation-summary"])
            .expect("lunar theory catalog validation summary should render");
    assert_eq!(
        lunar_theory_catalog_validation_summary,
        pleiades_elp::lunar_theory_catalog_validation_summary_for_report()
    );
    let selected_asteroid_boundary_summary = render_cli(&["selected-asteroid-boundary-summary"])
        .expect("selected asteroid boundary summary should render");
    assert!(selected_asteroid_boundary_summary.contains("Selected asteroid boundary evidence"));
    assert!(selected_asteroid_boundary_summary.contains("2451914.5"));
    assert!(selected_asteroid_boundary_summary.starts_with("Selected asteroid boundary evidence:"));
    let selected_asteroid_bridge_summary = render_cli(&["selected-asteroid-bridge-summary"])
        .expect("selected asteroid bridge summary should render");
    assert!(selected_asteroid_bridge_summary.contains("Selected asteroid bridge evidence"));
    assert!(selected_asteroid_bridge_summary.contains("2451915.0"));
    assert_eq!(
        selected_asteroid_bridge_summary,
        pleiades_jpl::selected_asteroid_bridge_summary_for_report()
    );
    assert_eq!(
        render_cli(&["selected-asteroid-bridge-summary", "extra"])
            .expect_err("selected asteroid bridge summary alias should reject extra arguments"),
        "selected-asteroid-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&[
            "reference-snapshot-selected-asteroid-bridge-summary",
            "extra"
        ])
        .expect_err(
            "reference-snapshot-selected-asteroid-bridge-summary should reject extra arguments"
        ),
        "reference-snapshot-selected-asteroid-bridge-summary does not accept extra arguments"
    );
    let reference_major_body_bridge_summary =
        render_cli(&["reference-snapshot-major-body-bridge-summary"])
            .expect("reference major-body bridge summary should render");
    assert!(reference_major_body_bridge_summary.contains("Reference major-body bridge evidence"));
    assert!(reference_major_body_bridge_summary.contains("2451915.0"));
    assert_eq!(
        reference_major_body_bridge_summary,
        pleiades_jpl::reference_snapshot_major_body_bridge_summary_for_report()
    );
    let reference_major_body_bridge_alias =
        render_cli(&["major-body-bridge-summary"]).expect("major body bridge alias should render");
    assert_eq!(
        reference_major_body_bridge_alias,
        reference_major_body_bridge_summary
    );
    assert_eq!(
        render_cli(&["major-body-bridge-summary", "extra"])
            .expect_err("major body bridge alias should reject extra arguments"),
        "major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-major-body-bridge-summary", "extra"])
            .expect_err("reference major-body bridge summary should reject extra arguments"),
        "reference-snapshot-major-body-bridge-summary does not accept extra arguments"
    );
    let reference_bridge_day_summary = render_cli(&["reference-snapshot-bridge-day-summary"])
        .expect("reference snapshot bridge day summary should render");
    assert!(reference_bridge_day_summary.contains("Reference snapshot bridge day:"));
    assert!(reference_bridge_day_summary.contains("2451914.0"));
    assert_eq!(
        reference_bridge_day_summary,
        pleiades_jpl::reference_snapshot_bridge_day_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-bridge-day-summary", "extra"])
            .expect_err("reference snapshot bridge day summary should reject extra arguments"),
        "reference-snapshot-bridge-day-summary does not accept extra arguments"
    );
    let bridge_day_alias =
        render_cli(&["bridge-day-summary"]).expect("bridge day alias should render");
    assert_eq!(bridge_day_alias, reference_bridge_day_summary);
    assert_eq!(
        render_cli(&["bridge-day-summary", "extra"])
            .expect_err("bridge day alias should reject extra arguments"),
        "bridge-day-summary does not accept extra arguments"
    );
    let bridge_day_epoch_alias = render_cli(&["2451914-bridge-day-summary"])
        .expect("2451914 bridge day alias should render");
    assert_eq!(bridge_day_epoch_alias, reference_bridge_day_summary);
    assert_eq!(
        render_cli(&["2451914-bridge-day-summary", "extra"])
            .expect_err("2451914 bridge day alias should reject extra arguments"),
        "reference-snapshot-2451914-bridge-day-summary does not accept extra arguments"
    );
    let bridge_day_major_alias = render_cli(&["2451914-major-body-bridge-day-summary"])
        .expect("2451914 major body bridge-day alias should render");
    assert_eq!(
        bridge_day_major_alias,
        pleiades_jpl::reference_snapshot_2451914_major_body_bridge_day_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2451914-major-body-bridge-day-summary", "extra"])
            .expect_err("2451914 major body bridge-day alias should reject extra arguments"),
        "reference-snapshot-2451914-major-body-bridge-day-summary does not accept extra arguments"
    );
    let selected_asteroid_dense_boundary_summary =
        render_cli(&["reference-snapshot-selected-asteroid-dense-boundary-summary"])
            .expect("selected asteroid dense boundary summary should render");
    assert!(selected_asteroid_dense_boundary_summary
        .contains("Selected asteroid dense boundary evidence"));
    assert!(selected_asteroid_dense_boundary_summary.contains("2451916.5"));
    assert_eq!(
        selected_asteroid_dense_boundary_summary,
        pleiades_jpl::selected_asteroid_dense_boundary_summary_for_report()
    );
    let selected_asteroid_dense_boundary_alias =
        render_cli(&["selected-asteroid-dense-boundary-summary"])
            .expect("selected asteroid dense boundary alias should render");
    assert_eq!(
        selected_asteroid_dense_boundary_alias,
        selected_asteroid_dense_boundary_summary
    );
    assert_eq!(
        render_cli(&["selected-asteroid-dense-boundary-summary", "extra"])
            .expect_err("selected asteroid dense boundary alias should reject extra arguments"),
        "selected-asteroid-dense-boundary-summary does not accept extra arguments"
    );
    let selected_asteroid_terminal_boundary_summary =
        render_cli(&["reference-snapshot-selected-asteroid-terminal-boundary-summary"])
            .expect("selected asteroid terminal boundary summary should render");
    assert!(selected_asteroid_terminal_boundary_summary.contains("terminal boundary evidence"));
    assert!(selected_asteroid_terminal_boundary_summary.contains("2500-01-01"));
    assert_eq!(
        selected_asteroid_terminal_boundary_summary,
        pleiades_jpl::selected_asteroid_terminal_boundary_summary_for_report()
    );
    let selected_asteroid_terminal_boundary_alias =
        render_cli(&["selected-asteroid-terminal-boundary-summary"])
            .expect("selected asteroid terminal boundary alias should render");
    assert_eq!(
        selected_asteroid_terminal_boundary_alias,
        selected_asteroid_terminal_boundary_summary
    );
    assert_eq!(
        render_cli(&["selected-asteroid-terminal-boundary-summary", "extra"])
            .expect_err("selected asteroid terminal boundary alias should reject extra arguments"),
        "selected-asteroid-terminal-boundary-summary does not accept extra arguments"
    );
    let selected_asteroid_source_evidence_summary =
        render_cli(&["selected-asteroid-source-evidence-summary"])
            .expect("selected asteroid source evidence summary should render");
    assert!(selected_asteroid_source_evidence_summary.contains("Selected asteroid source evidence"));
    assert!(selected_asteroid_source_evidence_summary.contains("Ceres"));
    assert_eq!(
        selected_asteroid_source_evidence_summary,
        pleiades_jpl::selected_asteroid_source_evidence_summary_for_report()
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-summary"])
            .expect("selected asteroid source summary alias should render"),
        selected_asteroid_source_evidence_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-summary"])
            .expect("reference snapshot selected asteroid source summary alias should render"),
        selected_asteroid_source_evidence_summary
    );

    let selected_asteroid_source_2378498_summary =
        render_cli(&["reference-snapshot-2378498-selected-asteroid-source-summary"])
            .expect("reference selected asteroid 2378498 source summary should render");
    assert!(selected_asteroid_source_2378498_summary.contains("2378498.5"));
    assert_eq!(
        selected_asteroid_source_2378498_summary,
        pleiades_jpl::selected_asteroid_source_2378498_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2378498-selected-asteroid-source-summary"])
            .expect("selected asteroid 2378498 source summary alias should render"),
        selected_asteroid_source_2378498_summary
    );
    assert_eq!(
        render_cli(&["2378498-selected-asteroid-source-summary", "extra"])
            .expect_err(
                "selected asteroid 2378498 source summary alias should reject extra arguments"
            ),
        "reference-snapshot-2378498-selected-asteroid-source-summary does not accept extra arguments"
    );

    let selected_asteroid_source_request_corpus_summary =
        render_cli(&["selected-asteroid-source-request-corpus-summary"])
            .expect("selected asteroid source request corpus summary should render");
    assert!(selected_asteroid_source_request_corpus_summary
        .contains("Selected asteroid source request corpus:"));
    assert!(selected_asteroid_source_request_corpus_summary.contains("observerless"));
    assert_eq!(
        selected_asteroid_source_request_corpus_summary,
        pleiades_jpl::selected_asteroid_source_request_corpus_summary_for_report()
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-request-corpus"])
            .expect("selected asteroid source request corpus alias should render"),
        selected_asteroid_source_request_corpus_summary
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-request-corpus-equatorial-summary"])
            .expect("selected asteroid source request corpus equatorial summary should render"),
        pleiades_jpl::selected_asteroid_source_request_corpus_equatorial_summary_for_report()
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-request-corpus-equatorial"])
            .expect("selected asteroid source request corpus equatorial alias should render"),
        pleiades_jpl::selected_asteroid_source_request_corpus_equatorial_summary_for_report()
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-request-corpus-summary", "extra"]).expect_err(
            "selected asteroid source request corpus summary should reject extra arguments"
        ),
        "selected-asteroid-source-request-corpus-summary does not accept extra arguments"
    );

    let selected_asteroid_source_window_summary =
        render_cli(&["selected-asteroid-source-window-summary"])
            .expect("selected asteroid source window summary should render");
    assert!(selected_asteroid_source_window_summary.contains("Selected asteroid source windows"));
    assert!(selected_asteroid_source_window_summary.contains("Ceres"));
    assert_eq!(
        selected_asteroid_source_window_summary,
        pleiades_jpl::selected_asteroid_source_window_summary_for_report()
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-window"])
            .expect("selected asteroid source window alias should render"),
        selected_asteroid_source_window_summary
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-window", "extra"])
            .expect_err("selected asteroid source window alias should reject extra arguments"),
        "selected-asteroid-source-window does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-window-summary"])
            .expect("reference snapshot selected asteroid source window summary should render"),
        selected_asteroid_source_window_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-window"])
            .expect("reference snapshot selected asteroid source window alias should render"),
        selected_asteroid_source_window_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-window", "extra"])
            .expect_err("reference snapshot selected asteroid source window alias should reject extra arguments"),
        "reference-snapshot-selected-asteroid-source-window does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-2451917-selected-asteroid-source-summary"])
            .expect("reference snapshot 2451917 selected asteroid source summary should render"),
        pleiades_jpl::selected_asteroid_source_2451917_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2451917-selected-asteroid-source-summary"])
            .expect("2451917 selected asteroid source summary alias should render"),
        pleiades_jpl::selected_asteroid_source_2451917_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2451917-selected-asteroid-source-summary", "extra"]).expect_err(
            "2451917 selected asteroid source summary alias should reject extra arguments"
        ),
        "reference-snapshot-2451917-selected-asteroid-source-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-2453000-selected-asteroid-source-summary"])
            .expect("reference snapshot 2453000 selected asteroid source summary should render"),
        pleiades_jpl::selected_asteroid_source_2453000_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2453000-selected-asteroid-source-summary"])
            .expect("2453000 selected asteroid source summary alias should render"),
        pleiades_jpl::selected_asteroid_source_2453000_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2453000-selected-asteroid-source-summary", "extra"]).expect_err(
            "2453000 selected asteroid source summary alias should reject extra arguments"
        ),
        "reference-snapshot-2453000-selected-asteroid-source-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-2500000-selected-asteroid-source-summary"])
            .expect("reference snapshot 2500000 selected asteroid source summary should render"),
        pleiades_jpl::selected_asteroid_source_2500000_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2500000-selected-asteroid-source-summary"])
            .expect("2500000 selected asteroid source summary alias should render"),
        pleiades_jpl::selected_asteroid_source_2500000_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2500000-selected-asteroid-source-summary", "extra"]).expect_err(
            "2500000 selected asteroid source summary alias should reject extra arguments"
        ),
        "reference-snapshot-2500000-selected-asteroid-source-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-2634167-selected-asteroid-source-summary"])
            .expect("reference snapshot 2634167 selected asteroid source summary should render"),
        pleiades_jpl::selected_asteroid_source_2634167_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2634167-selected-asteroid-source-summary"])
            .expect("2634167 selected asteroid source summary alias should render"),
        pleiades_jpl::selected_asteroid_source_2634167_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2634167-selected-asteroid-source-summary", "extra"]).expect_err(
            "2634167 selected asteroid source summary alias should reject extra arguments"
        ),
        "reference-snapshot-2634167-selected-asteroid-source-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-window", "extra"])
            .expect_err("selected asteroid source window alias should reject extra arguments"),
        "selected-asteroid-source-window does not accept extra arguments"
    );
    let selected_asteroid_batch_parity_summary =
        render_cli(&["selected-asteroid-batch-parity-summary"])
            .expect("selected asteroid batch parity summary should render");
    assert!(selected_asteroid_batch_parity_summary.contains("Selected asteroid batch parity"));
    assert!(selected_asteroid_batch_parity_summary.contains("batch/single parity preserved"));
    assert_eq!(
        selected_asteroid_batch_parity_summary,
        pleiades_jpl::selected_asteroid_batch_parity_summary_for_report()
    );
    assert_eq!(
        render_cli(&["selected-asteroid-batch-parity"])
            .expect("selected asteroid batch parity alias should render"),
        selected_asteroid_batch_parity_summary
    );
    let reference_asteroid_evidence_summary = render_cli(&["reference-asteroid-evidence-summary"])
        .expect("reference asteroid evidence summary should render");
    assert!(reference_asteroid_evidence_summary.contains("Selected asteroid evidence:"));
    assert!(reference_asteroid_evidence_summary.contains("Ceres"));
    assert_eq!(
        reference_asteroid_evidence_summary,
        pleiades_jpl::reference_asteroid_evidence_summary_for_report()
    );
    let reference_asteroid_equatorial_evidence_summary =
        render_cli(&["reference-asteroid-equatorial-evidence-summary"])
            .expect("reference asteroid equatorial evidence summary should render");
    assert!(reference_asteroid_equatorial_evidence_summary
        .contains("Selected asteroid equatorial evidence:"));
    assert!(reference_asteroid_equatorial_evidence_summary
        .contains("mean-obliquity equatorial transform"));
    assert_eq!(
        reference_asteroid_equatorial_evidence_summary,
        pleiades_jpl::reference_asteroid_equatorial_evidence_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-asteroid-equatorial-evidence"])
            .expect("reference asteroid equatorial evidence alias should render"),
        reference_asteroid_equatorial_evidence_summary
    );
    let reference_asteroid_source_window_summary =
        render_cli(&["reference-asteroid-source-window-summary"])
            .expect("reference asteroid source window summary should render");
    assert!(reference_asteroid_source_window_summary.contains("Reference asteroid source windows:"));
    assert!(reference_asteroid_source_window_summary
        .contains("source-backed samples across 6 bodies and 17 epochs"));
    assert_eq!(
        reference_asteroid_source_window_summary,
        pleiades_jpl::reference_asteroid_source_window_summary_for_report()
    );
    let reference_asteroid_source_window_alias = render_cli(&["reference-asteroid-source-window"])
        .expect("reference asteroid source window alias should render");
    assert_eq!(
        reference_asteroid_source_window_alias,
        reference_asteroid_source_window_summary
    );
    assert_eq!(
        render_cli(&["reference-asteroid-source-window", "extra"])
            .expect_err("reference asteroid source window alias should reject extra arguments"),
        "reference-asteroid-source-window-summary does not accept extra arguments"
    );
    let reference_asteroid_source_summary = render_cli(&["reference-asteroid-source-summary"])
        .expect("reference asteroid source summary should render");
    assert_eq!(
        reference_asteroid_source_summary,
        reference_asteroid_source_window_summary
    );
    let reference_holdout_overlap_summary = render_cli(&["reference-holdout-overlap-summary"])
        .expect("reference/hold-out overlap summary should render");
    assert!(reference_holdout_overlap_summary.contains("Reference/hold-out overlap:"));
    assert!(reference_holdout_overlap_summary.contains("shared body-epoch pairs"));
    assert_eq!(
        reference_holdout_overlap_summary,
        pleiades_jpl::reference_holdout_overlap_summary_for_report()
    );
    let holdout_overlap_summary =
        render_cli(&["holdout-overlap-summary"]).expect("hold-out overlap alias should render");
    assert_eq!(holdout_overlap_summary, reference_holdout_overlap_summary);
    assert_eq!(
        render_cli(&["holdout-overlap-summary", "extra"])
            .expect_err("hold-out overlap alias should reject extra arguments"),
        "holdout-overlap-summary does not accept extra arguments"
    );
    let independent_holdout_source_window_summary =
        render_cli(&["independent-holdout-source-window-summary"])
            .expect("independent hold-out source window summary should render");
    assert!(
        independent_holdout_source_window_summary.contains("Independent hold-out source windows:")
    );
    assert!(independent_holdout_source_window_summary.contains("source-backed samples"));
    assert_eq!(
        independent_holdout_source_window_summary,
        pleiades_jpl::independent_holdout_snapshot_source_window_summary_for_report()
    );
    let independent_holdout = render_cli(&["independent-holdout-summary"])
        .expect("independent hold-out summary should render");
    assert!(independent_holdout.contains("JPL independent hold-out:"));
    assert!(independent_holdout.contains("transparency evidence only"));
    assert_eq!(
        independent_holdout,
        pleiades_jpl::jpl_independent_holdout_summary_for_report()
    );

    let independent_holdout_source_summary = render_cli(&["independent-holdout-source-summary"])
        .expect("independent hold-out source summary should render");
    assert!(independent_holdout_source_summary.contains("Independent hold-out source:"));
    assert!(independent_holdout_source_summary.contains("hold-out source"));
    assert_eq!(
        independent_holdout_source_summary,
        pleiades_jpl::independent_holdout_source_summary_for_report()
    );
    let independent_holdout_manifest_summary =
        render_cli(&["independent-holdout-manifest-summary"])
            .expect("independent hold-out manifest summary should render");
    assert!(independent_holdout_manifest_summary.contains("Independent hold-out manifest:"));
    assert!(independent_holdout_manifest_summary.contains("repository-checked regression fixtures"));
    assert_eq!(
        independent_holdout_manifest_summary,
        pleiades_jpl::independent_holdout_manifest_summary_for_report()
    );
    assert_eq!(
        render_cli(&["independent-holdout-manifest"])
            .expect("independent hold-out manifest alias should render"),
        independent_holdout_manifest_summary
    );
    assert_eq!(
        render_cli(&["independent-holdout-manifest", "extra"])
            .expect_err("independent hold-out manifest alias should reject extra arguments"),
        "independent-holdout-manifest does not accept extra arguments"
    );
    let independent_holdout_high_curvature_summary =
        render_cli(&["independent-holdout-high-curvature-summary"])
            .expect("independent hold-out high-curvature summary should render");
    assert!(independent_holdout_high_curvature_summary
        .contains("JPL independent hold-out high-curvature evidence:"));
    assert!(
        independent_holdout_high_curvature_summary.contains("high-curvature interpolation window")
    );
    assert_eq!(
        independent_holdout_high_curvature_summary,
        pleiades_jpl::independent_holdout_high_curvature_summary_for_report()
    );
    let independent_holdout_batch_parity_summary =
        render_cli(&["independent-holdout-batch-parity-summary"])
            .expect("independent hold-out batch parity summary should render");
    assert!(
        independent_holdout_batch_parity_summary.contains("JPL independent hold-out batch parity:")
    );
    assert!(independent_holdout_batch_parity_summary.contains("single-query parity=preserved"));
    assert_eq!(
        independent_holdout_batch_parity_summary,
        pleiades_jpl::independent_holdout_snapshot_batch_parity_summary_for_report()
    );
    assert_eq!(
        render_cli(&["independent-holdout-batch-parity"])
            .expect("independent hold-out batch parity alias should render"),
        independent_holdout_batch_parity_summary
    );
    let independent_holdout_equatorial_parity_summary =
        render_cli(&["independent-holdout-equatorial-parity-summary"])
            .expect("independent hold-out equatorial parity summary should render");
    assert!(independent_holdout_equatorial_parity_summary
        .contains("JPL independent hold-out equatorial parity:"));
    assert!(independent_holdout_equatorial_parity_summary
        .contains("mean-obliquity transform against the checked-in ecliptic fixture"));
    assert_eq!(
        independent_holdout_equatorial_parity_summary,
        pleiades_jpl::independent_holdout_snapshot_equatorial_parity_summary_for_report()
    );
    assert_eq!(
        render_cli(&["independent-holdout-equatorial-parity"])
            .expect("independent hold-out equatorial parity alias should render"),
        independent_holdout_equatorial_parity_summary
    );
    assert_eq!(
        render_cli(&["independent-holdout-equatorial-parity-summary", "extra"]).expect_err(
            "independent hold-out equatorial parity summary should reject extra arguments"
        ),
        "independent-holdout-equatorial-parity-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["independent-holdout-equatorial-parity", "extra"]).expect_err(
            "independent hold-out equatorial parity alias should reject extra arguments"
        ),
        "independent-holdout-equatorial-parity-summary does not accept extra arguments"
    );
    let house_validation_summary =
        render_cli(&["house-validation-summary"]).expect("house validation summary should render");
    assert!(house_validation_summary.contains("House validation corpus: 9 scenarios"));
    assert!(house_validation_summary
        .contains("formula families: Equal, Whole Sign, Quadrant, Equatorial projection"));
    assert!(house_validation_summary
        .contains("latitude-sensitive systems: Koch, Placidus, Topocentric"));
    assert_eq!(
        house_validation_summary,
        house_validation_summary_for_report()
    );
    assert_eq!(
        render_cli(&["house-validation-summary", "extra"])
            .expect_err("house validation summary should reject extra arguments"),
        "house-validation-summary does not accept extra arguments"
    );
    let house_validation_alias =
        render_cli(&["house-validation"]).expect("house validation alias should render");
    assert_eq!(house_validation_alias, house_validation_summary);
    assert_eq!(
        render_cli(&["house-validation", "extra"])
            .expect_err("house validation alias should reject extra arguments"),
        "house-validation does not accept extra arguments"
    );
    let house_formula_families_summary = render_cli(&["house-formula-families-summary"])
        .expect("house formula families summary should render");
    assert!(house_formula_families_summary.contains("Equal"));
    assert!(house_formula_families_summary.contains("Whole Sign"));
    assert!(house_formula_families_summary.contains("Quadrant"));
    assert_eq!(
        house_formula_families_summary,
        validate_render_cli(&["house-formula-families-summary"])
            .expect("validate facade should render house formula families summary")
    );
    assert_eq!(
        render_cli(&["house-formula-families"])
            .expect("house formula families alias should render"),
        house_formula_families_summary
    );
    let house_latitude_sensitive_summary = render_cli(&["house-latitude-sensitive-summary"])
        .expect("latitude-sensitive house systems summary should render");
    assert_eq!(
        house_latitude_sensitive_summary,
        "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
    );
    assert_eq!(
        house_latitude_sensitive_summary,
        validate_render_cli(&["house-latitude-sensitive-summary"])
            .expect("validate facade should render latitude-sensitive house systems summary")
    );
    assert_eq!(
        render_cli(&["house-latitude-sensitive"])
            .expect("latitude-sensitive house systems alias should render"),
        house_latitude_sensitive_summary
    );
    assert_eq!(
        render_cli(&["house-latitude-sensitive-summary", "extra"])
            .expect_err("latitude-sensitive house systems summary should reject extra arguments"),
        "house-latitude-sensitive-summary does not accept extra arguments"
    );
    let house_latitude_sensitive_failure_modes_summary =
        render_cli(&["house-latitude-sensitive-failure-modes-summary"])
            .expect("latitude-sensitive house failure modes summary should render");
    assert_eq!(
        house_latitude_sensitive_failure_modes_summary,
        validate_render_cli(&["house-latitude-sensitive-failure-modes-summary"])
            .expect("validate facade should render latitude-sensitive house failure modes summary")
    );
    assert_eq!(
        render_cli(&["house-latitude-sensitive-failure-modes"])
            .expect("latitude-sensitive house failure modes alias should render"),
        house_latitude_sensitive_failure_modes_summary
    );
    assert_eq!(
        render_cli(&["house-latitude-sensitive-failure-modes-summary", "extra"]).expect_err(
            "latitude-sensitive house failure modes summary should reject extra arguments"
        ),
        "house-latitude-sensitive-failure-modes-summary does not accept extra arguments"
    );
    let house_latitude_sensitive_constraints_summary =
        render_cli(&["house-latitude-sensitive-constraints-summary"])
            .expect("latitude-sensitive house constraints summary should render");
    assert_eq!(
        house_latitude_sensitive_constraints_summary,
        validate_render_cli(&["house-latitude-sensitive-constraints-summary"])
            .expect("validate facade should render latitude-sensitive house constraints summary")
    );
    assert_eq!(
        render_cli(&["house-latitude-sensitive-constraints"])
            .expect("latitude-sensitive house constraints alias should render"),
        house_latitude_sensitive_constraints_summary
    );
    assert_eq!(
        render_cli(&["house-latitude-sensitive-constraints-summary", "extra"]).expect_err(
            "latitude-sensitive house constraints summary should reject extra arguments"
        ),
        "house-latitude-sensitive-constraints-summary does not accept extra arguments"
    );
    let house_code_aliases_summary = render_cli(&["house-code-aliases-summary"])
        .expect("house code aliases summary should render");
    assert!(house_code_aliases_summary.contains("P -> Placidus"));
    assert_eq!(
        house_code_aliases_summary,
        validate_render_cli(&["house-code-aliases-summary"])
            .expect("validate facade should render house code aliases summary")
    );
    assert_eq!(
        render_cli(&["house-code-alias-summary"])
            .expect("house code alias shorthand should render"),
        house_code_aliases_summary
    );
    assert_eq!(
        render_cli(&["house-code-alias-summary", "extra"])
            .expect_err("house code alias shorthand should reject extra arguments"),
        "house-code-alias-summary does not accept extra arguments"
    );
    let catalog_inventory_summary = render_cli(&["catalog-inventory-summary"])
        .expect("catalog inventory summary should render");
    assert!(catalog_inventory_summary.contains("Compatibility catalog inventory:"));
    assert_eq!(
        catalog_inventory_summary,
        validate_render_cli(&["catalog-inventory-summary"])
            .expect("validate facade should render catalog inventory summary")
    );
    assert_eq!(
        render_cli(&["catalog-inventory"]).expect("catalog inventory alias should render"),
        catalog_inventory_summary
    );
    let catalog_posture_summary =
        render_cli(&["catalog-posture-summary"]).expect("catalog posture summary should render");
    assert_eq!(
        catalog_posture_summary,
        validate_render_cli(&["catalog-posture-summary"])
            .expect("validate facade should render catalog posture summary")
    );
    assert_eq!(
        render_cli(&["catalog-posture"]).expect("catalog posture alias should render"),
        catalog_posture_summary
    );
    assert_eq!(
        render_cli(&["catalog-posture-summary", "extra"]).unwrap_err(),
        "catalog-posture-summary does not accept extra arguments"
    );
    let known_gaps_summary =
        render_cli(&["known-gaps-summary"]).expect("known gaps summary should render");
    assert_eq!(
        known_gaps_summary,
        validate_render_cli(&["known-gaps-summary"])
            .expect("validate facade should render known gaps summary")
    );
    assert_eq!(
        render_cli(&["known-gaps"]).expect("known gaps alias should render"),
        known_gaps_summary
    );
    assert_eq!(
        render_cli(&["known-gaps-summary", "extra"]).unwrap_err(),
        "known-gaps-summary does not accept extra arguments"
    );
    let ayanamsa_catalog_validation_summary_rendered =
        render_cli(&["ayanamsa-catalog-validation-summary"])
            .expect("ayanamsa catalog validation summary should render");
    assert_eq!(
        render_cli(&["ayanamsa-catalog-validation"]).unwrap(),
        ayanamsa_catalog_validation_summary_rendered
    );
    assert!(
        ayanamsa_catalog_validation_summary_rendered.contains("ayanamsa catalog validation: ok")
    );
    assert!(ayanamsa_catalog_validation_summary_rendered.contains("baseline=5, release=54"));
    assert!(ayanamsa_catalog_validation_summary_rendered.contains("custom-definition-only="));
    let ayanamsa_metadata_coverage_summary = render_cli(&["ayanamsa-metadata-coverage-summary"])
        .expect("ayanamsa metadata coverage summary should render");
    assert_eq!(
        render_cli(&["ayanamsa-metadata-coverage"]).unwrap(),
        ayanamsa_metadata_coverage_summary
    );
    assert!(ayanamsa_metadata_coverage_summary.contains("ayanamsa sidereal metadata:"));
    assert_eq!(
        ayanamsa_metadata_coverage_summary,
        validate_render_cli(&["ayanamsa-metadata-coverage-summary"])
            .expect("validation ayanamsa metadata coverage summary should render")
    );
    assert_eq!(
        render_cli(&["ayanamsa-metadata-coverage-summary", "extra"])
            .expect_err("ayanamsa metadata coverage summary should reject extra arguments"),
        "ayanamsa-metadata-coverage-summary does not accept extra arguments"
    );
    let ayanamsa_reference_offsets_summary = render_cli(&["ayanamsa-reference-offsets-summary"])
        .expect("ayanamsa reference offsets summary should render");
    assert_eq!(
        render_cli(&["ayanamsa-reference-offsets"]).unwrap(),
        ayanamsa_reference_offsets_summary
    );
    assert!(ayanamsa_reference_offsets_summary
        .contains("Ayanamsa reference offsets: representative zero-point examples:"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.24535793°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Lahiri (1940): epoch=JD 2415020; offset=22.44597222°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Yukteshwar: epoch=JD 2415020; offset=21.082222°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("True Revati: epoch=JD 1926902.658267; offset=0°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("True Mula: epoch=JD 1805889.671313; offset=0°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("True Pushya: epoch=JD 1855769.248315; offset=0°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Udayagiri: epoch=JD 1825235.164583; offset=0°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Hipparchus: epoch=JD 1674484; offset=-9.333333333333334°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Djwhal Khul: epoch=JD 2415020; offset=26.963097600000026°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Suryasiddhanta (Mean Sun): epoch=JD 1903396.8128654; offset=-0.21463395°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Aryabhata (Mean Sun): epoch=JD 1903396.7895321; offset=-0.23763238°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Babylonian (Kugler 2): epoch=JD 1797039.20682; offset=0°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Babylonian (Kugler 3): epoch=JD 1774637.420172; offset=0°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Babylonian (Huber): epoch=JD 1721171.5; offset=-0.12055555555555555°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Babylonian (Eta Piscium): epoch=JD 1807871.964797; offset=0°"));
    assert!(ayanamsa_reference_offsets_summary
        .contains("Babylonian (Aldebaran): epoch=JD 1801643.133503; offset=0°"));
    assert_eq!(
        ayanamsa_reference_offsets_summary,
        validate_render_cli(&["ayanamsa-reference-offsets-summary"])
            .expect("validation ayanamsa reference offsets summary should render")
    );
    let reference_high_curvature_summary = render_cli(&["reference-high-curvature-summary"])
        .expect("reference high-curvature summary should render");
    assert!(
        reference_high_curvature_summary.contains("Reference major-body high-curvature evidence:")
    );
    assert_eq!(
        reference_high_curvature_summary,
        validate_render_cli(&["reference-high-curvature-summary"])
            .expect("validation high-curvature summary should render")
    );
    assert_eq!(
        reference_high_curvature_summary,
        validate_render_cli(&["high-curvature-summary"])
            .expect("validation high-curvature summary alias should render")
    );
    assert_eq!(
        reference_high_curvature_summary,
        validate_render_cli(&["reference-snapshot-major-body-high-curvature-summary",])
            .expect("validation snapshot high-curvature summary alias should render")
    );
    assert_eq!(
        reference_high_curvature_summary,
        validate_render_cli(&["major-body-high-curvature-summary"])
            .expect("validation major-body high-curvature summary alias should render")
    );
    let reference_major_body_boundary_summary =
        render_cli(&["reference-snapshot-major-body-boundary-summary"])
            .expect("reference major-body boundary summary should render");
    assert!(
        reference_major_body_boundary_summary.contains("Reference major-body boundary evidence:")
    );
    assert_eq!(
        reference_major_body_boundary_summary,
        validate_render_cli(&["reference-snapshot-major-body-boundary-summary"])
            .expect("validation major-body boundary summary should render")
    );
    let major_body_boundary_alias = render_cli(&["major-body-boundary-summary"])
        .expect("major-body boundary alias should render");
    assert_eq!(
        major_body_boundary_alias,
        reference_major_body_boundary_summary
    );
    let reference_mars_jupiter_boundary_summary =
        render_cli(&["reference-snapshot-mars-jupiter-boundary-summary"])
            .expect("reference Mars/Jupiter boundary summary should render");
    assert!(reference_mars_jupiter_boundary_summary
        .contains("Reference Mars/Jupiter boundary evidence:"));
    assert_eq!(
        reference_mars_jupiter_boundary_summary,
        validate_render_cli(&["reference-snapshot-mars-jupiter-boundary-summary"])
            .expect("validation Mars/Jupiter boundary summary should render")
    );
    let mars_jupiter_boundary_alias = render_cli(&["mars-jupiter-boundary-summary"])
        .expect("Mars/Jupiter boundary alias should render");
    assert_eq!(
        mars_jupiter_boundary_alias,
        reference_mars_jupiter_boundary_summary
    );
    let reference_major_body_boundary_window_summary =
        render_cli(&["reference-snapshot-major-body-boundary-window-summary"])
            .expect("reference major-body boundary window summary should render");
    assert!(reference_major_body_boundary_window_summary
        .contains("Reference major-body boundary windows:"));
    assert_eq!(
        reference_major_body_boundary_window_summary,
        validate_render_cli(&["reference-snapshot-major-body-boundary-window-summary"])
            .expect("validation major-body boundary window summary should render")
    );
    let major_body_boundary_window_alias = render_cli(&["major-body-boundary-window-summary"])
        .expect("major-body boundary window alias should render");
    assert_eq!(
        major_body_boundary_window_alias,
        reference_major_body_boundary_window_summary
    );
    let reference_high_curvature_window_summary =
        render_cli(&["reference-high-curvature-window-summary"])
            .expect("reference high-curvature window summary should render");
    assert!(reference_high_curvature_window_summary
        .contains("Reference major-body high-curvature windows:"));
    assert_eq!(
        reference_high_curvature_window_summary,
        validate_render_cli(&["reference-high-curvature-window-summary"])
            .expect("validation high-curvature window summary should render")
    );
    assert_eq!(
        reference_high_curvature_window_summary,
        validate_render_cli(&["high-curvature-window-summary"])
            .expect("validation high-curvature window summary alias should render")
    );
    assert_eq!(
        reference_high_curvature_window_summary,
        validate_render_cli(&["reference-snapshot-major-body-high-curvature-window-summary",])
            .expect("validation snapshot high-curvature window summary alias should render")
    );
    assert_eq!(
        reference_high_curvature_window_summary,
        validate_render_cli(&["major-body-high-curvature-window-summary"])
            .expect("validation major-body high-curvature window summary alias should render")
    );
    let reference_high_curvature_epoch_coverage_summary =
        render_cli(&["reference-high-curvature-epoch-coverage-summary"])
            .expect("reference high-curvature epoch coverage summary should render");
    assert!(reference_high_curvature_epoch_coverage_summary
        .contains("Reference major-body high-curvature epoch coverage:"));
    assert_eq!(
        reference_high_curvature_epoch_coverage_summary,
        validate_render_cli(&["reference-high-curvature-epoch-coverage-summary"])
            .expect("validation high-curvature epoch coverage summary should render")
    );
    assert_eq!(
        reference_high_curvature_epoch_coverage_summary,
        validate_render_cli(&["high-curvature-epoch-coverage-summary"])
            .expect("validation high-curvature epoch coverage summary alias should render")
    );
    assert_eq!(
        reference_high_curvature_epoch_coverage_summary,
        validate_render_cli(&[
            "reference-snapshot-major-body-high-curvature-epoch-coverage-summary",
        ])
        .expect("validation snapshot high-curvature epoch coverage summary alias should render")
    );
    assert_eq!(
        reference_high_curvature_epoch_coverage_summary,
        validate_render_cli(&["major-body-high-curvature-epoch-coverage-summary"]).expect(
            "validation major-body high-curvature epoch coverage summary alias should render"
        )
    );
    let boundary_epoch_coverage_summary =
        render_cli(&["reference-snapshot-boundary-epoch-coverage-summary"])
            .expect("reference snapshot boundary epoch coverage summary should render");
    let boundary_epoch_coverage_alias = render_cli(&["boundary-epoch-coverage-summary"])
        .expect("boundary epoch coverage summary alias should render");
    let boundary_epoch_coverage_reference_alias =
        render_cli(&["reference-snapshot-boundary-epoch-coverage"])
            .expect("reference snapshot boundary epoch coverage alias should render");
    assert!(boundary_epoch_coverage_summary.contains("Reference snapshot boundary epoch coverage:"));
    assert!(boundary_epoch_coverage_summary.contains(
        "JD 2451915.5 (TDB): 16 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
    ));
    assert_eq!(
        boundary_epoch_coverage_alias,
        boundary_epoch_coverage_summary
    );
    assert_eq!(
        boundary_epoch_coverage_reference_alias,
        boundary_epoch_coverage_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-epoch-coverage", "extra"]).expect_err(
            "reference snapshot boundary epoch coverage alias should reject extra arguments"
        ),
        "reference-snapshot-boundary-epoch-coverage-summary does not accept extra arguments"
    );
    assert_eq!(
        boundary_epoch_coverage_summary,
        validate_render_cli(&["reference-snapshot-boundary-epoch-coverage-summary"])
            .expect("validation boundary epoch coverage summary should render")
    );
    let sparse_boundary_summary = render_cli(&["reference-snapshot-sparse-boundary-summary"])
        .expect("reference snapshot sparse boundary summary should render");
    let sparse_boundary_alias = render_cli(&["sparse-boundary-summary"])
        .expect("sparse boundary summary alias should render");
    let boundary_day_alias =
        render_cli(&["boundary-day-summary"]).expect("boundary day summary alias should render");
    let boundary_day_reference_alias = render_cli(&["reference-snapshot-boundary-day-summary"])
        .expect("reference snapshot boundary day summary should render");
    let boundary_day_short_reference_alias = render_cli(&["reference-snapshot-boundary-day"])
        .expect("reference snapshot boundary day alias should render");
    assert!(sparse_boundary_summary.contains("Reference snapshot boundary day:"));
    assert!(sparse_boundary_summary.contains(
        "JD 2451915.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
    ));
    assert_eq!(sparse_boundary_alias, sparse_boundary_summary);
    assert_eq!(boundary_day_alias, sparse_boundary_summary);
    assert_eq!(boundary_day_reference_alias, sparse_boundary_summary);
    assert_eq!(boundary_day_short_reference_alias, sparse_boundary_summary);
    assert_eq!(
        sparse_boundary_summary,
        validate_render_cli(&["reference-snapshot-sparse-boundary-summary"])
            .expect("validation sparse boundary summary should render")
    );
    assert_eq!(
        boundary_day_alias,
        validate_render_cli(&["boundary-day-summary"])
            .expect("validation boundary day summary should render")
    );
    assert_eq!(
        boundary_day_reference_alias,
        validate_render_cli(&["reference-snapshot-boundary-day-summary"])
            .expect("validation reference snapshot boundary day summary should render")
    );
    assert_eq!(
        boundary_day_short_reference_alias,
        validate_render_cli(&["reference-snapshot-boundary-day"])
            .expect("validation reference snapshot boundary day alias should render")
    );
    assert_eq!(
        render_cli(&["boundary-day-summary", "extra"])
            .expect_err("boundary day summary alias should reject extra arguments"),
        "reference-snapshot-boundary-day-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-day-summary", "extra"])
            .expect_err("reference snapshot boundary day summary should reject extra arguments"),
        "reference-snapshot-boundary-day-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-day", "extra"])
            .expect_err("reference snapshot boundary day alias should reject extra arguments"),
        "reference-snapshot-boundary-day-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-sparse-boundary-summary", "extra"])
            .expect_err("reference snapshot sparse boundary summary should reject extra arguments"),
        "reference-snapshot-sparse-boundary-summary does not accept extra arguments"
    );
    let pre_bridge_boundary_summary =
        render_cli(&["reference-snapshot-pre-bridge-boundary-summary"])
            .expect("reference snapshot pre-bridge boundary summary should render");
    let pre_bridge_boundary_alias = render_cli(&["pre-bridge-boundary-summary"])
        .expect("pre-bridge boundary summary alias should render");
    let pre_bridge_boundary_reference_alias =
        render_cli(&["reference-snapshot-pre-bridge-boundary"])
            .expect("reference snapshot pre-bridge boundary alias should render");
    assert!(pre_bridge_boundary_summary.contains("Reference snapshot pre-bridge boundary day:"));
    assert!(pre_bridge_boundary_summary.contains(
        "JD 2451914.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); pre-bridge boundary day"
    ));
    assert_eq!(pre_bridge_boundary_alias, pre_bridge_boundary_summary);
    assert_eq!(
        pre_bridge_boundary_reference_alias,
        pre_bridge_boundary_summary
    );
    assert_eq!(
        render_cli(&["reference-snapshot-pre-bridge-boundary", "extra"]).expect_err(
            "reference snapshot pre-bridge boundary alias should reject extra arguments"
        ),
        "reference-snapshot-pre-bridge-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        pre_bridge_boundary_summary,
        validate_render_cli(&["reference-snapshot-pre-bridge-boundary-summary"])
            .expect("validation pre-bridge boundary summary should render")
    );
    assert_eq!(
        render_cli(&["reference-snapshot-pre-bridge-boundary-summary", "extra"]).expect_err(
            "reference snapshot pre-bridge boundary summary should reject extra arguments"
        ),
        "reference-snapshot-pre-bridge-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["pre-bridge-boundary-summary", "extra"])
            .expect_err("pre-bridge boundary summary alias should reject extra arguments"),
        "pre-bridge-boundary-summary does not accept extra arguments"
    );
    let dense_boundary_summary = render_cli(&["reference-snapshot-dense-boundary-summary"])
        .expect("reference snapshot dense boundary summary should render");
    let dense_boundary_alias = render_cli(&["dense-boundary-summary"])
        .expect("dense boundary summary alias should render");
    assert!(dense_boundary_summary.contains("Reference snapshot dense boundary day:"));
    assert!(dense_boundary_summary.contains(
        "JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
    ));
    assert_eq!(dense_boundary_alias, dense_boundary_summary);
    assert_eq!(
        dense_boundary_summary,
        validate_render_cli(&["reference-snapshot-dense-boundary-summary"])
            .expect("validation dense boundary summary should render")
    );
    assert_eq!(
        render_cli(&["reference-snapshot-dense-boundary-summary", "extra"])
            .expect_err("reference snapshot dense boundary summary should reject extra arguments"),
        "reference-snapshot-dense-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["dense-boundary-summary", "extra"])
            .expect_err("dense boundary summary alias should reject extra arguments"),
        "dense-boundary-summary does not accept extra arguments"
    );
    let source_documentation_summary = render_cli(&["source-documentation-summary"])
        .expect("source documentation summary should render");
    assert!(source_documentation_summary.contains("VSOP87 source documentation:"));
    assert_eq!(
        source_documentation_summary,
        validate_render_cli(&["source-documentation-summary"])
            .expect("validation source documentation summary should render")
    );
    let source_documentation_alias =
        render_cli(&["source-documentation"]).expect("source documentation alias should render");
    assert_eq!(
        source_documentation_alias,
        validate_render_cli(&["source-documentation"])
            .expect("validation source documentation alias should render")
    );
    assert_eq!(
        render_cli(&["source-documentation", "extra"])
            .expect_err("source documentation alias should reject extra arguments"),
        "source-documentation does not accept extra arguments"
    );
    let source_documentation_health_summary = render_cli(&["source-documentation-health-summary"])
        .expect("source documentation health summary should render");
    assert!(source_documentation_health_summary.contains("VSOP87 source documentation health:"));
    assert_eq!(
        source_documentation_health_summary,
        validate_render_cli(&["source-documentation-health-summary"])
            .expect("validation source documentation health summary should render")
    );
    let source_documentation_health_alias = render_cli(&["source-documentation-health"])
        .expect("source documentation health alias should render");
    assert_eq!(
        source_documentation_health_alias,
        source_documentation_health_summary
    );
    assert_eq!(
        render_cli(&["source-documentation-health", "extra"])
            .expect_err("source documentation health alias should reject extra arguments"),
        "source-documentation-health does not accept extra arguments"
    );
    let source_audit_summary =
        render_cli(&["source-audit-summary"]).expect("source audit summary should render");
    assert!(source_audit_summary.contains("VSOP87 source audit:"));
    assert_eq!(
        source_audit_summary,
        validate_render_cli(&["source-audit-summary"])
            .expect("validation source audit summary should render")
    );
    let source_audit_alias =
        render_cli(&["source-audit"]).expect("source audit alias should render");
    assert_eq!(source_audit_alias, source_audit_summary);
    assert_eq!(
        render_cli(&["source-audit", "extra"])
            .expect_err("source audit alias should reject extra arguments"),
        "source-audit does not accept extra arguments"
    );
    let generated_binary_audit_summary = render_cli(&["generated-binary-audit-summary"])
        .expect("generated binary audit summary should render");
    assert!(generated_binary_audit_summary.contains("VSOP87 generated binary audit:"));
    assert_eq!(
        generated_binary_audit_summary,
        validate_render_cli(&["generated-binary-audit-summary"])
            .expect("validation generated binary audit summary should render")
    );
    let generated_binary_audit_alias = render_cli(&["generated-binary-audit"])
        .expect("generated binary audit alias should render");
    assert_eq!(generated_binary_audit_alias, generated_binary_audit_summary);
    assert_eq!(
        render_cli(&["generated-binary-audit", "extra"])
            .expect_err("generated binary audit alias should reject extra arguments"),
        "generated-binary-audit does not accept extra arguments"
    );
    assert!(release_summary.lines().any(|line| {
        line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
    }));
    assert!(release_summary
        .contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary"));
    assert!(release_summary
        .lines()
        .any(|line| line == packaged_artifact_access_report_line()));
    assert!(release_summary.lines().any(|line| {
        line == format!(
            "Packaged-artifact generation policy: {}",
            pleiades_data::packaged_artifact_generation_policy_summary_for_report()
        )
    }));
    assert!(release_summary.lines().any(|line| {
        line == format!(
            "Packaged frame treatment: {}",
            pleiades_data::packaged_frame_treatment_summary_for_report()
        )
    }));
    assert!(release_summary.contains(
        "Packaged lookup epoch policy: TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
    ));
    assert!(release_summary.lines().any(|line| {
        line == format!(
            "Packaged batch parity: {}",
            pleiades_data::packaged_mixed_tt_tdb_batch_parity_summary_for_report()
        )
    }));
    assert!(release_summary.contains(
        "Packaged batch parity: Packaged mixed TT/TDB batch parity: 11 requests across 11 bodies, TT requests=6, TDB requests=5; quality counts: Exact=0, Interpolated=11, Approximate=0, Unknown=0; order=preserved, single-query parity=preserved"
    ));
    assert!(release_summary.contains("Lunar high-curvature equatorial continuity evidence"));
    assert!(release_summary.contains("Artifact inspection:"));
    assert!(release_summary.contains("Release gate reminders:"));
    assert!(
        release_summary.contains("Compatibility profile summary: compatibility-profile-summary")
    );
    assert!(release_summary
        .lines()
        .any(|line| line == "Release notes summary: release-notes-summary"));
    assert!(release_summary
        .contains("lunar source selection: Compact Meeus-style truncated lunar baseline"));
    assert!(release_summary.contains("Wang"));
    assert!(release_summary.contains("Aries houses"));
    assert!(release_summary.contains("Fagan/Bradley"));
    assert!(release_summary.contains("Usha Shashi"));
    assert!(release_summary.contains("Galactic Center (Mula/Wilhelm)"));
    assert!(release_summary.contains("Mula Wilhelm"));
    assert!(release_summary.contains("Wilhelm"));
    assert!(release_summary.contains("Galactic Equator (Fiorenza)"));
    assert!(release_summary.contains("coverage=Luminaries: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence, bodies=2 (Sun, Moon), samples="));
    assert!(release_summary.contains("JPL interpolation posture: source="));
    assert!(release_summary.lines().any(|line| {
        line == "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
    }));
    assert!(release_summary.lines().any(|line| {
        line == "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
    }));
    assert!(release_summary.contains(
        "Validation report summary: validation-report-summary / validation-summary / report-summary"
    ));
    assert!(release_summary.contains("Artifact validation: validate-artifact"));
    assert!(release_summary.contains("Release bundle verification: verify-release-bundle"));
    assert!(release_summary
        .lines()
        .any(|line| line == "Workspace audit: workspace-audit / audit"));
    assert!(release_summary
        .contains("[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile"));
    assert!(release_summary.lines().any(|line| {
        line == "Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"
    }));
    assert!(release_summary.contains("Release checklist summary: release-checklist-summary"));
    assert!(release_summary.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));
    assert!(release_summary.contains("See release-notes and release-checklist"));
}
