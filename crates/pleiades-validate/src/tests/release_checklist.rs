//! release checklist, gate, smoke, summary, and notes tests (white-box; moved verbatim from the former `tests.rs`).

use super::test_support::*;
use super::*;
use pleiades_core::current_release_profile_identifiers;

#[test]
fn release_notes_command_renders_the_release_notes() {
    let rendered = render_cli(&["release-notes"]).expect("release notes should render");
    assert!(rendered.contains("Release notes"));
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(rendered.contains("API stability posture:"));
    assert!(rendered.contains("Deprecation policy:"));
    assert!(rendered.contains("Release-specific coverage:"));
    assert!(rendered.contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(rendered.contains("selected asteroid coverage"));
    assert!(rendered.contains("WvA"));
    assert!(rendered.contains("Selected asteroid evidence: 6 exact J2000 samples"));
    assert!(rendered.contains("Selected asteroid batch parity: 6 requests across 6 bodies at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); frame mix: 3 ecliptic, 3 equatorial; batch/single parity preserved"));
    assert!(rendered.contains("Reference snapshot coverage: 277 rows across 16 bodies and 23 epochs (95 asteroid rows; JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies:"));
    assert!(rendered.contains("Reference snapshot body-class coverage: major bodies: 182 rows across 10 bodies and 20 epochs; major windows: "));
    assert!(rendered.contains(&reference_snapshot_pre_bridge_boundary_summary_for_report()));
    assert!(
        rendered.contains(&reference_snapshot_2451914_major_body_pre_bridge_summary_for_report())
    );
    assert!(rendered.contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_dense_boundary_summary_for_report()));
    assert!(rendered
        .contains("selected asteroids: 95 rows across 6 bodies and 17 epochs; asteroid windows: "));
    assert!(rendered.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_high_curvature_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_source_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_manifest_summary_for_report()));
    assert!(
        rendered.contains("Comparison snapshot coverage: 162 rows across 10 bodies and 18 epochs")
    );
    assert!(rendered.contains("asteroid:433-Eros"));
    assert!(rendered.contains("Validation reference points:"));
    assert!(rendered.contains("Compatibility caveats:"));
    assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
    assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
    assert!(rendered.contains("Pullen SD (Neo-Porphyry) table of houses"));
    assert!(rendered.contains("Neo-Porphyry"));
    assert!(rendered.contains("Makransky Sunshine"));
    assert!(rendered.contains("Babylonian Huber"));
    assert!(rendered.contains("Babylonian (Britton)"));
    assert!(rendered.contains("Babylonian (Aldebaran)"));
    assert!(rendered.contains("Babylonian (Eta Piscium)"));
    assert!(rendered.contains("Babylonian (True Geoc)"));
    assert!(rendered.contains("Babylonian (True Topc)"));
    assert!(rendered.contains("Babylonian (True Obs)"));
    assert!(rendered.contains("Babylonian (House Obs)"));
    assert!(rendered.contains("Equal MC"));
    assert!(rendered.contains("Equal/MC house system"));
    assert!(rendered.contains("Equal Midheaven"));
    assert!(rendered.contains("Equal Midheaven house system"));
    assert!(rendered.contains("Babylonian (Kugler 1)"));
    assert!(rendered.contains("Krusinski/Pisa/Goelzer"));
    assert!(rendered.contains("Equal/MC = 10th"));
    assert!(rendered.contains("Galactic Equator (True)"));
    assert!(rendered.contains("Galactic Equator (IAU 1958)"));
    assert!(rendered.contains("Valens Moon ayanamsa"));
}

