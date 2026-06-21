//! request surface, backend matrix, and CLI rendering tests (white-box; moved verbatim from the former `tests.rs`).

use super::test_support::*;
use super::*;
use pleiades_core::{CoordinateFrame, TimeScale};

#[test]
fn request_surface_summary_validation_matches_the_report_line() {
    let summary = RequestSurfaceSummary::current();
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_chart_help_clause(),
        Ok(summary.chart_help_clause())
    );
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), request_surface_summary_for_report());
    assert_eq!(
        render_cli(&["request-surface"]).unwrap(),
        render_request_surface_summary_text()
    );
    assert_eq!(
            summary.summary_line(),
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        );
    assert_eq!(summary.summary_line().lines().count(), 1);
}

#[test]
fn request_surface_summary_validated_summary_line_rejects_drift() {
    let summary = RequestSurfaceSummary {
        instant: "",
        chart_request: "",
        backend_request: "",
        house_request: "",
        request_policy: "",
        cli_chart: "",
    };

    let error = summary
        .validated_chart_help_clause()
        .expect_err("summary should reject blank request-surface labels");
    assert!(error
        .to_string()
        .contains("primary request surface instant mismatch"));

    let error = summary
        .validated_summary_line()
        .expect_err("summary should reject blank request-surface labels");
    assert!(error
        .to_string()
        .contains("primary request surface instant mismatch"));
}

#[test]
fn request_surface_summary_alias_rejects_extra_arguments_with_alias_specific_diagnostics() {
    let summary_error = render_cli(&["request-surface-summary", "extra"])
        .expect_err("request surface summary should reject extra arguments");
    assert_eq!(
        summary_error,
        "request-surface-summary does not accept extra arguments"
    );

    let error = render_cli(&["request-surface", "extra"])
        .expect_err("request surface alias should reject extra arguments");
    assert_eq!(error, "request-surface does not accept extra arguments");
}

#[test]
fn request_policy_summary_semantic_check_rejects_stale_rendering() {
    let mut stale = render_request_policy_summary_text();
    stale.push_str(" stale");

    let error = ensure_request_policy_summary_matches_current_rendering(&stale)
        .expect_err("stale request policy summary should fail the release-bundle semantic check");
    assert!(error
        .to_string()
        .contains("request policy summary no longer matches"));
}

#[test]
fn request_semantics_summary_semantic_check_rejects_stale_rendering() {
    let mut stale = render_request_semantics_summary_text();
    stale.push_str(" stale");

    let error = ensure_request_semantics_summary_matches_current_rendering(&stale).expect_err(
        "stale request-semantics summary should fail the release-bundle semantic check",
    );
    assert!(error
        .to_string()
        .contains("request-semantics summary no longer matches"));
}

#[test]
fn request_surface_summary_semantic_check_rejects_stale_rendering() {
    let mut stale = render_request_surface_summary_text();
    stale.push_str(" stale");

    let error = ensure_request_surface_summary_matches_current_rendering(&stale)
        .expect_err("stale request surface summary should fail the release-bundle semantic check");
    assert!(error
        .to_string()
        .contains("request surface summary no longer matches"));
}

#[test]
fn backend_matrix_report_semantic_check_rejects_stale_rendering() {
    let mut stale = render_backend_matrix_report().expect("backend matrix should render");
    stale.push_str(" stale");

    let error = ensure_backend_matrix_report_matches_current_rendering(&stale)
        .expect_err("stale backend matrix report should fail the release-bundle semantic check");
    assert!(error
        .to_string()
        .contains("backend matrix no longer matches"));
}

#[test]
fn backend_matrix_summary_semantic_check_rejects_stale_rendering() {
    let mut stale = render_backend_matrix_summary();
    stale.push_str(" stale");

    let error = ensure_backend_matrix_summary_matches_current_rendering(&stale)
        .expect_err("stale backend matrix summary should fail the release-bundle semantic check");
    assert!(error
        .to_string()
        .contains("backend matrix summary no longer matches"));
}

