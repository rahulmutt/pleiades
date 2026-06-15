//! JPL snapshot, reference, and source corpus rendering tests (white-box; moved verbatim from the former `tests.rs`).

use super::*;

#[test]
fn lunar_reference_mixed_time_scale_batch_parity_summary_renders_the_explicit_slice() {
    let rendered = render_cli(&["lunar-reference-mixed-time-scale-batch-parity-summary"])
        .expect("lunar reference mixed TT/TDB batch parity summary should render");
    assert!(rendered.contains("lunar reference mixed TT/TDB batch parity"));
    assert_eq!(rendered, lunar_reference_batch_parity_summary_for_report());
    assert_eq!(
        render_cli(&["lunar-reference-mixed-tt-tdb-batch-parity-summary"])
            .expect("lunar reference mixed TT/TDB batch parity alias should render"),
        rendered
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
}

#[test]
fn reference_snapshot_mixed_time_scale_batch_parity_summary_renders_the_short_alias() {
    let rendered = render_cli(&["reference-snapshot-mixed-time-scale-batch-parity-summary"])
        .expect("reference snapshot mixed TT/TDB batch parity summary should render");
    assert_eq!(
        rendered,
        reference_snapshot_mixed_time_scale_batch_parity_summary_text()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-mixed-tt-tdb-batch-parity-summary"])
            .expect("reference snapshot mixed TT/TDB batch parity alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-snapshot-mixed-tt-tdb-batch-parity"])
            .expect("reference snapshot mixed TT/TDB batch parity short alias should render"),
        rendered
    );
    assert_eq!(
            render_cli(&["reference-snapshot-mixed-tt-tdb-batch-parity", "extra"])
                .expect_err(
                    "reference snapshot mixed TT/TDB batch parity short alias should reject extra arguments"
                ),
            "reference-snapshot-mixed-time-scale-batch-parity-summary does not accept extra arguments"
        );
}

#[test]
fn lunar_reference_evidence_summary_command_renders_the_summary() {
    let rendered = render_cli(&["lunar-reference-evidence-summary"])
        .expect("lunar reference evidence summary should render");

    assert_eq!(render_cli(&["lunar-reference-evidence"]).unwrap(), rendered);
    assert_eq!(
        rendered,
        format!(
            "Lunar reference evidence summary\n{}\n",
            lunar_reference_evidence_summary_for_report()
        )
    );
    assert!(rendered.contains("Lunar reference evidence summary"));
    assert!(rendered.contains("lunar reference evidence: 9 samples across 5 bodies"));
}

#[test]
fn selected_asteroid_coverage_summary_validates_selected_body_lists() {
    let asteroids = reference_asteroids();
    let summary = selected_asteroid_coverage_summary(asteroids)
        .expect("reference asteroids should produce a coverage summary");
    assert_eq!(
            summary.validated_summary_line(),
            Ok("selected asteroid coverage: 6 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)".to_string())
        );

    let mut drifted_bodies = asteroids.to_vec();
    drifted_bodies[0] = CelestialBody::Moon;
    let error = SelectedAsteroidCoverageSummary {
        body_count: drifted_bodies.len(),
        bodies: drifted_bodies,
    }
    .validate()
    .expect_err("non-asteroid bodies should be rejected");
    assert_eq!(
        error,
        SelectedAsteroidCoverageSummaryValidationError::UnsupportedBody {
            index: 0,
            body: "Moon".to_string(),
        }
    );
}

#[test]
fn independent_holdout_source_window_summary_validation_rejects_drift() {
    let summary = independent_holdout_snapshot_source_window_summary_for_report();
    let drifted_summary =
        summary.replace("source-backed samples", "tampered source-backed samples");

    let error = ensure_independent_holdout_source_window_summary_matches_current_rendering(
        &drifted_summary,
    )
    .expect_err("drifted independent-holdout source window summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current independent-holdout source-window posture"));
}

#[test]
fn independent_holdout_manifest_summary_command_renders_the_manifest_block() {
    let rendered = render_cli(&["independent-holdout-manifest-summary"])
        .expect("independent hold-out manifest summary should render");
    let alias = render_cli(&["independent-holdout-manifest"])
        .expect("independent hold-out manifest alias should render");

    assert_eq!(rendered, independent_holdout_manifest_summary_for_report());
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&["independent-holdout-manifest", "extra"])
            .expect_err("independent hold-out manifest alias should reject extra arguments"),
        "independent-holdout-manifest does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_source_summary_matches_current_rendering() {
    let summary = reference_snapshot_source_summary_for_report();

    ensure_reference_snapshot_source_summary_matches_current_rendering(&summary)
        .expect("reference snapshot source summary should match the current rendering");
}

#[test]
fn reference_snapshot_bridge_day_summary_validation_rejects_drift() {
    let summary = reference_snapshot_bridge_day_summary_for_report();
    let drifted_summary = summary.replace("2451914.0", "2451914.1");

    let error =
        ensure_reference_snapshot_bridge_day_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted reference snapshot bridge day summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current reference snapshot bridge day posture"));
}

#[test]
fn reference_snapshot_source_summary_validation_rejects_drift() {
    let summary = reference_snapshot_source_summary_for_report();
    let drifted_summary = summary.replace("coverage=", "coverage=drifted-");

    let error =
        ensure_reference_snapshot_source_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted reference snapshot source summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current reference snapshot source posture"));
}

#[test]
fn reference_snapshot_manifest_summary_validation_rejects_drift() {
    let summary = reference_snapshot_manifest_summary_for_report();
    let drifted_summary = summary.replace(
        "source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "source=drifted NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
    );

    let error =
        ensure_reference_snapshot_manifest_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted reference snapshot manifest summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current reference snapshot manifest posture"));
}

#[test]
fn reference_snapshot_source_window_summary_validation_rejects_drift() {
    let summary = reference_snapshot_source_window_summary_for_report();
    let drifted_summary = summary.replace(
        "Reference snapshot source windows:",
        "Reference snapshot source windows (drifted):",
    );

    let error =
        ensure_reference_snapshot_source_window_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted reference snapshot source window summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current reference snapshot source-window posture"));
}

#[test]
fn comparison_snapshot_source_summary_matches_current_rendering() {
    let summary = comparison_snapshot_source_summary_for_report();

    ensure_comparison_snapshot_source_summary_matches_current_rendering(&summary)
        .expect("comparison snapshot source summary should match the current rendering");
}

#[test]
fn comparison_snapshot_source_summary_validation_rejects_drift() {
    let summary = comparison_snapshot_source_summary_for_report();
    let drifted_summary = summary.replace("coverage=", "coverage=drifted-");

    let error =
        ensure_comparison_snapshot_source_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted comparison snapshot source summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current comparison snapshot source posture"));
}

#[test]
fn comparison_snapshot_source_window_summary_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["comparison-snapshot-source-window-summary"])
        .expect("comparison snapshot source window summary should render");

    assert!(rendered.contains("Comparison snapshot source windows:"));
    assert!(rendered.contains("232 source-backed samples"));
    assert_eq!(
        rendered,
        comparison_snapshot_source_window_summary_for_report()
    );
}

#[test]
fn comparison_snapshot_source_window_alias_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["comparison-snapshot-source-window"])
        .expect("comparison snapshot source window alias should render");

    assert_eq!(
        rendered,
        comparison_snapshot_source_window_summary_for_report()
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-source-window", "extra"])
            .expect_err("comparison snapshot source window alias should reject extra arguments"),
        "comparison-snapshot-source-window does not accept extra arguments"
    );
}

#[test]
fn comparison_and_reference_snapshot_source_summary_commands_render_the_source_blocks() {
    let comparison = render_cli(&["comparison-snapshot-source-summary"])
        .expect("comparison snapshot source summary should render");
    assert!(comparison.contains("Comparison snapshot source:"));
    assert_eq!(comparison, comparison_snapshot_source_summary_for_report());

    let reference = render_cli(&["reference-snapshot-source-summary"])
        .expect("reference snapshot source summary should render");
    assert!(reference.contains("Reference snapshot source:"));
    assert_eq!(reference, reference_snapshot_source_summary_for_report());

    let comparison_alias = render_cli(&["comparison-snapshot-source"])
        .expect("comparison snapshot source alias should render");
    assert_eq!(
        comparison_alias,
        comparison_snapshot_source_summary_for_report()
    );

    let reference_alias = render_cli(&["reference-snapshot-source"])
        .expect("reference snapshot source alias should render");
    assert_eq!(
        reference_alias,
        reference_snapshot_source_summary_for_report()
    );
}

