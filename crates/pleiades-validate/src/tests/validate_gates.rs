//! CLI dispatch tests for validate-apparent and validate-topocentric goldens gates.

use super::*;

#[test]
fn validate_apparent_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-apparent"])
        .expect("validate-apparent should succeed on committed goldens");
    assert!(
        result.contains("Apparent goldens"),
        "validate-apparent output should contain 'Apparent goldens': {result}"
    );
}

#[test]
fn apparent_gate_alias_matches_validate_apparent() {
    let via_primary = render_cli(&["validate-apparent"]).expect("validate-apparent should succeed");
    let via_alias = render_cli(&["apparent-gate"]).expect("apparent-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_apparent_rejects_extra_args() {
    let error = render_cli(&["validate-apparent", "extra"])
        .expect_err("validate-apparent should reject extra arguments");
    assert!(
        error.contains("validate-apparent does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn apparent_gate_rejects_extra_args() {
    let error = render_cli(&["apparent-gate", "extra"])
        .expect_err("apparent-gate should reject extra arguments");
    assert!(
        error.contains("validate-apparent does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn validate_topocentric_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-topocentric"])
        .expect("validate-topocentric should succeed on committed goldens");
    assert!(
        result.contains("Topocentric goldens"),
        "validate-topocentric output should contain 'Topocentric goldens': {result}"
    );
}

#[test]
fn topocentric_gate_alias_matches_validate_topocentric() {
    let via_primary =
        render_cli(&["validate-topocentric"]).expect("validate-topocentric should succeed");
    let via_alias =
        render_cli(&["topocentric-gate"]).expect("topocentric-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_topocentric_rejects_extra_args() {
    let error = render_cli(&["validate-topocentric", "extra"])
        .expect_err("validate-topocentric should reject extra arguments");
    assert!(
        error.contains("validate-topocentric does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn topocentric_gate_rejects_extra_args() {
    let error = render_cli(&["topocentric-gate", "extra"])
        .expect_err("topocentric-gate should reject extra arguments");
    assert!(
        error.contains("validate-topocentric does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn help_text_mentions_validate_topocentric_and_validate_apparent() {
    let help = render_cli(&["help"]).expect("help command should render");
    assert!(
        help.contains("validate-topocentric"),
        "help text should mention validate-topocentric"
    );
    assert!(
        help.contains("topocentric-gate"),
        "help text should mention topocentric-gate alias"
    );
    assert!(
        help.contains("validate-apparent"),
        "help text should mention validate-apparent"
    );
    assert!(
        help.contains("apparent-gate"),
        "help text should mention apparent-gate alias"
    );
}