#[test]
fn release_notes_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["release-notes-summary"]).expect("release notes summary should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains("Release notes summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Release-specific coverage:"));
    assert!(rendered.contains("Selected asteroid source evidence: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; bodies: Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"));
    assert!(rendered.contains("Selected asteroid source windows: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: Ceres: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Pallas: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Juno: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Vesta: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 10 samples across 10 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(rendered.contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(rendered.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451911_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451915_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451915_major_body_bridge_summary_for_report()));
    assert!(
        rendered.contains(&reference_snapshot_2451914_major_body_pre_bridge_summary_for_report())
    );
    assert!(rendered.contains(&reference_snapshot_2451914_bridge_day_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
    assert!(rendered
        .contains(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451917_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451917_major_body_bridge_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report()));
    assert!(rendered.contains("Custom-definition labels:"));
    assert!(rendered.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
    assert!(rendered.contains(&format!(
        "House code aliases: {}",
        profile.house_code_aliases_summary_line()
    )));
    assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
    assert!(rendered.contains("Compatibility caveats:"));
    assert!(rendered.contains(&format!(
        "Custom-definition labels: {}",
        profile.custom_definition_labels.len()
    )));
    assert!(rendered.contains(&format!(
        "Custom-definition label names: {}",
        profile.custom_definition_labels.join(", ")
    )));
    assert!(rendered.contains(
        profile
            .validated_target_house_scope_summary_line()
            .expect("target house scope summary should validate")
            .as_str()
    ));
    assert!(rendered.contains(
        profile
            .validated_target_ayanamsa_scope_summary_line()
            .expect("target ayanamsa scope summary should validate")
            .as_str()
    ));
    assert!(rendered.contains(&format!(
        "Compatibility caveats: {}",
        profile.known_gaps.len()
    )));
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_house_scope.join("; ")));
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_ayanamsa_scope.join("; ")));
    assert!(rendered.contains("API stability summary line: API stability posture: pleiades-api-stability/0.2.0; stable surfaces: 6; experimental surfaces: 3; deprecation policy items: 4; intentional limits: 3"));
    assert!(rendered.contains(&reference_snapshot_source_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_boundary_epoch_coverage_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_major_body_boundary_window_summary_for_report()));
    assert!(rendered.contains("Reference snapshot body-class coverage: major bodies: 182 rows across 10 bodies and 20 epochs; major windows: "));
    assert!(rendered
        .contains("selected asteroids: 95 rows across 6 bodies and 17 epochs; asteroid windows: "));
    assert!(rendered.contains(&pleiades_jpl::comparison_snapshot_source_summary_for_report()));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains(&format!(
        "Packaged-artifact access: {}",
        format_packaged_artifact_access_summary()
    )));
    assert!(rendered.contains("Packaged request policy:"));
    assert!(rendered.contains(&format!(
        "Packaged lookup epoch policy: {}",
        packaged_lookup_epoch_policy_summary_for_report()
    )));
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Packaged batch parity: {}",
            packaged_mixed_tt_tdb_batch_parity_summary_for_report()
        )
    }));
    assert!(rendered.contains("Packaged batch parity:"));
    assert!(rendered.contains("Release notes: release-notes"));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert_report_contains_exact_line(
        &rendered,
        "Workspace audit summary: workspace-audit-summary",
    );
    assert!(rendered.contains(profile.target_house_scope.join("; ").as_str()));
    assert!(rendered.contains(profile.target_ayanamsa_scope.join("; ").as_str()));
    assert!(rendered.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Reference snapshot coverage: 277 rows across 16 bodies and 23 epochs (95 asteroid rows; JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies:"));
    assert!(
        rendered.contains("Comparison snapshot coverage: 162 rows across 10 bodies and 18 epochs")
    );
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Artifact boundary envelope:"));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains("See release-notes for the full maintainer-facing artifact."));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));
}

#[test]
fn release_checklist_summary_helper_reports_expected_posture() {
    let summary = release_checklist_summary();

    assert_eq!(
        summary.release_profile_identifiers,
        current_release_profile_identifiers()
    );
    assert_eq!(
        summary.repository_managed_release_gates,
        release_checklist_repository_managed_release_gates().len()
    );
    assert_eq!(
        summary.manual_bundle_workflow_items,
        release_checklist_manual_bundle_workflow().len()
    );
    assert_eq!(
        summary.bundle_contents_items,
        release_checklist_bundle_contents().len()
    );
    assert_eq!(
        summary.external_publishing_reminders,
        release_checklist_external_publishing_reminders().len()
    );
    assert!(summary.validate().is_ok());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary.summary_line().contains("v1 compatibility="));
}

