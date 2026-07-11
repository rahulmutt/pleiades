//! Tests for miscellaneous commands: unknown-command rejection, ayanamsa/lunar summaries,
//! fallback commands, chart help policy, and source-fit sync summaries.

use crate::cli::render_cli;
use crate::commands::chart::render_chart;
use crate::help::shared_request_policy_help_block;

#[test]
fn unknown_command_is_rejected() {
    let error =
        render_cli(&["compatibility-profile-snapshot"]).expect_err("unknown commands should fail");

    assert!(error.contains("unknown command: compatibility-profile-snapshot"));
    for expected in [
        "compare-backends",
        "compare-backends-audit",
        "compatibility-profile",
        "compatibility-caveats-summary",
        "verify-compatibility-profile",
        "bundle-release",
        "verify-release-bundle",
        "release-notes",
        "release-summary",
        "packaged-lookup-epoch-policy-summary",
        "validate-artifact",
        "workspace-audit",
        "report",
        "chart",
    ] {
        assert!(error.contains(expected), "missing help text for {expected}");
    }
}

#[test]
fn custom_definition_ayanamsa_labels_summary_command_renders_the_labels() {
    let rendered = render_cli(&["custom-definition-ayanamsa-labels-summary"])
        .expect("custom-definition ayanamsa labels summary should render");

    assert_eq!(
        rendered,
        pleiades_validate::render_cli(&["custom-definition-ayanamsa-labels-summary"]).expect(
            "validation front end should render the custom-definition ayanamsa labels summary"
        )
    );
    assert_eq!(
        render_cli(&["custom-definition-ayanamsa-labels"])
            .expect("custom-definition ayanamsa labels alias should render"),
        rendered
    );
}

#[test]
fn release_specific_canonical_name_summary_commands_render_the_labels() {
    let profile = pleiades_core::current_compatibility_profile();

    let house_names = render_cli(&["release-house-system-canonical-names-summary"])
        .expect("release-specific house-system canonical names summary should render");
    assert_eq!(
        house_names,
        pleiades_validate::render_cli(&["release-house-system-canonical-names-summary"]).expect(
            "validation front end should render the release-specific house-system canonical names summary"
        )
    );
    assert_eq!(
        render_cli(&["release-house-system-canonical-names"])
            .expect("release-specific house-system canonical names alias should render"),
        house_names
    );
    assert_eq!(
        house_names,
        format!(
            "Release-specific house-system canonical names: {}",
            profile
                .validated_release_house_system_canonical_names_summary_line()
                .expect("release-specific house-system canonical names should validate")
        )
    );

    let ayanamsa_names = render_cli(&["release-ayanamsa-canonical-names-summary"])
        .expect("release-specific ayanamsa canonical names summary should render");
    assert_eq!(
        ayanamsa_names,
        pleiades_validate::render_cli(&["release-ayanamsa-canonical-names-summary"]).expect(
            "validation front end should render the release-specific ayanamsa canonical names summary"
        )
    );
    assert_eq!(
        render_cli(&["release-ayanamsa-canonical-names"])
            .expect("release-specific ayanamsa canonical names alias should render"),
        ayanamsa_names
    );
    assert_eq!(
        ayanamsa_names,
        format!(
            "Release-specific ayanamsa canonical names: {}",
            profile
                .validated_release_ayanamsa_canonical_names_summary_line()
                .expect("release-specific ayanamsa canonical names should validate")
        )
    );
}

#[test]
fn ayanamsa_audit_summary_command_renders_the_summary() {
    let rendered = render_cli(&["ayanamsa-audit-summary"])
        .expect("ayanamsa audit summary should render through the CLI");
    assert_eq!(
        rendered,
        pleiades_validate::render_cli(&["ayanamsa-audit-summary"])
            .expect("validation front end should render the ayanamsa audit summary")
    );
    assert_eq!(render_cli(&["ayanamsa-audit"]).unwrap(), rendered);
    assert!(rendered.contains("Ayanamsa audit: ayanamsa catalog validation:"));
    assert!(rendered.contains("ayanamsa sidereal metadata:"));
    assert!(rendered.contains("Ayanamsa reference offsets:"));
    assert!(rendered.contains("Ayanamsa provenance:"));
}