#[test]
fn cli_report_summary_lists_the_summary_command() {
    let rendered =
        render_cli(&["report-summary", "--rounds", "10"]).expect("report summary should render");
    assert!(rendered.contains("Validation report summary"));
    assert!(rendered.contains("Comparison corpus"));
    assert!(rendered.contains("Reference snapshot"));
    assert!(rendered.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_summary_for_report()));
    assert!(rendered.contains("Reference 2453000 major-body boundary evidence: 10 exact samples at JD 2453000.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2453000.5 boundary sample"));
    assert!(rendered.contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_body_class_coverage_summary_for_report()));
    assert!(
        rendered.contains("Comparison snapshot coverage: 162 rows across 10 bodies and 18 epochs")
    );
    assert!(rendered.contains("Body comparison summaries"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered.contains("Packaged-artifact profile"));
    assert!(rendered.contains(
            "Packaged-artifact output support: EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=derived; unlisted outputs: []; support counts: stored=0, derived=3, approximated=0, unsupported=3, unlisted=0"
        ));
    assert!(rendered.contains("Packaged-artifact target thresholds: profile id=pleiades-packaged-artifact-profile/stage-5-draft; target thresholds: production thresholds recorded; scopes=luminaries, major planets, pluto, lunar points, selected asteroids, custom bodies; fit envelope:"));
    assert!(rendered.contains("Packaged-artifact target-threshold state: target-threshold state: production thresholds recorded"));
    assert!(rendered.contains("Packaged-artifact fit envelope: fit envelope:"));
    assert!(rendered.contains("Packaged-artifact fit sample classes: fit sample classes:"));
    assert!(rendered.contains("Packaged-artifact target-threshold scope envelopes: scope envelopes: scope=luminaries; bodies=2 (Sun, Moon); fit envelope:"));
    assert!(rendered.contains("Packaged-artifact phase-2 corpus alignment: "));
    assert!(rendered.contains(
        "selected asteroid source request corpus=Selected asteroid source request corpus:"
    ));
    assert!(rendered
        .contains("Packaged-artifact generation manifest: Packaged artifact generation manifest:"));
    assert!(rendered.contains("Packaged batch parity:"));
    assert!(rendered.contains("Packaged frame parity:"));
    assert!(rendered.contains("Benchmark provenance"));
    assert!(rendered.contains("source revision:"));
    assert!(rendered.contains("workspace status:"));
    assert!(rendered.contains("rustc version:"));
    assert!(rendered.contains("Benchmark summaries"));
    assert!(rendered.contains("Packaged artifact decode benchmark"));

    let artifact_profile_coverage = render_cli(&["artifact-profile-coverage-summary"])
        .expect("artifact-profile-coverage-summary should render");
    assert_eq!(
        artifact_profile_coverage,
        format!(
            "Artifact profile coverage: {}",
            packaged_artifact_profile_coverage_summary_for_report()
        )
    );
    let packaged_frame_parity = render_cli(&["packaged-frame-parity-summary"])
        .expect("packaged-frame-parity-summary should render");
    assert_eq!(
        packaged_frame_parity,
        format!(
            "Packaged frame parity: {}",
            packaged_frame_parity_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["packaged-frame-parity"]).expect("packaged-frame-parity should render"),
        packaged_frame_parity
    );
    assert_eq!(
        render_cli(&["packaged-frame-parity", "extra"])
            .expect_err("packaged-frame-parity should reject extra arguments"),
        "packaged-frame-parity-summary does not accept extra arguments"
    );
    let packaged_frame_treatment = render_cli(&["packaged-frame-treatment-summary"])
        .expect("packaged-frame-treatment-summary should render");
    assert_eq!(
        packaged_frame_treatment,
        format!(
            "Packaged frame treatment: {}",
            packaged_frame_treatment_summary_for_report()
        )
    );

    let validation_report_summary = render_cli(&["validation-report-summary", "--rounds", "10"])
        .expect("validation-report-summary should render");
    assert!(validation_report_summary.contains("Validation report summary"));
    assert!(validation_report_summary.contains("Comparison corpus"));
    assert!(validation_report_summary.contains("Reference snapshot"));
    assert!(validation_report_summary.contains(&reference_snapshot_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_2451916_major_body_interior_summary_for_report()));
    assert!(
        validation_report_summary.contains(&reference_snapshot_source_window_summary_for_report())
    );
    assert!(validation_report_summary
        .contains(&reference_snapshot_body_class_coverage_summary_for_report()));
    assert!(validation_report_summary.contains("Body comparison summaries"));
    assert!(validation_report_summary.contains("Body-class error envelopes"));
    assert!(validation_report_summary.contains("Body-class tolerance posture"));
    assert!(validation_report_summary.contains("Expected tolerance status"));
    assert!(validation_report_summary.contains("margin Δlon="));
    assert!(validation_report_summary.contains("margin Δdist="));
    assert!(validation_report_summary.contains("Chart benchmark"));
    assert!(validation_report_summary.contains("Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="));
    assert!(validation_report_summary.contains("UTC convenience policy: built-in UTC convenience conversion remains out of scope; callers must supply TT/TDB offsets explicitly"));
    assert!(validation_report_summary.contains("Comparison tolerance audit"));
    assert!(validation_report_summary.contains("command: compare-backends-audit"));
    assert!(validation_report_summary.contains("status: clean"));
    assert!(validation_report_summary.contains("regression bodies:"));
    assert!(!validation_report_summary.contains("regression bodies: Pluto"));
    assert!(validation_report_summary
        .contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(validation_report_summary.contains("ayanamsa catalog validation: ok"));
    assert!(validation_report_summary.contains("Release notes summary: release-notes-summary"));
    assert!(validation_report_summary.contains("Packaged-artifact profile"));
    assert!(validation_report_summary.contains(
            "Packaged-artifact output support: EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=derived; unlisted outputs: []; support counts: stored=0, derived=3, approximated=0, unsupported=3, unlisted=0"
        ));
    assert!(validation_report_summary.contains(
        "Packaged-artifact speed policy: FittedDerivative; motion output support=derived"
    ));
    assert!(validation_report_summary.contains(
            "Packaged-artifact storage/reconstruction: Quantized linear segments stored in pleiades-compression artifact format; body-indexed segment tables support random access by body and lookup time across the advertised range; ecliptic and equatorial coordinates are reconstructed at runtime from stored channels; apparent, topocentric, and sidereal outputs remain unsupported; motion/speed is derived from fitted segment derivatives"
        ));
    assert!(validation_report_summary.contains("Packaged-artifact target thresholds: profile id=pleiades-packaged-artifact-profile/stage-5-draft; target thresholds: production thresholds recorded; scopes=luminaries, major planets, pluto, lunar points, selected asteroids, custom bodies; fit envelope:"));
    assert!(validation_report_summary.contains("Packaged-artifact fit envelope: fit envelope:"));
    assert!(validation_report_summary.contains("Packaged-artifact fit margins: mean Δlon="));
    assert!(
        validation_report_summary.contains("Packaged-artifact fit threshold violation count: 0")
    );
    assert!(validation_report_summary
        .contains("Packaged-artifact fit threshold violations: 0; details: none"));
    assert!(validation_report_summary
        .contains("Packaged-artifact fit sample classes: fit sample classes:"));
    assert!(validation_report_summary.contains("Packaged-artifact fit outliers: fit outliers:"));
    assert!(validation_report_summary.contains("Packaged-artifact target-threshold scope envelopes: scope envelopes: scope=luminaries; bodies=2 (Sun, Moon); fit envelope:"));
    assert!(validation_report_summary.contains("Packaged-artifact source-fit and hold-out sync: "));
    assert!(validation_report_summary.contains("Packaged-artifact phase-2 corpus alignment: "));
    assert!(validation_report_summary
        .contains("Packaged-artifact generation manifest: Packaged artifact generation manifest:"));
    assert!(validation_report_summary.contains("Packaged-artifact size: "));
    assert_report_contains_exact_line(
        &validation_report_summary,
        &format!(
            "Packaged-artifact generation policy: {}",
            packaged_artifact_generation_policy_summary_for_report()
        ),
    );
    assert_report_contains_exact_line(
        &validation_report_summary,
        &format!(
            "Packaged-artifact normalized intermediates: {}",
            packaged_artifact_normalized_intermediate_summary_for_report()
        ),
    );
    assert_report_contains_exact_line(
        &validation_report_summary,
        &format!(
            "Packaged-artifact generation residual bodies: {}",
            validated_packaged_artifact_generation_residual_bodies_summary_for_report()
                .expect("packaged artifact residual bodies summary should validate")
        ),
    );
    assert!(validation_report_summary.contains("Packaged request policy"));
    assert!(validation_report_summary
        .contains(&reference_snapshot_major_body_boundary_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_mars_jupiter_boundary_summary_for_report()));
    assert!(validation_report_summary
        .contains(&reference_snapshot_major_body_boundary_window_summary_for_report()));
    assert_report_contains_exact_line(
            &validation_report_summary,
            "Packaged lookup epoch policy: TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction",
        );
    assert!(validation_report_summary.contains("Packaged frame parity"));
    assert!(validation_report_summary.lines().any(|line| {
        line == format!(
            "  Packaged frame treatment: {}",
            packaged_frame_treatment_summary_for_report()
        )
    }));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"
        }));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"
        }));
    assert!(validation_report_summary.lines().any(|line| {
            line == "Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"
        }));
    assert!(validation_report_summary.contains("Frame policy:"));
    let mean_obliquity_frame_round_trip = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    assert!(validation_report_summary.lines().any(|line| {
        line == format!(
            "Mean-obliquity frame round-trip: {}",
            mean_obliquity_frame_round_trip
        )
    }));
    assert!(validation_report_summary.contains("Zodiac policy:"));
    assert!(validation_report_summary.contains(
            "Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.7.0, api-stability=pleiades-api-stability/0.2.0"
        ));
    assert!(validation_report_summary
        .contains("lookup epoch policy=TT-grid retag without relativistic correction"));
    assert!(validation_report_summary.contains("Benchmark summaries"));
    assert!(validation_report_summary.contains("Packaged artifact decode benchmark"));
}

#[test]
fn cli_rejects_zero_rounds_and_duplicate_bundle_release_args() {
    let benchmark_error = render_cli(&["benchmark", "--rounds", "0"])
        .expect_err("benchmark should reject zero rounds");
    assert!(benchmark_error.contains("invalid value for --rounds: expected a positive integer"));

    let duplicate_benchmark_rounds_error =
        render_cli(&["benchmark", "--rounds", "1", "--rounds", "2"])
            .expect_err("benchmark should reject duplicate rounds arguments");
    assert!(duplicate_benchmark_rounds_error.contains("duplicate value for --rounds argument"));

    let bundle_dir = unique_temp_dir("pleiades-release-bundle-zero-rounds-command");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    let bundle_error = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "0",
    ])
    .expect_err("bundle-release should reject zero rounds");
    assert!(bundle_error.contains("invalid value for --rounds: expected a positive integer"));

    let duplicate_out_error = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--out",
        &bundle_dir_string,
    ])
    .expect_err("bundle-release should reject duplicate --out arguments");
    assert!(duplicate_out_error.contains("duplicate value for --out <dir> argument"));

    let duplicate_rounds_error = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
        "--rounds",
        "2",
    ])
    .expect_err("bundle-release should reject duplicate --rounds arguments");
    assert!(duplicate_rounds_error.contains("duplicate value for --rounds argument"));

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