#[test]
fn release_checklist_command_renders_the_release_checklist() {
    let rendered = render_cli(&["release-checklist"]).expect("release checklist should render");
    assert!(rendered.contains("Release checklist"));
    let release_profiles = current_release_profile_identifiers();
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(rendered.contains("API stability summary: api-stability-summary"));
    assert!(rendered
            .lines()
            .any(|line| line == "Validation report summary: validation-report-summary / validation-summary / report-summary"));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
    assert!(rendered.contains("Repository-managed release gates:"));
    assert!(rendered.contains("[x] cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release"));
    assert!(rendered.contains("[x] cargo run -q -p pleiades-validate -- benchmark --rounds 5"));
    assert!(rendered.contains("[x] cargo run -q -p pleiades-validate -- report --rounds 5"));
    assert!(rendered.contains("Manual bundle workflow:"));
    assert!(rendered.contains("Bundle contents:"));
    assert!(rendered.contains("backend-matrix-summary.txt"));
    assert!(rendered.contains("api-stability-summary.txt"));
    assert!(rendered.contains("release-checklist-summary.txt"));
}

#[test]
fn release_checklist_summary_command_renders_the_summary() {
    let rendered = render_cli(&["release-checklist-summary"])
        .expect("release checklist summary should render");
    let release_profiles = current_release_profile_identifiers();
    assert!(rendered.contains("Release checklist summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains("Backend matrix summary: backend-matrix-summary"));
    assert!(rendered.contains("API stability summary: api-stability-summary"));
    assert!(rendered.contains("Zodiac policy:"));
    assert!(rendered
            .lines()
            .any(|line| line == "Validation report summary: validation-report-summary / validation-summary / report-summary"));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Compatibility profile verification: verify-compatibility-profile"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("Release summary: release-summary"));
    assert!(rendered.contains("Compact summary views: release-notes-summary, api-stability-summary, backend-matrix-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary"));
    assert!(rendered.contains("Repository-managed release gates: 10 items"));
    assert!(rendered.contains("Manual bundle workflow: 4 items"));
    assert!(rendered.contains("Bundle contents: 25 items"));
    assert!(rendered.contains("External publishing reminders: 3 items"));
    assert!(rendered.contains("See release-checklist for the full maintainer-facing artifact."));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));
}

#[test]
fn release_gate_command_aliases_the_release_checklist() {
    let checklist = render_cli(&["release-checklist"]).expect("release checklist should render");
    let gate = render_cli(&["release-gate"]).expect("release gate should render");
    let checklist_summary = render_cli(&["release-checklist-summary"])
        .expect("release checklist summary should render");
    let gate_summary =
        render_cli(&["release-gate-summary"]).expect("release gate summary should render");

    assert_eq!(gate, checklist);
    assert_eq!(gate_summary, checklist_summary);
    assert!(render_cli(&["release-gate", "extra"]).is_err());
    assert!(render_cli(&["release-gate-summary", "extra"]).is_err());
}

#[test]
fn release_smoke_command_renders_the_smoke_report() {
    let rendered = render_cli(&["release-smoke"]).expect("release smoke should render");

    assert!(rendered.contains("Release smoke"));
    assert!(rendered.contains("workspace audit: ok"));
    assert!(rendered.contains("compatibility profile verification: ok"));
    assert!(rendered.contains("artifact validation: ok"));
    assert!(rendered.contains("release bundle generation: ok"));
    assert!(rendered.contains("release bundle verification: ok"));
    assert!(render_cli(&["release-smoke", "extra"]).is_err());
}