#[test]
fn lunar_theory_source_selection_summary_command_renders_the_summary() {
    let rendered = render_cli(&["lunar-theory-source-selection-summary"])
        .expect("lunar theory source selection summary should render through the CLI");
    assert_eq!(
        rendered,
        pleiades_validate::render_cli(&["lunar-theory-source-selection-summary"])
            .expect("validation front end should render the lunar theory source selection summary")
    );
    assert_eq!(
        render_cli(&["lunar-theory-source-selection"]).unwrap(),
        rendered
    );
    assert_eq!(
        render_cli(&["lunar-theory-source-selection", "extra"])
            .expect_err("lunar theory source selection alias should reject extra arguments"),
        "lunar-theory-source-selection-summary does not accept extra arguments"
    );
    assert!(rendered.contains("lunar source selection:"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn fallback_summary_commands_remain_reachable_from_the_cli() {
    for (cli_args, validation_args) in [
        (
            &["benchmark-matrix", "--rounds", "1"][..],
            &["benchmark-matrix-summary", "--rounds", "1"][..],
        ),
        (&["catalog-posture"][..], &["catalog-posture-summary"][..]),
        (&["known-gaps"][..], &["known-gaps-summary"][..]),
        (
            &["jpl-provenance-only"][..],
            &["jpl-provenance-only-summary"][..],
        ),
        (
            &["production-generation"][..],
            &["production-generation-summary"][..],
        ),
        (
            &["production-generation-manifest"][..],
            &["production-generation-manifest-summary"][..],
        ),
        (
            &["production-generation-manifest-checksum"][..],
            &["production-generation-manifest-checksum-summary"][..],
        ),
        (
            &["production-generation-source-revision"][..],
            &["production-generation-source-revision-summary"][..],
        ),
    ] {
        assert_eq!(
            render_cli(cli_args)
                .unwrap_or_else(|error| panic!("{cli_args:?} should render: {error}")),
            pleiades_validate::render_cli(validation_args).unwrap_or_else(|error| {
                panic!("validation command {validation_args:?} should render: {error}")
            }),
            "CLI fallback should keep {cli_args:?} aligned with {validation_args:?}"
        );
    }
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_and_ayanamsa_audit_summary_commands_render_directly_from_the_cli() {
    for (summary_args, alias_args) in [
        (&["ayanamsa-audit-summary"][..], &["ayanamsa-audit"][..]),
        (
            &["lunar-reference-evidence-summary"][..],
            &["lunar-reference-evidence"][..],
        ),
        (
            &["packaged-artifact-body-cadence-summary"][..],
            &["packaged-artifact-body-cadence"][..],
        ),
        (
            &["packaged-artifact-fit-margins-summary"][..],
            &["packaged-artifact-fit-margins"][..],
        ),
        (
            &["packaged-artifact-generation-manifest-checksum-summary"][..],
            &["packaged-artifact-generation-manifest-checksum"][..],
        ),
        (
            &["packaged-artifact-normalized-intermediate-summary"][..],
            &["packaged-artifact-normalized-intermediate"][..],
        ),
        (
            &["packaged-artifact-phase2-corpus-alignment-summary"][..],
            &["packaged-artifact-phase2-corpus-alignment"][..],
        ),
        (
            &["packaged-artifact-target-threshold-state-summary"][..],
            &["packaged-artifact-target-threshold-state"][..],
        ),
    ] {
        let rendered = render_cli(summary_args)
            .unwrap_or_else(|error| panic!("{summary_args:?} should render: {error}"));
        assert_eq!(
            rendered,
            pleiades_validate::render_cli(summary_args).unwrap_or_else(|error| {
                panic!("validation command {summary_args:?} should render: {error}")
            })
        );
        assert_eq!(
            render_cli(alias_args)
                .unwrap_or_else(|error| panic!("{alias_args:?} should render: {error}")),
            rendered,
            "CLI alias should stay aligned with the summary command"
        );
        assert_eq!(
            render_cli(&[summary_args[0], "extra"]).unwrap_err(),
            format!("{} does not accept extra arguments", summary_args[0])
        );
    }
}

#[test]
fn release_house_validation_summary_and_alias_render_directly_from_the_cli() {
    let release_house_validation_summary = render_cli(&["release-house-validation-summary"])
        .expect("release house validation summary should render");
    assert_eq!(
        release_house_validation_summary,
        pleiades_validate::render_cli(&["release-house-validation-summary"])
            .expect("validation front end should render the release house validation summary")
    );
    assert_eq!(
        render_cli(&["release-house-validation"])
            .expect("release house validation alias should render"),
        release_house_validation_summary
    );
    assert_eq!(
        render_cli(&["release-house-validation", "extra"])
            .expect_err("release house validation alias should reject extra arguments"),
        "release-house-validation does not accept extra arguments"
    );
}

#[test]
fn chart_help_text_spells_out_the_shared_request_policy() {
    let help = render_chart(&["--help"]).expect("chart help should render");
    let request_policy = pleiades_validate::validated_request_policy_summary_for_report();
    assert!(help.contains(&shared_request_policy_help_block()));
    assert!(help.contains(&format!("Request policy: {request_policy}")));
    assert!(help.contains(&format!("Request semantics summary: {request_policy}")));
    assert!(help.contains("Request semantics summary:"));
    assert!(help.contains(
        "observer-bearing chart requests stay geocentric and use the observer only for houses"
    ));
    assert!(help.contains(pleiades_validate::current_request_surface_summary().chart_help_clause()));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_source_fit_holdout_sync_summary_and_alias_commands_render_the_summary() {
    let sync = render_cli(&["packaged-artifact-source-fit-holdout-sync-summary"])
        .expect("packaged artifact source-fit and hold-out sync summary should render");
    assert!(sync.contains("Packaged-artifact source-fit and hold-out sync: "));
    assert_eq!(
        render_cli(&["packaged-artifact-source-fit-holdout-sync"])
            .expect("packaged artifact source-fit and hold-out sync alias should render"),
        sync
    );
    assert_eq!(
        sync,
        format!(
            "Packaged-artifact source-fit and hold-out sync: {}",
            pleiades_data::packaged_artifact_source_fit_holdout_sync_summary_details().to_string()
        )
    );
    assert_eq!(
        render_cli(&["packaged-artifact-source-fit-holdout-sync-summary", "extra"]).expect_err(
            "packaged artifact source-fit and hold-out sync summary should reject extra arguments"
        ),
        "packaged-artifact-source-fit-holdout-sync-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["packaged-artifact-source-fit-holdout-sync", "extra"]).expect_err(
            "packaged artifact source-fit and hold-out sync alias should reject extra arguments"
        ),
        "packaged-artifact-source-fit-holdout-sync-summary does not accept extra arguments"
    );
}