#[test]
fn reference_major_body_bridge_summary_command_renders_the_bridge_day() {
    let rendered = render_cli(&["reference-snapshot-major-body-bridge-summary"])
        .expect("reference major-body bridge summary should render");
    assert!(rendered.contains("Reference major-body bridge evidence:"));
    assert!(rendered.contains("2451915.0"));
    assert_eq!(
        rendered,
        reference_snapshot_major_body_bridge_summary_for_report()
    );
    let alias =
        render_cli(&["major-body-bridge-summary"]).expect("major body bridge alias should render");
    assert_eq!(alias, rendered);
    let concise_alias = render_cli(&["bridge-summary"]).expect("bridge alias should render");
    assert_eq!(concise_alias, rendered);
    let epoch_alias = render_cli(&["2451915-major-body-bridge-summary"])
        .expect("2451915 major body bridge alias should render");
    assert!(epoch_alias.contains("Reference 2451915 major-body bridge evidence:"));
    assert_eq!(
        epoch_alias,
        pleiades_jpl::reference_snapshot_2451915_major_body_bridge_summary_for_report()
    );
    let concise_epoch_alias = render_cli(&["2451915-major-body-bridge"])
        .expect("2451915 major body bridge concise alias should render");
    assert_eq!(concise_epoch_alias, epoch_alias);
    assert_eq!(
        render_cli(&["major-body-bridge-summary", "extra"])
            .expect_err("major body bridge alias should reject extra arguments"),
        "major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["bridge-summary", "extra"])
            .expect_err("bridge alias should reject extra arguments"),
        "bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451915-major-body-bridge-summary", "extra"])
            .expect_err("2451915 major body bridge alias should reject extra arguments"),
        "reference-snapshot-2451915-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451915-major-body-bridge", "extra"])
            .expect_err("2451915 major body bridge concise alias should reject extra arguments"),
        "reference-snapshot-2451915-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-major-body-bridge-summary", "extra"])
            .expect_err("reference major-body bridge summary should reject extra arguments"),
        "reference-snapshot-major-body-bridge-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451917_major_body_bridge_summary_command_renders_the_bridge_day() {
    let rendered = render_cli(&["reference-snapshot-2451917-major-body-bridge-summary"])
        .expect("reference 2451917 major-body bridge summary should render");
    assert!(rendered.contains("Reference 2451917 major-body bridge evidence:"));
    assert!(rendered.contains("2451917.0"));
    assert_eq!(
        rendered,
        reference_snapshot_2451917_major_body_bridge_summary_for_report()
    );
    let alias = render_cli(&["2451917-major-body-bridge-summary"])
        .expect("2451917 major body bridge alias should render");
    assert_eq!(alias, rendered);
    let concise_alias = render_cli(&["2451917-major-body-bridge"])
        .expect("2451917 major body bridge concise alias should render");
    assert_eq!(concise_alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451917-major-body-bridge-summary",
            "extra"
        ])
        .expect_err("reference 2451917 major body bridge summary should reject extra arguments"),
        "reference-snapshot-2451917-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451917-major-body-bridge-summary", "extra"])
            .expect_err("2451917 major body bridge alias should reject extra arguments"),
        "reference-snapshot-2451917-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451917-major-body-bridge", "extra"])
            .expect_err("2451917 major body bridge concise alias should reject extra arguments"),
        "reference-snapshot-2451917-major-body-bridge-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_bridge_day_summary_command_renders_the_bridge_day() {
    let rendered = render_cli(&["reference-snapshot-bridge-day-summary"])
        .expect("reference snapshot bridge day summary should render");
    assert!(rendered.contains("Reference snapshot bridge day:"));
    assert!(rendered.contains("2451914.0"));
    assert_eq!(rendered, reference_snapshot_bridge_day_summary_for_report());
    assert_eq!(
        rendered,
        reference_snapshot_2451914_bridge_day_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-bridge-day-summary", "extra"])
            .expect_err("reference snapshot bridge day summary should reject extra arguments"),
        "reference-snapshot-bridge-day-summary does not accept extra arguments"
    );
    let bridge_day_epoch_alias = render_cli(&["2451914-bridge-day-summary"])
        .expect("2451914 bridge day alias should render");
    assert_eq!(bridge_day_epoch_alias, rendered);
    assert_eq!(
        render_cli(&["2451914-bridge-day-summary", "extra"])
            .expect_err("2451914 bridge day alias should reject extra arguments"),
        "reference-snapshot-2451914-bridge-day-summary does not accept extra arguments"
    );
    let bridge_day_major_alias = render_cli(&["2451914-major-body-bridge-day-summary"])
        .expect("2451914 major body bridge-day alias should render");
    assert_eq!(
        bridge_day_major_alias,
        reference_snapshot_2451914_major_body_bridge_day_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2451914-major-body-bridge-day-summary", "extra"])
            .expect_err("2451914 major body bridge-day alias should reject extra arguments"),
        "reference-snapshot-2451914-major-body-bridge-day-summary does not accept extra arguments"
    );
    let bridge_epoch_alias = render_cli(&["2451914-major-body-bridge-summary"])
        .expect("2451914 major body bridge alias should render");
    assert_eq!(bridge_epoch_alias, rendered);
    let concise_bridge_epoch_alias = render_cli(&["2451914-major-body-bridge"])
        .expect("2451914 major body bridge concise alias should render");
    assert_eq!(concise_bridge_epoch_alias, rendered);
    assert_eq!(
        render_cli(&["2451914-major-body-bridge-summary", "extra"])
            .expect_err("2451914 major body bridge alias should reject extra arguments"),
        "reference-snapshot-2451914-major-body-bridge-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451914-major-body-bridge", "extra"])
            .expect_err("2451914 major body bridge concise alias should reject extra arguments"),
        "reference-snapshot-2451914-major-body-bridge-summary does not accept extra arguments"
    );
    let bridge_day_alias =
        render_cli(&["bridge-day-summary"]).expect("bridge day alias should render");
    assert_eq!(bridge_day_alias, rendered);
    assert_eq!(
        render_cli(&["bridge-day-summary", "extra"])
            .expect_err("bridge day alias should reject extra arguments"),
        "bridge-day-summary does not accept extra arguments"
    );
}

#[test]
fn help_text_lists_the_bridge_day_and_boundary_epoch_coverage_commands() {
    let help = render_cli(&["help"]).expect("help text should render");
    assert!(help.contains(
            "reference-snapshot-bridge-day-summary  Print the compact reference snapshot bridge day summary"
        ));
    assert!(
        help.contains("bridge-day-summary       Alias for reference-snapshot-bridge-day-summary")
    );
    assert!(help.contains(
            "reference-snapshot-boundary-epoch-coverage-summary  Print the compact reference snapshot boundary epoch coverage summary"
        ));
    assert!(help.contains(
            "reference-snapshot-boundary-epoch-coverage  Alias for reference-snapshot-boundary-epoch-coverage-summary"
        ));
    assert!(help.contains(
            "boundary-epoch-coverage-summary  Alias for reference-snapshot-boundary-epoch-coverage-summary"
        ));
    assert!(help.contains(
            "reference-snapshot-pre-bridge-boundary  Alias for reference-snapshot-pre-bridge-boundary-summary"
        ));
    assert!(help.contains(
        "production-generation-boundary         Alias for production-generation-boundary-summary"
    ));
    assert!(help.contains("source-corpus-posture-summary  Alias for source-corpus-summary"));
    assert!(help.contains("source-corpus-posture     Alias for source-corpus-posture-summary"));
    assert!(help.contains("ayanamsa-audit-summary    Print the compact ayanamsa audit summary"));
    assert!(help.contains("ayanamsa-audit            Alias for ayanamsa-audit-summary"));
}

#[test]
fn reference_snapshot_major_body_boundary_window_summary_command_renders_the_boundary_window_block()
{
    let rendered = render_cli(&["reference-snapshot-major-body-boundary-window-summary"])
        .expect("reference major-body boundary window summary should render");
    assert!(rendered.contains("Reference major-body boundary windows:"));
    assert_eq!(
        rendered,
        reference_snapshot_major_body_boundary_window_summary_for_report()
    );
    let alias = render_cli(&["major-body-boundary-window-summary"])
        .expect("major body boundary window alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-major-body-boundary-window-summary",
            "extra"
        ])
        .expect_err("reference major-body boundary window summary should reject extra arguments"),
        "reference-snapshot-major-body-boundary-window-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["major-body-boundary-window-summary", "extra"])
            .expect_err("major body boundary window alias should reject extra arguments"),
        "reference-snapshot-major-body-boundary-window-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_boundary_epoch_coverage_summary_command_renders_the_boundary_epoch_coverage_block(
) {
    let rendered = render_cli(&["reference-snapshot-boundary-epoch-coverage-summary"])
        .expect("reference snapshot boundary epoch coverage summary should render");
    assert!(rendered.contains("Reference snapshot boundary epoch coverage:"));
    assert_eq!(
        rendered,
        reference_snapshot_boundary_epoch_coverage_summary_for_report()
    );
    let alias = render_cli(&["boundary-epoch-coverage-summary"])
        .expect("boundary epoch coverage alias should render");
    let reference_alias = render_cli(&["reference-snapshot-boundary-epoch-coverage"])
        .expect("reference snapshot boundary epoch coverage alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(reference_alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-boundary-epoch-coverage-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot boundary epoch coverage summary should reject extra arguments"
        ),
        "reference-snapshot-boundary-epoch-coverage-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-epoch-coverage", "extra"]).expect_err(
            "reference snapshot boundary epoch coverage alias should reject extra arguments"
        ),
        "reference-snapshot-boundary-epoch-coverage-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["boundary-epoch-coverage-summary", "extra"])
            .expect_err("boundary epoch coverage alias should reject extra arguments"),
        "reference-snapshot-boundary-epoch-coverage-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_bridge_summary_command_renders_the_bridge_day() {
    let rendered = render_cli(&["selected-asteroid-bridge-summary"])
        .expect("selected asteroid bridge summary should render");
    assert!(rendered.contains("Selected asteroid bridge evidence:"));
    assert!(rendered.contains("2451915.0"));
    assert_eq!(rendered, selected_asteroid_bridge_summary_for_report());

    assert_eq!(
        render_cli(&["selected-asteroid-bridge-summary", "extra"])
            .expect_err("selected asteroid bridge summary should reject extra arguments"),
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
}