#[test]
fn cli_help_lists_the_validation_commands() {
    let rendered = render_cli(&["help"]).expect("help should render");
    assert!(rendered.contains("compare-backends"));
    assert!(rendered.contains("compare-backends-audit"));
    assert!(rendered.contains("backend-matrix"));
    assert!(rendered.contains("capability-matrix"));
    assert!(rendered.contains("backend-matrix-summary"));
    assert!(rendered.contains("matrix-summary"));
    assert!(rendered.contains("compatibility-profile"));
    assert!(rendered.contains("compatibility-caveats-summary"));
    assert!(rendered.contains("compatibility-caveats    Alias for compatibility-caveats-summary"));
    assert!(rendered.contains("catalog-inventory"));
    assert!(rendered.contains("Alias for compatibility-profile"));
    assert!(rendered.contains("benchmark [--rounds N]"));
    assert!(rendered.contains("benchmark-matrix [--rounds N]"));
    assert!(rendered.contains("benchmark-matrix-summary [--rounds N]"));
    assert!(rendered.contains("comparison-corpus-summary"));
    assert!(rendered.contains("comparison-corpus         Alias for comparison-corpus-summary"));
    assert!(rendered.contains("comparison-corpus-release-guard-summary"));
    assert!(rendered.contains(
        "comparison-corpus-release-guard  Alias for comparison-corpus-release-guard-summary"
    ));
    assert!(rendered.contains("comparison-corpus-guard-summary"));
    assert!(rendered.contains("comparison-envelope-summary"));
    assert!(rendered.contains("comparison-envelope       Alias for comparison-envelope-summary"));
    assert!(rendered.contains("comparison-tolerance-summary"));
    assert!(rendered
        .contains("comparison-tolerance-summary  Alias for comparison-tolerance-policy-summary"));
    assert!(rendered.contains("comparison-tolerance-scope-coverage-summary"));
    assert!(rendered.contains(
            "comparison-tolerance-scope-coverage  Alias for comparison-tolerance-scope-coverage-summary"
        ));
    assert!(rendered.contains("benchmark-corpus-summary"));
    assert!(rendered.contains("interpolation-quality-request-corpus-summary"));
    assert!(rendered.contains("lunar-reference-evidence-summary"));
    assert!(
        rendered.contains("lunar-reference-evidence  Alias for lunar-reference-evidence-summary")
    );
    assert!(rendered.contains("lunar-reference-mixed-time-scale-batch-parity-summary"));
    assert!(rendered.contains("reference-snapshot-mixed-tt-tdb-batch-parity  Alias for reference-snapshot-mixed-time-scale-batch-parity-summary"));
    assert!(rendered.contains("reference-snapshot-batch-parity-summary"));
    assert!(rendered.contains("reference-snapshot-batch-parity          Alias for reference-snapshot-batch-parity-summary"));
    assert!(rendered.contains("reference-snapshot-equatorial-parity-summary"));
    assert!(rendered.contains("reference-snapshot-equatorial-parity     Alias for reference-snapshot-equatorial-parity-summary"));
    assert!(rendered.contains(
            "lunar-reference-mixed-tt-tdb-batch-parity-summary  Alias for lunar-reference-mixed-time-scale-batch-parity-summary"
        ));
    assert!(rendered.contains(
            "lunar-reference-mixed-tt-tdb-batch-parity  Alias for lunar-reference-mixed-time-scale-batch-parity-summary"
        ));
    assert!(rendered.contains("jpl-source-posture-summary"));
    assert!(rendered.contains("jpl-source-posture         Alias for jpl-source-posture-summary"));
    assert!(rendered.contains("jpl-provenance-only-summary"));
    assert!(rendered.contains("jpl-provenance-only  Alias for jpl-provenance-only-summary"));
    assert!(rendered.contains("report [--rounds N]"));
    assert!(rendered.contains("generate-report"));
    assert!(rendered.contains("validation-report-summary [--rounds N]"));
    assert!(rendered.contains("report-summary [--rounds N]"));
    assert!(rendered.contains("validation-summary"));
    assert!(rendered.contains("validate-artifact"));
    assert!(rendered.contains("generate-packaged-artifact"));
    assert!(rendered.contains("regenerate-packaged-artifact"));
    assert!(rendered.contains("artifact-summary"));
    assert!(rendered.contains("artifact-boundary-envelope-summary"));
    assert!(rendered.contains("packaged-artifact-output-support-summary"));
    assert!(rendered.contains(
        "packaged-artifact-output-support       Alias for packaged-artifact-output-support-summary"
    ));
    assert!(rendered.contains("packaged-artifact-speed-policy-summary"));
    assert!(rendered.contains(
        "packaged-artifact-speed-policy       Alias for packaged-artifact-speed-policy-summary"
    ));
    assert!(rendered.contains("motion-policy-summary"));
    assert!(rendered.contains("motion-policy               Alias for motion-policy-summary"));
    assert!(rendered.contains("packaged-artifact-access-summary"));
    assert!(
        rendered.contains("packaged-artifact-access  Alias for packaged-artifact-access-summary")
    );
    assert!(rendered.contains(
        "packaged-artifact-path-policy-summary  Alias for packaged-artifact-access-summary"
    ));
    assert!(rendered.contains(
        "packaged-artifact-path-policy  Alias for packaged-artifact-path-policy-summary"
    ));
    assert!(rendered.contains("packaged-artifact-storage-summary"));
    assert!(rendered.contains(
        "packaged-artifact-storage           Alias for packaged-artifact-storage-summary"
    ));
    assert!(rendered.contains("packaged-artifact-production-profile-summary"));
    assert!(rendered.contains("packaged-artifact-production-profile"));
    assert!(rendered.contains("packaged-artifact-target-threshold-summary"));
    assert!(rendered.contains(
        "packaged-artifact-target-threshold  Alias for packaged-artifact-target-threshold-summary"
    ));
    assert!(rendered.contains("packaged-artifact-target-threshold-state-summary"));
    assert!(rendered.contains("packaged-artifact-target-threshold-state  Alias for packaged-artifact-target-threshold-state-summary"));
    assert!(rendered.contains("packaged-artifact-target-threshold-scope-envelopes-summary"));
    assert!(rendered.contains("packaged-artifact-target-threshold-scope-envelopes  Alias for packaged-artifact-target-threshold-scope-envelopes-summary"));
    assert!(rendered.contains("packaged-artifact-source-fit-holdout-sync-summary"));
    assert!(rendered.contains("packaged-artifact-source-fit-holdout-sync  Alias for packaged-artifact-source-fit-holdout-sync-summary"));
    assert!(rendered.contains("packaged-artifact-fit-envelope-summary"));
    assert!(rendered.contains(
        "packaged-artifact-fit-envelope  Alias for packaged-artifact-fit-envelope-summary"
    ));
    assert!(rendered.contains("packaged-artifact-fit-margins-summary"));
    assert!(rendered.contains(
        "packaged-artifact-fit-margins      Alias for packaged-artifact-fit-margins-summary"
    ));
    assert!(rendered.contains("packaged-artifact-fit-sample-classes-summary"));
    assert!(rendered.contains("packaged-artifact-fit-sample-classes  Alias for packaged-artifact-fit-sample-classes-summary"));
    assert!(rendered.contains("packaged-artifact-fit-outliers-summary"));
    assert!(rendered.contains(
        "packaged-artifact-fit-outliers  Alias for packaged-artifact-fit-outliers-summary"
    ));
    assert!(rendered.contains("packaged-artifact-fit-threshold-violation-count-summary"));
    assert!(rendered.contains("packaged-artifact-fit-threshold-violation-count  Alias for packaged-artifact-fit-threshold-violation-count-summary"));
    assert!(rendered.contains("packaged-artifact-fit-threshold-violations-summary"));
    assert!(rendered.contains("packaged-artifact-fit-threshold-violations  Alias for packaged-artifact-fit-threshold-violations-summary"));
    assert!(rendered.contains("packaged-artifact-generation-manifest-summary"));
    assert!(rendered.contains(
            "packaged-artifact-generation-manifest  Alias for packaged-artifact-generation-manifest-summary"
        ));
    assert!(rendered.contains("packaged-artifact-generation-manifest-checksum-summary"));
    assert!(rendered.contains(
            "packaged-artifact-generation-manifest-checksum  Alias for packaged-artifact-generation-manifest-checksum-summary"
        ));
    assert!(rendered.contains("packaged-artifact-generation-policy-summary"));
    assert!(rendered.contains("packaged-artifact-normalized-intermediate-summary"));
    assert!(rendered.contains(
            "packaged-artifact generation manifest, packaged-artifact generation manifest summary, packaged-artifact generation manifest checksum summary, packaged-artifact generation manifest checksum sidecar, benchmark-corpus summary"
        ));
    assert!(rendered.contains(
            "packaged-artifact-generation-policy     Alias for packaged-artifact-generation-policy-summary"
        ));
    assert!(rendered.contains(
            "packaged-artifact-normalized-intermediate  Alias for packaged-artifact-normalized-intermediate-summary"
        ));
    assert!(rendered.contains("packaged-artifact-lookup-epoch-policy-summary"));
    assert!(rendered.contains("Alias for packaged-artifact-lookup-epoch-policy-summary"));
    assert!(rendered.contains("packaged-artifact-generation-residual-summary"));
    assert!(rendered.contains("packaged-artifact-generation-residual-bodies-summary"));
    assert!(rendered.contains("packaged-artifact-regeneration-summary"));
    assert!(rendered.contains(
        "packaged-artifact-regeneration      Alias for packaged-artifact-regeneration-summary"
    ));
    assert!(rendered.contains("packaged-frame-parity-summary"));
    assert!(
        rendered.contains("packaged-frame-parity         Alias for packaged-frame-parity-summary")
    );
    assert!(rendered.contains("packaged-frame-treatment-summary"));
    assert!(rendered.contains("workspace-audit"));
    assert!(rendered.contains("native-dependency-audit"));
    assert!(rendered.contains("native-dependency-audit-summary"));
    assert!(rendered.contains("source-documentation-summary"));
    assert!(
        rendered.contains("source-documentation         Alias for source-documentation-summary")
    );
    assert!(rendered.contains("source-documentation-health-summary"));
    assert!(rendered
        .contains("source-documentation-health  Alias for source-documentation-health-summary"));
    assert!(rendered
        .contains("source-audit-summary      Print the compact VSOP87 source audit summary"));
    assert!(rendered.contains("source-audit              Alias for source-audit-summary"));
    assert!(rendered.contains(
        "generated-binary-audit-summary  Print the compact VSOP87 generated binary audit summary"
    ));
    assert!(rendered.contains("generated-binary-audit    Alias for generated-binary-audit-summary"));
    assert!(rendered.contains("api-stability"));
    assert!(rendered.contains("api-stability-summary"));
    assert!(rendered.contains("compatibility-profile-summary"));
    assert!(rendered.contains("house-formula-families-summary"));
    assert!(rendered.contains("house-formula-families    Alias for house-formula-families-summary"));
    assert!(rendered.contains("house-latitude-sensitive-summary"));
    assert!(rendered.contains("house-latitude-sensitive-constraints-summary"));
    assert!(rendered.contains("house-latitude-sensitive-failure-modes-summary"));
    assert!(rendered.contains(
            "house-latitude-sensitive-failure-modes  Alias for house-latitude-sensitive-failure-modes-summary"
        ));
    assert!(rendered.contains(
            "house-latitude-sensitive-constraints  Alias for house-latitude-sensitive-constraints-summary"
        ));
    assert!(
        rendered.contains("house-latitude-sensitive  Alias for house-latitude-sensitive-summary")
    );
    assert!(rendered.contains("house-code-aliases-summary"));
    assert!(rendered.contains("house-code-alias-summary"));
    assert!(rendered.contains("ayanamsa-catalog-validation-summary"));
    assert!(rendered
        .contains("ayanamsa-catalog-validation  Alias for ayanamsa-catalog-validation-summary"));
    assert!(rendered.contains("ayanamsa-metadata-coverage-summary"));
    assert!(rendered
        .contains("ayanamsa-metadata-coverage  Alias for ayanamsa-metadata-coverage-summary"));
    assert!(rendered.contains("ayanamsa-reference-offsets-summary"));
    assert!(rendered
        .contains("ayanamsa-reference-offsets  Alias for ayanamsa-reference-offsets-summary"));
    assert!(rendered.contains("time-scale-policy-summary"));
    assert!(rendered.contains("time-scale-policy       Alias for time-scale-policy-summary"));
    assert!(rendered.contains("utc-convenience-policy-summary"));
    assert!(rendered.contains("utc-convenience-policy  Alias for utc-convenience-policy-summary"));
    assert!(rendered.contains("delta-t-policy-summary"));
    assert!(rendered.contains("delta-t-policy         Alias for delta-t-policy-summary"));
    assert!(rendered.contains("zodiac-policy-summary"));
    assert!(rendered.contains("zodiac-policy         Alias for zodiac-policy-summary"));
    assert!(rendered.contains("observer-policy-summary"));
    assert!(rendered.contains("observer-policy        Alias for observer-policy-summary"));
    assert!(rendered.contains("apparentness-policy-summary"));
    assert!(rendered.contains("apparentness-policy     Alias for apparentness-policy-summary"));
    assert!(rendered.contains("native-sidereal-policy-summary"));
    assert!(rendered.contains("native-sidereal-policy   Alias for native-sidereal-policy-summary"));
    assert!(rendered.contains("frame-policy-summary"));
    assert!(rendered.contains("frame-policy             Alias for frame-policy-summary"));
    assert!(rendered.contains("production-generation-boundary-summary"));
    assert!(rendered.contains(
        "production-generation-boundary         Alias for production-generation-boundary-summary"
    ));
    assert!(rendered.contains("production-generation-boundary-request-corpus-summary"));
    assert!(rendered.contains("production-generation-boundary-request-corpus  Alias for production-generation-boundary-request-corpus-summary"));
    assert!(rendered.contains("production-generation-boundary-request-corpus-equatorial-summary"));
    assert!(rendered.contains("production-generation-boundary-request-corpus-equatorial  Alias for production-generation-boundary-request-corpus-equatorial-summary"));
    assert!(rendered.contains("production-generation-body-class-coverage-summary"));
    assert!(rendered.contains("production-body-class-coverage-summary"));
    assert!(rendered.contains("production-generation-source-window-summary"));
    assert!(rendered.contains("production-generation-source-window  Alias for production-generation-source-window-summary"));
    assert!(rendered.contains("production-generation-summary"));
    assert!(rendered
        .contains("production-generation           Alias for production-generation-summary"));
    assert!(rendered.contains("production-generation-quarter-day-boundary-summary"));
    assert!(rendered.contains("production-generation-quarter-day-boundary  Alias for production-generation-quarter-day-boundary-summary"));
    assert!(rendered.contains("production-generation-boundary-source-summary"));
    assert!(rendered.contains("production-generation-boundary-source  Alias for production-generation-boundary-source-summary"));
    assert!(rendered.contains("production-generation-boundary-window-summary"));
    assert!(rendered.contains("production-generation-boundary-window  Alias for production-generation-boundary-window-summary"));
    assert!(rendered.contains("production-generation-manifest-summary"));
    assert!(rendered.contains(
        "production-generation-manifest  Alias for production-generation-manifest-summary"
    ));
    assert!(rendered.contains("production-generation-manifest-checksum-summary"));
    assert!(rendered.contains(
            "production-generation-manifest-checksum  Alias for production-generation-manifest-checksum-summary"
        ));
    assert!(rendered.contains("production-generation-source-summary"));
    assert!(rendered.contains("production-generation-source-revision-summary"));
    assert!(rendered.contains(
            "production-generation-source-revision  Alias for production-generation-source-revision-summary"
        ));
    assert!(rendered.contains("comparison-snapshot-source-window-summary"));
    assert!(rendered.contains(
        "comparison-snapshot-source-window  Alias for comparison-snapshot-source-window-summary"
    ));
    assert!(rendered.contains("comparison-snapshot-source-summary"));
    assert!(rendered.contains(
        "comparison-snapshot-source        Alias for comparison-snapshot-source-summary"
    ));
    assert!(rendered.contains("comparison-snapshot-body-class-coverage-summary"));
    assert!(rendered.contains("comparison-body-class-coverage-summary"));
    assert!(rendered.contains("comparison-snapshot-manifest-summary"));
    assert!(rendered.contains("comparison-snapshot-summary"));
    assert!(rendered.contains("j2000-snapshot           Alias for comparison-snapshot-summary"));
    assert!(rendered.contains("comparison-snapshot         Alias for comparison-snapshot-summary"));
    assert!(rendered.contains("comparison-snapshot-batch-parity-summary"));
    assert!(rendered.contains(
        "comparison-snapshot-batch-parity  Alias for comparison-snapshot-batch-parity-summary"
    ));
    assert!(rendered.contains("reference-snapshot-source-window-summary"));
    assert!(rendered.contains(
        "reference-snapshot-source-window  Alias for reference-snapshot-source-window-summary"
    ));
    assert!(rendered.contains("reference-snapshot-source-summary"));
    assert!(rendered
        .contains("reference-snapshot-source        Alias for reference-snapshot-source-summary"));
    assert!(rendered.contains("reference-snapshot-lunar-boundary-summary"));
    assert!(rendered
        .contains("lunar-boundary-summary   Alias for reference-snapshot-lunar-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451912-major-body-boundary-summary"));
    assert!(rendered.contains("2451912-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2415020-selected-body-boundary-summary"));
    assert!(rendered.contains("2415020-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451545-major-body-boundary-summary"));
    assert!(rendered.contains("2451545-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2453000-major-body-boundary-summary"));
    assert!(rendered.contains("2453000-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451910-major-body-boundary-summary"));
    assert!(rendered.contains("2451910-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451911-major-body-boundary-summary"));
    assert!(rendered.contains("2451911-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451913-major-body-boundary-summary"));
    assert!(rendered.contains("2451913-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-boundary-summary"));
    assert!(rendered.contains("2451914-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-bridge-day-summary"));
    assert!(rendered.contains("2451914-major-body-bridge-day-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-bridge-summary"));
    assert!(rendered.contains("2451914-major-body-bridge-summary"));
    assert!(rendered.contains(
        "2451914-major-body-bridge  Alias for reference-snapshot-2451914-major-body-bridge-summary"
    ));
    assert!(rendered.contains("reference-snapshot-2451915-major-body-boundary-summary"));
    assert!(rendered.contains("2451915-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451915-major-body-bridge-summary"));
    assert!(rendered.contains("2451915-major-body-bridge-summary"));
    assert!(rendered.contains(
        "2451915-major-body-bridge  Alias for reference-snapshot-2451915-major-body-bridge-summary"
    ));
    assert!(rendered.contains("reference-snapshot-2451916-major-body-interior-summary"));
    assert!(rendered.contains("2451916-major-body-interior-summary"));
}

