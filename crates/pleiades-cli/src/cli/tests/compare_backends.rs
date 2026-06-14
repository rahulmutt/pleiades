//! Tests for compare-backends and comparison-related commands.

use pleiades_validate::render_cli as validate_render_cli;

use super::super::test_support::help_command_names;
use crate::cli::render_cli;
#[test]
fn shared_command_help_is_kept_in_sync_with_the_validation_binary() {
    let cli_help = render_cli(&["help"]).expect("cli help should render");
    let validation_help = validate_render_cli(&["help"]).expect("validation help should render");

    let mut cli_commands = help_command_names(&cli_help);
    let validation_commands = help_command_names(&validation_help);

    assert!(
        cli_commands.remove("chart"),
        "cli should remain the only binary with the chart command"
    );
    assert!(
        cli_commands.remove("generate-spk-corpus"),
        "cli should remain the only binary with the generate-spk-corpus command"
    );
    assert_eq!(cli_commands, validation_commands);
}

#[test]
fn compare_backends_command_renders_the_comparison_report() {
    let rendered = render_cli(&["compare-backends"]).expect("compare-backends should render");
    assert!(rendered.contains("Comparison report"));
    assert!(rendered.contains("Comparison corpus"));
    assert!(rendered.contains("release-grade guard: Pluto excluded from tolerance evidence"));
    assert!(rendered.contains("epoch labels:"));
    assert!(rendered.contains("Reference backend:"));
    assert!(rendered.contains("Candidate backend:"));
    assert!(rendered.contains("Samples"));
}

#[test]
fn comparison_report_alias_renders_the_comparison_report() {
    let alias = render_cli(&["comparison-report"]).expect("comparison report should render");
    let command = render_cli(&["compare-backends"]).expect("compare-backends should render");

    assert_eq!(alias, command);
    assert_eq!(
        render_cli(&["comparison-report", "extra"]).unwrap_err(),
        "comparison-report does not accept extra arguments"
    );
}

#[test]
fn compare_backends_audit_command_renders_the_comparison_report() {
    let rendered =
        render_cli(&["compare-backends-audit"]).expect("compare-backends-audit should render");
    assert!(rendered.contains("Comparison tolerance audit"));
    assert!(rendered.contains("result: clean"));
    assert!(rendered.contains("within tolerance bodies: 9"));
    assert!(rendered.contains("outside tolerance bodies: 0"));
}

#[test]
fn comparison_audit_summary_command_forwards_to_validate() {
    let summary =
        render_cli(&["comparison-audit-summary"]).expect("comparison audit summary should render");
    assert!(summary.contains("status="));
    assert!(summary.contains("bodies checked="));
    assert_eq!(
        summary,
        render_cli(&["comparison-audit"]).expect("comparison audit alias should render")
    );
}

#[test]
fn target_scope_summary_commands_forward_to_validate() {
    let house_scope = render_cli(&["target-house-scope-summary"])
        .expect("target house scope summary should render");
    assert_eq!(
        house_scope,
        render_cli(&["target-house-scope"]).expect("target house scope alias should render")
    );
    assert_eq!(
        house_scope,
        validate_render_cli(&["target-house-scope-summary"])
            .expect("validation binary should render target house scope summary")
    );

    let ayanamsa_scope = render_cli(&["target-ayanamsa-scope-summary"])
        .expect("target ayanamsa scope summary should render");
    assert_eq!(
        ayanamsa_scope,
        render_cli(&["target-ayanamsa-scope"]).expect("target ayanamsa scope alias should render")
    );
    assert_eq!(
        ayanamsa_scope,
        validate_render_cli(&["target-ayanamsa-scope-summary"])
            .expect("validation binary should render target ayanamsa scope summary")
    );
}

#[test]
fn compare_backends_command_rejects_extra_arguments() {
    let error = render_cli(&["compare-backends", "extra"])
        .expect_err("compare-backends should reject extra arguments");
    assert_eq!(error, "compare-backends does not accept extra arguments");
}

#[test]
fn compare_backends_audit_command_rejects_extra_arguments() {
    let error = render_cli(&["compare-backends-audit", "extra"])
        .expect_err("compare-backends-audit should reject extra arguments");
    assert_eq!(
        error,
        "compare-backends-audit does not accept extra arguments"
    );
}