#[test]
fn selected_asteroid_source_evidence_summary_command_renders_the_source_evidence_block() {
    let rendered = render_cli(&["selected-asteroid-source-evidence-summary"])
        .expect("selected asteroid source evidence summary should render");
    assert!(rendered.contains("Selected asteroid source evidence:"));
    assert!(rendered.contains("Ceres"));
    assert_eq!(
        rendered,
        selected_asteroid_source_evidence_summary_for_report()
    );

    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-summary"])
            .expect("reference snapshot selected asteroid source summary should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-summary"])
            .expect("selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-summary", "extra"])
            .expect_err("selected asteroid source summary alias should reject extra arguments"),
        "selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2378498_source_summary_command_renders_the_epoch_slice() {
    let rendered = render_cli(&["reference-snapshot-2378498-selected-asteroid-source-summary"])
        .expect("reference snapshot 2378498 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2378498.5 source evidence:"));
    assert!(rendered.contains("JD 2378498.5 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2378498_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2378498-selected-asteroid-source-summary"])
            .expect("2378498 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2378498-selected-asteroid-source-summary", "extra"]).expect_err(
            "2378498 selected asteroid source summary alias should reject extra arguments"
        ),
        "2378498-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2001_source_summary_command_renders_the_epoch_slice() {
    let rendered = render_cli(&["reference-snapshot-2451917-selected-asteroid-source-summary"])
        .expect("reference snapshot 2451917 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2001-01-08 source evidence:"));
    assert!(rendered.contains("JD 2451917.5 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2451917_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2451917-selected-asteroid-source-summary"])
            .expect("2451917 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2451917-selected-asteroid-source-summary", "extra"]).expect_err(
            "2451917 selected asteroid source summary alias should reject extra arguments"
        ),
        "2451917-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2003_source_summary_command_renders_the_epoch_slice() {
    let rendered = render_cli(&["reference-snapshot-2453000-selected-asteroid-source-summary"])
        .expect("reference snapshot 2453000 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2003-12-27 source evidence:"));
    assert!(rendered.contains("JD 2453000.5 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2453000_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2453000-selected-asteroid-source-summary"])
            .expect("2453000 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2453000-selected-asteroid-source-summary", "extra"]).expect_err(
            "2453000 selected asteroid source summary alias should reject extra arguments"
        ),
        "2453000-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2500000_source_summary_command_renders_the_late_boundary_slice() {
    let rendered = render_cli(&["reference-snapshot-2500000-selected-asteroid-source-summary"])
        .expect("reference snapshot 2500000 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2500000 source evidence:"));
    assert!(rendered.contains("JD 2500000.0 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2500000_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2500000-selected-asteroid-source-summary"])
            .expect("2500000 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2500000-selected-asteroid-source-summary", "extra"]).expect_err(
            "2500000 selected asteroid source summary alias should reject extra arguments"
        ),
        "2500000-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_2634167_source_summary_command_renders_the_outer_boundary_slice() {
    let rendered = render_cli(&["reference-snapshot-2634167-selected-asteroid-source-summary"])
        .expect("reference snapshot 2634167 selected asteroid source summary should render");
    assert!(rendered.contains("Reference selected-asteroid 2634167 source evidence:"));
    assert!(rendered.contains("JD 2634167.0 (TDB)"));
    assert_eq!(
        rendered,
        pleiades_jpl::selected_asteroid_source_2634167_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2634167-selected-asteroid-source-summary"])
            .expect("2634167 selected asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["2634167-selected-asteroid-source-summary", "extra"]).expect_err(
            "2634167 selected asteroid source summary alias should reject extra arguments"
        ),
        "2634167-selected-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_source_request_corpus_summary_command_renders_the_request_corpus_slice() {
    let rendered = render_cli(&["selected-asteroid-source-request-corpus-summary"])
        .expect("selected asteroid source request corpus summary should render");
    assert!(rendered.contains("Selected asteroid source request corpus:"));
    assert!(rendered.contains("observerless"));
    assert_eq!(
        rendered,
        validated_selected_asteroid_source_request_corpus_summary_for_report()
            .expect("selected asteroid source request corpus summary should validate")
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-request-corpus"])
            .expect("selected asteroid source request corpus alias should render"),
        rendered
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
}

#[test]
fn selected_asteroid_source_window_summary_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["selected-asteroid-source-window-summary"])
        .expect("selected asteroid source window summary should render");
    assert!(rendered.contains("Selected asteroid source windows:"));
    assert!(rendered.contains("Ceres"));
    assert_eq!(
        rendered,
        validated_selected_asteroid_source_window_summary_for_report()
            .expect("selected asteroid source window summary should validate")
    );

    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-window-summary"])
            .expect("reference snapshot selected asteroid source window summary should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-snapshot-selected-asteroid-source-window"])
            .expect("reference snapshot selected asteroid source window alias should render"),
        rendered
    );
    assert_eq!(
            render_cli(&["reference-snapshot-selected-asteroid-source-window", "extra"])
                .expect_err("reference snapshot selected asteroid source window alias should reject extra arguments"),
            "reference-snapshot-selected-asteroid-source-window does not accept extra arguments"
        );
    assert_eq!(
        render_cli(&["selected-asteroid-source-window"])
            .expect("selected asteroid source window alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["selected-asteroid-source-window", "extra"])
            .expect_err("selected asteroid source window alias should reject extra arguments"),
        "selected-asteroid-source-window does not accept extra arguments"
    );
}

#[test]
fn reference_asteroid_source_window_summary_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["reference-asteroid-source-window-summary"])
        .expect("reference asteroid source window summary should render");
    assert!(rendered.contains("Reference asteroid source windows:"));
    assert!(rendered.contains("Ceres"));
    assert_eq!(
        rendered,
        validated_reference_asteroid_source_window_summary_for_report()
            .expect("reference asteroid source window summary should validate")
    );

    assert_eq!(
        render_cli(&["reference-asteroid-source-window"])
            .expect("reference asteroid source window alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-asteroid-source-window", "extra"])
            .expect_err("reference asteroid source window alias should reject extra arguments"),
        "reference-asteroid-source-window-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-asteroid-source-summary"])
            .expect("reference asteroid source summary alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-asteroid-source-summary", "extra"])
            .expect_err("reference asteroid source summary alias should reject extra arguments"),
        "reference-asteroid-source-summary does not accept extra arguments"
    );
}

#[test]
fn reference_asteroid_equatorial_evidence_summary_command_renders_the_equatorial_evidence_block() {
    let rendered = render_cli(&["reference-asteroid-equatorial-evidence-summary"])
        .expect("reference asteroid equatorial evidence summary should render");
    assert!(rendered.contains("Selected asteroid equatorial evidence:"));
    assert!(rendered.contains("mean-obliquity equatorial transform"));
    assert_eq!(
        rendered,
        reference_asteroid_equatorial_evidence_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-asteroid-equatorial-evidence"])
            .expect("reference asteroid equatorial evidence alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-asteroid-equatorial-evidence", "extra"]).expect_err(
            "reference asteroid equatorial evidence alias should reject extra arguments"
        ),
        "reference-asteroid-equatorial-evidence-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_boundary_day_summary_command_renders_the_boundary_day() {
    let rendered =
        render_cli(&["boundary-day-summary"]).expect("boundary day summary alias should render");
    assert!(rendered.contains("Reference snapshot boundary day:"));
    assert!(rendered.contains("2451915.5"));
    assert_eq!(
        rendered,
        reference_snapshot_sparse_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-day-summary"])
            .expect("reference snapshot boundary day summary should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-snapshot-boundary-day"])
            .expect("reference snapshot boundary day alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["reference-snapshot-sparse-boundary-summary"])
            .expect("reference snapshot sparse boundary summary should render"),
        rendered
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
}

#[test]
fn reference_snapshot_pre_bridge_boundary_summary_command_renders_the_pre_bridge_day() {
    let rendered = render_cli(&["reference-snapshot-pre-bridge-boundary-summary"])
        .expect("reference snapshot pre-bridge boundary summary should render");
    assert!(rendered.contains("Reference snapshot pre-bridge boundary day:"));
    assert!(rendered.contains(
            "JD 2451914.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); pre-bridge boundary day"
        ));
    assert_eq!(
        rendered,
        reference_snapshot_pre_bridge_boundary_summary_for_report()
    );

    let pre_bridge_alias = render_cli(&["pre-bridge-boundary-summary"])
        .expect("pre-bridge boundary summary alias should render");
    let pre_bridge_reference_alias = render_cli(&["reference-snapshot-pre-bridge-boundary"])
        .expect("reference snapshot pre-bridge boundary alias should render");
    assert_eq!(pre_bridge_alias, rendered);
    assert_eq!(pre_bridge_reference_alias, rendered);
    let pre_bridge_epoch_alias = render_cli(&["2451914-major-body-pre-bridge-summary"])
        .expect("2451914 major-body pre-bridge summary alias should render");
    assert_eq!(pre_bridge_epoch_alias, rendered);
    assert_eq!(
        render_cli(&["reference-snapshot-pre-bridge-boundary-summary", "extra"]).expect_err(
            "reference snapshot pre-bridge boundary summary should reject extra arguments"
        ),
        "reference-snapshot-pre-bridge-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["reference-snapshot-pre-bridge-boundary", "extra"]).expect_err(
            "reference snapshot pre-bridge boundary alias should reject extra arguments"
        ),
        "reference-snapshot-pre-bridge-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["pre-bridge-boundary-summary", "extra"])
            .expect_err("pre-bridge boundary summary alias should reject extra arguments"),
        "pre-bridge-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451914-major-body-pre-bridge-summary", "extra"]).expect_err(
            "2451914 major-body pre-bridge summary alias should reject extra arguments"
        ),
        "reference-snapshot-2451914-major-body-pre-bridge-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_dense_boundary_summary_command_renders_the_dense_day() {
    let rendered = render_cli(&["reference-snapshot-dense-boundary-summary"])
        .expect("reference snapshot dense boundary summary should render");
    assert!(rendered.contains("Reference snapshot dense boundary day:"));
    assert!(rendered.contains(
            "JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
        ));
    assert_eq!(
        rendered,
        reference_snapshot_dense_boundary_summary_for_report()
    );

    let dense_boundary_alias = render_cli(&["dense-boundary-summary"])
        .expect("dense boundary summary alias should render");
    assert_eq!(dense_boundary_alias, rendered);
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
}