#[test]
fn release_profile_identifiers_summary_alias_command_renders_and_rejects_extra_arguments() {
    let rendered = render_cli(&["release-profile-identifiers-summary"])
        .expect("release-profile identifiers summary should render");
    assert_eq!(
        render_cli(&["release-profile-identifiers"]).unwrap(),
        rendered
    );
    assert!(rendered.contains("Release profile identifiers summary"));
    assert!(rendered.contains("Summary line: v1 compatibility="));
    assert_eq!(
        render_cli(&["release-profile-identifiers-summary", "extra"])
            .expect_err("summary alias should reject extra arguments"),
        "release-profile-identifiers-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["release-profile-identifiers", "extra"])
            .expect_err("summary alias should reject extra arguments"),
        "release-profile-identifiers does not accept extra arguments"
    );
}

#[test]
fn backend_matrix_command_renders_the_implemented_catalog() {
    let rendered = render_cli(&["backend-matrix"]).expect("backend matrix should render");
    assert!(rendered.contains("Implemented backend matrices"));
    assert!(rendered.contains("summary: id=jpl-snapshot; version="));
    assert!(rendered.contains("JPL snapshot reference backend"));
    assert!(!rendered.contains("JPL source corpus contract: JPL source corpus contract:"));
    assert!(rendered.contains(
            "selected asteroid coverage: 6 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        ));
    assert!(rendered.contains("Selected asteroid source evidence: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; bodies: Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"));
    assert!(rendered.contains("Selected asteroid source windows: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: Ceres: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Pallas: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Juno: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Vesta: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 10 samples across 10 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(rendered.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(rendered.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(rendered.contains("exact J2000 evidence: 6 bodies at JD 2451545.0"));
    assert!(rendered.contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(rendered.contains("nominal range:"));
    assert!(rendered.contains("provenance sources:"));
    assert!(rendered.contains("implementation status: fixture-reference"));
    assert!(rendered.contains("implementation status: partial-source-backed"));
    assert!(rendered.contains("implementation status: preliminary-algorithm"));
    assert!(rendered.contains("implementation status: draft-artifact"));
    assert!(rendered.contains("implementation status: routing-facade"));
    assert!(rendered.contains("family posture: data-backed"));
    assert!(rendered.contains("family posture: algorithmic"));
    assert!(rendered.contains("family posture: routing"));
    assert!(rendered.contains("implementation note:"));
    assert!(rendered.contains("Sun through Neptune now use generated binary VSOP87B source tables derived from the vendored full-file inputs"));
    assert!(rendered.contains("expected error classes:"));
    assert!(rendered.contains("required external data files:"));
    assert!(rendered.contains("crates/pleiades-jpl/data/reference_snapshot.csv"));
    assert!(rendered.contains("source documentation:"));
    assert!(rendered.contains("source audit:"));
    assert!(rendered.contains("Sun: IMCCE/CELMECH VSOP87B VSOP87B.ear"));
    assert!(rendered.contains("Paul Schlyter-style mean orbital elements for planets"));
    assert!(rendered.contains("body source profiles:"));
    assert!(rendered.contains("VSOP87B.ear"));
    assert!(rendered.contains("geocentric planetary reduction against Earth coefficients"));
    assert!(rendered.contains("solar reduction from Earth coefficients"));
    assert!(rendered.contains("canonical J2000 VSOP87B evidence:"));
    assert!(rendered.contains("Sun: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Mercury: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Venus: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Mars: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Jupiter: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Saturn: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Uranus: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Neptune: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(rendered.contains("Pluto: kind=mean orbital elements fallback, accuracy=Approximate"));
    assert!(rendered.contains("Meeus-style truncated lunar orbit formulas"));
    assert!(rendered.contains("NASA/JPL Horizons API vector tables (DE441)"));
    assert!(rendered.contains("interpolation quality checks:"));
    assert!(rendered.contains("JPL interpolation posture: source="));
    assert!(rendered.contains("interpolation, bracket span"));
    assert!(rendered.contains("TDB"));
    assert!(rendered.contains("VSOP87 planetary backend"));
    assert!(rendered.contains("Pluto remains the current approximate mean-element fallback special case until a Pluto-specific source path is selected"));
    assert!(rendered.contains("ELP lunar backend (Moon and lunar nodes)"));
    assert!(rendered.contains("specification summary: ELP lunar theory specification:"));
    assert!(rendered.contains("compact lunar and lunar-point formulas provide the current deterministic baseline while documented production lunar-theory ingestion remains open"));
    assert!(rendered.contains("Lunar source windows"));
    assert!(rendered.contains("Lunar high-curvature continuity evidence"));
    assert!(rendered.contains("Lunar high-curvature equatorial continuity evidence"));
    assert!(rendered.contains("unsupported bodies: True Apogee, True Perigee"));
    assert!(rendered.contains("Body/date/channel claims:"));
    assert!(rendered.contains("Packaged data backend"));
    assert!(rendered.contains("Composite routed backend"));
}

#[test]
fn backend_matrix_report_rejects_invalid_backend_metadata() {
    let entry = BackendMatrixEntry {
        label: "broken backend",
        metadata: BackendMetadata {
            id: pleiades_core::BackendId::new("broken"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: pleiades_core::BackendProvenance {
                summary: "  ".to_string(),
                data_sources: vec![],
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![CelestialBody::Sun.into()],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        },
        implementation_status: BackendImplementationStatus::PreliminaryAlgorithm,
        status_note: "broken for testing",
        expected_error_kinds: &[],
        required_data_files: &[],
    };

    let error = validate_backend_matrix_entry(&entry)
        .expect_err("invalid backend metadata should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .to_string()
        .contains("backend matrix entry `broken backend` has invalid metadata"));
    assert!(error.to_string().contains("provenance summary"));
}

#[test]
fn validated_house_code_aliases_summary_for_report_matches_current_profile() {
    let profile = current_compatibility_profile();
    assert_eq!(
        validated_house_code_aliases_summary_for_profile(&profile).unwrap(),
        profile.house_code_aliases_summary_line()
    );
}

#[test]
fn validated_house_code_aliases_summary_for_report_rejects_invalid_profiles() {
    let profile = CompatibilityProfile {
        summary: "",
        ..current_compatibility_profile()
    };

    let error = validated_house_code_aliases_summary_for_profile(&profile)
        .expect_err("invalid compatibility profiles should be rejected");
    assert!(error.contains("compatibility profile summary is blank"));
}

#[test]
fn format_capabilities_reports_unavailable_for_invalid_flag_sets() {
    let capabilities = BackendCapabilities {
        geocentric: true,
        topocentric: false,
        apparent: false,
        mean: false,
        batch: true,
        native_sidereal: false,
    };

    assert_eq!(
        format_capabilities(&capabilities),
        "unavailable (backend capabilities must support mean or apparent output)"
    );
}

#[test]
fn backend_matrix_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["backend-matrix-summary"]).expect("backend matrix summary should render");
    assert!(rendered.contains("Backend matrix summary"));
    assert!(rendered.contains("Backends: 5"));
    assert!(rendered.contains(&format!(
        "Profile: {}",
        current_compatibility_profile().profile_id
    )));
    let jpl_entry = implemented_backend_catalog()
        .into_iter()
        .find(|entry| entry.label == "JPL snapshot reference backend")
        .expect("JPL backend matrix entry should exist");
    assert!(jpl_entry
        .status_note
        .contains("reference corpus now spans 277 rows across 16 bodies and 23 epochs"));
    assert!(rendered.contains("Families:"));
    assert!(rendered.contains("Algorithmic: 2"));
    assert!(rendered.contains("ReferenceData: 1"));
    assert!(rendered.contains("CompressedData: 1"));
    assert!(rendered.contains("Composite: 1"));
    assert!(rendered.contains("Implementation statuses:"));
    assert!(rendered.contains("Native sidereal posture: unsupported across first-party backends"));
    assert!(rendered.contains("Nominal ranges: bounded: 2, open-ended: 3"));
    assert!(rendered.contains("fixture-reference: 1"));
    assert!(rendered.contains("partial-source-backed: 1"));
    assert!(rendered.contains("preliminary-algorithm: 1"));
    assert!(rendered.contains("draft-artifact: 1"));
    assert!(rendered.contains("routing-facade: 1"));
    assert!(rendered.contains("Accuracy classes:"));
    assert!(rendered.contains("Exact: 1"));
    assert!(rendered.contains("Approximate: 4"));
    assert!(rendered.contains("VSOP87 source documentation: 8 source specs, 8 source-backed body profiles, 1 approximate fallback mean-element body profile (Pluto); source-backed bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep"));
    assert!(rendered.contains(
            "source-backed breakdown: 8 generated binary bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file bodies (none), 0 truncated slice bodies (none)"
        ));
    assert!(rendered.contains(
            "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
        ));
    assert!(rendered.contains("VSOP87 canonical J2000 batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
    assert!(rendered.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
    assert!(rendered.contains("VSOP87 canonical mixed TT/TDB batch parity:"));
    assert!(rendered.contains("JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"));
    assert!(rendered.contains("JPL production-generation coverage:"));
    assert!(rendered.contains("Production generation source windows:"));
    assert!(rendered.contains("JPL production-generation body-class coverage:"));
    assert!(rendered.contains("JPL production-generation corpus shape:"));
    assert!(rendered.contains("JPL production-generation boundary request corpus equatorial:"));
    assert!(rendered.contains("JPL source corpus contract: JPL evidence classification:"));
    assert!(rendered.contains("JPL source posture: documented hybrid snapshot/hold-out fixture backend with a separate generation-input path"));
    assert!(rendered
        .contains("Comparison corpus release-grade guard: Pluto excluded from tolerance evidence"));
    assert!(rendered.contains("Reference/hold-out overlap:"));
    assert!(rendered.contains("JPL independent hold-out:"));
    assert!(rendered.contains("Release-grade body claims:"));
    let source_corpus_summary = source_corpus_summary_for_report();
    assert!(rendered.contains(&format!("Source corpus: {source_corpus_summary}")));
    assert!(rendered
        .lines()
        .any(|line| line == format!("Source corpus posture: {source_corpus_summary}")));
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Body/date/channel claims: {}",
            format_body_date_channel_claims_summary_for_report()
        )
    }));
    assert!(rendered.contains("Catalog posture: house systems="));
    assert!(rendered.contains("Target house scope:"));
    assert!(rendered.contains("Target ayanamsa scope:"));
    assert!(rendered.contains("Pluto fallback: Pluto remains an explicitly approximate fallback"));
    assert!(rendered.contains(&reference_snapshot_boundary_epoch_coverage_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_sparse_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_pre_bridge_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_dense_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(rendered.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(rendered
            .contains("VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
    assert!(rendered
        .contains("VSOP87 generated binary audit: 8 checked-in blobs across 8 source files"));
    assert!(rendered.contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
    assert!(rendered.contains("VSOP87 canonical J2000 interim outliers: none"));
    assert!(rendered.contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples"));
    assert!(rendered.contains("VSOP87 canonical J1900 batch parity:"));
    assert!(rendered.contains("quality counts: Exact=8, Interpolated=0, Approximate=1, Unknown=0"));
    assert!(rendered.contains("generated binary VSOP87B"));
    assert!(rendered.contains("generated binary VSOP87B; VSOP87B."));
    assert!(rendered.contains("max Δlon="));
    assert!(rendered.contains("max Δlat="));
    assert!(rendered.contains("max Δdist="));
    assert!(rendered.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, 8 within interim limits, 0 outside interim limits; outside interim limits: none"
        ));
    assert!(rendered.contains(
            "ELP lunar theory specification: Compact Meeus-style truncated lunar baseline [meeus-style-truncated-lunar-baseline; family: Meeus-style truncated analytical baseline; selected key: source identifier=meeus-style-truncated-lunar-baseline]"
        ));
    assert!(rendered.contains(
            "lunar theory catalog: 1 entry, 1 selected entry; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]"
        ));
    assert!(rendered.contains(
            "lunar theory catalog validation: ok (1 entries, 1 selected; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]; selected key: source identifier=meeus-style-truncated-lunar-baseline; selected family key: source family=Meeus-style truncated analytical baseline; aliases=1; specification sync, round-trip, alias uniqueness, body coverage disjointness, and case-insensitive key matching verified)"
        ));
    assert!(rendered.contains("lunar reference error envelope: 9 samples across 5 bodies"));
    assert!(rendered.contains("max Δlon="));
    assert!(rendered.contains("mean Δlon="));
    assert!(rendered.contains("median Δlon="));
    assert!(rendered.contains("p95 Δlon="));
    assert!(rendered.contains("max Δlat="));
    assert!(rendered.contains("mean Δlat="));
    assert!(rendered.contains("median Δlat="));
    assert!(rendered.contains("p95 Δlat="));
    assert!(rendered.contains("limits: Δlon≤1e-4°"));
    assert!(rendered.contains("request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"));
    assert!(rendered.contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
    assert!(rendered.contains("date-range note: Validated against the published 1992-04-12 geocentric Moon example, the published 1992-04-12 geocentric Moon RA/Dec example used for the mean-obliquity equatorial transform, the reference-only published 1968-12-24 apparent geocentric Moon comparison datum, the reference-only published 2004-04-01 NASA RP 1349 apparent Moon table row, the reference-only published 2006-09-07 EclipseWise apparent Moon coordinate row, J2000 lunar-point anchors including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example"));
    assert!(rendered.contains("lunar equatorial reference evidence: 3 samples across 1 bodies"));
    assert!(
        rendered.contains("lunar equatorial reference error envelope: 3 samples across 1 bodies")
    );
    assert!(rendered.contains("mean ΔRA="));
    assert!(rendered.contains("median ΔRA="));
    assert!(rendered.contains("p95 ΔRA="));
    assert!(rendered.contains("limits: ΔRA≤1e-2°"));
    assert!(rendered.contains("Lunar high-curvature continuity evidence"));
    assert!(rendered.contains("Lunar high-curvature equatorial continuity evidence"));
    assert!(
        rendered.contains("lunar high-curvature continuity evidence: 6 samples across 1 bodies")
    );
    assert!(rendered.contains(
        "lunar high-curvature equatorial continuity evidence: 6 samples across 1 bodies"
    ));
    assert!(rendered.contains("within regression limits=true"));
    assert!(rendered.contains("citation: Jean Meeus"));
    assert!(
        rendered.contains("provenance: Published lunar position, node, and mean-point formulas")
    );
    assert!(rendered
        .contains("redistribution: No external coefficient-file redistribution constraints apply"));
    assert!(rendered.contains("license: The current baseline is handwritten pure Rust"));
    assert!(rendered.contains("2 unsupported bodies: True Apogee, True Perigee"));
    assert!(rendered.contains("Distinct bodies covered:"));
    assert!(rendered.contains("Distinct coordinate frames: 2 (Ecliptic, Equatorial)"));
    assert!(rendered.contains("Distinct time scales: 2 (TT, TDB)"));
    assert!(rendered.lines().any(|line| {
            line == "Time-scale policy: direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"
        }));
    assert!(rendered.lines().any(|line| {
            line == "Delta T policy: built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers"
        }));
    assert!(rendered.lines().any(|line| {
            line == "Observer policy: chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"
        }));
    assert!(rendered.lines().any(|line| {
            line == "Apparentness policy: current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"
        }));
    assert!(rendered.lines().any(|line| {
            line == "Request policy: time-scale=direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model; observer=chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported; apparentness=current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support; frame=ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        }));
    assert_eq!(
        validated_request_policy_summary_for_report()
            .expect("current request policy summary should validate")
            .summary_line(),
        request_policy_summary_for_report()
            .validated_summary_line()
            .expect("current request policy summary should render")
    );
    assert!(rendered.contains(
            "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        ));
    assert!(rendered.lines().any(|line| {
            line == "Primary request surfaces: pleiades-types::Instant (tagged instant plus caller-supplied retagging); pleiades-core::ChartRequest (chart assembly plus house-observer preflight); pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight); pleiades-houses::HouseRequest (house-only observer calculations); request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints); pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)"
        }));
    assert!(rendered
        .contains("pleiades-core::ChartRequest (chart assembly plus house-observer preflight)"));
    assert!(rendered.contains("Native sidereal policy:"));
    assert!(rendered.contains("Frame policy:"));
    let mean_obliquity_frame_round_trip = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    assert!(rendered.lines().any(|line| {
        line == format!(
            "Mean-obliquity frame round-trip: {}",
            mean_obliquity_frame_round_trip
        )
    }));
    assert!(rendered.contains("Zodiac policy:"));
    assert!(rendered.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(rendered.contains("API stability summary: api-stability-summary"));
    assert!(rendered.contains("Release notes summary: release-notes-summary"));
    assert!(rendered.contains("Reference snapshot coverage: 277 rows across 16 bodies and 23 epochs (95 asteroid rows; JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies:"));
    assert!(rendered.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_high_curvature_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_high_curvature_window_summary_for_report()));
    assert!(
        rendered.contains(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report())
    );
    assert!(rendered.contains(&reference_snapshot_source_summary_for_report()));
    assert!(rendered.contains(&reference_snapshot_manifest_summary_for_report()));
    assert!(rendered.contains("Comparison audit: compare-backends-audit; status="));
    assert!(rendered.contains("within tolerance bodies="));
    assert!(rendered.contains("outside tolerance bodies="));
    assert!(
        rendered.contains("Packaged-artifact summary: artifact-summary / artifact-posture-summary")
    );
    assert!(rendered.contains("Release checklist summary: release-checklist-summary"));
    assert!(rendered.contains("Release bundle verification: verify-release-bundle"));
    assert!(rendered
            .lines()
            .any(|line| line == "Validation report summary: validation-report-summary / validation-summary / report-summary"));
    assert!(rendered.contains("See release-summary for the compact one-screen release overview."));

    let capability_matrix =
        render_cli(&["capability-matrix"]).expect("capability matrix should render");
    assert_eq!(
        capability_matrix,
        render_cli(&["backend-matrix"]).expect("backend matrix should render")
    );

    let matrix_summary = render_cli(&["matrix-summary"]).expect("matrix summary should render");
    assert_eq!(matrix_summary, rendered);
}

#[test]
fn release_profile_identifier_summary_helper_rejects_drifted_pairs() {
    let compatibility_profile_id = current_compatibility_profile().profile_id;
    let release_profiles = ReleaseProfileIdentifiers {
        compatibility_profile_id,
        api_stability_profile_id: compatibility_profile_id,
    };

    let rendered = validated_release_profile_identifiers_summary_for_report(&release_profiles);
    assert!(rendered.contains("unavailable"));
    assert!(rendered.contains("release-profile identifiers must be distinct"));
}

#[test]
fn request_policy_report_title_validation_rejects_drift() {
    assert!(validate_request_policy_report_title(
        RequestPolicyReportKind::Policy,
        "Request semantics summary\n",
    )
    .is_err());
    assert!(validate_request_policy_report_title(
        RequestPolicyReportKind::Semantics,
        "Request policy summary\n",
    )
    .is_err());
    assert!(validate_request_policy_report_title(
        RequestPolicyReportKind::Policy,
        "Request policy summary\n",
    )
    .is_ok());
    assert!(validate_request_policy_report_title(
        RequestPolicyReportKind::Semantics,
        "Request semantics summary\n",
    )
    .is_ok());
}

#[test]
fn request_policy_summary_and_alias_commands_render_the_policy_block() {
    let request_policy =
        render_cli(&["request-policy-summary"]).expect("request policy summary should render");
    assert_eq!(request_policy, render_request_policy_summary());

    let request_policy_alias =
        render_cli(&["request-policy"]).expect("request policy alias should render");
    assert_eq!(request_policy_alias, request_policy);

    let utc_convenience_policy = render_cli(&["utc-convenience-policy-summary"])
        .expect("UTC convenience policy summary should render");
    assert!(utc_convenience_policy.contains("UTC convenience policy summary"));
    assert!(utc_convenience_policy.contains("UTC convenience policy: built-in UTC convenience conversion remains out of scope; callers must supply TT/TDB offsets explicitly"));
    assert_eq!(
        render_cli(&["utc-convenience-policy"])
            .expect("UTC convenience policy alias should render"),
        utc_convenience_policy
    );

    let request_semantics = render_cli(&["request-semantics-summary"])
        .expect("request semantics summary should render");
    assert!(request_semantics.contains("Request semantics summary"));
    assert!(request_semantics.contains("Unsupported modes:"));
    assert_eq!(
            request_semantics,
            format!(
                "{}Unsupported modes: built-in UTC convenience remains out of scope; built-in Delta T remains out of scope; topocentric body positions remain unsupported; apparent-place corrections are rejected unless a backend explicitly advertises support; native sidereal backend output remains unsupported unless a backend explicitly advertises it\n",
                request_policy.replacen("Request policy summary", "Request semantics summary", 1)
            )
        );

    let request_semantics_alias =
        render_cli(&["request-semantics"]).expect("request semantics alias should render");
    assert!(request_semantics_alias.contains("Request semantics summary"));
    assert!(request_semantics_alias.contains("Unsupported modes:"));
    assert_eq!(request_semantics_alias, request_semantics);

    let unsupported_modes_summary = render_cli(&["unsupported-modes-summary"])
        .expect("unsupported modes summary should render");
    assert!(unsupported_modes_summary.contains("Unsupported modes summary"));
    assert!(unsupported_modes_summary.contains("Unsupported modes:"));
    assert_eq!(
        render_cli(&["unsupported-modes"]).expect("unsupported modes alias should render"),
        unsupported_modes_summary
    );

    let native_sidereal_policy = render_cli(&["native-sidereal-policy-summary"])
        .expect("native sidereal policy summary should render");
    assert!(native_sidereal_policy.contains("Native sidereal policy summary"));
    assert!(native_sidereal_policy.contains("Native sidereal policy: native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    assert_eq!(
        render_cli(&["native-sidereal-policy"])
            .expect("native sidereal policy alias should render"),
        native_sidereal_policy
    );

    let zodiac_policy =
        render_cli(&["zodiac-policy-summary"]).expect("zodiac policy summary should render");
    assert!(zodiac_policy.contains("Zodiac policy summary"));
    assert!(zodiac_policy.contains("Zodiac policy: tropical only"));
    assert_eq!(
        render_cli(&["zodiac-policy"]).expect("zodiac policy alias should render"),
        zodiac_policy
    );

    let request_policy_summary_error = render_cli(&["request-policy-summary", "extra"])
        .expect_err("request policy summary should reject extra arguments");
    assert_eq!(
        request_policy_summary_error,
        "request-policy-summary does not accept extra arguments"
    );

    let request_policy_error = render_cli(&["request-policy", "extra"])
        .expect_err("request policy alias should reject extra arguments");
    assert_eq!(
        request_policy_error,
        "request-policy does not accept extra arguments"
    );

    let request_semantics_summary_error = render_cli(&["request-semantics-summary", "extra"])
        .expect_err("request semantics summary should reject extra arguments");
    assert_eq!(
        request_semantics_summary_error,
        "request-semantics-summary does not accept extra arguments"
    );

    let request_semantics_error = render_cli(&["request-semantics", "extra"])
        .expect_err("request semantics alias should reject extra arguments");
    assert_eq!(
        request_semantics_error,
        "request-semantics does not accept extra arguments"
    );

    let native_sidereal_policy_summary_error =
        render_cli(&["native-sidereal-policy-summary", "extra"])
            .expect_err("native sidereal policy summary should reject extra arguments");
    assert_eq!(
        native_sidereal_policy_summary_error,
        "native-sidereal-policy-summary does not accept extra arguments"
    );

    let zodiac_policy_summary_error = render_cli(&["zodiac-policy-summary", "extra"])
        .expect_err("zodiac policy summary should reject extra arguments");
    assert_eq!(
        zodiac_policy_summary_error,
        "zodiac-policy-summary does not accept extra arguments"
    );

    let zodiac_policy_error = render_cli(&["zodiac-policy", "extra"])
        .expect_err("zodiac policy alias should reject extra arguments");
    assert_eq!(
        zodiac_policy_error,
        "zodiac-policy does not accept extra arguments"
    );

    let native_sidereal_policy_error = render_cli(&["native-sidereal-policy", "extra"])
        .expect_err("native sidereal policy alias should reject extra arguments");
    assert_eq!(
        native_sidereal_policy_error,
        "native-sidereal-policy does not accept extra arguments"
    );
}
