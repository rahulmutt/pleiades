//! validation/benchmark report, mean-obliquity, and interpolation tests (white-box; moved verbatim from the former `tests.rs`).

use super::*;
use pleiades_core::{current_release_profile_identifiers, TimeScale};

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn benchmark_report_renders_a_time_summary() {
    let report = render_benchmark_report(10).expect("benchmark should render");
    let provenance = workspace_provenance();
    assert_eq!(provenance.validate(), Ok(()));
    assert!(report.contains(&provenance.summary_line()));
    assert!(report.contains("Benchmark report"));
    assert!(report.contains("Summary: backend="));
    assert!(report.contains("Representative 1600-2600 window"));
    assert!(report.contains("Apparentness: Mean"));
    assert!(report.contains("Methodology:"));
    assert!(report.contains("single-request and batch paths are measured separately"));
    assert!(report.contains("chart assembly is measured end to end"));
    assert!(report.contains("Single-request elapsed:"));
    assert!(report.contains("Batch elapsed:"));
    assert!(report.contains("Estimated corpus heap footprint:"));
    assert!(report.contains("Nanoseconds per request (single):"));
    assert!(report.contains("Nanoseconds per request (batch):"));
    assert!(report.contains("Batch throughput:"));
    assert!(report.contains("Artifact lookup benchmark report"));
    assert!(report.contains("Encoded bytes:"));
    assert!(report.contains("Nanoseconds per lookup:"));
    assert!(report.contains("Lookups per second:"));
    assert!(report.contains("Artifact decode benchmark report"));
    assert!(report.contains("Nanoseconds per decode:"));
    assert!(report.contains("Decodes per second:"));
    assert!(report.contains("Chart benchmark report"));
    assert!(report.contains("Summary: backend="));
    assert!(report.contains("Representative chart validation scenarios"));
    assert!(report.contains("Chart elapsed:"));
    assert!(report.contains("Nanoseconds per chart:"));
    assert!(report.contains("Charts per second:"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn benchmark_report_summary_line_mentions_the_backend_and_throughput() {
    let corpus = benchmark_corpus();
    let backend = default_candidate_backend();
    let report =
        benchmark_backend(&backend, &corpus, 1).expect("benchmark should produce a report");
    let summary = report.summary_line();
    assert_eq!(report.validated_summary_line(), Ok(summary.clone()));
    assert!(summary.contains("backend="));
    assert!(summary.contains("corpus="));
    assert!(summary.contains("apparentness="));
    assert!(summary.contains("samples per round="));
    assert!(summary.contains("single ns/request="));
    assert!(summary.contains("batch ns/request="));
    assert!(summary.contains("batch throughput="));
    assert!(summary.contains("estimated corpus heap footprint="));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn benchmark_matrix_summary_command_renders_the_matrix_block() {
    let rendered = render_cli(&["benchmark-matrix-summary", "--rounds", "1"])
        .expect("benchmark matrix summary should render");
    assert!(rendered.contains("Benchmark matrix summary"));
    assert!(rendered.contains("Benchmark corpora"));
    assert!(rendered.contains("comparison corpus: corpus name="));
    assert!(rendered.contains("benchmark corpus: corpus name="));
    assert!(rendered.contains("packaged-data benchmark corpus: corpus name="));
    assert!(rendered.contains("chart benchmark corpus: corpus name="));
    assert!(rendered.contains("Benchmark rows"));
    assert!(rendered.contains("reference benchmark: backend="));
    assert!(rendered.contains("candidate benchmark: backend="));
    assert!(rendered.contains("packaged-data benchmark: backend="));
    assert!(rendered.contains("chart benchmark: backend="));
    assert!(rendered.contains("artifact decode benchmark: artifact="));
    assert!(rendered.contains("packaged-artifact size: "));
    assert!(rendered.contains("Packaged-artifact fit posture"));
    assert!(rendered.contains("fit envelope: "));
    assert!(rendered.contains("fit margins: "));
    assert!(rendered.contains("fit threshold violation count: 0"));
    assert!(rendered.contains("fit threshold violations: 0; details: none"));
    assert!(rendered.contains("fit sample classes: boundary continuity="));
    assert!(rendered.contains("fit thresholds: mean Δlon≤"));
    assert!(rendered.contains("target thresholds: profile id="));
    assert!(rendered.contains("target-threshold scope envelopes: scope=luminaries;"));
    assert!(render_benchmark_matrix_summary(1)
        .expect("benchmark matrix summary should build")
        .contains("Packaged-artifact fit posture"));

    let alias_rendered = render_cli(&["benchmark-matrix", "--rounds", "1"])
        .expect("benchmark matrix alias should render");
    assert!(alias_rendered.contains("Benchmark matrix summary"));
    assert!(alias_rendered.contains("Benchmark corpora"));
    assert!(alias_rendered.contains("Benchmark rows"));
    assert!(alias_rendered.contains("Packaged-artifact fit posture"));
}

#[test]
fn benchmark_backend_rejects_zero_rounds() {
    let corpus = benchmark_corpus();
    let backend = default_candidate_backend();

    let error = benchmark_backend(&backend, &corpus, 0)
        .expect_err("zero-round benchmarks should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("benchmark rounds must be greater than zero"));
}

#[test]
fn benchmark_chart_backend_rejects_zero_rounds() {
    let error = benchmark_chart_backend(default_candidate_backend(), 0)
        .expect_err("zero-round chart benchmarks should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("chart benchmark rounds must be greater than zero"));
}

#[test]
fn benchmark_backend_rejects_zero_heap_footprint() {
    let corpus = benchmark_corpus();
    let backend = default_candidate_backend();
    let mut report =
        benchmark_backend(&backend, &corpus, 1).expect("benchmark should produce a report");
    report.estimated_corpus_heap_bytes = 0;

    let error = report
        .validate()
        .expect_err("zero-heap benchmarks should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("benchmark estimated corpus heap footprint must be greater than zero"));
}

#[test]
fn benchmark_workspace_provenance_display_matches_the_summary_helper() {
    let provenance = workspace_provenance();
    let expected = format!(
            "Benchmark provenance\n  source revision: {}\n  workspace status: {}\n  rustc version: {}\n  cargo version: {}\n  rustfmt version: {}\n  clippy version: {}",
            provenance.source_revision,
            provenance.workspace_status,
            provenance.rustc_version,
            provenance.cargo_version,
            provenance.rustfmt_version,
            provenance.clippy_version
        );

    assert_eq!(provenance.validate(), Ok(()));
    assert_eq!(provenance.summary_line(), expected);
    assert_eq!(provenance.to_string(), expected);
}

#[test]
fn benchmark_workspace_provenance_validation_rejects_blank_or_multiline_fields() {
    let blank_source_revision = WorkspaceProvenance {
        source_revision: String::new(),
        workspace_status: "clean".to_string(),
        rustc_version: "rustc 1.0.0 (dummy)".to_string(),
        cargo_version: "cargo 1.0.0 (dummy)".to_string(),
        rustfmt_version: "rustfmt 1.0.0 (dummy)".to_string(),
        clippy_version: "clippy 1.0.0 (dummy)".to_string(),
    };
    assert_eq!(
        blank_source_revision.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "source revision"
        }
    );

    let multiline_rustc_version = WorkspaceProvenance {
        source_revision: "abc123def456".to_string(),
        workspace_status: "dirty".to_string(),
        rustc_version: "rustc 1.0.0\nextra".to_string(),
        cargo_version: "cargo 1.0.0 (dummy)".to_string(),
        rustfmt_version: "rustfmt 1.0.0 (dummy)".to_string(),
        clippy_version: "clippy 1.0.0 (dummy)".to_string(),
    };
    assert_eq!(
        multiline_rustc_version.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "rustc version"
        }
    );

    let multiline_cargo_version = WorkspaceProvenance {
        source_revision: "abc123def456".to_string(),
        workspace_status: "dirty".to_string(),
        rustc_version: "rustc 1.0.0 (dummy)".to_string(),
        cargo_version: "cargo 1.0.0\nextra".to_string(),
        rustfmt_version: "rustfmt 1.0.0 (dummy)".to_string(),
        clippy_version: "clippy 1.0.0 (dummy)".to_string(),
    };
    assert_eq!(
        multiline_cargo_version.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "cargo version"
        }
    );

    let blank_rustfmt_version = WorkspaceProvenance {
        source_revision: "abc123def456".to_string(),
        workspace_status: "dirty".to_string(),
        rustc_version: "rustc 1.0.0 (dummy)".to_string(),
        cargo_version: "cargo 1.0.0 (dummy)".to_string(),
        rustfmt_version: String::new(),
        clippy_version: "clippy 1.0.0 (dummy)".to_string(),
    };
    assert_eq!(
        blank_rustfmt_version.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "rustfmt version"
        }
    );

    let multiline_clippy_version = WorkspaceProvenance {
        source_revision: "abc123def456".to_string(),
        workspace_status: "dirty".to_string(),
        rustc_version: "rustc 1.0.0 (dummy)".to_string(),
        cargo_version: "cargo 1.0.0 (dummy)".to_string(),
        rustfmt_version: "rustfmt 1.0.0 (dummy)".to_string(),
        clippy_version: "clippy 1.0.0\nextra".to_string(),
    };
    assert_eq!(
        multiline_clippy_version.validate().unwrap_err(),
        WorkspaceProvenanceValidationError::FieldInvalid {
            field: "clippy version"
        }
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn validation_report_validate_rejects_drifted_chart_benchmark_corpus() {
    let mut report = build_validation_report(10).expect("validation report should build");
    report.chart_benchmark_corpus.name.clear();

    let error = report
        .validate()
        .expect_err("chart benchmark corpus drift should be rejected");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("validation report chart benchmark corpus is invalid"));
    assert!(error
        .message
        .contains("corpus summary name must not be blank"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn validation_report_display_rejects_drifted_regression_archive() {
    let mut report = build_validation_report(10).expect("validation report should build");
    report.archived_regressions.corpus_name.clear();

    let rendered = report.to_string();
    assert!(rendered.contains("Validation report unavailable"));
    assert!(rendered.contains("regression archive corpus name must not be blank"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn validation_report_includes_corpus_metadata() {
    let report = render_validation_report(10).expect("validation report should render");
    let validation_report = build_validation_report(10).expect("validation report should build");
    assert!(report.contains("Validation report"));
    let release_profiles = current_release_profile_identifiers();
    assert!(report.contains("Compatibility profile"));
    assert!(report.contains(&format!(
        "  id: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(report.contains("API stability posture"));
    assert!(report.contains(&format!(
        "  id: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(report.contains(&format!(
        "Release profile identifiers: v1 compatibility={}, api-stability={}",
        release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
    )));
    assert!(report.contains("Implemented backend matrices"));
    assert!(report.contains("Selected asteroid coverage"));
    assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(report.contains("exact J2000 evidence: 6 bodies at JD 2451545.0"));
    assert!(report.contains("Ceres"));
    assert!(report.contains("Pallas"));
    assert!(report.contains("Juno"));
    assert!(report.contains("Vesta"));
    assert!(report.contains("asteroid:433-Eros"));
    assert!(report.contains("JPL snapshot reference backend"));
    assert!(report.contains("VSOP87 planetary backend"));
    assert!(report.contains("ELP lunar backend (Moon and lunar nodes)"));
    assert!(report.contains("Packaged data backend"));
    assert!(report.contains("Composite routed backend"));
    assert!(report.contains("Target compatibility catalog:"));
    assert!(report.contains("Comparison corpus"));
    assert!(report.contains("JPL Horizons release-grade comparison window"));
    assert!(
        report.contains("Comparison snapshot coverage: 162 rows across 10 bodies and 18 epochs")
    );
    assert!(report.contains("Apparentness: Mean"));
    assert!(report.contains("Benchmark corpus"));
    assert!(report.contains("Representative 1600-2600 window"));
    assert!(report.contains("estimated corpus heap footprint:"));
    assert!(report.contains("Chart benchmark corpus"));
    assert!(report.contains("Representative chart validation scenarios"));
    assert!(report.contains("Packaged artifact decode benchmark"));
    assert!(report.contains("Chart benchmark"));
    assert!(report.contains("House validation corpus"));
    assert!(report.contains("request: instant="));
    assert!(report.contains("Mid-latitude reference chart"));
    assert!(report.contains("Polar stress chart"));
    assert!(report.contains("Northern high-latitude stress chart"));
    assert!(report.contains("Equatorial reference chart"));
    assert!(report.contains("Southern hemisphere reference chart"));
    assert!(report.contains("Reference backend"));
    assert!(report.contains("Candidate backend"));
    assert!(report.contains("Comparison summary"));
    assert!(report.contains("95th percentile absolute deltas:"));
    assert!(report.contains(&format!(
        "max longitude delta: {:.12}° ({})",
        validation_report.comparison.summary.max_longitude_delta_deg,
        validation_report
            .comparison
            .summary
            .max_longitude_delta_body
            .as_ref()
            .expect("comparison summary should include a max longitude body")
    )));
    assert!(report.contains(&format!(
        "max latitude delta: {:.12}° ({})",
        validation_report.comparison.summary.max_latitude_delta_deg,
        validation_report
            .comparison
            .summary
            .max_latitude_delta_body
            .as_ref()
            .expect("comparison summary should include a max latitude body")
    )));
    assert!(report.contains(&format!(
        "max distance delta: {:.12} AU ({})",
        validation_report
            .comparison
            .summary
            .max_distance_delta_au
            .expect("comparison summary should include a max distance delta"),
        validation_report
            .comparison
            .summary
            .max_distance_delta_body
            .as_ref()
            .expect("comparison summary should include a max distance body")
    )));
    assert!(report.contains("Body-class error envelopes"));
    let body_class_envelopes = report
        .split("Body-class error envelopes")
        .nth(1)
        .expect("report should include body-class error envelopes");
    assert!(body_class_envelopes.contains("max longitude delta:"));
    assert!(body_class_envelopes.contains("median longitude delta:"));
    assert!(body_class_envelopes.contains("95th percentile longitude delta:"));
    assert!(body_class_envelopes.contains("median latitude delta:"));
    assert!(body_class_envelopes.contains("95th percentile latitude delta:"));
    assert!(body_class_envelopes.contains("rms longitude delta:"));
    assert!(body_class_envelopes.contains("rms latitude delta:"));
    assert!(body_class_envelopes.contains("median distance delta:"));
    assert!(body_class_envelopes.contains("95th percentile distance delta:"));
    assert!(body_class_envelopes.contains("rms distance delta:"));
    assert!(body_class_envelopes.contains(" ("));
    assert!(report.contains("Body-class tolerance posture"));
    let body_class_tolerance_posture = report
        .split("Body-class tolerance posture")
        .nth(1)
        .expect("report should include body-class tolerance posture");
    assert!(body_class_tolerance_posture.contains("backend family: composite"));
    assert!(body_class_tolerance_posture
        .contains("profile: phase-1 full-file VSOP87B planetary evidence"));
    assert!(body_class_tolerance_posture.contains("mean longitude delta:"));
    assert!(body_class_tolerance_posture.contains("median longitude delta:"));
    assert!(body_class_tolerance_posture.contains("95th percentile longitude delta:"));
    assert!(body_class_tolerance_posture.contains("rms longitude delta:"));
    assert!(body_class_tolerance_posture.contains("mean latitude delta:"));
    assert!(body_class_tolerance_posture.contains("median latitude delta:"));
    assert!(body_class_tolerance_posture.contains("95th percentile latitude delta:"));
    assert!(body_class_tolerance_posture.contains("rms latitude delta:"));
    assert!(body_class_tolerance_posture.contains("mean distance delta:"));
    assert!(body_class_tolerance_posture.contains("median distance delta:"));
    assert!(body_class_tolerance_posture.contains("95th percentile distance delta:"));
    assert!(body_class_tolerance_posture.contains("rms distance delta:"));
    assert!(body_class_tolerance_posture.contains("outside tolerance samples:"));
    assert!(report.contains("Expected tolerance status"));
    assert!(report.contains("margin Δlon="));
    assert!(report.contains("margin Δdist="));
    assert!(report.contains("Tolerance policy"));
    assert!(report.contains("candidate backend family: composite"));
    assert!(report.contains(
            "Major planets: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence"
        ));
    assert!(report.contains(
            "Pluto fallback (approximate): backend family=composite, profile=phase-1 Pluto approximate fallback evidence, bodies=0 (none), samples=0"
        ));
    assert!(report.contains("Luminaries"));
    assert!(report.contains("Major planets"));
    assert!(report.contains("interpolation quality checks:"));
    assert!(report.contains("JPL interpolation quality: 223 samples across 16 bodies"));
    assert!(report.contains("JPL interpolation quality kind coverage:"));
    assert!(report.contains("JPL interpolation posture: source="));
    assert!(report.contains("JPL interpolation body-class error envelopes:"));
    assert!(report.contains("Reference/hold-out overlap:"));
    assert!(report.contains("JPL independent hold-out:"));
    assert!(report.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
    assert!(report.contains("JPL independent hold-out equatorial parity:"));
    assert!(report.contains("JPL independent hold-out batch parity:"));
    assert!(report.contains("Lunar reference"));
    assert!(report.contains(
            "lunar reference evidence: 9 samples across 5 bodies, epoch range JD 2419914.5 (TT) → JD 2459278.5 (TT)"
        ));
    assert!(report.contains(
            "lunar reference mixed TT/TDB batch parity: 9 requests across 5 bodies, TT requests=5, TDB requests=4, order=preserved, single-query parity=preserved"
        ));
    assert!(report.contains(
            "lunar equatorial reference batch parity: 3 requests across 1 bodies, frame=Equatorial, order=preserved, single-query parity=preserved"
        ));
    assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(report.contains("exact J2000 evidence: 6 bodies at JD 2451545.0"));
    assert!(report.contains("Body comparison summaries"));
    assert!(report.contains("Sun: samples="));
    assert!(report.contains("Notable regressions"));
    assert!(report.contains("Archived regression cases"));
    assert!(report.contains("Reference benchmark"));
    assert!(report.contains("Candidate benchmark"));
    assert!(report.contains("Packaged-data benchmark corpus"));
    assert!(report.contains("Packaged-data benchmark"));
    assert!(report.contains("ns/request (single):"));
    assert!(report.contains("ns/request (batch):"));
    assert!(report.contains("batch throughput:"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn validation_report_summary_renders_a_compact_overview() {
    let report = render_validation_report_summary(10).expect("validation summary should render");
    let validation_report = build_validation_report(10).expect("validation report should build");
    let release_profiles = current_release_profile_identifiers();
    assert!(report.contains("Validation report summary"));
    assert!(report.contains(&format!(
        "Profile: {}",
        release_profiles.compatibility_profile_id
    )));
    assert!(report.contains(&format!(
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    )));
    assert!(report.contains("Comparison corpus"));
    assert!(report.contains("Source corpus: comparison corpus release-grade guard:"));
    assert!(report.contains("JPL source corpus contract:"));
    assert!(report.contains("evidence classification=release-tolerance=reference/comparison/production-generation validation summaries; hold-out=independent hold-out rows and interpolation-quality summaries; fixture exactness=reference snapshot exact J2000 evidence; provenance-only=source and manifest summaries"));
    assert!(report.contains("provenance-only=source and manifest summaries are provenance-only evidence; they validate corpus provenance and checksum posture but are excluded from tolerance, hold-out, and fixture-exactness claims"));
    assert!(!report.contains("JPL source corpus contract: JPL source corpus contract:"));
    assert!(report.contains("phase-2 corpus alignment:"));
    assert_eq!(
        validated_source_corpus_summary_for_report()
            .expect("source corpus summary should validate"),
        source_corpus_summary_for_report()
    );
    assert!(validation_report
        .to_string()
        .contains("Source corpus: comparison corpus release-grade guard:"));
    assert!(report.contains(&request_surface_summary_for_report()));
    assert!(report.contains("Reference snapshot"));
    assert!(report.contains(&reference_snapshot_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451911_major_body_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report()));
    assert!(report.contains(
        &pleiades_jpl::reference_snapshot_2451915_major_body_bridge_summary_for_report()
    ));
    assert!(report.contains(
        &pleiades_jpl::reference_snapshot_2451917_major_body_bridge_summary_for_report()
    ));
    assert!(report.contains(
        &pleiades_jpl::reference_snapshot_2451917_major_body_boundary_summary_for_report()
    ));
    assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_body_class_coverage_summary_for_report()));
    assert!(report.contains("release-grade guard: Pluto excluded from tolerance evidence"));
    assert!(report.contains("epoch labels: JD 2415020.5 (TT)"));
    assert!(report.contains("House validation corpus"));
    assert!(report.contains("House validation corpus: 9 scenarios"));
    assert!(report.contains("Comparison summary"));
    assert!(report.contains("95th percentile absolute deltas:"));
    assert!(report.contains("median longitude delta:"));
    assert!(report.contains("median latitude delta:"));
    assert!(report.contains("rms longitude delta:"));
    assert!(report.contains("rms latitude delta:"));
    assert!(report.contains("rms distance delta:"));
    assert!(report.contains("Tolerance policy"));
    assert!(report.contains("candidate backend family: composite"));
    assert!(report.contains("comparison window: JD"));
    assert!(report.contains("coordinate frames: Ecliptic"));
    assert!(report.contains(&format!(
        "max longitude delta: {:.12}° ({})",
        validation_report.comparison.summary.max_longitude_delta_deg,
        validation_report
            .comparison
            .summary
            .max_longitude_delta_body
            .as_ref()
            .expect("comparison summary should include a max longitude body")
    )));
    assert!(report.contains(&format!(
        "max latitude delta: {:.12}° ({})",
        validation_report.comparison.summary.max_latitude_delta_deg,
        validation_report
            .comparison
            .summary
            .max_latitude_delta_body
            .as_ref()
            .expect("comparison summary should include a max latitude body")
    )));
    assert!(report.contains(&format!(
        "max distance delta: {:.12} AU ({})",
        validation_report
            .comparison
            .summary
            .max_distance_delta_au
            .expect("comparison summary should include a max distance delta"),
        validation_report
            .comparison
            .summary
            .max_distance_delta_body
            .as_ref()
            .expect("comparison summary should include a max distance body")
    )));
    assert!(report.contains("Tolerance policy"));
    assert!(report.contains("candidate backend family: composite"));
    assert!(report.contains(
            "Major planets: backend family=composite, profile=phase-1 full-file VSOP87B planetary evidence"
        ));
    assert!(report.contains("Comparison tolerance audit"));
    assert!(report.contains("command: compare-backends-audit"));
    assert!(report.contains("status: clean"));
    assert!(!report.contains("regression bodies: Pluto"));
    let body_class_tolerance_posture = report
        .split("Body-class tolerance posture")
        .nth(1)
        .expect("report should include body-class tolerance posture");
    assert!(body_class_tolerance_posture.contains("mean Δlon="));
    assert!(body_class_tolerance_posture.contains("rms Δlon="));
    assert!(body_class_tolerance_posture.contains("mean Δlat="));
    assert!(body_class_tolerance_posture.contains("rms Δlat="));
    assert!(body_class_tolerance_posture.contains("mean Δdist="));
    assert!(body_class_tolerance_posture.contains("rms Δdist="));
    assert!(report.contains("JPL interpolation quality"));
    assert!(report.contains("JPL interpolation quality: 223 samples across 16 bodies"));
    assert!(report.contains("Reference/hold-out overlap:"));
    assert!(report.contains("JPL independent hold-out:"));
    assert!(report.contains("JPL independent hold-out equatorial parity:"));
    assert!(report.contains("JPL independent hold-out batch parity:"));
    assert!(report.contains("leave-one-out runtime interpolation evidence"));
    assert!(report.contains("@ JD"));
    assert!(report.contains("ELP lunar capability: lunar capability summary:"));
    assert!(report.contains(
            "ELP lunar request policy: frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        ));
    assert!(report.contains("Lunar reference"));
    assert!(report.contains(
            "lunar reference evidence: 9 samples across 5 bodies, epoch range JD 2419914.5 (TT) → JD 2459278.5 (TT)"
        ));
    assert!(report.contains("Lunar source windows\n  lunar source windows: 7 exact Moon samples across 1 bodies in 2 exact windows; 4 reference-only apparent Moon samples across 1 bodies in 4 apparent windows"));
    assert!(report.contains("Lunar high-curvature continuity evidence"));
    assert!(report.contains("Lunar high-curvature equatorial continuity evidence"));
    assert!(report.contains("within regression limits=true"));
    assert!(report.contains("Body comparison summaries"));
    assert!(report.contains("Body-class error envelopes"));
    assert!(report.contains("max longitude delta:"));
    assert!(report.contains("rms longitude delta:"));
    assert!(report.contains("rms latitude delta:"));
    assert!(report.contains("rms distance delta:"));
    let body_class_tolerance_posture = report
        .split("Body-class tolerance posture")
        .nth(1)
        .expect("report should include body-class tolerance posture");
    assert!(body_class_tolerance_posture.contains("mean Δlon="));
    assert!(body_class_tolerance_posture.contains("rms Δlon="));
    assert!(body_class_tolerance_posture.contains("mean Δlat="));
    assert!(body_class_tolerance_posture.contains("rms Δlat="));
    assert!(body_class_tolerance_posture.contains("mean Δdist="));
    assert!(body_class_tolerance_posture.contains("rms Δdist="));
    assert!(report.contains(" ("));
    assert!(report.contains("VSOP87 source-backed evidence"));
    assert!(report
            .contains("VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"));
    assert!(
        report.contains("VSOP87 generated binary audit: 8 checked-in blobs across 8 source files")
    );
    assert!(report.contains("VSOP87 canonical J2000 source-backed evidence: 8 samples"));
    assert!(report.contains("VSOP87 canonical J2000 interim outliers: none"));
    assert!(report.contains("VSOP87 canonical J2000 equatorial companion evidence: 8 samples"));
    assert!(report.contains("generated binary VSOP87B"));
    assert!(report.contains("generated binary VSOP87B; VSOP87B."));
    assert!(report.contains("max Δlon="));
    assert!(report.contains("max Δlat="));
    assert!(report.contains("max Δdist="));
    assert!(report.contains(
            "VSOP87 source-backed body evidence: 8 body profiles (0 vendored full-file, 8 generated binary), source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, 8 within interim limits, 0 outside interim limits; outside interim limits: none"
        ));
    assert!(report.contains("House validation corpus"));
    assert!(report.contains("Benchmark provenance"));
    assert!(report.contains("source revision:"));
    assert!(report.contains("workspace status:"));
    assert!(report.contains("rustc version:"));
    assert!(report.contains("Benchmark summaries"));
    assert!(report.contains("Release bundle verification: verify-release-bundle"));
    assert!(report.contains("Workspace audit: workspace-audit / audit"));
    assert!(report.contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(report.contains("Release notes summary: release-notes-summary"));
    assert!(report.contains("Release checklist summary: release-checklist-summary"));
    assert!(report.contains("Release summary: release-summary"));
    assert!(report.contains("Reference benchmark"));
    assert!(report.contains("Candidate benchmark"));
    assert!(report.contains("Packaged-data benchmark"));
    assert!(report.contains("Packaged artifact decode benchmark"));
    assert!(report.contains("Chart benchmark"));
}

#[test]
fn jpl_interpolation_quality_summary_includes_worst_case_body_labels() {
    let summary = jpl_interpolation_quality_summary().expect("summary should exist");
    assert!(!summary.max_bracket_span_body.is_empty());
    assert!(!summary.max_longitude_error_body.is_empty());
    assert!(!summary.max_latitude_error_body.is_empty());
    assert!(!summary.max_distance_error_body.is_empty());

    let rendered = format_jpl_interpolation_quality_summary(&summary);
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_bracket_span_body,
        format_instant(summary.max_bracket_span_epoch)
    )));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_longitude_error_body,
        format_instant(summary.max_longitude_error_epoch)
    )));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_latitude_error_body,
        format_instant(summary.max_latitude_error_epoch)
    )));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_distance_error_body,
        format_instant(summary.max_distance_error_epoch)
    )));
}

#[test]
fn interpolation_quality_request_corpus_summary_renders_the_explicit_slice() {
    let rendered = render_cli(&["interpolation-quality-request-corpus-summary"])
        .expect("interpolation quality request corpus summary should render");
    assert!(rendered.contains("Interpolation-quality sample request corpus:"));
    assert_eq!(
        rendered,
        interpolation_quality_sample_request_corpus_summary_for_report()
    );
    assert_eq!(
        render_cli(&["interpolation-quality-request-corpus"])
            .expect("interpolation quality request corpus alias should render"),
        rendered
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn artifact_validation_report_mentions_boundary_checks() {
    let report = render_artifact_report().expect("artifact report should render");
    assert!(report.contains("Artifact validation report"));
    assert!(report.contains("stage-5 packaged-data draft"));
    assert!(report.contains("byte order: little-endian"));
    assert!(report.contains("roundtrip decode: ok"));
    assert!(report.contains("checksum verified: ok"));
    assert!(report.contains("Bodies"));
    assert!(report.contains("Sun"));
    assert!(report.contains("Moon"));
    assert!(report.contains("Jupiter"));
    assert!(report.contains("Pluto"));
    assert!(report.contains("boundary checks"));
    assert!(report.contains("Artifact boundary envelope"));
    assert!(report.contains("Artifact request policy"));
    assert!(report.contains("Model error envelope"));
    assert!(report.contains("Body-class error envelopes"));
    assert!(report.contains("Artifact lookup benchmark"));
    assert!(report.contains("Artifact batch lookup benchmark"));
    assert!(report.contains("Artifact decode benchmark"));
    assert!(report.contains("nanoseconds per decode:"));
    assert!(report.contains("decodes per second:"));
    let body_class_envelopes = report
        .split("Body-class error envelopes")
        .nth(1)
        .expect("artifact report should include body-class error envelopes");
    assert!(body_class_envelopes.contains("max longitude delta:"));
    assert!(body_class_envelopes.contains(" ("));
    assert!(report.contains("Luminaries"));
    assert!(report.contains("Major planets"));
    assert!(report.contains("baseline backend"));
}

#[test]
fn mean_obliquity_frame_round_trip_summary_has_a_displayable_summary_line() {
    let summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");

    assert_eq!(summary.summary_line(), summary.to_string());
    assert!(summary.summary_line().contains("7 samples"));
    assert!(summary.summary_line().contains("max |Δlon|="));
    assert!(summary.summary_line().contains("mean |Δlon|="));
    assert!(summary.summary_line().contains("p95 |Δlon|="));
    assert!(summary.validate().is_ok());
}

#[test]
fn mean_obliquity_frame_round_trip_summary_render_cli_command_matches_display() {
    let rendered = render_cli(&["mean-obliquity-frame-round-trip-summary"])
        .expect("frame round-trip summary command should render");
    let alias = render_cli(&["mean-obliquity-frame-round-trip"])
        .expect("frame round-trip summary alias should render");
    let summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");

    assert_eq!(rendered, summary.to_string());
    assert_eq!(alias, summary.to_string());
    assert_eq!(
        render_cli(&["mean-obliquity-frame-round-trip", "extra"])
            .expect_err("frame round-trip summary alias should reject extra arguments"),
        "mean-obliquity-frame-round-trip does not accept extra arguments"
    );
}

#[test]
fn mean_obliquity_frame_round_trip_sample_corpus_matches_the_canonical_summary() {
    let samples = mean_obliquity_frame_round_trip_sample_corpus();

    assert_eq!(samples.len(), 7);
    assert!(samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() > 80.0));
    assert!(samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() < -80.0));
    assert!(samples
        .iter()
        .any(|(coordinates, _)| coordinates.longitude.degrees() > 350.0));
    assert_eq!(
        samples
            .first()
            .expect("sample corpus should not be empty")
            .1
            .scale,
        TimeScale::Tt
    );
    assert_eq!(
        samples
            .last()
            .expect("sample corpus should not be empty")
            .1
            .scale,
        TimeScale::Tt
    );

    let summary = mean_obliquity_frame_round_trip_summary_from_samples(&samples)
        .expect("canonical sample corpus should remain valid");
    assert_eq!(summary, mean_obliquity_frame_round_trip_summary().unwrap());
}

#[test]
fn mean_obliquity_frame_round_trip_sample_corpus_rejects_missing_equator() {
    let mut samples = mean_obliquity_frame_round_trip_sample_corpus();
    samples[1].0 = EclipticCoordinates::new(
        Longitude::from_degrees(90.0),
        pleiades_core::Latitude::from_degrees(1.0),
        Some(1.0),
    );

    let error = mean_obliquity_frame_round_trip_summary_from_samples(&samples)
        .expect_err("sample corpus without an equatorial sample should fail");
    assert!(error.contains("must include an equatorial sample"));
}

#[test]
fn mean_obliquity_frame_round_trip_sample_corpus_rejects_missing_wraparound() {
    let mut samples = mean_obliquity_frame_round_trip_sample_corpus();
    samples[5].0 = EclipticCoordinates::new(
        Longitude::from_degrees(320.0),
        pleiades_core::Latitude::from_degrees(89.25),
        Some(0.5),
    );

    let error = mean_obliquity_frame_round_trip_summary_from_samples(&samples)
        .expect_err("sample corpus without a wraparound sample should fail");
    assert!(error.contains("must include a wraparound longitude sample"));
}

#[test]
fn mean_obliquity_frame_round_trip_summary_formatter_rejects_drift() {
    let mut summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    summary.sample_count = 0;

    let rendered = format_mean_obliquity_frame_round_trip_summary_for_report(&summary);
    assert!(rendered.contains("mean-obliquity frame round-trip unavailable"));
    assert!(rendered.contains("mean-obliquity frame round-trip summary has no samples"));
}

#[test]
fn mean_obliquity_frame_round_trip_summary_validate_rejects_metric_drift() {
    let mut summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");
    summary.mean_longitude_delta_deg += 0.001;

    let error = summary
        .validate()
        .expect_err("metric drift should fail validation");
    assert!(error.contains("drifted from the canonical sample set"));
}

#[test]
fn mean_obliquity_frame_round_trip_summary_validated_summary_line_matches_display() {
    let summary = mean_obliquity_frame_round_trip_summary()
        .expect("mean-obliquity frame round-trip summary should exist");

    assert_eq!(
        summary.validated_summary_line().unwrap(),
        summary.to_string()
    );
}