#[test]
fn reference_snapshot_2451916_major_body_dense_boundary_summary_command_renders_the_dense_day() {
    let rendered = render_cli(&["reference-snapshot-2451916-major-body-dense-boundary-summary"])
        .expect("reference snapshot 2451916 major-body dense boundary summary should render");
    assert!(rendered.contains("Reference 2451916 major-body dense boundary evidence:"));
    assert!(rendered.contains("JD 2451916.5 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()
    );

    let alias = render_cli(&["2451916-major-body-dense-boundary-summary"])
        .expect("2451916 major-body dense boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
            render_cli(&["reference-snapshot-2451916-major-body-dense-boundary-summary", "extra"]).expect_err(
                "reference snapshot 2451916 major-body dense boundary summary should reject extra arguments"
            ),
            "reference-snapshot-2451916-major-body-dense-boundary-summary does not accept extra arguments"
        );
    assert_eq!(
            render_cli(&["2451916-major-body-dense-boundary-summary", "extra"])
                .expect_err("2451916 major-body dense boundary alias should reject extra arguments"),
            "reference-snapshot-2451916-major-body-dense-boundary-summary does not accept extra arguments"
        );
}

#[test]
fn selected_asteroid_dense_boundary_summary_command_renders_the_dense_day() {
    let rendered = render_cli(&["selected-asteroid-dense-boundary-summary"])
        .expect("selected asteroid dense boundary summary should render");
    assert!(rendered.contains("Selected asteroid dense boundary evidence:"));
    assert!(rendered.contains("2451916.5"));
    assert_eq!(
        rendered,
        selected_asteroid_dense_boundary_summary_for_report()
    );

    assert_eq!(
        render_cli(&["selected-asteroid-dense-boundary-summary", "extra"])
            .expect_err("selected asteroid dense boundary summary should reject extra arguments"),
        "selected-asteroid-dense-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn selected_asteroid_terminal_boundary_summary_command_renders_the_terminal_day() {
    let rendered = render_cli(&["selected-asteroid-terminal-boundary-summary"])
        .expect("selected asteroid terminal boundary summary should render");
    assert!(rendered.contains("Reference selected-asteroid terminal boundary evidence:"));
    assert!(rendered.contains("2500-01-01"));
    assert_eq!(
        rendered,
        selected_asteroid_terminal_boundary_summary_for_report()
    );

    assert_eq!(
        render_cli(&["selected-asteroid-terminal-boundary-summary", "extra"]).expect_err(
            "selected asteroid terminal boundary summary should reject extra arguments"
        ),
        "selected-asteroid-terminal-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn comparison_and_reference_snapshot_summary_commands_render_the_overall_blocks() {
    let comparison = render_cli(&["comparison-snapshot-summary"])
        .expect("comparison snapshot summary should render");
    assert!(comparison.contains("Comparison snapshot summary"));
    assert!(comparison.contains("Comparison snapshot coverage:"));
    assert_eq!(
        comparison,
        format!(
            "Comparison snapshot summary\n{}\n",
            comparison_snapshot_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["j2000-snapshot"]).expect("j2000 snapshot alias should render"),
        comparison
    );
    assert_eq!(
        render_cli(&["comparison-snapshot"]).expect("comparison snapshot alias should render"),
        comparison
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

    let reference = render_cli(&["reference-snapshot-summary"])
        .expect("reference snapshot summary should render");
    assert!(reference.contains("Reference snapshot summary"));
    assert!(reference.contains("Reference snapshot coverage:"));
    assert!(reference.contains("Reference 2500 major-body boundary evidence:"));
    assert!(reference
        .contains(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()));
    assert!(
        reference.contains(&reference_snapshot_2451916_major_body_boundary_summary_for_report())
    );
    assert!(
        reference.contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report())
    );
    assert!(
        reference.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report())
    );
    assert!(reference.contains(&reference_snapshot_bridge_day_summary_for_report()));
    assert!(
        reference.contains(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report())
    );
    assert!(
        reference.contains(&reference_snapshot_2451915_major_body_boundary_summary_for_report())
    );
    assert!(reference.contains(&reference_snapshot_2451915_major_body_bridge_summary_for_report()));
    assert!(reference.contains("Reference 2500 selected-body boundary evidence:"));
    assert!(reference.contains("Reference 2200 selected-body boundary evidence:"));
    assert!(reference.contains("Reference 2415020 selected-body boundary evidence:"));
    assert!(
        reference.contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report())
    );
    assert!(reference.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_dense_boundary_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_source_evidence_summary_for_report()));
    assert!(reference.contains(&selected_asteroid_source_window_summary_for_report()));
    assert!(reference.contains("Reference 2453000 major-body boundary evidence:"));
    assert!(reference.contains("Reference 2600000 major-body boundary evidence:"));
    assert!(reference.contains("Reference 2400000 major-body boundary evidence:"));
    assert!(reference.contains("Reference 2451545 major-body boundary evidence:"));
    assert!(
        reference.contains(&reference_snapshot_2451917_major_body_boundary_summary_for_report())
    );
    assert!(reference.contains(&reference_snapshot_2451917_major_body_bridge_summary_for_report()));
    assert!(
        reference.contains(&reference_snapshot_2360234_major_body_interior_summary_for_report())
    );
    assert!(
        reference.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report())
    );
    assert_eq!(
        reference,
        format!(
            "Reference snapshot summary\n{}\n",
            reference_snapshot_summary_for_report()
        )
    );
    assert_eq!(
        render_cli(&["reference-snapshot"]).expect("reference snapshot alias should render"),
        reference
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

    let reference_exact_j2000 = render_cli(&["reference-snapshot-exact-j2000-evidence-summary"])
        .expect("reference snapshot exact J2000 evidence should render");
    assert!(reference_exact_j2000.contains("Reference snapshot exact J2000 evidence summary"));
    assert!(reference_exact_j2000.contains(
        "Reference snapshot exact J2000 evidence: 16 exact J2000 samples at JD 2451545.0"
    ));
    assert_eq!(
        reference_exact_j2000,
        format!(
            "Reference snapshot exact J2000 evidence summary\n{}\n",
            reference_snapshot_exact_j2000_evidence_summary_for_report()
        )
    );
    let exact_j2000_evidence =
        render_cli(&["exact-j2000-evidence"]).expect("exact J2000 evidence alias should render");
    assert_eq!(exact_j2000_evidence, reference_exact_j2000);
    let reference_exact_j2000_alias = render_cli(&["reference-snapshot-exact-j2000-evidence"])
        .expect("reference snapshot exact J2000 evidence alias should render");
    assert_eq!(reference_exact_j2000_alias, reference_exact_j2000);
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
    assert_eq!(
        render_cli(&["reference-snapshot-exact-j2000-evidence-summary", "extra"])
            .expect_err("reference snapshot exact J2000 evidence should reject extra arguments"),
        "reference-snapshot-exact-j2000-evidence-summary does not accept extra arguments"
    );
}