#[test]
fn release_gate_checks_reject_non_directory_output_paths() {
    let output_path = unique_temp_dir("pleiades-release-gate-file").with_extension("txt");
    std::fs::write(&output_path, "not a directory").expect("temporary file should be creatable");

    let error = validate_release_gate_at(&output_path)
        .expect_err("release gate checks should reject file-backed output paths");

    assert!(error.contains("release gate"));
}

#[test]
fn release_summary_command_renders_the_quick_overview() {
    let rendered = render_cli(&["release-summary"]).expect("release summary should render");
    let release_profiles = current_release_profile_identifiers();
    let profile = current_compatibility_profile();
    assert!(rendered.contains("Release summary"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(rendered.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert_report_contains_exact_line(
        &rendered,
        &format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        ),
    );
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_house_scope.join("; ")));
    assert!(rendered
        .lines()
        .any(|line| line == profile.target_ayanamsa_scope.join("; ")));
    assert!(rendered.lines().any(|line| line == "Release summary"));
    assert!(rendered.contains("Comparison body-class tolerance: body-class tolerance posture:"));
    assert!(rendered.contains("Comparison body-class error envelopes:"));
    assert!(rendered.contains("Source corpus: comparison corpus release-grade guard:"));
    assert!(rendered.contains("Source corpus posture: comparison corpus release-grade guard:"));
    assert!(rendered.contains("Catalog posture: house systems="));
    assert_report_contains_exact_line(
        &rendered,
        &format!("Known gaps: {}", profile.known_gaps_summary_line()),
    );
    assert!(rendered.contains("Pluto fallback: "));
    assert!(rendered.contains("JPL source corpus contract:"));
    assert!(rendered.contains("phase-2 corpus alignment:"));
    assert!(rendered.contains("Release summary line:"));
    assert!(rendered.contains("Production generation body-class coverage:"));
    assert!(rendered.contains("Production generation corpus shape:"));
    assert!(rendered.contains(&format!(
        "Packaged lookup epoch policy: {}",
        packaged_lookup_epoch_policy_summary_for_report()
    )));
    assert!(rendered
        .lines()
        .any(|line| line == "Backend matrix summary: backend-matrix-summary"));
    assert_report_contains_exact_line(&rendered, &profile.catalog_inventory_summary_line());
    assert!(rendered.contains(&format!(
        "house latitude-sensitive constraints={}",
        profile.latitude_sensitive_house_constraints_summary_line()
    )));
    assert_report_contains_exact_line(
        &rendered,
        &format!(
            "House code aliases: {}",
            profile.house_code_aliases_summary_line()
        ),
    );
    assert!(rendered.contains("House formula families: 7 (Equal, Equatorial projection, Great-circle, Quadrant, Sector, Solar arc, Whole Sign)"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains(&reference_snapshot_2451914_bridge_day_summary_for_report()));
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Packaged batch parity: {}",
            packaged_mixed_tt_tdb_batch_parity_summary_for_report()
        )
    }));
    assert!(rendered
        .lines()
        .any(|line| line == "Backend matrix summary: backend-matrix-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert_report_contains_exact_line(
        &rendered,
        "Workspace audit summary: workspace-audit-summary",
    );
    assert_report_contains_exact_line(
        &rendered,
        "Release checklist summary: release-checklist-summary",
    );
    assert!(rendered.contains("Workspace audit: workspace-audit / audit"));
    assert!(rendered.contains("Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"));
    assert!(rendered.contains("Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"));
    assert!(rendered.contains("Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"));
    assert!(rendered.contains("Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"));
    assert!(rendered.contains("Native sidereal policy: native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    assert!(rendered.contains("Frame policy: ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    assert_eq!(
        render_cli(&["time-scale-policy"]).expect("time-scale policy alias should render"),
        render_time_scale_policy_summary_text()
    );
    assert_eq!(
        render_cli(&["delta-t-policy"]).expect("delta T policy alias should render"),
        render_delta_t_policy_summary_text()
    );
    assert_eq!(
        render_cli(&["observer-policy"]).expect("observer policy alias should render"),
        render_observer_policy_summary_text()
    );
    assert_eq!(
        render_cli(&["apparentness-policy"]).expect("apparentness policy alias should render"),
        render_apparentness_policy_summary_text()
    );
    assert_eq!(
        render_cli(&["frame-policy"]).expect("frame policy alias should render"),
        render_frame_policy_summary_text()
    );

    for (args, expected) in [
        (
            ["time-scale-policy-summary", "extra"],
            "time-scale-policy-summary does not accept extra arguments",
        ),
        (
            ["time-scale-policy", "extra"],
            "time-scale-policy does not accept extra arguments",
        ),
        (
            ["utc-convenience-policy-summary", "extra"],
            "utc-convenience-policy-summary does not accept extra arguments",
        ),
        (
            ["utc-convenience-policy", "extra"],
            "utc-convenience-policy does not accept extra arguments",
        ),
        (
            ["delta-t-policy-summary", "extra"],
            "delta-t-policy-summary does not accept extra arguments",
        ),
        (
            ["delta-t-policy", "extra"],
            "delta-t-policy does not accept extra arguments",
        ),
        (
            ["observer-policy-summary", "extra"],
            "observer-policy-summary does not accept extra arguments",
        ),
        (
            ["observer-policy", "extra"],
            "observer-policy does not accept extra arguments",
        ),
        (
            ["apparentness-policy-summary", "extra"],
            "apparentness-policy-summary does not accept extra arguments",
        ),
        (
            ["apparentness-policy", "extra"],
            "apparentness-policy does not accept extra arguments",
        ),
        (
            ["frame-policy-summary", "extra"],
            "frame-policy-summary does not accept extra arguments",
        ),
        (
            ["frame-policy", "extra"],
            "frame-policy does not accept extra arguments",
        ),
    ] {
        assert_eq!(
            render_cli(&args).expect_err("policy summary should reject extra arguments"),
            expected
        );
    }

    assert!(rendered.contains(&request_surface_summary_for_report()));
    let mean_obliquity_frame_round_trip = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Mean-obliquity frame round-trip: {}",
            mean_obliquity_frame_round_trip
        )
    }));
    assert!(rendered.contains("Zodiac policy: tropical only"));
    assert!(rendered.contains("ayanamsa catalog validation: ok"));
    assert!(rendered.contains("House systems:"));
    assert!(rendered.contains("House systems: 25 total (12 baseline, 13 release-specific)"));
    assert!(rendered.contains(&format!(
        "House-code aliases: {}",
        profile.house_code_alias_count()
    )));
    assert!(rendered.contains("Release-specific house-system canonical names: 13 (Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, Gauquelin sectors)"));
    assert!(rendered.contains("Wang"));
    assert!(rendered.contains("Aries houses"));
    assert!(rendered.contains("Ayanamsa reference offsets: representative zero-point examples:"));
    assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
    assert!(rendered.contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
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
    assert!(rendered.contains("Babylonian (Britton): epoch=JD 1805415.712776; offset=0°"));
    assert!(rendered.contains("Babylonian (Kugler 2): epoch=JD 1797039.20682; offset=0°"));
    assert!(rendered.contains("Babylonian (Kugler 3): epoch=JD 1774637.420172; offset=0°"));
    assert!(rendered.contains("Babylonian (Eta Piscium): epoch=JD 1807871.964797; offset=0°"));
    assert!(rendered.contains("Babylonian (Aldebaran): epoch=JD 1801643.133503; offset=0°"));
    assert!(rendered.contains("Aryabhata (499 CE): epoch=JD 1903396.7895320603; offset=0°"));
    assert!(rendered.contains("Sassanian: epoch=JD 1927135.8747793; offset=0°"));
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
    assert!(rendered.contains("Suryasiddhanta (Mean Sun): epoch=JD 1909045.584433; offset=0°"));
    assert!(rendered.contains("Aryabhata (Mean Sun): epoch=JD 1909650.815331; offset=0°"));
    assert!(rendered.contains("Release-specific ayanamsa canonical names: 54 (True Citra, J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), Galactic Center, Galactic Equator, True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Valens Moon)"));
    assert!(rendered.contains("Ayanamsa reference offsets: representative zero-point examples:"));
    assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("Galactic Equator (IAU 1958): epoch=JD 1667118.376332; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
    assert!(rendered.contains("Ayanamsa provenance: representative provenance examples:"));
    assert!(rendered.contains("True Citra — True Citra sidereal mode with the published zero point used by Swiss Ephemeris-style interoperability tables."));
    assert!(rendered.contains("True Revati — True-nakshatra mode with the Revati reference point fixed to the Swiss Ephemeris zero date."));
    assert!(rendered.contains("True Mula — True-nakshatra mode with the Mula reference point fixed to the Swiss Ephemeris zero date."));
    assert!(rendered.contains("True Pushya — True-nakshatra Pushya reference mode exposed by Swiss Ephemeris and anchored to the published zero date."));
    assert!(rendered.contains("Udayagiri — Udayagiri sidereal mode treated as the Lahiri/Chitrapaksha/Chitra Paksha 285 CE reference family in the Swiss Ephemeris interoperability catalog."));
    assert!(rendered.contains("True Sheoran — True-nakshatra Sheoran reference mode with the Swiss Ephemeris zero point at JD 1789947.090881 (+0188/08/09 14:10:52.11 UT)."));
    assert!(rendered.contains("Galactic Center (Rgilbrand) — Galactic-center reference mode attributed to Rgilbrand, with the Swiss Ephemeris zero point at JD 1861740.329525 (+0385/03/03 19:54:30.99 UT)."));
    assert!(rendered.contains("Babylonian (Kugler 1) — Babylonian sidereal mode associated with Kugler's first reconstruction, with the Swiss Ephemeris zero point at JD 1833923.577692 (+0309/01/05 01:51:52.62 UT)."));
    assert!(rendered.contains("Valens Moon — Valens Moon sidereal mode, catalogued with the Swiss Ephemeris reference epoch and offset from the header metadata."));
    assert!(rendered.contains("JN Bhasin"));
    assert!(rendered.contains("Validation reference points: 1 (stage-4 validation corpus)"));
    assert!(rendered.contains("Custom-definition labels: 9"));
    assert!(rendered.contains("Custom-definition label names: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Balarama, Aphoric, Takra"));
    assert!(rendered.contains("Custom-definition ayanamsas:"));
    assert!(rendered.contains("Compatibility caveats: 2"));
    assert!(rendered.contains(&format!(
        "Ayanamsas: {} total ({} baseline, {} release-specific)",
        profile.ayanamsas.len(),
        profile.baseline_ayanamsas.len(),
        profile.release_ayanamsas.len()
    )));
    assert!(rendered.contains("Comparison envelope:"));
    assert!(rendered.contains("median longitude delta:"));
    assert!(rendered.contains("95th percentile longitude delta:"));
    assert!(rendered.contains("median latitude delta:"));
    assert!(rendered.contains("95th percentile latitude delta:"));
    assert!(
        rendered.contains("Comparison snapshot coverage: 162 rows across 10 bodies and 18 epochs")
    );
    assert!(rendered.contains("Body-class error envelopes:"));
    assert!(rendered.contains("max Δlon="));
    assert!(rendered.contains("median Δlon="));
    assert!(rendered.contains("rms Δlat="));
    assert!(rendered.contains("max longitude delta:"));
    assert!(rendered.contains("median longitude delta:"));
    assert!(rendered.contains("rms longitude delta:"));
    assert!(rendered.contains("max latitude delta:"));
    assert!(rendered.contains("median latitude delta:"));
    assert!(rendered.contains("95th percentile latitude delta:"));
    assert!(rendered.contains("rms latitude delta:"));
    assert!(rendered.contains("Validation evidence:"));
    assert!(rendered.contains("House validation corpus: 9 scenarios (Mid-latitude reference chart, Western hemisphere reference chart, Equatorial reference chart, Polar stress chart, Northern high-latitude stress chart, Northern high-latitude mountain stress chart, Southern high-latitude mountain stress chart, Southern polar stress chart, Southern hemisphere reference chart), 108 samples, 108 successes, 0 failures; hemisphere coverage: north=5, south=3, equatorial=1; longitude coverage: prime-meridian=2, non-prime-meridian=7; formula families: Equal, Whole Sign, Quadrant, Equatorial projection; latitude-sensitive systems: Koch, Placidus, Topocentric; constraints: Koch [Quadrant system with documented high-latitude pathologies.], Placidus [Quadrant system; can fail or become unstable at extreme latitudes.], Topocentric [Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.]"));
    assert!(rendered.contains("comparison samples"));
    assert!(rendered.contains("Time-scale policy:"));
    assert!(rendered.contains("Observer policy:"));
    assert!(rendered.contains("Apparentness policy:"));
    assert!(rendered.contains("Native sidereal policy:"));
    assert!(rendered.contains("Zodiac policy:"));
    assert!(rendered.contains("notable regressions"));
    assert!(rendered.contains("outside-tolerance bodies"));
    assert!(rendered.contains("Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="));
    assert!(rendered.contains("coverage=Luminaries: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence, bodies=2 (Sun, Moon), samples="));
    assert!(rendered.contains("window=JD 2415020.5 (TT) → JD 2453000.5 (TT)"));
    assert!(rendered.contains("frames=Ecliptic"));
    assert!(rendered.contains("Luminaries: Δlon≤7.500°, Δlat≤0.750°, Δdist=0.001 AU"));
    assert!(rendered.contains("Major planets: Δlon≤0.010°, Δlat≤0.010°, Δdist=0.001 AU"));
    assert!(rendered
        .contains("Pluto fallback (approximate): Δlon≤45.000°, Δlat≤1.000°, Δdist=0.250 AU"));
    assert!(rendered.contains("evidence=9 bodies"));
    assert!(rendered.contains("Body-class tolerance posture:"));
    assert!(rendered.contains("Expected tolerance status:"));
    assert!(rendered.contains("Comparison audit: status=clean, bodies checked=9"));
    assert!(rendered.contains("JPL interpolation evidence:"));
    assert!(rendered.contains("Reference/hold-out overlap:"));
    assert!(rendered.contains("JPL independent hold-out:"));
    assert!(rendered.contains("JPL independent hold-out equatorial parity:"));
    assert!(rendered.contains("JPL independent hold-out batch parity:"));
    assert_report_contains_exact_line(
            &rendered,
            "JPL request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false",
        );
    assert_report_contains_exact_line(
            &rendered,
            "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant",
        );
    assert!(rendered.contains("JPL frame treatment: checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform"));
    assert!(rendered.contains("Reference snapshot coverage:"));
    assert!(rendered.contains("Selected asteroid evidence:"));
    assert!(rendered.contains("Selected asteroid batch parity:"));
    assert!(rendered.contains("VSOP87 evidence:"));
    assert!(rendered.contains("VSOP87 source-backed body-class envelopes:"));
    assert!(rendered.contains("VSOP87 canonical J2000 equatorial body-class envelopes:"));
    assert!(rendered.contains("Luminary: samples=1, bodies: Sun"));
    assert!(rendered.contains("median Δlon="));
    assert!(rendered.contains("p95 Δlon="));
    assert!(rendered.contains("median Δlat="));
    assert!(rendered.contains("p95 Δlat="));
    assert!(rendered.contains("median Δdist="));
    assert!(rendered.contains("p95 Δdist="));
    assert!(rendered.contains(
        "Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
    ));
    assert!(rendered.contains("VSOP87 source documentation:"));
    assert!(rendered.contains("VSOP87 frame treatment:"));
    assert!(rendered.contains("VSOP87 request policy:"));
    assert!(rendered.contains("VSOP87 source audit:"));
    assert!(rendered.contains("VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"));
    let source_documentation_alias =
        render_cli(&["source-documentation"]).expect("source documentation alias should render");
    assert_eq!(
        source_documentation_alias,
        format_vsop87_source_documentation_summary()
    );
    let source_documentation_health_alias = render_cli(&["source-documentation-health"])
        .expect("source documentation health alias should render");
    assert_eq!(
        source_documentation_health_alias,
        format_vsop87_source_documentation_health_summary()
    );
    assert_eq!(
        render_cli(&["source-documentation-health", "extra"])
            .expect_err("source documentation health alias should reject extra arguments"),
        "source-documentation-health does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["source-documentation", "extra"])
            .expect_err("source documentation alias should reject extra arguments"),
        "source-documentation does not accept extra arguments"
    );
    let mut source_documentation_summary = source_documentation_summary();
    source_documentation_summary.source_specification_count += 1;
    let source_documentation_error = source_documentation_summary
        .validate()
        .expect_err("source documentation summary should detect catalog drift");
    assert_eq!(
        format_validated_vsop87_source_documentation_summary_for_report(
            &source_documentation_summary
        ),
        format!("VSOP87 source documentation: unavailable ({source_documentation_error})")
    );
    let mut source_documentation_health_summary = source_documentation_health_summary();
    source_documentation_health_summary.source_file_count += 1;
    let source_documentation_health_error = source_documentation_health_summary
        .validate()
        .expect_err("source documentation health should detect catalog drift");
    assert_eq!(
        format_validated_vsop87_source_documentation_health_summary_for_report(
            &source_documentation_health_summary
        ),
        format!(
            "VSOP87 source documentation health: unavailable ({source_documentation_health_error})"
        )
    );
    let source_audit_alias =
        render_cli(&["source-audit"]).expect("source audit alias should render");
    assert_eq!(source_audit_alias, source_audit_summary_for_report());
    assert_eq!(
        render_cli(&["source-audit", "extra"])
            .expect_err("source audit alias should reject extra arguments"),
        "source-audit does not accept extra arguments"
    );
    let generated_binary_audit_alias = render_cli(&["generated-binary-audit"])
        .expect("generated binary audit alias should render");
    assert_eq!(
        generated_binary_audit_alias,
        generated_binary_audit_summary_for_report()
    );
    assert_eq!(
        render_cli(&["generated-binary-audit", "extra"])
            .expect_err("generated binary audit alias should reject extra arguments"),
        "generated-binary-audit does not accept extra arguments"
    );
    let reference_asteroid_source_summary_alias =
        render_cli(&["reference-asteroid-source-summary"])
            .expect("reference asteroid source summary alias should render");
    assert_eq!(
        reference_asteroid_source_summary_alias,
        reference_asteroid_source_window_summary_for_report()
    );
    assert!(rendered.contains("VSOP87 canonical J2000 source-backed evidence:"));
    assert!(rendered.contains("VSOP87 canonical J2000 equatorial companion evidence:"));
    assert!(rendered.contains("VSOP87 canonical J2000 batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
    assert!(rendered.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
    assert!(rendered.contains("JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"));
    assert!(rendered.contains("VSOP87 canonical J1900 batch parity:"));
    assert!(rendered.contains("VSOP87 source-backed body evidence:"));
    assert!(rendered.contains("Lunar reference envelope:"));
    assert!(rendered.contains("Lunar equatorial reference envelope:"));
    assert!(rendered.contains("Lunar source windows:"));
    assert!(rendered.contains("JPL interpolation quality:"));
    assert!(rendered.contains(&reference_snapshot_2451916_major_body_interior_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report()));
    assert!(rendered.contains("Compact summary views: compatibility-profile-summary, release-notes-summary, backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Artifact boundary envelope:"));
    assert!(rendered.contains(
        &artifact_inspection_summary_for_report()
            .expect("artifact inspection summary should build")
    ));
    assert!(rendered.contains("residual-bearing bodies: asteroid:433-Eros"));
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release gate reminders:"));
    assert!(rendered.contains("verify-compatibility-profile"));
    assert!(rendered.contains("See release-notes and release-checklist"));
}