#[test]
fn comparison_and_benchmark_corpus_summary_commands_render_the_corpus_blocks() {
    assert_eq!(
        validated_comparison_corpus_release_guard_summary_for_report(),
        Ok("Pluto excluded from tolerance evidence")
    );

    let comparison = render_cli(&["comparison-corpus-summary"])
        .expect("comparison corpus summary should render");
    assert!(comparison.contains("Comparison corpus summary"));
    assert!(comparison.contains("name: JPL Horizons release-grade comparison window"));
    assert!(comparison.contains("release-grade guard: Pluto excluded from tolerance evidence"));
    assert_eq!(comparison, render_comparison_corpus_summary_text());

    let guard = render_cli(&["comparison-corpus-release-guard-summary"])
        .expect("comparison corpus release guard summary should render");
    assert!(guard.contains("Comparison corpus release-grade guard summary"));
    assert!(guard.contains("Release-grade guard: Pluto excluded from tolerance evidence"));
    assert_eq!(guard, render_comparison_corpus_release_guard_summary_text());
    assert_eq!(
        render_cli(&["comparison-corpus"]).expect("comparison corpus alias should render"),
        comparison
    );
    assert_eq!(
        render_cli(&["comparison-corpus"])
            .expect("comparison corpus alias should match canonical rendering"),
        render_comparison_corpus_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-corpus-release-guard"])
            .expect("comparison corpus release guard short alias should render"),
        guard
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard-summary"])
            .expect("comparison corpus guard alias should render"),
        guard
    );
    assert_eq!(
        render_cli(&["comparison-corpus-guard"])
            .expect("comparison corpus guard short alias should render"),
        guard
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

    let comparison_envelope = render_cli(&["comparison-envelope-summary"])
        .expect("comparison envelope summary should render");
    assert!(comparison_envelope.contains("Comparison envelope summary"));
    assert_eq!(
        comparison_envelope,
        render_comparison_envelope_summary_text()
    );

    let comparison_envelope_alias =
        render_cli(&["comparison-envelope"]).expect("comparison envelope alias should render");
    assert_eq!(comparison_envelope_alias, comparison_envelope);

    let release_body_claims_summary = render_cli(&["release-body-claims-summary"])
        .expect("release body claims summary should render");
    assert_eq!(
        release_body_claims_summary,
        render_release_body_claims_summary_text()
    );
    assert!(release_body_claims_summary.contains(
            "Release-grade body claims: Moon and supported lunar points (Mean Node, True Node, Mean Apogee, Mean Perigee) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported; Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies"
        ));
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
        render_body_date_channel_claims_summary_text()
    );
    assert!(body_date_channel_claims_summary.contains("Body/date/channel claims: bodies="));
    assert!(body_date_channel_claims_summary
        .contains("frame policy=ecliptic body positions are the default request shape"));
    assert!(body_date_channel_claims_summary
        .contains("date range=JD 2268932.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(body_date_channel_claims_summary
        .contains("production generation coverage=Production generation coverage:"));
    assert!(body_date_channel_claims_summary.contains(
            "coverage posture=production-generation coverage and corpus shape remain aligned across the advertised 1600-2600 CE window; coverage="
        ));
    assert!(body_date_channel_claims_summary.contains("body-class coverage=major bodies:"));
    let source_corpus_summary =
        source_corpus_summary_details().expect("source corpus summary should exist");
    let body_date_channel_claims_details =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    assert_eq!(
        body_date_channel_claims_details.production_generation_date_range,
        source_corpus_summary.production_generation_date_range
    );
    assert_eq!(
        body_date_channel_claims_details.coverage_posture,
        source_corpus_summary.coverage_posture
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
    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims
        .release_body_claims
        .push_str(" drifted");
    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("release body claims drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `release_body_claims` is out of sync with the current posture"
        );
    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims.frame_policy.push_str(" drifted");
    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("frame policy drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `frame_policy` is out of sync with the current posture"
        );

    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims
        .production_generation_date_range
        .push_str(" drifted");
    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("production generation date range drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `production_generation_date_range` is out of sync with the current posture"
        );

    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims
        .production_generation_coverage
        .push_str(" drifted");
    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("production generation coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `production_generation_coverage` is out of sync with the current posture"
        );

    let pluto_fallback_summary =
        render_cli(&["pluto-fallback-summary"]).expect("Pluto fallback summary should render");
    assert_eq!(pluto_fallback_summary, render_pluto_fallback_summary_text());
    assert!(pluto_fallback_summary.contains(
            "Release-grade body claims: Moon and supported lunar points (Mean Node, True Node, Mean Apogee, Mean Perigee) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported; Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies"
        ));
    assert!(pluto_fallback_summary.contains(
            "Pluto fallback policy: Pluto remains an explicitly approximate fallback; release-grade major-body claims exclude Pluto"
        ));
    let release_body_claims_line = release_body_claims_summary_for_report()
        .validated_summary_line()
        .expect("release body claims posture should validate");
    let pluto_fallback_line = pluto_fallback_summary_for_report()
        .validated_summary_line()
        .expect("Pluto fallback posture should validate");
    assert_eq!(
        validate_release_body_claims_posture(release_body_claims_line, pluto_fallback_line,),
        Ok(())
    );
    assert!(validate_release_body_claims_posture(
            &release_body_claims_line.replace("Pluto remains an explicitly approximate fallback; ", ""),
            "Pluto remains an explicitly approximate fallback; release-grade major-body claims include Pluto",
        )
        .is_err());
    assert_eq!(
        render_cli(&["pluto-fallback"]).expect("Pluto fallback alias should render"),
        pluto_fallback_summary
    );
    assert_eq!(
        render_cli(&["pluto-fallback", "extra"])
            .expect_err("Pluto fallback alias should reject extra arguments"),
        "pluto-fallback does not accept extra arguments"
    );

    let comparison_tolerance_policy_error = render_cli(&["comparison-tolerance-summary", "extra"])
        .expect_err("comparison tolerance alias should reject extra arguments");
    assert_eq!(
        comparison_tolerance_policy_error,
        "comparison-tolerance-summary does not accept extra arguments"
    );

    let comparison_envelope_error = render_cli(&["comparison-envelope", "extra"])
        .expect_err("comparison envelope alias should reject extra arguments");
    assert_eq!(
        comparison_envelope_error,
        "comparison-envelope does not accept extra arguments"
    );

    let benchmark_corpus = benchmark_corpus();
    let benchmark_summary = benchmark_corpus.summary();
    assert_eq!(benchmark_summary.epoch_count, 11);
    assert_eq!(benchmark_summary.earliest_julian_day, 2_268_559.0);
    assert_eq!(benchmark_summary.latest_julian_day, 2_634_532.0);
    assert!(benchmark_summary
        .epochs
        .iter()
        .any(|instant| instant.julian_day.days() == 2_451_545.0));
    assert!(benchmark_summary
        .epochs
        .iter()
        .any(|instant| instant.julian_day.days() == 2_634_167.0));

    let benchmark =
        render_cli(&["benchmark-corpus-summary"]).expect("benchmark corpus summary should render");
    assert!(benchmark.contains("Benchmark corpus summary"));
    assert!(benchmark.contains("name: Representative 1600-2600 window"));
    assert!(benchmark.contains("epoch labels: JD 2268559.0 (TT)"));
    assert!(benchmark.contains("JD 2451545.0 (TT)"));
    assert!(benchmark.contains("JD 2634532.0 (TT)"));
    assert_eq!(benchmark, render_benchmark_corpus_summary_text());
    assert_eq!(
        benchmark,
        validated_benchmark_corpus_summary_for_report()
            .expect("benchmark corpus summary should validate")
    );
    assert!(ensure_benchmark_corpus_summary_matches_current_rendering(&benchmark).is_ok());
    let chart_benchmark = render_cli(&["chart-benchmark-corpus-summary"])
        .expect("chart benchmark corpus summary should render");
    assert!(chart_benchmark.contains("Chart benchmark corpus summary"));
    assert!(chart_benchmark.contains("name: Representative chart validation scenarios"));
    assert!(chart_benchmark.contains("requests: 9"));
    assert!(chart_benchmark.contains("epochs: 1"));
    assert!(chart_benchmark.contains("epoch labels: JD 2451545.0 (TT)"));
    assert!(chart_benchmark.contains("bodies: 10"));
    assert_eq!(
        chart_benchmark,
        render_chart_benchmark_corpus_summary_text()
    );
    assert_eq!(
        chart_benchmark,
        validated_chart_benchmark_corpus_summary_for_report()
            .expect("chart benchmark corpus summary should validate")
    );
    assert!(
        ensure_chart_benchmark_corpus_summary_matches_current_rendering(&chart_benchmark).is_ok()
    );
    let drifted_chart_benchmark = chart_benchmark.replace(
        "Chart benchmark corpus summary",
        "Drifted chart benchmark corpus summary",
    );
    let error =
        ensure_chart_benchmark_corpus_summary_matches_current_rendering(&drifted_chart_benchmark)
            .expect_err("chart benchmark corpus summary drift should be rejected");
    assert!(error
        .to_string()
        .contains("chart benchmark corpus summary no longer matches"));
    let drifted_benchmark = benchmark.replace(
        "Benchmark corpus summary",
        "Drifted benchmark corpus summary",
    );
    let error = ensure_benchmark_corpus_summary_matches_current_rendering(&drifted_benchmark)
        .expect_err("benchmark corpus summary drift should be rejected");
    assert!(error
        .to_string()
        .contains("benchmark corpus summary no longer matches"));
}

#[test]
fn manifest_summary_commands_render_the_matching_blocks() {
    let comparison = render_cli(&["comparison-snapshot-manifest-summary"])
        .expect("comparison snapshot manifest summary should render");
    assert!(comparison.contains("Comparison snapshot manifest:"));
    assert_eq!(
        comparison,
        validated_comparison_snapshot_manifest_summary_for_report()
            .expect("comparison snapshot manifest summary should validate")
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-manifest"])
            .expect("comparison snapshot manifest alias should render"),
        comparison
    );
    assert_eq!(
        render_cli(&["comparison-snapshot-manifest", "extra"])
            .expect_err("comparison snapshot manifest alias should reject extra arguments"),
        "comparison-snapshot-manifest does not accept extra arguments"
    );

    let reference = render_cli(&["reference-snapshot-manifest-summary"])
        .expect("reference snapshot manifest summary should render");
    assert!(reference.contains("Reference snapshot manifest:"));
    assert_eq!(reference, reference_snapshot_manifest_summary_for_report());
    assert_eq!(
        render_cli(&["reference-snapshot-manifest"])
            .expect("reference snapshot manifest alias should render"),
        reference
    );
    assert_eq!(
        render_cli(&["reference-snapshot-manifest", "extra"])
            .expect_err("reference snapshot manifest alias should reject extra arguments"),
        "reference-snapshot-manifest does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_batch_and_equatorial_parity_aliases_render_the_same_reports() {
    let batch = render_cli(&["reference-snapshot-batch-parity"])
        .expect("reference snapshot batch parity alias should render");
    assert_eq!(batch, reference_snapshot_batch_parity_summary_text());
    assert_eq!(
        render_cli(&["reference-snapshot-batch-parity", "extra"])
            .expect_err("reference snapshot batch parity alias should reject extra arguments"),
        "reference-snapshot-batch-parity-summary does not accept extra arguments"
    );

    let equatorial = render_cli(&["reference-snapshot-equatorial-parity"])
        .expect("reference snapshot equatorial parity alias should render");
    assert_eq!(
        equatorial,
        reference_snapshot_equatorial_parity_summary_for_report()
    );
    assert_eq!(
        render_cli(&["reference-snapshot-equatorial-parity", "extra"])
            .expect_err("reference snapshot equatorial parity alias should reject extra arguments"),
        "reference-snapshot-equatorial-parity-summary does not accept extra arguments"
    );
}

#[test]
fn comparison_snapshot_batch_parity_alias_renders_the_same_report() {
    let batch = render_cli(&["comparison-snapshot-batch-parity"])
        .expect("comparison snapshot batch parity alias should render");
    assert_eq!(batch, comparison_snapshot_batch_parity_summary_text());
    assert_eq!(
        render_cli(&["comparison-snapshot-batch-parity", "extra"])
            .expect_err("comparison snapshot batch parity alias should reject extra arguments"),
        "comparison-snapshot-batch-parity-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_source_window_summary_command_renders_the_source_windows_block() {
    let rendered = render_cli(&["reference-snapshot-source-window-summary"])
        .expect("reference snapshot source window summary should render");

    assert!(rendered.contains("Reference snapshot source windows:"));
    assert!(rendered.contains("source-backed samples"));
    assert_eq!(
        rendered,
        reference_snapshot_source_window_summary_for_report()
    );

    let alias = render_cli(&["reference-snapshot-source-window"])
        .expect("reference snapshot source window alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&["reference-snapshot-source-window", "extra"])
            .expect_err("reference snapshot source window alias should reject extra arguments"),
        "reference-snapshot-source-window does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_lunar_boundary_summary_command_renders_the_lunar_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-lunar-boundary-summary"])
        .expect("reference snapshot lunar boundary summary should render");

    assert!(rendered.contains("Reference lunar boundary evidence:"));
    assert_eq!(
        rendered,
        reference_snapshot_lunar_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_1500_selected_body_boundary_summary_command_renders_the_early_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-1500-selected-body-boundary-summary"])
        .expect("reference snapshot 1500 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 1500 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2268932.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_1500_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["1500-selected-body-boundary-summary"])
        .expect("1500 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1500_selected_body_boundary_summary_for_report()
    );

    let epoch_alias = render_cli(&["2268932-selected-body-boundary-summary"])
        .expect("2268932 selected-body boundary summary alias should render");
    assert_eq!(
        epoch_alias,
        reference_snapshot_2268932_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        reference_snapshot_2268932_selected_body_boundary_summary_for_report(),
        reference_snapshot_1500_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2268932-selected-body-boundary-summary", "extra"]).expect_err(
            "2268932 selected-body boundary summary alias should reject extra arguments"
        ),
        "reference-snapshot-2268932-selected-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_1600_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-1600-selected-body-boundary-summary"])
        .expect("reference snapshot 1600 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 1600 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2305457.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus, Mars, Jupiter, Uranus, Neptune"));
    assert_eq!(
        rendered,
        reference_snapshot_1600_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["1600-selected-body-boundary-summary"])
        .expect("1600 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1600_selected_body_boundary_summary_for_report()
    );

    let epoch_alias = render_cli(&["2305457-selected-body-boundary-summary"])
        .expect("2305457 selected-body boundary summary alias should render");
    assert_eq!(
        epoch_alias,
        reference_snapshot_2305457_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        reference_snapshot_2305457_selected_body_boundary_summary_for_report(),
        reference_snapshot_1600_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2305457-selected-body-boundary-summary", "extra"]).expect_err(
            "2305457 selected-body boundary summary alias should reject extra arguments"
        ),
        "reference-snapshot-2305457-selected-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_1750_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-1750-selected-body-boundary-summary"])
        .expect("reference snapshot 1750 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 1750 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2360234.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
    assert_eq!(
        rendered,
        reference_snapshot_1750_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["1750-selected-body-boundary-summary"])
        .expect("1750 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1750_selected_body_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_1750_major_body_interior_summary_command_renders_the_interior_block() {
    let rendered = render_cli(&["reference-snapshot-1750-major-body-interior-summary"])
        .expect("reference snapshot 1750 major-body interior summary should render");

    assert!(rendered.contains("Reference 1750 major-body interior comparison evidence:"));
    assert!(rendered.contains("JD 2360234.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
    assert_eq!(
        rendered,
        reference_snapshot_1750_major_body_interior_summary_for_report()
    );

    let alias = render_cli(&["1750-major-body-interior-summary"])
        .expect("1750 major-body interior summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1750_major_body_interior_summary_for_report()
    );
}

#[test]
fn reference_snapshot_2200_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-2200-selected-body-boundary-summary"])
        .expect("reference snapshot 2200 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 2200 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2524593.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_2200_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["2200-selected-body-boundary-summary"])
        .expect("2200 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_2200_selected_body_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_1900_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-1900-selected-body-boundary-summary"])
        .expect("reference snapshot 1900 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 1900 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2415020.5 (TDB)"));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_1900_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["1900-selected-body-boundary-summary"])
        .expect("1900 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_1900_selected_body_boundary_summary_for_report()
    );

    let epoch_alias = render_cli(&["2415020-selected-body-boundary-summary"])
        .expect("2415020 selected-body boundary summary alias should render");
    assert_eq!(
        epoch_alias,
        reference_snapshot_2415020_selected_body_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2415020-selected-body-boundary-summary",
            "extra"
        ])
        .expect_err("2415020 selected-body boundary summary should reject extra arguments"),
        "reference-snapshot-2415020-selected-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2415020-selected-body-boundary-summary", "extra"]).expect_err(
            "2415020 selected-body boundary summary alias should reject extra arguments"
        ),
        "reference-snapshot-2415020-selected-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2500_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-2500-selected-body-boundary-summary"])
        .expect("reference snapshot 2500 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 2500 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2634167.0 (TDB)"));
    assert!(rendered.contains("Mars, Mercury, Moon, Sun, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_2500_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["2500-selected-body-boundary-summary"])
        .expect("2500 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_2500_selected_body_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_2634167_selected_body_boundary_summary_command_renders_the_selected_body_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-2634167-selected-body-boundary-summary"])
        .expect("reference snapshot 2634167 selected-body boundary summary should render");

    assert!(rendered.contains("Reference 2634167 selected-body boundary evidence:"));
    assert!(rendered.contains("JD 2634167.0 (TDB)"));
    assert!(rendered.contains("Mars, Mercury, Moon, Sun, Venus"));
    assert_eq!(
        rendered,
        reference_snapshot_2634167_selected_body_boundary_summary_for_report()
    );

    let alias = render_cli(&["2634167-selected-body-boundary-summary"])
        .expect("2634167 selected-body boundary summary alias should render");
    assert_eq!(
        alias,
        reference_snapshot_2634167_selected_body_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_1749_major_body_boundary_summary_command_renders_the_1749_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-1749-major-body-boundary-summary"])
        .expect("reference snapshot 1749 major-body boundary summary should render");

    assert!(rendered.contains("Reference 1749 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2360233.5 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_1749_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["1749-major-body-boundary-summary"])
        .expect("1749 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    let epoch_alias = render_cli(&["2360233-major-body-boundary-summary"])
        .expect("2360233 major-body boundary alias should render");
    assert_eq!(epoch_alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-1749-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 1749 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-1749-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["1749-major-body-boundary-summary", "extra"])
            .expect_err("1749 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-1749-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2360233-major-body-boundary-summary", "extra"])
            .expect_err("2360233 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-1749-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_early_major_body_boundary_summary_command_renders_the_early_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-early-major-body-boundary-summary"])
        .expect("reference snapshot early major-body boundary summary should render");

    assert!(rendered.contains("Reference early major-body boundary evidence:"));
    assert_eq!(
        rendered,
        reference_snapshot_early_major_body_boundary_summary_for_report()
    );
    let exact_jd_alias = render_cli(&["2378498-major-body-boundary-summary"])
        .expect("2378498 major-body boundary alias should render");
    assert_eq!(
        exact_jd_alias,
        reference_snapshot_2378498_major_body_boundary_summary_for_report()
    );
    assert_eq!(
        exact_jd_alias,
        rendered.replace(
            "Reference early major-body boundary evidence",
            "Reference 2378498 major-body boundary evidence"
        )
    );
    let alias = render_cli(&["early-major-body-boundary-summary"])
        .expect("early major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&["2378498-major-body-boundary-summary", "extra"])
            .expect_err("2378498 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2378498-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&[
            "reference-snapshot-early-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot early major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-early-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["early-major-body-boundary-summary", "extra"])
            .expect_err("early major-body boundary alias should reject extra arguments"),
        "reference-snapshot-early-major-body-boundary-summary does not accept extra arguments"
    );

    let reference_snapshot_1800_major_body_boundary_summary =
        render_cli(&["reference-snapshot-1800-major-body-boundary-summary"])
            .expect("reference snapshot 1800 major-body boundary summary should render");

    assert!(reference_snapshot_1800_major_body_boundary_summary
        .contains("Reference 1800 major-body boundary evidence:"));
    assert_eq!(
        reference_snapshot_1800_major_body_boundary_summary,
        reference_snapshot_1800_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["1800-major-body-boundary-summary"])
        .expect("1800 major-body boundary alias should render");
    assert_eq!(alias, reference_snapshot_1800_major_body_boundary_summary);
    let epoch_alias = render_cli(&["2378499-major-body-boundary-summary"])
        .expect("2378499 major-body boundary alias should render");
    assert_eq!(
        epoch_alias,
        reference_snapshot_1800_major_body_boundary_summary
    );
    assert_eq!(
        render_cli(&[
            "reference-snapshot-1800-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 1800 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-1800-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["1800-major-body-boundary-summary", "extra"])
            .expect_err("1800 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-1800-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2378499-major-body-boundary-summary", "extra"])
            .expect_err("2378499 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-1800-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2500_major_body_boundary_summary_command_renders_the_terminal_boundary_block()
{
    let rendered = render_cli(&["reference-snapshot-2500-major-body-boundary-summary"])
        .expect("reference snapshot 2500 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2500 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2500000.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2500_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2500-major-body-boundary-summary"])
        .expect("2500 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2500-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2500 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2500-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2500-major-body-boundary-summary", "extra"])
            .expect_err("2500 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2500-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2400000_major_body_boundary_summary_command_renders_the_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-2400000-major-body-boundary-summary"])
        .expect("reference snapshot 2400000 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2400000 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2400000.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2400000_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2400000-major-body-boundary-summary"])
        .expect("2400000 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2400000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2400000 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2400000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2400000-major-body-boundary-summary", "extra"])
            .expect_err("2400000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2400000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451545_major_body_boundary_summary_command_renders_the_j2000_block() {
    let rendered = render_cli(&["reference-snapshot-2451545-major-body-boundary-summary"])
        .expect("reference snapshot 2451545 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2451545 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2451545.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2451545_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2451545-major-body-boundary-summary"])
        .expect("2451545 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451545-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2451545 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2451545-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451545-major-body-boundary-summary", "extra"])
            .expect_err("2451545 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451545-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2453000_major_body_boundary_summary_command_renders_the_late_boundary_block()
{
    let rendered = render_cli(&["reference-snapshot-2453000-major-body-boundary-summary"])
        .expect("reference snapshot 2453000 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2453000 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2453000.5 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2453000_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2453000-major-body-boundary-summary"])
        .expect("2453000 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2453000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2453000 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2453000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2453000-major-body-boundary-summary", "extra"])
            .expect_err("2453000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2453000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2500000_major_body_boundary_summary_command_renders_the_terminal_boundary_block(
) {
    let rendered = render_cli(&["reference-snapshot-2500000-major-body-boundary-summary"])
        .expect("reference snapshot 2500000 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2500000 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2500000.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2500000_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2500000-major-body-boundary-summary"])
        .expect("2500000 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2500000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2500000 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2500000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2500000-major-body-boundary-summary", "extra"])
            .expect_err("2500000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2500000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2600000_major_body_boundary_summary_command_renders_the_outer_boundary_block()
{
    let rendered = render_cli(&["reference-snapshot-2600000-major-body-boundary-summary"])
        .expect("reference snapshot 2600000 major-body boundary summary should render");

    assert!(rendered.contains("Reference 2600000 major-body boundary evidence:"));
    assert!(rendered.contains("JD 2600000.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_2600000_major_body_boundary_summary_for_report()
    );
    let alias = render_cli(&["2600000-major-body-boundary-summary"])
        .expect("2600000 major-body boundary alias should render");
    assert_eq!(alias, rendered);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2600000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err(
            "reference snapshot 2600000 major-body boundary summary should reject extra arguments"
        ),
        "reference-snapshot-2600000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2600000-major-body-boundary-summary", "extra"])
            .expect_err("2600000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2600000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451910_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451910 = render_cli(&["reference-snapshot-2451910-major-body-boundary-summary"])
        .expect("2451910 major-body boundary summary should render");
    assert!(boundary_2451910.contains("Reference 2451910 major-body boundary evidence:"));
    assert!(boundary_2451910.contains("JD 2451910.5 (TDB)"));
    let boundary_2451910_alias = render_cli(&["2451910-major-body-boundary-summary"])
        .expect("2451910 major-body boundary alias should render");
    assert_eq!(boundary_2451910_alias, boundary_2451910);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451910-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451910 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451910-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451910-major-body-boundary-summary", "extra"])
            .expect_err("2451910 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451910-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451911_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451911 = render_cli(&["reference-snapshot-2451911-major-body-boundary-summary"])
        .expect("2451911 major-body boundary summary should render");
    assert!(boundary_2451911.contains("Reference 2451911 major-body boundary evidence:"));
    assert!(boundary_2451911.contains("JD 2451911.5 (TDB)"));
    let boundary_2451911_alias = render_cli(&["2451911-major-body-boundary-summary"])
        .expect("2451911 major-body boundary alias should render");
    assert_eq!(boundary_2451911_alias, boundary_2451911);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451911-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451911 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451911-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451911-major-body-boundary-summary", "extra"])
            .expect_err("2451911 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451911-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451912_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451912 = render_cli(&["reference-snapshot-2451912-major-body-boundary-summary"])
        .expect("2451912 major-body boundary summary should render");
    assert!(boundary_2451912.contains("Reference 2451912 major-body boundary evidence:"));
    assert!(boundary_2451912.contains("JD 2451912.5 (TDB)"));
    let boundary_2451912_alias = render_cli(&["2451912-major-body-boundary-summary"])
        .expect("2451912 major-body boundary alias should render");
    assert_eq!(boundary_2451912_alias, boundary_2451912);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451912-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451912 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451912-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451912-major-body-boundary-summary", "extra"])
            .expect_err("2451912 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451912-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451913_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451913 = render_cli(&["reference-snapshot-2451913-major-body-boundary-summary"])
        .expect("2451913 major-body boundary summary should render");
    assert!(boundary_2451913.contains("Reference 2451913 major-body boundary evidence:"));
    assert!(boundary_2451913.contains("JD 2451913.5 (TDB)"));
    let boundary_2451913_alias = render_cli(&["2451913-major-body-boundary-summary"])
        .expect("2451913 major-body boundary alias should render");
    assert_eq!(boundary_2451913_alias, boundary_2451913);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451913-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451913 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451913-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451913-major-body-boundary-summary", "extra"])
            .expect_err("2451913 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451913-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451914_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451914 = render_cli(&["reference-snapshot-2451914-major-body-boundary-summary"])
        .expect("2451914 major-body boundary summary should render");
    assert!(boundary_2451914.contains("Reference 2451914 major-body boundary evidence:"));
    assert!(boundary_2451914.contains("JD 2451914.5 (TDB)"));
    let boundary_2451914_alias = render_cli(&["2451914-major-body-boundary-summary"])
        .expect("2451914 major-body boundary alias should render");
    assert_eq!(boundary_2451914_alias, boundary_2451914);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451914-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451914 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451914-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451914-major-body-boundary-summary", "extra"])
            .expect_err("2451914 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451914-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451915_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451915 = render_cli(&["reference-snapshot-2451915-major-body-boundary-summary"])
        .expect("2451915 major-body boundary summary should render");
    assert!(boundary_2451915.contains("Reference 2451915 major-body boundary evidence:"));
    assert!(boundary_2451915.contains("JD 2451915.5 (TDB)"));
    let boundary_2451915_alias = render_cli(&["2451915-major-body-boundary-summary"])
        .expect("2451915 major-body boundary alias should render");
    assert_eq!(boundary_2451915_alias, boundary_2451915);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451915-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451915 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451915-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451915-major-body-boundary-summary", "extra"])
            .expect_err("2451915 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451915-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451916_major_body_interior_summary_aliases_render_the_same_reports() {
    let interior_2451916 = render_cli(&["reference-snapshot-2451916-major-body-interior-summary"])
        .expect("2451916 major-body interior summary should render");
    assert!(interior_2451916.contains("Reference 2451916 major-body interior evidence:"));
    assert!(interior_2451916.contains("JD 2451916.0 (TDB)"));
    let interior_2451916_alias = render_cli(&["2451916-major-body-interior-summary"])
        .expect("2451916 major-body interior alias should render");
    assert_eq!(interior_2451916_alias, interior_2451916);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451916-major-body-interior-summary",
            "extra"
        ])
        .expect_err("2451916 major-body interior summary should reject extra arguments"),
        "reference-snapshot-2451916-major-body-interior-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451916-major-body-interior-summary", "extra"])
            .expect_err("2451916 major-body interior alias should reject extra arguments"),
        "reference-snapshot-2451916-major-body-interior-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451917_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451917 = render_cli(&["reference-snapshot-2451917-major-body-boundary-summary"])
        .expect("2451917 major-body boundary summary should render");
    assert!(boundary_2451917.contains("Reference 2451917 major-body boundary evidence:"));
    assert!(boundary_2451917.contains("JD 2451917.5 (TDB)"));
    let boundary_2451917_alias = render_cli(&["2451917-major-body-boundary-summary"])
        .expect("2451917 major-body boundary alias should render");
    assert_eq!(boundary_2451917_alias, boundary_2451917);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451917-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451917 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451917-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451917-major-body-boundary-summary", "extra"])
            .expect_err("2451917 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451917-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451918_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451918 = render_cli(&["reference-snapshot-2451918-major-body-boundary-summary"])
        .expect("2451918 major-body boundary summary should render");
    assert!(boundary_2451918.contains("Reference 2451918 major-body boundary evidence:"));
    assert!(boundary_2451918.contains("JD 2451918.5 (TDB)"));
    let boundary_2451918_alias = render_cli(&["2451918-major-body-boundary-summary"])
        .expect("2451918 major-body boundary alias should render");
    assert_eq!(boundary_2451918_alias, boundary_2451918);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451918-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451918 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451918-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451918-major-body-boundary-summary", "extra"])
            .expect_err("2451918 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451918-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451919_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2451919 = render_cli(&["reference-snapshot-2451919-major-body-boundary-summary"])
        .expect("2451919 major-body boundary summary should render");
    assert!(boundary_2451919.contains("Reference 2451919 major-body boundary evidence:"));
    assert!(boundary_2451919.contains("JD 2451919.5 (TDB)"));
    let boundary_2451919_alias = render_cli(&["2451919-major-body-boundary-summary"])
        .expect("2451919 major-body boundary alias should render");
    assert_eq!(boundary_2451919_alias, boundary_2451919);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451919-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451919 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451919-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451919-major-body-boundary-summary", "extra"])
            .expect_err("2451919 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451919-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451920_major_body_interior_summary_aliases_render_the_same_reports() {
    let interior_2451920 = render_cli(&["reference-snapshot-2451920-major-body-interior-summary"])
        .expect("2451920 major-body interior summary should render");
    assert!(interior_2451920.contains("Reference 2451920 major-body interior evidence:"));
    assert!(interior_2451920.contains("JD 2451920.5 (TDB)"));
    let interior_2451920_alias = render_cli(&["2451920-major-body-interior-summary"])
        .expect("2451920 major-body interior alias should render");
    assert_eq!(interior_2451920_alias, interior_2451920);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451920-major-body-interior-summary",
            "extra"
        ])
        .expect_err("2451920 major-body interior summary should reject extra arguments"),
        "reference-snapshot-2451920-major-body-interior-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451920-major-body-interior-summary", "extra"])
            .expect_err("2451920 major-body interior alias should reject extra arguments"),
        "reference-snapshot-2451920-major-body-interior-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2360234_major_body_interior_summary_aliases_render_the_same_reports() {
    let interior_2360234 = render_cli(&["reference-snapshot-2360234-major-body-interior-summary"])
        .expect("2360234 major-body interior comparison summary should render");
    assert!(interior_2360234.contains("Reference 2360234 major-body interior comparison evidence:"));
    assert!(interior_2360234.contains("JD 2360234.5 (TDB)"));
    let interior_2360234_alias = render_cli(&["2360234-major-body-interior-summary"])
        .expect("2360234 major-body interior comparison alias should render");
    assert_eq!(interior_2360234_alias, interior_2360234);
    assert_eq!(
            render_cli(&["reference-snapshot-2360234-major-body-interior-summary", "extra"])
                .expect_err(
                    "2360234 major-body interior comparison summary should reject extra arguments",
                ),
            "reference-snapshot-2360234-major-body-interior-summary does not accept extra arguments"
        );
    assert_eq!(
        render_cli(&["2360234-major-body-interior-summary", "extra"]).expect_err(
            "2360234 major-body interior comparison alias should reject extra arguments",
        ),
        "reference-snapshot-2360234-major-body-interior-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_mars_outer_boundary_summary_command_renders_the_outer_boundary_block() {
    let rendered = render_cli(&["reference-snapshot-mars-outer-boundary-summary"])
        .expect("reference snapshot Mars outer-boundary summary should render");

    assert!(rendered.contains("Reference Mars outer-boundary evidence:"));
    assert!(rendered.contains("JD 2600000.0 (TDB)"));
    assert!(rendered.contains("JD 2634167.0 (TDB)"));
    assert_eq!(
        rendered,
        reference_snapshot_mars_outer_boundary_summary_for_report()
    );
    let alias = render_cli(&["mars-outer-boundary-summary"])
        .expect("Mars outer-boundary alias should render");
    assert_eq!(alias, rendered);
}

#[test]
fn reference_snapshot_2600000_major_body_boundary_summary_aliases_render_the_same_reports() {
    let boundary_2600000 = render_cli(&["reference-snapshot-2600000-major-body-boundary-summary"])
        .expect("2600000 major-body boundary summary should render");
    assert!(boundary_2600000.contains("Reference 2600000 major-body boundary evidence:"));
    assert!(boundary_2600000.contains("JD 2600000.0 (TDB)"));
    let boundary_2600000_alias = render_cli(&["2600000-major-body-boundary-summary"])
        .expect("2600000 major-body boundary alias should render");
    assert_eq!(boundary_2600000_alias, boundary_2600000);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2600000-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2600000 major-body boundary summary should reject extra arguments",),
        "reference-snapshot-2600000-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2600000-major-body-boundary-summary", "extra"])
            .expect_err("2600000 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2600000-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn source_corpus_summary_aliases_render_the_same_report() {
    let rendered =
        render_cli(&["source-corpus-summary"]).expect("source corpus summary should render");
    assert_eq!(rendered, source_corpus_summary_for_report());
    assert!(rendered.contains("shared schema=epoch_jd, body, x_km, y_km, z_km"));
    assert!(rendered.contains("generation command=generate-packaged-artifact --check"));
    assert!(
        rendered.contains("production generation source=strategy=documented hybrid fixture corpus")
    );
    assert!(rendered.contains("license posture=public-source provenance only; checked-in fixtures remain repository-local regression data"));
    assert!(rendered.contains(
        "redistribution posture=repository-checked regression fixtures, not a broad public corpus"
    ));
    assert!(rendered.contains("schema=epoch_jd, body, x_km, y_km, z_km"));
    assert!(rendered.contains("production generation source revision=source revision="));
    assert!(rendered.contains("reference_snapshot.csv checksum=0x"));
    assert!(rendered.contains("independent_holdout_snapshot.csv checksum=0x"));
    assert!(rendered.contains("production generation source windows=357 source-backed samples across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB))"));
    assert!(rendered.contains("source density floors=reference major bodies:"));
    assert!(rendered.contains("production generation body-class coverage=major bodies: 262 rows across 10 bodies and 31 epochs"));
    assert!(rendered
        .contains("production generation date range=JD 2268932.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(rendered.contains("production generation quarter-day boundary samples=8 rows across 4 bodies and 2 epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB))"));
    assert!(rendered.contains("coverage posture=production-generation coverage and corpus shape remain aligned across the advertised 1600-2600 CE window; coverage="));
    assert!(rendered.contains("body-class coverage=major bodies:"));
    assert!(rendered.contains("production generation boundary window="));
    assert!(rendered.contains("production generation boundary source="));
    assert!(rendered.contains("production generation boundary request corpus="));
    assert!(rendered.contains("production generation boundary request corpus equatorial="));
    assert!(rendered
        .contains("reference snapshot sparse boundary=16 exact samples at JD 2451915.5 (TDB)"));
    assert!(rendered.contains(
        "reference snapshot exact J2000 evidence=16 exact J2000 samples at JD 2451545.0 (TDB)"
    ));
    assert!(rendered.contains("reference snapshot equatorial parity=357 rows across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB))"));
    assert!(rendered.contains("reference snapshot body-class coverage=major bodies: 262 rows across 10 bodies and 31 epochs"));
    let reference_snapshot_manifest = required_summary_payload(
        reference_snapshot_manifest_summary_for_report(),
        "Reference snapshot manifest: ",
        "reference snapshot manifest",
    )
    .expect("reference snapshot manifest payload should be available");
    assert!(rendered.contains(&format!(
        "reference snapshot manifest={reference_snapshot_manifest}"
    )));
    assert!(rendered.contains(
        "independent-holdout body-class coverage=84 rows across 16 bodies and 14 epochs"
    ));
    assert!(rendered.contains("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"));
    assert!(rendered.contains("evidence classification=release-tolerance=reference/comparison/production-generation validation summaries; hold-out=independent hold-out rows and interpolation-quality summaries; fixture exactness=reference snapshot exact J2000 evidence; provenance-only=source and manifest summaries"));
    assert!(rendered.contains("provenance-only=source and manifest summaries are provenance-only evidence; they validate corpus provenance and checksum posture but are excluded from tolerance, hold-out, and fixture-exactness claims"));
    assert!(rendered
        .contains("lunar source windows=7 exact Moon samples across 1 bodies in 2 exact windows"));
    assert!(rendered.contains("release-grade body claims=Moon and supported lunar points (Mean Node, True Node, Mean Apogee, Mean Perigee) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported; Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies"));
    assert!(rendered.contains("date range=JD 2268932.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(rendered.contains("production generation coverage=Production generation coverage:"));
    assert!(rendered.contains(
            "coverage posture=production-generation coverage and corpus shape remain aligned across the advertised 1600-2600 CE window; coverage="
        ));
    assert!(rendered.contains("body-class coverage=major bodies:"));
    assert!(rendered.contains(
            "equatorial output is backend-specific and derived via mean-obliquity transforms when supported"
        ));
    assert!(rendered.contains("corpus shape=Production generation corpus shape:"));
    assert!(rendered
        .contains("provenance-only=source and manifest summaries are provenance-only evidence"));
    assert_eq!(
        render_cli(&["jpl-provenance-only-summary"])
            .expect("JPL provenance-only summary should render"),
        pleiades_jpl::jpl_provenance_only_summary_for_report()
    );
    assert_eq!(
        render_cli(&["jpl-provenance-only"]).expect("JPL provenance-only alias should render"),
        pleiades_jpl::jpl_provenance_only_summary_for_report()
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
    assert_eq!(
        render_cli(&["source-corpus"]).expect("source corpus alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["source-corpus", "extra"])
            .expect_err("source corpus alias should reject extra arguments"),
        "source-corpus does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["source-corpus-posture-summary"])
            .expect("source corpus posture summary should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["source-corpus-posture"]).expect("source corpus posture alias should render"),
        rendered
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
}

#[test]
fn jpl_source_posture_summary_aliases_render_the_same_report() {
    let rendered = render_cli(&["jpl-source-posture-summary"])
        .expect("JPL source posture summary should render");
    assert_eq!(
        rendered,
        pleiades_jpl::jpl_source_posture_summary_for_report()
    );
    assert_eq!(
        render_cli(&["jpl-source-posture"]).expect("JPL source posture alias should render"),
        pleiades_jpl::jpl_source_posture_summary_for_report()
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
}

#[test]
fn required_summary_payload_rejects_missing_prefixes() {
    let error = required_summary_payload(
        "drifted payload".to_string(),
        "expected prefix: ",
        "example field",
    )
    .expect_err("missing prefix should fail closed");
    assert_eq!(
        error,
        "source corpus summary field `example field` is out of sync with the current posture"
    );
}

#[test]
fn required_labelled_summary_payload_rejects_duplicate_prefixes() {
    let error = required_labelled_summary_payload(
        "JPL source corpus contract: JPL source corpus contract: nested drift".to_string(),
        "JPL source corpus contract: ",
        "JPL source corpus contract",
    )
    .expect_err("duplicate labelled prefixes should fail closed");
    assert_eq!(
            error,
            "source corpus summary field `JPL source corpus contract` is out of sync with the current posture"
        );
}

#[test]
fn source_corpus_summary_details_validate_field_drift() {
    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.shared_schema = "epoch_jd, body, x_km, y_km, z_km, drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("shared schema drift should fail closed");
    assert_eq!(
        error.to_string(),
        "the source corpus summary field `shared_schema` is out of sync with the current posture"
    );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.generation_command = "generate-packaged-artifact --check --drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("generation command drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `generation_command` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.comparison_corpus_release_grade_guard =
        "Comparison corpus release-grade guard: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("comparison corpus release-grade guard drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `comparison_corpus_release_grade_guard` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.jpl_source_corpus_contract = "JPL source corpus contract: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("jpl source corpus contract drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `jpl_source_corpus_contract` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.jpl_evidence_classification = "JPL evidence classification: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("jpl evidence classification drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `jpl_evidence_classification` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.jpl_provenance_only = "JPL provenance-only evidence: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("jpl provenance-only drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `jpl_provenance_only` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.lunar_source_window = "lunar source windows=drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("lunar source window drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `lunar_source_window` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_source = "Production generation source: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation source drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_source` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_source_revision = "source revision=drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation source revision drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_source_revision` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_coverage = "Production generation coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_source_windows =
        "Production generation source windows: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation source windows drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_source_windows` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_body_class_coverage =
        "Production generation body-class coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation body-class coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_body_class_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_boundary_window =
        "Production generation boundary windows: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation boundary window drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_boundary_window` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_boundary_source =
        "Production generation boundary overlay source: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation boundary source drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_boundary_source` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_boundary_request_corpus =
        "Production generation boundary request corpus: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation boundary request corpus drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_boundary_request_corpus` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_boundary_request_corpus_equatorial =
        "Production generation boundary request corpus: drifted".to_string();

    let error = summary.validated_summary_line().expect_err(
        "production generation boundary request corpus equatorial drift should fail closed",
    );
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_boundary_request_corpus_equatorial` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_date_range = "JD drifted..JD drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation date range drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_date_range` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.production_generation_quarter_day_boundary_samples =
        "8 rows across 4 bodies and 2 epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB))"
            .to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("production generation quarter-day boundary samples drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `production_generation_quarter_day_boundary_samples` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_sparse_boundary =
        "Reference snapshot boundary day: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot sparse boundary drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_sparse_boundary` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_exact_j2000_evidence =
        "Reference snapshot exact J2000 evidence: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot exact J2000 evidence drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_exact_j2000_evidence` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_exact_j2000_body_class_coverage =
        "Reference snapshot exact J2000 body-class coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot exact J2000 body-class coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_exact_j2000_body_class_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_equatorial_parity =
        "JPL reference snapshot equatorial parity: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot equatorial parity drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_equatorial_parity` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_body_class_coverage =
        "Reference snapshot body-class coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot body-class coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_body_class_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.reference_snapshot_manifest = "Reference snapshot manifest: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("reference snapshot manifest drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `reference_snapshot_manifest` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.independent_holdout_body_class_coverage =
        "Independent hold-out body-class coverage: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("independent hold-out body-class coverage drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `independent_holdout_body_class_coverage` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.independent_holdout_source_window =
        "Independent hold-out source windows: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("independent hold-out source window drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `independent_holdout_source_window` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.pluto_fallback = "Pluto fallback: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("pluto fallback drift should fail closed");
    assert_eq!(
        error.to_string(),
        "the source corpus summary field `pluto_fallback` is out of sync with the current posture"
    );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.release_grade_body_claims = "Release-grade body claims: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("release-grade body claims drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `release_grade_body_claims` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.body_date_channel_claims = "Body/date/channel claims: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("body/date/channel claims drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `body_date_channel_claims` is out of sync with the current posture"
        );

    let mut body_date_channel_claims =
        body_date_channel_claims_summary_details().expect("body/date/channel claims should exist");
    body_date_channel_claims.coverage_posture = "coverage posture=drifted".to_string();

    let error = body_date_channel_claims
        .validated_summary_line()
        .expect_err("body/date/channel coverage posture drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the body/date/channel claims summary field `coverage_posture` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.coverage_posture = "coverage posture=drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("coverage posture drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `coverage_posture` is out of sync with the current posture"
        );

    let mut summary = source_corpus_summary_details().expect("source corpus summary should exist");
    summary.phase2_corpus_alignment = "Phase-2 corpus alignment: drifted".to_string();

    let error = summary
        .validated_summary_line()
        .expect_err("phase-2 corpus alignment drift should fail closed");
    assert_eq!(
            error.to_string(),
            "the source corpus summary field `phase2_corpus_alignment` is out of sync with the current posture"
        );
}
